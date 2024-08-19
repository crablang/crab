# Macro Fragment Specifiers

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".
More information may be found in the tracking issue at <https://github.com/rust-lang/rust/issues/123742>.

## Summary

- The `expr` [fragment specifier] now also supports `const` and `_` expressions.
- The `expr_2021` fragment specifier has been added for backwards compatibility.

[fragment specifier]: ../../reference/macros-by-example.html#metavariables

## Details

As new syntax is added to Rust, existing `macro_rules` fragment specifiers are sometimes not allowed to match on the new syntax in order to retain backwards compatibility. Supporting the new syntax in the old fragment specifiers is sometimes deferred until the next edition, which provides an opportunity to update them.

Indeed this happened with [`const` expressions] added in 1.79 and [`_` expressions] added in 1.59. In the 2021 Edition and earlier, the `expr` fragment specifier does *not* match those expressions. This is because you may have a scenario like:

```rust,edition2021
macro_rules! example {
    ($e:expr) => { println!("first rule"); };
    (const $e:expr) => { println!("second rule"); };
}

fn main() {
    example!(const { 1 + 1 });
}
```

Here, in the 2021 Edition, the macro will match the *second* rule. If earlier editions had changed `expr` to match the newly introduced `const` expressions, then it would match the *first* rule, which would be a breaking change.

In the 2024 Edition, `expr` specifiers now also match `const` and `_` expressions. To support the old behavior, the `expr_2021` fragment specifier has been added which does *not* match the new expressions.

[`const` expressions]: ../../reference/expressions/block-expr.html#const-blocks
[`_` expressions]: ../../reference/expressions/underscore-expr.html

## Migration

The [`edition_2024_expr_fragment_specifier`] lint will change all uses of the `expr` specifier to `expr_2021` to ensure that the behavior of existing macros does not change. The lint is part of the `rust-2024-compatibility` lint group which is included in the automatic edition migration. In order to migrate your code to be Rust 2024 Edition compatible, run:

```sh
cargo fix --edition
```

In *most* cases, you will likely want to keep the `expr` specifier instead, in order to support the new expressions. You will need to review your macro to determine if there are other rules that would otherwise match with `const` or `_` and determine if there is a conflict. If you want the new behavior, just revert any changes made by the lint.

Alternatively, you can manually enable the lint to find macros where you may need to update the `expr` specifier.

```rust
// Add this to the root of your crate to do a manual migration.
#![warn(edition_2024_expr_fragment_specifier)]
```

[`edition_2024_expr_fragment_specifier`]: ../../rustc/lints/listing/allowed-by-default.html#edition-2024-expr-fragment-specifier
