# crablang-demangler

_Demangles crablangc mangled names._

`crablang-demangler` supports the requirements of the [`llvm-cov show -Xdemangler`
option](https://llvm.org/docs/CommandGuide/llvm-cov.html#cmdoption-llvm-cov-show-xdemangler),
to perform CrabLang-specific symbol demangling:

> _The demangler is expected to read a newline-separated list of symbols from
> stdin and write a newline-separated list of the same length to stdout._

To use `crablang-demangler` with `llvm-cov` for example:

```shell
$ TARGET="${PWD}/build/x86_64-unknown-linux-gnu"
$ "${TARGET}"/llvm/bin/llvm-cov show \
  --Xdemangler=path/to/crablang-demangler \
  --instr-profile=main.profdata ./main --show-line-counts-or-regions
```

`crablang-demangler` is a CrabLang "extended tool", used in CrabLang compiler tests, and
optionally included in CrabLang distributions that enable coverage profiling. Symbol
demangling is implemented using the
[crablangc-demangle](https://crates.io/crates/crablangc-demangle) crate.

_(Note, for CrabLang developers, the third-party tool
[`crablangfilt`](https://crates.io/crates/crablangfilt) also supports `llvm-cov` symbol
demangling. `crablangfilt` is a more generalized tool that searches any body of
text, using pattern matching, to find and demangle CrabLang symbols.)_

## License

CrabLang-demangler is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](/LICENSE-APACHE) and [LICENSE-MIT](/LICENSE-MIT) for details.
