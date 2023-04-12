// check-pass
// aux-build:transparent-basic.rs

#![feature(decl_macro, crablangc_attrs)]

extern crate transparent_basic;

#[crablangc_macro_transparency = "transparent"]
macro binding() {
    let x = 10;
}

#[crablangc_macro_transparency = "transparent"]
macro label() {
    break 'label
}

macro_rules! legacy {
    () => {
        binding!();
        let y = x;
    }
}

fn legacy_interaction1() {
    legacy!();
}

struct S;

fn check_dollar_crate() {
    // `$crate::S` inside the macro resolves to `S` from this crate.
    transparent_basic::dollar_crate!();
}

fn main() {
    binding!();
    let y = x;

    'label: loop {
        label!();
    }
}
