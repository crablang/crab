# How Safe and Unsafe Interact

What's the relationship between Safe Rust and Unsafe Rust? How do they
interact?

The separation between Safe Rust and Unsafe Rust is controlled with the
`unsafe` keyword, which acts as an interface from one to the other. This is
why we can say Safe Rust is a safe language: all the unsafe parts are kept
exclusively behind the `unsafe` boundary. If you wish, you can even toss
`#![forbid(unsafe_code)]` into your code base to statically guarantee that
you're only writing Safe Rust.

The `unsafe` keyword has two uses: to declare the existence of contracts the
compiler can't check, and to declare that a programmer has checked that these
contracts have been upheld.

You can use `unsafe` to indicate the existence of unchecked contracts on
_functions_ and _trait declarations_. On functions, `unsafe` means that
users of the function must check that function's documentation to ensure
they are using it in a way that maintains the contracts the function
requires. On trait declarations, `unsafe` means that implementors of the
trait must check the trait documentation to ensure their implementation
maintains the contracts the trait requires.

You can use `unsafe` on a block to declare that all unsafe actions performed
within are verified to uphold the contracts of those operations. For instance,
the index passed to [`slice::get_unchecked`][get_unchecked] is in-bounds.

You can use `unsafe` on a trait implementation to declare that the implementation
upholds the trait's contract. For instance, that a type implementing [`Send`] is
really safe to move to another thread.

The standard library has a number of unsafe functions, including:

* [`slice::get_unchecked`][get_unchecked], which performs unchecked indexing,
  allowing memory safety to be freely violated.
* [`mem::transmute`][transmute] reinterprets some value as having a given type,
  bypassing type safety in arbitrary ways (see [conversions] for details).
* Every raw pointer to a sized type has an [`offset`][ptr_offset] method that
  invokes Undefined Behavior if the passed offset is not ["in bounds"][ptr_offset].
* All FFI (Foreign Function Interface) functions are `unsafe` to call because the
  other language can do arbitrary operations that the Rust compiler can't check.

As of Rust 1.29.2 the standard library defines the following unsafe traits
(there are others, but they are not stabilized yet and some of them may never
be):

* [`Send`] is a marker trait (a trait with no API) that promises implementors
  are safe to send (move) to another thread.
* [`Sync`] is a marker trait that promises threads can safely share implementors
  through a shared reference.
* [`GlobalAlloc`] allows customizing the memory allocator of the whole program.

Much of the Rust standard library also uses Unsafe Rust internally. These
implementations have generally been rigorously manually checked, so the Safe Rust
interfaces built on top of these implementations can be assumed to be safe.

The need for all of this separation boils down a single fundamental property
of Safe Rust, the *soundness property*:

**No matter what, Safe Rust can't cause Undefined Behavior.**

The design of the safe/unsafe split means that there is an asymmetric trust
relationship between Safe and Unsafe Rust. Safe Rust inherently has to
trust that any Unsafe Rust it touches has been written correctly.
On the other hand, Unsafe Rust cannot trust Safe Rust without care.

As an example, Rust has the [`PartialOrd`] and [`Ord`] traits to differentiate
between types which can "just" be compared, and those that provide a "total"
ordering (which basically means that comparison behaves reasonably).

[`BTreeMap`] doesn't really make sense for partially-ordered types, and so it
requires that its keys implement `Ord`. However, `BTreeMap` has Unsafe Rust code
inside of its implementation. Because it would be unacceptable for a sloppy `Ord`
implementation (which is Safe to write) to cause Undefined Behavior, the Unsafe
code in BTreeMap must be written to be robust against `Ord` implementations which
aren't actually total — even though that's the whole point of requiring `Ord`.

The Unsafe Rust code just can't trust the Safe Rust code to be written correctly.
That said, `BTreeMap` will still behave completely erratically if you feed in
values that don't have a total ordering. It just won't ever cause Undefined
Behavior.

One may wonder, if `BTreeMap` cannot trust `Ord` because it's Safe, why can it
trust *any* Safe code? For instance `BTreeMap` relies on integers and slices to
be implemented correctly. Those are safe too, right?

The difference is one of scope. When `BTreeMap` relies on integers and slices,
it's relying on one very specific implementation. This is a measured risk that
can be weighed against the benefit. In this case there's basically zero risk;
if integers and slices are broken, *everyone* is broken. Also, they're maintained
by the same people who maintain `BTreeMap`, so it's easy to keep tabs on them.

On the other hand, `BTreeMap`'s key type is generic. Trusting its `Ord` implementation
means trusting every `Ord` implementation in the past, present, and future.
Here the risk is high: someone somewhere is going to make a mistake and mess up
their `Ord` implementation, or even just straight up lie about providing a total
ordering because "it seems to work". When that happens, `BTreeMap` needs to be
prepared.

The same logic applies to trusting a closure that's passed to you to behave
correctly.

This problem of unbounded generic trust is the problem that `unsafe` traits
exist to resolve. The `BTreeMap` type could theoretically require that keys
implement a new trait called `UnsafeOrd`, rather than `Ord`, that might look
like this:

```rust
use std::cmp::Ordering;

unsafe trait UnsafeOrd {
    fn cmp(&self, other: &Self) -> Ordering;
}
```

Then, a type would use `unsafe` to implement `UnsafeOrd`, indicating that
they've ensured their implementation maintains whatever contracts the
trait expects. In this situation, the Unsafe Rust in the internals of
`BTreeMap` would be justified in trusting that the key type's `UnsafeOrd`
implementation is correct. If it isn't, it's the fault of the unsafe trait
implementation, which is consistent with Rust's safety guarantees.

The decision of whether to mark a trait `unsafe` is an API design choice. A
safe trait is easier to implement, but any unsafe code that relies on it must
defend against incorrect behavior. Marking a trait `unsafe` shifts this
responsibility to the implementor. Rust has traditionally avoided marking
traits `unsafe` because it makes Unsafe Rust pervasive, which isn't desirable.

`Send` and `Sync` are marked unsafe because thread safety is a *fundamental
property* that unsafe code can't possibly hope to defend against in the way it
could defend against a buggy `Ord` implementation. Similarly, `GlobalAllocator`
is keeping accounts of all the memory in the program and other things like
`Box` or `Vec` build on top of it. If it does something weird (giving the same
chunk of memory to another request when it is still in use), there's no chance
to detect that and do anything about it.

The decision of whether to mark your own traits `unsafe` depends on the same
sort of consideration. If `unsafe` code can't reasonably expect to defend
against a broken implementation of the trait, then marking the trait `unsafe` is
a reasonable choice.

As an aside, while `Send` and `Sync` are `unsafe` traits, they are *also*
automatically implemented for types when such derivations are provably safe
to do. `Send` is automatically derived for all types composed only of values
whose types also implement `Send`. `Sync` is automatically derived for all
types composed only of values whose types also implement `Sync`. This minimizes
the pervasive unsafety of making these two traits `unsafe`. And not many people
are going to *implement* memory allocators (or use them directly, for that
matter).

This is the balance between Safe and Unsafe Rust. The separation is designed to
make using Safe Rust as ergonomic as possible, but requires extra effort and
care when writing Unsafe Rust. The rest of this book is largely a discussion
of the sort of care that must be taken, and what contracts Unsafe Rust must uphold.

[`Send`]: ../std/marker/trait.Send.html
[`Sync`]: ../std/marker/trait.Sync.html
[`GlobalAlloc`]: ../std/alloc/trait.GlobalAlloc.html
[conversions]: conversions.html
[ptr_offset]: ../std/primitive.pointer.html#method.offset
[get_unchecked]: ../std/primitive.slice.html#method.get_unchecked
[transmute]: ../std/mem/fn.transmute.html
[`PartialOrd`]: ../std/cmp/trait.PartialOrd.html
[`Ord`]: ../std/cmp/trait.Ord.html
[`BTreeMap`]: ../std/collections/struct.BTreeMap.html
