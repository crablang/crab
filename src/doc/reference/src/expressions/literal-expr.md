# Literal expressions

> **<sup>Syntax</sup>**\
> _LiteralExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [CHAR_LITERAL]\
> &nbsp;&nbsp; | [STRING_LITERAL]\
> &nbsp;&nbsp; | [RAW_STRING_LITERAL]\
> &nbsp;&nbsp; | [BYTE_LITERAL]\
> &nbsp;&nbsp; | [BYTE_STRING_LITERAL]\
> &nbsp;&nbsp; | [RAW_BYTE_STRING_LITERAL]\
> &nbsp;&nbsp; | [C_STRING_LITERAL]\
> &nbsp;&nbsp; | [RAW_C_STRING_LITERAL]\
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

In the descriptions below, the _string representation_ of a token is the sequence of characters from the input which matched the token's production in a *Lexer* grammar snippet.

> **Note**: this string representation never includes a character `U+000D` (CR) immediately followed by `U+000A` (LF): this pair would have been previously transformed into a single `U+000A` (LF).

## Escapes

The descriptions of textual literal expressions below make use of several forms of _escape_.

Each form of escape is characterised by:
 * an _escape sequence_: a sequence of characters, which always begins with `U+005C` (`\`)
 * an _escaped value_: either a single character or an empty sequence of characters

In the definitions of escapes below:
 * An _octal digit_ is any of the characters in the range \[`0`-`7`].
 * A _hexadecimal digit_ is any of the characters in the ranges \[`0`-`9`], \[`a`-`f`], or \[`A`-`F`].

### Simple escapes

Each sequence of characters occurring in the first column of the following table is an escape sequence.

In each case, the escaped value is the character given in the corresponding entry in the second column.

| Escape sequence | Escaped value            |
|-----------------|--------------------------|
| `\0`            | U+0000 (NUL)             |
| `\t`            | U+0009 (HT)              |
| `\n`            | U+000A (LF)              |
| `\r`            | U+000D (CR)              |
| `\"`            | U+0022 (QUOTATION MARK)  |
| `\'`            | U+0027 (APOSTROPHE)      |
| `\\`            | U+005C (REVERSE SOLIDUS) |

### 8-bit escapes

The escape sequence consists of `\x` followed by two hexadecimal digits.

The escaped value is the character whose [Unicode scalar value] is the result of interpreting the final two characters in the escape sequence as a hexadecimal integer, as if by [`u8::from_str_radix`] with radix 16.

> **Note**: the escaped value therefore has a [Unicode scalar value] in the range of [`u8`][numeric types].

### 7-bit escapes

The escape sequence consists of `\x` followed by an octal digit then a hexadecimal digit.

The escaped value is the character whose [Unicode scalar value] is the result of interpreting the final two characters in the escape sequence as a hexadecimal integer, as if by [`u8::from_str_radix`] with radix 16.

### Unicode escapes

The escape sequence consists of `\u{`, followed by a sequence of characters each of which is a hexadecimal digit or `_`, followed by `}`.

The escaped value is the character whose [Unicode scalar value] is the result of interpreting the hexadecimal digits contained in the escape sequence as a hexadecimal integer, as if by [`u32::from_str_radix`] with radix 16.

> **Note**: the permitted forms of a [CHAR_LITERAL] or [STRING_LITERAL] token ensure that there is such a character.

### String continuation escapes

The escape sequence consists of `\` followed immediately by `U+000A` (LF), and all following whitespace characters before the next non-whitespace character.
For this purpose, the whitespace characters are `U+0009` (HT), `U+000A` (LF), `U+000D` (CR), and `U+0020` (SPACE).

The escaped value is an empty sequence of characters.

> **Note**: The effect of this form of escape is that a string continuation skips following whitespace, including additional newlines.
> Thus `a`, `b` and `c` are equal:
> ```rust
> let a = "foobar";
> let b = "foo\
>          bar";
> let c = "foo\
>
>      bar";
>
> assert_eq!(a, b);
> assert_eq!(b, c);
> ```
>
> Skipping additional newlines (as in example c) is potentially confusing and unexpected.
> This behavior may be adjusted in the future.
> Until a decision is made, it is recommended to avoid relying on skipping multiple newlines with line continuations.
> See [this issue](https://github.com/rust-lang/reference/pull/1042) for more information.

## Character literal expressions

A character literal expression consists of a single [CHAR_LITERAL] token.

The expression's type is the primitive [`char`][textual types] type.

The token must not have a suffix.

The token's _literal content_ is the sequence of characters following the first `U+0027` (`'`) and preceding the last `U+0027` (`'`) in the string representation of the token.

The literal expression's _represented character_ is derived from the literal content as follows:

* If the literal content is one of the following forms of escape sequence, the represented character is the escape sequence's escaped value:
    * [Simple escapes]
    * [7-bit escapes]
    * [Unicode escapes]

* Otherwise the represented character is the single character that makes up the literal content.

The expression's value is the [`char`][textual types] corresponding to the represented character's [Unicode scalar value].

> **Note**: the permitted forms of a [CHAR_LITERAL] token ensure that these rules always produce a single character.

Examples of character literal expressions:

```rust
'R';                               // R
'\'';                              // '
'\x52';                            // R
'\u{00E6}';                        // LATIN SMALL LETTER AE (U+00E6)
```

## String literal expressions

A string literal expression consists of a single [STRING_LITERAL] or [RAW_STRING_LITERAL] token.

The expression's type is a shared reference (with `static` lifetime) to the primitive [`str`][textual types] type.
That is, the type is `&'static str`.

The token must not have a suffix.

The token's _literal content_ is the sequence of characters following the first `U+0022` (`"`) and preceding the last `U+0022` (`"`) in the string representation of the token.

The literal expression's _represented string_ is a sequence of characters derived from the literal content as follows:

* If the token is a [STRING_LITERAL], each escape sequence of any of the following forms occurring in the literal content is replaced by the escape sequence's escaped value.
    * [Simple escapes]
    * [7-bit escapes]
    * [Unicode escapes]
    * [String continuation escapes]

  These replacements take place in left-to-right order.
  For example, the token `"\\x41"` is converted to the characters `\` `x` `4` `1`.

* If the token is a [RAW_STRING_LITERAL], the represented string is identical to the literal content.

The expression's value is a reference to a statically allocated [`str`][textual types] containing the UTF-8 encoding of the represented string.

Examples of string literal expressions:

```rust
"foo"; r"foo";                     // foo
"\"foo\""; r#""foo""#;             // "foo"

"foo #\"# bar";
r##"foo #"# bar"##;                // foo #"# bar

"\x52"; "R"; r"R";                 // R
"\\x52"; r"\x52";                  // \x52
```

## Byte literal expressions

A byte literal expression consists of a single [BYTE_LITERAL] token.

The expression's type is the primitive [`u8`][numeric types] type.

The token must not have a suffix.

The token's _literal content_ is the sequence of characters following the first `U+0027` (`'`) and preceding the last `U+0027` (`'`) in the string representation of the token.

The literal expression's _represented character_ is derived from the literal content as follows:

* If the literal content is one of the following forms of escape sequence, the represented character is the escape sequence's escaped value:
    * [Simple escapes]
    * [8-bit escapes]

* Otherwise the represented character is the single character that makes up the literal content.

The expression's value is the represented character's [Unicode scalar value].

> **Note**: the permitted forms of a [BYTE_LITERAL] token ensure that these rules always produce a single character, whose Unicode scalar value is in the range of [`u8`][numeric types].

Examples of byte literal expressions:

```rust
b'R';                              // 82
b'\'';                             // 39
b'\x52';                           // 82
b'\xA0';                           // 160
```

## Byte string literal expressions

A byte string literal expression consists of a single [BYTE_STRING_LITERAL] or [RAW_BYTE_STRING_LITERAL] token.

The expression's type is a shared reference (with `static` lifetime) to an array whose element type is [`u8`][numeric types].
That is, the type is `&'static [u8; N]`, where `N` is the number of bytes in the represented string described below.

The token must not have a suffix.

The token's _literal content_ is the sequence of characters following the first `U+0022` (`"`) and preceding the last `U+0022` (`"`) in the string representation of the token.

The literal expression's _represented string_ is a sequence of characters derived from the literal content as follows:

* If the token is a [BYTE_STRING_LITERAL], each escape sequence of any of the following forms occurring in the literal content is replaced by the escape sequence's escaped value.
    * [Simple escapes]
    * [8-bit escapes]
    * [String continuation escapes]

  These replacements take place in left-to-right order.
  For example, the token `b"\\x41"` is converted to the characters `\` `x` `4` `1`.

* If the token is a [RAW_BYTE_STRING_LITERAL], the represented string is identical to the literal content.

The expression's value is a reference to a statically allocated array containing the [Unicode scalar values] of the characters in the represented string, in the same order.

> **Note**: the permitted forms of [BYTE_STRING_LITERAL] and [RAW_BYTE_STRING_LITERAL] tokens ensure that these rules always produce array element values in the range of [`u8`][numeric types].

Examples of byte string literal expressions:

```rust
b"foo"; br"foo";                     // foo
b"\"foo\""; br#""foo""#;             // "foo"

b"foo #\"# bar";
br##"foo #"# bar"##;                 // foo #"# bar

b"\x52"; b"R"; br"R";                // R
b"\\x52"; br"\x52";                  // \x52
```

## C string literal expressions

A C string literal expression consists of a single [C_STRING_LITERAL] or [RAW_C_STRING_LITERAL] token.

The expression's type is a shared reference (with `static` lifetime) to the standard library [CStr] type.
That is, the type is `&'static core::ffi::CStr`.

The token must not have a suffix.

The token's _literal content_ is the sequence of characters following the first `"` and preceding the last `"` in the string representation of the token.

The literal expression's _represented bytes_ are a sequence of bytes derived from the literal content as follows:

* If the token is a [C_STRING_LITERAL], the literal content is treated as a sequence of items, each of which is either a single Unicode character other than `\` or an [escape].
The sequence of items is converted to a sequence of bytes as follows:
  * Each single Unicode character contributes its UTF-8 representation.
  * Each [simple escape] contributes the [Unicode scalar value] of its escaped value.
  * Each [8-bit escape] contributes a single byte containing the [Unicode scalar value] of its escaped value.
  * Each [unicode escape] contributes the UTF-8 representation of its escaped value.
  * Each [string continuation escape] contributes no bytes.

* If the token is a [RAW_C_STRING_LITERAL], the represented bytes are the UTF-8 encoding of the literal content.

> **Note**: the permitted forms of [C_STRING_LITERAL] and [RAW_C_STRING_LITERAL] tokens ensure that the represented bytes never include a null byte.

The expression's value is a reference to a statically allocated [CStr] whose array of bytes contains the represented bytes followed by a null byte.

Examples of C string literal expressions:

```rust
c"foo"; cr"foo";                     // foo
c"\"foo\""; cr#""foo""#;             // "foo"

c"foo #\"# bar";
cr##"foo #"# bar"##;                 // foo #"# bar

c"\x52"; c"R"; cr"R";                // R
c"\\x52"; cr"\x52";                  // \x52

c"æ";                                // LATIN SMALL LETTER AE (U+00E6)
c"\u{00E6}";                         // LATIN SMALL LETTER AE (U+00E6)
c"\xC3\xA6";                         // LATIN SMALL LETTER AE (U+00E6)

c"\xE6".to_bytes();                  // [230]
c"\u{00E6}".to_bytes();              // [195, 166]
```

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


[Escape]: #escapes
[Simple escape]: #simple-escapes
[Simple escapes]: #simple-escapes
[8-bit escape]: #8-bit-escapes
[8-bit escapes]: #8-bit-escapes
[7-bit escape]: #7-bit-escapes
[7-bit escapes]: #7-bit-escapes
[Unicode escape]: #unicode-escapes
[Unicode escapes]: #unicode-escapes
[String continuation escape]: #string-continuation-escapes
[String continuation escapes]: #string-continuation-escapes
[boolean type]: ../types/boolean.md
[constant expression]: ../const_eval.md#constant-expressions
[CStr]: ../../core/ffi/struct.CStr.html
[floating-point types]: ../types/numeric.md#floating-point-types
[lint check]: ../attributes/diagnostics.md#lint-check-attributes
[literal tokens]: ../tokens.md#literals
[numeric cast]: operator-expr.md#numeric-cast
[numeric types]: ../types/numeric.md
[suffix]: ../tokens.md#suffixes
[negation operator]: operator-expr.md#negation-operators
[overflow]: operator-expr.md#overflow
[textual types]: ../types/textual.md
[Unicode scalar value]: http://www.unicode.org/glossary/#unicode_scalar_value
[Unicode scalar values]: http://www.unicode.org/glossary/#unicode_scalar_value
[`f32::from_str`]: ../../core/primitive.f32.md#method.from_str
[`f32::INFINITY`]: ../../core/primitive.f32.md#associatedconstant.INFINITY
[`f32::NAN`]: ../../core/primitive.f32.md#associatedconstant.NAN
[`f64::from_str`]: ../../core/primitive.f64.md#method.from_str
[`f64::INFINITY`]: ../../core/primitive.f64.md#associatedconstant.INFINITY
[`f64::NAN`]: ../../core/primitive.f64.md#associatedconstant.NAN
[`u8::from_str_radix`]: ../../core/primitive.u8.md#method.from_str_radix
[`u32::from_str_radix`]: ../../core/primitive.u32.md#method.from_str_radix
[`u128::from_str_radix`]: ../../core/primitive.u128.md#method.from_str_radix
[CHAR_LITERAL]: ../tokens.md#character-literals
[STRING_LITERAL]: ../tokens.md#string-literals
[RAW_STRING_LITERAL]: ../tokens.md#raw-string-literals
[BYTE_LITERAL]: ../tokens.md#byte-literals
[BYTE_STRING_LITERAL]: ../tokens.md#byte-string-literals
[RAW_BYTE_STRING_LITERAL]: ../tokens.md#raw-byte-string-literals
[C_STRING_LITERAL]: ../tokens.md#c-string-literals
[RAW_C_STRING_LITERAL]: ../tokens.md#raw-c-string-literals
[INTEGER_LITERAL]: ../tokens.md#integer-literals
[FLOAT_LITERAL]: ../tokens.md#floating-point-literals
