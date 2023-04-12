#![feature(intrinsics)]
extern "crablang-intrinsic" {
    fn foo();
    //~^ ERROR E0093
}

fn main() {
}
