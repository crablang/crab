# Operator expressions

> **<sup>Syntax</sup>**\
> _OperatorExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_BorrowExpression_]\
> &nbsp;&nbsp; | [_DereferenceExpression_]\
> &nbsp;&nbsp; | [_ErrorPropagationExpression_]\
> &nbsp;&nbsp; | [_NegationExpression_]\
> &nbsp;&nbsp; | [_ArithmeticOrLogicalExpression_]\
> &nbsp;&nbsp; | [_ComparisonExpression_]\
> &nbsp;&nbsp; | [_LazyBooleanExpression_]\
> &nbsp;&nbsp; | [_TypeCastExpression_]\
> &nbsp;&nbsp; | [_AssignmentExpression_]\
> &nbsp;&nbsp; | [_CompoundAssignmentExpression_]

Operators are defined for built in types by the Rust language.
Many of the following operators can also be overloaded using traits in `std::ops` or `std::cmp`.

## Overflow

Integer operators will panic when they overflow when compiled in debug mode.
The `-C debug-assertions` and `-C overflow-checks` compiler flags can be used to control this more directly.
The following things are considered to be overflow:

* When `+`, `*` or binary `-` create a value greater than the maximum value, or less than the minimum value that can be stored.
* Applying unary `-` to the most negative value of any signed integer type, unless the operand is a [literal expression] (or a literal expression standing alone inside one or more [grouped expressions][grouped expression]).
* Using `/` or `%`, where the left-hand argument is the smallest integer of a signed integer type and the right-hand argument is `-1`.
  These checks occur even when `-C overflow-checks` is disabled, for legacy reasons.
* Using `<<` or `>>` where the right-hand argument is greater than or equal to the number of bits in the type of the left-hand argument, or is negative.

> **Note**: The exception for literal expressions behind unary `-` means that forms such as `-128_i8` or `let j: i8 = -(128)` never cause a panic and have the expected value of -128.
>
> In these cases, the literal expression already has the most negative value for its type (for example, `128_i8` has the value -128) because integer literals are truncated to their type per the description in [Integer literal expressions][literal expression].
>
> Negation of these most negative values leaves the value unchanged due to two's complement overflow conventions.
>
> In `rustc`, these most negative expressions are also ignored by the `overflowing_literals` lint check.

## Borrow operators

> **<sup>Syntax</sup>**\
> _BorrowExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; (`&`|`&&`) [_Expression_]\
> &nbsp;&nbsp; | (`&`|`&&`) `mut` [_Expression_]

The `&` (shared borrow) and `&mut` (mutable borrow) operators are unary prefix operators.
When applied to a [place expression], this expressions produces a reference (pointer) to the location that the value refers to.
The memory location is also placed into a borrowed state for the duration of the reference.
For a shared borrow (`&`), this implies that the place may not be mutated, but it may be read or shared again.
For a mutable borrow (`&mut`), the place may not be accessed in any way until the borrow expires.
`&mut` evaluates its operand in a mutable place expression context.
If the `&` or `&mut` operators are applied to a [value expression], then a [temporary value] is created.

These operators cannot be overloaded.

```rust
{
    // a temporary with value 7 is created that lasts for this scope.
    let shared_reference = &7;
}
let mut array = [-2, 3, 9];
{
    // Mutably borrows `array` for this scope.
    // `array` may only be used through `mutable_reference`.
    let mutable_reference = &mut array;
}
```

Even though `&&` is a single token ([the lazy 'and' operator](#lazy-boolean-operators)), when used in the context of borrow expressions it works as two borrows:

```rust
// same meanings:
let a = &&  10;
let a = & & 10;

// same meanings:
let a = &&&&  mut 10;
let a = && && mut 10;
let a = & & & & mut 10;
```

### Raw address-of operators

Related to the borrow operators are the *raw address-of operators*, which do not have first-class syntax, but are exposed via the macros [`ptr::addr_of!(expr)`][addr_of] and [`ptr::addr_of_mut!(expr)`][addr_of_mut].
The expression `expr` is evaluated in place expression context.
`ptr::addr_of!(expr)` then creates a const raw pointer of type `*const T` to the given place, and `ptr::addr_of_mut!(expr)` creates a mutable raw pointer of type `*mut T`.

The raw address-of operators must be used instead of a borrow operator whenever the place expression could evaluate to a place that is not properly aligned or does not store a valid value as determined by its type, or whenever creating a reference would introduce incorrect aliasing assumptions.
In those situations, using a borrow operator would cause [undefined behavior] by creating an invalid reference, but a raw pointer may still be constructed using an address-of operator.

The following is an example of creating a raw pointer to an unaligned place through a `packed` struct:

```rust
use std::ptr;

#[repr(packed)]
struct Packed {
    f1: u8,
    f2: u16,
}

let packed = Packed { f1: 1, f2: 2 };
// `&packed.f2` would create an unaligned reference, and thus be Undefined Behavior!
let raw_f2 = ptr::addr_of!(packed.f2);
assert_eq!(unsafe { raw_f2.read_unaligned() }, 2);
```

The following is an example of creating a raw pointer to a place that does not contain a valid value:

```rust
use std::{ptr, mem::MaybeUninit};

struct Demo {
    field: bool,
}

let mut uninit = MaybeUninit::<Demo>::uninit();
// `&uninit.as_mut().field` would create a reference to an uninitialized `bool`,
// and thus be Undefined Behavior!
let f1_ptr = unsafe { ptr::addr_of_mut!((*uninit.as_mut_ptr()).field) };
unsafe { f1_ptr.write(true); }
let init = unsafe { uninit.assume_init() };
```


## The dereference operator

> **<sup>Syntax</sup>**\
> _DereferenceExpression_ :\
> &nbsp;&nbsp; `*` [_Expression_]

The `*` (dereference) operator is also a unary prefix operator.
When applied to a [pointer](../types/pointer.md) it denotes the pointed-to location.
If the expression is of type `&mut T` or `*mut T`, and is either a local variable, a (nested) field of a local variable or is a mutable [place expression], then the resulting memory location can be assigned to.
Dereferencing a raw pointer requires `unsafe`.

On non-pointer types `*x` is equivalent to `*std::ops::Deref::deref(&x)` in an [immutable place expression context](../expressions.md#mutability) and `*std::ops::DerefMut::deref_mut(&mut x)` in a mutable place expression context.

```rust
let x = &7;
assert_eq!(*x, 7);
let y = &mut 9;
*y = 11;
assert_eq!(*y, 11);
```

## The question mark operator

> **<sup>Syntax</sup>**\
> _ErrorPropagationExpression_ :\
> &nbsp;&nbsp; [_Expression_] `?`

The question mark operator (`?`) unwraps valid values or returns erroneous values, propagating them to the calling function.
It is a unary postfix operator that can only be applied to the types `Result<T, E>` and `Option<T>`.

When applied to values of the `Result<T, E>` type, it propagates errors.
If the value is `Err(e)`, then it will return `Err(From::from(e))` from the enclosing function or closure.
If applied to `Ok(x)`, then it will unwrap the value to evaluate to `x`.

```rust
# use std::num::ParseIntError;
fn try_to_parse() -> Result<i32, ParseIntError> {
    let x: i32 = "123".parse()?; // x = 123
    let y: i32 = "24a".parse()?; // returns an Err() immediately
    Ok(x + y)                    // Doesn't run.
}

let res = try_to_parse();
println!("{:?}", res);
# assert!(res.is_err())
```

When applied to values of the `Option<T>` type, it propagates `None`s.
If the value is `None`, then it will return `None`.
If applied to `Some(x)`, then it will unwrap the value to evaluate to `x`.

```rust
fn try_option_some() -> Option<u8> {
    let val = Some(1)?;
    Some(val)
}
assert_eq!(try_option_some(), Some(1));

fn try_option_none() -> Option<u8> {
    let val = None?;
    Some(val)
}
assert_eq!(try_option_none(), None);
```

`?` cannot be overloaded.

## Negation operators

> **<sup>Syntax</sup>**\
> _NegationExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; `-` [_Expression_]\
> &nbsp;&nbsp; | `!` [_Expression_]

These are the last two unary operators.
This table summarizes the behavior of them on primitive types and which traits are used to overload these operators for other types.
Remember that signed integers are always represented using two's complement.
The operands of all of these operators are evaluated in [value expression context][value expression] so are moved or copied.

| Symbol | Integer     | `bool`        | Floating Point | Overloading Trait  |
|--------|-------------|-------------- |----------------|--------------------|
| `-`    | Negation*   |               | Negation       | `std::ops::Neg`    |
| `!`    | Bitwise NOT | [Logical NOT] |                | `std::ops::Not`    |

\* Only for signed integer types.

Here are some example of these operators

```rust
let x = 6;
assert_eq!(-x, -6);
assert_eq!(!x, -7);
assert_eq!(true, !false);
```

## Arithmetic and Logical Binary Operators

> **<sup>Syntax</sup>**\
> _ArithmeticOrLogicalExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_Expression_] `+` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `-` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `*` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `/` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `%` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `&` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `|` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `^` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `<<` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `>>` [_Expression_]

Binary operators expressions are all written with infix notation.
This table summarizes the behavior of arithmetic and logical binary operators on primitive types and which traits are used to overload these operators for other types.
Remember that signed integers are always represented using two's complement.
The operands of all of these operators are evaluated in [value expression context][value expression] so are moved or copied.

| Symbol | Integer                 | `bool`        | Floating Point | Overloading Trait  | Overloading Compound Assignment Trait |
|--------|-------------------------|---------------|----------------|--------------------| ------------------------------------- |
| `+`    | Addition                |               | Addition       | `std::ops::Add`    | `std::ops::AddAssign`                 |
| `-`    | Subtraction             |               | Subtraction    | `std::ops::Sub`    | `std::ops::SubAssign`                 |
| `*`    | Multiplication          |               | Multiplication | `std::ops::Mul`    | `std::ops::MulAssign`                 |
| `/`    | Division*†              |               | Division       | `std::ops::Div`    | `std::ops::DivAssign`                 |
| `%`    | Remainder**†            |               | Remainder      | `std::ops::Rem`    | `std::ops::RemAssign`                 |
| `&`    | Bitwise AND             | [Logical AND] |                | `std::ops::BitAnd` | `std::ops::BitAndAssign`              |
| <code>&#124;</code> | Bitwise OR | [Logical OR]  |                | `std::ops::BitOr`  | `std::ops::BitOrAssign`               |
| `^`    | Bitwise XOR             | [Logical XOR] |                | `std::ops::BitXor` | `std::ops::BitXorAssign`              |
| `<<`   | Left Shift              |               |                | `std::ops::Shl`    | `std::ops::ShlAssign`                 |
| `>>`   | Right Shift***          |               |                | `std::ops::Shr`    |  `std::ops::ShrAssign`                |

\* Integer division rounds towards zero.

\*\* Rust uses a remainder defined with [truncating division](https://en.wikipedia.org/wiki/Modulo_operation#Variants_of_the_definition). Given `remainder = dividend % divisor`, the remainder will have the same sign as the dividend.

\*\*\* Arithmetic right shift on signed integer types, logical right shift on
unsigned integer types.

† For integer types, division by zero panics.

Here are examples of these operators being used.

```rust
assert_eq!(3 + 6, 9);
assert_eq!(5.5 - 1.25, 4.25);
assert_eq!(-5 * 14, -70);
assert_eq!(14 / 3, 4);
assert_eq!(100 % 7, 2);
assert_eq!(0b1010 & 0b1100, 0b1000);
assert_eq!(0b1010 | 0b1100, 0b1110);
assert_eq!(0b1010 ^ 0b1100, 0b110);
assert_eq!(13 << 3, 104);
assert_eq!(-10 >> 2, -3);
```

## Comparison Operators

> **<sup>Syntax</sup>**\
> _ComparisonExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_Expression_] `==` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `!=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `>` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `<` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `>=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `<=` [_Expression_]

Comparison operators are also defined both for primitive types and many types in the standard library.
Parentheses are required when chaining comparison operators. For example, the expression `a == b == c` is invalid and may be written as `(a == b) == c`.

Unlike arithmetic and logical operators, the traits for overloading these operators are used more generally to show how a type may be compared and will likely be assumed to define actual comparisons by functions that use these traits as bounds.
Many functions and macros in the standard library can then use that assumption (although not to ensure safety).
Unlike the arithmetic and logical operators above, these operators implicitly take shared borrows of their operands, evaluating them in [place expression context][place expression]:

```rust
# let a = 1;
# let b = 1;
a == b;
// is equivalent to
::std::cmp::PartialEq::eq(&a, &b);
```

This means that the operands don't have to be moved out of.

| Symbol | Meaning                  | Overloading method         |
|--------|--------------------------|----------------------------|
| `==`   | Equal                    | `std::cmp::PartialEq::eq`  |
| `!=`   | Not equal                | `std::cmp::PartialEq::ne`  |
| `>`    | Greater than             | `std::cmp::PartialOrd::gt` |
| `<`    | Less than                | `std::cmp::PartialOrd::lt` |
| `>=`   | Greater than or equal to | `std::cmp::PartialOrd::ge` |
| `<=`   | Less than or equal to    | `std::cmp::PartialOrd::le` |

Here are examples of the comparison operators being used.

```rust
assert!(123 == 123);
assert!(23 != -12);
assert!(12.5 > 12.2);
assert!([1, 2, 3] < [1, 3, 4]);
assert!('A' <= 'B');
assert!("World" >= "Hello");
```

## Lazy boolean operators

> **<sup>Syntax</sup>**\
> _LazyBooleanExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_Expression_] `||` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `&&` [_Expression_]

The operators `||` and `&&` may be applied to operands of boolean type.
The `||` operator denotes logical 'or', and the `&&` operator denotes logical 'and'.
They differ from `|` and `&` in that the right-hand operand is only evaluated when the left-hand operand does not already determine the result of the expression.
That is, `||` only evaluates its right-hand operand when the left-hand operand evaluates to `false`, and `&&` only when it evaluates to `true`.

```rust
let x = false || true; // true
let y = false && panic!(); // false, doesn't evaluate `panic!()`
```

## Type cast expressions

> **<sup>Syntax</sup>**\
> _TypeCastExpression_ :\
> &nbsp;&nbsp; [_Expression_] `as` [_TypeNoBounds_]

A type cast expression is denoted with the binary operator `as`.

Executing an `as` expression casts the value on the left-hand side to the type on the right-hand side.

An example of an `as` expression:

```rust
# fn sum(values: &[f64]) -> f64 { 0.0 }
# fn len(values: &[f64]) -> i32 { 0 }
fn average(values: &[f64]) -> f64 {
    let sum: f64 = sum(values);
    let size: f64 = len(values) as f64;
    sum / size
}
```

`as` can be used to explicitly perform [coercions](../type-coercions.md), as well as the following additional casts.
Any cast that does not fit either a coercion rule or an entry in the table is a compiler error.
Here `*T` means either `*const T` or `*mut T`. `m` stands for optional `mut` in
reference types and `mut` or `const` in pointer types.

| Type of `e`           | `U`                   | Cast performed by `e as U`       |
|-----------------------|-----------------------|----------------------------------|
| Integer or Float type | Integer or Float type | Numeric cast                     |
| Enumeration           | Integer type          | Enum cast                        |
| `bool` or `char`      | Integer type          | Primitive to integer cast        |
| `u8`                  | `char`                | `u8` to `char` cast              |
| `*T`                  | `*V` where `V: Sized` \* | Pointer to pointer cast       |
| `*T` where `T: Sized` | Integer type          | Pointer to address cast          |
| Integer type          | `*V` where `V: Sized` | Address to pointer cast          |
| `&m₁ T`               | `*m₂ T` \*\*          | Reference to pointer cast        |
| `&m₁ [T; n]`          | `*m₂ T` \*\*          | Array to pointer cast            |
| [Function item]       | [Function pointer]    | Function item to function pointer cast |
| [Function item]       | `*V` where `V: Sized` | Function item to pointer cast    |
| [Function item]       | Integer               | Function item to address cast    |
| [Function pointer]    | `*V` where `V: Sized` | Function pointer to pointer cast |
| [Function pointer]    | Integer               | Function pointer to address cast |
| Closure \*\*\*        | Function pointer      | Closure to function pointer cast |

\* or `T` and `V` are compatible unsized types, e.g., both slices, both the same trait object.

\*\* only when `m₁` is `mut` or `m₂` is `const`. Casting `mut` reference to
`const` pointer is allowed.

\*\*\* only for closures that do not capture (close over) any local variables

### Semantics

#### Numeric cast

* Casting between two integers of the same size (e.g. i32 -> u32) is a no-op
  (Rust uses 2's complement for negative values of fixed integers)
* Casting from a larger integer to a smaller integer (e.g. u32 -> u8) will
  truncate
* Casting from a smaller integer to a larger integer (e.g. u8 -> u32) will
    * zero-extend if the source is unsigned
    * sign-extend if the source is signed
* Casting from a float to an integer will round the float towards zero
    * `NaN` will return `0`
    * Values larger than the maximum integer value, including `INFINITY`, will saturate to the maximum value of the integer type.
    * Values smaller than the minimum integer value, including `NEG_INFINITY`, will saturate to the minimum value of the integer type.
* Casting from an integer to float will produce the closest possible float \*
    * if necessary, rounding is according to `roundTiesToEven` mode \*\*\*
    * on overflow, infinity (of the same sign as the input) is produced
    * note: with the current set of numeric types, overflow can only happen
      on `u128 as f32` for values greater or equal to `f32::MAX + (0.5 ULP)`
* Casting from an f32 to an f64 is perfect and lossless
* Casting from an f64 to an f32 will produce the closest possible f32 \*\*
    * if necessary, rounding is according to `roundTiesToEven` mode \*\*\*
    * on overflow, infinity (of the same sign as the input) is produced

\* if integer-to-float casts with this rounding mode and overflow behavior are
not supported natively by the hardware, these casts will likely be slower than
expected.

\*\* if f64-to-f32 casts with this rounding mode and overflow behavior are not
supported natively by the hardware, these casts will likely be slower than
expected.

\*\*\* as defined in IEEE 754-2008 &sect;4.3.1: pick the nearest floating point
number, preferring the one with an even least significant digit if exactly
halfway between two floating point numbers.

#### Enum cast

Casts an enum to its discriminant, then uses a numeric cast if needed.
Casting is limited to the following kinds of enumerations:

* [Unit-only enums]
* [Field-less enums] without [explicit discriminants], or where only unit-variants have explicit discriminants

#### Primitive to integer cast

* `false` casts to `0`, `true` casts to `1`
* `char` casts to the value of the code point, then uses a numeric cast if needed.

#### `u8` to `char` cast

Casts to the `char` with the corresponding code point.

#### Pointer to address cast

Casting from a raw pointer to an integer produces the machine address of the referenced memory.
If the integer type is smaller than the pointer type, the address may be truncated; using `usize` avoids this.

#### Address to pointer cast

Casting from an integer to a raw pointer interprets the integer as a memory address and produces a pointer referencing that memory.

<div class="warning">

Warning:
This interacts with the Rust memory model, which is still under development.
A pointer obtained from this cast may suffer additional restrictions even if it is bitwise equal to a valid pointer.
Dereferencing such a pointer may be [undefined behavior] if aliasing rules are not followed.

</div>

A trivial example of sound address arithmetic:

```rust
let mut values: [i32; 2] = [1, 2];
let p1: *mut i32 = values.as_mut_ptr();
let first_address = p1 as usize;
let second_address = first_address + 4; // 4 == size_of::<i32>()
let p2 = second_address as *mut i32;
unsafe {
    *p2 += 1;
}
assert_eq!(values[1], 3);
```

## Assignment expressions

> **<sup>Syntax</sup>**\
> _AssignmentExpression_ :\
> &nbsp;&nbsp; [_Expression_] `=` [_Expression_]

An *assignment expression* moves a value into a specified place.

An assignment expression consists of a [mutable] [assignee expression], the *assignee operand*, followed by an equals sign (`=`) and a [value expression], the *assigned value operand*.
In its most basic form, an assignee expression is a [place expression], and we discuss this case first.
The more general case of destructuring assignment is discussed below, but this case always decomposes into sequential assignments to place expressions, which may be considered the more fundamental case.

### Basic assignments

Evaluating assignment expressions begins by evaluating its operands.
The assigned value operand is evaluated first, followed by the assignee expression.
For destructuring assignment, subexpressions of the assignee expression are evaluated left-to-right.

> **Note**: This is different than other expressions in that the right operand is evaluated before the left one.

It then has the effect of first [dropping] the value at the assigned place, unless the place is an uninitialized local variable or an uninitialized field of a local variable.
Next it either [copies or moves] the assigned value to the assigned place.

An assignment expression always produces [the unit value][unit].

Example:

```rust
let mut x = 0;
let y = 0;
x = y;
```

### Destructuring assignments

Destructuring assignment is a counterpart to destructuring pattern matches for variable declaration, permitting assignment to complex values, such as tuples or structs.
For instance, we may swap two mutable variables:

```rust
let (mut a, mut b) = (0, 1);
// Swap `a` and `b` using destructuring assignment.
(b, a) = (a, b);
```

In contrast to destructuring declarations using `let`, patterns may not appear on the left-hand side of an assignment due to syntactic ambiguities.
Instead, a group of expressions that correspond to patterns are designated to be [assignee expressions][assignee expression], and permitted on the left-hand side of an assignment.
Assignee expressions are then desugared to pattern matches followed by sequential assignment.
The desugared patterns must be irrefutable: in particular, this means that only slice patterns whose length is known at compile-time, and the trivial slice `[..]`, are permitted for destructuring assignment.

The desugaring method is straightforward, and is illustrated best by example.

```rust
# struct Struct { x: u32, y: u32 }
# let (mut a, mut b) = (0, 0);
(a, b) = (3, 4);

[a, b] = [3, 4];

Struct { x: a, y: b } = Struct { x: 3, y: 4};

// desugars to:

{
    let (_a, _b) = (3, 4);
    a = _a;
    b = _b;
}

{
    let [_a, _b] = [3, 4];
    a = _a;
    b = _b;
}

{
    let Struct { x: _a, y: _b } = Struct { x: 3, y: 4};
    a = _a;
    b = _b;
}
```

Identifiers are not forbidden from being used multiple times in a single assignee expression.

[Underscore expressions][_UnderscoreExpression_] and empty [range expressions][_RangeExpression_] may be used to ignore certain values, without binding them.

Note that default binding modes do not apply for the desugared expression.

## Compound assignment expressions

> **<sup>Syntax</sup>**\
> _CompoundAssignmentExpression_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_Expression_] `+=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `-=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `*=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `/=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `%=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `&=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `|=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `^=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `<<=` [_Expression_]\
> &nbsp;&nbsp; | [_Expression_] `>>=` [_Expression_]

*Compound assignment expressions* combine arithmetic and logical binary operators with assignment expressions.

For example:

```rust
let mut x = 5;
x += 1;
assert!(x == 6);
```

The syntax of compound assignment is a [mutable] [place expression], the *assigned operand*, then one of the operators followed by an `=` as a single token (no whitespace), and then a [value expression], the *modifying operand*.

Unlike other place operands, the assigned place operand must be a place expression.
Attempting to use a value expression is a compiler error rather than promoting it to a temporary.

Evaluation of compound assignment expressions depends on the types of the operators.

If both types are primitives, then the modifying operand will be evaluated first followed by the assigned operand.
It will then set the value of the assigned operand's place to the value of performing the operation of the operator with the values of the assigned operand and modifying operand.

> **Note**: This is different than other expressions in that the right operand is evaluated before the left one.

Otherwise, this expression is syntactic sugar for calling the function of the overloading compound assignment trait of the operator (see the table earlier in this chapter).
A mutable borrow of the assigned operand is automatically taken.

For example, the following expression statements in `example` are equivalent:

```rust
# struct Addable;
# use std::ops::AddAssign;

impl AddAssign<Addable> for Addable {
    /* */
# fn add_assign(&mut self, other: Addable) {}
}

fn example() {
# let (mut a1, a2) = (Addable, Addable);
  a1 += a2;

# let (mut a1, a2) = (Addable, Addable);
  AddAssign::add_assign(&mut a1, a2);
}
```

Like assignment expressions, compound assignment expressions always produce [the unit value][unit].

<div class="warning">

Warning: The evaluation order of operands swaps depending on the types of the operands:
with primitive types the right-hand side will get evaluated first, while with non-primitive types the left-hand side will get evaluated first.
Try not to write code that depends on the evaluation order of operands in compound assignment expressions.
See [this test] for an example of using this dependency.

</div>

[copies or moves]: ../expressions.md#moved-and-copied-types
[dropping]: ../destructors.md
[explicit discriminants]: ../items/enumerations.md#explicit-discriminants
[field-less enums]: ../items/enumerations.md#field-less-enum
[grouped expression]: grouped-expr.md
[literal expression]: literal-expr.md#integer-literal-expressions
[logical and]: ../types/boolean.md#logical-and
[logical not]: ../types/boolean.md#logical-not
[logical or]: ../types/boolean.md#logical-or
[logical xor]: ../types/boolean.md#logical-xor
[mutable]: ../expressions.md#mutability
[place expression]: ../expressions.md#place-expressions-and-value-expressions
[assignee expression]: ../expressions.md#place-expressions-and-value-expressions
[undefined behavior]: ../behavior-considered-undefined.md
[unit]: ../types/tuple.md
[Unit-only enums]: ../items/enumerations.md#unit-only-enum
[value expression]: ../expressions.md#place-expressions-and-value-expressions
[temporary value]: ../expressions.md#temporaries
[this test]: https://github.com/rust-lang/rust/blob/1.58.0/src/test/ui/expr/compound-assignment/eval-order.rs
[float-float]: https://github.com/rust-lang/rust/issues/15536
[Function pointer]: ../types/function-pointer.md
[Function item]: ../types/function-item.md
[undefined behavior]: ../behavior-considered-undefined.md
[addr_of]: ../../std/ptr/macro.addr_of.html
[addr_of_mut]: ../../std/ptr/macro.addr_of_mut.html

[_BorrowExpression_]: #borrow-operators
[_DereferenceExpression_]: #the-dereference-operator
[_ErrorPropagationExpression_]: #the-question-mark-operator
[_NegationExpression_]: #negation-operators
[_ArithmeticOrLogicalExpression_]: #arithmetic-and-logical-binary-operators
[_ComparisonExpression_]: #comparison-operators
[_LazyBooleanExpression_]: #lazy-boolean-operators
[_TypeCastExpression_]: #type-cast-expressions
[_AssignmentExpression_]: #assignment-expressions
[_CompoundAssignmentExpression_]: #compound-assignment-expressions

[_Expression_]: ../expressions.md
[_TypeNoBounds_]: ../types.md#type-expressions
[_RangeExpression_]: ./range-expr.md
[_UnderscoreExpression_]: ./underscore-expr.md
