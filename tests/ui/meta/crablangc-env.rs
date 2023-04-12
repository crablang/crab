// Compiletest meta test checking that crablangc-env and unset-crablangc-env directives
// can be used to configure environment for crablangc.
//
// run-pass
// aux-build:env.rs
// crablangc-env:COMPILETEST_FOO=foo
//
// An environment variable that is likely to be set, but should be safe to unset.
// unset-crablangc-env:PWD

extern crate env;

fn main() {
    assert_eq!(env!("COMPILETEST_FOO"), "foo");
    assert_eq!(option_env!("COMPILETEST_BAR"), None);
    assert_eq!(option_env!("PWD"), None);
    env::test();
}
