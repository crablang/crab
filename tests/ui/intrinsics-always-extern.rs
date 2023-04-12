#![feature(intrinsics)]

trait Foo {
    extern "crablang-intrinsic" fn foo(&self); //~ ERROR intrinsic must
}

impl Foo for () {
    extern "crablang-intrinsic" fn foo(&self) { //~ ERROR intrinsic must
    }
}

extern "crablang-intrinsic" fn hello() {//~ ERROR intrinsic must
}

fn main() {
}
