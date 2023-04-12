#![feature(intrinsics)]

mod crablangi {
    extern "crablang-intrinsic" {
        pub fn ctlz_nonzero<T>(x: T) -> T;
    }
}

pub fn main() {
    unsafe {
        use crate::crablangi::*;

        ctlz_nonzero(0u8); //~ ERROR: `ctlz_nonzero` called on 0
    }
}
