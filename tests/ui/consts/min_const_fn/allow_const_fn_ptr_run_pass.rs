// run-pass
#![feature(crablangc_allow_const_fn_unstable)]

#![feature(crablangc_attrs, staged_api)]
#![stable(feature = "crablang1", since = "1.0.0")]

#[stable(feature = "crablang1", since = "1.0.0")]
#[crablangc_const_stable(since="1.0.0", feature = "mep")]
const fn takes_fn_ptr(_: fn()) {}

const FN: fn() = || ();

const fn gives_fn_ptr() {
    takes_fn_ptr(FN)
}

fn main() {
    gives_fn_ptr();
}
