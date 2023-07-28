# Patterns

> **<sup>Syntax</sup>**\
> _Pattern_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; `|`<sup>?</sup> _PatternNoTopAlt_  ( `|` _PatternNoTopAlt_ )<sup>\*</sup>
>
> _PatternNoTopAlt_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _PatternWithoutRange_\
> &nbsp;&nbsp; | [_RangePattern_]
>
> _PatternWithoutRange_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_LiteralPattern_]\
> &nbsp;&nbsp; | [_IdentifierPattern_]\
> &nbsp;&nbsp; | [_WildcardPattern_]\
> &nbsp;&nbsp; | [_RestPattern_]\
> &nbsp;&nbsp; | [_ReferencePattern_]\
> &nbsp;&nbsp; | [_StructPattern_]\
> &nbsp;&nbsp; | [_TupleStructPattern_]\
> &nbsp;&nbsp; | [_TuplePattern_]\
> &nbsp;&nbsp; | [_GroupedPattern_]\
> &nbsp;&nbsp; | [_SlicePattern_]\
> &nbsp;&nbsp; | [_PathPattern_]\
> &nbsp;&nbsp; | [_MacroInvocation_]

Patterns are used to match values against structures and to, optionally, bind variables to values inside these structures.
They are also used in variable declarations and parameters for functions and closures.

The pattern in the following example does four things:

* Tests if `person` has the `car` field filled with something.
* Tests if the person's `age` field is between 13 and 19, and binds its value to the `person_age` variable.
* Binds a reference to the `name` field to the variable `person_name`.
* Ignores the rest of the fields of `person`.
  The remaining fields can have any value and are not bound to any variables.

```rust
# struct Car;
# struct Computer;
# struct Person {
#     name: String,
#     car: Option<Car>,
#     computer: Option<Computer>,
#     age: u8,
# }
# let person = Person {
#     name: String::from("John"),
#     car: Some(Car),
#     computer: None,
#     age: 15,
# };
if let
    Person {
        car: Some(_),
        age: person_age @ 13..=19,
        name: ref person_name,
        ..
    } = person
{
    println!("{} has a car and is {} years old.", person_name, person_age);
}
```

Patterns are used in:

* [`let` declarations](statements.md#let-statements)
* [Function](items/functions.md) and [closure](expressions/closure-expr.md) parameters
* [`match` expressions](expressions/match-expr.md)
* [`if let` expressions](expressions/if-expr.md)
* [`while let` expressions](expressions/loop-expr.md#predicate-pattern-loops)
* [`for` expressions](expressions/loop-expr.md#iterator-loops)

## Destructuring

Patterns can be used to *destructure* [structs], [enums], and [tuples].
Destructuring breaks up a value into its component pieces.
The syntax used is almost the same as when creating such values.
In a pattern whose [scrutinee] expression has a `struct`, `enum` or `tuple` type, a placeholder (`_`) stands in for a *single* data field, whereas a wildcard `..`  stands in for *all* the remaining fields of a particular variant.
When destructuring a data structure with named (but not numbered) fields, it is allowed to write `fieldname` as a shorthand for `fieldname: fieldname`.

```rust
# enum Message {
#     Quit,
#     WriteString(String),
#     Move { x: i32, y: i32 },
#     ChangeColor(u8, u8, u8),
# }
# let message = Message::Quit;
match message {
    Message::Quit => println!("Quit"),
    Message::WriteString(write) => println!("{}", &write),
    Message::Move{ x, y: 0 } => println!("move {} horizontally", x),
    Message::Move{ .. } => println!("other move"),
    Message::ChangeColor { 0: red, 1: green, 2: _ } => {
        println!("color change, red: {}, green: {}", red, green);
    }
};
```

## Refutability

A pattern is said to be *refutable* when it has the possibility of not being matched by the value it is being matched against.
*Irrefutable* patterns, on the other hand, always match the value they are being matched against.
Examples:

```rust
let (x, y) = (1, 2);               // "(x, y)" is an irrefutable pattern

if let (a, 3) = (1, 2) {           // "(a, 3)" is refutable, and will not match
    panic!("Shouldn't reach here");
} else if let (a, 4) = (3, 4) {    // "(a, 4)" is refutable, and will match
    println!("Matched ({}, 4)", a);
}
```

## Literal patterns

> **<sup>Syntax</sup>**\
> _LiteralPattern_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; `true` | `false`\
> &nbsp;&nbsp; | [CHAR_LITERAL]\
> &nbsp;&nbsp; | [BYTE_LITERAL]\
> &nbsp;&nbsp; | [STRING_LITERAL]\
> &nbsp;&nbsp; | [RAW_STRING_LITERAL]\
> &nbsp;&nbsp; | [BYTE_STRING_LITERAL]\
> &nbsp;&nbsp; | [RAW_BYTE_STRING_LITERAL]\
> &nbsp;&nbsp; | `-`<sup>?</sup> [INTEGER_LITERAL]\
> &nbsp;&nbsp; | `-`<sup>?</sup> [FLOAT_LITERAL]

[CHAR_LITERAL]: tokens.md#character-literals
[BYTE_LITERAL]: tokens.md#byte-literals
[STRING_LITERAL]: tokens.md#string-literals
[RAW_STRING_LITERAL]: tokens.md#raw-string-literals
[BYTE_STRING_LITERAL]: tokens.md#byte-string-literals
[RAW_BYTE_STRING_LITERAL]: tokens.md#raw-byte-string-literals
[INTEGER_LITERAL]: tokens.md#integer-literals
[FLOAT_LITERAL]: tokens.md#floating-point-literals

_Literal patterns_ match exactly the same value as what is created by the literal.
Since negative numbers are not [literals], literal patterns also accept an optional minus sign before the literal, which acts like the negation operator.

<div class="warning">

Floating-point literals are currently accepted, but due to the complexity of comparing them, they are going to be forbidden on literal patterns in a future version of Rust (see [issue #41620](https://github.com/rust-lang/rust/issues/41620)).

</div>

Literal patterns are always refutable.

Examples:

```rust
for i in -2..5 {
    match i {
        -1 => println!("It's minus one"),
        1 => println!("It's a one"),
        2|4 => println!("It's either a two or a four"),
        _ => println!("Matched none of the arms"),
    }
}
```

## Identifier patterns

> **<sup>Syntax</sup>**\
> _IdentifierPattern_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; `ref`<sup>?</sup> `mut`<sup>?</sup> [IDENTIFIER] (`@` [_PatternNoTopAlt_] ) <sup>?</sup>

Identifier patterns bind the value they match to a variable.
The identifier must be unique within the pattern.
The variable will shadow any variables of the same name in scope.
The scope of the new binding depends on the context of where the pattern is used (such as a `let` binding or a `match` arm).

Patterns that consist of only an identifier, possibly with a `mut`, match any value and bind it to that identifier.
This is the most commonly used pattern in variable declarations and parameters for functions and closures.

```rust
let mut variable = 10;
fn sum(x: i32, y: i32) -> i32 {
#    x + y
# }
```

To bind the matched value of a pattern to a variable, use the syntax `variable @ subpattern`.
For example, the following binds the value 2 to `e` (not the entire range: the range here is a range subpattern).

```rust
let x = 2;

match x {
    e @ 1 ..= 5 => println!("got a range element {}", e),
    _ => println!("anything"),
}
```

By default, identifier patterns bind a variable to a copy of or move from the matched value depending on whether the matched value implements [`Copy`].
This can be changed to bind to a reference by using the `ref` keyword, or to a mutable reference using `ref mut`. For example:

```rust
# let a = Some(10);
match a {
    None => (),
    Some(value) => (),
}

match a {
    None => (),
    Some(ref value) => (),
}
```

In the first match expression, the value is copied (or moved).
In the second match, a reference to the same memory location is bound to the variable value.
This syntax is needed because in destructuring subpatterns the `&` operator can't be applied to the value's fields.
For example, the following is not valid:

```rust,compile_fail
# struct Person {
#    name: String,
#    age: u8,
# }
# let value = Person { name: String::from("John"), age: 23 };
if let Person { name: &person_name, age: 18..=150 } = value { }
```

To make it valid, write the following:

```rust
# struct Person {
#    name: String,
#    age: u8,
# }
# let value = Person { name: String::from("John"), age: 23 };
if let Person {name: ref person_name, age: 18..=150 } = value { }
```

Thus, `ref` is not something that is being matched against.
Its objective is exclusively to make the matched binding a reference, instead of potentially copying or moving what was matched.

[Path patterns](#path-patterns) take precedence over identifier patterns.
It is an error if `ref` or `ref mut` is specified and the identifier shadows a constant.

Identifier patterns are irrefutable if the `@` subpattern is irrefutable or the subpattern is not specified.

### Binding modes

To service better ergonomics, patterns operate in different *binding modes* in order to make it easier to bind references to values.
When a reference value is matched by a non-reference pattern, it will be automatically treated as a `ref` or `ref mut` binding.
Example:

```rust
let x: &Option<i32> = &Some(3);
if let Some(y) = x {
    // y was converted to `ref y` and its type is &i32
}
```

*Non-reference patterns* include all patterns except bindings, [wildcard patterns](#wildcard-pattern) (`_`), [`const` patterns](#path-patterns) of reference types, and [reference patterns](#reference-patterns).

If a binding pattern does not explicitly have `ref`, `ref mut`, or `mut`, then it uses the *default binding mode* to determine how the variable is bound.
The default binding mode starts in "move" mode which uses move semantics.
When matching a pattern, the compiler starts from the outside of the pattern and works inwards.
Each time a reference is matched using a non-reference pattern, it will automatically dereference the value and update the default binding mode.
References will set the default binding mode to `ref`.
Mutable references will set the mode to `ref mut` unless the mode is already `ref` in which case it remains `ref`.
If the automatically dereferenced value is still a reference, it is dereferenced and this process repeats.

Move bindings and reference bindings can be mixed together in the same pattern.
Doing so will result in partial move of the object bound to and the object cannot be used afterwards.
This applies only if the type cannot be copied.

In the example below, `name` is moved out of `person`.
Trying to use `person` as a whole or `person.name` would result in an error because of *partial move*.

Example:

```rust
# struct Person {
#    name: String,
#    age: u8,
# }
# let person = Person{ name: String::from("John"), age: 23 };
// `name` is moved from person and `age` referenced
let Person { name, ref age } = person;
```

## Wildcard pattern

> **<sup>Syntax</sup>**\
> _WildcardPattern_ :\
> &nbsp;&nbsp; `_`

The _wildcard pattern_ (an underscore symbol) matches any value.
It is used to ignore values when they don't matter.
Inside other patterns it matches a single data field (as opposed to the `..` which matches the remaining fields).
Unlike identifier patterns, it does not copy, move or borrow the value it matches.

Examples:

```rust
# let x = 20;
let (a, _) = (10, x);   // the x is always matched by _
# assert_eq!(a, 10);

// ignore a function/closure param
let real_part = |a: f64, _: f64| { a };

// ignore a field from a struct
# struct RGBA {
#    r: f32,
#    g: f32,
#    b: f32,
#    a: f32,
# }
# let color = RGBA{r: 0.4, g: 0.1, b: 0.9, a: 0.5};
let RGBA{r: red, g: green, b: blue, a: _} = color;
# assert_eq!(color.r, red);
# assert_eq!(color.g, green);
# assert_eq!(color.b, blue);

// accept any Some, with any value
# let x = Some(10);
if let Some(_) = x {}
```

The wildcard pattern is always irrefutable.

## Rest patterns

> **<sup>Syntax</sup>**\
> _RestPattern_ :\
> &nbsp;&nbsp; `..`

The _rest pattern_ (the `..` token) acts as a variable-length pattern which matches zero or more elements that haven't been matched already before and after.
It may only be used in [tuple](#tuple-patterns), [tuple struct](#tuple-struct-patterns), and [slice](#slice-patterns) patterns, and may only appear once as one of the elements in those patterns.
It is also allowed in an [identifier pattern](#identifier-patterns) for [slice patterns](#slice-patterns) only.

The rest pattern is always irrefutable.

Examples:

```rust
# let words = vec!["a", "b", "c"];
# let slice = &words[..];
match slice {
    [] => println!("slice is empty"),
    [one] => println!("single element {}", one),
    [head, tail @ ..] => println!("head={} tail={:?}", head, tail),
}

match slice {
    // Ignore everything but the last element, which must be "!".
    [.., "!"] => println!("!!!"),

    // `start` is a slice of everything except the last element, which must be "z".
    [start @ .., "z"] => println!("starts with: {:?}", start),

    // `end` is a slice of everything but the first element, which must be "a".
    ["a", end @ ..] => println!("ends with: {:?}", end),

    // 'whole' is the entire slice and `last` is the final element
    whole @ [.., last] => println!("the last element of {:?} is {}", whole, last),

    rest => println!("{:?}", rest),
}

if let [.., penultimate, _] = slice {
    println!("next to last is {}", penultimate);
}

# let tuple = (1, 2, 3, 4, 5);
// Rest patterns may also be used in tuple and tuple struct patterns.
match tuple {
    (1, .., y, z) => println!("y={} z={}", y, z),
    (.., 5) => println!("tail must be 5"),
    (..) => println!("matches everything else"),
}
```

## Range patterns

> **<sup>Syntax</sup>**\
> _RangePattern_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _RangeInclusivePattern_\
> &nbsp;&nbsp; | _RangeFromPattern_\
> &nbsp;&nbsp; | _RangeToInclusivePattern_\
> &nbsp;&nbsp; | _ObsoleteRangePattern_
>
> _RangeInclusivePattern_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _RangePatternBound_ `..=` _RangePatternBound_
>
> _RangeFromPattern_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _RangePatternBound_ `..`
>
> _RangeToInclusivePattern_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; `..=` _RangePatternBound_
>
> _ObsoleteRangePattern_ :\
> &nbsp;&nbsp; _RangePatternBound_ `...` _RangePatternBound_
>
> _RangePatternBound_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [CHAR_LITERAL]\
> &nbsp;&nbsp; | [BYTE_LITERAL]\
> &nbsp;&nbsp; | `-`<sup>?</sup> [INTEGER_LITERAL]\
> &nbsp;&nbsp; | `-`<sup>?</sup> [FLOAT_LITERAL]\
> &nbsp;&nbsp; | [_PathExpression_]

*Range patterns* match scalar values within the range defined by their bounds.
They comprise a *sigil* (one of `..`, `..=`, or `...`) and a bound on one or both sides.
A bound on the left of the sigil is a *lower bound*.
A bound on the right is an *upper bound*.

A range pattern with both a lower and upper bound will match all values between and including both of its bounds.
It is written as its lower bound, followed by `..=`, followed by its upper bound.
The type of the range pattern is the type unification of its upper and lower bounds.

For example, a pattern `'m'..='p'` will match only the values `'m'`, `'n'`, `'o'`, and `'p'`.

The lower bound cannot be greater than the upper bound.
That is, in `a..=b`, a &le; b must be the case.
For example, it is an error to have a range pattern `10..=0`.

A range pattern with only a lower bound will match any value greater than or equal to the lower bound.
It is written as its lower bound followed by `..`, and has the same type as its lower bound.
For example, `1..` will match 1, 9, or 9001, or 9007199254740991 (if it is of an appropriate size), but not 0, and not negative numbers for signed integers.

A range pattern with only an upper bound matches any value less than or equal to the upper bound.
It is written as `..=` followed by its upper bound, and has the same type as its upper bound.
For example, `..=10` will match 10, 1, 0, and for signed integer types, all negative values.

Range patterns with only one bound cannot be used as the top-level pattern for subpatterns in [slice patterns](#slice-patterns).

The bounds is written as one of:

* A character, byte, integer, or float literal.
* A `-` followed by an integer or float literal.
* A [path]

If the bounds is written as a path, after macro resolution, the path must resolve to a constant item of the type `char`, an integer type, or a float type.

The type and value of the bounds is dependent upon how it is written out.
If the bounds is a [path], the pattern has the type and value of the [constant] the path resolves to.
If it is a literal, it has the type and value of the corresponding [literal expression].
If is a literal preceded by a `-`, it has the same type as the corresponding [literal expression] and the value of [negating] the value of the corresponding literal expression.

Examples:

```rust
# let c = 'f';
let valid_variable = match c {
    'a'..='z' => true,
    'A'..='Z' => true,
    'α'..='ω' => true,
    _ => false,
};

# let ph = 10;
println!("{}", match ph {
    0..=6 => "acid",
    7 => "neutral",
    8..=14 => "base",
    _ => unreachable!(),
});

# let uint: u32 = 5;
match uint {
    0 => "zero!",
    1.. => "positive number!",
};

// using paths to constants:
# const TROPOSPHERE_MIN : u8 = 6;
# const TROPOSPHERE_MAX : u8 = 20;
#
# const STRATOSPHERE_MIN : u8 = TROPOSPHERE_MAX + 1;
# const STRATOSPHERE_MAX : u8 = 50;
#
# const MESOSPHERE_MIN : u8 = STRATOSPHERE_MAX + 1;
# const MESOSPHERE_MAX : u8 = 85;
#
# let altitude = 70;
#
println!("{}", match altitude {
    TROPOSPHERE_MIN..=TROPOSPHERE_MAX => "troposphere",
    STRATOSPHERE_MIN..=STRATOSPHERE_MAX => "stratosphere",
    MESOSPHERE_MIN..=MESOSPHERE_MAX => "mesosphere",
    _ => "outer space, maybe",
});

# pub mod binary {
#     pub const MEGA : u64 = 1024*1024;
#     pub const GIGA : u64 = 1024*1024*1024;
# }
# let n_items = 20_832_425;
# let bytes_per_item = 12;
if let size @ binary::MEGA..=binary::GIGA = n_items * bytes_per_item {
    println!("It fits and occupies {} bytes", size);
}

# trait MaxValue {
#     const MAX: u64;
# }
# impl MaxValue for u8 {
#     const MAX: u64 = (1 << 8) - 1;
# }
# impl MaxValue for u16 {
#     const MAX: u64 = (1 << 16) - 1;
# }
# impl MaxValue for u32 {
#     const MAX: u64 = (1 << 32) - 1;
# }
// using qualified paths:
println!("{}", match 0xfacade {
    0 ..= <u8 as MaxValue>::MAX => "fits in a u8",
    0 ..= <u16 as MaxValue>::MAX => "fits in a u16",
    0 ..= <u32 as MaxValue>::MAX => "fits in a u32",
    _ => "too big",
});
```

Range patterns for fix-width integer and `char` types are irrefutable when they span the entire set of possible values of a type.
For example, `0u8..=255u8` is irrefutable.
The range of values for an integer type is the closed range from its minimum to maximum value.
The range of values for a `char` type are precisely those ranges containing all Unicode Scalar Values: `'\u{0000}'..='\u{D7FF}'` and `'\u{E000}'..='\u{10FFFF}'`.

Floating point range patterns are deprecated and may be removed in a future Rust release.
See [issue #41620](https://github.com/rust-lang/rust/issues/41620) for more information.

> **Edition Differences**: Before the 2021 edition, range patterns with both a lower and upper bound may also be written using `...` in place of `..=`, with the same meaning.

> **Note**: Although range patterns use the same syntax as [range expressions], there are no exclusive range patterns.
> That is, neither `x .. y` nor `.. x` are valid range patterns.

## Reference patterns

> **<sup>Syntax</sup>**\
> _ReferencePattern_ :\
> &nbsp;&nbsp; (`&`|`&&`) `mut`<sup>?</sup> [_PatternWithoutRange_]

Reference patterns dereference the pointers that are being matched and, thus, borrow them.

For example, these two matches on `x: &i32` are equivalent:

```rust
let int_reference = &3;

let a = match *int_reference { 0 => "zero", _ => "some" };
let b = match int_reference { &0 => "zero", _ => "some" };

assert_eq!(a, b);
```

The grammar production for reference patterns has to match the token `&&` to match a reference to a reference because it is a token by itself, not two `&` tokens.

Adding the `mut` keyword dereferences a mutable reference. The mutability must match the mutability of the reference.

Reference patterns are always irrefutable.

## Struct patterns

> **<sup>Syntax</sup>**\
> _StructPattern_ :\
> &nbsp;&nbsp; [_PathInExpression_] `{`\
> &nbsp;&nbsp; &nbsp;&nbsp; _StructPatternElements_ <sup>?</sup>\
> &nbsp;&nbsp; `}`
>
> _StructPatternElements_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; _StructPatternFields_ (`,` | `,` _StructPatternEtCetera_)<sup>?</sup>\
> &nbsp;&nbsp; | _StructPatternEtCetera_
>
> _StructPatternFields_ :\
> &nbsp;&nbsp; _StructPatternField_ (`,` _StructPatternField_) <sup>\*</sup>
>
> _StructPatternField_ :\
> &nbsp;&nbsp; [_OuterAttribute_] <sup>\*</sup>\
> &nbsp;&nbsp; (\
> &nbsp;&nbsp; &nbsp;&nbsp; &nbsp;&nbsp; [TUPLE_INDEX] `:` [_Pattern_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [IDENTIFIER] `:` [_Pattern_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | `ref`<sup>?</sup> `mut`<sup>?</sup> [IDENTIFIER]\
> &nbsp;&nbsp; )
>
> _StructPatternEtCetera_ :\
> &nbsp;&nbsp; [_OuterAttribute_] <sup>\*</sup>\
> &nbsp;&nbsp; `..`

[_OuterAttribute_]: attributes.md
[TUPLE_INDEX]: tokens.md#tuple-index

Struct patterns match struct values that match all criteria defined by its subpatterns.
They are also used to [destructure](#destructuring) a struct.

On a struct pattern, the fields are referenced by name, index (in the case of tuple structs) or ignored by use of `..`:

```rust
# struct Point {
#     x: u32,
#     y: u32,
# }
# let s = Point {x: 1, y: 1};
#
match s {
    Point {x: 10, y: 20} => (),
    Point {y: 10, x: 20} => (),    // order doesn't matter
    Point {x: 10, ..} => (),
    Point {..} => (),
}

# struct PointTuple (
#     u32,
#     u32,
# );
# let t = PointTuple(1, 2);
#
match t {
    PointTuple {0: 10, 1: 20} => (),
    PointTuple {1: 10, 0: 20} => (),   // order doesn't matter
    PointTuple {0: 10, ..} => (),
    PointTuple {..} => (),
}
```

If `..` is not used, it is required to match all fields:

```rust
# struct Struct {
#    a: i32,
#    b: char,
#    c: bool,
# }
# let mut struct_value = Struct{a: 10, b: 'X', c: false};
#
match struct_value {
    Struct{a: 10, b: 'X', c: false} => (),
    Struct{a: 10, b: 'X', ref c} => (),
    Struct{a: 10, b: 'X', ref mut c} => (),
    Struct{a: 10, b: 'X', c: _} => (),
    Struct{a: _, b: _, c: _} => (),
}
```

The `ref` and/or `mut` _IDENTIFIER_ syntax matches any value and binds it to a variable with the same name as the given field.

```rust
# struct Struct {
#    a: i32,
#    b: char,
#    c: bool,
# }
# let struct_value = Struct{a: 10, b: 'X', c: false};
#
let Struct{a: x, b: y, c: z} = struct_value;          // destructure all fields
```

A struct pattern is refutable when one of its subpatterns is refutable.

## Tuple struct patterns

> **<sup>Syntax</sup>**\
> _TupleStructPattern_ :\
> &nbsp;&nbsp; [_PathInExpression_] `(` _TupleStructItems_<sup>?</sup> `)`
>
> _TupleStructItems_ :\
> &nbsp;&nbsp; [_Pattern_]&nbsp;( `,` [_Pattern_] )<sup>\*</sup> `,`<sup>?</sup>

Tuple struct patterns match tuple struct and enum values that match all criteria defined by its subpatterns.
They are also used to [destructure](#destructuring) a tuple struct or enum value.

A tuple struct pattern is refutable when one of its subpatterns is refutable.

## Tuple patterns

> **<sup>Syntax</sup>**\
> _TuplePattern_ :\
> &nbsp;&nbsp; `(` _TuplePatternItems_<sup>?</sup> `)`
>
> _TuplePatternItems_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_Pattern_] `,`\
> &nbsp;&nbsp; | [_RestPattern_]\
> &nbsp;&nbsp; | [_Pattern_]&nbsp;(`,` [_Pattern_])<sup>+</sup> `,`<sup>?</sup>

Tuple patterns match tuple values that match all criteria defined by its subpatterns.
They are also used to [destructure](#destructuring) a tuple.

The form `(..)` with a single [_RestPattern_] is a special form that does not require a comma, and matches a tuple of any size.

The tuple pattern is refutable when one of its subpatterns is refutable.

An example of using tuple patterns:

```rust
let pair = (10, "ten");
let (a, b) = pair;

assert_eq!(a, 10);
assert_eq!(b, "ten");
```

## Grouped patterns

> **<sup>Syntax</sup>**\
> _GroupedPattern_ :\
> &nbsp;&nbsp; `(` [_Pattern_] `)`

Enclosing a pattern in parentheses can be used to explicitly control the precedence of compound patterns.
For example, a reference pattern next to a range pattern such as `&0..=5` is ambiguous and is not allowed, but can be expressed with parentheses.

```rust
let int_reference = &3;
match int_reference {
    &(0..=5) => (),
    _ => (),
}
```

## Slice patterns

> **<sup>Syntax</sup>**\
> _SlicePattern_ :\
> &nbsp;&nbsp; `[` _SlicePatternItems_<sup>?</sup> `]`
>
> _SlicePatternItems_ :\
> &nbsp;&nbsp; [_Pattern_] \(`,` [_Pattern_])<sup>\*</sup> `,`<sup>?</sup>

Slice patterns can match both arrays of fixed size and slices of dynamic size.

```rust
// Fixed size
let arr = [1, 2, 3];
match arr {
    [1, _, _] => "starts with one",
    [a, b, c] => "starts with something else",
};
```
```rust
// Dynamic size
let v = vec![1, 2, 3];
match v[..] {
    [a, b] => { /* this arm will not apply because the length doesn't match */ }
    [a, b, c] => { /* this arm will apply */ }
    _ => { /* this wildcard is required, since the length is not known statically */ }
};
```

Slice patterns are irrefutable when matching an array as long as each element is irrefutable.
When matching a slice, it is irrefutable only in the form with a single `..` [rest pattern](#rest-patterns) or [identifier pattern](#identifier-patterns) with the `..` rest pattern as a subpattern.

Within a slice, a range pattern without both lower and upper bound must be enclosed in parentheses, as in `(a..)`, to clarify it is intended to match against a single slice element.
A range pattern with both lower and upper bound, like `a..=b`, is not required to be enclosed in parentheses.

## Path patterns

> **<sup>Syntax</sup>**\
> _PathPattern_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_PathExpression_]

_Path patterns_ are patterns that refer either to constant values or
to structs or enum variants that have no fields.

Unqualified path patterns can refer to:

* enum variants
* structs
* constants
* associated constants

Qualified path patterns can only refer to associated constants.

Constants cannot be a union type.
Struct and enum constants must have `#[derive(PartialEq, Eq)]` (not merely implemented).

Path patterns are irrefutable when they refer to structs or an enum variant when the enum has only one variant or a constant whose type is irrefutable.
They are refutable when they refer to refutable constants or enum variants for enums with multiple variants.

## Or-patterns

_Or-patterns_ are patterns that match on one of two or more sub-patterns (for example `A | B | C`).
They can nest arbitrarily.
Syntactically, or-patterns are allowed in any of the places where other patterns are allowed (represented by the _Pattern_ production), with the exceptions of `let`-bindings and function and closure arguments (represented by the _PatternNoTopAlt_ production).

### Static semantics

1. Given a pattern `p | q` at some depth for some arbitrary patterns `p` and `q`, the pattern is considered ill-formed if:

   + the type inferred for `p` does not unify with the type inferred for `q`, or
   + the same set of bindings are not introduced in `p` and `q`, or
   + the type of any two bindings with the same name in `p` and `q` do not unify with respect to types or binding modes.

   Unification of types is in all instances aforementioned exact and implicit [type coercions] do not apply.

2. When type checking an expression `match e_s { a_1 => e_1, ... a_n => e_n }`,
   for each match arm `a_i` which contains a pattern of form `p_i | q_i`,
   the pattern `p_i | q_i` is considered ill formed if,
   at the depth `d` where it exists the fragment of `e_s` at depth `d`,
   the type of the expression fragment does not unify with `p_i | q_i`.

3. With respect to exhaustiveness checking, a pattern `p | q` is considered to cover `p` as well as `q`.
   For some constructor `c(x, ..)` the distributive law applies such that `c(p | q, ..rest)` covers the same set of value as `c(p, ..rest) | c(q, ..rest)` does.
   This can be applied recursively until there are no more nested patterns of form `p | q` other than those that exist at the top level.

   Note that by *"constructor"* we do not refer to tuple struct patterns, but rather we refer to a pattern for any product type.
   This includes enum variants, tuple structs, structs with named fields, arrays, tuples, and slices.

### Dynamic semantics

1. The dynamic semantics of pattern matching a scrutinee expression `e_s` against a pattern `c(p | q, ..rest)` at depth `d` where `c` is some constructor,
   `p` and `q` are arbitrary patterns,
   and `rest` is optionally any remaining potential factors in `c`,
   is defined as being the same as that of `c(p, ..rest) | c(q, ..rest)`.

### Precedence with other undelimited patterns

As shown elsewhere in this chapter, there are several types of patterns that are syntactically undelimited, including identifier patterns, reference patterns, and or-patterns.
Or-patterns always have the lowest-precedence.
This allows us to reserve syntactic space for a possible future type ascription feature and also to reduce ambiguity.
For example, `x @ A(..) | B(..)` will result in an error that `x` is not bound in all patterns.
`&A(x) | B(x)` will result in a type mismatch between `x` in the different subpatterns.

[_GroupedPattern_]: #grouped-patterns
[_IdentifierPattern_]: #identifier-patterns
[_LiteralPattern_]: #literal-patterns
[_MacroInvocation_]: macros.md#macro-invocation
[_ObsoleteRangePattern_]: #range-patterns
[_PathInExpression_]: paths.md#paths-in-expressions
[_PathExpression_]: expressions/path-expr.md
[_PathPattern_]: #path-patterns
[_Pattern_]: #patterns
[_PatternNoTopAlt_]: #patterns
[_PatternWithoutRange_]: #patterns
[_QualifiedPathInExpression_]: paths.md#qualified-paths
[_RangePattern_]: #range-patterns
[_ReferencePattern_]: #reference-patterns
[_RestPattern_]: #rest-patterns
[_SlicePattern_]: #slice-patterns
[_StructPattern_]: #struct-patterns
[_TuplePattern_]: #tuple-patterns
[_TupleStructPattern_]: #tuple-struct-patterns
[_WildcardPattern_]: #wildcard-pattern

[`Copy`]: special-types-and-traits.md#copy
[IDENTIFIER]: identifiers.md
[constant]: items/constant-items.md
[enums]: items/enumerations.md
[literals]: expressions/literal-expr.md
[literal expression]: expressions/literal-expr.md
[negating]: expressions/operator-expr.md#negation-operators
[path]: expressions/path-expr.md
[range expressions]: expressions/range-expr.md
[structs]: items/structs.md
[tuples]: types/tuple.md
[scrutinee]: glossary.md#scrutinee
[type coercions]: type-coercions.md
