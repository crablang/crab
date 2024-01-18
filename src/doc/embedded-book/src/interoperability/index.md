# Interoperability

Interoperability between Rust and C code is always dependent
on transforming data between the two languages.
For this purpose, there is a dedicated module
in the `stdlib` called
[`std::ffi`](https://doc.rust-lang.org/std/ffi/index.html).

`std::ffi` provides type definitions for C primitive types,
such as `char`, `int`, and `long`.
It also provides some utility for converting more complex
types such as strings, mapping both `&str` and `String`
to C types that are easier and safer to handle.

As of Rust 1.30,
functionalities of `std::ffi` are available
in either `core::ffi` or `alloc::ffi`
depending on whether or not memory allocation is involved.
The [`cty`] crate and the [`cstr_core`] crate
also offer similar functionalities.

[`cstr_core`]: https://crates.io/crates/cstr_core
[`cty`]: https://crates.io/crates/cty

| Rust type      | Intermediate | C type         |
|----------------|--------------|----------------|
| `String`       | `CString`    | `char *`       |
| `&str`         | `CStr`       | `const char *` |
| `()`           | `c_void`     | `void`         |
| `u32` or `u64` | `c_uint`     | `unsigned int` |
| etc            | ...          | ...            |

A value of a C primitive type can be used
as one of the corresponding Rust type and vice versa,
since the former is simply a type alias of the latter.
For example, the following code compiles on platforms
where `unsigned int` is 32-bit long.

```rust,ignore
fn foo(num: u32) {
    let c_num: c_uint = num;
    let r_num: u32 = c_num;
}
```

## Interoperability with other build systems

A common requirement for including Rust in your embedded project is combining
Cargo with your existing build system, such as make or cmake.

We are collecting examples and use cases for this on our issue tracker in
[issue #61].

[issue #61]: https://github.com/rust-embedded/book/issues/61

## Interoperability with RTOSs

Integrating Rust with an RTOS such as FreeRTOS or ChibiOS is still a work in
progress; especially calling RTOS functions from Rust can be tricky.

We are collecting examples and use cases for this on our issue tracker in
[issue #62].

[issue #62]: https://github.com/rust-embedded/book/issues/62
