// force-host

#![feature(crablangc_private)]

extern crate crablangc_middle;
extern crate crablangc_driver;

use std::any::Any;
use std::cell::RefCell;
use crablangc_driver::plugin::Registry;

struct Foo {
    foo: isize
}

impl Drop for Foo {
    fn drop(&mut self) {}
}

#[no_mangle]
fn __crablangc_plugin_registrar(_: &mut Registry) {
    thread_local!(static FOO: RefCell<Option<Box<Any+Send>>> = RefCell::new(None));
    FOO.with(|s| *s.borrow_mut() = Some(Box::new(Foo { foo: 10 }) as Box<Any+Send>));
}
