{{#include types-redirect.html}}
# Types

Every variable, item, and value in a Rust program has a type. The _type_ of a
*value* defines the interpretation of the memory holding it and the operations
that may be performed on the value.

Built-in types are tightly integrated into the language, in nontrivial ways
that are not possible to emulate in user-defined types. User-defined types have
limited capabilities.

The list of types is:

* Primitive types:
    * [Boolean] — `bool`
    * [Numeric] — integer and float
    * [Textual] — `char` and `str`
    * [Never] — `!` — a type with no values
* Sequence types:
    * [Tuple]
    * [Array]
    * [Slice]
* User-defined types:
    * [Struct]
    * [Enum]
    * [Union]
* Function types:
    * [Functions]
    * [Closures]
* Pointer types:
    * [References]
    * [Raw pointers]
    * [Function pointers]
* Trait types:
    * [Trait objects]
    * [Impl trait]

## Type expressions

> **<sup>Syntax</sup>**\
> _Type_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _TypeNoBounds_\
> &nbsp;&nbsp; | [_ImplTraitType_]\
> &nbsp;&nbsp; | [_TraitObjectType_]
>
> _TypeNoBounds_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_ParenthesizedType_]\
> &nbsp;&nbsp; | [_ImplTraitTypeOneBound_]\
> &nbsp;&nbsp; | [_TraitObjectTypeOneBound_]\
> &nbsp;&nbsp; | [_TypePath_]\
> &nbsp;&nbsp; | [_TupleType_]\
> &nbsp;&nbsp; | [_NeverType_]\
> &nbsp;&nbsp; | [_RawPointerType_]\
> &nbsp;&nbsp; | [_ReferenceType_]\
> &nbsp;&nbsp; | [_ArrayType_]\
> &nbsp;&nbsp; | [_SliceType_]\
> &nbsp;&nbsp; | [_InferredType_]\
> &nbsp;&nbsp; | [_QualifiedPathInType_]\
> &nbsp;&nbsp; | [_BareFunctionType_]\
> &nbsp;&nbsp; | [_MacroInvocation_]

A _type expression_ as defined in the _Type_ grammar rule above is the syntax
for referring to a type. It may refer to:

* Sequence types ([tuple], [array], [slice]).
* [Type paths] which can reference:
    * Primitive types ([boolean], [numeric], [textual]).
    * Paths to an [item] ([struct], [enum], [union], [type alias], [trait]).
    * [`Self` path] where `Self` is the implementing type.
    * Generic [type parameters].
* Pointer types ([reference], [raw pointer], [function pointer]).
* The [inferred type] which asks the compiler to determine the type.
* [Parentheses] which are used for disambiguation.
* Trait types: [Trait objects] and [impl trait].
* The [never] type.
* [Macros] which expand to a type expression.

### Parenthesized types

> _ParenthesizedType_ :\
> &nbsp;&nbsp; `(` [_Type_] `)`

In some situations the combination of types may be ambiguous. Use parentheses
around a type to avoid ambiguity. For example, the `+` operator for [type
boundaries] within a [reference type] is unclear where the
boundary applies, so the use of parentheses is required. Grammar rules that
require this disambiguation use the [_TypeNoBounds_] rule instead of
[_Type_].

```rust
# use std::any::Any;
type T<'a> = &'a (dyn Any + Send);
```

## Recursive types

Nominal types &mdash; [structs], [enumerations], and [unions] &mdash; may be
recursive. That is, each `enum` variant or `struct` or `union` field may
refer, directly or indirectly, to the enclosing `enum` or `struct` type
itself. Such recursion has restrictions:

* Recursive types must include a nominal type in the recursion (not mere [type
  aliases], or other structural types such as [arrays] or [tuples]). So `type
  Rec = &'static [Rec]` is not allowed.
* The size of a recursive type must be finite; in other words the recursive
  fields of the type must be [pointer types].

An example of a *recursive* type and its use:

```rust
enum List<T> {
    Nil,
    Cons(T, Box<List<T>>)
}

let a: List<i32> = List::Cons(7, Box::new(List::Cons(13, Box::new(List::Nil))));
```

[_ArrayType_]: types/array.md
[_BareFunctionType_]: types/function-pointer.md
[_ImplTraitTypeOneBound_]: types/impl-trait.md
[_ImplTraitType_]: types/impl-trait.md
[_InferredType_]: types/inferred.md
[_MacroInvocation_]: macros.md#macro-invocation
[_NeverType_]: types/never.md
[_ParenthesizedType_]: types.md#parenthesized-types
[_QualifiedPathInType_]: paths.md#qualified-paths
[_RawPointerType_]: types/pointer.md#raw-pointers-const-and-mut
[_ReferenceType_]: types/pointer.md#shared-references-
[_SliceType_]: types/slice.md
[_TraitObjectTypeOneBound_]: types/trait-object.md
[_TraitObjectType_]: types/trait-object.md
[_TupleType_]: types/tuple.md#tuple-types
[_TypeNoBounds_]: types.md#type-expressions
[_TypePath_]: paths.md#paths-in-types
[_Type_]: types.md#type-expressions

[Array]: types/array.md
[Boolean]: types/boolean.md
[Closures]: types/closure.md
[Enum]: types/enum.md
[Function pointers]: types/function-pointer.md
[Functions]: types/function-item.md
[Impl trait]: types/impl-trait.md
[Macros]: macros.md
[Numeric]: types/numeric.md
[Parentheses]: #parenthesized-types
[Raw pointers]: types/pointer.md#raw-pointers-const-and-mut
[References]: types/pointer.md#shared-references-
[Slice]: types/slice.md
[Struct]: types/struct.md
[Textual]: types/textual.md
[Trait objects]: types/trait-object.md
[Tuple]: types/tuple.md
[Type paths]: paths.md#paths-in-types
[Union]: types/union.md
[`Self` path]: paths.md#self-1
[arrays]: types/array.md
[enumerations]: types/enum.md
[function pointer]: types/function-pointer.md
[inferred type]: types/inferred.md
[item]: items.md
[never]: types/never.md
[pointer types]: types/pointer.md
[raw pointer]: types/pointer.md#raw-pointers-const-and-mut
[reference type]: types/pointer.md#shared-references-
[reference]: types/pointer.md#shared-references-
[structs]: types/struct.md
[trait]: types/trait-object.md
[tuples]: types/tuple.md
[type alias]: items/type-aliases.md
[type aliases]: items/type-aliases.md
[type boundaries]: trait-bounds.md
[type parameters]: types/parameters.md
[unions]: types/union.md
