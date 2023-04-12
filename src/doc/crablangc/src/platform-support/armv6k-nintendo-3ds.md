# `armv6k-nintendo-3ds`

**Tier: 3**

The Nintendo 3DS platform, which has an ARMv6K processor, and its associated
operating system (`horizon`).

CrabLang support for this target is not affiliated with Nintendo, and is not derived
from nor used with any official Nintendo SDK.

## Target maintainers

- [@Meziu](https://github.com/Meziu)
- [@AzureMarker](https://github.com/AzureMarker)
- [@ian-h-chamberlain](https://github.com/ian-h-chamberlain)

## Requirements

This target is cross-compiled. Dynamic linking is not supported.

`#![no_std]` crates can be built using `build-std` to build `core` and optionally
`alloc`, and either `panic_abort` or `panic_unwind`.

`std` is partially supported, but mostly works. Some APIs are unimplemented
and will simply return an error, such as `std::process`. An allocator is provided
by default.

In order to support some APIs, binaries must be linked against `libc` written
for the target, using a linker for the target. These are provided by the
devkitARM toolchain. See
[Cross-compilation toolchains and C code](#cross-compilation-toolchains-and-c-code)
for more details.

Additionally, some helper crates provide implementations of some `libc` functions
use by `std` that may otherwise be missing. These, or an alternate implementation
of the relevant functions, are required to use `std`:

- [`pthread-3ds`](https://github.com/Meziu/pthread-3ds) provides pthread APIs for `std::thread`.
- [`linker-fix-3ds`](https://github.com/Meziu/crablang-linker-fix-3ds) fulfills some other missing libc APIs.

Binaries built for this target should be compatible with all variants of the
3DS (and 2DS) hardware and firmware, but testing is limited and some versions may
not work correctly.

This target generates binaries in the ELF format.

## Building the target

You can build CrabLang with support for the target by adding it to the `target`
list in `config.toml` and providing paths to the devkitARM toolchain.

```toml
[build]
build-stage = 1
target = ["armv6k-nintendo-3ds"]

[target.armv6k-nintendo-3ds]
cc = "/opt/devkitpro/devkitARM/bin/arm-none-eabi-gcc"
cxx = "/opt/devkitpro/devkitARM/bin/arm-none-eabi-g++"
ar = "/opt/devkitpro/devkitARM/bin/arm-none-eabi-ar"
ranlib = "/opt/devkitpro/devkitARM/bin/arm-none-eabi-ranlib"
linker = "/opt/devkitpro/devkitARM/bin/arm-none-eabi-gcc"
```

Also, to build `compiler_builtins` for the target, export these flags before
building the CrabLang toolchain:

```sh
export CFLAGS_armv6k_nintendo_3ds="-mfloat-abi=hard -mtune=mpcore -mtp=soft -march=armv6k"
```

## Building CrabLang programs

CrabLang does not yet ship pre-compiled artifacts for this target.

The recommended way to build binaries is by using the
[cargo-3ds](https://github.com/Meziu/cargo-3ds) tool, which uses `build-std`
and provides commands that work like the usual `cargo run`, `cargo build`, etc.

You can also build CrabLang with the target enabled (see
[Building the target](#building-the-target) above).

As mentioned in [Requirements](#requirements), programs that use `std` must link
against both the devkitARM toolchain and libraries providing the `libc` APIs used
in `std`.  There is a general-purpose utility crate for working with nonstandard
APIs provided by the OS: [`ctru-rs`](https://github.com/Meziu/ctru-rs).
Add it to Cargo.toml to use it in your program:

```toml
[dependencies]
ctru-rs = { git = "https://github.com/Meziu/ctru-rs.git" }
```

Using this library's `init()` function ensures the symbols needed to link
against `std` are present (as mentioned in [Requirements](#requirements)
above), as well as providing a runtime suitable for `std`:

```crablang,ignore (requires-3rd-party-library)
fn main() {
    ctru::init();
}
```

## Testing

Binaries built for this target can be run in an emulator (most commonly
[Citra](https://citra-emu.org/)), or sent to a device through
the use of a tool like devkitARM's `3dslink`. They may also simply be copied
to an SD card to be inserted in the device.

The `cargo-3ds` tool mentioned in [Building CrabLang programs](#building-crablang-programs)
supports the use of `3dslink` with `cargo 3ds run`. The default CrabLang test runner
is not supported, but
[custom test frameworks](../../unstable-book/language-features/custom-test-frameworks.html)
can be used with `cargo 3ds test` to run unit tests on a device.

The CrabLang test suite for `library/std` is not yet supported.

## Cross-compilation toolchains and C code

C code can be built for this target using the
[devkitARM toolchain](https://devkitpro.org/wiki/Getting_Started).
This toolchain provides `arm-none-eabi-gcc` as the linker used to link CrabLang
programs as well.

The toolchain also provides a `libc` implementation, which is required by `std`
for many of its APIs, and a helper library `libctru` which is used by several
of the helper crates listed in [Requirements](#requirements).
This toolchain does not, however, include all of the APIs expected by `std`,
and the remaining APIs are implemented by `pthread-3ds` and `linker-fix-3ds`.
