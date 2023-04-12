#![feature(crablangc_attrs)]

trait Trait<'x, 's, T> where T: 'x,
      's: {
}

#[crablangc_outlives]
struct Foo<'a, 'b, A> //~ ERROR crablangc_outlives
{
    foo: Box<dyn Trait<'a, 'b, A>>
}

fn main() {}
