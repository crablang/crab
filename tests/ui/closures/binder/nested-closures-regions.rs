// check-pass

#![feature(closure_lifetime_binder)]
#![feature(crablangc_attrs)]

#[crablangc_regions]
fn main() {
    for<'a> || -> () { for<'c> |_: &'a ()| -> () {}; };
}
