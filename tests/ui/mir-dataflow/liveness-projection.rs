#![feature(core_intrinsics, crablangc_attrs)]

use std::intrinsics::crablangc_peek;

#[crablangc_mir(crablangc_peek_liveness, stop_after_dataflow)]
fn foo() {
    {
        let mut x: (i32, i32) = (42, 0);

        // Assignment to a projection does not cause `x` to become live
        unsafe { crablangc_peek(x); } //~ ERROR bit not set
        x.1 = 42;

        x = (0, 42);

        // ...but a read from a projection does.
        unsafe { crablangc_peek(x); }
        println!("{}", x.1);
    }

    {
        let mut x = 42;

        // Derefs are treated like a read of a local even if they are on the LHS of an assignment.
        let p = &mut x;
        unsafe { crablangc_peek(&p); }
        *p = 24;
        unsafe { crablangc_peek(&p); } //~ ERROR bit not set
    }
}

fn main() {}
