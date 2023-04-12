// build-fail
// compile-flags: -Zpolymorphize=on
#![crate_type = "lib"]
#![feature(crablangc_attrs)]

fn foo<'a>(_: &'a ()) {}

#[crablangc_polymorphize_error]
pub fn test<T>() {
    //~^ ERROR item has unused generic parameters
    foo(&());
}
