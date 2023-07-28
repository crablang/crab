# Static items

> **<sup>Syntax</sup>**\
> _StaticItem_ :\
> &nbsp;&nbsp; `static` `mut`<sup>?</sup> [IDENTIFIER] `:` [_Type_]
>              ( `=` [_Expression_] )<sup>?</sup> `;`

A *static item* is similar to a [constant], except that it represents a precise
memory location in the program. All references to the static refer to the same
memory location. Static items have the `static` lifetime, which outlives all
other lifetimes in a Rust program. Static items do not call [`drop`] at the
end of the program.

The static initializer is a [constant expression] evaluated at compile time.
Static initializers may refer to other statics.

Non-`mut` static items that contain a type that is not [interior mutable] may
be placed in read-only memory.

All access to a static is safe, but there are a number of restrictions on
statics:

* The type must have the `Sync` trait bound to allow thread-safe access.
* Constants cannot refer to statics.

The initializer expression must be omitted in an [external block], and must be
provided for free static items.

## Statics & generics

A static item defined in a generic scope (for example in a blanket or default
implementation) will result in exactly one static item being defined, as if
the static definition was pulled out of the current scope into the module.
There will *not* be one item per monomorphization.

This code:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

trait Tr {
    fn default_impl() {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        println!("default_impl: counter was {}", COUNTER.fetch_add(1, Ordering::Relaxed));
    }

    fn blanket_impl();
}

struct Ty1 {}
struct Ty2 {}

impl<T> Tr for T {
    fn blanket_impl() {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        println!("blanket_impl: counter was {}", COUNTER.fetch_add(1, Ordering::Relaxed));
    }
}

fn main() {
    <Ty1 as Tr>::default_impl();
    <Ty2 as Tr>::default_impl();
    <Ty1 as Tr>::blanket_impl();
    <Ty2 as Tr>::blanket_impl();
}
```

prints

```text
default_impl: counter was 0
default_impl: counter was 1
blanket_impl: counter was 0
blanket_impl: counter was 1
```

## Mutable statics

If a static item is declared with the `mut` keyword, then it is allowed to be
modified by the program. One of Rust's goals is to make concurrency bugs hard
to run into, and this is obviously a very large source of race conditions or
other bugs. For this reason, an `unsafe` block is required when either reading
or writing a mutable static variable. Care should be taken to ensure that
modifications to a mutable static are safe with respect to other threads
running in the same process.

Mutable statics are still very useful, however. They can be used with C
libraries and can also be bound from C libraries in an `extern` block.

```rust
# fn atomic_add(_: &mut u32, _: u32) -> u32 { 2 }

static mut LEVELS: u32 = 0;

// This violates the idea of no shared state, and this doesn't internally
// protect against races, so this function is `unsafe`
unsafe fn bump_levels_unsafe1() -> u32 {
    let ret = LEVELS;
    LEVELS += 1;
    return ret;
}

// Assuming that we have an atomic_add function which returns the old value,
// this function is "safe" but the meaning of the return value may not be what
// callers expect, so it's still marked as `unsafe`
unsafe fn bump_levels_unsafe2() -> u32 {
    return atomic_add(&mut LEVELS, 1);
}
```

Mutable statics have the same restrictions as normal statics, except that the
type does not have to implement the `Sync` trait.

## Using Statics or Consts

It can be confusing whether or not you should use a constant item or a static
item. Constants should, in general, be preferred over statics unless one of the
following are true:

* Large amounts of data are being stored
* The single-address property of statics is required.
* Interior mutability is required.

[constant]: constant-items.md
[`drop`]: ../destructors.md
[constant expression]: ../const_eval.md#constant-expressions
[external block]: external-blocks.md
[interior mutable]: ../interior-mutability.md
[IDENTIFIER]: ../identifiers.md
[_Type_]: ../types.md#type-expressions
[_Expression_]: ../expressions.md
