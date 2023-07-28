# Namespaces

A *namespace* is a logical grouping of declared [names]. Names are segregated
into separate namespaces based on the kind of entity the name refers to.
Namespaces allow the occurrence of a name in one namespace to not conflict
with the same name in another namespace.

Within a namespace, names are organized in a hierarchy, where each level of
the hierarchy has its own collection of named entities.

There are several different namespaces that each contain different kinds of
entities. The usage of a name will look for the declaration of that name in
different namespaces, based on the context, as described in the [name
resolution] chapter.

The following is a list of namespaces, with their corresponding entities:

* Type Namespace
    * [Module declarations]
    * [External crate declarations]
    * [External crate prelude] items
    * [Struct], [union], [enum], enum variant declarations
    * [Trait item declarations]
    * [Type aliases]
    * [Associated type declarations]
    * Built-in types: [boolean], [numeric], and [textual]
    * [Generic type parameters]
    * [`Self` type]
    * [Tool attribute modules]
* Value Namespace
    * [Function declarations]
    * [Constant item declarations]
    * [Static item declarations]
    * [Struct constructors]
    * [Enum variant constructors]
    * [`Self` constructors]
    * [Generic const parameters]
    * [Associated const declarations]
    * [Associated function declarations]
    * Local bindings — [`let`], [`if let`], [`while let`], [`for`], [`match`]
      arms, [function parameters], [closure parameters]
    * Captured [closure] variables
* Macro Namespace
    * [`macro_rules` declarations]
    * [Built-in attributes]
    * [Tool attributes]
    * [Function-like procedural macros]
    * [Derive macros]
    * [Derive macro helpers]
    * [Attribute macros]
* Lifetime Namespace
    * [Generic lifetime parameters]
* Label Namespace
    * [Loop labels]
    * [Block labels]

An example of how overlapping names in different namespaces can be used unambiguously:

```rust
// Foo introduces a type in the type namespace and a constructor in the value
// namespace.
struct Foo(u32);

// The `Foo` macro is declared in the macro namespace.
macro_rules! Foo {
    () => {};
}

// `Foo` in the `f` parameter type refers to `Foo` in the type namespace.
// `'Foo` introduces a new lifetime in the lifetime namespace.
fn example<'Foo>(f: Foo) {
    // `Foo` refers to the `Foo` constructor in the value namespace.
    let ctor = Foo;
    // `Foo` refers to the `Foo` macro in the macro namespace.
    Foo!{}
    // `'Foo` introduces a label in the label namespace.
    'Foo: loop {
        // `'Foo` refers to the `'Foo` lifetime parameter, and `Foo`
        // refers to the type namespace.
        let x: &'Foo Foo;
        // `'Foo` refers to the label.
        break 'Foo;
    }
}
```

## Named entities without a namespace

The following entities have explicit names, but the names are not a part of
any specific namespace.

### Fields

Even though struct, enum, and union fields are named, the named fields do not
live in an explicit namespace. They can only be accessed via a [field
expression], which only inspects the field names of the specific type being
accessed.

### Use declarations

A [use declaration] has named aliases that it imports into scope, but the
`use` item itself does not belong to a specific namespace. Instead, it can
introduce aliases into multiple namespaces, depending on the item kind being
imported.

<!-- TODO: describe how `use` works on the use-declarations page, and link to it here. -->

## Sub-namespaces

The macro namespace is split into two sub-namespaces: one for [bang-style macros] and one for [attributes].
When an attribute is resolved, any bang-style macros in scope will be ignored.
And conversely resolving a bang-style macro will ignore attribute macros in scope.
This prevents one style from shadowing another.

For example, the [`cfg` attribute] and the [`cfg` macro] are two different entities with the same name in the macro namespace, but they can still be used in their respective context.

It is still an error for a [`use` import] to shadow another macro, regardless of their sub-namespaces.

[`cfg` attribute]: ../conditional-compilation.md#the-cfg-attribute
[`cfg` macro]: ../conditional-compilation.md#the-cfg-macro
[`for`]: ../expressions/loop-expr.md#iterator-loops
[`if let`]: ../expressions/if-expr.md#if-let-expressions
[`let`]: ../statements.md#let-statements
[`macro_rules` declarations]: ../macros-by-example.md
[`match`]: ../expressions/match-expr.md
[`Self` constructors]: ../paths.md#self-1
[`Self` type]: ../paths.md#self-1
[`use` import]: ../items/use-declarations.md
[`while let`]: ../expressions/loop-expr.md#predicate-pattern-loops
[Associated const declarations]: ../items/associated-items.md#associated-constants
[Associated function declarations]: ../items/associated-items.md#associated-functions-and-methods
[Associated type declarations]: ../items/associated-items.md#associated-types
[Attribute macros]: ../procedural-macros.md#attribute-macros
[attributes]: ../attributes.md
[bang-style macros]: ../macros.md
[Block labels]: ../expressions/loop-expr.md#labelled-block-expressions
[boolean]: ../types/boolean.md
[Built-in attributes]: ../attributes.md#built-in-attributes-index
[closure parameters]: ../expressions/closure-expr.md
[closure]: ../expressions/closure-expr.md
[Constant item declarations]: ../items/constant-items.md
[Derive macro helpers]: ../procedural-macros.md#derive-macro-helper-attributes
[Derive macros]: ../procedural-macros.md#derive-macros
[entity]: ../glossary.md#entity
[Enum variant constructors]: ../items/enumerations.md
[enum]: ../items/enumerations.md
[External crate declarations]: ../items/extern-crates.md
[External crate prelude]: preludes.md#extern-prelude
[field expression]: ../expressions/field-expr.md
[Function declarations]: ../items/functions.md
[function parameters]: ../items/functions.md#function-parameters
[Function-like procedural macros]: ../procedural-macros.md#function-like-procedural-macros
[Generic const parameters]: ../items/generics.md#const-generics
[Generic lifetime parameters]: ../items/generics.md
[Generic type parameters]: ../items/generics.md
[Loop labels]: ../expressions/loop-expr.md#loop-labels
[Module declarations]: ../items/modules.md
[name resolution]: name-resolution.md
[names]: ../names.md
[numeric]: ../types/numeric.md
[Static item declarations]: ../items/static-items.md
[Struct constructors]: ../items/structs.md
[Struct]: ../items/structs.md
[textual]: ../types/textual.md
[Tool attribute modules]: ../attributes.md#tool-attributes
[Tool attributes]: ../attributes.md#tool-attributes
[Trait item declarations]: ../items/traits.md
[Type aliases]: ../items/type-aliases.md
[union]: ../items/unions.md
[use declaration]: ../items/use-declarations.md
