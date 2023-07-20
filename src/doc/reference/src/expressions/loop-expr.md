# Loops and other breakable expressions

> **<sup>Syntax</sup>**\
> _LoopExpression_ :\
> &nbsp;&nbsp; [_LoopLabel_]<sup>?</sup> (\
> &nbsp;&nbsp; &nbsp;&nbsp; &nbsp;&nbsp; [_InfiniteLoopExpression_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_PredicateLoopExpression_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_PredicatePatternLoopExpression_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_IteratorLoopExpression_]\
> &nbsp;&nbsp; &nbsp;&nbsp; | [_LabelBlockExpression_]\
> &nbsp;&nbsp; )

[_LoopLabel_]: #loop-labels
[_InfiniteLoopExpression_]: #infinite-loops
[_PredicateLoopExpression_]: #predicate-loops
[_PredicatePatternLoopExpression_]: #predicate-pattern-loops
[_IteratorLoopExpression_]: #iterator-loops
[_LabelBlockExpression_]: #labelled-block-expressions

Rust supports five loop expressions:

*   A [`loop` expression](#infinite-loops) denotes an infinite loop.
*   A [`while` expression](#predicate-loops) loops until a predicate is false.
*   A [`while let` expression](#predicate-pattern-loops) tests a pattern.
*   A [`for` expression](#iterator-loops) extracts values from an iterator, looping until the iterator is empty.
*   A [labelled block expression](#labelled-block-expressions) runs a loop exactly once, but allows exiting the loop early with `break`.

All five types of loop support [`break` expressions](#break-expressions), and [labels](#loop-labels).
All except labelled block expressions support [`continue` expressions](#continue-expressions).
Only `loop` and labelled block expressions support [evaluation to non-trivial values](#break-and-loop-values).

## Infinite loops

> **<sup>Syntax</sup>**\
> _InfiniteLoopExpression_ :\
> &nbsp;&nbsp; `loop` [_BlockExpression_]

A `loop` expression repeats execution of its body continuously:
`loop { println!("I live."); }`.

A `loop` expression without an associated `break` expression is diverging and has type [`!`](../types/never.md).
A `loop` expression containing associated [`break` expression(s)](#break-expressions) may terminate, and must have type compatible with the value of the `break` expression(s).

## Predicate loops

> **<sup>Syntax</sup>**\
> _PredicateLoopExpression_ :\
> &nbsp;&nbsp; `while` [_Expression_]<sub>_except struct expression_</sub> [_BlockExpression_]

A `while` loop begins by evaluating the [boolean] loop conditional operand.
If the loop conditional operand evaluates to `true`, the loop body block executes, then control returns to the loop conditional operand.
If the loop conditional expression evaluates to `false`, the `while` expression completes.

An example:

```rust
let mut i = 0;

while i < 10 {
    println!("hello");
    i = i + 1;
}
```

## Predicate pattern loops

> **<sup>Syntax</sup>**\
> [_PredicatePatternLoopExpression_] :\
> &nbsp;&nbsp; `while` `let` [_Pattern_] `=` [_Scrutinee_]<sub>_except lazy boolean operator expression_</sub>
>              [_BlockExpression_]


A `while let` loop is semantically similar to a `while` loop but in place of a condition expression it expects the keyword `let` followed by a pattern, an `=`, a [scrutinee] expression and a block expression.
If the value of the scrutinee matches the pattern, the loop body block executes then control returns to the pattern matching statement.
Otherwise, the while expression completes.

```rust
let mut x = vec![1, 2, 3];

while let Some(y) = x.pop() {
    println!("y = {}", y);
}

while let _ = 5 {
    println!("Irrefutable patterns are always true");
    break;
}
```

A `while let` loop is equivalent to a `loop` expression containing a [`match` expression] as follows.

<!-- ignore: expansion example -->
```rust,ignore
'label: while let PATS = EXPR {
    /* loop body */
}
```

is equivalent to

<!-- ignore: expansion example -->
```rust,ignore
'label: loop {
    match EXPR {
        PATS => { /* loop body */ },
        _ => break,
    }
}
```

Multiple patterns may be specified with the `|` operator.
This has the same semantics as with `|` in `match` expressions:

```rust
let mut vals = vec![2, 3, 1, 2, 2];
while let Some(v @ 1) | Some(v @ 2) = vals.pop() {
    // Prints 2, 2, then 1
    println!("{}", v);
}
```

As is the case in [`if let` expressions], the scrutinee cannot be a [lazy boolean operator expression][_LazyBooleanOperatorExpression_].

## Iterator loops

> **<sup>Syntax</sup>**\
> _IteratorLoopExpression_ :\
> &nbsp;&nbsp; `for` [_Pattern_] `in` [_Expression_]<sub>_except struct expression_</sub>
>              [_BlockExpression_]

A `for` expression is a syntactic construct for looping over elements provided by an implementation of `std::iter::IntoIterator`.
If the iterator yields a value, that value is matched against the irrefutable pattern, the body of the loop is executed, and then control returns to the head of the `for` loop.
If the iterator is empty, the `for` expression completes.

An example of a `for` loop over the contents of an array:

```rust
let v = &["apples", "cake", "coffee"];

for text in v {
    println!("I like {}.", text);
}
```

An example of a for loop over a series of integers:

```rust
let mut sum = 0;
for n in 1..11 {
    sum += n;
}
assert_eq!(sum, 55);
```

A `for` loop is equivalent to a `loop` expression containing a [`match` expression] as follows:

<!-- ignore: expansion example -->
```rust,ignore
'label: for PATTERN in iter_expr {
    /* loop body */
}
```

is equivalent to

<!-- ignore: expansion example -->
```rust,ignore
{
    let result = match IntoIterator::into_iter(iter_expr) {
        mut iter => 'label: loop {
            let mut next;
            match Iterator::next(&mut iter) {
                Option::Some(val) => next = val,
                Option::None => break,
            };
            let PATTERN = next;
            let () = { /* loop body */ };
        },
    };
    result
}
```

`IntoIterator`, `Iterator`, and `Option` are always the standard library items here, not whatever those names resolve to in the current scope.
The variable names `next`, `iter`, and `val` are for exposition only, they do not actually have names the user can type.

> **Note**: that the outer `match` is used to ensure that any [temporary values] in `iter_expr` don't get dropped before the loop is finished.
> `next` is declared before being assigned because it results in types being inferred correctly more often.

## Loop labels

> **<sup>Syntax</sup>**\
> _LoopLabel_ :\
> &nbsp;&nbsp; [LIFETIME_OR_LABEL] `:`

A loop expression may optionally have a _label_. The label is written as a lifetime preceding the loop expression, as in `'foo: loop { break 'foo; }`, `'bar: while false {}`, `'humbug: for _ in 0..0 {}`.
If a label is present, then labeled `break` and `continue` expressions nested within this loop may exit out of this loop or return control to its head.
See [break expressions](#break-expressions) and [continue expressions](#continue-expressions).

Labels follow the hygiene and shadowing rules of local variables. For example, this code will print "outer loop":

```rust
'a: loop {
    'a: loop {
        break 'a;
    }
    print!("outer loop");
    break 'a;
}
```

## `break` expressions

> **<sup>Syntax</sup>**\
> _BreakExpression_ :\
> &nbsp;&nbsp; `break` [LIFETIME_OR_LABEL]<sup>?</sup> [_Expression_]<sup>?</sup>

When `break` is encountered, execution of the associated loop body is immediately terminated, for example:

```rust
let mut last = 0;
for x in 1..100 {
    if x > 12 {
        break;
    }
    last = x;
}
assert_eq!(last, 12);
```

A `break` expression is normally associated with the innermost `loop`, `for` or `while` loop enclosing the `break` expression,
but a [label](#loop-labels) can be used to specify which enclosing loop is affected.
Example:

```rust
'outer: loop {
    while true {
        break 'outer;
    }
}
```

A `break` expression is only permitted in the body of a loop, and has one of the forms `break`, `break 'label` or ([see below](#break-and-loop-values)) `break EXPR` or `break 'label EXPR`.

## Labelled block expressions

> **<sup>Syntax</sup>**\
> _LabelBlockExpression_ :\
> &nbsp;&nbsp; [_BlockExpression_]

Labelled block expressions are exactly like block expressions, except that they allow using `break` expressions within the block.
Unlike loops, `break` expressions within a labelled block expression *must* have a label (i.e. the label is not optional).
Similarly, labelled block expressions *must* begin with a label.

```rust
# fn do_thing() {}
# fn condition_not_met() -> bool { true }
# fn do_next_thing() {}
# fn do_last_thing() {}
let result = 'block: {
    do_thing();
    if condition_not_met() {
        break 'block 1;
    }
    do_next_thing();
    if condition_not_met() {
        break 'block 2;
    }
    do_last_thing();
    3
};
```

## `continue` expressions

> **<sup>Syntax</sup>**\
> _ContinueExpression_ :\
> &nbsp;&nbsp; `continue` [LIFETIME_OR_LABEL]<sup>?</sup>

When `continue` is encountered, the current iteration of the associated loop body is immediately terminated, returning control to the loop *head*.
In the case of a `while` loop, the head is the conditional expression controlling the loop.
In the case of a `for` loop, the head is the call-expression controlling the loop.

Like `break`, `continue` is normally associated with the innermost enclosing loop, but `continue 'label` may be used to specify the loop affected.
A `continue` expression is only permitted in the body of a loop.

## `break` and loop values

When associated with a `loop`, a break expression may be used to return a value from that loop, via one of the forms `break EXPR` or `break 'label EXPR`, where `EXPR` is an expression whose result is returned from the `loop`.
For example:

```rust
let (mut a, mut b) = (1, 1);
let result = loop {
    if b > 10 {
        break b;
    }
    let c = a + b;
    a = b;
    b = c;
};
// first number in Fibonacci sequence over 10:
assert_eq!(result, 13);
```

In the case a `loop` has an associated `break`, it is not considered diverging, and the `loop` must have a type compatible with each `break` expression.
`break` without an expression is considered identical to `break` with expression `()`.

[LIFETIME_OR_LABEL]: ../tokens.md#lifetimes-and-loop-labels
[_BlockExpression_]: block-expr.md
[_Expression_]: ../expressions.md
[_Pattern_]: ../patterns.md
[_Scrutinee_]: match-expr.md
[`match` expression]: match-expr.md
[boolean]: ../types/boolean.md
[scrutinee]: ../glossary.md#scrutinee
[temporary values]: ../expressions.md#temporaries
[_LazyBooleanOperatorExpression_]: operator-expr.md#lazy-boolean-operators
[`if let` expressions]: if-expr.md#if-let-expressions
