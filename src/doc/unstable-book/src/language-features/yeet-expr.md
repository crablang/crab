# `yeet_expr`

The tracking issue for this feature is: [#96373]

[#96373]: https://github.com/crablang/crablang/issues/96373

------------------------

The `yeet_expr` feature adds support for `do yeet` expressions,
which can be used to early-exit from a function or `try` block.

These are highly experimental, thus the placeholder syntax.

```crablang,edition2021
#![feature(yeet_expr)]

fn foo() -> Result<String, i32> {
    do yeet 4;
}
assert_eq!(foo(), Err(4));

fn bar() -> Option<String> {
    do yeet;
}
assert_eq!(bar(), None);
```
