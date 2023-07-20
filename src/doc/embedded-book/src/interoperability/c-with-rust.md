# A little C with your Rust

Using C or C++ inside of a Rust project consists of two major parts:

- Wrapping the exposed C API for use with Rust
- Building your C or C++ code to be integrated with the Rust code

As C++ does not have a stable ABI for the Rust compiler to target, it is recommended to use the `C` ABI when combining Rust with C or C++.

## Defining the interface

Before consuming C or C++ code from Rust, it is necessary to define (in Rust) what data types and function signatures exist in the linked code. In C or C++, you would include a header (`.h` or `.hpp`) file which defines this data. In Rust, it is necessary to either manually translate these definitions to Rust, or use a tool to generate these definitions.

First, we will cover manually translating these definitions from C/C++ to Rust.

### Wrapping C functions and Datatypes

Typically, libraries written in C or C++ will provide a header file defining all types and functions used in public interfaces. An example file may look like this:

```C
/* File: cool.h */
typedef struct CoolStruct {
    int x;
    int y;
} CoolStruct;

void cool_function(int i, char c, CoolStruct* cs);
```

When translated to Rust, this interface would look as such:

```rust,ignore
/* File: cool_bindings.rs */
#[repr(C)]
pub struct CoolStruct {
    pub x: cty::c_int,
    pub y: cty::c_int,
}

extern "C" {
    pub fn cool_function(
        i: cty::c_int,
        c: cty::c_char,
        cs: *mut CoolStruct
    );
}
```

Let's take a look at this definition one piece at a time, to explain each of the parts.

```rust,ignore
#[repr(C)]
pub struct CoolStruct { ... }
```

By default, Rust does not guarantee order, padding, or the size of data included in a `struct`. In order to guarantee compatibility with C code, we include the `#[repr(C)]` attribute, which instructs the Rust compiler to always use the same rules C does for organizing data within a struct.

```rust,ignore
pub x: cty::c_int,
pub y: cty::c_int,
```

Due to the flexibility of how C or C++ defines an `int` or `char`, it is recommended to use primitive data types defined in `cty`, which will map types from C to types in Rust.

```rust,ignore
extern "C" { pub fn cool_function( ... ); }
```

This statement defines the signature of a function that uses the C ABI, called `cool_function`. By defining the signature without defining the body of the function, the definition of this function will need to be provided elsewhere, or linked into the final library or binary from a static library.

```rust,ignore
    i: cty::c_int,
    c: cty::c_char,
    cs: *mut CoolStruct
```

Similar to our datatype above, we define the datatypes of the function arguments using C-compatible definitions. We also retain the same argument names, for clarity.

We have one new type here, `*mut CoolStruct`. As C does not have a concept of Rust's references, which would look like this: `&mut CoolStruct`, we instead have a raw pointer. As dereferencing this pointer is `unsafe`, and the pointer may in fact be a `null` pointer, care must be taken to ensure the guarantees typical of Rust when interacting with C or C++ code.

### Automatically generating the interface

Rather than manually generating these interfaces, which may be tedious and error prone, there is a tool called [bindgen] which will perform these conversions automatically. For instructions of the usage of [bindgen], please refer to the [bindgen user's manual], however the typical process consists of the following:

1. Gather all C or C++ headers defining interfaces or datatypes you would like to use with Rust.
2. Write a `bindings.h` file, which `#include "..."`'s each of the files you gathered in step one.
3. Feed this `bindings.h` file, along with any compilation flags used to compile
  your code into `bindgen`. Tip: use `Builder.ctypes_prefix("cty")` /
  `--ctypes-prefix=cty` and `Builder.use_core()` / `--use-core` to make the generated code `#![no_std]` compatible.
4. `bindgen` will produce the generated Rust code to the output of the terminal window. This file may be piped to a file in your project, such as `bindings.rs`. You may use this file in your Rust project to interact with C/C++ code compiled and linked as an external library. Tip: don't forget to use the [`cty`](https://crates.io/crates/cty) crate if your types in the generated bindings are prefixed with `cty`.

[bindgen]: https://github.com/rust-lang/rust-bindgen
[bindgen user's manual]: https://rust-lang.github.io/rust-bindgen/

## Building your C/C++ code

As the Rust compiler does not directly know how to compile C or C++ code (or code from any other language, which presents a C interface), it is necessary to compile your non-Rust code ahead of time.

For embedded projects, this most commonly means compiling the C/C++ code to a static archive (such as `cool-library.a`), which can then be combined with your Rust code at the final linking step.

If the library you would like to use is already distributed as a static archive, it is not necessary to rebuild your code. Just convert the provided interface header file as described above, and include the static archive at compile/link time.

If your code exists as a source project, it will be necessary to compile your C/C++ code to a static library, either by triggering your existing build system (such as `make`, `CMake`, etc.), or by porting the necessary compilation steps to use a tool called the `cc` crate. For both of these steps, it is necessary to use a `build.rs` script.

### Rust `build.rs` build scripts

A `build.rs` script is a file written in Rust syntax, that is executed on your compilation machine, AFTER dependencies of your project have been built, but BEFORE your project is built.

The full reference may be found [here](https://doc.rust-lang.org/cargo/reference/build-scripts.html). `build.rs` scripts are useful for generating code (such as via [bindgen]), calling out to external build systems such as `Make`, or directly compiling C/C++ through use of the `cc` crate.

### Triggering external build systems

For projects with complex external projects or build systems, it may be easiest to use [`std::process::Command`] to "shell out" to your other build systems by traversing relative paths, calling a fixed command (such as `make library`), and then copying the resulting static library to the proper location in the `target` build directory.

While your crate may be targeting a `no_std` embedded platform, your `build.rs` executes only on machines compiling your crate. This means you may use any Rust crates which will run on your compilation host.

[`std::process::Command`]: https://doc.rust-lang.org/std/process/struct.Command.html

### Building C/C++ code with the `cc` crate

For projects with limited dependencies or complexity, or for projects where it is difficult to modify the build system to produce a static library (rather than a final binary or executable), it may be easier to instead utilize the [`cc` crate], which provides an idiomatic Rust interface to the compiler provided by the host.

[`cc` crate]: https://github.com/alexcrichton/cc-rs

In the simplest case of compiling a single C file as a dependency to a static library, an example `build.rs` script using the [`cc` crate] would look like this:

```rust,ignore
fn main() {
    cc::Build::new()
        .file("src/foo.c")
        .compile("foo");
}
```

The `build.rs` is placed at the root of the package. Then `cargo build` will compile and execute it before the build of the package. A static archive named `libfoo.a` is generated and placed in the `target` directory.
