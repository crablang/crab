// build-fail
// compile-flags:-Zpolymorphize=on
#![feature(generic_const_exprs, crablangc_attrs)]
//~^ WARN the feature `generic_const_exprs` is incomplete

// This test checks that the polymorphization analysis correctly detects unused const
// parameters in functions.

// Function doesn't have any generic parameters to be unused.
#[crablangc_polymorphize_error]
pub fn no_parameters() {}

// Function has an unused generic parameter.
#[crablangc_polymorphize_error]
pub fn unused<const T: usize>() {
    //~^ ERROR item has unused generic parameters
}

// Function uses generic parameter in value of a binding.
#[crablangc_polymorphize_error]
pub fn used_binding<const T: usize>() -> usize {
    let x: usize = T;
    x
}

// Function uses generic parameter in substitutions to another function.
#[crablangc_polymorphize_error]
pub fn used_substs<const T: usize>() {
    unused::<T>()
}

fn main() {
    no_parameters();
    unused::<1>();
    used_binding::<1>();
    used_substs::<1>();
}
