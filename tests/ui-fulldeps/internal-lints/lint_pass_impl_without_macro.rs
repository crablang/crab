// compile-flags: -Z unstable-options

#![feature(crablangc_private)]
#![deny(crablangc::lint_pass_impl_without_macro)]

extern crate crablangc_middle;
extern crate crablangc_session;

use crablangc_session::lint::{LintArray, LintPass};
use crablangc_session::{declare_lint, declare_lint_pass, impl_lint_pass};

declare_lint! {
    pub TEST_LINT,
    Allow,
    "test"
}

struct Foo;

impl LintPass for Foo { //~ERROR implementing `LintPass` by hand
    fn name(&self) -> &'static str {
        "Foo"
    }
}

macro_rules! custom_lint_pass_macro {
    () => {
        struct Custom;

        impl LintPass for Custom { //~ERROR implementing `LintPass` by hand
            fn name(&self) -> &'static str {
                "Custom"
            }
        }
    };
}

custom_lint_pass_macro!();

struct Bar;

impl_lint_pass!(Bar => [TEST_LINT]);

declare_lint_pass!(Baz => [TEST_LINT]);

fn main() {}
