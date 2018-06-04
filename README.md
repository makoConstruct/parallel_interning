# Concurrent Interner

An [interner](https://en.wikipedia.org/wiki/String_interning) that can be efficiently shared between multiple threads. Built on top of [CHashMap](https://crates.io/crates/chashmap).

```rust
let interner: Arc<Interner<String>> = super::Interner::new();
let a:InternHandle<String> = Interner::get(&interner, "repeat".into());
let b:InternHandle<String> = Interner::get(&interner, "repeat".into());
let c:InternHandle<String> = Interner::get(&interner, "unique".into());

assert!(a == b);
assert!(a != c);
```

## How it works

An `InternHandle` is just an Arc pointing to an object that has a copy of the interned thing, and an arc pointing to the `Interner` the `InternHandle` was made with. The `Interner` is really just a `CHashMap` mapping from interned things to Weak refs of the `InternHandle`.

When you intern something, the `Interner` will check to see if it's already there.
    
* If it's there, it will try to upgrade the Weak ref of the corresponding `InternedSlot<Thing>` into an Arc and give you that.
	* If it can't upgrade, that means there are no longer any Arcs pointing at the `InternedSlot` any more, so it must be on the verge of being dropped. When an InternedSlot is dropped, it erases its corresponding Weak from the `Interner`. Anticipating this, we relinquishe our read lock and take it again until either the thing has been found to be erased, or until a new Weak takes its place that *can* upgrade into an arc.
		* If it finds the thing has been erased, it inserts it and gives you the Arc.
		* If it finds a new Weak that successfully upgrades, well that's good, it gives you the Arc that results from the upgrade.
		* It will be extremely rare that any of this ever happens, but I wanted to make absolutely sure that this code was safe.
	* If it can upgrade, good, here is your `Interned<Thing>`. We never needed to take a write lock.
* If it's not there, it's inserted and it gives you the resultant Interned<Thing>.

