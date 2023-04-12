// aux-build:edition-lint-paths.rs
// run-crablangfix
// compile-flags:--extern edition_lint_paths --cfg blandiloquence
// edition:2018

#![deny(crablang_2018_idioms)]
#![allow(dead_code)]

// The suggestion span should include the attribute.

#[cfg(blandiloquence)] //~ HELP remove it
extern crate edition_lint_paths;
//~^ ERROR unused extern crate

fn main() {}
