// This checks that CRABLANG_TEST_THREADS not being 1, 2, ... is detected
// properly.

// run-fail
// error-pattern:should be a positive integer
// compile-flags: --test
// exec-env:CRABLANG_TEST_THREADS=foo
// ignore-emscripten

#[test]
fn do_nothing() {}
