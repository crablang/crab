# Or patterns in macro-rules

## Summary

- How patterns work in `macro_rules` macros changes slightly:
	- `$_:pat` in `macro_rules` now matches usage of `|` too: e.g. `A | B`.
	- The new `$_:pat_param` behaves like `$_:pat` did before; it does not match (top level) `|`.
	- `$_:pat_param` is available in all editions.

## Details

Starting in Rust 1.53.0, [patterns](https://doc.rust-lang.org/stable/reference/patterns.html)
are extended to support `|` nested anywhere in the pattern.
This enables you to write `Some(1 | 2)` instead of `Some(1) | Some(2)`.
Since this was simply not allowed before, this is not a breaking change.

However, this change also affects [`macro_rules` macros](https://doc.rust-lang.org/stable/reference/macros-by-example.html).
Such macros can accept patterns using the `:pat` fragment specifier.
Currently, `:pat` does *not* match top level `|`, since before Rust 1.53,
not all patterns (at all nested levels) could contain a `|`.
Macros that accept patterns like `A | B`,
such as [`matches!()`](https://doc.rust-lang.org/1.51.0/std/macro.matches.html)
use something like `$($_:pat)|+`. 

Because this would potentially break existing macros, the meaning of `:pat` did 
not change in Rust 1.53.0 to include `|`. Instead, that change happens in Rust 2021. 
In the new edition, the `:pat` fragment specifier *will* match `A | B`.

`$_:pat` fragments in Rust 2021 cannot be followed by an explicit `|`. Since there are times 
that one still wishes to match pattern fragments followed by a `|`, the fragment specified `:pat_param` 
has been added to retain the older behavior.

It's important to remember that editions are _per crate_, so the only relevant edition is the edition
of the crate where the macro is defined. The edition of the crate where the macro is used does not 
change how the macro works.

## Migration 

A lint, `rust_2021_incompatible_or_patterns`, gets triggered whenever there is a use `$_:pat` which
will change meaning in Rust 2021. 

You can automatically migrate your code to be Rust 2021 Edition compatible or ensure it is already compatible by
running:

```sh
cargo fix --edition
```

If you have a macro which relies on `$_:pat` not matching the top level use of `|` in patterns, 
you'll need to change each occurrence of `$_:pat` to `$_:pat_param`.

For example:

```rust
macro_rules! my_macro { 
	($x:pat | $y:pat) => {
		// TODO: implementation
	} 
}

// This macro works in Rust 2018 since `$x:pat` does not match against `|`:
my_macro!(1 | 2);

// In Rust 2021 however, the `$_:pat` fragment matches `|` and is not allowed
// to be followed by a `|`. To make sure this macro still works in Rust 2021
// change the macro to the following:
macro_rules! my_macro { 
	($x:pat_param | $y:pat) => { // <- this line is different
		// TODO: implementation
	} 
}
```