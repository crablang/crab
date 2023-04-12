// https://github.com/crablang/crablang/issues/107147

#![warn(clippy::needless_pass_by_value)]

struct Foo<'a>(&'a [(); 100]);

fn test(x: Foo<'_>) {}

fn main() {}
