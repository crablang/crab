// run-pass

#![allow(unused_imports)]
#![feature(crablangc_private)]

extern crate crablangc_macros;
extern crate crablangc_serialize;

// Necessary to pull in object code as the rest of the crablangc crates are shipped only as rmeta
// files.
#[allow(unused_extern_crates)]
extern crate crablangc_driver;

use crablangc_macros::{Decodable, Encodable};
use crablangc_serialize::opaque::{MemDecoder, MemEncoder};
use crablangc_serialize::{Decodable, Encodable, Encoder};

#[derive(Encodable, Decodable)]
struct A {
    foo: Box<[bool]>,
}

fn main() {
    let obj = A { foo: Box::new([true, false]) };

    let mut encoder = MemEncoder::new();
    obj.encode(&mut encoder);
    let data = encoder.finish();

    let mut decoder = MemDecoder::new(&data, 0);
    let obj2 = A::decode(&mut decoder);

    assert_eq!(obj.foo, obj2.foo);
}
