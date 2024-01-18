# Linkage

> Note: This section is described more in terms of the compiler than of
> the language.

The compiler supports various methods to link crates together both
statically and dynamically. This section will explore the various methods to
link crates together, and more information about native libraries can be
found in the [FFI section of the book][ffi].

[ffi]: ../book/ch19-01-unsafe-rust.html#using-extern-functions-to-call-external-code

In one session of compilation, the compiler can generate multiple artifacts
through the usage of either command line flags or the `crate_type` attribute.
If one or more command line flags are specified, all `crate_type` attributes will
be ignored in favor of only building the artifacts specified by command line.

* `--crate-type=bin`, `#![crate_type = "bin"]` - A runnable executable will be
  produced. This requires that there is a `main` function in the crate which
  will be run when the program begins executing. This will link in all Rust and
  native dependencies, producing a single distributable binary.
  This is the default crate type.

* `--crate-type=lib`, `#![crate_type = "lib"]` - A Rust library will be produced.
  This is an ambiguous concept as to what exactly is produced because a library
  can manifest itself in several forms. The purpose of this generic `lib` option
  is to generate the "compiler recommended" style of library. The output library
  will always be usable by rustc, but the actual type of library may change from
  time-to-time. The remaining output types are all different flavors of
  libraries, and the `lib` type can be seen as an alias for one of them (but the
  actual one is compiler-defined).

* `--crate-type=dylib`, `#![crate_type = "dylib"]` - A dynamic Rust library will
  be produced. This is different from the `lib` output type in that this forces
  dynamic library generation. The resulting dynamic library can be used as a
  dependency for other libraries and/or executables. This output type will
  create `*.so` files on Linux, `*.dylib` files on macOS, and `*.dll` files on
  Windows.

* `--crate-type=staticlib`, `#![crate_type = "staticlib"]` - A static system
  library will be produced. This is different from other library outputs in that
  the compiler will never attempt to link to `staticlib` outputs. The
  purpose of this output type is to create a static library containing all of
  the local crate's code along with all upstream dependencies. This output type
  will create `*.a` files on Linux, macOS and Windows (MinGW), and `*.lib` files
  on Windows (MSVC). This format is recommended for use in situations such as
  linking Rust code into an existing non-Rust application
  because it will not have dynamic dependencies on other Rust code.

* `--crate-type=cdylib`, `#![crate_type = "cdylib"]` - A dynamic system
  library will be produced.  This is used when compiling
  a dynamic library to be loaded from another language.  This output type will
  create `*.so` files on Linux, `*.dylib` files on macOS, and `*.dll` files on
  Windows.

* `--crate-type=rlib`, `#![crate_type = "rlib"]` - A "Rust library" file will be
  produced. This is used as an intermediate artifact and can be thought of as a
  "static Rust library". These `rlib` files, unlike `staticlib` files, are
  interpreted by the compiler in future linkage. This essentially means
  that `rustc` will look for metadata in `rlib` files like it looks for metadata
  in dynamic libraries. This form of output is used to produce statically linked
  executables as well as `staticlib` outputs.

* `--crate-type=proc-macro`, `#![crate_type = "proc-macro"]` - The output
  produced is not specified, but if a `-L` path is provided to it then the
  compiler will recognize the output artifacts as a macro and it can be loaded
  for a program. Crates compiled with this crate type  must only export
  [procedural macros]. The compiler will automatically set the `proc_macro`
  [configuration option]. The crates are always compiled with the same target
  that the compiler itself was built with. For example, if you are executing
  the compiler from Linux with an `x86_64` CPU, the target will be
  `x86_64-unknown-linux-gnu` even if the crate is a dependency of another crate
  being built for a different target.

Note that these outputs are stackable in the sense that if multiple are
specified, then the compiler will produce each form of output without
having to recompile. However, this only applies for outputs specified by the
same method. If only `crate_type` attributes are specified, then they will all
be built, but if one or more `--crate-type` command line flags are specified,
then only those outputs will be built.

With all these different kinds of outputs, if crate A depends on crate B, then
the compiler could find B in various different forms throughout the system. The
only forms looked for by the compiler, however, are the `rlib` format and the
dynamic library format. With these two options for a dependent library, the
compiler must at some point make a choice between these two formats. With this
in mind, the compiler follows these rules when determining what format of
dependencies will be used:

1. If a static library is being produced, all upstream dependencies are
   required to be available in `rlib` formats. This requirement stems from the
   reason that a dynamic library cannot be converted into a static format.

   Note that it is impossible to link in native dynamic dependencies to a static
   library, and in this case warnings will be printed about all unlinked native
   dynamic dependencies.

2. If an `rlib` file is being produced, then there are no restrictions on what
   format the upstream dependencies are available in. It is simply required that
   all upstream dependencies be available for reading metadata from.

   The reason for this is that `rlib` files do not contain any of their upstream
   dependencies. It wouldn't be very efficient for all `rlib` files to contain a
   copy of `libstd.rlib`!

3. If an executable is being produced and the `-C prefer-dynamic` flag is not
   specified, then dependencies are first attempted to be found in the `rlib`
   format. If some dependencies are not available in an rlib format, then
   dynamic linking is attempted (see below).

4. If a dynamic library or an executable that is being dynamically linked is
   being produced, then the compiler will attempt to reconcile the available
   dependencies in either the rlib or dylib format to create a final product.

   A major goal of the compiler is to ensure that a library never appears more
   than once in any artifact. For example, if dynamic libraries B and C were
   each statically linked to library A, then a crate could not link to B and C
   together because there would be two copies of A. The compiler allows mixing
   the rlib and dylib formats, but this restriction must be satisfied.

   The compiler currently implements no method of hinting what format a library
   should be linked with. When dynamically linking, the compiler will attempt to
   maximize dynamic dependencies while still allowing some dependencies to be
   linked in via an rlib.

   For most situations, having all libraries available as a dylib is recommended
   if dynamically linking. For other situations, the compiler will emit a
   warning if it is unable to determine which formats to link each library with.

In general, `--crate-type=bin` or `--crate-type=lib` should be sufficient for
all compilation needs, and the other options are just available if more
fine-grained control is desired over the output format of a crate.

## Static and dynamic C runtimes

The standard library in general strives to support both statically linked and
dynamically linked C runtimes for targets as appropriate. For example the
`x86_64-pc-windows-msvc` and `x86_64-unknown-linux-musl` targets typically come
with both runtimes and the user selects which one they'd like. All targets in
the compiler have a default mode of linking to the C runtime. Typically targets
are linked dynamically by default, but there are exceptions which are static by
default such as:

* `arm-unknown-linux-musleabi`
* `arm-unknown-linux-musleabihf`
* `armv7-unknown-linux-musleabihf`
* `i686-unknown-linux-musl`
* `x86_64-unknown-linux-musl`

The linkage of the C runtime is configured to respect the `crt-static` target
feature. These target features are typically configured from the command line
via flags to the compiler itself. For example to enable a static runtime you
would execute:

```sh
rustc -C target-feature=+crt-static foo.rs
```

whereas to link dynamically to the C runtime you would execute:

```sh
rustc -C target-feature=-crt-static foo.rs
```

Targets which do not support switching between linkage of the C runtime will
ignore this flag. It's recommended to inspect the resulting binary to ensure
that it's linked as you would expect after the compiler succeeds.

Crates may also learn about how the C runtime is being linked. Code on MSVC, for
example, needs to be compiled differently (e.g. with `/MT` or `/MD`) depending
on the runtime being linked. This is exported currently through the
[`cfg` attribute `target_feature` option]:

```rust
#[cfg(target_feature = "crt-static")]
fn foo() {
    println!("the C runtime should be statically linked");
}

#[cfg(not(target_feature = "crt-static"))]
fn foo() {
    println!("the C runtime should be dynamically linked");
}
```

Also note that Cargo build scripts can learn about this feature through
[environment variables][cargo]. In a build script you can detect the linkage
via:

```rust
use std::env;

fn main() {
    let linkage = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or(String::new());

    if linkage.contains("crt-static") {
        println!("the C runtime will be statically linked");
    } else {
        println!("the C runtime will be dynamically linked");
    }
}
```

[cargo]: ../cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts

To use this feature locally, you typically will use the `RUSTFLAGS` environment
variable to specify flags to the compiler through Cargo. For example to compile
a statically linked binary on MSVC you would execute:

```sh
RUSTFLAGS='-C target-feature=+crt-static' cargo build --target x86_64-pc-windows-msvc
```

[`cfg` attribute `target_feature` option]: conditional-compilation.md#target_feature
[configuration option]: conditional-compilation.md
[procedural macros]: procedural-macros.md
