## Procedural Macros

*Procedural macros* allow creating syntax extensions as execution of a function.
Procedural macros come in one of three flavors:

* [Function-like macros] - `custom!(...)`
* [Derive macros] - `#[derive(CustomDerive)]`
* [Attribute macros] - `#[CustomAttribute]`

Procedural macros allow you to run code at compile time that operates over Rust
syntax, both consuming and producing Rust syntax. You can sort of think of
procedural macros as functions from an AST to another AST.

Procedural macros must be defined in a crate with the [crate type] of
`proc-macro`.

> **Note**: When using Cargo, Procedural macro crates are defined with the
> `proc-macro` key in your manifest:
>
> ```toml
> [lib]
> proc-macro = true
> ```

As functions, they must either return syntax, panic, or loop endlessly. Returned
syntax either replaces or adds the syntax depending on the kind of procedural
macro. Panics are caught by the compiler and are turned into a compiler error.
Endless loops are not caught by the compiler which hangs the compiler.

Procedural macros run during compilation, and thus have the same resources that
the compiler has. For example, standard input, error, and output are the same
that the compiler has access to. Similarly, file access is the same. Because
of this, procedural macros have the same security concerns that [Cargo's
build scripts] have.

Procedural macros have two ways of reporting errors. The first is to panic. The
second is to emit a [`compile_error`] macro invocation.

### The `proc_macro` crate

Procedural macro crates almost always will link to the compiler-provided
[`proc_macro` crate]. The `proc_macro` crate provides types required for
writing procedural macros and facilities to make it easier.

This crate primarily contains a [`TokenStream`] type. Procedural macros operate
over *token streams* instead of AST nodes, which is a far more stable interface
over time for both the compiler and for procedural macros to target. A
*token stream* is roughly equivalent to `Vec<TokenTree>` where a `TokenTree`
can roughly be thought of as lexical token. For example `foo` is an `Ident`
token, `.` is a `Punct` token, and `1.2` is a `Literal` token. The `TokenStream`
type, unlike `Vec<TokenTree>`, is cheap to clone.

All tokens have an associated `Span`. A `Span` is an opaque value that cannot
be modified but can be manufactured. `Span`s represent an extent of source
code within a program and are primarily used for error reporting. While you
cannot modify a `Span` itself, you can always change the `Span` *associated*
with any token, such as through getting a `Span` from another token.

### Procedural macro hygiene

Procedural macros are *unhygienic*. This means they behave as if the output
token stream was simply written inline to the code it's next to. This means that
it's affected by external items and also affects external imports.

Macro authors need to be careful to ensure their macros work in as many contexts
as possible given this limitation. This often includes using absolute paths to
items in libraries (for example, `::std::option::Option` instead of `Option`) or
by ensuring that generated functions have names that are unlikely to clash with
other functions (like `__internal_foo` instead of `foo`).

### Function-like procedural macros

*Function-like procedural macros* are procedural macros that are invoked using
the macro invocation operator (`!`).

These macros are defined by a [public]&#32;[function] with the `proc_macro`
[attribute] and a signature of `(TokenStream) -> TokenStream`. The input
[`TokenStream`] is what is inside the delimiters of the macro invocation and the
output [`TokenStream`] replaces the entire macro invocation.

For example, the following macro definition ignores its input and outputs a
function `answer` into its scope.

<!-- ignore: test doesn't support proc-macro -->
```rust,ignore
# #![crate_type = "proc-macro"]
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn make_answer(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
```

And then we use it in a binary crate to print "42" to standard output.

<!-- ignore: requires external crates -->
```rust,ignore
extern crate proc_macro_examples;
use proc_macro_examples::make_answer;

make_answer!();

fn main() {
    println!("{}", answer());
}
```

Function-like procedural macros may be invoked in any macro invocation
position, which includes [statements], [expressions], [patterns], [type
expressions], [item] positions, including items in [`extern` blocks], inherent
and trait [implementations], and [trait definitions].

### Derive macros

*Derive macros* define new inputs for the [`derive` attribute]. These macros
can create new [items] given the token stream of a [struct], [enum], or [union].
They can also define [derive macro helper attributes].

Custom derive macros are defined by a [public]&#32;[function] with the
`proc_macro_derive` attribute and a signature of `(TokenStream) -> TokenStream`.

The input [`TokenStream`] is the token stream of the item that has the `derive`
attribute on it. The output [`TokenStream`] must be a set of items that are
then appended to the [module] or [block] that the item from the input
[`TokenStream`] is in.

The following is an example of a derive macro. Instead of doing anything
useful with its input, it just appends a function `answer`.

<!-- ignore: test doesn't support proc-macro -->
```rust,ignore
# #![crate_type = "proc-macro"]
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(AnswerFn)]
pub fn derive_answer_fn(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
```

And then using said derive macro:

<!-- ignore: requires external crates -->
```rust,ignore
extern crate proc_macro_examples;
use proc_macro_examples::AnswerFn;

#[derive(AnswerFn)]
struct Struct;

fn main() {
    assert_eq!(42, answer());
}
```

#### Derive macro helper attributes

Derive macros can add additional [attributes] into the scope of the [item]
they are on. Said attributes are called *derive macro helper attributes*. These
attributes are [inert], and their only purpose is to be fed into the derive
macro that defined them. That said, they can be seen by all macros.

The way to define helper attributes is to put an `attributes` key in the
`proc_macro_derive` macro with a comma separated list of identifiers that are
the names of the helper attributes.

For example, the following derive macro defines a helper attribute
`helper`, but ultimately doesn't do anything with it.

<!-- ignore: test doesn't support proc-macro -->
```rust,ignore
# #![crate_type="proc-macro"]
# extern crate proc_macro;
# use proc_macro::TokenStream;

#[proc_macro_derive(HelperAttr, attributes(helper))]
pub fn derive_helper_attr(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
```

And then usage on the derive macro on a struct:

<!-- ignore: requires external crates -->
```rust,ignore
#[derive(HelperAttr)]
struct Struct {
    #[helper] field: ()
}
```

### Attribute macros

*Attribute macros* define new [outer attributes][attributes] which can be
attached to [items], including items in [`extern` blocks], inherent and trait
[implementations], and [trait definitions].

Attribute macros are defined by a [public]&#32;[function] with the
`proc_macro_attribute` [attribute] that has a signature of `(TokenStream,
TokenStream) -> TokenStream`. The first [`TokenStream`] is the delimited token
tree following the attribute's name, not including the outer delimiters. If
the attribute is written as a bare attribute name, the attribute
[`TokenStream`] is empty. The second [`TokenStream`] is the rest of the [item]
including other [attributes] on the [item]. The returned [`TokenStream`]
replaces the [item] with an arbitrary number of [items].

For example, this attribute macro takes the input stream and returns it as is,
effectively being the no-op of attributes.

<!-- ignore: test doesn't support proc-macro -->
```rust,ignore
# #![crate_type = "proc-macro"]
# extern crate proc_macro;
# use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn return_as_is(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
```

This following example shows the stringified [`TokenStream`s] that the attribute
macros see. The output will show in the output of the compiler. The output is
shown in the comments after the function prefixed with "out:".

<!-- ignore: test doesn't support proc-macro -->
```rust,ignore
// my-macro/src/lib.rs
# extern crate proc_macro;
# use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn show_streams(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}
```

<!-- ignore: requires external crates -->
```rust,ignore
// src/lib.rs
extern crate my_macro;

use my_macro::show_streams;

// Example: Basic function
#[show_streams]
fn invoke1() {}
// out: attr: ""
// out: item: "fn invoke1() {}"

// Example: Attribute with input
#[show_streams(bar)]
fn invoke2() {}
// out: attr: "bar"
// out: item: "fn invoke2() {}"

// Example: Multiple tokens in the input
#[show_streams(multiple => tokens)]
fn invoke3() {}
// out: attr: "multiple => tokens"
// out: item: "fn invoke3() {}"

// Example:
#[show_streams { delimiters }]
fn invoke4() {}
// out: attr: "delimiters"
// out: item: "fn invoke4() {}"
```

### Declarative macro tokens and procedural macro tokens

Declarative `macro_rules` macros and procedural macros use similar, but
different definitions for tokens (or rather [`TokenTree`s].)

Token trees in `macro_rules` (corresponding to `tt` matchers) are defined as
- Delimited groups (`(...)`, `{...}`, etc)
- All operators supported by the language, both single-character and
  multi-character ones (`+`, `+=`).
    - Note that this set doesn't include the single quote `'`.
- Literals (`"string"`, `1`, etc)
    - Note that negation (e.g. `-1`) is never a part of such literal tokens,
      but a separate operator token.
- Identifiers, including keywords (`ident`, `r#ident`, `fn`)
- Lifetimes (`'ident`)
- Metavariable substitutions in `macro_rules` (e.g. `$my_expr` in
  `macro_rules! mac { ($my_expr: expr) => { $my_expr } }` after the `mac`'s
  expansion, which will be considered a single token tree regardless of the
  passed expression)

Token trees in procedural macros are defined as
- Delimited groups (`(...)`, `{...}`, etc)
- All punctuation characters used in operators supported by the language (`+`,
  but not `+=`), and also the single quote `'` character (typically used in
  lifetimes, see below for lifetime splitting and joining behavior)
- Literals (`"string"`, `1`, etc)
    - Negation (e.g. `-1`) is supported as a part of integer
      and floating point literals.
- Identifiers, including keywords (`ident`, `r#ident`, `fn`)

Mismatches between these two definitions are accounted for when token streams
are passed to and from procedural macros. \
Note that the conversions below may happen lazily, so they might not happen if
the tokens are not actually inspected.

When passed to a proc-macro
- All multi-character operators are broken into single characters.
- Lifetimes are broken into a `'` character and an identifier.
- All metavariable substitutions are represented as their underlying token
  streams.
    - Such token streams may be wrapped into delimited groups ([`Group`]) with
      implicit delimiters ([`Delimiter::None`]) when it's necessary for
      preserving parsing priorities.
    - `tt` and `ident` substitutions are never wrapped into such groups and
      always represented as their underlying token trees.

When emitted from a proc macro
- Punctuation characters are glued into multi-character operators
  when applicable.
- Single quotes `'` joined with identifiers are glued into lifetimes.
- Negative literals are converted into two tokens (the `-` and the literal)
  possibly wrapped into a delimited group ([`Group`]) with implicit delimiters
  ([`Delimiter::None`]) when it's necessary for preserving parsing priorities.

Note that neither declarative nor procedural macros support doc comment tokens
(e.g. `/// Doc`), so they are always converted to token streams representing
their equivalent `#[doc = r"str"]` attributes when passed to macros.

[Attribute macros]: #attribute-macros
[Cargo's build scripts]: ../cargo/reference/build-scripts.html
[Derive macros]: #derive-macros
[Function-like macros]: #function-like-procedural-macros
[`Delimiter::None`]: ../proc_macro/enum.Delimiter.html#variant.None
[`Group`]: ../proc_macro/struct.Group.html
[`TokenStream`]: ../proc_macro/struct.TokenStream.html
[`TokenStream`s]: ../proc_macro/struct.TokenStream.html
[`TokenTree`s]: ../proc_macro/enum.TokenTree.html
[`compile_error`]: ../std/macro.compile_error.html
[`derive` attribute]: attributes/derive.md
[`extern` blocks]: items/external-blocks.md
[`macro_rules`]: macros-by-example.md
[`proc_macro` crate]: ../proc_macro/index.html
[attribute]: attributes.md
[attributes]: attributes.md
[block]: expressions/block-expr.md
[crate type]: linkage.md
[derive macro helper attributes]: #derive-macro-helper-attributes
[enum]: items/enumerations.md
[expressions]: expressions.md
[function]: items/functions.md
[implementations]: items/implementations.md
[inert]: attributes.md#active-and-inert-attributes
[item]: items.md
[items]: items.md
[module]: items/modules.md
[patterns]: patterns.md
[public]: visibility-and-privacy.md
[statements]: statements.md
[struct]: items/structs.md
[trait definitions]: items/traits.md
[type expressions]: types.md#type-expressions
[type]: types.md
[union]: items/unions.md
