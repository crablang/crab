#![feature(crablangc_attrs)]

#[crablangc_outlives]
struct Foo<'a, 'b, T> { //~ ERROR crablangc_outlives
    x: &'a &'b T
}

fn main() {}
