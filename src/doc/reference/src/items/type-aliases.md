# Type aliases

> **<sup>Syntax</sup>**\
> _TypeAlias_ :\
> &nbsp;&nbsp; `type` [IDENTIFIER]&nbsp;[_GenericParams_]<sup>?</sup>
>              ( `:` [_TypeParamBounds_] )<sup>?</sup>
>              [_WhereClause_]<sup>?</sup> ( `=` [_Type_] [_WhereClause_]<sup>?</sup>)<sup>?</sup> `;`

A _type alias_ defines a new name for an existing [type]. Type aliases are
declared with the keyword `type`. Every value has a single, specific type, but
may implement several different traits, or be compatible with several different
type constraints.

For example, the following defines the type `Point` as a synonym for the type
`(u8, u8)`, the type of pairs of unsigned 8 bit integers:

```rust
type Point = (u8, u8);
let p: Point = (41, 68);
```

A type alias to a tuple-struct or unit-struct cannot be used to qualify that type's constructor:

```rust,compile_fail
struct MyStruct(u32);

use MyStruct as UseAlias;
type TypeAlias = MyStruct;

let _ = UseAlias(5); // OK
let _ = TypeAlias(5); // Doesn't work
```

A type alias, when not used as an associated type, must include a [_Type_] and
may not include [_TypeParamBounds_].

A type alias, when used as an [associated type] in a [trait], must not include a
[_Type_] specification but may include [_TypeParamBounds_].

A type alias, when used as an [associated type] in a [trait impl], must include
a [_Type_] specification and may not include [_TypeParamBounds_].

Where clauses before the equals sign on a type alias in a [trait impl] (like
`type TypeAlias<T> where T: Foo = Bar<T>`) are deprecated. Where clauses after
the equals sign (like `type TypeAlias<T> = Bar<T> where T: Foo`) are preferred.

[IDENTIFIER]: ../identifiers.md
[_GenericParams_]: generics.md
[_TypeParamBounds_]: ../trait-bounds.md
[_WhereClause_]: generics.md#where-clauses
[_Type_]: ../types.md#type-expressions
[associated type]: associated-items.md#associated-types
[trait]: traits.md
[type]: ../types.md
[trait impl]: implementations.md#trait-implementations
