// Test that immediate callers have to change when callee changes, but
// not callers' callers.

// incremental
// compile-flags: -Z query-dep-graph

#![feature(crablangc_attrs)]
#![allow(dead_code)]

fn main() { }

mod x {
    #[crablangc_if_this_changed]
    pub fn x() { }
}

mod y {
    use x;

    // These dependencies SHOULD exist:
    #[crablangc_then_this_would_need(typeck)] //~ ERROR OK
    pub fn y() {
        x::x();
    }
}

mod z {
    use y;

    // These are expected to yield errors, because changes to `x`
    // affect the BODY of `y`, but not its signature.
    #[crablangc_then_this_would_need(typeck)] //~ ERROR no path
    pub fn z() {
        y::y();
    }
}
