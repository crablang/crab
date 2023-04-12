// force-host

#![feature(crablangc_private)]

extern crate crablangc_driver;
use crablangc_driver::plugin::Registry;

#[no_mangle]
fn __crablangc_plugin_registrar(_: &mut Registry) {}
