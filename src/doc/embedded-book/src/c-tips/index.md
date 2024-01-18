# Tips for embedded C developers

This chapter collects a variety of tips that might be useful to experienced
embedded C developers looking to start writing Rust. It will especially
highlight how things you might already be used to in C are different in Rust.

## Preprocessor

In embedded C it is very common to use the preprocessor for a variety of
purposes, such as:

* Compile-time selection of code blocks with `#ifdef`
* Compile-time array sizes and computations
* Macros to simplify common patterns (to avoid function call overhead)

In Rust there is no preprocessor, and so many of these use cases are addressed
differently. In the rest of this section we cover various alternatives to
using the preprocessor.

### Compile-Time Code Selection

The closest match to `#ifdef ... #endif` in Rust are [Cargo features]. These
are a little more formal than the C preprocessor: all possible features are
explicitly listed per crate, and can only be either on or off. Features are
turned on when you list a crate as a dependency, and are additive: if any crate
in your dependency tree enables a feature for another crate, that feature will
be enabled for all users of that crate.

[Cargo features]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section

For example, you might have a crate which provides a library of signal
processing primitives. Each one might take some extra time to compile or
declare some large table of constants which you'd like to avoid. You could
declare a Cargo feature for each component in your `Cargo.toml`:

```toml
[features]
FIR = []
IIR = []
```

Then, in your code, use `#[cfg(feature="FIR")]` to control what is included.

```rust
/// In your top-level lib.rs

#[cfg(feature="FIR")]
pub mod fir;

#[cfg(feature="IIR")]
pub mod iir;
```

You can similarly include code blocks only if a feature is _not_ enabled, or if
any combination of features are or are not enabled.

Additionally, Rust provides a number of automatically-set conditions you can
use, such as `target_arch` to select different code based on architecture. For
full details of the conditional compilation support, refer to the
[conditional compilation] chapter of the Rust reference.

[conditional compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html

The conditional compilation will only apply to the next statement or block. If
a block can not be used in the current scope then the `cfg` attribute will
need to be used multiple times.  It's worth noting that most of the time it is
better to simply include all the code and allow the compiler to remove dead
code when optimising: it's simpler for you and your users, and in general the
compiler will do a good job of removing unused code.

### Compile-Time Sizes and Computation

Rust supports `const fn`, functions which are guaranteed to be evaluable at
compile-time and can therefore be used where constants are required, such as
in the size of arrays. This can be used alongside features mentioned above,
for example:

```rust
const fn array_size() -> usize {
    #[cfg(feature="use_more_ram")]
    { 1024 }
    #[cfg(not(feature="use_more_ram"))]
    { 128 }
}

static BUF: [u32; array_size()] = [0u32; array_size()];
```

These are new to stable Rust as of 1.31, so documentation is still sparse. The
functionality available to `const fn` is also very limited at the time of
writing; in future Rust releases it is expected to expand on what is permitted
in a `const fn`.

### Macros

Rust provides an extremely powerful [macro system]. While the C preprocessor
operates almost directly on the text of your source code, the Rust macro system
operates at a higher level. There are two varieties of Rust macro: _macros by
example_ and _procedural macros_. The former are simpler and most common; they
look like function calls and can expand to a complete expression, statement,
item, or pattern. Procedural macros are more complex but permit extremely
powerful additions to the Rust language: they can transform arbitrary Rust
syntax into new Rust syntax.

[macro system]: https://doc.rust-lang.org/book/ch19-06-macros.html

In general, where you might have used a C preprocessor macro, you probably want
to see if a macro-by-example can do the job instead. They can be defined in
your crate and easily used by your own crate or exported for other users. Be
aware that since they must expand to complete expressions, statements, items,
or patterns, some use cases of C preprocessor macros will not work, for example
a macro that expands to part of a variable name or an incomplete set of items
in a list.

As with Cargo features, it is worth considering if you even need the macro. In
many cases a regular function is easier to understand and will be inlined to
the same code as a macro. The `#[inline]` and `#[inline(always)]` [attributes]
give you further control over this process, although care should be taken here
as well — the compiler will automatically inline functions from the same crate
where appropriate, so forcing it to do so inappropriately might actually lead
to decreased performance.

[attributes]: https://doc.rust-lang.org/reference/attributes.html#inline-attribute

Explaining the entire Rust macro system is out of scope for this tips page, so
you are encouraged to consult the Rust documentation for full details.

## Build System

Most Rust crates are built using Cargo (although it is not required). This
takes care of many difficult problems with traditional build systems. However,
you may wish to customise the build process. Cargo provides [`build.rs`
scripts] for this purpose. They are Rust scripts which can interact with the
Cargo build system as required.

[`build.rs` scripts]: https://doc.rust-lang.org/cargo/reference/build-scripts.html

Common use cases for build scripts include:

* provide build-time information, for example statically embedding the build
  date or Git commit hash into your executable
* generate linker scripts at build time depending on selected features or other
  logic
* change the Cargo build configuration
* add extra static libraries to link against

At present there is no support for post-build scripts, which you might
traditionally have used for tasks like automatic generation of binaries from
the build objects or printing build information.

### Cross-Compiling

Using Cargo for your build system also simplifies cross-compiling. In most
cases it suffices to tell Cargo `--target thumbv6m-none-eabi` and find a
suitable executable in `target/thumbv6m-none-eabi/debug/myapp`.

For platforms not natively supported by Rust, you will need to build `libcore`
for that target yourself. On such platforms, [Xargo] can be used as a stand-in
for Cargo which automatically builds `libcore` for you.

[Xargo]: https://github.com/japaric/xargo

## Iterators vs Array Access

In C you are probably used to accessing arrays directly by their index:

```c
int16_t arr[16];
int i;
for(i=0; i<sizeof(arr)/sizeof(arr[0]); i++) {
    process(arr[i]);
}
```

In Rust this is an anti-pattern: indexed access can be slower (as it needs to
be bounds checked) and may prevent various compiler optimisations. This is an
important distinction and worth repeating: Rust will check for out-of-bounds
access on manual array indexing to guarantee memory safety, while C will
happily index outside the array.

Instead, use iterators:

```rust,ignore
let arr = [0u16; 16];
for element in arr.iter() {
    process(*element);
}
```

Iterators provide a powerful array of functionality you would have to implement
manually in C, such as chaining, zipping, enumerating, finding the min or max,
summing, and more. Iterator methods can also be chained, giving very readable
data processing code.

See the [Iterators in the Book] and [Iterator documentation] for more details.

[Iterators in the Book]: https://doc.rust-lang.org/book/ch13-02-iterators.html
[Iterator documentation]: https://doc.rust-lang.org/core/iter/trait.Iterator.html

## References vs Pointers

In Rust, pointers (called [_raw pointers_]) exist but are only used in specific
circumstances, as dereferencing them is always considered `unsafe` -- Rust
cannot provide its usual guarantees about what might be behind the pointer.

[_raw pointers_]: https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer

In most cases, we instead use _references_, indicated by the `&` symbol, or
_mutable references_, indicated by `&mut`. References behave similarly to
pointers, in that they can be dereferenced to access the underlying values, but
they are a key part of Rust's ownership system: Rust will strictly enforce that
you may only have one mutable reference _or_ multiple non-mutable references to
the same value at any given time.

In practice this means you have to be more careful about whether you need
mutable access to data: where in C the default is mutable and you must be
explicit about `const`, in Rust the opposite is true.

One situation where you might still use raw pointers is interacting directly
with hardware (for example, writing a pointer to a buffer into a DMA peripheral
register), and they are also used under the hood for all peripheral access
crates to allow you to read and write memory-mapped registers.

## Volatile Access

In C, individual variables may be marked `volatile`, indicating to the compiler
that the value in the variable may change between accesses. Volatile variables
are commonly used in an embedded context for memory-mapped registers.

In Rust, instead of marking a variable as `volatile`, we use specific methods
to perform volatile access: [`core::ptr::read_volatile`] and
[`core::ptr::write_volatile`]. These methods take a `*const T` or a `*mut T`
(_raw pointers_, as discussed above) and perform a volatile read or write.

[`core::ptr::read_volatile`]: https://doc.rust-lang.org/core/ptr/fn.read_volatile.html
[`core::ptr::write_volatile`]: https://doc.rust-lang.org/core/ptr/fn.write_volatile.html

For example, in C you might write:

```c
volatile bool signalled = false;

void ISR() {
    // Signal that the interrupt has occurred
    signalled = true;
}

void driver() {
    while(true) {
        // Sleep until signalled
        while(!signalled) { WFI(); }
        // Reset signalled indicator
        signalled = false;
        // Perform some task that was waiting for the interrupt
        run_task();
    }
}
```

The equivalent in Rust would use volatile methods on each access:

```rust,ignore
static mut SIGNALLED: bool = false;

#[interrupt]
fn ISR() {
    // Signal that the interrupt has occurred
    // (In real code, you should consider a higher level primitive,
    //  such as an atomic type).
    unsafe { core::ptr::write_volatile(&mut SIGNALLED, true) };
}

fn driver() {
    loop {
        // Sleep until signalled
        while unsafe { !core::ptr::read_volatile(&SIGNALLED) } {}
        // Reset signalled indicator
        unsafe { core::ptr::write_volatile(&mut SIGNALLED, false) };
        // Perform some task that was waiting for the interrupt
        run_task();
    }
}
```

A few things are worth noting in the code sample:
  * We can pass `&mut SIGNALLED` into the function requiring `*mut T`, since
    `&mut T` automatically converts to a `*mut T` (and the same for `*const T`)
  * We need `unsafe` blocks for the `read_volatile`/`write_volatile` methods,
    since they are `unsafe` functions. It is the programmer's responsibility
    to ensure safe use: see the methods' documentation for further details.

It is rare to require these functions directly in your code, as they will
usually be taken care of for you by higher-level libraries. For memory mapped
peripherals, the peripheral access crates will implement volatile access
automatically, while for concurrency primitives there are better abstractions
available (see the [Concurrency chapter]).

[Concurrency chapter]: ../concurrency/index.md

## Packed and Aligned Types

In embedded C it is common to tell the compiler a variable must have a certain
alignment or a struct must be packed rather than aligned, usually to meet
specific hardware or protocol requirements.

In Rust this is controlled by the `repr` attribute on a struct or union. The
default representation provides no guarantees of layout, so should not be used
for code that interoperates with hardware or C. The compiler may re-order
struct members or insert padding and the behaviour may change with future
versions of Rust.

```rust
struct Foo {
    x: u16,
    y: u8,
    z: u16,
}

fn main() {
    let v = Foo { x: 0, y: 0, z: 0 };
    println!("{:p} {:p} {:p}", &v.x, &v.y, &v.z);
}

// 0x7ffecb3511d0 0x7ffecb3511d4 0x7ffecb3511d2
// Note ordering has been changed to x, z, y to improve packing.
```

To ensure layouts that are interoperable with C, use `repr(C)`:

```rust
#[repr(C)]
struct Foo {
    x: u16,
    y: u8,
    z: u16,
}

fn main() {
    let v = Foo { x: 0, y: 0, z: 0 };
    println!("{:p} {:p} {:p}", &v.x, &v.y, &v.z);
}

// 0x7fffd0d84c60 0x7fffd0d84c62 0x7fffd0d84c64
// Ordering is preserved and the layout will not change over time.
// `z` is two-byte aligned so a byte of padding exists between `y` and `z`.
```

To ensure a packed representation, use `repr(packed)`:

```rust
#[repr(packed)]
struct Foo {
    x: u16,
    y: u8,
    z: u16,
}

fn main() {
    let v = Foo { x: 0, y: 0, z: 0 };
    // References must always be aligned, so to check the addresses of the
    // struct's fields, we use `std::ptr::addr_of!()` to get a raw pointer
    // instead of just printing `&v.x`.
    let px = std::ptr::addr_of!(v.x);
    let py = std::ptr::addr_of!(v.y);
    let pz = std::ptr::addr_of!(v.z);
    println!("{:p} {:p} {:p}", px, py, pz);
}

// 0x7ffd33598490 0x7ffd33598492 0x7ffd33598493
// No padding has been inserted between `y` and `z`, so now `z` is unaligned.
```

Note that using `repr(packed)` also sets the alignment of the type to `1`.

Finally, to specify a specific alignment, use `repr(align(n))`, where `n` is
the number of bytes to align to (and must be a power of two):

```rust
#[repr(C)]
#[repr(align(4096))]
struct Foo {
    x: u16,
    y: u8,
    z: u16,
}

fn main() {
    let v = Foo { x: 0, y: 0, z: 0 };
    let u = Foo { x: 0, y: 0, z: 0 };
    println!("{:p} {:p} {:p}", &v.x, &v.y, &v.z);
    println!("{:p} {:p} {:p}", &u.x, &u.y, &u.z);
}

// 0x7ffec909a000 0x7ffec909a002 0x7ffec909a004
// 0x7ffec909b000 0x7ffec909b002 0x7ffec909b004
// The two instances `u` and `v` have been placed on 4096-byte alignments,
// evidenced by the `000` at the end of their addresses.
```

Note we can combine `repr(C)` with `repr(align(n))` to obtain an aligned and
C-compatible layout. It is not permissible to combine `repr(align(n))` with
`repr(packed)`, since `repr(packed)` sets the alignment to `1`. It is also not
permissible for a `repr(packed)` type to contain a `repr(align(n))` type.

For further details on type layouts, refer to the [type layout] chapter of the
Rust Reference.

[type layout]: https://doc.rust-lang.org/reference/type-layout.html

## Other Resources

* In this book:
    * [A little C with your Rust](../interoperability/c-with-rust.md)
    * [A little Rust with your C](../interoperability/rust-with-c.md)
* [The Rust Embedded FAQs](https://docs.rust-embedded.org/faq.html)
* [Rust Pointers for C Programmers](http://blahg.josefsipek.net/?p=580)
* [I used to use pointers - now what?](https://github.com/diwic/reffers-rs/blob/master/docs/Pointers.md)
