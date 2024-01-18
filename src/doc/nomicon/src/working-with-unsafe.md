# Working with Unsafe

Rust generally only gives us the tools to talk about Unsafe Rust in a scoped and
binary manner. Unfortunately, reality is significantly more complicated than
that. For instance, consider the following toy function:

```rust
fn index(idx: usize, arr: &[u8]) -> Option<u8> {
    if idx < arr.len() {
        unsafe {
            Some(*arr.get_unchecked(idx))
        }
    } else {
        None
    }
}
```

This function is safe and correct. We check that the index is in bounds, and if
it is, index into the array in an unchecked manner. We say that such a correct
unsafely implemented function is *sound*, meaning that safe code cannot cause
Undefined Behavior through it (which, remember, is the single fundamental
property of Safe Rust).

But even in such a trivial function, the scope of the unsafe block is
questionable. Consider changing the `<` to a `<=`:

```rust
fn index(idx: usize, arr: &[u8]) -> Option<u8> {
    if idx <= arr.len() {
        unsafe {
            Some(*arr.get_unchecked(idx))
        }
    } else {
        None
    }
}
```

This program is now *unsound*, Safe Rust can cause Undefined Behavior, and yet
*we only modified safe code*. This is the fundamental problem of safety: it's
non-local. The soundness of our unsafe operations necessarily depends on the
state established by otherwise "safe" operations.

Safety is modular in the sense that opting into unsafety doesn't require you
to consider arbitrary other kinds of badness. For instance, doing an unchecked
index into a slice doesn't mean you suddenly need to worry about the slice being
null or containing uninitialized memory. Nothing fundamentally changes. However
safety *isn't* modular in the sense that programs are inherently stateful and
your unsafe operations may depend on arbitrary other state.

This non-locality gets much worse when we incorporate actual persistent state.
Consider a simple implementation of `Vec`:

```rust
use std::ptr;

// Note: This definition is naive. See the chapter on implementing Vec.
pub struct Vec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

// Note this implementation does not correctly handle zero-sized types.
// See the chapter on implementing Vec.
impl<T> Vec<T> {
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            // not important for this example
            self.reallocate();
        }
        unsafe {
            ptr::write(self.ptr.add(self.len), elem);
            self.len += 1;
        }
    }
    # fn reallocate(&mut self) { }
}

# fn main() {}
```

This code is simple enough to reasonably audit and informally verify. Now consider
adding the following method:

<!-- ignore: simplified code -->
```rust,ignore
fn make_room(&mut self) {
    // grow the capacity
    self.cap += 1;
}
```

This code is 100% Safe Rust but it is also completely unsound. Changing the
capacity violates the invariants of Vec (that `cap` reflects the allocated space
in the Vec). This is not something the rest of Vec can guard against. It *has*
to trust the capacity field because there's no way to verify it.

Because it relies on invariants of a struct field, this `unsafe` code
does more than pollute a whole function: it pollutes a whole *module*.
Generally, the only bullet-proof way to limit the scope of unsafe code is at the
module boundary with privacy.

However this works *perfectly*. The existence of `make_room` is *not* a
problem for the soundness of Vec because we didn't mark it as public. Only the
module that defines this function can call it. Also, `make_room` directly
accesses the private fields of Vec, so it can only be written in the same module
as Vec.

It is therefore possible for us to write a completely safe abstraction that
relies on complex invariants. This is *critical* to the relationship between
Safe Rust and Unsafe Rust.

We have already seen that Unsafe code must trust *some* Safe code, but shouldn't
trust *generic* Safe code. Privacy is important to unsafe code for similar reasons:
it prevents us from having to trust all the safe code in the universe from messing
with our trusted state.

Safety lives!
