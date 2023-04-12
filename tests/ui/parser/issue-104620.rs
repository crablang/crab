#![feature(crablangc_attrs)]

#![crablangc_dummy=5z] //~ ERROR unexpected expression: `5z`
fn main() {}
