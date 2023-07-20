# Literal expressions

> **<sup>Syntax</sup>**\
> _LiteralExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [CHAR_LITERAL]\
> &nbsp;&nbsp; | [STRING_LITERAL]\
> &nbsp;&nbsp; | [RAW_STRING_LITERAL]\
> &nbsp;&nbsp; | [BYTE_LITERAL]\
> &nbsp;&nbsp; | [BYTE_STRING_LITERAL]\
> &nbsp;&nbsp; | [RAW_BYTE_STRING_LITERAL]\
> &nbsp;&nbsp; | [INTEGER_LITERAL]\
> &nbsp;&nbsp; | [FLOAT_LITERAL]\
> &nbsp;&nbsp; | `true` | `false`

A _literal expression_ is an expression consisting of a single token, rather than a sequence of tokens, that immediately and directly denotes the value it evaluates to, rather than referring to it by name or some other evaluation rule.

A literal is a form of [constant expression], so is evaluated (primarily) at compile time.

Each of the lexical [literal][literal tokens] forms described earlier can make up a literal expression, as can the keywords `true` and `false`.

```rust
"hello";   // string type
'5';       // character type
5;         // integer type
```

## Character literal expressions

A character literal expression consists of a single [CHAR_LITERAL] token.

> **Note**: This section is incomplete.

## String literal expressions

A string literal expression consists of a single [STRING_LITERAL] or [RAW_STRING_LITERAL] token.

> **Note**: This section is incomplete.

## Byte literal expressions

A byte literal expression consists of a single [BYTE_LITERAL] token.

> **Note**: This section is incomplete.

## Byte string literal expressions

A string literal expression consists of a single [BYTE_STRING_LITERAL] or [RAW_BYTE_STRING_LITERAL] token.

> **Note**: This section is incomplete.

## Integer literal expressions

An integer literal expression consists of a single [INTEGER_LITERAL] token.

If the token has a [suffix], the suffix must be the name of one of the [primitive integer types][numeric types]: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`, `usize`, or `isize`, and the expression has that type.

If the token has no suffix, the expression's type is determined by type inference:

* If an integer type can be _uniquely_ determined from the surrounding program context, the expression has that type.

* If the program context under-constrains the type, it defaults to the signed 32-bit integer `i32`.

* If the program context over-constrains the type, it is considered a static type error.

Examples of integer literal expressions:

```rust
123;                               // type i32
123i32;                            // type i32
123u32;                            // type u32
123_u32;                           // type u32
let a: u64 = 123;                  // type u64

0xff;                              // type i32
0xff_u8;                           // type u8

0o70;                              // type i32
0o70_i16;                          // type i16

0b1111_1111_1001_0000;             // type i32
0b1111_1111_1001_0000i64;          // type i64

0usize;                            // type usize
```

The value of the expression is determined from the string representation of the token as follows:

* An integer radix is chosen by inspecting the first two characters of the string, as follows:

    * `0b` indicates radix 2
    * `0o` indicates radix 8
    * `0x` indicates radix 16
    * otherwise the radix is 10.

* If the radix is not 10, the first two characters are removed from the string.

* Any suffix is removed from the string.

* Any underscores are removed from the string.

* The string is converted to a `u128` value as if by [`u128::from_str_radix`] with the chosen radix.
If the value does not fit in `u128`, it is a compiler error.

* The `u128` value is converted to the expression's type via a [numeric cast].

> **Note**: The final cast will truncate the value of the literal if it does not fit in the expression's type.
> `rustc` includes a [lint check] named `overflowing_literals`, defaulting to `deny`, which rejects expressions where this occurs.

> **Note**: `-1i8`, for example, is an application of the [negation operator] to the literal expression `1i8`, not a single integer literal expression.
> See [Overflow] for notes on representing the most negative value for a signed type.

## Floating-point literal expressions

A floating-point literal expression has one of two forms:
 * a single [FLOAT_LITERAL] token
 * a single [INTEGER_LITERAL] token which has a suffix and no radix indicator

If the token has a [suffix], the suffix must be the name of one of the [primitive floating-point types][floating-point types]: `f32` or `f64`, and the expression has that type.

If the token has no suffix, the expression's type is determined by type inference:

* If a floating-point type can be _uniquely_ determined from the surrounding program context, the expression has that type.

* If the program context under-constrains the type, it defaults to `f64`.

* If the program context over-constrains the type, it is considered a static type error.

Examples of floating-point literal expressions:

```rust
123.0f64;        // type f64
0.1f64;          // type f64
0.1f32;          // type f32
12E+99_f64;      // type f64
5f32;            // type f32
let x: f64 = 2.; // type f64
```

The value of the expression is determined from the string representation of the token as follows:

* Any suffix is removed from the string.

* Any underscores are removed from the string.

* The string is converted to the expression's type as if by [`f32::from_str`] or [`f64::from_str`].

> **Note**: `-1.0`, for example, is an application of the [negation operator] to the literal expression `1.0`, not a single floating-point literal expression.

> **Note**: `inf` and `NaN` are not literal tokens.
> The [`f32::INFINITY`], [`f64::INFINITY`], [`f32::NAN`], and [`f64::NAN`] constants can be used instead of literal expressions.
> In `rustc`, a literal large enough to be evaluated as infinite will trigger the `overflowing_literals` lint check.

## Boolean literal expressions

A boolean literal expression consists of one of the keywords `true` or `false`.

The expression's type is the primitive [boolean type], and its value is:
 * true if the keyword is `true`
 * false if the keyword is `false`


[boolean type]: ../types/boolean.md
[constant expression]: ../const_eval.md#constant-expressions
[floating-point types]: ../types/numeric.md#floating-point-types
[lint check]: ../attributes/diagnostics.md#lint-check-attributes
[literal tokens]: ../tokens.md#literals
[numeric cast]: operator-expr.md#numeric-cast
[numeric types]: ../types/numeric.md
[suffix]: ../tokens.md#suffixes
[negation operator]: operator-expr.md#negation-operators
[overflow]: operator-expr.md#overflow
[`f32::from_str`]: ../../core/primitive.f32.md#method.from_str
[`f32::INFINITY`]: ../../core/primitive.f32.md#associatedconstant.INFINITY
[`f32::NAN`]: ../../core/primitive.f32.md#associatedconstant.NAN
[`f64::from_str`]: ../../core/primitive.f64.md#method.from_str
[`f64::INFINITY`]: ../../core/primitive.f64.md#associatedconstant.INFINITY
[`f64::NAN`]: ../../core/primitive.f64.md#associatedconstant.NAN
[`u128::from_str_radix`]: ../../core/primitive.u128.md#method.from_str_radix
[CHAR_LITERAL]: ../tokens.md#character-literals
[STRING_LITERAL]: ../tokens.md#string-literals
[RAW_STRING_LITERAL]: ../tokens.md#raw-string-literals
[BYTE_LITERAL]: ../tokens.md#byte-literals
[BYTE_STRING_LITERAL]: ../tokens.md#byte-string-literals
[RAW_BYTE_STRING_LITERAL]: ../tokens.md#raw-byte-string-literals
[INTEGER_LITERAL]: ../tokens.md#integer-literals
[FLOAT_LITERAL]: ../tokens.md#floating-point-literals
