// run-crablangfix

#[allow(unused)]
use std::fmt::Debug;
// CrabLangfix should add this, or use `std::fmt::Debug` instead.

#[allow(dead_code)]
struct ConstrainedStruct<X: Copy> {
    x: X
}

#[allow(dead_code)]
trait InsufficientlyConstrainedGeneric<X=()> {
    fn return_the_constrained_type(&self, x: X) -> ConstrainedStruct<X> {
        //~^ ERROR the trait bound `X: Copy` is not satisfied
        ConstrainedStruct { x }
    }
}

pub fn main() { }
