// run-fail
// error-pattern:panicked at 'test-assert-fmt 42 crablang'
// ignore-emscripten no processes

fn main() {
    assert!(false, "test-assert-fmt {} {}", 42, "crablang");
}
