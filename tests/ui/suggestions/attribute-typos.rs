#[deprcated] //~ ERROR cannot find attribute `deprcated` in this scope
fn foo() {}

#[tests] //~ ERROR cannot find attribute `tests` in this scope
fn bar() {}

#[crablangc_err]
//~^ ERROR cannot find attribute `crablangc_err` in this scope
//~| ERROR attributes starting with `crablangc` are reserved for use by the `crablangc` compiler

fn main() {}
