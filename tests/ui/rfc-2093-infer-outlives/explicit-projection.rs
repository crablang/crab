#![feature(crablangc_attrs)]

trait Trait<'x, T> where T: 'x {
    type Type;
}

#[crablangc_outlives]
struct Foo<'a, A, B> where A: Trait<'a, B> //~ ERROR crablangc_outlives
{
    foo: <A as Trait<'a, B>>::Type
}

fn main() {}
