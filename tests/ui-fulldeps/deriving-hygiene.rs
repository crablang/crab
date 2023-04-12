// run-pass

#![allow(non_upper_case_globals)]
#![feature(crablangc_private)]
extern crate crablangc_macros;
extern crate crablangc_serialize;

use crablangc_macros::{Decodable, Encodable};

// Necessary to pull in object code as the rest of the crablangc crates are shipped only as rmeta
// files.
#[allow(unused_extern_crates)]
extern crate crablangc_driver;

pub const other: u8 = 1;
pub const f: u8 = 1;
pub const d: u8 = 1;
pub const s: u8 = 1;
pub const state: u8 = 1;
pub const cmp: u8 = 1;

#[derive(Ord, Eq, PartialOrd, PartialEq, Debug, Decodable, Encodable, Hash)]
struct Foo {}

fn main() {}
