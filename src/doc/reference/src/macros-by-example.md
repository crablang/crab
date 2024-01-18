# Macros By Example

> **<sup>Syntax</sup>**\
> _MacroRulesDefinition_ :\
> &nbsp;&nbsp; `macro_rules` `!` [IDENTIFIER] _MacroRulesDef_
>
> _MacroRulesDef_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; `(` _MacroRules_ `)` `;`\
> &nbsp;&nbsp; | `[` _MacroRules_ `]` `;`\
> &nbsp;&nbsp; | `{` _MacroRules_ `}`
>
> _MacroRules_ :\
> &nbsp;&nbsp; _MacroRule_ ( `;` _MacroRule_ )<sup>\*</sup> `;`<sup>?</sup>
>
> _MacroRule_ :\
> &nbsp;&nbsp; _MacroMatcher_ `=>` _MacroTranscriber_
>
> _MacroMatcher_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; `(` _MacroMatch_<sup>\*</sup> `)`\
> &nbsp;&nbsp; | `[` _MacroMatch_<sup>\*</sup> `]`\
> &nbsp;&nbsp; | `{` _MacroMatch_<sup>\*</sup> `}`
>
> _MacroMatch_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_Token_]<sub>_except `$` and [delimiters]_</sub>\
> &nbsp;&nbsp; | _MacroMatcher_\
> &nbsp;&nbsp; | `$` ( [IDENTIFIER_OR_KEYWORD] <sub>_except `crate`_</sub> | [RAW_IDENTIFIER] | `_` ) `:` _MacroFragSpec_\
> &nbsp;&nbsp; | `$` `(` _MacroMatch_<sup>+</sup> `)` _MacroRepSep_<sup>?</sup> _MacroRepOp_
>
> _MacroFragSpec_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; `block` | `expr` | `ident` | `item` | `lifetime` | `literal`\
> &nbsp;&nbsp; | `meta` | `pat` | `pat_param` | `path` | `stmt` | `tt` | `ty` | `vis`
>
> _MacroRepSep_ :\
> &nbsp;&nbsp; [_Token_]<sub>_except [delimiters] and MacroRepOp_</sub>
>
> _MacroRepOp_ :\
> &nbsp;&nbsp; `*` | `+` | `?`
>
> _MacroTranscriber_ :\
> &nbsp;&nbsp; [_DelimTokenTree_]

`macro_rules` allows users to define syntax extension in a declarative way.  We
call such extensions "macros by example" or simply "macros".

Each macro by example has a name, and one or more _rules_. Each rule has two
parts: a _matcher_, describing the syntax that it matches, and a _transcriber_,
describing the syntax that will replace a successfully matched invocation. Both
the matcher and the transcriber must be surrounded by delimiters. Macros can
expand to expressions, statements, items (including traits, impls, and foreign
items), types, or patterns.

## Transcribing

When a macro is invoked, the macro expander looks up macro invocations by name,
and tries each macro rule in turn. It transcribes the first successful match; if
this results in an error, then future matches are not tried. When matching, no
lookahead is performed; if the compiler cannot unambiguously determine how to
parse the macro invocation one token at a time, then it is an error. In the
following example, the compiler does not look ahead past the identifier to see
if the following token is a `)`, even though that would allow it to parse the
invocation unambiguously:

```rust,compile_fail
macro_rules! ambiguity {
    ($($i:ident)* $j:ident) => { };
}

ambiguity!(error); // Error: local ambiguity
```

In both the matcher and the transcriber, the `$` token is used to invoke special
behaviours from the macro engine (described below in [Metavariables] and
[Repetitions]). Tokens that aren't part of such an invocation are matched and
transcribed literally, with one exception. The exception is that the outer
delimiters for the matcher will match any pair of delimiters. Thus, for
instance, the matcher `(())` will match `{()}` but not `{{}}`. The character
`$` cannot be matched or transcribed literally.

### Forwarding a matched fragment

When forwarding a matched fragment to another macro-by-example, matchers in
the second macro will see an opaque AST of the fragment type. The second macro
can't use literal tokens to match the fragments in the matcher, only a
fragment specifier of the same type. The `ident`, `lifetime`, and `tt`
fragment types are an exception, and *can* be matched by literal tokens. The
following illustrates this restriction:

```rust,compile_fail
macro_rules! foo {
    ($l:expr) => { bar!($l); }
// ERROR:               ^^ no rules expected this token in macro call
}

macro_rules! bar {
    (3) => {}
}

foo!(3);
```

The following illustrates how tokens can be directly matched after matching a
`tt` fragment:

```rust
// compiles OK
macro_rules! foo {
    ($l:tt) => { bar!($l); }
}

macro_rules! bar {
    (3) => {}
}

foo!(3);
```

## Metavariables

In the matcher, `$` _name_ `:` _fragment-specifier_ matches a Rust syntax
fragment of the kind specified and binds it to the metavariable `$`_name_. Valid
fragment specifiers are:

  * `item`: an [_Item_]
  * `block`: a [_BlockExpression_]
  * `stmt`: a [_Statement_] without the trailing semicolon (except for item
    statements that require semicolons)
  * `pat_param`: a [_PatternNoTopAlt_]
  * `pat`: at least any [_PatternNoTopAlt_], and possibly more depending on edition
  * `expr`: an [_Expression_]
  * `ty`: a [_Type_]
  * `ident`: an [IDENTIFIER_OR_KEYWORD] or [RAW_IDENTIFIER]
  * `path`: a [_TypePath_] style path
  * `tt`: a [_TokenTree_]&nbsp;(a single [token] or tokens in matching delimiters `()`, `[]`, or `{}`)
  * `meta`: an [_Attr_], the contents of an attribute
  * `lifetime`: a [LIFETIME_TOKEN]
  * `vis`: a possibly empty [_Visibility_] qualifier
  * `literal`: matches `-`<sup>?</sup>[_LiteralExpression_]

In the transcriber, metavariables are referred to simply by `$`_name_, since
the fragment kind is specified in the matcher. Metavariables are replaced with
the syntax element that matched them. The keyword metavariable `$crate` can be
used to refer to the current crate; see [Hygiene] below. Metavariables can be
transcribed more than once or not at all.

For reasons of backwards compatibility, though `_` [is also an
expression][_UnderscoreExpression_], a standalone underscore is not matched by
the `expr` fragment specifier. However, `_` is matched by the `expr` fragment
specifier when it appears as a subexpression.

> **Edition Differences**: Starting with the 2021 edition, `pat` fragment-specifiers match top-level or-patterns (that is, they accept [_Pattern_]).
>
> Before the 2021 edition, they match exactly the same fragments as `pat_param` (that is, they accept [_PatternNoTopAlt_]).
>
> The relevant edition is the one in effect for the `macro_rules!` definition.

## Repetitions

In both the matcher and transcriber, repetitions are indicated by placing the
tokens to be repeated inside `$(`…`)`, followed by a repetition operator,
optionally with a separator token between. The separator token can be any token
other than a delimiter or one of the repetition operators, but `;` and `,` are
the most common. For instance, `$( $i:ident ),*` represents any number of
identifiers separated by commas. Nested repetitions are permitted.

The repetition operators are:

- `*` — indicates any number of repetitions.
- `+` — indicates any number but at least one.
- `?` — indicates an optional fragment with zero or one occurrence.

Since `?` represents at most one occurrence, it cannot be used with a
separator.

The repeated fragment both matches and transcribes to the specified number of
the fragment, separated by the separator token. Metavariables are matched to
every repetition of their corresponding fragment. For instance, the `$( $i:ident
),*` example above matches `$i` to all of the identifiers in the list.

During transcription, additional restrictions apply to repetitions so that the
compiler knows how to expand them properly:

1.  A metavariable must appear in exactly the same number, kind, and nesting
    order of repetitions in the transcriber as it did in the matcher. So for the
    matcher `$( $i:ident ),*`, the transcribers `=> { $i }`,
    `=> { $( $( $i)* )* }`, and `=> { $( $i )+ }` are all illegal, but
    `=> { $( $i );* }` is correct and replaces a comma-separated list of
    identifiers with a semicolon-separated list.
2.  Each repetition in the transcriber must contain at least one metavariable to
    decide how many times to expand it. If multiple metavariables appear in the
    same repetition, they must be bound to the same number of fragments. For
    instance, `( $( $i:ident ),* ; $( $j:ident ),* ) => (( $( ($i,$j) ),* ))` must
    bind the same number of `$i` fragments as `$j` fragments. This means that
    invoking the macro with `(a, b, c; d, e, f)` is legal and expands to
    `((a,d), (b,e), (c,f))`, but `(a, b, c; d, e)` is illegal because it does
    not have the same number. This requirement applies to every layer of nested
    repetitions.

## Scoping, Exporting, and Importing

For historical reasons, the scoping of macros by example does not work entirely
like items. Macros have two forms of scope: textual scope, and path-based scope.
Textual scope is based on the order that things appear in source files, or even
across multiple files, and is the default scoping. It is explained further below.
Path-based scope works exactly the same way that item scoping does. The scoping,
exporting, and importing of macros is controlled largely by attributes.

When a macro is invoked by an unqualified identifier (not part of a multi-part
path), it is first looked up in textual scoping. If this does not yield any
results, then it is looked up in path-based scoping. If the macro's name is
qualified with a path, then it is only looked up in path-based scoping.

<!-- ignore: requires external crates -->
```rust,ignore
use lazy_static::lazy_static; // Path-based import.

macro_rules! lazy_static { // Textual definition.
    (lazy) => {};
}

lazy_static!{lazy} // Textual lookup finds our macro first.
self::lazy_static!{} // Path-based lookup ignores our macro, finds imported one.
```

### Textual Scope

Textual scope is based largely on the order that things appear in source files,
and works similarly to the scope of local variables declared with `let` except
it also applies at the module level. When `macro_rules!` is used to define a
macro, the macro enters the scope after the definition (note that it can still
be used recursively, since names are looked up from the invocation site), up
until its surrounding scope, typically a module, is closed. This can enter child
modules and even span across multiple files:

<!-- ignore: requires external modules -->
```rust,ignore
//// src/lib.rs
mod has_macro {
    // m!{} // Error: m is not in scope.

    macro_rules! m {
        () => {};
    }
    m!{} // OK: appears after declaration of m.

    mod uses_macro;
}

// m!{} // Error: m is not in scope.

//// src/has_macro/uses_macro.rs

m!{} // OK: appears after declaration of m in src/lib.rs
```

It is not an error to define a macro multiple times; the most recent declaration
will shadow the previous one unless it has gone out of scope.

```rust
macro_rules! m {
    (1) => {};
}

m!(1);

mod inner {
    m!(1);

    macro_rules! m {
        (2) => {};
    }
    // m!(1); // Error: no rule matches '1'
    m!(2);

    macro_rules! m {
        (3) => {};
    }
    m!(3);
}

m!(1);
```

Macros can be declared and used locally inside functions as well, and work
similarly:

```rust
fn foo() {
    // m!(); // Error: m is not in scope.
    macro_rules! m {
        () => {};
    }
    m!();
}


// m!(); // Error: m is not in scope.
```

### The `macro_use` attribute

The *`macro_use` attribute* has two purposes. First, it can be used to make a
module's macro scope not end when the module is closed, by applying it to a
module:

```rust
#[macro_use]
mod inner {
    macro_rules! m {
        () => {};
    }
}

m!();
```

Second, it can be used to import macros from another crate, by attaching it to
an `extern crate` declaration appearing in the crate's root module. Macros
imported this way are imported into the [`macro_use` prelude], not textually,
which means that they can be shadowed by any other name. While macros imported
by `#[macro_use]` can be used before the import statement, in case of a
conflict, the last macro imported wins. Optionally, a list of macros to import
can be specified using the [_MetaListIdents_] syntax; this is not supported
when `#[macro_use]` is applied to a module.

<!-- ignore: requires external crates -->
```rust,ignore
#[macro_use(lazy_static)] // Or #[macro_use] to import all macros.
extern crate lazy_static;

lazy_static!{}
// self::lazy_static!{} // Error: lazy_static is not defined in `self`
```

Macros to be imported with `#[macro_use]` must be exported with
`#[macro_export]`, which is described below.

### Path-Based Scope

By default, a macro has no path-based scope. However, if it has the
`#[macro_export]` attribute, then it is declared in the crate root scope and can
be referred to normally as such:

```rust
self::m!();
m!(); // OK: Path-based lookup finds m in the current module.

mod inner {
    super::m!();
    crate::m!();
}

mod mac {
    #[macro_export]
    macro_rules! m {
        () => {};
    }
}
```

Macros labeled with `#[macro_export]` are always `pub` and can be referred to
by other crates, either by path or by `#[macro_use]` as described above.

## Hygiene

By default, all identifiers referred to in a macro are expanded as-is, and are
looked up at the macro's invocation site. This can lead to issues if a macro
refers to an item or macro which isn't in scope at the invocation site. To
alleviate this, the `$crate` metavariable can be used at the start of a path to
force lookup to occur inside the crate defining the macro.

<!-- ignore: requires external crates -->
```rust,ignore
//// Definitions in the `helper_macro` crate.
#[macro_export]
macro_rules! helped {
    // () => { helper!() } // This might lead to an error due to 'helper' not being in scope.
    () => { $crate::helper!() }
}

#[macro_export]
macro_rules! helper {
    () => { () }
}

//// Usage in another crate.
// Note that `helper_macro::helper` is not imported!
use helper_macro::helped;

fn unit() {
    helped!();
}
```

Note that, because `$crate` refers to the current crate, it must be used with a
fully qualified module path when referring to non-macro items:

```rust
pub mod inner {
    #[macro_export]
    macro_rules! call_foo {
        () => { $crate::inner::foo() };
    }

    pub fn foo() {}
}
```

Additionally, even though `$crate` allows a macro to refer to items within its
own crate when expanding, its use has no effect on visibility. An item or macro
referred to must still be visible from the invocation site. In the following
example, any attempt to invoke `call_foo!()` from outside its crate will fail
because `foo()` is not public.

```rust
#[macro_export]
macro_rules! call_foo {
    () => { $crate::foo() };
}

fn foo() {}
```

> **Version & Edition Differences**: Prior to Rust 1.30, `$crate` and
> `local_inner_macros` (below) were unsupported. They were added alongside
> path-based imports of macros (described above), to ensure that helper macros
> did not need to be manually imported by users of a macro-exporting crate.
> Crates written for earlier versions of Rust that use helper macros need to be
> modified to use `$crate` or `local_inner_macros` to work well with path-based
> imports.

When a macro is exported, the `#[macro_export]` attribute can have the
`local_inner_macros` keyword added to automatically prefix all contained macro
invocations with `$crate::`. This is intended primarily as a tool to migrate
code written before `$crate` was added to the language to work with Rust 2018's
path-based imports of macros. Its use is discouraged in new code.

```rust
#[macro_export(local_inner_macros)]
macro_rules! helped {
    () => { helper!() } // Automatically converted to $crate::helper!().
}

#[macro_export]
macro_rules! helper {
    () => { () }
}
```

## Follow-set Ambiguity Restrictions

The parser used by the macro system is reasonably powerful, but it is limited in
order to prevent ambiguity in current or future versions of the language. In
particular, in addition to the rule about ambiguous expansions, a nonterminal
matched by a metavariable must be followed by a token which has been decided can
be safely used after that kind of match.

As an example, a macro matcher like `$i:expr [ , ]` could in theory be accepted
in Rust today, since `[,]` cannot be part of a legal expression and therefore
the parse would always be unambiguous. However, because `[` can start trailing
expressions, `[` is not a character which can safely be ruled out as coming
after an expression. If `[,]` were accepted in a later version of Rust, this
matcher would become ambiguous or would misparse, breaking working code.
Matchers like `$i:expr,` or `$i:expr;` would be legal, however, because `,` and
`;` are legal expression separators. The specific rules are:

  * `expr` and `stmt` may only be followed by one of: `=>`, `,`, or `;`.
  * `pat_param` may only be followed by one of: `=>`, `,`, `=`, `|`, `if`, or `in`.
  * `pat` may only be followed by one of: `=>`, `,`, `=`, `if`, or `in`.
  * `path` and `ty` may only be followed by one of: `=>`, `,`, `=`, `|`, `;`,
    `:`, `>`, `>>`, `[`, `{`, `as`, `where`, or a macro variable of `block`
    fragment specifier.
  * `vis` may only be followed by one of: `,`, an identifier other than a
    non-raw `priv`, any token that can begin a type, or a metavariable with a
    `ident`, `ty`, or `path` fragment specifier.
  * All other fragment specifiers have no restrictions.

> **Edition Differences**: Before the 2021 edition, `pat` may also be followed by `|`.

When repetitions are involved, then the rules apply to every possible number of
expansions, taking separators into account. This means:

  * If the repetition includes a separator, that separator must be able to
    follow the contents of the repetition.
  * If the repetition can repeat multiple times (`*` or `+`), then the contents
    must be able to follow themselves.
  * The contents of the repetition must be able to follow whatever comes
    before, and whatever comes after must be able to follow the contents of the
    repetition.
  * If the repetition can match zero times (`*` or `?`), then whatever comes
    after must be able to follow whatever comes before.


For more detail, see the [formal specification].

[Hygiene]: #hygiene
[IDENTIFIER]: identifiers.md
[IDENTIFIER_OR_KEYWORD]: identifiers.md
[RAW_IDENTIFIER]: identifiers.md
[LIFETIME_TOKEN]: tokens.md#lifetimes-and-loop-labels
[Metavariables]: #metavariables
[Repetitions]: #repetitions
[_Attr_]: attributes.md
[_BlockExpression_]: expressions/block-expr.md
[_DelimTokenTree_]: macros.md
[_Expression_]: expressions.md
[_Item_]: items.md
[_LiteralExpression_]: expressions/literal-expr.md
[_MetaListIdents_]: attributes.md#meta-item-attribute-syntax
[_Pattern_]: patterns.md
[_PatternNoTopAlt_]: patterns.md
[_Statement_]: statements.md
[_TokenTree_]: macros.md#macro-invocation
[_Token_]: tokens.md
[delimiters]: tokens.md#delimiters
[_TypePath_]: paths.md#paths-in-types
[_Type_]: types.md#type-expressions
[_UnderscoreExpression_]: expressions/underscore-expr.md
[_Visibility_]: visibility-and-privacy.md
[formal specification]: macro-ambiguity.md
[token]: tokens.md
[`macro_use` prelude]: names/preludes.md#macro_use-prelude
