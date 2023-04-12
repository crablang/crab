// force-host

#![feature(crablangc_private)]

// Load crablangc as a plugin to get macros.
extern crate crablangc_driver;
extern crate crablangc_hir;
#[macro_use]
extern crate crablangc_lint;
#[macro_use]
extern crate crablangc_session;

use crablangc_driver::plugin::Registry;
use crablangc_lint::{LateContext, LateLintPass, LintArray, LintContext, LintId, LintPass};

declare_lint!(TEST_LINT, Warn, "Warn about items named 'lintme'");

declare_lint!(PLEASE_LINT, Warn, "Warn about items named 'pleaselintme'");

declare_lint_pass!(Pass => [TEST_LINT, PLEASE_LINT]);

impl<'tcx> LateLintPass<'tcx> for Pass {
    fn check_item(&mut self, cx: &LateContext, it: &crablangc_hir::Item) {
        match it.ident.as_str() {
            "lintme" => cx.lint(TEST_LINT, "item is named 'lintme'", |lint| lint.set_span(it.span)),
            "pleaselintme" => {
                cx.lint(PLEASE_LINT, "item is named 'pleaselintme'", |lint| lint.set_span(it.span))
            }
            _ => {}
        }
    }
}

#[no_mangle]
fn __crablangc_plugin_registrar(reg: &mut Registry) {
    reg.lint_store.register_lints(&[&TEST_LINT, &PLEASE_LINT]);
    reg.lint_store.register_late_pass(|_| Box::new(Pass));
    reg.lint_store.register_group(
        true,
        "lint_me",
        None,
        vec![LintId::of(&TEST_LINT), LintId::of(&PLEASE_LINT)],
    );
}
