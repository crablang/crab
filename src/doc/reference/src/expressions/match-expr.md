# `match` expressions

> **<sup>Syntax</sup>**\
> _MatchExpression_ :\
> &nbsp;&nbsp; `match` _Scrutinee_ `{`\
> &nbsp;&nbsp; &nbsp;&nbsp; [_InnerAttribute_]<sup>\*</sup>\
> &nbsp;&nbsp; &nbsp;&nbsp; _MatchArms_<sup>?</sup>\
> &nbsp;&nbsp; `}`
>
>_Scrutinee_ :\
> &nbsp;&nbsp; [_Expression_]<sub>_except struct expression_</sub>
>
> _MatchArms_ :\
> &nbsp;&nbsp; ( _MatchArm_ `=>`
>                             ( [_ExpressionWithoutBlock_][_Expression_] `,`
>                             | [_ExpressionWithBlock_][_Expression_] `,`<sup>?</sup> )
>                           )<sup>\*</sup>\
> &nbsp;&nbsp; _MatchArm_ `=>` [_Expression_] `,`<sup>?</sup>
>
> _MatchArm_ :\
> &nbsp;&nbsp; [_OuterAttribute_]<sup>\*</sup> [_Pattern_] _MatchArmGuard_<sup>?</sup>
>
> _MatchArmGuard_ :\
> &nbsp;&nbsp; `if` [_Expression_]

A *`match` expression* branches on a pattern.
The exact form of matching that occurs depends on the [pattern].
A `match` expression has a *[scrutinee] expression*, which is the value to compare to the patterns.
The scrutinee expression and the patterns must have the same type.

A `match` behaves differently depending on whether or not the scrutinee expression is a [place expression or value expression][place expression].
If the scrutinee expression is a [value expression], it is first evaluated into a temporary location, and the resulting value is sequentially compared to the patterns in the arms until a match is found.
The first arm with a matching pattern is chosen as the branch target of the `match`, any variables bound by the pattern are assigned to local variables in the arm's block, and control enters the block.

When the scrutinee expression is a [place expression], the match does not allocate a temporary location;
however, a by-value binding may copy or move from the memory location.
When possible, it is preferable to match on place expressions, as the lifetime of these matches inherits the lifetime of the place expression rather than being restricted to the inside of the match.

An example of a `match` expression:

```rust
let x = 1;

match x {
    1 => println!("one"),
    2 => println!("two"),
    3 => println!("three"),
    4 => println!("four"),
    5 => println!("five"),
    _ => println!("something else"),
}
```

Variables bound within the pattern are scoped to the match guard and the arm's expression.
The [binding mode] (move, copy, or reference) depends on the pattern.

Multiple match patterns may be joined with the `|` operator.
Each pattern will be tested in left-to-right sequence until a successful match is found.

```rust
let x = 9;
let message = match x {
    0 | 1  => "not many",
    2 ..= 9 => "a few",
    _      => "lots"
};

assert_eq!(message, "a few");

// Demonstration of pattern match order.
struct S(i32, i32);

match S(1, 2) {
    S(z @ 1, _) | S(_, z @ 2) => assert_eq!(z, 1),
    _ => panic!(),
}
```

> Note: The `2..=9` is a [Range Pattern], not a [Range Expression].
> Thus, only those types of ranges supported by range patterns can be used in match arms.

Every binding in each `|` separated pattern must appear in all of the patterns in the arm.
Every binding of the same name must have the same type, and have the same binding mode.

## Match guards

Match arms can accept _match guards_ to further refine the criteria for matching a case.
Pattern guards appear after the pattern and consist of a `bool`-typed expression following the `if` keyword.

When the pattern matches successfully, the pattern guard expression is executed.
If the expression evaluates to true, the pattern is successfully matched against.
Otherwise, the next pattern, including other matches with the `|` operator in the same arm, is tested.

```rust
# let maybe_digit = Some(0);
# fn process_digit(i: i32) { }
# fn process_other(i: i32) { }
let message = match maybe_digit {
    Some(x) if x < 10 => process_digit(x),
    Some(x) => process_other(x),
    None => panic!(),
};
```

> Note: Multiple matches using the `|` operator can cause the pattern guard and the side effects it has to execute multiple times.
> For example:
>
> ```rust
> # use std::cell::Cell;
> let i : Cell<i32> = Cell::new(0);
> match 1 {
>     1 | _ if { i.set(i.get() + 1); false } => {}
>     _ => {}
> }
> assert_eq!(i.get(), 2);
> ```

A pattern guard may refer to the variables bound within the pattern they follow.
Before evaluating the guard, a shared reference is taken to the part of the scrutinee the variable matches on.
While evaluating the guard, this shared reference is then used when accessing the variable.
Only when the guard evaluates to true is the value moved, or copied, from the scrutinee into the variable.
This allows shared borrows to be used inside guards without moving out of the scrutinee in case guard fails to match.
Moreover, by holding a shared reference while evaluating the guard, mutation inside guards is also prevented.

## Attributes on match arms

Outer attributes are allowed on match arms.
The only attributes that have meaning on match arms are [`cfg`] and the [lint check attributes].

[Inner attributes] are allowed directly after the opening brace of the match expression in the same expression contexts as [attributes on block expressions].

[_Expression_]: ../expressions.md
[place expression]: ../expressions.md#place-expressions-and-value-expressions
[value expression]: ../expressions.md#place-expressions-and-value-expressions
[_InnerAttribute_]: ../attributes.md
[_OuterAttribute_]: ../attributes.md
[`cfg`]: ../conditional-compilation.md
[lint check attributes]: ../attributes/diagnostics.md#lint-check-attributes
[Range Expression]: range-expr.md

[_Pattern_]: ../patterns.md
[pattern]: ../patterns.md
[Inner attributes]: ../attributes.md
[Range Pattern]: ../patterns.md#range-patterns
[attributes on block expressions]: block-expr.md#attributes-on-block-expressions
[binding mode]: ../patterns.md#binding-modes
[scrutinee]: ../glossary.md#scrutinee
