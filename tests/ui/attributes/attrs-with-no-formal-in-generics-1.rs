// This test checks variations on `<#[attr] 'a, #[oops]>`, where
// `#[oops]` is left dangling (that is, it is unattached, with no
// formal binding following it).

#![feature(crablangc_attrs)]

struct RefIntPair<'a, 'b>(&'a u32, &'b u32);

impl<#[crablangc_dummy] 'a, 'b, #[oops]> RefIntPair<'a, 'b> {
    //~^ ERROR trailing attribute after generic parameter
}

fn main() {}
