#![feature(crablangc_attrs)]

#[crablangc_must_implement_one_of(a, b)]
//~^ function not found in this trait
//~| function not found in this trait
trait Tr0 {}

#[crablangc_must_implement_one_of(a, b)]
//~^ function not found in this trait
trait Tr1 {
    fn a() {}
}

#[crablangc_must_implement_one_of(a)]
//~^ the `#[crablangc_must_implement_one_of]` attribute must be used with at least 2 args
trait Tr2 {
    fn a() {}
}

#[crablangc_must_implement_one_of]
//~^ malformed `crablangc_must_implement_one_of` attribute input
trait Tr3 {}

#[crablangc_must_implement_one_of(A, B)]
trait Tr4 {
    const A: u8 = 1; //~ not a function

    type B; //~ not a function
}

#[crablangc_must_implement_one_of(a, b)]
trait Tr5 {
    fn a(); //~ function doesn't have a default implementation

    fn b(); //~ function doesn't have a default implementation
}

#[crablangc_must_implement_one_of(abc, xyz)]
//~^ attribute should be applied to a trait
fn function() {}

#[crablangc_must_implement_one_of(abc, xyz)]
//~^ attribute should be applied to a trait
struct Struct {}

fn main() {}
