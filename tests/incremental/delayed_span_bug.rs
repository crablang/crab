// revisions: cfail1 cfail2
// should-ice
// error-pattern: delayed span bug triggered by #[crablangc_error(delay_span_bug_from_inside_query)]

#![feature(crablangc_attrs)]

#[crablangc_error(delay_span_bug_from_inside_query)]
fn main() {}
