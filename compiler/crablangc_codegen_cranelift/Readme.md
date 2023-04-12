# Cranelift codegen backend for crablang

The goal of this project is to create an alternative codegen backend for the crablang compiler based on [Cranelift](https://github.com/bytecodealliance/wasmtime/blob/main/cranelift).
This has the potential to improve compilation times in debug mode.
If your project doesn't use any of the things listed under "Not yet supported", it should work fine.
If not please open an issue.

## Building and testing

```bash
$ git clone https://github.com/bjorn3/crablangc_codegen_cranelift
$ cd crablangc_codegen_cranelift
$ ./y.rs prepare
$ ./y.rs build
```

To run the test suite replace the last command with:

```bash
$ ./test.sh
```

For more docs on how to build and test see [build_system/usage.txt](build_system/usage.txt) or the help message of `./y.rs`.

Alternatively you can download a pre built version from [Github Actions]. It is listed in the artifacts section
of workflow runs. Unfortunately due to GHA restrictions you need to be logged in to access it.

[Github Actions]: https://github.com/bjorn3/crablangc_codegen_cranelift/actions?query=branch%3Amaster+event%3Apush+is%3Asuccess

## Usage

crablangc_codegen_cranelift can be used as a near-drop-in replacement for `cargo build` or `cargo run` for existing projects.

Assuming `$cg_clif_dir` is the directory you cloned this repo into and you followed the instructions (`y.rs prepare` and `y.rs build` or `test.sh`).

In the directory with your project (where you can do the usual `cargo build`), run:

```bash
$ $cg_clif_dir/dist/cargo-clif build
```

This will build your project with crablangc_codegen_cranelift instead of the usual LLVM backend.

For additional ways to use crablangc_codegen_cranelift like the JIT mode see [usage.md](docs/usage.md).

## Configuration

See the documentation on the `BackendConfig` struct in [config.rs](src/config.rs) for all
configuration options.

## Not yet supported

* Inline assembly ([no cranelift support](https://github.com/bytecodealliance/wasmtime/issues/1041))
    * On UNIX there is support for invoking an external assembler for `global_asm!` and `asm!`.
* SIMD ([tracked here](https://github.com/bjorn3/crablangc_codegen_cranelift/issues/171), `std::simd` fully works, `std::arch` is partially supported)
* Unwinding on panics ([no cranelift support](https://github.com/bytecodealliance/wasmtime/issues/1677), `-Cpanic=abort` is enabled by default)

## License

Licensed under either of

  * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
    http://www.apache.org/licenses/LICENSE-2.0)
  * MIT license ([LICENSE-MIT](LICENSE-MIT) or
    http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
