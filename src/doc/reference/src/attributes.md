{{#include attributes-redirect.html}}
# Attributes

> **<sup>Syntax</sup>**\
> _InnerAttribute_ :\
> &nbsp;&nbsp; `#` `!` `[` _Attr_ `]`
>
> _OuterAttribute_ :\
> &nbsp;&nbsp; `#` `[` _Attr_ `]`
>
> _Attr_ :\
> &nbsp;&nbsp; [_SimplePath_] _AttrInput_<sup>?</sup>
>
> _AttrInput_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_DelimTokenTree_]\
> &nbsp;&nbsp; | `=` [_Expression_]

An _attribute_ is a general, free-form metadatum that is interpreted according
to name, convention, language, and compiler version. Attributes are modeled
on Attributes in [ECMA-335], with the syntax coming from [ECMA-334] \(C#).

_Inner attributes_, written with a bang (`!`) after the hash (`#`), apply to the
item that the attribute is declared within. _Outer attributes_, written without
the bang after the hash, apply to the thing that follows the attribute.

The attribute consists of a path to the attribute, followed by an optional
delimited token tree whose interpretation is defined by the attribute.
Attributes other than macro attributes also allow the input to be an equals
sign (`=`) followed by an expression. See the [meta item
syntax](#meta-item-attribute-syntax) below for more details.

Attributes can be classified into the following kinds:

* [Built-in attributes]
* [Macro attributes][attribute macros]
* [Derive macro helper attributes]
* [Tool attributes](#tool-attributes)

Attributes may be applied to many things in the language:

* All [item declarations] accept outer attributes while [external blocks],
  [functions], [implementations], and [modules] accept inner attributes.
* Most [statements] accept outer attributes (see [Expression Attributes] for
  limitations on expression statements).
* [Block expressions] accept outer and inner attributes, but only when they are
  the outer expression of an [expression statement] or the final expression of
  another block expression.
* [Enum] variants and [struct] and [union] fields accept outer attributes.
* [Match expression arms][match expressions] accept outer attributes.
* [Generic lifetime or type parameter][generics] accept outer attributes.
* Expressions accept outer attributes in limited situations, see [Expression
  Attributes] for details.
* [Function][functions], [closure] and [function pointer]
  parameters accept outer attributes. This includes attributes on variadic parameters
  denoted with `...` in function pointers and [external blocks][variadic functions].

Some examples of attributes:

```rust
// General metadata applied to the enclosing module or crate.
#![crate_type = "lib"]

// A function marked as a unit test
#[test]
fn test_foo() {
    /* ... */
}

// A conditionally-compiled module
#[cfg(target_os = "linux")]
mod bar {
    /* ... */
}

// A lint attribute used to suppress a warning/error
#[allow(non_camel_case_types)]
type int8_t = i8;

// Inner attribute applies to the entire function.
fn some_unused_variables() {
  #![allow(unused_variables)]

  let x = ();
  let y = ();
  let z = ();
}
```

## Meta Item Attribute Syntax

A "meta item" is the syntax used for the _Attr_ rule by most [built-in
attributes]. It has the following grammar:

> **<sup>Syntax</sup>**\
> _MetaItem_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SimplePath_]\
> &nbsp;&nbsp; | [_SimplePath_] `=` [_Expression_]\
> &nbsp;&nbsp; | [_SimplePath_] `(` _MetaSeq_<sup>?</sup> `)`
>
> _MetaSeq_ :\
> &nbsp;&nbsp; _MetaItemInner_ ( `,` MetaItemInner )<sup>\*</sup> `,`<sup>?</sup>
>
> _MetaItemInner_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _MetaItem_\
> &nbsp;&nbsp; | [_Expression_]

Expressions in meta items must macro-expand to literal expressions, which must not
include integer or float type suffixes. Expressions which are not literal expressions
will be syntactically accepted (and can be passed to proc-macros), but will be rejected after parsing.

Note that if the attribute appears within another macro, it will be expanded
after that outer macro. For example, the following code will expand the
`Serialize` proc-macro first, which must preserve the `include_str!` call in
order for it to be expanded:

```rust ignore
#[derive(Serialize)]
struct Foo {
    #[doc = include_str!("x.md")]
    x: u32
}
```

Additionally, macros in attributes will be expanded only after all other attributes applied to the item:

```rust ignore
#[macro_attr1] // expanded first
#[doc = mac!()] // `mac!` is expanded fourth.
#[macro_attr2] // expanded second
#[derive(MacroDerive1, MacroDerive2)] // expanded third
fn foo() {}
```

Various built-in attributes use different subsets of the meta item syntax to
specify their inputs. The following grammar rules show some commonly used
forms:

> **<sup>Syntax</sup>**\
> _MetaWord_:\
> &nbsp;&nbsp; [IDENTIFIER]
>
> _MetaNameValueStr_:\
> &nbsp;&nbsp; [IDENTIFIER] `=` ([STRING_LITERAL] | [RAW_STRING_LITERAL])
>
> _MetaListPaths_:\
> &nbsp;&nbsp; [IDENTIFIER] `(` ( [_SimplePath_] (`,` [_SimplePath_])* `,`<sup>?</sup> )<sup>?</sup> `)`
>
> _MetaListIdents_:\
> &nbsp;&nbsp; [IDENTIFIER] `(` ( [IDENTIFIER] (`,` [IDENTIFIER])* `,`<sup>?</sup> )<sup>?</sup> `)`
>
> _MetaListNameValueStr_:\
> &nbsp;&nbsp; [IDENTIFIER] `(` ( _MetaNameValueStr_ (`,` _MetaNameValueStr_)* `,`<sup>?</sup> )<sup>?</sup> `)`

Some examples of meta items are:

Style | Example
------|--------
_MetaWord_ | `no_std`
_MetaNameValueStr_ | `doc = "example"`
_MetaListPaths_ | `allow(unused, clippy::inline_always)`
_MetaListIdents_ | `macro_use(foo, bar)`
_MetaListNameValueStr_ | `link(name = "CoreFoundation", kind = "framework")`

## Active and inert attributes

An attribute is either active or inert. During attribute processing, *active
attributes* remove themselves from the thing they are on while *inert attributes*
stay on.

The [`cfg`] and [`cfg_attr`] attributes are active. The [`test`] attribute is
inert when compiling for tests and active otherwise. [Attribute macros] are
active. All other attributes are inert.

## Tool attributes

The compiler may allow attributes for external tools where each tool resides
in its own namespace in the [tool prelude]. The first segment of the attribute
path is the name of the tool, with one or more additional segments whose
interpretation is up to the tool.

When a tool is not in use, the tool's attributes are accepted without a
warning. When the tool is in use, the tool is responsible for processing and
interpretation of its attributes.

Tool attributes are not available if the [`no_implicit_prelude`] attribute is
used.

```rust
// Tells the rustfmt tool to not format the following element.
#[rustfmt::skip]
struct S {
}

// Controls the "cyclomatic complexity" threshold for the clippy tool.
#[clippy::cyclomatic_complexity = "100"]
pub fn f() {}
```

> Note: `rustc` currently recognizes the tools "clippy" and "rustfmt".

## Built-in attributes index

The following is an index of all built-in attributes.

- Conditional compilation
  - [`cfg`] — Controls conditional compilation.
  - [`cfg_attr`] — Conditionally includes attributes.
- Testing
  - [`test`] — Marks a function as a test.
  - [`ignore`] — Disables a test function.
  - [`should_panic`] — Indicates a test should generate a panic.
- Derive
  - [`derive`] — Automatic trait implementations.
  - [`automatically_derived`] — Marker for implementations created by
    `derive`.
- Macros
  - [`macro_export`] — Exports a `macro_rules` macro for cross-crate usage.
  - [`macro_use`] — Expands macro visibility, or imports macros from other
    crates.
  - [`proc_macro`] — Defines a function-like macro.
  - [`proc_macro_derive`] — Defines a derive macro.
  - [`proc_macro_attribute`] — Defines an attribute macro.
- Diagnostics
  - [`allow`], [`warn`], [`deny`], [`forbid`] — Alters the default lint level.
  - [`deprecated`] — Generates deprecation notices.
  - [`must_use`] — Generates a lint for unused values.
- ABI, linking, symbols, and FFI
  - [`link`] — Specifies a native library to link with an `extern` block.
  - [`link_name`] — Specifies the name of the symbol for functions or statics
    in an `extern` block.
  - [`link_ordinal`] — Specifies the ordinal of the symbol for functions or
    statics in an `extern` block.
  - [`no_link`] — Prevents linking an extern crate.
  - [`repr`] — Controls type layout.
  - [`crate_type`] — Specifies the type of crate (library, executable, etc.).
  - [`no_main`] — Disables emitting the `main` symbol.
  - [`export_name`] — Specifies the exported symbol name for a function or
    static.
  - [`link_section`] — Specifies the section of an object file to use for a
    function or static.
  - [`no_mangle`] — Disables symbol name encoding.
  - [`used`] — Forces the compiler to keep a static item in the output
    object file.
  - [`crate_name`] — Specifies the crate name.
- Code generation
  - [`inline`] — Hint to inline code.
  - [`cold`] — Hint that a function is unlikely to be called.
  - [`no_builtins`] — Disables use of certain built-in functions.
  - [`target_feature`] — Configure platform-specific code generation.
  - [`track_caller`] - Pass the parent call location to `std::panic::Location::caller()`.
  - [`instruction_set`] - Specify the instruction set used to generate a functions code
- Documentation
  - `doc` — Specifies documentation. See [The Rustdoc Book] for more
    information. [Doc comments] are transformed into `doc` attributes.
- Preludes
  - [`no_std`] — Removes std from the prelude.
  - [`no_implicit_prelude`] — Disables prelude lookups within a module.
- Modules
  - [`path`] — Specifies the filename for a module.
- Limits
  - [`recursion_limit`] — Sets the maximum recursion limit for certain
    compile-time operations.
  - [`type_length_limit`] — Sets the maximum size of a polymorphic type.
- Runtime
  - [`panic_handler`] — Sets the function to handle panics.
  - [`global_allocator`] — Sets the global memory allocator.
  - [`windows_subsystem`] — Specifies the windows subsystem to link with.
- Features
  - `feature` — Used to enable unstable or experimental compiler features. See
    [The Unstable Book] for features implemented in `rustc`.
- Type System
  - [`non_exhaustive`] — Indicate that a type will have more fields/variants
    added in future.
- Debugger
  - [`debugger_visualizer`] — Embeds a file that specifies debugger output for a type.

[Doc comments]: comments.md#doc-comments
[ECMA-334]: https://www.ecma-international.org/publications-and-standards/standards/ecma-334/
[ECMA-335]: https://www.ecma-international.org/publications-and-standards/standards/ecma-335/
[Expression Attributes]: expressions.md#expression-attributes
[IDENTIFIER]: identifiers.md
[RAW_STRING_LITERAL]: tokens.md#raw-string-literals
[STRING_LITERAL]: tokens.md#string-literals
[The Rustdoc Book]: ../rustdoc/the-doc-attribute.html
[The Unstable Book]: ../unstable-book/index.html
[_DelimTokenTree_]: macros.md
[_Expression_]: expressions.md
[_SimplePath_]: paths.md#simple-paths
[`allow`]: attributes/diagnostics.md#lint-check-attributes
[`automatically_derived`]: attributes/derive.md#the-automatically_derived-attribute
[`cfg_attr`]: conditional-compilation.md#the-cfg_attr-attribute
[`cfg`]: conditional-compilation.md#the-cfg-attribute
[`cold`]: attributes/codegen.md#the-cold-attribute
[`crate_name`]: crates-and-source-files.md#the-crate_name-attribute
[`crate_type`]: linkage.md
[`debugger_visualizer`]: attributes/debugger.md#the-debugger_visualizer-attribute
[`deny`]: attributes/diagnostics.md#lint-check-attributes
[`deprecated`]: attributes/diagnostics.md#the-deprecated-attribute
[`derive`]: attributes/derive.md
[`export_name`]: abi.md#the-export_name-attribute
[`forbid`]: attributes/diagnostics.md#lint-check-attributes
[`global_allocator`]: runtime.md#the-global_allocator-attribute
[`ignore`]: attributes/testing.md#the-ignore-attribute
[`inline`]: attributes/codegen.md#the-inline-attribute
[`instruction_set`]: attributes/codegen.md#the-instruction_set-attribute
[`link_name`]: items/external-blocks.md#the-link_name-attribute
[`link_ordinal`]: items/external-blocks.md#the-link_ordinal-attribute
[`link_section`]: abi.md#the-link_section-attribute
[`link`]: items/external-blocks.md#the-link-attribute
[`macro_export`]: macros-by-example.md#path-based-scope
[`macro_use`]: macros-by-example.md#the-macro_use-attribute
[`must_use`]: attributes/diagnostics.md#the-must_use-attribute
[`no_builtins`]: attributes/codegen.md#the-no_builtins-attribute
[`no_implicit_prelude`]: names/preludes.md#the-no_implicit_prelude-attribute
[`no_link`]: items/extern-crates.md#the-no_link-attribute
[`no_main`]: crates-and-source-files.md#the-no_main-attribute
[`no_mangle`]: abi.md#the-no_mangle-attribute
[`no_std`]: names/preludes.md#the-no_std-attribute
[`non_exhaustive`]: attributes/type_system.md#the-non_exhaustive-attribute
[`panic_handler`]: runtime.md#the-panic_handler-attribute
[`path`]: items/modules.md#the-path-attribute
[`proc_macro_attribute`]: procedural-macros.md#attribute-macros
[`proc_macro_derive`]: procedural-macros.md#derive-macros
[`proc_macro`]: procedural-macros.md#function-like-procedural-macros
[`recursion_limit`]: attributes/limits.md#the-recursion_limit-attribute
[`repr`]: type-layout.md#representations
[`should_panic`]: attributes/testing.md#the-should_panic-attribute
[`target_feature`]: attributes/codegen.md#the-target_feature-attribute
[`test`]: attributes/testing.md#the-test-attribute
[`track_caller`]: attributes/codegen.md#the-track_caller-attribute
[`type_length_limit`]: attributes/limits.md#the-type_length_limit-attribute
[`used`]: abi.md#the-used-attribute
[`warn`]: attributes/diagnostics.md#lint-check-attributes
[`windows_subsystem`]: runtime.md#the-windows_subsystem-attribute
[attribute macros]: procedural-macros.md#attribute-macros
[block expressions]: expressions/block-expr.md
[built-in attributes]: #built-in-attributes-index
[derive macro helper attributes]: procedural-macros.md#derive-macro-helper-attributes
[enum]: items/enumerations.md
[expression statement]: statements.md#expression-statements
[external blocks]: items/external-blocks.md
[functions]: items/functions.md
[generics]: items/generics.md
[implementations]: items/implementations.md
[item declarations]: items.md
[match expressions]: expressions/match-expr.md
[modules]: items/modules.md
[statements]: statements.md
[struct]: items/structs.md
[tool prelude]: names/preludes.md#tool-prelude
[union]: items/unions.md
[closure]: expressions/closure-expr.md
[function pointer]: types/function-pointer.md
[variadic functions]: items/external-blocks.html#variadic-functions
