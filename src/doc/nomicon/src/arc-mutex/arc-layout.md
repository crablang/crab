# Layout

Let's start by making the layout for our implementation of `Arc`.

An `Arc<T>` provides thread-safe shared ownership of a value of type `T`,
allocated in the heap. Sharing implies immutability in Rust, so we don't need to
design anything that manages access to that value, right? Although interior
mutability types like Mutex allow Arc's users to create shared mutability, Arc
itself doesn't need to concern itself with these issues.

However there _is_ one place where Arc needs to concern itself with mutation:
destruction. When all the owners of the Arc go away, we need to be able to
`drop` its contents and free its allocation. So we need a way for an owner to
know if it's the _last_ owner, and the simplest way to do that is with a count
of the owners -- Reference Counting.

Unfortunately, this reference count is inherently shared mutable state, so Arc
_does_ need to think about synchronization. We _could_ use a Mutex for this, but
that's overkill. Instead, we'll use atomics. And since everyone already needs a
pointer to the T's allocation, we might as well put the reference count in that
same allocation.

Naively, it would look something like this:

```rust
use std::sync::atomic;

pub struct Arc<T> {
    ptr: *mut ArcInner<T>,
}

pub struct ArcInner<T> {
    rc: atomic::AtomicUsize,
    data: T,
}
```

This would compile, however it would be incorrect. First of all, the compiler
will give us too strict variance. For example, an `Arc<&'static str>` couldn't
be used where an `Arc<&'a str>` was expected. More importantly, it will give
incorrect ownership information to the drop checker, as it will assume we don't
own any values of type `T`. As this is a structure providing shared ownership of
a value, at some point there will be an instance of this structure that entirely
owns its data. See [the chapter on ownership and lifetimes](../ownership.md) for
all the details on variance and drop check.

To fix the first problem, we can use `NonNull<T>`. Note that `NonNull<T>` is a
wrapper around a raw pointer that declares that:

* We are covariant over `T`
* Our pointer is never null

To fix the second problem, we can include a `PhantomData` marker containing an
`ArcInner<T>`. This will tell the drop checker that we have some notion of
ownership of a value of `ArcInner<T>` (which itself contains some `T`).

With these changes we get our final structure:

```rust
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::AtomicUsize;

pub struct Arc<T> {
    ptr: NonNull<ArcInner<T>>,
    phantom: PhantomData<ArcInner<T>>,
}

pub struct ArcInner<T> {
    rc: AtomicUsize,
    data: T,
}
```
