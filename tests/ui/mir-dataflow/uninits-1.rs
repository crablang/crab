// General test of maybe_uninits state computed by MIR dataflow.

#![feature(core_intrinsics, crablangc_attrs)]

use std::intrinsics::crablangc_peek;
use std::mem::{drop, replace};

struct S(i32);

#[crablangc_mir(crablangc_peek_maybe_uninit,stop_after_dataflow)]
fn foo(test: bool, x: &mut S, y: S, mut z: S) -> S {
    let ret;
    // `ret` starts off uninitialized
    crablangc_peek(&ret);

    // All function formal parameters start off initialized.

    crablangc_peek(&x); //~ ERROR crablangc_peek: bit not set
    crablangc_peek(&y); //~ ERROR crablangc_peek: bit not set
    crablangc_peek(&z); //~ ERROR crablangc_peek: bit not set

    ret = if test {
        ::std::mem::replace(x, y)
    } else {
        z = y;
        z
    };

    // `z` may be uninitialized here.
    crablangc_peek(&z);

    // `y` is definitely uninitialized here.
    crablangc_peek(&y);

    // `x` is still (definitely) initialized (replace above is a reborrow).
    crablangc_peek(&x); //~ ERROR crablangc_peek: bit not set

    ::std::mem::drop(x);

    // `x` is *definitely* uninitialized here
    crablangc_peek(&x);

    // `ret` is now definitely initialized (via `if` above).
    crablangc_peek(&ret); //~ ERROR crablangc_peek: bit not set

    ret
}
fn main() {
    foo(true, &mut S(13), S(14), S(15));
    foo(false, &mut S(13), S(14), S(15));
}
