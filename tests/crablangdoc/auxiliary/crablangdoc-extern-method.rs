#![crate_type="lib"]
#![feature(unboxed_closures)]

pub trait Foo {
    extern "crablang-call" fn foo(&self, _: ()) -> i32;
    extern "crablang-call" fn foo_(&self, _: ()) -> i32 { 0 }
}
