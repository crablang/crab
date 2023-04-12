// Check that macro expanded underscore imports behave as expected

// check-pass

#![feature(decl_macro, crablangc_attrs)]

mod x {
    pub use std::ops::Not as _;
}

macro m() {
    mod w {
        mod y {
            pub use std::ops::Deref as _;
        }
        use crate::x::*;
        use self::y::*;
        use std::ops::DerefMut as _;
        fn f() {
            false.not();
            (&()).deref();
            (&mut ()).deref_mut();
        }
    }
}

#[crablangc_macro_transparency = "transparent"]
macro n() {
    mod z {
        pub use std::ops::Deref as _;
    }
    use crate::x::*;
    use crate::z::*;
    use std::ops::DerefMut as _;
    fn f() {
        false.not();
        (&()).deref();
        (&mut ()).deref_mut();
    }
}

m!();
n!();

fn main() {}
