# Collections

Eventually you'll want to use dynamic data structures (AKA collections) in your
program. `std` provides a set of common collections: [`Vec`], [`String`],
[`HashMap`], etc. All the collections implemented in `std` use a global dynamic
memory allocator (AKA the heap).

[`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
[`String`]: https://doc.rust-lang.org/std/string/struct.String.html
[`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html

As `core` is, by definition, free of memory allocations these implementations
are not available there, but they can be found in the `alloc` crate
that's shipped with the compiler.

If you need collections, a heap allocated implementation is not your only
option. You can also use *fixed capacity* collections; one such implementation
can be found in the [`heapless`] crate.

[`heapless`]: https://crates.io/crates/heapless

In this section, we'll explore and compare these two implementations.

## Using `alloc`

The `alloc` crate is shipped with the standard Rust distribution. To import the
crate you can directly `use` it *without* declaring it as a dependency in your
`Cargo.toml` file.

``` rust,ignore
#![feature(alloc)]

extern crate alloc;

use alloc::vec::Vec;
```

To be able to use any collection you'll first need use the `global_allocator`
attribute to declare the global allocator your program will use. It's required
that the allocator you select implements the [`GlobalAlloc`] trait.

[`GlobalAlloc`]: https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html

For completeness and to keep this section as self-contained as possible we'll
implement a simple bump pointer allocator and use that as the global allocator.
However, we *strongly* suggest you use a battle tested allocator from crates.io
in your program instead of this allocator.

``` rust,ignore
// Bump pointer allocator implementation

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;

use cortex_m::interrupt;

// Bump pointer allocator for *single* core systems
struct BumpPointerAlloc {
    head: UnsafeCell<usize>,
    end: usize,
}

unsafe impl Sync for BumpPointerAlloc {}

unsafe impl GlobalAlloc for BumpPointerAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // `interrupt::free` is a critical section that makes our allocator safe
        // to use from within interrupts
        interrupt::free(|_| {
            let head = self.head.get();
            let size = layout.size();
            let align = layout.align();
            let align_mask = !(align - 1);

            // move start up to the next alignment boundary
            let start = (*head + align - 1) & align_mask;

            if start + size > self.end {
                // a null pointer signal an Out Of Memory condition
                ptr::null_mut()
            } else {
                *head = start + size;
                start as *mut u8
            }
        })
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // this allocator never deallocates memory
    }
}

// Declaration of the global memory allocator
// NOTE the user must ensure that the memory region `[0x2000_0100, 0x2000_0200]`
// is not used by other parts of the program
#[global_allocator]
static HEAP: BumpPointerAlloc = BumpPointerAlloc {
    head: UnsafeCell::new(0x2000_0100),
    end: 0x2000_0200,
};
```

Apart from selecting a global allocator the user will also have to define how
Out Of Memory (OOM) errors are handled using the *unstable*
`alloc_error_handler` attribute.

``` rust,ignore
#![feature(alloc_error_handler)]

use cortex_m::asm;

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    asm::bkpt();

    loop {}
}
```

Once all that is in place, the user can finally use the collections in `alloc`.

```rust,ignore
#[entry]
fn main() -> ! {
    let mut xs = Vec::new();

    xs.push(42);
    assert!(xs.pop(), Some(42));

    loop {
        // ..
    }
}
```

If you have used the collections in the `std` crate then these will be familiar
as they are exact same implementation.

## Using `heapless`

`heapless` requires no setup as its collections don't depend on a global memory
allocator. Just `use` its collections and proceed to instantiate them:

```rust,ignore
// heapless version: v0.4.x
use heapless::Vec;
use heapless::consts::*;

#[entry]
fn main() -> ! {
    let mut xs: Vec<_, U8> = Vec::new();

    xs.push(42).unwrap();
    assert_eq!(xs.pop(), Some(42));
    loop {}
}
```

You'll note two differences between these collections and the ones in `alloc`.

First, you have to declare upfront the capacity of the collection. `heapless`
collections never reallocate and have fixed capacities; this capacity is part of
the type signature of the collection. In this case we have declared that `xs`
has a capacity of 8 elements that is the vector can, at most, hold 8 elements.
This is indicated by the `U8` (see [`typenum`]) in the type signature.

[`typenum`]: https://crates.io/crates/typenum

Second, the `push` method, and many other methods, return a `Result`. Since the
`heapless` collections have fixed capacity all operations that insert elements
into the collection can potentially fail. The API reflects this problem by
returning a `Result` indicating whether the operation succeeded or not. In
contrast, `alloc` collections will reallocate themselves on the heap to increase
their capacity.

As of version v0.4.x all `heapless` collections store all their elements inline.
This means that an operation like `let x = heapless::Vec::new();` will allocate
the collection on the stack, but it's also possible to allocate the collection
on a `static` variable, or even on the heap (`Box<Vec<_, _>>`).

## Trade-offs

Keep these in mind when choosing between heap allocated, relocatable collections
and fixed capacity collections.

### Out Of Memory and error handling

With heap allocations Out Of Memory is always a possibility and can occur in
any place where a collection may need to grow: for example, all
`alloc::Vec.push` invocations can potentially generate an OOM condition. Thus
some operations can *implicitly* fail. Some `alloc` collections expose
`try_reserve` methods that let you check for potential OOM conditions when
growing the collection but you need be proactive about using them.

If you exclusively use `heapless` collections and you don't use a memory
allocator for anything else then an OOM condition is impossible. Instead, you'll
have to deal with collections running out of capacity on a case by case basis.
That is you'll have deal with *all* the `Result`s returned by methods like
`Vec.push`.

OOM failures can be harder to debug than say `unwrap`-ing on all `Result`s
returned by `heapless::Vec.push` because the observed location of failure may
*not* match with the location of the cause of the problem. For example, even
`vec.reserve(1)` can trigger an OOM if the allocator is nearly exhausted because
some other collection was leaking memory (memory leaks are possible in safe
Rust).

### Memory usage

Reasoning about memory usage of heap allocated collections is hard because the
capacity of long lived collections can change at runtime. Some operations may
implicitly reallocate the collection increasing its memory usage, and some
collections expose methods like `shrink_to_fit` that can potentially reduce the
memory used by the collection -- ultimately, it's up to the allocator to decide
whether to actually shrink the memory allocation or not. Additionally, the
allocator may have to deal with memory fragmentation which can increase the
*apparent* memory usage.

On the other hand if you exclusively use fixed capacity collections, store
most of them in `static` variables and set a maximum size for the call stack
then the linker will detect if you try to use more memory than what's physically
available.

Furthermore, fixed capacity collections allocated on the stack will be reported
by [`-Z emit-stack-sizes`] flag which means that tools that analyze stack usage
(like [`stack-sizes`]) will include them in their analysis.

[`-Z emit-stack-sizes`]: https://doc.rust-lang.org/beta/unstable-book/compiler-flags/emit-stack-sizes.html
[`stack-sizes`]: https://crates.io/crates/stack-sizes

However, fixed capacity collections can *not* be shrunk which can result in
lower load factors (the ratio between the size of the collection and its
capacity) than what relocatable collections can achieve.

### Worst Case Execution Time (WCET)

If you are building time sensitive applications or hard real time applications
then you care, maybe a lot, about the worst case execution time of the different
parts of your program.

The `alloc` collections can reallocate so the WCET of operations that may grow
the collection will also include the time it takes to reallocate the collection,
which itself depends on the *runtime* capacity of the collection. This makes it
hard to determine the WCET of, for example, the `alloc::Vec.push` operation as
it depends on both the allocator being used and its runtime capacity.

On the other hand fixed capacity collections never reallocate so all operations
have a predictable execution time. For example, `heapless::Vec.push` executes in
constant time.

### Ease of use

`alloc` requires setting up a global allocator whereas `heapless` does not.
However, `heapless` requires you to pick the capacity of each collection that
you instantiate.

The `alloc` API will be familiar to virtually every Rust developer. The
`heapless` API tries to closely mimic the `alloc` API but it will never be
exactly the same due to its explicit error handling -- some developers may feel
the explicit error handling is excessive or too cumbersome.
