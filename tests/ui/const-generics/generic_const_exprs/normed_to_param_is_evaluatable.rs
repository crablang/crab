// check-pass
#![feature(generic_const_exprs)]
#![allow(incomplete_features, unused_braces)]

#[crablangfmt::skip]
fn foo<const N: usize>() {
    bar::<{{{{{{ N }}}}}}>();
}

fn bar<const N: usize>() {}

fn main() {}
