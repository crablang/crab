# Functions

> **<sup>Syntax</sup>**\
> _Function_ :\
> &nbsp;&nbsp; _FunctionQualifiers_ `fn` [IDENTIFIER]&nbsp;[_GenericParams_]<sup>?</sup>\
> &nbsp;&nbsp; &nbsp;&nbsp; `(` _FunctionParameters_<sup>?</sup> `)`\
> &nbsp;&nbsp; &nbsp;&nbsp; _FunctionReturnType_<sup>?</sup> [_WhereClause_]<sup>?</sup>\
> &nbsp;&nbsp; &nbsp;&nbsp; ( [_BlockExpression_] | `;` )
>
> _FunctionQualifiers_ :\
> &nbsp;&nbsp; `const`<sup>?</sup> `async`[^async-edition]<sup>?</sup> `unsafe`<sup>?</sup> (`extern` _Abi_<sup>?</sup>)<sup>?</sup>
>
> _Abi_ :\
> &nbsp;&nbsp; [STRING_LITERAL] | [RAW_STRING_LITERAL]
>
> _FunctionParameters_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _SelfParam_ `,`<sup>?</sup>\
> &nbsp;&nbsp; | (_SelfParam_ `,`)<sup>?</sup> _FunctionParam_ (`,` _FunctionParam_)<sup>\*</sup> `,`<sup>?</sup>
>
> _SelfParam_ :\
> &nbsp;&nbsp; [_OuterAttribute_]<sup>\*</sup> ( _ShorthandSelf_ | _TypedSelf_ )
>
> _ShorthandSelf_ :\
> &nbsp;&nbsp;  (`&` | `&` [_Lifetime_])<sup>?</sup> `mut`<sup>?</sup> `self`
>
> _TypedSelf_ :\
> &nbsp;&nbsp; `mut`<sup>?</sup> `self` `:` [_Type_]
>
> _FunctionParam_ :\
> &nbsp;&nbsp; [_OuterAttribute_]<sup>\*</sup> (
>   _FunctionParamPattern_ | `...` | [_Type_] [^fn-param-2015]
> )
>
> _FunctionParamPattern_ :\
> &nbsp;&nbsp; [_PatternNoTopAlt_] `:` ( [_Type_] | `...` )
>
> _FunctionReturnType_ :\
> &nbsp;&nbsp; `->` [_Type_]
>
> [^async-edition]: The `async` qualifier is not allowed in the 2015 edition.
>
> [^fn-param-2015]: Function parameters with only a type are only allowed
>   in an associated function of a [trait item] in the 2015 edition.

A _function_ consists of a [block], along with a name, a set of parameters, and an output type.
Other than a name, all these are optional.
Functions are declared with the keyword `fn`.
Functions may declare a set of *input* [*variables*][variables] as parameters, through which the caller passes arguments into the function, and the *output* [*type*][type] of the value the function will return to its caller on completion.
If the output type is not explicitly stated, it is the [unit type].

When referred to, a _function_ yields a first-class *value* of the corresponding zero-sized [*function item type*], which when called evaluates to a direct call to the function.

For example, this is a simple function:
```rust
fn answer_to_life_the_universe_and_everything() -> i32 {
    return 42;
}
```

## Function parameters

Function parameters are irrefutable [patterns], so any pattern that is valid in
an else-less `let` binding is also valid as a parameter:

```rust
fn first((value, _): (i32, i32)) -> i32 { value }
```

If the first parameter is a _SelfParam_, this indicates that the function is a
[method]. Functions with a self parameter may only appear as an [associated
function] in a [trait] or [implementation].

A parameter with the `...` token indicates a [variadic function], and may only
be used as the last parameter of an [external block] function. The variadic
parameter may have an optional identifier, such as `args: ...`.

## Function body

The block of a function is conceptually wrapped in a block that binds the
argument patterns and then `return`s the value of the function's block. This
means that the tail expression of the block, if evaluated, ends up being
returned to the caller. As usual, an explicit return expression within
the body of the function will short-cut that implicit return, if reached.

For example, the function above behaves as if it was written as:

<!-- ignore: example expansion -->
```rust,ignore
// argument_0 is the actual first argument passed from the caller
let (value, _) = argument_0;
return {
    value
};
```

Functions without a body block are terminated with a semicolon. This form
may only appear in a [trait] or [external block].

## Generic functions

A _generic function_ allows one or more _parameterized types_ to appear in its
signature. Each type parameter must be explicitly declared in an
angle-bracket-enclosed and comma-separated list, following the function name.

```rust
// foo is generic over A and B

fn foo<A, B>(x: A, y: B) {
# }
```

Inside the function signature and body, the name of the type parameter can be
used as a type name. [Trait] bounds can be specified for type
parameters to allow methods with that trait to be called on values of that
type. This is specified using the `where` syntax:

```rust
# use std::fmt::Debug;
fn foo<T>(x: T) where T: Debug {
# }
```

When a generic function is referenced, its type is instantiated based on the
context of the reference. For example, calling the `foo` function here:

```rust
use std::fmt::Debug;

fn foo<T>(x: &[T]) where T: Debug {
    // details elided
}

foo(&[1, 2]);
```

will instantiate type parameter `T` with `i32`.

The type parameters can also be explicitly supplied in a trailing [path]
component after the function name. This might be necessary if there is not
sufficient context to determine the type parameters. For example,
`mem::size_of::<u32>() == 4`.

## Extern function qualifier

The `extern` function qualifier allows providing function _definitions_ that can
be called with a particular ABI:

<!-- ignore: fake ABI -->
```rust,ignore
extern "ABI" fn foo() { /* ... */ }
```

These are often used in combination with [external block] items which provide
function _declarations_ that can be used to call functions without providing
their _definition_:

<!-- ignore: fake ABI -->
```rust,ignore
extern "ABI" {
  fn foo(); /* no body */
}
unsafe { foo() }
```

When `"extern" Abi?*` is omitted from `FunctionQualifiers` in function items,
the ABI `"Rust"` is assigned. For example:

```rust
fn foo() {}
```

is equivalent to:

```rust
extern "Rust" fn foo() {}
```

Functions can be called by foreign code, and using an ABI that
differs from Rust allows, for example, to provide functions that can be
called from other programming languages like C:

```rust
// Declares a function with the "C" ABI
extern "C" fn new_i32() -> i32 { 0 }

// Declares a function with the "stdcall" ABI
# #[cfg(target_arch = "x86_64")]
extern "stdcall" fn new_i32_stdcall() -> i32 { 0 }
```

Just as with [external block], when the `extern` keyword is used and the `"ABI"`
is omitted, the ABI used defaults to `"C"`. That is, this:

```rust
extern fn new_i32() -> i32 { 0 }
let fptr: extern fn() -> i32 = new_i32;
```

is equivalent to:

```rust
extern "C" fn new_i32() -> i32 { 0 }
let fptr: extern "C" fn() -> i32 = new_i32;
```

Functions with an ABI that differs from `"Rust"` do not support unwinding in the
exact same way that Rust does. Therefore, unwinding past the end of functions
with such ABIs causes the process to abort.

> **Note**: The LLVM backend of the `rustc` implementation
aborts the process by executing an illegal instruction.

## Const functions

Functions qualified with the `const` keyword are [const functions], as are
[tuple struct] and [tuple variant] constructors. _Const functions_  can be
called from within [const contexts].

Const functions may use the [`extern`] function qualifier, but only with the `"Rust"` and `"C"` ABIs.

Const functions are not allowed to be [async](#async-functions).

## Async functions

Functions may be qualified as async, and this can also be combined with the
`unsafe` qualifier:

```rust
async fn regular_example() { }
async unsafe fn unsafe_example() { }
```

Async functions do no work when called: instead, they
capture their arguments into a future. When polled, that future will
execute the function's body.

An async function is roughly equivalent to a function
that returns [`impl Future`] and with an [`async move` block][async-blocks] as
its body:

```rust
// Source
async fn example(x: &str) -> usize {
    x.len()
}
```

is roughly equivalent to:

```rust
# use std::future::Future;
// Desugared
fn example<'a>(x: &'a str) -> impl Future<Output = usize> + 'a {
    async move { x.len() }
}
```

The actual desugaring is more complex:

- The return type in the desugaring is assumed to capture all lifetime
  parameters from the `async fn` declaration. This can be seen in the
  desugared example above, which explicitly outlives, and hence
  captures, `'a`.
- The [`async move` block][async-blocks] in the body captures all function
  parameters, including those that are unused or bound to a `_`
  pattern. This ensures that function parameters are dropped in the
  same order as they would be if the function were not async, except
  that the drop occurs when the returned future has been fully
  awaited.

For more information on the effect of async, see [`async` blocks][async-blocks].

[async-blocks]: ../expressions/block-expr.md#async-blocks
[`impl Future`]: ../types/impl-trait.md

> **Edition differences**: Async functions are only available beginning with
> Rust 2018.

### Combining `async` and `unsafe`

It is legal to declare a function that is both async and unsafe. The
resulting function is unsafe to call and (like any async function)
returns a future. This future is just an ordinary future and thus an
`unsafe` context is not required to "await" it:

```rust
// Returns a future that, when awaited, dereferences `x`.
//
// Soundness condition: `x` must be safe to dereference until
// the resulting future is complete.
async unsafe fn unsafe_example(x: *const i32) -> i32 {
  *x
}

async fn safe_example() {
    // An `unsafe` block is required to invoke the function initially:
    let p = 22;
    let future = unsafe { unsafe_example(&p) };

    // But no `unsafe` block required here. This will
    // read the value of `p`:
    let q = future.await;
}
```

Note that this behavior is a consequence of the desugaring to a
function that returns an `impl Future` -- in this case, the function
we desugar to is an `unsafe` function, but the return value remains
the same.

Unsafe is used on an async function in precisely the same way that it
is used on other functions: it indicates that the function imposes
some additional obligations on its caller to ensure soundness. As in any
other unsafe function, these conditions may extend beyond the initial
call itself -- in the snippet above, for example, the `unsafe_example`
function took a pointer `x` as argument, and then (when awaited)
dereferenced that pointer. This implies that `x` would have to be
valid until the future is finished executing, and it is the caller's
responsibility to ensure that.

## Attributes on functions

[Outer attributes][attributes] are allowed on functions. [Inner
attributes][attributes] are allowed directly after the `{` inside its [block].

This example shows an inner attribute on a function. The function is documented
with just the word "Example".

```rust
fn documented() {
    #![doc = "Example"]
}
```

> Note: Except for lints, it is idiomatic to only use outer attributes on
> function items.

The attributes that have meaning on a function are [`cfg`], [`cfg_attr`], [`deprecated`],
[`doc`], [`export_name`], [`link_section`], [`no_mangle`], [the lint check
attributes], [`must_use`], [the procedural macro attributes], [the testing
attributes], and [the optimization hint attributes]. Functions also accept
attributes macros.

## Attributes on function parameters

[Outer attributes][attributes] are allowed on function parameters and the
permitted [built-in attributes] are restricted to `cfg`, `cfg_attr`, `allow`,
`warn`, `deny`, and `forbid`.

```rust
fn len(
    #[cfg(windows)] slice: &[u16],
    #[cfg(not(windows))] slice: &[u8],
) -> usize {
    slice.len()
}
```

Inert helper attributes used by procedural macro attributes applied to items are also
allowed but be careful to not include these inert attributes in your final `TokenStream`.

For example, the following code defines an inert `some_inert_attribute` attribute that
is not formally defined anywhere and the `some_proc_macro_attribute` procedural macro is
responsible for detecting its presence and removing it from the output token stream.

<!-- ignore: requires proc macro -->
```rust,ignore
#[some_proc_macro_attribute]
fn foo_oof(#[some_inert_attribute] arg: u8) {
}
```

[IDENTIFIER]: ../identifiers.md
[RAW_STRING_LITERAL]: ../tokens.md#raw-string-literals
[STRING_LITERAL]: ../tokens.md#string-literals
[_BlockExpression_]: ../expressions/block-expr.md
[_GenericParams_]: generics.md
[_Lifetime_]: ../trait-bounds.md
[_PatternNoTopAlt_]: ../patterns.md
[_Type_]: ../types.md#type-expressions
[_WhereClause_]: generics.md#where-clauses
[_OuterAttribute_]: ../attributes.md
[const contexts]: ../const_eval.md#const-context
[const functions]: ../const_eval.md#const-functions
[tuple struct]: structs.md
[tuple variant]: enumerations.md
[`extern`]: #extern-function-qualifier
[external block]: external-blocks.md
[path]: ../paths.md
[block]: ../expressions/block-expr.md
[variables]: ../variables.md
[type]: ../types.md#type-expressions
[unit type]: ../types/tuple.md
[*function item type*]: ../types/function-item.md
[Trait]: traits.md
[attributes]: ../attributes.md
[`cfg`]: ../conditional-compilation.md#the-cfg-attribute
[`cfg_attr`]: ../conditional-compilation.md#the-cfg_attr-attribute
[the lint check attributes]: ../attributes/diagnostics.md#lint-check-attributes
[the procedural macro attributes]: ../procedural-macros.md
[the testing attributes]: ../attributes/testing.md
[the optimization hint attributes]: ../attributes/codegen.md#optimization-hints
[`deprecated`]: ../attributes/diagnostics.md#the-deprecated-attribute
[`doc`]: ../../rustdoc/the-doc-attribute.html
[`must_use`]: ../attributes/diagnostics.md#the-must_use-attribute
[patterns]: ../patterns.md
[`export_name`]: ../abi.md#the-export_name-attribute
[`link_section`]: ../abi.md#the-link_section-attribute
[`no_mangle`]: ../abi.md#the-no_mangle-attribute
[built-in attributes]: ../attributes.html#built-in-attributes-index
[trait item]: traits.md
[method]: associated-items.md#methods
[associated function]: associated-items.md#associated-functions-and-methods
[implementation]: implementations.md
[variadic function]: external-blocks.md#variadic-functions
