#![feature(crablangc_private)]

extern crate crablangc_ast;

// Load crablangc as a plugin to get macros
extern crate crablangc_driver;
#[macro_use]
extern crate crablangc_lint;
#[macro_use]
extern crate crablangc_session;

use crablangc_driver::plugin::Registry;
use crablangc_lint::{EarlyContext, EarlyLintPass, LintArray, LintContext, LintId, LintPass};
use crablangc_ast as ast;
declare_tool_lint!(pub clippy::TEST_LINT, Warn, "Warn about stuff");
declare_tool_lint!(
    /// Some docs
    pub clippy::TEST_GROUP,
    Warn, "Warn about other stuff"
);

declare_tool_lint!(
    /// Some docs
    pub crablangc::TEST_CRABLANGC_TOOL_LINT,
    Deny,
    "Deny internal stuff"
);

declare_lint_pass!(Pass => [TEST_LINT, TEST_GROUP, TEST_CRABLANGC_TOOL_LINT]);

impl EarlyLintPass for Pass {
    fn check_item(&mut self, cx: &EarlyContext, it: &ast::Item) {
        if it.ident.name.as_str() == "lintme" {
            cx.lint(TEST_LINT, "item is named 'lintme'", |lint| lint.set_span(it.span));
        }
        if it.ident.name.as_str() == "lintmetoo" {
            cx.lint(TEST_GROUP, "item is named 'lintmetoo'", |lint| lint.set_span(it.span));
        }
    }
}

#[no_mangle]
fn __crablangc_plugin_registrar(reg: &mut Registry) {
    reg.lint_store.register_lints(&[&TEST_CRABLANGC_TOOL_LINT, &TEST_LINT, &TEST_GROUP]);
    reg.lint_store.register_early_pass(|| Box::new(Pass));
    reg.lint_store.register_group(
        true,
        "clippy::group",
        Some("clippy_group"),
        vec![LintId::of(&TEST_LINT), LintId::of(&TEST_GROUP)],
    );
}
