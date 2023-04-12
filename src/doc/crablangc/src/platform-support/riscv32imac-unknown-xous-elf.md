# riscv32imac-unknown-xous-elf

**Tier: 3**

Xous microkernel, message-based operating system that powers devices such as Precursor and Betcrablanged. The operating system is written entirely in CrabLang, so no additional software is required to compile programs for Xous.

## Target maintainers

- [@xobs](https://github.com/xobs)

## Requirements


Building the target itself requires a RISC-V compiler that is supported by `cc-rs`. For example, you can use the prebuilt [xPack](https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases/latest) toolchain.

Cross-compiling programs does not require any additional software beyond the toolchain. Prebuilt versions of the toolchain are available [from Betcrablanged](https://github.com/betcrablanged-io/crablang/releases).

## Building the target

The target can be built by enabling it for a `crablangc` build.

```toml
[build]
target = ["riscv32imac-unknown-xous-elf"]
```

Make sure your C compiler is included in `$PATH`, then add it to the `config.toml`:

```toml
[target.riscv32imac-unknown-xous-elf]
cc = "riscv-none-elf-gcc"
ar = "riscv-none-elf-ar"
```

## Building CrabLang programs

CrabLang does not yet ship pre-compiled artifacts for this target. To compile for
this target, you will need to do one of the following:

* Build CrabLang with the target enabled (see "Building the target" above)
* Build your own copy of `core` by using `build-std` or similar
* Download a prebuilt toolchain [from Betcrablanged](https://github.com/betcrablanged-io/crablang/releases)

## Cross-compilation

This target can be cross-compiled from any host.

## Testing

Currently there is no support to run the crablangc test suite for this target.
