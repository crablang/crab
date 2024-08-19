# Missing macro fragment specifiers

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".
More information may be found in the tracking issue at <https://github.com/rust-lang/rust/issues/128143>.

## Summary

- The [`missing_fragment_specifier`] lint is now a hard error.

[`missing_fragment_specifier`]: ../../rustc/lints/listing/deny-by-default.html#missing-fragment-specifier

## Details

The [`missing_fragment_specifier`] lint detects a situation when an **unused** pattern in a `macro_rules!` macro definition has a meta-variable (e.g. `$e`) that is not followed by a fragment specifier (e.g. `:expr`). This is now a hard error in the 2024 Edition.

```rust,compile_fail
macro_rules! foo {
   () => {};
   ($name) => { }; // ERROR: missing fragment specifier
}

fn main() {
   foo!();
}
```

Calling the macro with arguments that would match a rule with a missing specifier (e.g., `foo!($name)`) is a hard error in all editions. However, simply defining a macro with missing fragment specifiers is not, though we did add a lint in Rust 1.17.

We'd like to make this a hard error in all editions, but there would be too much breakage right now. So we're starting by making this a hard error in Rust 2024.[^future-incompat]

[^future-incompat]: The lint is marked as a "future-incompatible" warning to indicate that it may become a hard error in all editions in a future release. See [#40107] for more information.

[#40107]: https://github.com/rust-lang/rust/issues/40107

## Migration

To migrate your code to the 2024 Edition, remove the unused matcher rule from the macro. The [`missing_fragment_specifier`] lint is on by default in all editions, and should alert you to macros with this issue.

There is no automatic migration for this change. We expect that this style of macro is extremely rare. The lint has been a future-incompatibility lint since Rust 1.17, a deny-by-default lint since Rust 1.20, and since Rust 1.82, it has warned about dependencies that are using this pattern.
