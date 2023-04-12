#![feature(crablangc_attrs)]

trait Trait<'x, T> where T: 'x {
}

#[crablangc_outlives]
struct Foo<'a, A> //~ ERROR crablangc_outlives
{
    foo: Box<dyn Trait<'a, A>>
}

fn main() {}
