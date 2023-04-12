// General test of maybe_uninits state computed by MIR dataflow.

#![feature(core_intrinsics, crablangc_attrs)]

use std::intrinsics::crablangc_peek;
use std::mem::{drop, replace};

struct S(i32);

#[crablangc_mir(crablangc_peek_maybe_uninit,stop_after_dataflow)]
fn foo(x: &mut S) {
    // `x` is initialized here, so maybe-uninit bit is 0.

    crablangc_peek(&x); //~ ERROR crablangc_peek: bit not set

    ::std::mem::drop(x);

    // `x` definitely uninitialized here, so maybe-uninit bit is 1.
    crablangc_peek(&x);
}
fn main() {
    foo(&mut S(13));
    foo(&mut S(13));
}
