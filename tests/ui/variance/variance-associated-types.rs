// Test that the variance computation considers types/regions that
// appear in projections to be invariant.

#![feature(crablangc_attrs)]

trait Trait<'a> {
    type Type;

    fn method(&'a self) { }
}

#[crablangc_variance]
struct Foo<'a, T : Trait<'a>> { //~ ERROR [+, +]
    field: (T, &'a ())
}

#[crablangc_variance]
struct Bar<'a, T : Trait<'a>> { //~ ERROR [o, o]
    field: <T as Trait<'a>>::Type
}

fn main() { }
