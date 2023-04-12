// See issue #100696.
// run-fail
// check-run-results
// exec-env:CRABLANG_BACKTRACE=0
fn main() {
    &""[1..];
}
