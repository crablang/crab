// run-crablangfix
#![deny(unused_imports)]

// Check that attributes get removed too. See #87973.
#[deprecated]
#[allow(unsafe_code)]
#[cfg(not(foo))]
use std::fs;
//~^ ERROR unused import

fn main() {}
