# Dropping

We now need a way to decrease the reference count and drop the data once it is
low enough, otherwise the data will live forever on the heap.

To do this, we can implement `Drop`.

Basically, we need to:

1. Decrement the reference count
2. If there is only one reference remaining to the data, then:
3. Atomically fence the data to prevent reordering of the use and deletion of
   the data
4. Drop the inner data

First, we'll need to get access to the `ArcInner`:

<!-- ignore: simplified code -->
```rust,ignore
let inner = unsafe { self.ptr.as_ref() };
```

Now, we need to decrement the reference count. To streamline our code, we can
also return if the returned value from `fetch_sub` (the value of the reference
count before decrementing it) is not equal to `1` (which happens when we are not
the last reference to the data).

<!-- ignore: simplified code -->
```rust,ignore
if inner.rc.fetch_sub(1, Ordering::Release) != 1 {
    return;
}
```

We then need to create an atomic fence to prevent reordering of the use of the
data and deletion of the data. As described in [the standard library's
implementation of `Arc`][3]:
> This fence is needed to prevent reordering of use of the data and deletion of
> the data. Because it is marked `Release`, the decreasing of the reference
> count synchronizes with this `Acquire` fence. This means that use of the data
> happens before decreasing the reference count, which happens before this
> fence, which happens before the deletion of the data.
>
> As explained in the [Boost documentation][1],
>
> > It is important to enforce any possible access to the object in one
> > thread (through an existing reference) to *happen before* deleting
> > the object in a different thread. This is achieved by a "release"
> > operation after dropping a reference (any access to the object
> > through this reference must obviously happened before), and an
> > "acquire" operation before deleting the object.
>
> In particular, while the contents of an Arc are usually immutable, it's
> possible to have interior writes to something like a Mutex<T>. Since a Mutex
> is not acquired when it is deleted, we can't rely on its synchronization logic
> to make writes in thread A visible to a destructor running in thread B.
>
> Also note that the Acquire fence here could probably be replaced with an
> Acquire load, which could improve performance in highly-contended situations.
> See [2].
>
> [1]: https://www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html
> [2]: https://github.com/rust-lang/rust/pull/41714
[3]: https://github.com/rust-lang/rust/blob/e1884a8e3c3e813aada8254edfa120e85bf5ffca/library/alloc/src/sync.rs#L1440-L1467

To do this, we do the following:

```rust
# use std::sync::atomic::Ordering;
use std::sync::atomic;
atomic::fence(Ordering::Acquire);
```

Finally, we can drop the data itself. We use `Box::from_raw` to drop the boxed
`ArcInner<T>` and its data. This takes a `*mut T` and not a `NonNull<T>`, so we
must convert using `NonNull::as_ptr`.

<!-- ignore: simplified code -->
```rust,ignore
unsafe { Box::from_raw(self.ptr.as_ptr()); }
```

This is safe as we know we have the last pointer to the `ArcInner` and that its
pointer is valid.

Now, let's wrap this all up inside the `Drop` implementation:

<!-- ignore: simplified code -->
```rust,ignore
impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };
        if inner.rc.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        // This fence is needed to prevent reordering of the use and deletion
        // of the data.
        atomic::fence(Ordering::Acquire);
        // This is safe as we know we have the last pointer to the `ArcInner`
        // and that its pointer is valid.
        unsafe { Box::from_raw(self.ptr.as_ptr()); }
    }
}
```
