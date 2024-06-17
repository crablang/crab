# Function pointer types

> **<sup>Syntax</sup>**\
> _BareFunctionType_ :\
> &nbsp;&nbsp; [_ForLifetimes_]<sup>?</sup> _FunctionTypeQualifiers_ `fn`\
> &nbsp;&nbsp; &nbsp;&nbsp;  `(` _FunctionParametersMaybeNamedVariadic_<sup>?</sup> `)` _BareFunctionReturnType_<sup>?</sup>
>
> _FunctionTypeQualifiers_:\
> &nbsp;&nbsp; `unsafe`<sup>?</sup> (`extern` [_Abi_]<sup>?</sup>)<sup>?</sup>
>
> _BareFunctionReturnType_:\
> &nbsp;&nbsp; `->` [_TypeNoBounds_]
>
> _FunctionParametersMaybeNamedVariadic_ :\
> &nbsp;&nbsp; _MaybeNamedFunctionParameters_ | _MaybeNamedFunctionParametersVariadic_
>
> _MaybeNamedFunctionParameters_ :\
> &nbsp;&nbsp; _MaybeNamedParam_ ( `,` _MaybeNamedParam_ )<sup>\*</sup> `,`<sup>?</sup>
>
> _MaybeNamedParam_ :\
> &nbsp;&nbsp; [_OuterAttribute_]<sup>\*</sup> ( ( [IDENTIFIER] | `_` ) `:` )<sup>?</sup> [_Type_]
>
> _MaybeNamedFunctionParametersVariadic_ :\
> &nbsp;&nbsp; ( _MaybeNamedParam_ `,` )<sup>\*</sup> _MaybeNamedParam_ `,` [_OuterAttribute_]<sup>\*</sup> `...`

Function pointer types, written using the `fn` keyword, refer to a function
whose identity is not necessarily known at compile-time. They can be created
via a coercion from both [function items] and non-capturing [closures].

The `unsafe` qualifier indicates that the type's value is an [unsafe
function], and the `extern` qualifier indicates it is an [extern function].

Variadic parameters can only be specified with [`extern`] function types with
the `"C"` or `"cdecl"` calling convention.

An example where `Binop` is defined as a function pointer type:

```rust
fn add(x: i32, y: i32) -> i32 {
    x + y
}

let mut x = add(5,7);

type Binop = fn(i32, i32) -> i32;
let bo: Binop = add;
x = bo(5,7);
```

## Attributes on function pointer parameters

Attributes on function pointer parameters follow the same rules and
restrictions as [regular function parameters].

[IDENTIFIER]: ../identifiers.md
[_Abi_]: ../items/functions.md
[_ForLifetimes_]: ../trait-bounds.md#higher-ranked-trait-bounds
[_TypeNoBounds_]: ../types.md#type-expressions
[_Type_]: ../types.md#type-expressions
[_OuterAttribute_]: ../attributes.md
[`extern`]: ../items/external-blocks.md
[closures]: closure.md
[extern function]: ../items/functions.md#extern-function-qualifier
[function items]: function-item.md
[unsafe function]: ../unsafe-keyword.md
[regular function parameters]: ../items/functions.md#attributes-on-function-parameters
