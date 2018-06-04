
extern crate chashmap;

use std::sync::{Weak, Arc};
use std::cell::UnsafeCell;
use std::cmp::PartialEq;
use std::hash::Hash;
use std::ops::Deref;
use chashmap::CHashMap;


#[derive(Debug, Clone)]
pub struct InternedSlot<T> where T: PartialEq + Hash + Clone {
	v: T,
	owner: Arc<Interner<T>>,
}

#[derive(Debug, Clone)]
pub struct Interned<T> where T: PartialEq + Hash + Clone {
	v: Arc<InternedSlot<T>>,
}

impl<T> PartialEq for Interned<T> where T: PartialEq + Hash + Clone {
	fn eq(&self, other: &Self) -> bool {
		self.v.as_ref() as *const InternedSlot<T> == other.v.as_ref() as *const InternedSlot<T>
	}
}

#[derive(Debug, Clone)]
pub struct Interner<T> where T: PartialEq + Hash + Clone {
	contents: CHashMap<T, Weak<InternedSlot<T>>>,
}

impl<T> Drop for InternedSlot<T> where T:PartialEq + Hash + Clone {
	fn drop(&mut self){ //erase self from interner
		self.owner.contents.remove(&self.v);
	}
}

impl<T> Deref for Interned<T> where T: PartialEq + Hash + Clone {
	type Target = T;
	fn deref(&self) -> &Self::Target { &self.v.v }
}

impl<T> Interner<T> where T:PartialEq + Hash + Clone {
	
	pub fn new()-> Arc<Self> { Arc::new(Interner{ contents: CHashMap::default(), }) }
	
	pub fn get(this: &Arc<Self>, v: T)-> Interned<T> {
		let upsertion = ||-> Arc<InternedSlot<T>> {
			loop {
				let ret = UnsafeCell::new(None); //we use UnsafeCell because we want to write to this from either of the two closures, but we can guarantee that one and only one of the closures that alias it will be called, and that they certainly wont be called concurrently
				this.contents.upsert(
					v.clone(),
					||{
						let rar = Arc::new(InternedSlot{ v:v.clone(), owner:this.clone(), });
						let r = Arc::downgrade(&rar);
						unsafe{ *ret.get() = Some(rar) };
						r
					},
					|armr|{
						if let Some(ar) = armr.upgrade() {
							unsafe{ *ret.get() = Some(ar) };
						}
					},
				);
				if let Some(ar) = unsafe{ ret.into_inner() } {
					return ar;
				}
			}
			
			// loop{
			// 	//this scope was taken from the CHashMap source code's upsert function and altered. I needed to do things with the flow control that the API didn't quite support. I could get around needing to return a value from one or the other closure, but not the looping stuff.
			// 	let lock = self.contents.table.read();
			// 	{
			// 		// Lookup the key or a free bucket in the inner table.
			// 		let mut bucket = lock.lookup_or_free(v);

			// 		match *bucket {
			// 			// The bucket had KV pair!
			// 			chashmap::Bucket::Contains(_, ref val) => {
			// 				if let Some(ar) = val.upgrade() {
			// 					return ar;
			// 				} //else just cycle, release and retake the lock, check again, the situation should have changed. If the system is working, a weak ref will soon be removed by the Interned's destructor. If we find another weak ref there, we should assume it's a different weak ref that has been created and orphaned again, and that it too will be removed.
			// 			},
			// 			// The bucket was empty, simply insert.
			// 			ref mut x => {
			// 				let ar = Arc::new(Interned{ v: v.clone(), });
			// 				*x = chashmap::Bucket::Contains(v.clone(), Arc::downgrade(ar));
			// 				self.expand(lock);
			// 				return ar;
			// 			}
			// 		}
			// 	}
				// Expand the table (this will only happen if the function haven't returned yet).
			// }
		};
		
		Interned{ v:
			if let Some(rg) = this.contents.get(&v) {
				if let Some(ar) = rg.upgrade() {
					ar
				}else{
					upsertion()
				}
			}else{
				upsertion()
			}
		}
	}
}


#[cfg(test)]
mod test {
	use super::Interner;
	use std::sync::Arc;
	
	#[test]
	fn main_test(){
		let interner = Interner::new();
		let a = Interner::get(&interner, 1u32);
		let b = Interner::get(&interner, 1u32);
		let c = Interner::get(&interner, 3u32);
		{
			let _d = Interner::get(&interner, 4u32);
			assert!(interner.contents.len() == 3);
		}
		assert!(interner.contents.len() == 2);
		assert!(a == b);
		assert!(a != c);
	}
	
	#[test]
	fn with_strings(){
		let interner: Arc<Interner<String>> = Interner::new();
		let a = Interner::get(&interner, "heh".into());
		let b = Interner::get(&interner, "hah".into());
		let c = Interner::get(&interner, "heh".into());
		assert!(interner.contents.len() == 2);
		assert!(a != b);
		assert!(a == c);
	}
	
	// #[test]
	// fn concurrency(){
	// 	let interner: Arc<Interner<usize>> = Interner::new();
	// 	let n_threads = 90;
	// }
}