# Base Code

Now that we've decided the layout for our implementation of `Arc`, let's create
some basic code.

## Constructing the Arc

We'll first need a way to construct an `Arc<T>`.

This is pretty simple, as we just need to box the `ArcInner<T>` and get a
`NonNull<T>` pointer to it.

<!-- ignore: simplified code -->
```rust,ignore
impl<T> Arc<T> {
    pub fn new(data: T) -> Arc<T> {
        // We start the reference count at 1, as that first reference is the
        // current pointer.
        let boxed = Box::new(ArcInner {
            rc: AtomicUsize::new(1),
            data,
        });
        Arc {
            // It is okay to call `.unwrap()` here as we get a pointer from
            // `Box::into_raw` which is guaranteed to not be null.
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            phantom: PhantomData,
        }
    }
}
```

## Send and Sync

Since we're building a concurrency primitive, we'll need to be able to send it
across threads. Thus, we can implement the `Send` and `Sync` marker traits. For
more information on these, see [the section on `Send` and
`Sync`](../send-and-sync.md).

This is okay because:
* You can only get a mutable reference to the value inside an `Arc` if and only
  if it is the only `Arc` referencing that data (which only happens in `Drop`)
* We use atomics for the shared mutable reference counting

<!-- ignore: simplified code -->
```rust,ignore
unsafe impl<T: Sync + Send> Send for Arc<T> {}
unsafe impl<T: Sync + Send> Sync for Arc<T> {}
```

We need to have the bound `T: Sync + Send` because if we did not provide those
bounds, it would be possible to share values that are thread-unsafe across a
thread boundary via an `Arc`, which could possibly cause data races or
unsoundness.

For example, if those bounds were not present, `Arc<Rc<u32>>` would be `Sync` or
`Send`, meaning that you could clone the `Rc` out of the `Arc` to send it across
a thread (without creating an entirely new `Rc`), which would create data races
as `Rc` is not thread-safe.

## Getting the `ArcInner`

To dereference the `NonNull<T>` pointer into a `&T`, we can call
`NonNull::as_ref`. This is unsafe, unlike the typical `as_ref` function, so we
must call it like this:

<!-- ignore: simplified code -->
```rust,ignore
unsafe { self.ptr.as_ref() }
```

We'll be using this snippet a few times in this code (usually with an associated
`let` binding).

This unsafety is okay because while this `Arc` is alive, we're guaranteed that
the inner pointer is valid.

## Deref

Alright. Now we can make `Arc`s (and soon will be able to clone and destroy them correctly), but how do we get
to the data inside?

What we need now is an implementation of `Deref`.

We'll need to import the trait:

<!-- ignore: simplified code -->
```rust,ignore
use std::ops::Deref;
```

And here's the implementation:

<!-- ignore: simplified code -->
```rust,ignore
impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let inner = unsafe { self.ptr.as_ref() };
        &inner.data
    }
}
```

Pretty simple, eh? This simply dereferences the `NonNull` pointer to the
`ArcInner<T>`, then gets a reference to the data inside.

## Code

Here's all the code from this section:

<!-- ignore: simplified code -->
```rust,ignore
use std::ops::Deref;

impl<T> Arc<T> {
    pub fn new(data: T) -> Arc<T> {
        // We start the reference count at 1, as that first reference is the
        // current pointer.
        let boxed = Box::new(ArcInner {
            rc: AtomicUsize::new(1),
            data,
        });
        Arc {
            // It is okay to call `.unwrap()` here as we get a pointer from
            // `Box::into_raw` which is guaranteed to not be null.
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            phantom: PhantomData,
        }
    }
}

unsafe impl<T: Sync + Send> Send for Arc<T> {}
unsafe impl<T: Sync + Send> Sync for Arc<T> {}


impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let inner = unsafe { self.ptr.as_ref() };
        &inner.data
    }
}
```
