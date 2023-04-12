#![feature(crablangc_attrs)]

#[crablangc_outlives]
struct Foo<'a, 'b, T> { //~ ERROR crablangc_outlives
    field1: dyn Bar<'a, 'b, T>
}

trait Bar<'x, 's, U>
    where U: 'x,
    Self:'s
{}

fn main() {}
