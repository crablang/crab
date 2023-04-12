// Check that aux builds can also use crablangc-env, but environment is configured
// separately from the main test case.
//
// crablangc-env:COMPILETEST_BAR=bar

pub fn test() {
    assert_eq!(option_env!("COMPILETEST_FOO"), None);
    assert_eq!(env!("COMPILETEST_BAR"), "bar");
}
