# Reserving syntax

## Summary

- `any_identifier#`, `any_identifier"..."`, and `any_identifier'...'` are now reserved
  syntax, and no longer tokenize.
- This is mostly relevant to macros. E.g. `quote!{ #a#b }` is no longer accepted.
- It doesn't treat keywords specially, so e.g. `match"..." {}` is no longer accepted.
- Insert whitespace between the identifier and the subsequent `#`, `"`, or `'`
  to avoid errors.
- Edition migrations will help you insert whitespace in such cases.

## Details

To make space for new syntax in the future,
we've decided to reserve syntax for prefixed identifiers and literals:
`prefix#identifier`, `prefix"string"`, `prefix'c'`, and `prefix#123`,
where `prefix` can be any identifier.
(Except those prefixes that already have a meaning, such as `b'...'` (byte
chars) and `r"..."` (raw strings).)

This provides syntax we can expand into in the future without requiring an
edition boundary. We may use this for temporary syntax until the next edition,
or for permanent syntax if appropriate.

Without an edition, this would be a breaking change, since macros can currently
accept syntax such as `hello"world"`, which they will see as two separate
tokens: `hello` and `"world"`. The (automatic) fix is simple though: just
insert a space: `hello "world"`. Likewise, `prefix#ident` should become
`prefix #ident`. Edition migrations will help with this fix.

Other than turning these into a tokenization error,
[the RFC][10] does not attach a meaning to any prefix yet.
Assigning meaning to specific prefixes is left to future proposals,
which will now&mdash;thanks to reserving these prefixes&mdash;not be breaking changes.

Some new prefixes you might potentially see in the future (though we haven't
committed to any of them yet):

- `k#keyword` to allow writing keywords that don't exist yet in the current edition.
  For example, while `async` is not a keyword in edition 2015,
  this prefix would've allowed us to accept `k#async` in edition 2015
  without having to wait for edition 2018 to reserve `async` as a keyword.

- `f""` as a short-hand for a format string.
  For example, `f"hello {name}"` as a short-hand for the equivalent `format!()` invocation.

- `s""` for `String` literals.

- `c""` or `z""` for null-terminated C strings.

[10]: https://github.com/rust-lang/rfcs/pull/3101


## Migration 

As a part of the 2021 edition a migration lint, `rust_2021_prefixes_incompatible_syntax`, has been added in order to aid in automatic migration of Rust 2018 codebases to Rust 2021.

In order to have `rustfix` migrate your code to be Rust 2021 Edition compatible, run:

```sh
cargo fix --edition
```

Should you want or need to manually migrate your code, migration is fairly straight-forward.

Let's say you have a macro that is defined like so:

```rust
macro_rules! my_macro {
    ($a:tt $b:tt) => {};
}
```

In Rust 2015 and 2018 it's legal for this macro to be called like so with no space between the first token tree and the second:

```rust,ignore
my_macro!(z"hey");
```

This `z` prefix is no longer allowed in Rust 2021, so in order to call this macro, you must add a space after the prefix like so:

```rust,ignore
my_macro!(z "hey");
```
