// run-pass

#![feature(crablangc_private)]

extern crate crablangc_macros;
#[allow(dead_code)]
extern crate crablangc_serialize;

// Necessary to pull in object code as the rest of the crablangc crates are shipped only as rmeta
// files.
#[allow(unused_extern_crates)]
extern crate crablangc_driver;

use crablangc_macros::{Decodable, Encodable};

#[derive(Decodable, Encodable, Debug)]
struct A {
    a: String,
}

trait Trait {
    fn encode(&self);
}

impl<T> Trait for T {
    fn encode(&self) {
        unimplemented!()
    }
}

fn main() {}
