# Conditional compilation

> **<sup>Syntax</sup>**\
> _ConfigurationPredicate_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _ConfigurationOption_\
> &nbsp;&nbsp; | _ConfigurationAll_\
> &nbsp;&nbsp; | _ConfigurationAny_\
> &nbsp;&nbsp; | _ConfigurationNot_
>
> _ConfigurationOption_ :\
> &nbsp;&nbsp; [IDENTIFIER]&nbsp;(`=` ([STRING_LITERAL] | [RAW_STRING_LITERAL]))<sup>?</sup>
>
> _ConfigurationAll_\
> &nbsp;&nbsp; `all` `(` _ConfigurationPredicateList_<sup>?</sup> `)`
>
> _ConfigurationAny_\
> &nbsp;&nbsp; `any` `(` _ConfigurationPredicateList_<sup>?</sup> `)`
>
> _ConfigurationNot_\
> &nbsp;&nbsp; `not` `(` _ConfigurationPredicate_ `)`
>
> _ConfigurationPredicateList_\
> &nbsp;&nbsp; _ConfigurationPredicate_ (`,` _ConfigurationPredicate_)<sup>\*</sup> `,`<sup>?</sup>

*Conditionally compiled source code* is source code that is compiled only under certain conditions. Source code can be made conditionally compiled using the [`cfg`] and [`cfg_attr`] [attributes] and the built-in [`cfg` macro]. Whether to compile can depend on the target architecture of the compiled crate, arbitrary values passed to the compiler, and other things further described below.

Each form of conditional compilation takes a _configuration predicate_ that
evaluates to true or false. The predicate is one of the following:

* A configuration option. The predicate is true if the option is set, and false if it is unset.
* `all()` with a comma-separated list of configuration predicates. It is true if all of the given predicates are true, or if the list is empty.
* `any()` with a comma-separated list of configuration predicates. It is true if at least one of the given predicates is true. If there are no predicates, it is false.
* `not()` with a configuration predicate. It is true if its predicate is false and false if its predicate is true.

_Configuration options_ are either names or key-value pairs, and are either set or unset.
Names are written as a single identifier, such as `unix`.
Key-value pairs are written as an identifier, `=`, and then a string, such as `target_arch = "x86_64"`.

> **Note**: Whitespace around the `=` is ignored, so `foo="bar"` and `foo = "bar"` are equivalent.

Keys do not need to be unique. For example, both `feature = "std"` and `feature = "serde"` can be set at the same time.

## Set Configuration Options

Which configuration options are set is determined statically during the
compilation of the crate. Some options are _compiler-set_ based on data
about the compilation. Other options are _arbitrarily-set_ based on input
passed to the compiler outside of the code. It is not possible to set a
configuration option from within the source code of the crate being compiled.

> **Note**: For `rustc`, arbitrary-set configuration options are set using the
> [`--cfg`] flag.

> **Note**: Configuration options with the key `feature` are a convention used
> by [Cargo][cargo-feature] for specifying compile-time options and optional
> dependencies.

<div class="warning">

Warning: Arbitrarily-set configuration options can clash with compiler-set configuration options. For example, it is possible to do `rustc --cfg "unix" program.rs` while compiling to a Windows target, and have both `unix` and `windows` configuration options set at the same time. Doing this would be unwise.

</div>

### `target_arch`

Key-value option set once with the target's CPU architecture. The value is
similar to the first element of the platform's target triple, but not
identical.

Example values:

* `"x86"`
* `"x86_64"`
* `"mips"`
* `"powerpc"`
* `"powerpc64"`
* `"arm"`
* `"aarch64"`

### `target_feature`

Key-value option set for each platform feature available for the current
compilation target.

Example values:

* `"avx"`
* `"avx2"`
* `"crt-static"`
* `"rdrand"`
* `"sse"`
* `"sse2"`
* `"sse4.1"`

See the [`target_feature` attribute] for more details on the available
features. An additional feature of `crt-static` is available to the
`target_feature` option to indicate that a [static C runtime] is available.

### `target_os`

Key-value option set once with the target's operating system. This value is
similar to the second and third element of the platform's target triple.

Example values:

* `"windows"`
* `"macos"`
* `"ios"`
* `"linux"`
* `"android"`
* `"freebsd"`
* `"dragonfly"`
* `"openbsd"`
* `"netbsd"`
* `"none"` (typical for embedded targets)

### `target_family`

Key-value option providing a more generic description of a target, such as the family of the
operating systems or architectures that the target generally falls into. Any number of
`target_family` key-value pairs can be set.

Example values:

* `"unix"`
* `"windows"`
* `"wasm"`
* Both `"unix"` and `"wasm"`

### `unix` and `windows`

`unix` is set if `target_family = "unix"` is set and `windows` is set if
`target_family = "windows"` is set.

### `target_env`

Key-value option set with further disambiguating information about the target
platform with information about the ABI or `libc` used. For historical reasons,
this value is only defined as not the empty-string when actually needed for
disambiguation. Thus, for example, on many GNU platforms, this value will be
empty. This value is similar to the fourth element of the platform's target
triple. One difference is that embedded ABIs such as `gnueabihf` will simply
define `target_env` as `"gnu"`.

Example values:

* `""`
* `"gnu"`
* `"msvc"`
* `"musl"`
* `"sgx"`

### `target_abi`

Key-value option set to further disambiguate the `target_env` with information
about the target ABI. For historical reasons,
this value is only defined as not the empty-string when actually needed for
disambiguation. Thus, for example, on many GNU platforms, this value will be
empty.

Example values:

* `""`
* `"llvm"`
* `"eabihf"`
* `"abi64"`
* `"sim"`
* `"macabi"`

### `target_endian`

Key-value option set once with either a value of "little" or "big" depending
on the endianness of the target's CPU.

### `target_pointer_width`

Key-value option set once with the target's pointer width in bits.

Example values:

* `"16"`
* `"32"`
* `"64"`

### `target_vendor`

Key-value option set once with the vendor of the target.

Example values:

* `"apple"`
* `"fortanix"`
* `"pc"`
* `"unknown"`

### `target_has_atomic`

Key-value option set for each bit width that the target supports
atomic loads, stores, and compare-and-swap operations.

When this cfg is present, all of the stable [`core::sync::atomic`] APIs are available for
the relevant atomic width.

[`core::sync::atomic`]: ../core/sync/atomic/index.html

Possible values:

* `"8"`
* `"16"`
* `"32"`
* `"64"`
* `"128"`
* `"ptr"`

### `test`

Enabled when compiling the test harness. Done with `rustc` by using the
[`--test`] flag. See [Testing] for more on testing support.

### `debug_assertions`

Enabled by default when compiling without optimizations.
This can be used to enable extra debugging code in development but not in
production.  For example, it controls the behavior of the standard library's
[`debug_assert!`] macro.

### `proc_macro`

Set when the crate being compiled is being compiled with the `proc_macro`
[crate type].

### `panic`

Key-value option set depending on the panic strategy. Note that more values may be added in the future.

Example values:

* `"abort"`
* `"unwind"`

## Forms of conditional compilation

### The `cfg` attribute

> **<sup>Syntax</sup>**\
> _CfgAttrAttribute_ :\
> &nbsp;&nbsp; `cfg` `(` _ConfigurationPredicate_ `)`

<!-- should we say they're active attributes here? -->

The `cfg` [attribute] conditionally includes the thing it is attached to based
on a configuration predicate.

It is written as `cfg`, `(`, a configuration predicate, and finally `)`.

If the predicate is true, the thing is rewritten to not have the `cfg` attribute
on it. If the predicate is false, the thing is removed from the source code.

When a crate-level `cfg` has a false predicate, the behavior is slightly
different: any crate attributes preceding the `cfg` are kept, and any crate
attributes following the `cfg` are removed. This allows `#![no_std]` and
`#![no_core]` crates to avoid linking `std`/`core` even if a `#![cfg(...)]` has
removed the entire crate.

Some examples on functions:

```rust
// The function is only included in the build when compiling for macOS
#[cfg(target_os = "macos")]
fn macos_only() {
  // ...
}

// This function is only included when either foo or bar is defined
#[cfg(any(foo, bar))]
fn needs_foo_or_bar() {
  // ...
}

// This function is only included when compiling for a unixish OS with a 32-bit
// architecture
#[cfg(all(unix, target_pointer_width = "32"))]
fn on_32bit_unix() {
  // ...
}

// This function is only included when foo is not defined
#[cfg(not(foo))]
fn needs_not_foo() {
  // ...
}

// This function is only included when the panic strategy is set to unwind
#[cfg(panic = "unwind")]
fn when_unwinding() {
  // ...
}

```

The `cfg` attribute is allowed anywhere attributes are allowed.

### The `cfg_attr` attribute

> **<sup>Syntax</sup>**\
> _CfgAttrAttribute_ :\
> &nbsp;&nbsp; `cfg_attr` `(` _ConfigurationPredicate_ `,` _CfgAttrs_<sup>?</sup> `)`
>
> _CfgAttrs_ :\
> &nbsp;&nbsp; [_Attr_]&nbsp;(`,` [_Attr_])<sup>\*</sup> `,`<sup>?</sup>

The `cfg_attr` [attribute] conditionally includes [attributes] based on a
configuration predicate.

When the configuration predicate is true, this attribute expands out to the
attributes listed after the predicate. For example, the following module will
either be found at `linux.rs` or `windows.rs` based on the target.

<!-- ignore: `mod` needs multiple files -->
```rust,ignore
#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(windows, path = "windows.rs")]
mod os;
```

Zero, one, or more attributes may be listed. Multiple attributes will each be
expanded into separate attributes. For example:

<!-- ignore: fake attributes -->
```rust,ignore
#[cfg_attr(feature = "magic", sparkles, crackles)]
fn bewitched() {}

// When the `magic` feature flag is enabled, the above will expand to:
#[sparkles]
#[crackles]
fn bewitched() {}
```

> **Note**: The `cfg_attr` can expand to another `cfg_attr`. For example,
> `#[cfg_attr(target_os = "linux", cfg_attr(feature = "multithreaded", some_other_attribute))]`
> is valid. This example would be equivalent to
> `#[cfg_attr(all(target_os = "linux", feature ="multithreaded"), some_other_attribute)]`.

The `cfg_attr` attribute is allowed anywhere attributes are allowed.

### The `cfg` macro

The built-in `cfg` macro takes in a single configuration predicate and evaluates
to the `true` literal when the predicate is true and the `false` literal when
it is false.

For example:

```rust
let machine_kind = if cfg!(unix) {
  "unix"
} else if cfg!(windows) {
  "windows"
} else {
  "unknown"
};

println!("I'm running on a {} machine!", machine_kind);
```

[IDENTIFIER]: identifiers.md
[RAW_STRING_LITERAL]: tokens.md#raw-string-literals
[STRING_LITERAL]: tokens.md#string-literals
[Testing]: attributes/testing.md
[_Attr_]: attributes.md
[`--cfg`]: ../rustc/command-line-arguments.html#--cfg-configure-the-compilation-environment
[`--test`]: ../rustc/command-line-arguments.html#--test-build-a-test-harness
[`cfg`]: #the-cfg-attribute
[`cfg` macro]: #the-cfg-macro
[`cfg_attr`]: #the-cfg_attr-attribute
[`debug_assert!`]: ../std/macro.debug_assert.html
[`target_feature` attribute]: attributes/codegen.md#the-target_feature-attribute
[attribute]: attributes.md
[attributes]: attributes.md
[cargo-feature]: ../cargo/reference/features.html
[crate type]: linkage.md
[static C runtime]: linkage.md#static-and-dynamic-c-runtimes
