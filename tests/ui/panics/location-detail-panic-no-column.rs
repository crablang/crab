// run-fail
// check-run-results
// compile-flags: -Zlocation-detail=line,file
// exec-env:CRABLANG_BACKTRACE=0

fn main() {
    panic!("column-redacted");
}
