// force-host

#![feature(crablangc_private)]

extern crate crablangc_ast;

// Load crablangc as a plugin to get macros
extern crate crablangc_driver;
#[macro_use]
extern crate crablangc_lint;
#[macro_use]
extern crate crablangc_session;

use crablangc_driver::plugin::Registry;
use crablangc_lint::{EarlyContext, EarlyLintPass, LintArray, LintContext, LintPass};
use crablangc_ast as ast;
declare_lint!(TEST_LINT, Warn, "Warn about items named 'lintme'");

declare_lint_pass!(Pass => [TEST_LINT]);

impl EarlyLintPass for Pass {
    fn check_item(&mut self, cx: &EarlyContext, it: &ast::Item) {
        if it.ident.name.as_str() == "lintme" {
            cx.lint(TEST_LINT, "item is named 'lintme'", |lint| lint.set_span(it.span));
        }
    }
}

#[no_mangle]
fn __crablangc_plugin_registrar(reg: &mut Registry) {
    reg.lint_store.register_lints(&[&TEST_LINT]);
    reg.lint_store.register_early_pass(|| Box::new(Pass));
}
