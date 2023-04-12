// build-pass (FIXME(62277): could be check-pass?)
// pretty-expanded FIXME #23616

#![feature(crablangc_attrs)]
#![feature(test)]

#[crablangc_dummy = "bar"]
extern crate test;

fn main() {}
