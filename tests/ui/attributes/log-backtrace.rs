// run-pass
//
// This test makes sure that log-backtrace option at least parses correctly
//
// dont-check-compiler-stdout
// dont-check-compiler-stderr
// crablangc-env:CRABLANGC_LOG=info
// crablangc-env:CRABLANGC_LOG_BACKTRACE=crablangc_metadata::creader
fn main() {}
