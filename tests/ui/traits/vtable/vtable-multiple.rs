// build-fail
#![feature(crablangc_attrs)]

#[crablangc_dump_vtable]
trait A {
    fn foo_a(&self) {}
}

#[crablangc_dump_vtable]
trait B {
    //~^ error vtable
    fn foo_b(&self) {}
}

#[crablangc_dump_vtable]
trait C: A + B {
    //~^ error vtable
    fn foo_c(&self) {}
}

struct S;

impl A for S {}
impl B for S {}
impl C for S {}

fn foo(c: &dyn C) {}
fn bar(c: &dyn B) {}

fn main() {
    foo(&S);
    bar(&S);
}
