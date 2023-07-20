# Struct expressions

> **<sup>Syntax</sup>**\
> _StructExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _StructExprStruct_\
> &nbsp;&nbsp; | _StructExprTuple_\
> &nbsp;&nbsp; | _StructExprUnit_
>
> _StructExprStruct_ :\
> &nbsp;&nbsp; [_PathInExpression_] `{` (_StructExprFields_ | _StructBase_)<sup>?</sup> `}`
>
> _StructExprFields_ :\
> &nbsp;&nbsp; _StructExprField_ (`,` _StructExprField_)<sup>\*</sup> (`,` _StructBase_ | `,`<sup>?</sup>)
>
> _StructExprField_ :\
> &nbsp;&nbsp; [_OuterAttribute_] <sup>\*</sup>\
> &nbsp;&nbsp; (\
> &nbsp;&nbsp; &nbsp;&nbsp; &nbsp;&nbsp; [IDENTIFIER]\
> &nbsp;&nbsp; &nbsp;&nbsp; | ([IDENTIFIER] | [TUPLE_INDEX]) `:` [_Expression_]\
> &nbsp;&nbsp; )
>
> _StructBase_ :\
> &nbsp;&nbsp; `..` [_Expression_]
>
> _StructExprTuple_ :\
> &nbsp;&nbsp; [_PathInExpression_] `(`\
> &nbsp;&nbsp; &nbsp;&nbsp; ( [_Expression_] (`,` [_Expression_])<sup>\*</sup> `,`<sup>?</sup> )<sup>?</sup>\
> &nbsp;&nbsp; `)`
>
> _StructExprUnit_ : [_PathInExpression_]

A *struct expression* creates a struct, enum, or union value.
It consists of a path to a [struct], [enum variant], or [union] item followed by the values for the fields of the item.
There are three forms of struct expressions: struct, tuple, and unit.

The following are examples of struct expressions:

```rust
# struct Point { x: f64, y: f64 }
# struct NothingInMe { }
# struct TuplePoint(f64, f64);
# mod game { pub struct User<'a> { pub name: &'a str, pub age: u32, pub score: usize } }
# struct Cookie; fn some_fn<T>(t: T) {}
Point {x: 10.0, y: 20.0};
NothingInMe {};
TuplePoint(10.0, 20.0);
TuplePoint { 0: 10.0, 1: 20.0 }; // Results in the same value as the above line
let u = game::User {name: "Joe", age: 35, score: 100_000};
some_fn::<Cookie>(Cookie);
```

## Field struct expression

A struct expression with fields enclosed in curly braces allows you to specify the value for each individual field in any order.
The field name is separated from its value with a colon.

A value of a [union] type can only be created using this syntax, and it must specify exactly one field.

## Functional update syntax

A struct expression that constructs a value of a struct type can terminate with the syntax `..` followed by an expression to denote a functional update.
The expression following `..` (the base) must have the same struct type as the new struct type being formed.

The entire expression uses the given values for the fields that were specified and moves or copies the remaining fields from the base expression.
As with all struct expressions, all of the fields of the struct must be [visible], even those not explicitly named.

```rust
# struct Point3d { x: i32, y: i32, z: i32 }
let mut base = Point3d {x: 1, y: 2, z: 3};
let y_ref = &mut base.y;
Point3d {y: 0, z: 10, .. base}; // OK, only base.x is accessed
drop(y_ref);
```

Struct expressions with curly braces can't be used directly in a [loop] or [if] expression's head, or in the [scrutinee] of an [if let] or [match] expression.
However, struct expressions can be used in these situations if they are within another expression, for example inside [parentheses].

The field names can be decimal integer values to specify indices for constructing tuple structs.
This can be used with base structs to fill out the remaining indices not specified:

```rust
struct Color(u8, u8, u8);
let c1 = Color(0, 0, 0);  // Typical way of creating a tuple struct.
let c2 = Color{0: 255, 1: 127, 2: 0};  // Specifying fields by index.
let c3 = Color{1: 0, ..c2};  // Fill out all other fields using a base struct.
```

### Struct field init shorthand

When initializing a data structure (struct, enum, union) with named (but not numbered) fields, it is allowed to write `fieldname` as a shorthand for `fieldname: fieldname`.
This allows a compact syntax with less duplication.
For example:

```rust
# struct Point3d { x: i32, y: i32, z: i32 }
# let x = 0;
# let y_value = 0;
# let z = 0;
Point3d { x: x, y: y_value, z: z };
Point3d { x, y: y_value, z };
```

## Tuple struct expression

A struct expression with fields enclosed in parentheses constructs a tuple struct.
Though it is listed here as a specific expression for completeness, it is equivalent to a [call expression] to the tuple struct's constructor. For example:

```rust
struct Position(i32, i32, i32);
Position(0, 0, 0);  // Typical way of creating a tuple struct.
let c = Position;  // `c` is a function that takes 3 arguments.
let pos = c(8, 6, 7);  // Creates a `Position` value.
```

## Unit struct expression

A unit struct expression is just the path to a unit struct item.
This refers to the unit struct's implicit constant of its value.
The unit struct value can also be constructed with a fieldless struct expression. For example:

```rust
struct Gamma;
let a = Gamma;  // Gamma unit value.
let b = Gamma{};  // Exact same value as `a`.
```

[_OuterAttribute_]: ../attributes.md
[IDENTIFIER]: ../identifiers.md
[TUPLE_INDEX]: ../tokens.md#tuple-index
[_Expression_]: ../expressions.md
[_PathInExpression_]: ../paths.md#paths-in-expressions
[call expression]: call-expr.md
[enum variant]: ../items/enumerations.md
[if let]: if-expr.md#if-let-expressions
[if]: if-expr.md#if-expressions
[loop]: loop-expr.md
[match]: match-expr.md
[parentheses]: grouped-expr.md
[struct]: ../items/structs.md
[union]: ../items/unions.md
[visible]: ../visibility-and-privacy.md
[scrutinee]: ../glossary.md#scrutinee
