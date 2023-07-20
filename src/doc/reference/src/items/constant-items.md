# Constant items

> **<sup>Syntax</sup>**\
> _ConstantItem_ :\
> &nbsp;&nbsp; `const` ( [IDENTIFIER] | `_` ) `:` [_Type_] ( `=` [_Expression_] )<sup>?</sup> `;`

A *constant item* is an optionally named _[constant value]_ which is not associated
with a specific memory location in the program. Constants are essentially inlined
wherever they are used, meaning that they are copied directly into the relevant
context when used. This includes usage of constants from external crates, and
non-[`Copy`] types. References to the same constant are not necessarily
guaranteed to refer to the same memory address.

Constants must be explicitly typed. The type must have a `'static` lifetime: any
references in the initializer must have `'static` lifetimes.

Constants may refer to the address of other constants, in which case the
address will have elided lifetimes where applicable, otherwise – in most cases
– defaulting to the `static` lifetime. (See [static lifetime
elision].) The compiler is, however, still at liberty to translate the constant
many times, so the address referred to may not be stable.

```rust
const BIT1: u32 = 1 << 0;
const BIT2: u32 = 1 << 1;

const BITS: [u32; 2] = [BIT1, BIT2];
const STRING: &'static str = "bitstring";

struct BitsNStrings<'a> {
    mybits: [u32; 2],
    mystring: &'a str,
}

const BITS_N_STRINGS: BitsNStrings<'static> = BitsNStrings {
    mybits: BITS,
    mystring: STRING,
};
```

The constant expression may only be omitted in a [trait definition].

## Constants with Destructors

Constants can contain destructors. Destructors are run when the value goes out
of scope.

```rust
struct TypeWithDestructor(i32);

impl Drop for TypeWithDestructor {
    fn drop(&mut self) {
        println!("Dropped. Held {}.", self.0);
    }
}

const ZERO_WITH_DESTRUCTOR: TypeWithDestructor = TypeWithDestructor(0);

fn create_and_drop_zero_with_destructor() {
    let x = ZERO_WITH_DESTRUCTOR;
    // x gets dropped at end of function, calling drop.
    // prints "Dropped. Held 0.".
}
```

## Unnamed constant

Unlike an [associated constant], a [free] constant may be unnamed by using
an underscore instead of the name. For example:

```rust
const _: () =  { struct _SameNameTwice; };

// OK although it is the same name as above:
const _: () =  { struct _SameNameTwice; };
```

As with [underscore imports], macros may safely emit the same unnamed constant in
the same scope more than once. For example, the following should not produce an error:

```rust
macro_rules! m {
    ($item: item) => { $item $item }
}

m!(const _: () = (););
// This expands to:
// const _: () = ();
// const _: () = ();
```

## Evaluation

[Free][free] constants are always [evaluated][const_eval] at compile-time to surface
panics. This happens even within an unused function:

```rust,compile_fail
// Compile-time panic
const PANIC: () = std::unimplemented!();

fn unused_generic_function<T>() {
    // A failing compile-time assertion
    const _: () = assert!(usize::BITS == 0);
}
```

[const_eval]: ../const_eval.md
[associated constant]: ../items/associated-items.md#associated-constants
[constant value]: ../const_eval.md#constant-expressions
[free]: ../glossary.md#free-item
[static lifetime elision]: ../lifetime-elision.md#static-lifetime-elision
[trait definition]: traits.md
[IDENTIFIER]: ../identifiers.md
[underscore imports]: use-declarations.md#underscore-imports
[_Type_]: ../types.md#type-expressions
[_Expression_]: ../expressions.md
[`Copy`]: ../special-types-and-traits.md#copy
