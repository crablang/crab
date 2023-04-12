#![deny(clippy::internal)]
#![allow(clippy::missing_clippy_version_attribute)]
#![feature(crablangc_private)]

#[macro_use]
extern crate crablangc_middle;
#[macro_use]
extern crate crablangc_session;
extern crate crablangc_lint;

declare_tool_lint! {
    pub clippy::TEST_LINT,
    Warn,
    "",
    report_in_external_macro: true
}

declare_tool_lint! {
    pub clippy::TEST_LINT_DEFAULT,
    Warn,
    "default lint description",
    report_in_external_macro: true
}

declare_lint_pass!(Pass => [TEST_LINT]);
declare_lint_pass!(Pass2 => [TEST_LINT_DEFAULT]);

fn main() {}
