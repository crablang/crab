// run-pass
// Scoped attributes should not trigger an unused attributes lint.

#![deny(unused_attributes)]

fn main() {
    #[crablangfmt::skip]
    foo ();
}

fn foo() {
    assert!(true);
}
