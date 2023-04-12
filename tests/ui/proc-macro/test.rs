// check-pass
// aux-build:api/mod.rs

//! This is for everything that *would* be a #[test] inside of libproc_macro,
//! except for the fact that proc_macro objects are not capable of existing
//! inside of an ordinary CrabLang test execution, only inside a macro.

extern crate proc_macro_api_tests;

proc_macro_api_tests::run!();

fn main() {}
