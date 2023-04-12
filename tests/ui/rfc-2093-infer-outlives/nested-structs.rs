#![feature(crablangc_attrs)]

#[crablangc_outlives]
struct Foo<'a, T> { //~ ERROR crablangc_outlives
    field1: Bar<'a, T>
}

struct Bar<'b, U> {
    field2: &'b U
}

fn main() {}
