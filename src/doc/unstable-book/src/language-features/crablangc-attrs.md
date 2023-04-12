# `crablangc_attrs`

This feature has no tracking issue, and is therefore internal to
the compiler, not being intended for general use.

Note: `crablangc_attrs` enables many crablangc-internal attributes and this page
only discuss a few of them.

------------------------

The `crablangc_attrs` feature allows debugging crablangc type layouts by using
`#[crablangc_layout(...)]` to debug layout at compile time (it even works
with `cargo check`) as an alternative to `crablangc -Z print-type-sizes`
that is way more verbose.

Options provided by `#[crablangc_layout(...)]` are `debug`, `size`, `align`,
`abi`. Note that it only works on sized types without generics.

## Examples

```crablang,compile_fail
#![feature(crablangc_attrs)]

#[crablangc_layout(abi, size)]
pub enum X {
    Y(u8, u8, u8),
    Z(isize),
}
```

When that is compiled, the compiler will error with something like

```text
error: abi: Aggregate { sized: true }
 --> src/lib.rs:4:1
  |
4 | / pub enum T {
5 | |     Y(u8, u8, u8),
6 | |     Z(isize),
7 | | }
  | |_^

error: size: Size { raw: 16 }
 --> src/lib.rs:4:1
  |
4 | / pub enum T {
5 | |     Y(u8, u8, u8),
6 | |     Z(isize),
7 | | }
  | |_^

error: aborting due to 2 previous errors
```
