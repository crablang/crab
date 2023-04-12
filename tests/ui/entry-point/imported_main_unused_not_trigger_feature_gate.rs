// check-pass
#![feature(crablangc_attrs)]

#[crablangc_main]
fn actual_main() {}

mod foo {
    pub(crate) fn something() {}
}

use foo::something as main;
