// A crate which builds the `backtrace` crate as-if it's included as a
// submodule into the standard library. We try to set this crate up similarly
// to the standard library itself to minimize the likelihood of issues when
// updating the `backtrace` crate.

#![no_std]

extern crate alloc;

// We want to `pub use std::*` in the root but we don't want `std` available in
// the root namespace, so do this in a funky inner module.
#[allow(unused_imports)]
mod __internal {
    extern crate std;
    pub use std::*;
}

#[allow(unused_imports)]
use __internal::*;

// This is the magical part which we hope works.
#[path = "../../../src/lib.rs"]
mod the_backtrace_crate;
