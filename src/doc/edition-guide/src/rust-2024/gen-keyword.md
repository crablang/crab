# `gen` keyword

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".
More information may be found in the tracking issue at <https://github.com/rust-lang/rust/issues/123904>.

## Summary

- `gen` is a [reserved keyword].

[reserved keyword]: ../../reference/keywords.html#reserved-keywords

## Details

The `gen` keyword has been reserved as part of [RFC #3513] to introduce "gen blocks" in a future release of Rust. `gen` blocks will provide a way to make it easier to write certain kinds of iterators. Reserving the keyword now will make it easier to stabilize `gen` blocks before the next edition.

[RFC #3513]: https://rust-lang.github.io/rfcs/3513-gen-blocks.html

## Migration

Introducing the `gen` keyword can cause a problem for any identifiers that are already called `gen`. For example, any variable or function name called `gen` would clash with the new keyword. To overcome this, Rust supports the `r#` prefix for a [raw identifier], which allows identifiers to overlap with keywords.

The [`keyword_idents_2024`] lint will automatically modify any identifier named `gen` to be `r#gen` so that code continues to work on both editions. This lint is part of the `rust-2024-compatibility` lint group, which will automatically be applied when running `cargo fix --edition`. To migrate your code to be Rust 2024 Edition compatible, run:

```sh
cargo fix --edition
```

For example, this will change:

```rust
fn gen() {
    println!("generating!");
}

fn main() {
    gen();
}
```

to be:

```rust
fn r#gen() {
    println!("generating!");
}

fn main() {
    r#gen();
}
```

Alternatively, you can manually enable the lint to find places where `gen` identifiers need to be modified to `r#gen`:

```rust
// Add this to the root of your crate to do a manual migration.
#![warn(keyword_idents_2024)]
```

[raw identifier]: ../../reference/identifiers.html#raw-identifiers
[`keyword_idents_2024`]: ../../rustc/lints/listing/allowed-by-default.html#keyword-idents-2024
