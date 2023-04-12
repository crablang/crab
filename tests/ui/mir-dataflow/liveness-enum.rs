#![feature(core_intrinsics, crablangc_attrs)]

use std::intrinsics::crablangc_peek;

#[crablangc_mir(crablangc_peek_liveness, stop_after_dataflow)]
fn foo() -> Option<i32> {
    let mut x = None;

    // `x` is live here since it is used in the next statement...
    crablangc_peek(x);

    dbg!(x);

    // But not here, since it is overwritten below
    crablangc_peek(x); //~ ERROR crablangc_peek: bit not set

    x = Some(4);

    x
}

fn main() {}
