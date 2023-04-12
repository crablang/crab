#[crablangc_attribute_should_be_reserved]
//~^ ERROR cannot find attribute `crablangc_attribute_should_be_reserved` in this scope
//~| ERROR attributes starting with `crablangc` are reserved for use by the `crablangc` compiler

macro_rules! foo {
    () => (());
}

fn main() {
    foo!(); //~ ERROR cannot determine resolution for the macro `foo`
}
