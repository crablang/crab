# `if` and `if let` expressions

## `if` expressions

> **<sup>Syntax</sup>**\
> _IfExpression_ :\
> &nbsp;&nbsp; `if` [_Expression_]<sub>_except struct expression_</sub> [_BlockExpression_]\
> &nbsp;&nbsp; (`else` (
>   [_BlockExpression_]
> | _IfExpression_
> | _IfLetExpression_ ) )<sup>\?</sup>

An `if` expression is a conditional branch in program control.
The syntax of an `if` expression is a condition operand, followed by a consequent block, any number of `else if` conditions and blocks, and an optional trailing `else` block.
The condition operands must have the [boolean type].
If a condition operand evaluates to `true`, the consequent block is executed and any subsequent `else if` or `else` block is skipped.
If a condition operand evaluates to `false`, the consequent block is skipped and any subsequent `else if` condition is evaluated.
If all `if` and `else if` conditions evaluate to `false` then any `else` block is executed.
An if expression evaluates to the same value as the executed block, or `()` if no block is evaluated.
An `if` expression must have the same type in all situations.

```rust
# let x = 3;
if x == 4 {
    println!("x is four");
} else if x == 3 {
    println!("x is three");
} else {
    println!("x is something else");
}

let y = if 12 * 15 > 150 {
    "Bigger"
} else {
    "Smaller"
};
assert_eq!(y, "Bigger");
```

## `if let` expressions

> **<sup>Syntax</sup>**\
> _IfLetExpression_ :\
> &nbsp;&nbsp; `if` `let` [_Pattern_] `=` [_Scrutinee_]<sub>_except lazy boolean operator expression_</sub>
>              [_BlockExpression_]\
> &nbsp;&nbsp; (`else` (
>   [_BlockExpression_]
> | _IfExpression_
> | _IfLetExpression_ ) )<sup>\?</sup>

An `if let` expression is semantically similar to an `if` expression but in place of a condition operand it expects the keyword `let` followed by a pattern, an `=` and a [scrutinee] operand.
If the value of the scrutinee matches the pattern, the corresponding block will execute.
Otherwise, flow proceeds to the following `else` block if it exists.
Like `if` expressions, `if let` expressions have a value determined by the block that is evaluated.

```rust
let dish = ("Ham", "Eggs");

// this body will be skipped because the pattern is refuted
if let ("Bacon", b) = dish {
    println!("Bacon is served with {}", b);
} else {
    // This block is evaluated instead.
    println!("No bacon will be served");
}

// this body will execute
if let ("Ham", b) = dish {
    println!("Ham is served with {}", b);
}

if let _ = 5 {
    println!("Irrefutable patterns are always true");
}
```

`if` and `if let` expressions can be intermixed:

```rust
let x = Some(3);
let a = if let Some(1) = x {
    1
} else if x == Some(2) {
    2
} else if let Some(y) = x {
    y
} else {
    -1
};
assert_eq!(a, 3);
```

An `if let` expression is equivalent to a [`match` expression] as follows:

<!-- ignore: expansion example -->
```rust,ignore
if let PATS = EXPR {
    /* body */
} else {
    /*else */
}
```

is equivalent to

<!-- ignore: expansion example -->
```rust,ignore
match EXPR {
    PATS => { /* body */ },
    _ => { /* else */ },    // () if there is no else
}
```

Multiple patterns may be specified with the `|` operator. This has the same semantics as with `|` in `match` expressions:

```rust
enum E {
    X(u8),
    Y(u8),
    Z(u8),
}
let v = E::Y(12);
if let E::X(n) | E::Y(n) = v {
    assert_eq!(n, 12);
}
```

The expression cannot be a [lazy boolean operator expression][_LazyBooleanOperatorExpression_].
Use of a lazy boolean operator is ambiguous with a planned feature change of the language (the implementation of if-let chains - see [eRFC 2947][_eRFCIfLetChain_]).
When lazy boolean operator expression is desired, this can be achieved by using parenthesis as below:

<!-- ignore: pseudo code -->
```rust,ignore
// Before...
if let PAT = EXPR && EXPR { .. }

// After...
if let PAT = ( EXPR && EXPR ) { .. }

// Before...
if let PAT = EXPR || EXPR { .. }

// After...
if let PAT = ( EXPR || EXPR ) { .. }
```

[_BlockExpression_]: block-expr.md
[_Expression_]: ../expressions.md
[_LazyBooleanOperatorExpression_]: operator-expr.md#lazy-boolean-operators
[_Pattern_]: ../patterns.md
[_Scrutinee_]: match-expr.md
[_eRFCIfLetChain_]: https://github.com/rust-lang/rfcs/blob/master/text/2497-if-let-chains.md#rollout-plan-and-transitioning-to-rust-2018
[`match` expression]: match-expr.md
[boolean type]: ../types/boolean.md
[scrutinee]: ../glossary.md#scrutinee
