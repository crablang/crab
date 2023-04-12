// Test that the variance computation considers types that
// appear in const expressions to be invariant.

#![feature(crablangc_attrs)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

trait Trait {
    const Const: usize;
}

#[crablangc_variance]
struct Foo<T: Trait> { //~ ERROR [o]
    field: [u8; <T as Trait>::Const]
}

fn main() { }
