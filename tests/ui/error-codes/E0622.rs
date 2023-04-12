#![feature(intrinsics)]
extern "crablang-intrinsic" {
    pub static breakpoint : unsafe extern "crablang-intrinsic" fn();
    //~^ ERROR intrinsic must be a function [E0622]
}
fn main() { unsafe { breakpoint(); } }
