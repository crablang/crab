# Beneath `std`

This section documents features that are normally provided by the `std` crate and
that `#![no_std]` developers have to deal with (i.e. provide) to build
`#![no_std]` binary crates.

## Using `libc`

In order to build a `#[no_std]` executable we will need `libc` as a dependency.
We can specify this using our `Cargo.toml` file:

```toml
[dependencies]
libc = { version = "0.2.146", default-features = false }
```

Note that the default features have been disabled. This is a critical step -
**the default features of `libc` include the `std` crate and so must be
disabled.**

Alternatively, we can use the unstable `rustc_private` private feature together
with an `extern crate libc;` declaration as shown in the examples below.

## Writing an executable without `std`

We will probably need a nightly version of the compiler to produce
a `#![no_std]` executable because on many platforms, we have to provide the
`eh_personality` [lang item], which is unstable.

Controlling the entry point is possible in two ways: the `#[start]` attribute,
or overriding the default shim for the C `main` function with your own.
Additionally, it's required to define a [panic handler function](panic-handler.html).

The function marked `#[start]` is passed the command line parameters
in the same format as C (aside from the exact integer types being used):

```rust
#![feature(start, lang_items, core_intrinsics, rustc_private)]
#![allow(internal_features)]
#![no_std]

// Necessary for `panic = "unwind"` builds on some platforms.
#![feature(panic_unwind)]
extern crate unwind;

// Pull in the system libc library for what crt0.o likely requires.
extern crate libc;

use core::panic::PanicInfo;

// Entry point for this program.
#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    0
}

// These functions are used by the compiler, but not for an empty program like this.
// They are normally provided by `std`.
#[lang = "eh_personality"]
fn rust_eh_personality() {}
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! { core::intrinsics::abort() }
```

To override the compiler-inserted `main` shim, we have to disable it
with `#![no_main]` and then create the appropriate symbol with the
correct ABI and the correct name, which requires overriding the
compiler's name mangling too:

```rust
#![feature(lang_items, core_intrinsics, rustc_private)]
#![allow(internal_features)]
#![no_std]
#![no_main]

// Necessary for `panic = "unwind"` builds on some platforms.
#![feature(panic_unwind)]
extern crate unwind;

// Pull in the system libc library for what crt0.o likely requires.
extern crate libc;

use core::ffi::{c_char, c_int};
use core::panic::PanicInfo;

// Entry point for this program.
#[no_mangle] // ensure that this symbol is included in the output as `main`
extern "C" fn main(_argc: c_int, _argv: *const *const c_char) -> c_int {
    0
}

// These functions are used by the compiler, but not for an empty program like this.
// They are normally provided by `std`.
#[lang = "eh_personality"]
fn rust_eh_personality() {}
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! { core::intrinsics::abort() }
```

If you are working with a target that doesn't have binary releases of the
standard library available via rustup (this probably means you are building the
`core` crate yourself) and need compiler-rt intrinsics (i.e. you are probably
getting linker errors when building an executable:
``undefined reference to `__aeabi_memcpy'``), you need to manually link to the
[`compiler_builtins` crate] to get those intrinsics and solve the linker errors.

[`compiler_builtins` crate]: https://crates.io/crates/compiler_builtins
[lang item]: https://doc.rust-lang.org/nightly/unstable-book/language-features/lang-items.html
