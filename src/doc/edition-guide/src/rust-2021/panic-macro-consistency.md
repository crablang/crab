# Panic macro consistency

## Summary

- `panic!(..)` now always uses `format_args!(..)`, just like `println!()`.
- `panic!("{")` is no longer accepted, without escaping the `{` as `{{`.
- `panic!(x)` is no longer accepted if `x` is not a string literal.
  - Use `std::panic::panic_any(x)` to panic with a non-string payload.
  - Or use `panic!("{}", x)` to use `x`'s `Display` implementation.
- The same applies to `assert!(expr, ..)`.

## Details

The `panic!()` macro is one of Rust's most well known macros.
However, it has [some subtle surprises](https://github.com/rust-lang/rfcs/blob/master/text/3007-panic-plan.md)
that we can't just change due to backwards compatibility.

```rust,ignore
// Rust 2018
panic!("{}", 1); // Ok, panics with the message "1"
panic!("{}"); // Ok, panics with the message "{}"
```

The `panic!()` macro only uses string formatting when it's invoked with more than one argument.
When invoked with a single argument, it doesn't even look at that argument.

```rust,ignore
// Rust 2018
let a = "{";
println!(a); // Error: First argument must be a format string literal
panic!(a); // Ok: The panic macro doesn't care
```

It even accepts non-strings such as `panic!(123)`, which is uncommon and rarely useful since it
produces a surprisingly unhelpful message: `panicked at 'Box<Any>'`.

This will especially be a problem once
[implicit format arguments](https://rust-lang.github.io/rfcs/2795-format-args-implicit-identifiers.html)
are stabilized.
That feature will make `println!("hello {name}")` a short-hand for `println!("hello {}", name)`.
However, `panic!("hello {name}")` would not work as expected,
since `panic!()` doesn't process a single argument as format string.

To avoid that confusing situation, Rust 2021 features a more consistent `panic!()` macro.
The new `panic!()` macro will no longer accept arbitrary expressions as the only argument.
It will, just like `println!()`, always process the first argument as format string.
Since `panic!()` will no longer accept arbitrary payloads,
[`panic_any()`](https://doc.rust-lang.org/stable/std/panic/fn.panic_any.html)
will be the only way to panic with something other than a formatted string.

```rust,ignore
// Rust 2021
panic!("{}", 1); // Ok, panics with the message "1"
panic!("{}"); // Error, missing argument
panic!(a); // Error, must be a string literal
```

In addition, `core::panic!()` and `std::panic!()` will be identical in Rust 2021.
Currently, there are some historical differences between those two,
which can be noticeable when switching `#![no_std]` on or off.

## Migration

A lint, `non_fmt_panics`, gets triggered whenever there is some call to `panic` that uses some 
deprecated behavior that will error in Rust 2021. The `non_fmt_panics` lint has already been a warning 
by default on all editions since the 1.50 release (with several enhancements made in later releases). 
If your code is already warning free, then it should already be ready to go for Rust 2021!

You can automatically migrate your code to be Rust 2021 Edition compatible or ensure it is already compatible by
running:

```sh
cargo fix --edition
```

Should you choose or need to manually migrate, you'll need to update all panic invocations to either use the same 
formatting as `println` or use `std::panic::panic_any` to panic with non-string data.

For example, in the case of `panic!(MyStruct)`, you'll need to convert to using `std::panic::panic_any` (note
that this is a function not a macro): `std::panic::panic_any(MyStruct)`.

In the case of panic messages that include curly braces but the wrong number of arguments (e.g., `panic!("Some curlies: {}")`), 
you can panic with the string literal by either using the same syntax as `println!` (i.e., `panic!("{}", "Some curlies: {}")`) 
or by escaping the curly braces (i.e., `panic!("Some curlies: {{}}")`).