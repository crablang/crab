# External blocks

> **<sup>Syntax</sup>**\
> _ExternBlock_ :\
> &nbsp;&nbsp; `unsafe`<sup>?</sup> `extern` [_Abi_]<sup>?</sup> `{`\
> &nbsp;&nbsp; &nbsp;&nbsp; [_InnerAttribute_]<sup>\*</sup>\
> &nbsp;&nbsp; &nbsp;&nbsp; _ExternalItem_<sup>\*</sup>\
> &nbsp;&nbsp; `}`
>
> _ExternalItem_ :\
> &nbsp;&nbsp; [_OuterAttribute_]<sup>\*</sup> (\
> &nbsp;&nbsp; &nbsp;&nbsp; &nbsp;&nbsp; [_MacroInvocationSemi_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | ( [_Visibility_]<sup>?</sup> ( [_StaticItem_] | [_Function_] ) )\
> &nbsp;&nbsp; )

External blocks provide _declarations_ of items that are not _defined_ in the
current crate and are the basis of Rust's foreign function interface. These are
akin to unchecked imports.

Two kinds of item _declarations_ are allowed in external blocks: [functions] and
[statics]. Calling functions or accessing statics that are declared in external
blocks is only allowed in an `unsafe` context.

The `unsafe` keyword is syntactically allowed to appear before the `extern`
keyword, but it is rejected at a semantic level. This allows macros to consume
the syntax and make use of the `unsafe` keyword, before removing it from the
token stream.

## Functions

Functions within external blocks are declared in the same way as other Rust
functions, with the exception that they must not have a body and are instead
terminated by a semicolon. Patterns are not allowed in parameters, only
[IDENTIFIER] or `_` may be used. Function qualifiers (`const`, `async`,
`unsafe`, and `extern`) are not allowed.

Functions within external blocks may be called by Rust code, just like
functions defined in Rust. The Rust compiler automatically translates between
the Rust ABI and the foreign ABI.

A function declared in an extern block is implicitly `unsafe`. When coerced to
a function pointer, a function declared in an extern block has type `unsafe
extern "abi" for<'l1, ..., 'lm> fn(A1, ..., An) -> R`, where `'l1`, ... `'lm`
are its lifetime parameters, `A1`, ..., `An` are the declared types of its
parameters and `R` is the declared return type.

## Statics

Statics within external blocks are declared in the same way as [statics] outside of external blocks,
except that they do not have an expression initializing their value.
It is `unsafe` to access a static item declared in an extern block, whether or
not it's mutable, because there is nothing guaranteeing that the bit pattern at the static's
memory is valid for the type it is declared with, since some arbitrary (e.g. C) code is in charge
of initializing the static.

Extern statics can be either immutable or mutable just like [statics] outside of external blocks.
An immutable static *must* be initialized before any Rust code is executed. It is not enough for
the static to be initialized before Rust code reads from it.

## ABI

By default external blocks assume that the library they are calling uses the
standard C ABI on the specific platform. Other ABIs may be specified using an
`abi` string, as shown here:

```rust
// Interface to the Windows API
extern "stdcall" { }
```

There are three ABI strings which are cross-platform, and which all compilers
are guaranteed to support:

* `extern "Rust"` -- The default ABI when you write a normal `fn foo()` in any
  Rust code.
* `extern "C"` -- This is the same as `extern fn foo()`; whatever the default
  your C compiler supports.
* `extern "system"` -- Usually the same as `extern "C"`, except on Win32, in
  which case it's `"stdcall"`, or what you should use to link to the Windows
  API itself

There are also some platform-specific ABI strings:

* `extern "cdecl"` -- The default for x86\_32 C code.
* `extern "stdcall"` -- The default for the Win32 API on x86\_32.
* `extern "win64"` -- The default for C code on x86\_64 Windows.
* `extern "sysv64"` -- The default for C code on non-Windows x86\_64.
* `extern "aapcs"` -- The default for ARM.
* `extern "fastcall"` -- The `fastcall` ABI -- corresponds to MSVC's
  `__fastcall` and GCC and clang's `__attribute__((fastcall))`
* `extern "vectorcall"` -- The `vectorcall` ABI -- corresponds to MSVC's
  `__vectorcall` and clang's `__attribute__((vectorcall))`
* `extern "thiscall"` -- The default for C++ member functions on MSVC -- corresponds to MSVC's
  `__thiscall` and GCC and clang's `__attribute__((thiscall))`
* `extern "efiapi"` -- The ABI used for [UEFI] functions.

## Variadic functions

Functions within external blocks may be variadic by specifying `...` as the
last argument. There must be at least one parameter before the variadic
parameter. The variadic parameter may optionally be specified with an
identifier.

```rust
extern "C" {
    fn foo(x: i32, ...);
    fn with_name(format: *const u8, args: ...);
}
```

## Attributes on extern blocks

The following [attributes] control the behavior of external blocks.

### The `link` attribute

The *`link` attribute* specifies the name of a native library that the
compiler should link with for the items within an `extern` block. It uses the
[_MetaListNameValueStr_] syntax to specify its inputs. The `name` key is the
name of the native library to link. The `kind` key is an optional value which
specifies the kind of library with the following possible values:

- `dylib` — Indicates a dynamic library. This is the default if `kind` is not
  specified.
- `static` — Indicates a static library.
- `framework` — Indicates a macOS framework. This is only valid for macOS
  targets.
- `raw-dylib` — Indicates a dynamic library where the compiler will generate
  an import library to link against (see [`dylib` versus `raw-dylib`] below
  for details). This is only valid for Windows targets.

The `name` key must be included if `kind` is specified.

The optional `modifiers` argument is a way to specify linking modifiers for the
library to link.
Modifiers are specified as a comma-delimited string with each modifier prefixed
with either a `+` or `-` to indicate that the modifier is enabled or disabled,
respectively.
Specifying multiple `modifiers` arguments in a single `link` attribute,
or multiple identical modifiers in the same `modifiers` argument is not currently supported. \
Example: `#[link(name = "mylib", kind = "static", modifiers = "+whole-archive")`.

The `wasm_import_module` key may be used to specify the [WebAssembly module]
name for the items within an `extern` block when importing symbols from the
host environment. The default module name is `env` if `wasm_import_module` is
not specified.

<!-- ignore: requires extern linking -->
```rust,ignore
#[link(name = "crypto")]
extern {
    // …
}

#[link(name = "CoreFoundation", kind = "framework")]
extern {
    // …
}

#[link(wasm_import_module = "foo")]
extern {
    // …
}
```

It is valid to add the `link` attribute on an empty extern block. You can use
this to satisfy the linking requirements of extern blocks elsewhere in your
code (including upstream crates) instead of adding the attribute to each extern
block.

#### Linking modifiers: `bundle`

This modifier is only compatible with the `static` linking kind.
Using any other kind will result in a compiler error.

When building a rlib or staticlib `+bundle` means that the native static library
will be packed into the rlib or staticlib archive, and then retrieved from there
during linking of the final binary.

When building a rlib `-bundle` means that the native static library is registered as a dependency
of that rlib "by name", and object files from it are included only during linking of the final
binary, the file search by that name is also performed during final linking. \
When building a staticlib `-bundle` means that the native static library is simply not included
into the archive and some higher level build system will need to add it later during linking of
the final binary.

This modifier has no effect when building other targets like executables or dynamic libraries.

The default for this modifier is `+bundle`.

More implementation details about this modifier can be found in
[`bundle` documentation for rustc].

#### Linking modifiers: `whole-archive`

This modifier is only compatible with the `static` linking kind.
Using any other kind will result in a compiler error.

`+whole-archive` means that the static library is linked as a whole archive
without throwing any object files away.

The default for this modifier is `-whole-archive`.

More implementation details about this modifier can be found in
[`whole-archive` documentation for rustc].

### Linking modifiers: `verbatim`

This modifier is compatible with all linking kinds.

`+verbatim` means that rustc itself won't add any target-specified library prefixes or suffixes
(like `lib` or `.a`) to the library name, and will try its best to ask for the same thing from the
linker.

`-verbatim` means that rustc will either add a target-specific prefix and suffix to the library
name before passing it to linker, or won't prevent linker from implicitly adding it.

The default for this modifier is `-verbatim`.

More implementation details about this modifier can be found in
[`verbatim` documentation for rustc].

#### `dylib` versus `raw-dylib`

On Windows, linking against a dynamic library requires that an import library
is provided to the linker: this is a special static library that declares all
of the symbols exported by the dynamic library in such a way that the linker
knows that they have to be dynamically loaded at runtime.

Specifying `kind = "dylib"` instructs the Rust compiler to link an import
library based on the `name` key. The linker will then use its normal library
resolution logic to find that import library. Alternatively, specifying
`kind = "raw-dylib"` instructs the compiler to generate an import library
during compilation and provide that to the linker instead.

`raw-dylib` is only supported on Windows. Using it when targeting other
platforms will result in a compiler error.

#### The `import_name_type` key

On x86 Windows, names of functions are "decorated" (i.e., have a specific prefix
and/or suffix added) to indicate their calling convention. For example, a
`stdcall` calling convention function with the name `fn1` that has no arguments
would be decorated as `_fn1@0`. However, the [PE Format] does also permit names
to have no prefix or be undecorated. Additionally, the MSVC and GNU toolchains
use different decorations for the same calling conventions which means, by
default, some Win32 functions cannot be called using the `raw-dylib` link kind
via the GNU toolchain.

To allow for these differences, when using the `raw-dylib` link kind you may
also specify the `import_name_type` key with one of the following values to
change how functions are named in the generated import library:

* `decorated`: The function name will be fully-decorated using the MSVC
  toolchain format.
* `noprefix`: The function name will be decorated using the MSVC toolchain
  format, but skipping the leading `?`, `@`, or optionally `_`.
* `undecorated`: The function name will not be decorated.

If the `import_name_type` key is not specified, then the function name will be
fully-decorated using the target toolchain's format.

Variables are never decorated and so the `import_name_type` key has no effect on
how they are named in the generated import library.

The `import_name_type` key is only supported on x86 Windows. Using it when
targeting other platforms will result in a compiler error.

### The `link_name` attribute

The *`link_name` attribute* may be specified on declarations inside an `extern`
block to indicate the symbol to import for the given function or static. It
uses the [_MetaNameValueStr_] syntax to specify the name of the symbol.

```rust
extern {
    #[link_name = "actual_symbol_name"]
    fn name_in_rust();
}
```

Using this attribute with the `link_ordinal` attribute will result in a
compiler error.

### The `link_ordinal` attribute

The *`link_ordinal` attribute* can be applied on declarations inside an `extern`
block to indicate the numeric ordinal to use when generating the import library
to link against. An ordinal is a unique number per symbol exported by a dynamic
library on Windows and can be used when the library is being loaded to find
that symbol rather than having to look it up by name.

<div class="warning">

Warning: `link_ordinal` should only be used in cases where the ordinal of the
symbol is known to be stable: if the ordinal of a symbol is not explicitly set
when its containing binary is built then one will be automatically assigned to
it, and that assigned ordinal may change between builds of the binary.

</div>

<!-- ignore: Only works on x86 Windows -->
```rust,ignore
#[link(name = "exporter", kind = "raw-dylib")]
extern "stdcall" {
    #[link_ordinal(15)]
    fn imported_function_stdcall(i: i32);
}
```

This attribute is only used with the `raw-dylib` linking kind.
Using any other kind will result in a compiler error.

Using this attribute with the `link_name` attribute will result in a
compiler error.

### Attributes on function parameters

Attributes on extern function parameters follow the same rules and
restrictions as [regular function parameters].

[IDENTIFIER]: ../identifiers.md
[UEFI]: https://uefi.org/specifications
[WebAssembly module]: https://webassembly.github.io/spec/core/syntax/modules.html
[functions]: functions.md
[statics]: static-items.md
[_Abi_]: functions.md
[_Function_]: functions.md
[_InnerAttribute_]: ../attributes.md
[_MacroInvocationSemi_]: ../macros.md#macro-invocation
[_MetaListNameValueStr_]: ../attributes.md#meta-item-attribute-syntax
[_MetaNameValueStr_]: ../attributes.md#meta-item-attribute-syntax
[_OuterAttribute_]: ../attributes.md
[_StaticItem_]: static-items.md
[_Visibility_]: ../visibility-and-privacy.md
[attributes]: ../attributes.md
[regular function parameters]: functions.md#attributes-on-function-parameters
[`bundle` documentation for rustc]: ../../rustc/command-line-arguments.html#linking-modifiers-bundle
[`whole-archive` documentation for rustc]: ../../rustc/command-line-arguments.html#linking-modifiers-whole-archive
[`verbatim` documentation for rustc]: ../../rustc/command-line-arguments.html#linking-modifiers-verbatim
[`dylib` versus `raw-dylib`]: #dylib-versus-raw-dylib
[PE Format]: https://learn.microsoft.com/windows/win32/debug/pe-format#import-name-type
