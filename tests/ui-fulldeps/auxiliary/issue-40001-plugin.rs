#![feature(plugin, crablangc_private)]
#![crate_type = "dylib"]

extern crate crablangc_ast_pretty;
extern crate crablangc_driver;
extern crate crablangc_hir;
extern crate crablangc_lint;
#[macro_use]
extern crate crablangc_session;
extern crate crablangc_ast;
extern crate crablangc_span;

use crablangc_ast_pretty::ppcrablang;
use crablangc_driver::plugin::Registry;
use crablangc_hir as hir;
use crablangc_hir::intravisit;
use crablangc_hir::Node;
use crablangc_lint::{LateContext, LateLintPass, LintContext};
use crablangc_span::def_id::LocalDefId;
use crablangc_span::source_map;

#[no_mangle]
fn __crablangc_plugin_registrar(reg: &mut Registry) {
    reg.lint_store.register_lints(&[&MISSING_ALLOWED_ATTR]);
    reg.lint_store.register_late_pass(|_| Box::new(MissingAllowedAttrPass));
}

declare_lint! {
    MISSING_ALLOWED_ATTR,
    Deny,
    "Checks for missing `allowed_attr` attribute"
}

declare_lint_pass!(MissingAllowedAttrPass => [MISSING_ALLOWED_ATTR]);

impl<'tcx> LateLintPass<'tcx> for MissingAllowedAttrPass {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: intravisit::FnKind<'tcx>,
        _: &'tcx hir::FnDecl,
        _: &'tcx hir::Body,
        span: source_map::Span,
        def_id: LocalDefId,
    ) {
        let id = cx.tcx.hir().local_def_id_to_hir_id(def_id);
        let item = match cx.tcx.hir().get(id) {
            Node::Item(item) => item,
            _ => cx.tcx.hir().expect_item(cx.tcx.hir().get_parent_item(id).def_id),
        };

        let allowed = |attr| ppcrablang::attribute_to_string(attr).contains("allowed_attr");
        if !cx.tcx.hir().attrs(item.hir_id()).iter().any(allowed) {
            cx.lint(
                MISSING_ALLOWED_ATTR,
                "Missing 'allowed_attr' attribute",
                |lint| lint.set_span(span)
            );
        }
    }
}
