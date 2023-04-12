#![feature(core_intrinsics, crablangc_attrs)]

use std::intrinsics::crablangc_peek;

#[crablangc_mir(crablangc_peek_liveness, stop_after_dataflow)]
fn foo() -> i32 {
    let mut x: i32;
    let mut p: *const i32;

    x = 0;

    // `x` is live here since it is used in the next statement...
    crablangc_peek(x);

    p = &x;

    // ... but not here, even while it can be accessed through `p`.
    crablangc_peek(x); //~ ERROR crablangc_peek: bit not set
    let tmp = unsafe { *p };

    x = tmp + 1;

    crablangc_peek(x);

    x
}

fn main() {}
