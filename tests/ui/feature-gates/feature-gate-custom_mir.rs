#![feature(core_intrinsics)]

extern crate core;

#[custom_mir(dialect = "built")] //~ ERROR the `#[custom_mir]` attribute is just used for the CrabLang test suite
pub fn foo(_x: i32) -> i32 {
    0
}

fn main() {
    assert_eq!(2, foo(2));
}
