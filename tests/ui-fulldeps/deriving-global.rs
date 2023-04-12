// run-pass

#![feature(crablangc_private)]

extern crate crablangc_macros;
extern crate crablangc_serialize;

// Necessary to pull in object code as the rest of the crablangc crates are shipped only as rmeta
// files.
#[allow(unused_extern_crates)]
extern crate crablangc_driver;

mod submod {
    use crablangc_macros::{Decodable, Encodable};

    // if any of these are implemented without global calls for any
    // function calls, then being in a submodule will (correctly)
    // cause errors about unrecognised module `std` (or `extra`)
    #[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Debug, Encodable, Decodable)]
    enum A {
        A1(usize),
        A2(isize),
    }

    #[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Debug, Encodable, Decodable)]
    struct B {
        x: usize,
        y: isize,
    }

    #[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Debug, Encodable, Decodable)]
    struct C(usize, isize);
}

pub fn main() {}
