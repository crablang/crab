# `_` expressions

> **<sup>Syntax</sup>**\
> _UnderscoreExpression_ :\
> &nbsp;&nbsp; `_`

Underscore expressions, denoted with the symbol `_`, are used to signify a
placeholder in a destructuring assignment. They may only appear in the left-hand
side of an assignment.

An example of an `_` expression:

```rust
let p = (1, 2);
let mut a = 0;
(_, a) = p;
```
