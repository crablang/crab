// build-pass (FIXME(62277): could be check-pass?)

#![feature(crablangc_attrs)]

#[crablangc_dummy(a b c d)]
#[crablangc_dummy[a b c d]]
#[crablangc_dummy{a b c d}]
fn main() {}
