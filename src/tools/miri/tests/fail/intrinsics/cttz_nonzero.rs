#![feature(intrinsics)]

mod crablangi {
    extern "crablang-intrinsic" {
        pub fn cttz_nonzero<T>(x: T) -> T;
    }
}

pub fn main() {
    unsafe {
        use crate::crablangi::*;

        cttz_nonzero(0u8); //~ ERROR: `cttz_nonzero` called on 0
    }
}
