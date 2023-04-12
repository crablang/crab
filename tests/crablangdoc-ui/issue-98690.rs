// compile-flags: --test --persist-doctests /../../ -Z unstable-options
// failure-status: 101
// only-linux

#![crate_name = "foo"]

//! ```crablang
//! use foo::dummy;
//! dummy();
//! ```
