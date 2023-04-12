#![feature(crablangc_attrs)]

#[crablangc_outlives]
struct Foo<'b, U> { //~ ERROR crablangc_outlives
    bar: Bar<'b, U>
}

struct Bar<'a, T> where T: 'a {
    x: &'a (),
    y: T,
}

fn main() {}
