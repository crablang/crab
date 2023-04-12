#![feature(intrinsics)]
extern "crablang-intrinsic" {
    fn atomic_foo(); //~ ERROR E0092
}

fn main() {
}
