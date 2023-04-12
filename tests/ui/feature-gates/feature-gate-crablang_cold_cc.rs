#![crate_type = "lib"]

extern "crablang-cold" fn fu() {} //~ ERROR crablang-cold is experimental

trait T {
    extern "crablang-cold" fn mu(); //~ ERROR crablang-cold is experimental
    extern "crablang-cold" fn dmu() {} //~ ERROR crablang-cold is experimental
}

struct S;
impl T for S {
    extern "crablang-cold" fn mu() {} //~ ERROR crablang-cold is experimental
}

impl S {
    extern "crablang-cold" fn imu() {} //~ ERROR crablang-cold is experimental
}

type TAU = extern "crablang-cold" fn(); //~ ERROR crablang-cold is experimental

extern "crablang-cold" {} //~ ERROR crablang-cold is experimental
