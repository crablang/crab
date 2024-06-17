# Glossary

### Abstract syntax tree

An ‘abstract syntax tree’, or ‘AST’, is an intermediate representation of
the structure of the program when the compiler is compiling it.

### Alignment

The alignment of a value specifies what addresses values are preferred to
start at. Always a power of two. References to a value must be aligned.
[More][alignment].

### Arity

Arity refers to the number of arguments a function or operator takes.
For some examples, `f(2, 3)` and `g(4, 6)` have arity 2, while `h(8, 2, 6)`
has arity 3. The `!` operator has arity 1.

### Array

An array, sometimes also called a fixed-size array or an inline array, is a value
describing a collection of elements, each selected by an index that can be computed
at run time by the program. It occupies a contiguous region of memory.

### Associated item

An associated item is an item that is associated with another item. Associated
items are defined in [implementations] and declared in [traits]. Only
functions, constants, and type aliases can be associated. Contrast to a [free
item].

### Blanket implementation

Any implementation where a type appears [uncovered](#uncovered-type). `impl<T> Foo
for T`, `impl<T> Bar<T> for T`, `impl<T> Bar<Vec<T>> for T`, and `impl<T> Bar<T>
for Vec<T>` are considered blanket impls. However, `impl<T> Bar<Vec<T>> for
Vec<T>` is not a blanket impl, as all instances of `T` which appear in this `impl`
are covered by `Vec`.

### Bound

Bounds are constraints on a type or trait. For example, if a bound
is placed on the argument a function takes, types passed to that function
must abide by that constraint.

### Combinator

Combinators are higher-order functions that apply only functions and
earlier defined combinators to provide a result from its arguments.
They can be used to manage control flow in a modular fashion.

### Crate

A crate is the unit of compilation and linking. There are different [types of
crates], such as libraries or executables. Crates may link and refer to other
library crates, called external crates. A crate has a self-contained tree of
[modules], starting from an unnamed root module called the crate root. [Items]
may be made visible to other crates by marking them as public in the crate
root, including through [paths] of public modules.
[More][crate].

### Dispatch

Dispatch is the mechanism to determine which specific version of code is actually
run when it involves polymorphism. Two major forms of dispatch are static dispatch and
dynamic dispatch. While Rust favors static dispatch, it also supports dynamic dispatch
through a mechanism called ‘trait objects’.

### Dynamically sized type

A dynamically sized type (DST) is a type without a statically known size or alignment.

### Entity

An [*entity*] is a language construct that can be referred to in some way
within the source program, usually via a [path][paths]. Entities include
[types], [items], [generic parameters], [variable bindings], [loop labels],
[lifetimes], [fields], [attributes], and [lints].

### Expression

An expression is a combination of values, constants, variables, operators
and functions that evaluate to a single value, with or without side-effects.

For example, `2 + (3 * 4)` is an expression that returns the value 14.

### Free item

An [item] that is not a member of an [implementation], such as a *free
function* or a *free const*. Contrast to an [associated item].

### Fundamental traits

A fundamental trait is one where adding an impl of it for an existing type is a breaking change.
The `Fn` traits and `Sized` are fundamental.

### Fundamental type constructors

A fundamental type constructor is a type where implementing a [blanket implementation](#blanket-implementation) over it
is a breaking change. `&`, `&mut`, `Box`, and `Pin`  are fundamental.

Any time a type `T` is considered [local](#local-type), `&T`, `&mut T`, `Box<T>`, and `Pin<T>`
are also considered local. Fundamental type constructors cannot [cover](#uncovered-type) other types.
Any time the term "covered type" is used,
the `T` in `&T`, `&mut T`, `Box<T>`, and `Pin<T>` is not considered covered.

### Inhabited

A type is inhabited if it has constructors and therefore can be instantiated. An inhabited type is
not "empty" in the sense that there can be values of the type. Opposite of
[Uninhabited](#uninhabited).

### Inherent implementation

An [implementation] that applies to a nominal type, not to a trait-type pair.
[More][inherent implementation].

### Inherent method

A [method] defined in an [inherent implementation], not in a trait
implementation.

### Initialized

A variable is initialized if it has been assigned a value and hasn't since been
moved from. All other memory locations are assumed to be uninitialized. Only
unsafe Rust can create a memory location without initializing it.

### Local trait

A `trait` which was defined in the current crate. A trait definition is local
or not independent of applied type arguments. Given `trait Foo<T, U>`,
`Foo` is always local, regardless of the types substituted for `T` and `U`.

### Local type

A `struct`, `enum`, or `union` which was defined in the current crate.
This is not affected by applied type arguments. `struct Foo` is considered local, but
`Vec<Foo>` is not. `LocalType<ForeignType>` is local. Type aliases do not
affect locality.

### Module

A module is a container for zero or more [items]. Modules are organized in a
tree, starting from an unnamed module at the root called the crate root or the
root module. [Paths] may be used to refer to items from other modules, which
may be restricted by [visibility rules].
[More][modules]

### Name

A [*name*] is an [identifier] or [lifetime or loop label] that refers to an
[entity](#entity). A *name binding* is when an entity declaration introduces
an identifier or label associated with that entity. [Paths],
identifiers, and labels are used to refer to an entity.

### Name resolution

[*Name resolution*] is the compile-time process of tying [paths],
[identifiers], and [labels] to [entity](#entity) declarations.

### Namespace

A *namespace* is a logical grouping of declared [names](#name) based on the
kind of [entity](#entity) the name refers to. Namespaces allow the occurrence
of a name in one namespace to not conflict with the same name in another
namespace.

Within a namespace, names are organized in a hierarchy, where each level of
the hierarchy has its own collection of named entities.

### Nominal types

Types that can be referred to by a path directly. Specifically [enums],
[structs], [unions], and [trait objects].

### Object safe traits

[Traits] that can be used as [trait objects]. Only traits that follow specific
[rules][object safety] are object safe.

### Path

A [*path*] is a sequence of one or more path segments used to refer to an
[entity](#entity) in the current scope or other levels of a
[namespace](#namespace) hierarchy.

### Prelude

Prelude, or The Rust Prelude, is a small collection of items - mostly traits - that are
imported into every module of every crate. The traits in the prelude are pervasive.

### Scope

A [*scope*] is the region of source text where a named [entity](#entity) may
be referenced with that name.

### Scrutinee

A scrutinee is the expression that is matched on in `match` expressions and
similar pattern matching constructs. For example, in `match x { A => 1, B => 2 }`,
the expression `x` is the scrutinee.

### Size

The size of a value has two definitions.

The first is that it is how much memory must be allocated to store that value.

The second is that it is the offset in bytes between successive elements in an
array with that item type.

It is a multiple of the alignment, including zero. The size can change
depending on compiler version (as new optimizations are made) and target
platform (similar to how `usize` varies per-platform).

[More][alignment].

### Slice

A slice is dynamically-sized view into a contiguous sequence, written as `[T]`.

It is often seen in its borrowed forms, either mutable or shared. The shared
slice type is `&[T]`, while the mutable slice type is `&mut [T]`, where `T` represents
the element type.

### Statement

A statement is the smallest standalone element of a programming language
that commands a computer to perform an action.

### String literal

A string literal is a string stored directly in the final binary, and so will be
valid for the `'static` duration.

Its type is `'static` duration borrowed string slice, `&'static str`.

### String slice

A string slice is the most primitive string type in Rust, written as `str`. It is
often seen in its borrowed forms, either mutable or shared. The shared
string slice type is `&str`, while the mutable string slice type is `&mut str`.

Strings slices are always valid UTF-8.

### Trait

A trait is a language item that is used for describing the functionalities a type must provide.
It allows a type to make certain promises about its behavior.

Generic functions and generic structs can use traits to constrain, or bound, the types they accept.

### Turbofish

Paths with generic parameters in expressions must prefix the opening brackets with a `::`.
Combined with the angular brackets for generics, this looks like a fish `::<>`.
As such, this syntax is colloquially referred to as turbofish syntax.

Examples:

```rust
let ok_num = Ok::<_, ()>(5);
let vec = [1, 2, 3].iter().map(|n| n * 2).collect::<Vec<_>>();
```

This `::` prefix is required to disambiguate generic paths with multiple comparisons in a comma-separate list.
See [the bastion of the turbofish][turbofish test] for an example where not having the prefix would be ambiguous.

### Uncovered type

A type which does not appear as an argument to another type. For example,
`T` is uncovered, but the `T` in `Vec<T>` is covered. This is only relevant for
type arguments.

### Undefined behavior

Compile-time or run-time behavior that is not specified. This may result in,
but is not limited to: process termination or corruption; improper, incorrect,
or unintended computation; or platform-specific results.
[More][undefined-behavior].

### Uninhabited

A type is uninhabited if it has no constructors and therefore can never be instantiated. An
uninhabited type is "empty" in the sense that there are no values of the type. The canonical
example of an uninhabited type is the [never type] `!`, or an enum with no variants
`enum Never { }`. Opposite of [Inhabited](#inhabited).

[alignment]: type-layout.md#size-and-alignment
[associated item]: #associated-item
[attributes]: attributes.md
[*entity*]: names.md
[crate]: crates-and-source-files.md
[enums]: items/enumerations.md
[fields]: expressions/field-expr.md
[free item]: #free-item
[generic parameters]: items/generics.md
[identifier]: identifiers.md
[identifiers]: identifiers.md
[implementation]: items/implementations.md
[implementations]: items/implementations.md
[inherent implementation]: items/implementations.md#inherent-implementations
[item]: items.md
[items]: items.md
[labels]: tokens.md#lifetimes-and-loop-labels
[lifetime or loop label]: tokens.md#lifetimes-and-loop-labels
[lifetimes]: tokens.md#lifetimes-and-loop-labels
[lints]: attributes/diagnostics.md#lint-check-attributes
[loop labels]: tokens.md#lifetimes-and-loop-labels
[method]: items/associated-items.md#methods
[modules]: items/modules.md
[*Name resolution*]: names/name-resolution.md
[*name*]: names.md
[*namespace*]: names/namespaces.md
[never type]: types/never.md
[object safety]: items/traits.md#object-safety
[*path*]: paths.md
[Paths]: paths.md
[*scope*]: names/scopes.md
[structs]: items/structs.md
[trait objects]: types/trait-object.md
[traits]: items/traits.md
[turbofish test]: https://github.com/rust-lang/rust/blob/1.58.0/src/test/ui/parser/bastion-of-the-turbofish.rs
[types of crates]: linkage.md
[types]: types.md
[undefined-behavior]: behavior-considered-undefined.md
[unions]: items/unions.md
[variable bindings]: patterns.md
[visibility rules]: visibility-and-privacy.md
