# Array and array index expressions

## Array expressions

> **<sup>Syntax</sup>**\
> _ArrayExpression_ :\
> &nbsp;&nbsp; `[` _ArrayElements_<sup>?</sup> `]`
>
> _ArrayElements_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_Expression_] ( `,` [_Expression_] )<sup>\*</sup> `,`<sup>?</sup>\
> &nbsp;&nbsp; | [_Expression_] `;` [_Expression_]

*Array expressions* construct [arrays][array].
Array expressions come in two forms.

The first form lists out every value in the array.
The syntax for this form is a comma-separated list of expressions of uniform type enclosed in square brackets.
This produces an array containing each of these values in the order they are written.

The syntax for the second form is two expressions separated by a semicolon (`;`) enclosed in square brackets.
The expression before the `;` is called the *repeat operand*.
The expression after the `;` is called the *length operand*.
It must have type `usize` and be a [constant expression], such as a [literal] or a [constant item].
An array expression of this form creates an array with the length of the value of the length operand with each element being a copy of the repeat operand.
That is, `[a; b]` creates an array containing `b` copies of the value of `a`.
If the length operand has a value greater than 1 then this requires that the type of the repeat operand is [`Copy`] or that it must be a [path] to a constant item.

When the repeat operand is a constant item, it is evaluated the length operand's value times.
If that value is `0`, then the constant item is not evaluated at all.
For expressions that are not a constant item, it is evaluated exactly once, and then the result is copied the length operand's value times.

```rust
[1, 2, 3, 4];
["a", "b", "c", "d"];
[0; 128];              // array with 128 zeros
[0u8, 0u8, 0u8, 0u8,];
[[1, 0, 0], [0, 1, 0], [0, 0, 1]]; // 2D array
const EMPTY: Vec<i32> = Vec::new();
[EMPTY; 2];
```

## Array and slice indexing expressions

> **<sup>Syntax</sup>**\
> _IndexExpression_ :\
> &nbsp;&nbsp; [_Expression_] `[` [_Expression_] `]`

[Array] and [slice]-typed values can be indexed by writing a square-bracket-enclosed expression of type `usize` (the index) after them.
When the array is mutable, the resulting [memory location] can be assigned to.

For other types an index expression `a[b]` is equivalent to `*std::ops::Index::index(&a, b)`, or `*std::ops::IndexMut::index_mut(&mut a, b)` in a mutable place expression context.
Just as with methods, Rust will also insert dereference operations on `a` repeatedly to find an implementation.

Indices are zero-based for arrays and slices.
Array access is a [constant expression], so bounds can be checked at compile-time with a constant index value.
Otherwise a check will be performed at run-time that will put the thread in a _panicked state_ if it fails.

```rust,should_panic
// lint is deny by default.
#![warn(unconditional_panic)]

([1, 2, 3, 4])[2];        // Evaluates to 3

let b = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
b[1][2];                  // multidimensional array indexing

let x = (["a", "b"])[10]; // warning: index out of bounds

let n = 10;
let y = (["a", "b"])[n];  // panics

let arr = ["a", "b"];
arr[10];                  // warning: index out of bounds
```

The array index expression can be implemented for types other than arrays and slices by implementing the [Index] and [IndexMut] traits.

[`Copy`]: ../special-types-and-traits.md#copy
[IndexMut]: ../../std/ops/trait.IndexMut.html
[Index]: ../../std/ops/trait.Index.html
[_Expression_]: ../expressions.md
[array]: ../types/array.md
[constant expression]: ../const_eval.md#constant-expressions
[constant item]: ../items/constant-items.md
[literal]: ../tokens.md#literals
[memory location]: ../expressions.md#place-expressions-and-value-expressions
[path]: path-expr.md
[slice]: ../types/slice.md
