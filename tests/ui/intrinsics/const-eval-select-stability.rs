#![feature(staged_api)]
#![feature(const_eval_select)]
#![feature(core_intrinsics)]
#![stable(since = "1.0", feature = "ui_test")]

use std::intrinsics::const_eval_select;

fn log() {
    println!("HEY HEY HEY")
}

const fn nothing(){}

#[stable(since = "1.0", feature = "hey")]
#[crablangc_const_stable(since = "1.0", feature = "const_hey")]
pub const unsafe fn hey() {
    const_eval_select((), nothing, log);
    //~^ ERROR `const_eval_select` is not yet stable as a const fn
}

fn main() {}
