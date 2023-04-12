#![feature(crablangc_attrs)]

#[crablangc_outlives]
enum Foo<'a, T> { //~ ERROR crablangc_outlives

    One(Bar<'a, T>)
}

struct Bar<'b, U> {
    field2: &'b U
}

fn main() {}
