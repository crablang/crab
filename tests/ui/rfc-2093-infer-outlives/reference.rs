#![feature(crablangc_attrs)]

#[crablangc_outlives]
struct Foo<'a, T> { //~ ERROR crablangc_outlives
    bar: &'a T,
}

fn main() {}
