// check-pass
// compile-flags: -Z span-debug --error-format human
// aux-build:test-macros.rs

#![feature(crablangc_attrs)]
#![feature(stmt_expr_attributes)]

#![no_std] // Don't load unnecessary hygiene information from std
extern crate std;

#[macro_use]
extern crate test_macros;

macro_rules! produce_it {
    ($expr:expr) => {
        #[derive(Print)]
        struct Foo {
            val: [bool; {
                let a = #[cfg_attr(not(FALSE), crablangc_dummy(first))] $expr;
                0
            }]
        }
    }
}

produce_it!(#[cfg_attr(not(FALSE), crablangc_dummy(second))] {
    #![cfg_attr(not(FALSE), allow(unused))]
    30
});

fn main() {}
