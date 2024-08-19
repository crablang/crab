# Warnings promoted to errors

## Summary

- Code that triggered the `bare_trait_objects` and `ellipsis_inclusive_range_patterns` lints will error in Rust 2021.

## Details

Two existing lints are becoming hard errors in Rust 2021, but these lints will remain warnings in older editions.

### `bare_trait_objects`:

The use of the `dyn` keyword to identify [trait objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
will be mandatory in Rust 2021.

For example, the following code which does not include the `dyn` keyword in `&MyTrait`
will produce an error instead of just a lint in Rust 2021:

```rust
pub trait MyTrait {}

pub fn my_function(_trait_object: &MyTrait) { // should be `&dyn MyTrait`
  unimplemented!()
}
```

### `ellipsis_inclusive_range_patterns`:

The [deprecated `...` syntax](https://doc.rust-lang.org/stable/reference/patterns.html#range-patterns)
for inclusive range patterns (i.e., ranges where the end value is *included* in the range) is no longer 
accepted in Rust 2021. It has been superseded by `..=`, which is consistent with expressions.

For example, the following code which uses `...` in a pattern will produce an error instead of 
just a lint in Rust 2021:

```rust
pub fn less_or_eq_to_100(n: u8) -> bool {
  matches!(n, 0...100) // should be `0..=100`
}
```

## Migrations 

If your Rust 2015 or 2018 code does not produce any warnings for `bare_trait_objects` 
or `ellipsis_inclusive_range_patterns` and you've not allowed these lints through the 
use of `#![allow()]` or some other mechanism, then there's no need to migrate.

To automatically migrate any crate that uses `...` in patterns or does not use `dyn` with
trait objects, you can run `cargo fix --edition`.