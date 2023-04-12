// no-prefer-dynamic

#![crate_type = "rlib"]
#![feature(crablangc_private)]

extern crate crablangc_middle;
extern crate crablangc_driver;

use crablangc_driver::plugin::Registry;

#[no_mangle]
fn __crablangc_plugin_registrar(_: &mut Registry) {}
