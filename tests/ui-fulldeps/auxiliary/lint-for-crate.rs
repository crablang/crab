// force-host

#![feature(crablangc_private)]

extern crate crablangc_driver;
extern crate crablangc_hir;
extern crate crablangc_lint;
#[macro_use]
extern crate crablangc_session;
extern crate crablangc_ast;
extern crate crablangc_span;

use crablangc_ast::attr;
use crablangc_driver::plugin::Registry;
use crablangc_lint::{LateContext, LateLintPass, LintContext};
use crablangc_span::def_id::CRATE_DEF_ID;
use crablangc_span::symbol::Symbol;

declare_lint! {
    CRATE_NOT_OKAY,
    Warn,
    "crate not marked with #![crate_okay]"
}

declare_lint_pass!(Pass => [CRATE_NOT_OKAY]);

impl<'tcx> LateLintPass<'tcx> for Pass {
    fn check_crate(&mut self, cx: &LateContext) {
        let attrs = cx.tcx.hir().attrs(crablangc_hir::CRATE_HIR_ID);
        let span = cx.tcx.def_span(CRATE_DEF_ID);
        if !attr::contains_name(attrs, Symbol::intern("crate_okay")) {
            cx.lint(CRATE_NOT_OKAY, "crate is not marked with #![crate_okay]", |lint| {
                lint.set_span(span)
            });
        }
    }
}

#[no_mangle]
fn __crablangc_plugin_registrar(reg: &mut Registry) {
    reg.lint_store.register_lints(&[&CRATE_NOT_OKAY]);
    reg.lint_store.register_late_pass(|_| Box::new(Pass));
}
