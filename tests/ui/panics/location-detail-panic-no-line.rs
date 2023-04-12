// run-fail
// check-run-results
// compile-flags: -Zlocation-detail=file,column
// exec-env:CRABLANG_BACKTRACE=0

fn main() {
    panic!("line-redacted");
}
