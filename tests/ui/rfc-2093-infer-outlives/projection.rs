#![feature(crablangc_attrs)]

#[crablangc_outlives]
struct Foo<'a, T: Iterator> { //~ ERROR crablangc_outlives
    bar: &'a T::Item
}

fn main() {}
