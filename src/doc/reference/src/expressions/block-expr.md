# Block expressions

> **<sup>Syntax</sup>**\
> _BlockExpression_ :\
> &nbsp;&nbsp; `{`\
> &nbsp;&nbsp; &nbsp;&nbsp; [_InnerAttribute_]<sup>\*</sup>\
> &nbsp;&nbsp; &nbsp;&nbsp; _Statements_<sup>?</sup>\
> &nbsp;&nbsp; `}`
>
> _Statements_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_Statement_]<sup>\+</sup>\
> &nbsp;&nbsp; | [_Statement_]<sup>\+</sup> [_ExpressionWithoutBlock_]\
> &nbsp;&nbsp; | [_ExpressionWithoutBlock_]

A *block expression*, or *block*, is a control flow expression and anonymous namespace scope for items and variable declarations.
As a control flow expression, a block sequentially executes its component non-item declaration statements and then its final optional expression.
As an anonymous namespace scope, item declarations are only in scope inside the block itself and variables declared by `let` statements are in scope from the next statement until the end of the block.

The syntax for a block is `{`, then any [inner attributes], then any number of [statements], then an optional expression, called the final operand, and finally a `}`.

Statements are usually required to be followed by a semicolon, with two exceptions:

1. Item declaration statements do not need to be followed by a semicolon.
2. Expression statements usually require a following semicolon except if its outer expression is a flow control expression.

Furthermore, extra semicolons between statements are allowed, but these semicolons do not affect semantics.

When evaluating a block expression, each statement, except for item declaration statements, is executed sequentially.
Then the final operand is executed, if given.

The type of a block is the type of the final operand, or `()` if the final operand is omitted.

```rust
# fn fn_call() {}
let _: () = {
    fn_call();
};

let five: i32 = {
    fn_call();
    5
};

assert_eq!(5, five);
```

> Note: As a control flow expression, if a block expression is the outer expression of an expression statement, the expected type is `()` unless it is followed immediately by a semicolon.

Blocks are always [value expressions] and evaluate the last operand in value expression context.

> **Note**: This can be used to force moving a value if really needed.
> For example, the following example fails on the call to `consume_self` because the struct was moved out of `s` in the block expression.
>
> ```rust,compile_fail
> struct Struct;
>
> impl Struct {
>     fn consume_self(self) {}
>     fn borrow_self(&self) {}
> }
>
> fn move_by_block_expression() {
>     let s = Struct;
>
>     // Move the value out of `s` in the block expression.
>     (&{ s }).borrow_self();
>
>     // Fails to execute because `s` is moved out of.
>     s.consume_self();
> }
> ```

## `async` blocks

> **<sup>Syntax</sup>**\
> _AsyncBlockExpression_ :\
> &nbsp;&nbsp; `async` `move`<sup>?</sup> _BlockExpression_

An *async block* is a variant of a block expression which evaluates to a future.
The final expression of the block, if present, determines the result value of the future.

Executing an async block is similar to executing a closure expression:
its immediate effect is to produce and return an anonymous type.
Whereas closures return a type that implements one or more of the [`std::ops::Fn`] traits, however, the type returned for an async block implements the [`std::future::Future`] trait.
The actual data format for this type is unspecified.

> **Note:** The future type that rustc generates is roughly equivalent to an enum with one variant per `await` point, where each variant stores the data needed to resume from its corresponding point.

> **Edition differences**: Async blocks are only available beginning with Rust 2018.

### Capture modes

Async blocks capture variables from their environment using the same [capture modes] as closures.
Like closures, when written `async { .. }` the capture mode for each variable will be inferred from the content of the block.
`async move { .. }` blocks however will move all referenced variables into the resulting future.

### Async context

Because async blocks construct a future, they define an **async context** which can in turn contain [`await` expressions].
Async contexts are established by async blocks as well as the bodies of async functions, whose semantics are defined in terms of async blocks.

### Control-flow operators

Async blocks act like a function boundary, much like closures.
Therefore, the `?` operator and `return` expressions both affect the output of the future, not the enclosing function or other context.
That is, `return <expr>` from within an async block will return the result of `<expr>` as the output of the future.
Similarly, if `<expr>?` propagates an error, that error is propagated as the result of the future.

Finally, the `break` and `continue` keywords cannot be used to branch out from an async block.
Therefore the following is illegal:

```rust,compile_fail
loop {
    async move {
        break; // error[E0267]: `break` inside of an `async` block
    }
}
```

## `unsafe` blocks

> **<sup>Syntax</sup>**\
> _UnsafeBlockExpression_ :\
> &nbsp;&nbsp; `unsafe` _BlockExpression_

_See [`unsafe` block](../unsafe-blocks.md) for more information on when to use `unsafe`_

A block of code can be prefixed with the `unsafe` keyword to permit [unsafe operations].
Examples:

```rust
unsafe {
    let b = [13u8, 17u8];
    let a = &b[0] as *const u8;
    assert_eq!(*a, 13);
    assert_eq!(*a.offset(1), 17);
}

# unsafe fn an_unsafe_fn() -> i32 { 10 }
let a = unsafe { an_unsafe_fn() };
```

## Labelled block expressions

Labelled block expressions are documented in the [Loops and other breakable expressions] section.

## Attributes on block expressions

[Inner attributes] are allowed directly after the opening brace of a block expression in the following situations:

* [Function] and [method] bodies.
* Loop bodies ([`loop`], [`while`], [`while let`], and [`for`]).
* Block expressions used as a [statement].
* Block expressions as elements of [array expressions], [tuple expressions],
  [call expressions], and tuple-style [struct] expressions.
* A block expression as the tail expression of another block expression.
<!-- Keep list in sync with expressions.md -->

The attributes that have meaning on a block expression are [`cfg`] and [the lint check attributes].

For example, this function returns `true` on unix platforms and `false` on other platforms.

```rust
fn is_unix_platform() -> bool {
    #[cfg(unix)] { true }
    #[cfg(not(unix))] { false }
}
```

[_ExpressionWithoutBlock_]: ../expressions.md
[_InnerAttribute_]: ../attributes.md
[_Statement_]: ../statements.md
[`await` expressions]: await-expr.md
[`cfg`]: ../conditional-compilation.md
[`for`]: loop-expr.md#iterator-loops
[`loop`]: loop-expr.md#infinite-loops
[`std::ops::Fn`]: ../../std/ops/trait.Fn.html
[`std::future::Future`]: ../../std/future/trait.Future.html
[`while let`]: loop-expr.md#predicate-pattern-loops
[`while`]: loop-expr.md#predicate-loops
[array expressions]: array-expr.md
[call expressions]: call-expr.md
[capture modes]: ../types/closure.md#capture-modes
[function]: ../items/functions.md
[inner attributes]: ../attributes.md
[method]: ../items/associated-items.md#methods
[mutable reference]: ../types/pointer.md#mutables-references-
[shared references]: ../types/pointer.md#shared-references-
[statement]: ../statements.md
[statements]: ../statements.md
[struct]: struct-expr.md
[the lint check attributes]: ../attributes/diagnostics.md#lint-check-attributes
[tuple expressions]: tuple-expr.md
[unsafe operations]: ../unsafety.md
[value expressions]: ../expressions.md#place-expressions-and-value-expressions
[Loops and other breakable expressions]: loop-expr.md#labelled-block-expressions
