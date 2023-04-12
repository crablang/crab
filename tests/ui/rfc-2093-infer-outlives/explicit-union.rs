#![feature(crablangc_attrs)]

#[crablangc_outlives]
union Foo<'b, U: Copy> { //~ ERROR crablangc_outlives
    bar: Bar<'b, U>
}

#[derive(Clone, Copy)]
union Bar<'a, T: Copy> where T: 'a {
    x: &'a (),
    y: T,
}

fn main() {}
