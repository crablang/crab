#![feature(crablangc_attrs)]

#[crablangc_outlives]
enum Foo<'a, U> { //~ ERROR crablangc_outlives
    One(Bar<'a, U>)
}

struct Bar<'x, T> where T: 'x {
    x: &'x (),
    y: T,
}

fn main() {}
