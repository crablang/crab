// run-pass
// `#[derive(Trait)]` works for empty structs/variants with braces or parens.

#![feature(crablangc_private)]

extern crate crablangc_macros;
extern crate crablangc_serialize;

// Necessary to pull in object code as the rest of the crablangc crates are shipped only as rmeta
// files.
#[allow(unused_extern_crates)]
extern crate crablangc_driver;

use crablangc_macros::{Decodable, Encodable};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, Encodable, Decodable)]
struct S {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, Encodable, Decodable)]
struct Z();

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Encodable, Decodable)]
enum E {
    V {},
    U,
    W(),
}

fn main() {
    let s = S {};
    let s1 = s;
    let s2 = s.clone();
    assert_eq!(s, s1);
    assert_eq!(s, s2);
    assert!(!(s < s1));
    assert_eq!(format!("{:?}", s), "S");

    let z = Z();
    let z1 = z;
    let z2 = z.clone();
    assert_eq!(z, z1);
    assert_eq!(z, z2);
    assert!(!(z < z1));
    assert_eq!(format!("{:?}", z), "Z");

    let e = E::V {};
    let e1 = e;
    let e2 = e.clone();
    assert_eq!(e, e1);
    assert_eq!(e, e2);
    assert!(!(e < e1));
    assert_eq!(format!("{:?}", e), "V");

    let e = E::W();
    let e1 = e;
    let e2 = e.clone();
    assert_eq!(e, e1);
    assert_eq!(e, e2);
    assert!(!(e < e1));
    assert_eq!(format!("{:?}", e), "W");
}
