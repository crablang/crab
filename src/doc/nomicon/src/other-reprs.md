# Alternative representations

Rust allows you to specify alternative data layout strategies from the default.
There's also the [unsafe code guidelines] (note that it's **NOT** normative).

## repr(C)

This is the most important `repr`. It has fairly simple intent: do what C does.
The order, size, and alignment of fields is exactly what you would expect from C
or C++. Any type you expect to pass through an FFI boundary should have
`repr(C)`, as C is the lingua-franca of the programming world. This is also
necessary to soundly do more elaborate tricks with data layout such as
reinterpreting values as a different type.

We strongly recommend using [rust-bindgen] and/or [cbindgen] to manage your FFI
boundaries for you. The Rust team works closely with those projects to ensure
that they work robustly and are compatible with current and future guarantees
about type layouts and `repr`s.

The interaction of `repr(C)` with Rust's more exotic data layout features must be
kept in mind. Due to its dual purpose as "for FFI" and "for layout control",
`repr(C)` can be applied to types that will be nonsensical or problematic if
passed through the FFI boundary.

* ZSTs are still zero-sized, even though this is not a standard behavior in
C, and is explicitly contrary to the behavior of an empty type in C++, which
says they should still consume a byte of space.

* DST pointers (wide pointers) and tuples are not a concept
  in C, and as such are never FFI-safe.

* Enums with fields also aren't a concept in C or C++, but a valid bridging
  of the types [is defined][really-tagged].

* If `T` is an [FFI-safe non-nullable pointer
  type](ffi.html#the-nullable-pointer-optimization),
  `Option<T>` is guaranteed to have the same layout and ABI as `T` and is
  therefore also FFI-safe. As of this writing, this covers `&`, `&mut`,
  and function pointers, all of which can never be null.

* Tuple structs are like structs with regards to `repr(C)`, as the only
  difference from a struct is that the fields aren’t named.

* `repr(C)` is equivalent to one of `repr(u*)` (see the next section) for
fieldless enums. The chosen size is the default enum size for the target platform's C
application binary interface (ABI). Note that enum representation in C is implementation
defined, so this is really a "best guess". In particular, this may be incorrect
when the C code of interest is compiled with certain flags.

* Fieldless enums with `repr(C)` or `repr(u*)` still may not be set to an
integer value without a corresponding variant, even though this is
permitted behavior in C or C++. It is undefined behavior to (unsafely)
construct an instance of an enum that does not match one of its
variants. (This allows exhaustive matches to continue to be written and
compiled as normal.)

## repr(transparent)

`#[repr(transparent)]` can only be used on a struct or single-variant enum that has a single non-zero-sized field (there may be additional zero-sized fields).
The effect is that the layout and ABI of the whole struct/enum is guaranteed to be the same as that one field.

> NOTE: There's a `transparent_unions` nightly feature to apply `repr(transparent)` to unions,
> but it hasn't been stabilized due to design concerns. See the [tracking issue][issue-60405] for more details.

The goal is to make it possible to transmute between the single field and the
struct/enum. An example of that is [`UnsafeCell`], which can be transmuted into
the type it wraps ([`UnsafeCell`] also uses the unstable [no_niche][no-niche-pull],
so its ABI is not actually guaranteed to be the same when nested in other types).

Also, passing the struct/enum through FFI where the inner field type is expected on
the other side is guaranteed to work. In particular, this is necessary for
`struct Foo(f32)` or `enum Foo { Bar(f32) }` to always have the same ABI as `f32`.

This repr is only considered part of the public ABI of a type if either the single
field is `pub`, or if its layout is documented in prose. Otherwise, the layout should
not be relied upon by other crates.

More details are in the [RFC 1758][rfc-transparent] and the [RFC 2645][rfc-transparent-unions-enums].

## repr(u*), repr(i*)

These specify the size to make a fieldless enum. If the discriminant overflows
the integer it has to fit in, it will produce a compile-time error. You can
manually ask Rust to allow this by setting the overflowing element to explicitly
be 0. However Rust will not allow you to create an enum where two variants have
the same discriminant.

The term "fieldless enum" only means that the enum doesn't have data in any
of its variants. A fieldless enum without a `repr(u*)` or `repr(C)` is
still a Rust native type, and does not have a stable ABI representation.
Adding a `repr` causes it to be treated exactly like the specified
integer size for ABI purposes.

If the enum has fields, the effect is similar to the effect of `repr(C)`
in that there is a defined layout of the type. This makes it possible to
pass the enum to C code, or access the type's raw representation and directly
manipulate its tag and fields. See [the RFC][really-tagged] for details.

These `repr`s have no effect on a struct.

Adding an explicit `repr(u*)`, `repr(i*)`, or `repr(C)` to an enum with fields suppresses the null-pointer optimization, like:

```rust
# use std::mem::size_of;
enum MyOption<T> {
    Some(T),
    None,
}

#[repr(u8)]
enum MyReprOption<T> {
    Some(T),
    None,
}

assert_eq!(8, size_of::<MyOption<&u16>>());
assert_eq!(16, size_of::<MyReprOption<&u16>>());
```

This optimization still applies to fieldless enums with an explicit `repr(u*)`, `repr(i*)`, or `repr(C)`.

## repr(packed)

`repr(packed)` forces Rust to strip any padding, and only align the type to a
byte. This may improve the memory footprint, but will likely have other negative
side-effects.

In particular, most architectures *strongly* prefer values to be aligned. This
may mean the unaligned loads are penalized (x86), or even fault (some ARM
chips). For simple cases like directly loading or storing a packed field, the
compiler might be able to paper over alignment issues with shifts and masks.
However if you take a reference to a packed field, it's unlikely that the
compiler will be able to emit code to avoid an unaligned load.

[As this can cause undefined behavior][ub loads], the lint has been implemented
and it will become a hard error.

`repr(packed)` is not to be used lightly. Unless you have extreme requirements,
this should not be used.

This repr is a modifier on `repr(C)` and `repr(Rust)`.

## repr(align(n))

`repr(align(n))` (where `n` is a power of two) forces the type to have an
alignment of *at least* n.

This enables several tricks, like making sure neighboring elements of an array
never share the same cache line with each other (which may speed up certain
kinds of concurrent code).

This is a modifier on `repr(C)` and `repr(Rust)`. It is incompatible with
`repr(packed)`.

[unsafe code guidelines]: https://rust-lang.github.io/unsafe-code-guidelines/layout.html
[drop flags]: drop-flags.html
[ub loads]: https://github.com/rust-lang/rust/issues/27060
[issue-60405]: https://github.com/rust-lang/rust/issues/60405
[`UnsafeCell`]: ../std/cell/struct.UnsafeCell.html
[rfc-transparent]: https://github.com/rust-lang/rfcs/blob/master/text/1758-repr-transparent.md
[rfc-transparent-unions-enums]: https://rust-lang.github.io/rfcs/2645-transparent-unions.html
[really-tagged]: https://github.com/rust-lang/rfcs/blob/master/text/2195-really-tagged-unions.md
[rust-bindgen]: https://rust-lang.github.io/rust-bindgen/
[cbindgen]: https://github.com/eqrion/cbindgen
[no-niche-pull]: https://github.com/rust-lang/rust/pull/68491
