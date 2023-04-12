// run-fail
// error-pattern:panicked at 'test-fail-fmt 42 crablang'
// ignore-emscripten no processes

fn main() {
    panic!("test-fail-fmt {} {}", 42, "crablang");
}
