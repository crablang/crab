# Cloning

Now that we've got some basic code set up, we'll need a way to clone the `Arc`.

Basically, we need to:

1. Increment the atomic reference count
2. Construct a new instance of the `Arc` from the inner pointer

First, we need to get access to the `ArcInner`:

<!-- ignore: simplified code -->
```rust,ignore
let inner = unsafe { self.ptr.as_ref() };
```

We can update the atomic reference count as follows:

<!-- ignore: simplified code -->
```rust,ignore
let old_rc = inner.rc.fetch_add(1, Ordering::???);
```

But what ordering should we use here? We don't really have any code that will
need atomic synchronization when cloning, as we do not modify the internal value
while cloning. Thus, we can use a Relaxed ordering here, which implies no
happens-before relationship but is atomic. When `Drop`ping the Arc, however,
we'll need to atomically synchronize when decrementing the reference count. This
is described more in [the section on the `Drop` implementation for
`Arc`](arc-drop.md). For more information on atomic relationships and Relaxed
ordering, see [the section on atomics](../atomics.md).

Thus, the code becomes this:

<!-- ignore: simplified code -->
```rust,ignore
let old_rc = inner.rc.fetch_add(1, Ordering::Relaxed);
```

We'll need to add another import to use `Ordering`:

```rust
use std::sync::atomic::Ordering;
```

However, we have one problem with this implementation right now. What if someone
decides to `mem::forget` a bunch of Arcs? The code we have written so far (and
will write) assumes that the reference count accurately portrays how many Arcs
are in memory, but with `mem::forget` this is false. Thus, when more and more
Arcs are cloned from this one without them being `Drop`ped and the reference
count being decremented, we can overflow! This will cause use-after-free which
is **INCREDIBLY BAD!**

To handle this, we need to check that the reference count does not go over some
arbitrary value (below `usize::MAX`, as we're storing the reference count as an
`AtomicUsize`), and do *something*.

The standard library's implementation decides to just abort the program (as it
is an incredibly unlikely case in normal code and if it happens, the program is
probably incredibly degenerate) if the reference count reaches `isize::MAX`
(about half of `usize::MAX`) on any thread, on the assumption that there are
probably not about 2 billion threads (or about **9 quintillion** on some 64-bit
machines) incrementing the reference count at once. This is what we'll do.

It's pretty simple to implement this behavior:

<!-- ignore: simplified code -->
```rust,ignore
if old_rc >= isize::MAX as usize {
    std::process::abort();
}
```

Then, we need to return a new instance of the `Arc`:

<!-- ignore: simplified code -->
```rust,ignore
Self {
    ptr: self.ptr,
    phantom: PhantomData
}
```

Now, let's wrap this all up inside the `Clone` implementation:

<!-- ignore: simplified code -->
```rust,ignore
use std::sync::atomic::Ordering;

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Arc<T> {
        let inner = unsafe { self.ptr.as_ref() };
        // Using a relaxed ordering is alright here as we don't need any atomic
        // synchronization here as we're not modifying or accessing the inner
        // data.
        let old_rc = inner.rc.fetch_add(1, Ordering::Relaxed);

        if old_rc >= isize::MAX as usize {
            std::process::abort();
        }

        Self {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}
```
