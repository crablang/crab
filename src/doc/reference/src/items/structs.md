# Structs

> **<sup>Syntax</sup>**\
> _Struct_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _StructStruct_\
> &nbsp;&nbsp; | _TupleStruct_
>
> _StructStruct_ :\
> &nbsp;&nbsp; `struct`
>   [IDENTIFIER]&nbsp;
>   [_GenericParams_]<sup>?</sup>
>   [_WhereClause_]<sup>?</sup>
>   ( `{` _StructFields_<sup>?</sup> `}` | `;` )
>
> _TupleStruct_ :\
> &nbsp;&nbsp; `struct`
>   [IDENTIFIER]&nbsp;
>   [_GenericParams_]<sup>?</sup>
>   `(` _TupleFields_<sup>?</sup> `)`
>   [_WhereClause_]<sup>?</sup>
>   `;`
>
> _StructFields_ :\
> &nbsp;&nbsp; _StructField_ (`,` _StructField_)<sup>\*</sup> `,`<sup>?</sup>
>
> _StructField_ :\
> &nbsp;&nbsp; [_OuterAttribute_]<sup>\*</sup>\
> &nbsp;&nbsp; [_Visibility_]<sup>?</sup>\
> &nbsp;&nbsp; [IDENTIFIER] `:` [_Type_]
>
> _TupleFields_ :\
> &nbsp;&nbsp; _TupleField_ (`,` _TupleField_)<sup>\*</sup> `,`<sup>?</sup>
>
> _TupleField_ :\
> &nbsp;&nbsp; [_OuterAttribute_]<sup>\*</sup>\
> &nbsp;&nbsp; [_Visibility_]<sup>?</sup>\
> &nbsp;&nbsp; [_Type_]

A _struct_ is a nominal [struct type] defined with the keyword `struct`.
A struct declaration defines the given name in the [type namespace] of the module or block where it is located.

An example of a `struct` item and its use:

```rust
struct Point {x: i32, y: i32}
let p = Point {x: 10, y: 11};
let px: i32 = p.x;
```

A _tuple struct_ is a nominal [tuple type], and is also defined with the keyword `struct`.
In addition to defining a type, it also defines a constructor of the same name in the [value namespace].
The constructor is a function which can be called to create a new instance of the struct.
For example:

```rust
struct Point(i32, i32);
let p = Point(10, 11);
let px: i32 = match p { Point(x, _) => x };
```

A _unit-like struct_ is a struct without any fields, defined by leaving off the
list of fields entirely. Such a struct implicitly defines a [constant] of its
type with the same name. For example:

```rust
struct Cookie;
let c = [Cookie, Cookie {}, Cookie, Cookie {}];
```

is equivalent to

```rust
struct Cookie {}
const Cookie: Cookie = Cookie {};
let c = [Cookie, Cookie {}, Cookie, Cookie {}];
```

The precise memory layout of a struct is not specified. One can specify a
particular layout using the [`repr` attribute].

[_GenericParams_]: generics.md
[_OuterAttribute_]: ../attributes.md
[_Type_]: ../types.md#type-expressions
[_Visibility_]: ../visibility-and-privacy.md
[_WhereClause_]: generics.md#where-clauses
[`repr` attribute]: ../type-layout.md#representations
[IDENTIFIER]: ../identifiers.md
[constant]: constant-items.md
[struct type]: ../types/struct.md
[tuple type]: ../types/tuple.md
[type namespace]: ../names/namespaces.md
[value namespace]: ../names/namespaces.md
