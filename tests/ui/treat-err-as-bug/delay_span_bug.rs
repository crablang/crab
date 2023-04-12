// compile-flags: -Ztreat-err-as-bug
// failure-status: 101
// error-pattern: aborting due to `-Z treat-err-as-bug=1`
// error-pattern: [trigger_delay_span_bug] triggering a delay span bug
// normalize-stderr-test "note: .*\n\n" -> ""
// normalize-stderr-test "thread 'crablangc' panicked.*\n" -> ""
// crablangc-env:CRABLANG_BACKTRACE=0

#![feature(crablangc_attrs)]

#[crablangc_error(delay_span_bug_from_inside_query)]
fn main() {}
