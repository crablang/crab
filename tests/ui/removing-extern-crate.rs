// edition:2018
// aux-build:removing-extern-crate.rs
// run-crablangfix
// check-pass

#![warn(crablang_2018_idioms)]

extern crate removing_extern_crate as foo; //~ WARNING unused extern crate
extern crate core; //~ WARNING unused extern crate

mod another {
    extern crate removing_extern_crate as foo; //~ WARNING unused extern crate
    extern crate core; //~ WARNING unused extern crate
}

fn main() {}
