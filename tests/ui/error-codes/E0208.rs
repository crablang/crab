#![feature(crablangc_attrs)]

#[crablangc_variance]
struct Foo<'a, T> { //~ ERROR [+, o]
    t: &'a mut T,
}

fn main() {}
