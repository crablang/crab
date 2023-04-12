use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::macros::root_macro_call_first_node;
use clippy_utils::visitors::is_local_used;
use if_chain::if_chain;
use crablangc_hir::{Body, Impl, ImplItem, ImplItemKind, ItemKind};
use crablangc_lint::{LateContext, LateLintPass};
use crablangc_session::{declare_tool_lint, impl_lint_pass};
use std::ops::ControlFlow;

declare_clippy_lint! {
    /// ### What it does
    /// Checks methods that contain a `self` argument but don't use it
    ///
    /// ### Why is this bad?
    /// It may be clearer to define the method as an associated function instead
    /// of an instance method if it doesn't require `self`.
    ///
    /// ### Example
    /// ```crablang,ignore
    /// struct A;
    /// impl A {
    ///     fn method(&self) {}
    /// }
    /// ```
    ///
    /// Could be written:
    ///
    /// ```crablang,ignore
    /// struct A;
    /// impl A {
    ///     fn method() {}
    /// }
    /// ```
    #[clippy::version = "1.40.0"]
    pub UNUSED_SELF,
    pedantic,
    "methods that contain a `self` argument but don't use it"
}

pub struct UnusedSelf {
    avoid_breaking_exported_api: bool,
}

impl_lint_pass!(UnusedSelf => [UNUSED_SELF]);

impl UnusedSelf {
    pub fn new(avoid_breaking_exported_api: bool) -> Self {
        Self {
            avoid_breaking_exported_api,
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for UnusedSelf {
    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &ImplItem<'_>) {
        if impl_item.span.from_expansion() {
            return;
        }
        let parent = cx.tcx.hir().get_parent_item(impl_item.hir_id()).def_id;
        let parent_item = cx.tcx.hir().expect_item(parent);
        let assoc_item = cx.tcx.associated_item(impl_item.owner_id);
        let contains_todo = |cx, body: &'_ Body<'_>| -> bool {
            clippy_utils::visitors::for_each_expr(body.value, |e| {
                if let Some(macro_call) = root_macro_call_first_node(cx, e) {
                    if cx.tcx.item_name(macro_call.def_id).as_str() == "todo" {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                } else {
                    ControlFlow::Continue(())
                }
            })
            .is_some()
        };
        if_chain! {
            if let ItemKind::Impl(Impl { of_trait: None, .. }) = parent_item.kind;
            if assoc_item.fn_has_self_parameter;
            if let ImplItemKind::Fn(.., body_id) = &impl_item.kind;
            if !cx.effective_visibilities.is_exported(impl_item.owner_id.def_id) || !self.avoid_breaking_exported_api;
            let body = cx.tcx.hir().body(*body_id);
            if let [self_param, ..] = body.params;
            if !is_local_used(cx, body, self_param.pat.hir_id);
            if !contains_todo(cx, body);
            then {
                span_lint_and_help(
                    cx,
                    UNUSED_SELF,
                    self_param.span,
                    "unused `self` argument",
                    None,
                    "consider refactoring to an associated function",
                );
            }
        }
    }
}
