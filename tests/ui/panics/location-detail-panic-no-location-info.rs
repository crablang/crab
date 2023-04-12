// run-fail
// check-run-results
// compile-flags: -Zlocation-detail=none
// exec-env:CRABLANG_BACKTRACE=0

fn main() {
    panic!("no location info");
}
