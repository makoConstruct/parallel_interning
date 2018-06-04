# Concurrent Interner

An [interner](https://en.wikipedia.org/wiki/String_interning) that can be efficiently shared between multiple threads. Built on top of [CHashMap](https://crates.io/crates/chashmap).

```rust
let interner: Arc<Interner<String>> = super::Interner::new();
let a:Interned<String> = Interner::get(&interner, "repeat".into());
let b:Interned<String> = Interner::get(&interner, "repeat".into());
let c:Interned<String> = Interner::get(&interner, "unique".into());

assert!(a == b);
assert!(a != c);
```

## How it works

An `Interned<Thing>` is just an Arc pointing to a struct (an `InternedSlot<Thing>`) that has a copy of the interned thing, and an arc pointing to the `Interner` that the `Interned` was made with. The `Interner` is really just a `CHashMap` mapping from interned things to Weak refs of the `Interned`.

When you intern something, the `Interner` will check to see if it's already there.
    
* If it's there, it will try to upgrade the Weak ref of the corresponding `InternedSlot<Thing>` into an `Interned<Thing>` and give you that.
	* If it can't upgrade, that means there are no longer any Arcs pointing at that `InternedSlot` any more, so it must be on the verge of being dropped. When an InternedSlot is dropped, it erases its corresponding Weak from the `Interner`. Anticipating this, we (repeatedly, if necessary) relinquish our read lock and take it again until either the thing has been found to be erased, or until a new Weak takes its place that *can* upgrade into an Arc.
		* If it finds the thing has been erased from the map, it inserts it and gives you the Arc.
		* If it finds a new Weak that successfully upgrades, well that's good, it gives you the Arc that results from the upgrade.
		* It will be extremely rare that this stuff ever happens, but I wanted to make absolutely sure that the code is safe.
	* If it can upgrade, good, here is your `Interned<Thing>`. We never needed to take a write lock.
* If it's not there, it inserts it and gives you the resultant Interned<Thing>.

