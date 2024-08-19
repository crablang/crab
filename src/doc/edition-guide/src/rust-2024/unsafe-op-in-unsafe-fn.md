# unsafe_op_in_unsafe_fn warning

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".

## Summary

- The [`unsafe_op_in_unsafe_fn`] lint now warns by default.
  This warning detects calls to unsafe operations in unsafe functions without an explicit unsafe block.

[`unsafe_op_in_unsafe_fn`]: ../../rustc/lints/listing/allowed-by-default.html#unsafe-op-in-unsafe-fn

## Details

The [`unsafe_op_in_unsafe_fn`] lint will fire if there are [unsafe operations] in an unsafe function without an explicit [`unsafe {}` block][unsafe-block].

```rust
# #![warn(unsafe_op_in_unsafe_fn)]
unsafe fn get_unchecked<T>(x: &[T], i: usize) -> &T {
  x.get_unchecked(i) // WARNING: requires unsafe block
}
```

The solution is to wrap any unsafe operations in an `unsafe` block:

```rust
# #![deny(unsafe_op_in_unsafe_fn)]
unsafe fn get_unchecked<T>(x: &[T], i: usize) -> &T {
  unsafe { x.get_unchecked(i) }
}
```

This change is intended to help protect against accidental use of unsafe operations in an unsafe function.
The `unsafe` function keyword was performing two roles.
One was to declare that *calling* the function requires unsafe, and that the caller is responsible to uphold additional safety requirements.
The other role was to allow the use of unsafe operations inside of the function.
This second role was determined to be too risky without explicit `unsafe` blocks.

More information and motivation may be found in [RFC #2585].

[unsafe operations]: ../../reference/unsafety.html
[unsafe-block]: ../../reference/expressions/block-expr.html#unsafe-blocks
[RFC #2585]: https://rust-lang.github.io/rfcs/2585-unsafe-block-in-unsafe-fn.html

## Migration

The [`unsafe_op_in_unsafe_fn`] lint is part of the `rust-2024-compatibility` lint group.
In order to migrate your code to be Rust 2024 Edition compatible, run:

```sh
cargo fix --edition
```

Alternatively, you can manually enable the lint to find places where unsafe blocks need to be added, or switch it to `allow` to silence the lint completely.

```rust
// Add this to the root of your crate to do a manual migration.
#![warn(unsafe_op_in_unsafe_fn)]
```
