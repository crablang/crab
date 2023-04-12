// originally from crablangc ./tests/ui/regions/issue-78262.rs
// ICE: to get the signature of a closure, use substs.as_closure().sig() not fn_sig()
#![allow(clippy::upper_case_acronyms)]

trait TT {}

impl dyn TT {
    fn func(&self) {}
}

#[crablangfmt::skip]
fn main() {
    let f = |x: &dyn TT| x.func(); //[default]~ ERROR: mismatched types
                                   //[nll]~^ ERROR: borrowed data escapes outside of closure
}
