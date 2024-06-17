# Allocating Memory

Using `NonNull` throws a wrench in an important feature of Vec (and indeed all of
the std collections): creating an empty Vec doesn't actually allocate at all. This
is not the same as allocating a zero-sized memory block, which is not allowed by
the global allocator (it results in undefined behavior!). So if we can't allocate,
but also can't put a null pointer in `ptr`, what do we do in `Vec::new`? Well, we
just put some other garbage in there!

This is perfectly fine because we already have `cap == 0` as our sentinel for no
allocation. We don't even need to handle it specially in almost any code because
we usually need to check if `cap > len` or `len > 0` anyway. The recommended
Rust value to put here is `mem::align_of::<T>()`. `NonNull` provides a convenience
for this: `NonNull::dangling()`. There are quite a few places where we'll
want to use `dangling` because there's no real allocation to talk about but
`null` would make the compiler do bad things.

So:

<!-- ignore: explanation code -->
```rust,ignore
use std::mem;

impl<T> Vec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        Vec {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }
}
# fn main() {}
```

I slipped in that assert there because zero-sized types will require some
special handling throughout our code, and I want to defer the issue for now.
Without this assert, some of our early drafts will do some Very Bad Things.

Next we need to figure out what to actually do when we *do* want space. For that,
we use the global allocation functions [`alloc`][alloc], [`realloc`][realloc],
and [`dealloc`][dealloc] which are available in stable Rust in
[`std::alloc`][std_alloc]. These functions are expected to become deprecated in
favor of the methods of [`std::alloc::Global`][Global] after this type is stabilized.

We'll also need a way to handle out-of-memory (OOM) conditions. The standard
library provides a function [`alloc::handle_alloc_error`][handle_alloc_error],
which will abort the program in a platform-specific manner.
The reason we abort and don't panic is because unwinding can cause allocations
to happen, and that seems like a bad thing to do when your allocator just came
back with "hey I don't have any more memory".

Of course, this is a bit silly since most platforms don't actually run out of
memory in a conventional way. Your operating system will probably kill the
application by another means if you legitimately start using up all the memory.
The most likely way we'll trigger OOM is by just asking for ludicrous quantities
of memory at once (e.g. half the theoretical address space). As such it's
*probably* fine to panic and nothing bad will happen. Still, we're trying to be
like the standard library as much as possible, so we'll just kill the whole
program.

Okay, now we can write growing. Roughly, we want to have this logic:

```text
if cap == 0:
    allocate()
    cap = 1
else:
    reallocate()
    cap *= 2
```

But Rust's only supported allocator API is so low level that we'll need to do a
fair bit of extra work. We also need to guard against some special
conditions that can occur with really large allocations or empty allocations.

In particular, `ptr::offset` will cause us a lot of trouble, because it has
the semantics of LLVM's GEP inbounds instruction. If you're fortunate enough to
not have dealt with this instruction, here's the basic story with GEP: alias
analysis, alias analysis, alias analysis. It's super important to an optimizing
compiler to be able to reason about data dependencies and aliasing.

As a simple example, consider the following fragment of code:

<!-- ignore: simplified code -->
```rust,ignore
*x *= 7;
*y *= 3;
```

If the compiler can prove that `x` and `y` point to different locations in
memory, the two operations can in theory be executed in parallel (by e.g.
loading them into different registers and working on them independently).
However the compiler can't do this in general because if x and y point to
the same location in memory, the operations need to be done to the same value,
and they can't just be merged afterwards.

When you use GEP inbounds, you are specifically telling LLVM that the offsets
you're about to do are within the bounds of a single "allocated" entity. The
ultimate payoff being that LLVM can assume that if two pointers are known to
point to two disjoint objects, all the offsets of those pointers are *also*
known to not alias (because you won't just end up in some random place in
memory). LLVM is heavily optimized to work with GEP offsets, and inbounds
offsets are the best of all, so it's important that we use them as much as
possible.

So that's what GEP's about, how can it cause us trouble?

The first problem is that we index into arrays with unsigned integers, but
GEP (and as a consequence `ptr::offset`) takes a signed integer. This means
that half of the seemingly valid indices into an array will overflow GEP and
actually go in the wrong direction! As such we must limit all allocations to
`isize::MAX` elements. This actually means we only need to worry about
byte-sized objects, because e.g. `> isize::MAX` `u16`s will truly exhaust all of
the system's memory. However in order to avoid subtle corner cases where someone
reinterprets some array of `< isize::MAX` objects as bytes, std limits all
allocations to `isize::MAX` bytes.

On all 64-bit targets that Rust currently supports we're artificially limited
to significantly less than all 64 bits of the address space (modern x64
platforms only expose 48-bit addressing), so we can rely on just running out of
memory first. However on 32-bit targets, particularly those with extensions to
use more of the address space (PAE x86 or x32), it's theoretically possible to
successfully allocate more than `isize::MAX` bytes of memory.

However since this is a tutorial, we're not going to be particularly optimal
here, and just unconditionally check, rather than use clever platform-specific
`cfg`s.

The other corner-case we need to worry about is empty allocations. There will
be two kinds of empty allocations we need to worry about: `cap = 0` for all T,
and `cap > 0` for zero-sized types.

These cases are tricky because they come
down to what LLVM means by "allocated". LLVM's notion of an
allocation is significantly more abstract than how we usually use it. Because
LLVM needs to work with different languages' semantics and custom allocators,
it can't really intimately understand allocation. Instead, the main idea behind
allocation is "doesn't overlap with other stuff". That is, heap allocations,
stack allocations, and globals don't randomly overlap. Yep, it's about alias
analysis. As such, Rust can technically play a bit fast and loose with the notion of
an allocation as long as it's *consistent*.

Getting back to the empty allocation case, there are a couple of places where
we want to offset by 0 as a consequence of generic code. The question is then:
is it consistent to do so? For zero-sized types, we have concluded that it is
indeed consistent to do a GEP inbounds offset by an arbitrary number of
elements. This is a runtime no-op because every element takes up no space,
and it's fine to pretend that there's infinite zero-sized types allocated
at `0x01`. No allocator will ever allocate that address, because they won't
allocate `0x00` and they generally allocate to some minimal alignment higher
than a byte. Also generally the whole first page of memory is
protected from being allocated anyway (a whole 4k, on many platforms).

However what about for positive-sized types? That one's a bit trickier. In
principle, you can argue that offsetting by 0 gives LLVM no information: either
there's an element before the address or after it, but it can't know which.
However we've chosen to conservatively assume that it may do bad things. As
such we will guard against this case explicitly.

*Phew*

Ok with all the nonsense out of the way, let's actually allocate some memory:

<!-- ignore: simplified code -->
```rust,ignore
use std::alloc::{self, Layout};

impl<T> Vec<T> {
    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            // This can't overflow since self.cap <= isize::MAX.
            let new_cap = 2 * self.cap;

            // `Layout::array` checks that the number of bytes is <= usize::MAX,
            // but this is redundant since old_layout.size() <= isize::MAX,
            // so the `unwrap` should never fail.
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // If allocation fails, `new_ptr` will be null, in which case we abort.
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }
}
# fn main() {}
```

[Global]: ../../std/alloc/struct.Global.html
[handle_alloc_error]: ../../alloc/alloc/fn.handle_alloc_error.html
[alloc]: ../../alloc/alloc/fn.alloc.html
[realloc]: ../../alloc/alloc/fn.realloc.html
[dealloc]: ../../alloc/alloc/fn.dealloc.html
[std_alloc]: ../../alloc/alloc/index.html
