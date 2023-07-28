use crate::FnCtxt;
use rustc_ast::util::parser::PREC_POSTFIX;
use rustc_errors::MultiSpan;
use rustc_errors::{Applicability, Diagnostic, DiagnosticBuilder, ErrorGuaranteed};
use rustc_hir as hir;
use rustc_hir::def::{CtorKind, Res};
use rustc_hir::intravisit::Visitor;
use rustc_hir::lang_items::LangItem;
use rustc_hir::{is_range_literal, Node};
use rustc_infer::infer::{DefineOpaqueTypes, InferOk};
use rustc_middle::lint::in_external_macro;
use rustc_middle::middle::stability::EvalResult;
use rustc_middle::ty::adjustment::AllowTwoPhase;
use rustc_middle::ty::error::{ExpectedFound, TypeError};
use rustc_middle::ty::fold::BottomUpFolder;
use rustc_middle::ty::print::with_no_trimmed_paths;
use rustc_middle::ty::{self, Article, AssocItem, Ty, TypeAndMut, TypeFoldable};
use rustc_span::symbol::sym;
use rustc_span::{BytePos, Span, DUMMY_SP};
use rustc_trait_selection::infer::InferCtxtExt as _;
use rustc_trait_selection::traits::ObligationCause;

use super::method::probe;

use std::cmp::min;
use std::iter;

impl<'a, 'tcx> FnCtxt<'a, 'tcx> {
    pub fn emit_type_mismatch_suggestions(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'tcx>,
        expr_ty: Ty<'tcx>,
        expected: Ty<'tcx>,
        expected_ty_expr: Option<&'tcx hir::Expr<'tcx>>,
        error: Option<TypeError<'tcx>>,
    ) {
        if expr_ty == expected {
            return;
        }

        self.annotate_alternative_method_deref(err, expr, error);

        // Use `||` to give these suggestions a precedence
        let suggested = self.suggest_missing_parentheses(err, expr)
            || self.suggest_remove_last_method_call(err, expr, expected)
            || self.suggest_associated_const(err, expr, expected)
            || self.suggest_deref_ref_or_into(err, expr, expected, expr_ty, expected_ty_expr)
            || self.suggest_option_to_bool(err, expr, expr_ty, expected)
            || self.suggest_compatible_variants(err, expr, expected, expr_ty)
            || self.suggest_non_zero_new_unwrap(err, expr, expected, expr_ty)
            || self.suggest_calling_boxed_future_when_appropriate(err, expr, expected, expr_ty)
            || self.suggest_no_capture_closure(err, expected, expr_ty)
            || self.suggest_boxing_when_appropriate(err, expr.span, expr.hir_id, expected, expr_ty)
            || self.suggest_block_to_brackets_peeling_refs(err, expr, expr_ty, expected)
            || self.suggest_copied_cloned_or_as_ref(err, expr, expr_ty, expected, expected_ty_expr)
            || self.suggest_clone_for_ref(err, expr, expr_ty, expected)
            || self.suggest_into(err, expr, expr_ty, expected)
            || self.suggest_floating_point_literal(err, expr, expected)
            || self.suggest_null_ptr_for_literal_zero_given_to_ptr_arg(err, expr, expected)
            || self.suggest_coercing_result_via_try_operator(err, expr, expected, expr_ty);

        if !suggested {
            self.note_source_of_type_mismatch_constraint(
                err,
                expr,
                TypeMismatchSource::Ty(expected),
            );
        }
    }

    pub fn emit_coerce_suggestions(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'tcx>,
        expr_ty: Ty<'tcx>,
        expected: Ty<'tcx>,
        expected_ty_expr: Option<&'tcx hir::Expr<'tcx>>,
        error: Option<TypeError<'tcx>>,
    ) {
        if expr_ty == expected {
            return;
        }

        self.annotate_expected_due_to_let_ty(err, expr, error);

        if self.is_destruct_assignment_desugaring(expr) {
            return;
        }
        self.emit_type_mismatch_suggestions(err, expr, expr_ty, expected, expected_ty_expr, error);
        self.note_type_is_not_clone(err, expected, expr_ty, expr);
        self.note_internal_mutation_in_method(err, expr, Some(expected), expr_ty);
        self.suggest_method_call_on_range_literal(err, expr, expr_ty, expected);
        self.suggest_return_binding_for_missing_tail_expr(err, expr, expr_ty, expected);
        self.note_wrong_return_ty_due_to_generic_arg(err, expr, expr_ty);
    }

    /// Really hacky heuristic to remap an `assert_eq!` error to the user
    /// expressions provided to the macro.
    fn adjust_expr_for_assert_eq_macro(
        &self,
        found_expr: &mut &'tcx hir::Expr<'tcx>,
        expected_expr: &mut Option<&'tcx hir::Expr<'tcx>>,
    ) {
        let Some(expected_expr) = expected_expr else {
            return;
        };

        if !found_expr.span.eq_ctxt(expected_expr.span) {
            return;
        }

        if !found_expr
            .span
            .ctxt()
            .outer_expn_data()
            .macro_def_id
            .is_some_and(|def_id| self.tcx.is_diagnostic_item(sym::assert_eq_macro, def_id))
        {
            return;
        }

        let hir::ExprKind::Unary(
            hir::UnOp::Deref,
            hir::Expr { kind: hir::ExprKind::Path(found_path), .. },
        ) = found_expr.kind
        else {
            return;
        };
        let hir::ExprKind::Unary(
            hir::UnOp::Deref,
            hir::Expr { kind: hir::ExprKind::Path(expected_path), .. },
        ) = expected_expr.kind
        else {
            return;
        };

        for (path, name, idx, var) in [
            (expected_path, "left_val", 0, expected_expr),
            (found_path, "right_val", 1, found_expr),
        ] {
            if let hir::QPath::Resolved(_, path) = path
                && let [segment] = path.segments
                && segment.ident.name.as_str() == name
                && let Res::Local(hir_id) = path.res
                && let Some((_, hir::Node::Expr(match_expr))) = self.tcx.hir().parent_iter(hir_id).nth(2)
                && let hir::ExprKind::Match(scrutinee, _, _) = match_expr.kind
                && let hir::ExprKind::Tup(exprs) = scrutinee.kind
                && let hir::ExprKind::AddrOf(_, _, macro_arg) = exprs[idx].kind
            {
                *var = macro_arg;
            }
        }
    }

    /// Requires that the two types unify, and prints an error message if
    /// they don't.
    pub fn demand_suptype(&self, sp: Span, expected: Ty<'tcx>, actual: Ty<'tcx>) {
        if let Some(mut e) = self.demand_suptype_diag(sp, expected, actual) {
            e.emit();
        }
    }

    pub fn demand_suptype_diag(
        &self,
        sp: Span,
        expected: Ty<'tcx>,
        actual: Ty<'tcx>,
    ) -> Option<DiagnosticBuilder<'tcx, ErrorGuaranteed>> {
        self.demand_suptype_with_origin(&self.misc(sp), expected, actual)
    }

    #[instrument(skip(self), level = "debug")]
    pub fn demand_suptype_with_origin(
        &self,
        cause: &ObligationCause<'tcx>,
        expected: Ty<'tcx>,
        actual: Ty<'tcx>,
    ) -> Option<DiagnosticBuilder<'tcx, ErrorGuaranteed>> {
        match self.at(cause, self.param_env).sup(DefineOpaqueTypes::Yes, expected, actual) {
            Ok(InferOk { obligations, value: () }) => {
                self.register_predicates(obligations);
                None
            }
            Err(e) => Some(self.err_ctxt().report_mismatched_types(&cause, expected, actual, e)),
        }
    }

    pub fn demand_eqtype(&self, sp: Span, expected: Ty<'tcx>, actual: Ty<'tcx>) {
        if let Some(mut err) = self.demand_eqtype_diag(sp, expected, actual) {
            err.emit();
        }
    }

    pub fn demand_eqtype_diag(
        &self,
        sp: Span,
        expected: Ty<'tcx>,
        actual: Ty<'tcx>,
    ) -> Option<DiagnosticBuilder<'tcx, ErrorGuaranteed>> {
        self.demand_eqtype_with_origin(&self.misc(sp), expected, actual)
    }

    pub fn demand_eqtype_with_origin(
        &self,
        cause: &ObligationCause<'tcx>,
        expected: Ty<'tcx>,
        actual: Ty<'tcx>,
    ) -> Option<DiagnosticBuilder<'tcx, ErrorGuaranteed>> {
        match self.at(cause, self.param_env).eq(DefineOpaqueTypes::Yes, expected, actual) {
            Ok(InferOk { obligations, value: () }) => {
                self.register_predicates(obligations);
                None
            }
            Err(e) => Some(self.err_ctxt().report_mismatched_types(cause, expected, actual, e)),
        }
    }

    pub fn demand_coerce(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        checked_ty: Ty<'tcx>,
        expected: Ty<'tcx>,
        expected_ty_expr: Option<&'tcx hir::Expr<'tcx>>,
        allow_two_phase: AllowTwoPhase,
    ) -> Ty<'tcx> {
        let (ty, err) =
            self.demand_coerce_diag(expr, checked_ty, expected, expected_ty_expr, allow_two_phase);
        if let Some(mut err) = err {
            err.emit();
        }
        ty
    }

    /// Checks that the type of `expr` can be coerced to `expected`.
    ///
    /// N.B., this code relies on `self.diverges` to be accurate. In particular, assignments to `!`
    /// will be permitted if the diverges flag is currently "always".
    #[instrument(level = "debug", skip(self, expr, expected_ty_expr, allow_two_phase))]
    pub fn demand_coerce_diag(
        &self,
        mut expr: &'tcx hir::Expr<'tcx>,
        checked_ty: Ty<'tcx>,
        expected: Ty<'tcx>,
        mut expected_ty_expr: Option<&'tcx hir::Expr<'tcx>>,
        allow_two_phase: AllowTwoPhase,
    ) -> (Ty<'tcx>, Option<DiagnosticBuilder<'tcx, ErrorGuaranteed>>) {
        let expected = self.resolve_vars_with_obligations(expected);

        let e = match self.try_coerce(expr, checked_ty, expected, allow_two_phase, None) {
            Ok(ty) => return (ty, None),
            Err(e) => e,
        };

        self.adjust_expr_for_assert_eq_macro(&mut expr, &mut expected_ty_expr);

        self.set_tainted_by_errors(self.tcx.sess.delay_span_bug(
            expr.span,
            "`TypeError` when attempting coercion but no error emitted",
        ));
        let expr = expr.peel_drop_temps();
        let cause = self.misc(expr.span);
        let expr_ty = self.resolve_vars_with_obligations(checked_ty);
        let mut err = self.err_ctxt().report_mismatched_types(&cause, expected, expr_ty, e);

        let is_insufficiently_polymorphic =
            matches!(e, TypeError::RegionsInsufficientlyPolymorphic(..));

        // FIXME(#73154): For now, we do leak check when coercing function
        // pointers in typeck, instead of only during borrowck. This can lead
        // to these `RegionsInsufficientlyPolymorphic` errors that aren't helpful.
        if !is_insufficiently_polymorphic {
            self.emit_coerce_suggestions(
                &mut err,
                expr,
                expr_ty,
                expected,
                expected_ty_expr,
                Some(e),
            );
        }

        (expected, Some(err))
    }

    /// Notes the point at which a variable is constrained to some type incompatible
    /// with some expectation given by `source`.
    pub fn note_source_of_type_mismatch_constraint(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        source: TypeMismatchSource<'tcx>,
    ) -> bool {
        let hir = self.tcx.hir();

        let hir::ExprKind::Path(hir::QPath::Resolved(None, p)) = expr.kind else {
            return false;
        };
        let [hir::PathSegment { ident, args: None, .. }] = p.segments else {
            return false;
        };
        let hir::def::Res::Local(local_hir_id) = p.res else {
            return false;
        };
        let hir::Node::Pat(pat) = hir.get(local_hir_id) else {
            return false;
        };
        let (init_ty_hir_id, init) = match hir.get_parent(pat.hir_id) {
            hir::Node::Local(hir::Local { ty: Some(ty), init, .. }) => (ty.hir_id, *init),
            hir::Node::Local(hir::Local { init: Some(init), .. }) => (init.hir_id, Some(*init)),
            _ => return false,
        };
        let Some(init_ty) = self.node_ty_opt(init_ty_hir_id) else {
            return false;
        };

        // Locate all the usages of the relevant binding.
        struct FindExprs<'tcx> {
            hir_id: hir::HirId,
            uses: Vec<&'tcx hir::Expr<'tcx>>,
        }
        impl<'tcx> Visitor<'tcx> for FindExprs<'tcx> {
            fn visit_expr(&mut self, ex: &'tcx hir::Expr<'tcx>) {
                if let hir::ExprKind::Path(hir::QPath::Resolved(None, path)) = ex.kind
                    && let hir::def::Res::Local(hir_id) = path.res
                    && hir_id == self.hir_id
                {
                    self.uses.push(ex);
                }
                hir::intravisit::walk_expr(self, ex);
            }
        }

        let mut expr_finder = FindExprs { hir_id: local_hir_id, uses: init.into_iter().collect() };
        let body =
            hir.body(hir.maybe_body_owned_by(self.body_id).expect("expected item to have body"));
        expr_finder.visit_expr(body.value);

        use rustc_infer::infer::type_variable::*;
        use rustc_middle::infer::unify_key::*;
        // Replaces all of the variables in the given type with a fresh inference variable.
        let mut fudger = BottomUpFolder {
            tcx: self.tcx,
            ty_op: |ty| {
                if let ty::Infer(infer) = ty.kind() {
                    match infer {
                        ty::InferTy::TyVar(_) => self.next_ty_var(TypeVariableOrigin {
                            kind: TypeVariableOriginKind::MiscVariable,
                            span: DUMMY_SP,
                        }),
                        ty::InferTy::IntVar(_) => self.next_int_var(),
                        ty::InferTy::FloatVar(_) => self.next_float_var(),
                        _ => bug!(),
                    }
                } else {
                    ty
                }
            },
            lt_op: |_| self.tcx.lifetimes.re_erased,
            ct_op: |ct| {
                if let ty::ConstKind::Infer(_) = ct.kind() {
                    self.next_const_var(
                        ct.ty(),
                        ConstVariableOrigin {
                            kind: ConstVariableOriginKind::MiscVariable,
                            span: DUMMY_SP,
                        },
                    )
                } else {
                    ct
                }
            },
        };

        let expected_ty = match source {
            TypeMismatchSource::Ty(expected_ty) => expected_ty,
            // Try to deduce what the possible value of `expr` would be if the
            // incompatible arg were compatible. For example, given `Vec<i32>`
            // and `vec.push(1u32)`, we ideally want to deduce that the type of
            // `vec` *should* have been `Vec<u32>`. This will allow us to then
            // run the subsequent code with this expectation, finding out exactly
            // when this type diverged from our expectation.
            TypeMismatchSource::Arg { call_expr, incompatible_arg: idx } => {
                let hir::ExprKind::MethodCall(segment, _, args, _) = call_expr.kind else {
                    return false;
                };
                let Some(arg_ty) = self.node_ty_opt(args[idx].hir_id) else {
                    return false;
                };
                let possible_rcvr_ty = expr_finder.uses.iter().find_map(|binding| {
                    let possible_rcvr_ty = self.node_ty_opt(binding.hir_id)?;
                    // Fudge the receiver, so we can do new inference on it.
                    let possible_rcvr_ty = possible_rcvr_ty.fold_with(&mut fudger);
                    let method = self
                        .lookup_method_for_diagnostic(
                            possible_rcvr_ty,
                            segment,
                            DUMMY_SP,
                            call_expr,
                            binding,
                        )
                        .ok()?;
                    // Unify the method signature with our incompatible arg, to
                    // do inference in the *opposite* direction and to find out
                    // what our ideal rcvr ty would look like.
                    let _ = self
                        .at(&ObligationCause::dummy(), self.param_env)
                        .eq(DefineOpaqueTypes::No, method.sig.inputs()[idx + 1], arg_ty)
                        .ok()?;
                    self.select_obligations_where_possible(|errs| {
                        // Yeet the errors, we're already reporting errors.
                        errs.clear();
                    });
                    Some(self.resolve_vars_if_possible(possible_rcvr_ty))
                });
                if let Some(rcvr_ty) = possible_rcvr_ty {
                    rcvr_ty
                } else {
                    return false;
                }
            }
        };

        // If our expected_ty does not equal init_ty, then it *began* as incompatible.
        // No need to note in this case...
        if !self.can_eq(self.param_env, expected_ty, init_ty.fold_with(&mut fudger)) {
            return false;
        }

        for window in expr_finder.uses.windows(2) {
            // Bindings always update their recorded type after the fact, so we
            // need to look at the *following* usage's type to see when the
            // binding became incompatible.
            let [binding, next_usage] = *window else {
                continue;
            };

            // Don't go past the binding (always gonna be a nonsense label if so)
            if binding.hir_id == expr.hir_id {
                break;
            }

            let Some(next_use_ty) = self.node_ty_opt(next_usage.hir_id) else {
                continue;
            };

            // If the type is not constrained in a way making it not possible to
            // equate with `expected_ty` by this point, skip.
            if self.can_eq(self.param_env, expected_ty, next_use_ty.fold_with(&mut fudger)) {
                continue;
            }

            if let hir::Node::Expr(parent_expr) = hir.get_parent(binding.hir_id)
                && let hir::ExprKind::MethodCall(segment, rcvr, args, _) = parent_expr.kind
                && rcvr.hir_id == binding.hir_id
            {
                // If our binding became incompatible while it was a receiver
                // to a method call, we may be able to make a better guess to
                // the source of a type mismatch.
                let Some(rcvr_ty) = self.node_ty_opt(rcvr.hir_id) else { continue; };
                let rcvr_ty = rcvr_ty.fold_with(&mut fudger);
                let Ok(method) =
                    self.lookup_method_for_diagnostic(rcvr_ty, segment, DUMMY_SP, parent_expr, rcvr)
                else {
                    continue;
                };

                let ideal_rcvr_ty = rcvr_ty.fold_with(&mut fudger);
                let ideal_method = self
                    .lookup_method_for_diagnostic(ideal_rcvr_ty, segment, DUMMY_SP, parent_expr, rcvr)
                    .ok()
                    .and_then(|method| {
                        let _ = self.at(&ObligationCause::dummy(), self.param_env)
                            .eq(DefineOpaqueTypes::No, ideal_rcvr_ty, expected_ty)
                            .ok()?;
                        Some(method)
                    });

                // Find what argument caused our rcvr to become incompatible
                // with the expected ty.
                for (idx, (expected_arg_ty, arg_expr)) in
                    std::iter::zip(&method.sig.inputs()[1..], args).enumerate()
                {
                    let Some(arg_ty) = self.node_ty_opt(arg_expr.hir_id) else { continue; };
                    let arg_ty = arg_ty.fold_with(&mut fudger);
                    let _ = self.try_coerce(
                        arg_expr,
                        arg_ty,
                        *expected_arg_ty,
                        AllowTwoPhase::No,
                        None,
                    );
                    self.select_obligations_where_possible(|errs| {
                        // Yeet the errors, we're already reporting errors.
                        errs.clear();
                    });
                    // If our rcvr, after inference due to unifying the signature
                    // with the expected argument type, is still compatible with
                    // the rcvr, then it must've not been the source of blame.
                    if self.can_eq(self.param_env, rcvr_ty, expected_ty) {
                        continue;
                    }
                    err.span_label(arg_expr.span, format!("this argument has type `{arg_ty}`..."));
                    err.span_label(
                        binding.span,
                        format!("... which causes `{ident}` to have type `{next_use_ty}`"),
                    );
                    // Using our "ideal" method signature, suggest a fix to this
                    // blame arg, if possible. Don't do this if we're coming from
                    // arg mismatch code, because we'll possibly suggest a mutually
                    // incompatible fix at the original mismatch site.
                    if matches!(source, TypeMismatchSource::Ty(_))
                        && let Some(ideal_method) = ideal_method
                    {
                        self.emit_type_mismatch_suggestions(
                            err,
                            arg_expr,
                            arg_ty,
                            self.resolve_vars_if_possible(ideal_method.sig.inputs()[idx + 1]),
                            None,
                            None,
                        );
                    }
                    return true;
                }
            }
            err.span_label(
                binding.span,
                format!("here the type of `{ident}` is inferred to be `{next_use_ty}`"),
            );
            return true;
        }

        // We must've not found something that constrained the expr.
        false
    }

    fn annotate_expected_due_to_let_ty(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        error: Option<TypeError<'tcx>>,
    ) {
        let parent = self.tcx.hir().parent_id(expr.hir_id);
        match (self.tcx.hir().find(parent), error) {
            (Some(hir::Node::Local(hir::Local { ty: Some(ty), init: Some(init), .. })), _)
                if init.hir_id == expr.hir_id =>
            {
                // Point at `let` assignment type.
                err.span_label(ty.span, "expected due to this");
            }
            (
                Some(hir::Node::Expr(hir::Expr {
                    kind: hir::ExprKind::Assign(lhs, rhs, _), ..
                })),
                Some(TypeError::Sorts(ExpectedFound { expected, .. })),
            ) if rhs.hir_id == expr.hir_id && !expected.is_closure() => {
                // We ignore closures explicitly because we already point at them elsewhere.
                // Point at the assigned-to binding.
                let mut primary_span = lhs.span;
                let mut secondary_span = lhs.span;
                let mut post_message = "";
                match lhs.kind {
                    hir::ExprKind::Path(hir::QPath::Resolved(
                        None,
                        hir::Path {
                            res:
                                hir::def::Res::Def(
                                    hir::def::DefKind::Static(_) | hir::def::DefKind::Const,
                                    def_id,
                                ),
                            ..
                        },
                    )) => {
                        if let Some(hir::Node::Item(hir::Item {
                            ident,
                            kind: hir::ItemKind::Static(ty, ..) | hir::ItemKind::Const(ty, ..),
                            ..
                        })) = self.tcx.hir().get_if_local(*def_id)
                        {
                            primary_span = ty.span;
                            secondary_span = ident.span;
                            post_message = " type";
                        }
                    }
                    hir::ExprKind::Path(hir::QPath::Resolved(
                        None,
                        hir::Path { res: hir::def::Res::Local(hir_id), .. },
                    )) => {
                        if let Some(hir::Node::Pat(pat)) = self.tcx.hir().find(*hir_id) {
                            primary_span = pat.span;
                            secondary_span = pat.span;
                            match self.tcx.hir().find_parent(pat.hir_id) {
                                Some(hir::Node::Local(hir::Local { ty: Some(ty), .. })) => {
                                    primary_span = ty.span;
                                    post_message = " type";
                                }
                                Some(hir::Node::Local(hir::Local { init: Some(init), .. })) => {
                                    primary_span = init.span;
                                    post_message = " value";
                                }
                                Some(hir::Node::Param(hir::Param { ty_span, .. })) => {
                                    primary_span = *ty_span;
                                    post_message = " parameter type";
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }

                if primary_span != secondary_span
                    && self
                        .tcx
                        .sess
                        .source_map()
                        .is_multiline(secondary_span.shrink_to_hi().until(primary_span))
                {
                    // We are pointing at the binding's type or initializer value, but it's pattern
                    // is in a different line, so we point at both.
                    err.span_label(secondary_span, "expected due to the type of this binding");
                    err.span_label(primary_span, format!("expected due to this{post_message}"));
                } else if post_message.is_empty() {
                    // We are pointing at either the assignment lhs or the binding def pattern.
                    err.span_label(primary_span, "expected due to the type of this binding");
                } else {
                    // We are pointing at the binding's type or initializer value.
                    err.span_label(primary_span, format!("expected due to this{post_message}"));
                }

                if !lhs.is_syntactic_place_expr() {
                    // We already emitted E0070 "invalid left-hand side of assignment", so we
                    // silence this.
                    err.downgrade_to_delayed_bug();
                }
            }
            (
                Some(hir::Node::Expr(hir::Expr {
                    kind: hir::ExprKind::Binary(_, lhs, rhs), ..
                })),
                Some(TypeError::Sorts(ExpectedFound { expected, .. })),
            ) if rhs.hir_id == expr.hir_id
                && self.typeck_results.borrow().expr_ty_adjusted_opt(lhs) == Some(expected) =>
            {
                err.span_label(lhs.span, format!("expected because this is `{expected}`"));
            }
            _ => {}
        }
    }

    fn annotate_alternative_method_deref(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        error: Option<TypeError<'tcx>>,
    ) {
        let parent = self.tcx.hir().parent_id(expr.hir_id);
        let Some(TypeError::Sorts(ExpectedFound { expected, .. })) = error else {
            return;
        };
        let Some(hir::Node::Expr(hir::Expr { kind: hir::ExprKind::Assign(lhs, rhs, _), .. })) =
            self.tcx.hir().find(parent)
        else {
            return;
        };
        if rhs.hir_id != expr.hir_id || expected.is_closure() {
            return;
        }
        let hir::ExprKind::Unary(hir::UnOp::Deref, deref) = lhs.kind else {
            return;
        };
        let hir::ExprKind::MethodCall(path, base, args, _) = deref.kind else {
            return;
        };
        let Some(self_ty) = self.typeck_results.borrow().expr_ty_adjusted_opt(base) else {
            return;
        };

        let Ok(pick) = self.lookup_probe_for_diagnostic(
            path.ident,
            self_ty,
            deref,
            probe::ProbeScope::TraitsInScope,
            None,
        ) else {
            return;
        };
        let in_scope_methods = self.probe_for_name_many(
            probe::Mode::MethodCall,
            path.ident,
            Some(expected),
            probe::IsSuggestion(true),
            self_ty,
            deref.hir_id,
            probe::ProbeScope::TraitsInScope,
        );
        let other_methods_in_scope: Vec<_> =
            in_scope_methods.iter().filter(|c| c.item.def_id != pick.item.def_id).collect();

        let all_methods = self.probe_for_name_many(
            probe::Mode::MethodCall,
            path.ident,
            Some(expected),
            probe::IsSuggestion(true),
            self_ty,
            deref.hir_id,
            probe::ProbeScope::AllTraits,
        );
        let suggestions: Vec<_> = all_methods
            .into_iter()
            .filter(|c| c.item.def_id != pick.item.def_id)
            .map(|c| {
                let m = c.item;
                let generic_args = ty::GenericArgs::for_item(self.tcx, m.def_id, |param, _| {
                    self.var_for_def(deref.span, param)
                });
                let mutability =
                    match self.tcx.fn_sig(m.def_id).skip_binder().input(0).skip_binder().kind() {
                        ty::Ref(_, _, hir::Mutability::Mut) => "&mut ",
                        ty::Ref(_, _, _) => "&",
                        _ => "",
                    };
                vec![
                    (
                        deref.span.until(base.span),
                        format!(
                            "{}({}",
                            with_no_trimmed_paths!(
                                self.tcx.def_path_str_with_args(m.def_id, generic_args,)
                            ),
                            mutability,
                        ),
                    ),
                    match &args[..] {
                        [] => (base.span.shrink_to_hi().with_hi(deref.span.hi()), ")".to_string()),
                        [first, ..] => (base.span.between(first.span), ", ".to_string()),
                    },
                ]
            })
            .collect();
        if suggestions.is_empty() {
            return;
        }
        let mut path_span: MultiSpan = path.ident.span.into();
        path_span.push_span_label(
            path.ident.span,
            with_no_trimmed_paths!(format!(
                "refers to `{}`",
                self.tcx.def_path_str(pick.item.def_id),
            )),
        );
        let container_id = pick.item.container_id(self.tcx);
        let container = with_no_trimmed_paths!(self.tcx.def_path_str(container_id));
        for def_id in pick.import_ids {
            let hir_id = self.tcx.hir().local_def_id_to_hir_id(def_id);
            path_span.push_span_label(
                self.tcx.hir().span(hir_id),
                format!("`{container}` imported here"),
            );
        }
        let tail = with_no_trimmed_paths!(match &other_methods_in_scope[..] {
            [] => return,
            [candidate] => format!(
                "the method of the same name on {} `{}`",
                match candidate.kind {
                    probe::CandidateKind::InherentImplCandidate(..) => "the inherent impl for",
                    _ => "trait",
                },
                self.tcx.def_path_str(candidate.item.container_id(self.tcx))
            ),
            [.., last] if other_methods_in_scope.len() < 5 => {
                format!(
                    "the methods of the same name on {} and `{}`",
                    other_methods_in_scope[..other_methods_in_scope.len() - 1]
                        .iter()
                        .map(|c| format!(
                            "`{}`",
                            self.tcx.def_path_str(c.item.container_id(self.tcx))
                        ))
                        .collect::<Vec<String>>()
                        .join(", "),
                    self.tcx.def_path_str(last.item.container_id(self.tcx))
                )
            }
            _ => format!(
                "the methods of the same name on {} other traits",
                other_methods_in_scope.len()
            ),
        });
        err.span_note(
            path_span,
            format!(
                "the `{}` call is resolved to the method in `{container}`, shadowing {tail}",
                path.ident,
            ),
        );
        if suggestions.len() > other_methods_in_scope.len() {
            err.note(format!(
                "additionally, there are {} other available methods that aren't in scope",
                suggestions.len() - other_methods_in_scope.len()
            ));
        }
        err.multipart_suggestions(
            format!(
                "you might have meant to call {}; you can use the fully-qualified path to call {} \
                 explicitly",
                if suggestions.len() == 1 {
                    "the other method"
                } else {
                    "one of the other methods"
                },
                if suggestions.len() == 1 { "it" } else { "one of them" },
            ),
            suggestions,
            Applicability::MaybeIncorrect,
        );
    }

    pub(crate) fn suggest_coercing_result_via_try_operator(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'tcx>,
        expected: Ty<'tcx>,
        found: Ty<'tcx>,
    ) -> bool {
        let ty::Adt(e, args_e) = expected.kind() else {
            return false;
        };
        let ty::Adt(f, args_f) = found.kind() else {
            return false;
        };
        if e.did() != f.did() {
            return false;
        }
        if Some(e.did()) != self.tcx.get_diagnostic_item(sym::Result) {
            return false;
        }
        let map = self.tcx.hir();
        if let Some(hir::Node::Expr(expr)) = map.find_parent(expr.hir_id)
            && let hir::ExprKind::Ret(_) = expr.kind
        {
            // `return foo;`
        } else if map.get_return_block(expr.hir_id).is_some() {
            // Function's tail expression.
        } else {
            return false;
        }
        let e = args_e.type_at(1);
        let f = args_f.type_at(1);
        if self
            .infcx
            .type_implements_trait(
                self.tcx.get_diagnostic_item(sym::Into).unwrap(),
                [f, e],
                self.param_env,
            )
            .must_apply_modulo_regions()
        {
            err.multipart_suggestion(
                "use `?` to coerce and return an appropriate `Err`, and wrap the resulting value \
                 in `Ok` so the expression remains of type `Result`",
                vec![
                    (expr.span.shrink_to_lo(), "Ok(".to_string()),
                    (expr.span.shrink_to_hi(), "?)".to_string()),
                ],
                Applicability::MaybeIncorrect,
            );
            return true;
        }
        false
    }

    /// If the expected type is an enum (Issue #55250) with any variants whose
    /// sole field is of the found type, suggest such variants. (Issue #42764)
    fn suggest_compatible_variants(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        expected: Ty<'tcx>,
        expr_ty: Ty<'tcx>,
    ) -> bool {
        if let ty::Adt(expected_adt, args) = expected.kind() {
            if let hir::ExprKind::Field(base, ident) = expr.kind {
                let base_ty = self.typeck_results.borrow().expr_ty(base);
                if self.can_eq(self.param_env, base_ty, expected)
                    && let Some(base_span) = base.span.find_ancestor_inside(expr.span)
                {
                    err.span_suggestion_verbose(
                        expr.span.with_lo(base_span.hi()),
                        format!("consider removing the tuple struct field `{ident}`"),
                        "",
                        Applicability::MaybeIncorrect,
                    );
                    return true;
                }
            }

            // If the expression is of type () and it's the return expression of a block,
            // we suggest adding a separate return expression instead.
            // (To avoid things like suggesting `Ok(while .. { .. })`.)
            if expr_ty.is_unit() {
                let mut id = expr.hir_id;
                let mut parent;

                // Unroll desugaring, to make sure this works for `for` loops etc.
                loop {
                    parent = self.tcx.hir().parent_id(id);
                    if let Some(parent_span) = self.tcx.hir().opt_span(parent) {
                        if parent_span.find_ancestor_inside(expr.span).is_some() {
                            // The parent node is part of the same span, so is the result of the
                            // same expansion/desugaring and not the 'real' parent node.
                            id = parent;
                            continue;
                        }
                    }
                    break;
                }

                if let Some(hir::Node::Block(&hir::Block {
                    span: block_span, expr: Some(e), ..
                })) = self.tcx.hir().find(parent)
                {
                    if e.hir_id == id {
                        if let Some(span) = expr.span.find_ancestor_inside(block_span) {
                            let return_suggestions = if self
                                .tcx
                                .is_diagnostic_item(sym::Result, expected_adt.did())
                            {
                                vec!["Ok(())"]
                            } else if self.tcx.is_diagnostic_item(sym::Option, expected_adt.did()) {
                                vec!["None", "Some(())"]
                            } else {
                                return false;
                            };
                            if let Some(indent) =
                                self.tcx.sess.source_map().indentation_before(span.shrink_to_lo())
                            {
                                // Add a semicolon, except after `}`.
                                let semicolon =
                                    match self.tcx.sess.source_map().span_to_snippet(span) {
                                        Ok(s) if s.ends_with('}') => "",
                                        _ => ";",
                                    };
                                err.span_suggestions(
                                    span.shrink_to_hi(),
                                    "try adding an expression at the end of the block",
                                    return_suggestions
                                        .into_iter()
                                        .map(|r| format!("{semicolon}\n{indent}{r}")),
                                    Applicability::MaybeIncorrect,
                                );
                            }
                            return true;
                        }
                    }
                }
            }

            let compatible_variants: Vec<(String, _, _, Option<String>)> = expected_adt
                .variants()
                .iter()
                .filter(|variant| {
                    variant.fields.len() == 1
                })
                .filter_map(|variant| {
                    let sole_field = &variant.single_field();

                    let field_is_local = sole_field.did.is_local();
                    let field_is_accessible =
                        sole_field.vis.is_accessible_from(expr.hir_id.owner.def_id, self.tcx)
                        // Skip suggestions for unstable public fields (for example `Pin::pointer`)
                        && matches!(self.tcx.eval_stability(sole_field.did, None, expr.span, None), EvalResult::Allow | EvalResult::Unmarked);

                    if !field_is_local && !field_is_accessible {
                        return None;
                    }

                    let note_about_variant_field_privacy = (field_is_local && !field_is_accessible)
                        .then(|| " (its field is private, but it's local to this crate and its privacy can be changed)".to_string());

                    let sole_field_ty = sole_field.ty(self.tcx, args);
                    if self.can_coerce(expr_ty, sole_field_ty) {
                        let variant_path =
                            with_no_trimmed_paths!(self.tcx.def_path_str(variant.def_id));
                        // FIXME #56861: DRYer prelude filtering
                        if let Some(path) = variant_path.strip_prefix("std::prelude::")
                            && let Some((_, path)) = path.split_once("::")
                        {
                            return Some((path.to_string(), variant.ctor_kind(), sole_field.name, note_about_variant_field_privacy));
                        }
                        Some((variant_path, variant.ctor_kind(), sole_field.name, note_about_variant_field_privacy))
                    } else {
                        None
                    }
                })
                .collect();

            let suggestions_for = |variant: &_, ctor_kind, field_name| {
                let prefix = match self.tcx.hir().maybe_get_struct_pattern_shorthand_field(expr) {
                    Some(ident) => format!("{ident}: "),
                    None => String::new(),
                };

                let (open, close) = match ctor_kind {
                    Some(CtorKind::Fn) => ("(".to_owned(), ")"),
                    None => (format!(" {{ {field_name}: "), " }"),

                    // unit variants don't have fields
                    Some(CtorKind::Const) => unreachable!(),
                };

                // Suggest constructor as deep into the block tree as possible.
                // This fixes https://github.com/rust-lang/rust/issues/101065,
                // and also just helps make the most minimal suggestions.
                let mut expr = expr;
                while let hir::ExprKind::Block(block, _) = &expr.kind
                    && let Some(expr_) = &block.expr
                {
                    expr = expr_
                }

                vec![
                    (expr.span.shrink_to_lo(), format!("{prefix}{variant}{open}")),
                    (expr.span.shrink_to_hi(), close.to_owned()),
                ]
            };

            match &compatible_variants[..] {
                [] => { /* No variants to format */ }
                [(variant, ctor_kind, field_name, note)] => {
                    // Just a single matching variant.
                    err.multipart_suggestion_verbose(
                        format!(
                            "try wrapping the expression in `{variant}`{note}",
                            note = note.as_deref().unwrap_or("")
                        ),
                        suggestions_for(&**variant, *ctor_kind, *field_name),
                        Applicability::MaybeIncorrect,
                    );
                    return true;
                }
                _ => {
                    // More than one matching variant.
                    err.multipart_suggestions(
                        format!(
                            "try wrapping the expression in a variant of `{}`",
                            self.tcx.def_path_str(expected_adt.did())
                        ),
                        compatible_variants.into_iter().map(
                            |(variant, ctor_kind, field_name, _)| {
                                suggestions_for(&variant, ctor_kind, field_name)
                            },
                        ),
                        Applicability::MaybeIncorrect,
                    );
                    return true;
                }
            }
        }

        false
    }

    fn suggest_non_zero_new_unwrap(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        expected: Ty<'tcx>,
        expr_ty: Ty<'tcx>,
    ) -> bool {
        let tcx = self.tcx;
        let (adt, unwrap) = match expected.kind() {
            // In case Option<NonZero*> is wanted, but * is provided, suggest calling new
            ty::Adt(adt, args) if tcx.is_diagnostic_item(sym::Option, adt.did()) => {
                // Unwrap option
                let ty::Adt(adt, _) = args.type_at(0).kind() else {
                    return false;
                };

                (adt, "")
            }
            // In case NonZero* is wanted, but * is provided also add `.unwrap()` to satisfy types
            ty::Adt(adt, _) => (adt, ".unwrap()"),
            _ => return false,
        };

        let map = [
            (sym::NonZeroU8, tcx.types.u8),
            (sym::NonZeroU16, tcx.types.u16),
            (sym::NonZeroU32, tcx.types.u32),
            (sym::NonZeroU64, tcx.types.u64),
            (sym::NonZeroU128, tcx.types.u128),
            (sym::NonZeroI8, tcx.types.i8),
            (sym::NonZeroI16, tcx.types.i16),
            (sym::NonZeroI32, tcx.types.i32),
            (sym::NonZeroI64, tcx.types.i64),
            (sym::NonZeroI128, tcx.types.i128),
        ];

        let Some((s, _)) = map.iter().find(|&&(s, t)| {
            self.tcx.is_diagnostic_item(s, adt.did()) && self.can_coerce(expr_ty, t)
        }) else {
            return false;
        };

        let path = self.tcx.def_path_str(adt.non_enum_variant().def_id);

        err.multipart_suggestion(
            format!("consider calling `{s}::new`"),
            vec![
                (expr.span.shrink_to_lo(), format!("{path}::new(")),
                (expr.span.shrink_to_hi(), format!("){unwrap}")),
            ],
            Applicability::MaybeIncorrect,
        );

        true
    }

    pub fn get_conversion_methods(
        &self,
        span: Span,
        expected: Ty<'tcx>,
        checked_ty: Ty<'tcx>,
        hir_id: hir::HirId,
    ) -> Vec<AssocItem> {
        let methods = self.probe_for_return_type(
            span,
            probe::Mode::MethodCall,
            expected,
            checked_ty,
            hir_id,
            |m| {
                self.has_only_self_parameter(m)
                    && self
                        .tcx
                        // This special internal attribute is used to permit
                        // "identity-like" conversion methods to be suggested here.
                        //
                        // FIXME (#46459 and #46460): ideally
                        // `std::convert::Into::into` and `std::borrow:ToOwned` would
                        // also be `#[rustc_conversion_suggestion]`, if not for
                        // method-probing false-positives and -negatives (respectively).
                        //
                        // FIXME? Other potential candidate methods: `as_ref` and
                        // `as_mut`?
                        .has_attr(m.def_id, sym::rustc_conversion_suggestion)
            },
        );

        methods
    }

    /// This function checks whether the method is not static and does not accept other parameters than `self`.
    fn has_only_self_parameter(&self, method: &AssocItem) -> bool {
        match method.kind {
            ty::AssocKind::Fn => {
                method.fn_has_self_parameter
                    && self.tcx.fn_sig(method.def_id).skip_binder().inputs().skip_binder().len()
                        == 1
            }
            _ => false,
        }
    }

    /// Identify some cases where `as_ref()` would be appropriate and suggest it.
    ///
    /// Given the following code:
    /// ```compile_fail,E0308
    /// struct Foo;
    /// fn takes_ref(_: &Foo) {}
    /// let ref opt = Some(Foo);
    ///
    /// opt.map(|param| takes_ref(param));
    /// ```
    /// Suggest using `opt.as_ref().map(|param| takes_ref(param));` instead.
    ///
    /// It only checks for `Option` and `Result` and won't work with
    /// ```ignore (illustrative)
    /// opt.map(|param| { takes_ref(param) });
    /// ```
    fn can_use_as_ref(&self, expr: &hir::Expr<'_>) -> Option<(Vec<(Span, String)>, &'static str)> {
        let hir::ExprKind::Path(hir::QPath::Resolved(_, ref path)) = expr.kind else {
            return None;
        };

        let hir::def::Res::Local(local_id) = path.res else {
            return None;
        };

        let local_parent = self.tcx.hir().parent_id(local_id);
        let Some(Node::Param(hir::Param { hir_id: param_hir_id, .. })) =
            self.tcx.hir().find(local_parent)
        else {
            return None;
        };

        let param_parent = self.tcx.hir().parent_id(*param_hir_id);
        let Some(Node::Expr(hir::Expr {
            hir_id: expr_hir_id,
            kind: hir::ExprKind::Closure(hir::Closure { fn_decl: closure_fn_decl, .. }),
            ..
        })) = self.tcx.hir().find(param_parent)
        else {
            return None;
        };

        let expr_parent = self.tcx.hir().parent_id(*expr_hir_id);
        let hir = self.tcx.hir().find(expr_parent);
        let closure_params_len = closure_fn_decl.inputs.len();
        let (
            Some(Node::Expr(hir::Expr {
                kind: hir::ExprKind::MethodCall(method_path, receiver, ..),
                ..
            })),
            1,
        ) = (hir, closure_params_len)
        else {
            return None;
        };

        let self_ty = self.typeck_results.borrow().expr_ty(receiver);
        let name = method_path.ident.name;
        let is_as_ref_able = match self_ty.peel_refs().kind() {
            ty::Adt(def, _) => {
                (self.tcx.is_diagnostic_item(sym::Option, def.did())
                    || self.tcx.is_diagnostic_item(sym::Result, def.did()))
                    && (name == sym::map || name == sym::and_then)
            }
            _ => false,
        };
        if is_as_ref_able {
            Some((
                vec![(method_path.ident.span.shrink_to_lo(), "as_ref().".to_string())],
                "consider using `as_ref` instead",
            ))
        } else {
            None
        }
    }

    /// If the given `HirId` corresponds to a block with a trailing expression, return that expression
    pub(crate) fn maybe_get_block_expr(
        &self,
        expr: &hir::Expr<'tcx>,
    ) -> Option<&'tcx hir::Expr<'tcx>> {
        match expr {
            hir::Expr { kind: hir::ExprKind::Block(block, ..), .. } => block.expr,
            _ => None,
        }
    }

    /// Returns whether the given expression is an `else if`.
    pub(crate) fn is_else_if_block(&self, expr: &hir::Expr<'_>) -> bool {
        if let hir::ExprKind::If(..) = expr.kind {
            let parent_id = self.tcx.hir().parent_id(expr.hir_id);
            if let Some(Node::Expr(hir::Expr {
                kind: hir::ExprKind::If(_, _, Some(else_expr)),
                ..
            })) = self.tcx.hir().find(parent_id)
            {
                return else_expr.hir_id == expr.hir_id;
            }
        }
        false
    }

    // Returns whether the given expression is a destruct assignment desugaring.
    // For example, `(a, b) = (1, &2);`
    // Here we try to find the pattern binding of the expression,
    // `default_binding_modes` is false only for destruct assignment desugaring.
    pub(crate) fn is_destruct_assignment_desugaring(&self, expr: &hir::Expr<'_>) -> bool {
        if let hir::ExprKind::Path(hir::QPath::Resolved(
            _,
            hir::Path { res: hir::def::Res::Local(bind_hir_id), .. },
        )) = expr.kind
        {
            let bind = self.tcx.hir().find(*bind_hir_id);
            let parent = self.tcx.hir().find(self.tcx.hir().parent_id(*bind_hir_id));
            if let Some(hir::Node::Pat(hir::Pat { kind: hir::PatKind::Binding(_, _hir_id, _, _), .. })) = bind &&
                let Some(hir::Node::Pat(hir::Pat { default_binding_modes: false, .. })) = parent {
                    return true;
                }
        }
        return false;
    }

    /// This function is used to determine potential "simple" improvements or users' errors and
    /// provide them useful help. For example:
    ///
    /// ```compile_fail,E0308
    /// fn some_fn(s: &str) {}
    ///
    /// let x = "hey!".to_owned();
    /// some_fn(x); // error
    /// ```
    ///
    /// No need to find every potential function which could make a coercion to transform a
    /// `String` into a `&str` since a `&` would do the trick!
    ///
    /// In addition of this check, it also checks between references mutability state. If the
    /// expected is mutable but the provided isn't, maybe we could just say "Hey, try with
    /// `&mut`!".
    pub fn suggest_deref_or_ref(
        &self,
        expr: &hir::Expr<'tcx>,
        checked_ty: Ty<'tcx>,
        expected: Ty<'tcx>,
    ) -> Option<(
        Vec<(Span, String)>,
        String,
        Applicability,
        bool, /* verbose */
        bool, /* suggest `&` or `&mut` type annotation */
    )> {
        let sess = self.sess();
        let sp = expr.span;

        // If the span is from an external macro, there's no suggestion we can make.
        if in_external_macro(sess, sp) {
            return None;
        }

        let sm = sess.source_map();

        let replace_prefix = |s: &str, old: &str, new: &str| {
            s.strip_prefix(old).map(|stripped| new.to_string() + stripped)
        };

        // `ExprKind::DropTemps` is semantically irrelevant for these suggestions.
        let expr = expr.peel_drop_temps();

        match (&expr.kind, expected.kind(), checked_ty.kind()) {
            (_, &ty::Ref(_, exp, _), &ty::Ref(_, check, _)) => match (exp.kind(), check.kind()) {
                (&ty::Str, &ty::Array(arr, _) | &ty::Slice(arr)) if arr == self.tcx.types.u8 => {
                    if let hir::ExprKind::Lit(_) = expr.kind
                        && let Ok(src) = sm.span_to_snippet(sp)
                        && replace_prefix(&src, "b\"", "\"").is_some()
                    {
                        let pos = sp.lo() + BytePos(1);
                        return Some((
                            vec![(sp.with_hi(pos), String::new())],
                            "consider removing the leading `b`".to_string(),
                            Applicability::MachineApplicable,
                            true,
                            false,
                        ));
                    }
                }
                (&ty::Array(arr, _) | &ty::Slice(arr), &ty::Str) if arr == self.tcx.types.u8 => {
                    if let hir::ExprKind::Lit(_) = expr.kind
                        && let Ok(src) = sm.span_to_snippet(sp)
                        && replace_prefix(&src, "\"", "b\"").is_some()
                    {
                        return Some((
                            vec![(sp.shrink_to_lo(), "b".to_string())],
                            "consider adding a leading `b`".to_string(),
                            Applicability::MachineApplicable,
                            true,
                            false,
                        ));
                    }
                }
                _ => {}
            },
            (_, &ty::Ref(_, _, mutability), _) => {
                // Check if it can work when put into a ref. For example:
                //
                // ```
                // fn bar(x: &mut i32) {}
                //
                // let x = 0u32;
                // bar(&x); // error, expected &mut
                // ```
                let ref_ty = match mutability {
                    hir::Mutability::Mut => {
                        Ty::new_mut_ref(self.tcx,self.tcx.lifetimes.re_static, checked_ty)
                    }
                    hir::Mutability::Not => {
                        Ty::new_imm_ref(self.tcx,self.tcx.lifetimes.re_static, checked_ty)
                    }
                };
                if self.can_coerce(ref_ty, expected) {
                    let mut sugg_sp = sp;
                    if let hir::ExprKind::MethodCall(ref segment, receiver, args, _) = expr.kind {
                        let clone_trait =
                            self.tcx.require_lang_item(LangItem::Clone, Some(segment.ident.span));
                        if args.is_empty()
                            && self.typeck_results.borrow().type_dependent_def_id(expr.hir_id).map(
                                |did| {
                                    let ai = self.tcx.associated_item(did);
                                    ai.trait_container(self.tcx) == Some(clone_trait)
                                },
                            ) == Some(true)
                            && segment.ident.name == sym::clone
                        {
                            // If this expression had a clone call when suggesting borrowing
                            // we want to suggest removing it because it'd now be unnecessary.
                            sugg_sp = receiver.span;
                        }
                    }

                    if let hir::ExprKind::Unary(hir::UnOp::Deref, ref inner) = expr.kind
                        && let Some(1) = self.deref_steps(expected, checked_ty)
                    {
                        // We have `*&T`, check if what was expected was `&T`.
                        // If so, we may want to suggest removing a `*`.
                        sugg_sp = sugg_sp.with_hi(inner.span.lo());
                        return Some((
                            vec![(sugg_sp, String::new())],
                            "consider removing deref here".to_string(),
                            Applicability::MachineApplicable,
                            true,
                            false,
                        ));
                    }

                    let needs_parens = match expr.kind {
                        // parenthesize if needed (Issue #46756)
                        hir::ExprKind::Cast(_, _) | hir::ExprKind::Binary(_, _, _) => true,
                        // parenthesize borrows of range literals (Issue #54505)
                        _ if is_range_literal(expr) => true,
                        _ => false,
                    };

                    if let Some((sugg, msg)) = self.can_use_as_ref(expr) {
                        return Some((
                            sugg,
                            msg.to_string(),
                            Applicability::MachineApplicable,
                            true,
                            false,
                        ));
                    }

                    let prefix = match self.tcx.hir().maybe_get_struct_pattern_shorthand_field(expr) {
                        Some(ident) => format!("{ident}: "),
                        None => String::new(),
                    };

                    if let Some(hir::Node::Expr(hir::Expr {
                        kind: hir::ExprKind::Assign(..),
                        ..
                    })) = self.tcx.hir().find_parent(expr.hir_id)
                    {
                        if mutability.is_mut() {
                            // Suppressing this diagnostic, we'll properly print it in `check_expr_assign`
                            return None;
                        }
                    }

                    let sugg = mutability.ref_prefix_str();
                    let (sugg, verbose) = if needs_parens {
                        (
                            vec![
                                (sp.shrink_to_lo(), format!("{prefix}{sugg}(")),
                                (sp.shrink_to_hi(), ")".to_string()),
                            ],
                            false,
                        )
                    } else {
                        (vec![(sp.shrink_to_lo(), format!("{prefix}{sugg}"))], true)
                    };
                    return Some((
                        sugg,
                        format!("consider {}borrowing here", mutability.mutably_str()),
                        Applicability::MachineApplicable,
                        verbose,
                        false,
                    ));
                }
            }
            (
                hir::ExprKind::AddrOf(hir::BorrowKind::Ref, _, ref expr),
                _,
                &ty::Ref(_, checked, _),
            ) if self.can_sub(self.param_env, checked, expected) => {
                let make_sugg = |start: Span, end: BytePos| {
                    // skip `(` for tuples such as `(c) = (&123)`.
                    // make sure we won't suggest like `(c) = 123)` which is incorrect.
                    let sp = sm.span_extend_while(start.shrink_to_lo(), |c| c == '(' || c.is_whitespace())
                                .map_or(start, |s| s.shrink_to_hi());
                    Some((
                        vec![(sp.with_hi(end), String::new())],
                        "consider removing the borrow".to_string(),
                        Applicability::MachineApplicable,
                        true,
                        true,
                    ))
                };

                // We have `&T`, check if what was expected was `T`. If so,
                // we may want to suggest removing a `&`.
                if sm.is_imported(expr.span) {
                    // Go through the spans from which this span was expanded,
                    // and find the one that's pointing inside `sp`.
                    //
                    // E.g. for `&format!("")`, where we want the span to the
                    // `format!()` invocation instead of its expansion.
                    if let Some(call_span) =
                        iter::successors(Some(expr.span), |s| s.parent_callsite())
                            .find(|&s| sp.contains(s))
                        && sm.is_span_accessible(call_span)
                    {
                        return make_sugg(sp, call_span.lo())
                    }
                    return None;
                }
                if sp.contains(expr.span) && sm.is_span_accessible(expr.span) {
                    return make_sugg(sp, expr.span.lo())
                }
            }
            (
                _,
                &ty::RawPtr(TypeAndMut { ty: ty_b, mutbl: mutbl_b }),
                &ty::Ref(_, ty_a, mutbl_a),
            ) => {
                if let Some(steps) = self.deref_steps(ty_a, ty_b)
                    // Only suggest valid if dereferencing needed.
                    && steps > 0
                    // The pointer type implements `Copy` trait so the suggestion is always valid.
                    && let Ok(src) = sm.span_to_snippet(sp)
                {
                    let derefs = "*".repeat(steps);
                    let old_prefix = mutbl_a.ref_prefix_str();
                    let new_prefix = mutbl_b.ref_prefix_str().to_owned() + &derefs;

                    let suggestion = replace_prefix(&src, old_prefix, &new_prefix).map(|_| {
                        // skip `&` or `&mut ` if both mutabilities are mutable
                        let lo = sp.lo()
                            + BytePos(min(old_prefix.len(), mutbl_b.ref_prefix_str().len()) as _);
                        // skip `&` or `&mut `
                        let hi = sp.lo() + BytePos(old_prefix.len() as _);
                        let sp = sp.with_lo(lo).with_hi(hi);

                        (
                            sp,
                            format!(
                                "{}{derefs}",
                                if mutbl_a != mutbl_b { mutbl_b.prefix_str() } else { "" }
                            ),
                            if mutbl_b <= mutbl_a {
                                Applicability::MachineApplicable
                            } else {
                                Applicability::MaybeIncorrect
                            },
                        )
                    });

                    if let Some((span, src, applicability)) = suggestion {
                        return Some((
                            vec![(span, src)],
                            "consider dereferencing".to_string(),
                            applicability,
                            true,
                            false,
                        ));
                    }
                }
            }
            _ if sp == expr.span => {
                if let Some(mut steps) = self.deref_steps(checked_ty, expected) {
                    let mut expr = expr.peel_blocks();
                    let mut prefix_span = expr.span.shrink_to_lo();
                    let mut remove = String::new();

                    // Try peeling off any existing `&` and `&mut` to reach our target type
                    while steps > 0 {
                        if let hir::ExprKind::AddrOf(_, mutbl, inner) = expr.kind {
                            // If the expression has `&`, removing it would fix the error
                            prefix_span = prefix_span.with_hi(inner.span.lo());
                            expr = inner;
                            remove.push_str(mutbl.ref_prefix_str());
                            steps -= 1;
                        } else {
                            break;
                        }
                    }
                    // If we've reached our target type with just removing `&`, then just print now.
                    if steps == 0 && !remove.trim().is_empty() {
                        return Some((
                            vec![(prefix_span, String::new())],
                            format!("consider removing the `{}`", remove.trim()),
                            // Do not remove `&&` to get to bool, because it might be something like
                            // { a } && b, which we have a separate fixup suggestion that is more
                            // likely correct...
                            if remove.trim() == "&&" && expected == self.tcx.types.bool {
                                Applicability::MaybeIncorrect
                            } else {
                                Applicability::MachineApplicable
                            },
                            true,
                            false,
                        ));
                    }

                    // For this suggestion to make sense, the type would need to be `Copy`,
                    // or we have to be moving out of a `Box<T>`
                    if self.type_is_copy_modulo_regions(self.param_env, expected)
                        // FIXME(compiler-errors): We can actually do this if the checked_ty is
                        // `steps` layers of boxes, not just one, but this is easier and most likely.
                        || (checked_ty.is_box() && steps == 1)
                        // We can always deref a binop that takes its arguments by ref.
                        || matches!(
                            self.tcx.hir().get_parent(expr.hir_id),
                            hir::Node::Expr(hir::Expr { kind: hir::ExprKind::Binary(op, ..), .. })
                                if !op.node.is_by_value()
                        )
                    {
                        let deref_kind = if checked_ty.is_box() {
                            "unboxing the value"
                        } else if checked_ty.is_ref() {
                            "dereferencing the borrow"
                        } else {
                            "dereferencing the type"
                        };

                        // Suggest removing `&` if we have removed any, otherwise suggest just
                        // dereferencing the remaining number of steps.
                        let message = if remove.is_empty() {
                            format!("consider {deref_kind}")
                        } else {
                            format!(
                                "consider removing the `{}` and {} instead",
                                remove.trim(),
                                deref_kind
                            )
                        };

                        let prefix = match self.tcx.hir().maybe_get_struct_pattern_shorthand_field(expr) {
                            Some(ident) => format!("{ident}: "),
                            None => String::new(),
                        };

                        let (span, suggestion) = if self.is_else_if_block(expr) {
                            // Don't suggest nonsense like `else *if`
                            return None;
                        } else if let Some(expr) = self.maybe_get_block_expr(expr) {
                            // prefix should be empty here..
                            (expr.span.shrink_to_lo(), "*".to_string())
                        } else {
                            (prefix_span, format!("{}{}", prefix, "*".repeat(steps)))
                        };
                        if suggestion.trim().is_empty() {
                            return None;
                        }

                        return Some((
                            vec![(span, suggestion)],
                            message,
                            Applicability::MachineApplicable,
                            true,
                            false,
                        ));
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub fn suggest_cast(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        checked_ty: Ty<'tcx>,
        expected_ty: Ty<'tcx>,
        expected_ty_expr: Option<&'tcx hir::Expr<'tcx>>,
    ) -> bool {
        if self.tcx.sess.source_map().is_imported(expr.span) {
            // Ignore if span is from within a macro.
            return false;
        }

        let Ok(src) = self.tcx.sess.source_map().span_to_snippet(expr.span) else {
            return false;
        };

        // If casting this expression to a given numeric type would be appropriate in case of a type
        // mismatch.
        //
        // We want to minimize the amount of casting operations that are suggested, as it can be a
        // lossy operation with potentially bad side effects, so we only suggest when encountering
        // an expression that indicates that the original type couldn't be directly changed.
        //
        // For now, don't suggest casting with `as`.
        let can_cast = false;

        let mut sugg = vec![];

        if let Some(hir::Node::ExprField(field)) = self.tcx.hir().find_parent(expr.hir_id) {
            // `expr` is a literal field for a struct, only suggest if appropriate
            if field.is_shorthand {
                // This is a field literal
                sugg.push((field.ident.span.shrink_to_lo(), format!("{}: ", field.ident)));
            } else {
                // Likely a field was meant, but this field wasn't found. Do not suggest anything.
                return false;
            }
        };

        if let hir::ExprKind::Call(path, args) = &expr.kind
            && let (hir::ExprKind::Path(hir::QPath::TypeRelative(base_ty, path_segment)), 1) =
                (&path.kind, args.len())
            // `expr` is a conversion like `u32::from(val)`, do not suggest anything (#63697).
            && let (hir::TyKind::Path(hir::QPath::Resolved(None, base_ty_path)), sym::from) =
                (&base_ty.kind, path_segment.ident.name)
        {
            if let Some(ident) = &base_ty_path.segments.iter().map(|s| s.ident).next() {
                match ident.name {
                    sym::i128
                    | sym::i64
                    | sym::i32
                    | sym::i16
                    | sym::i8
                    | sym::u128
                    | sym::u64
                    | sym::u32
                    | sym::u16
                    | sym::u8
                    | sym::isize
                    | sym::usize
                        if base_ty_path.segments.len() == 1 =>
                    {
                        return false;
                    }
                    _ => {}
                }
            }
        }

        let msg = format!(
            "you can convert {} `{}` to {} `{}`",
            checked_ty.kind().article(),
            checked_ty,
            expected_ty.kind().article(),
            expected_ty,
        );
        let cast_msg = format!(
            "you can cast {} `{}` to {} `{}`",
            checked_ty.kind().article(),
            checked_ty,
            expected_ty.kind().article(),
            expected_ty,
        );
        let lit_msg = format!(
            "change the type of the numeric literal from `{checked_ty}` to `{expected_ty}`",
        );

        let close_paren = if expr.precedence().order() < PREC_POSTFIX {
            sugg.push((expr.span.shrink_to_lo(), "(".to_string()));
            ")"
        } else {
            ""
        };

        let mut cast_suggestion = sugg.clone();
        cast_suggestion.push((expr.span.shrink_to_hi(), format!("{close_paren} as {expected_ty}")));
        let mut into_suggestion = sugg.clone();
        into_suggestion.push((expr.span.shrink_to_hi(), format!("{close_paren}.into()")));
        let mut suffix_suggestion = sugg.clone();
        suffix_suggestion.push((
            if matches!(
                (&expected_ty.kind(), &checked_ty.kind()),
                (ty::Int(_) | ty::Uint(_), ty::Float(_))
            ) {
                // Remove fractional part from literal, for example `42.0f32` into `42`
                let src = src.trim_end_matches(&checked_ty.to_string());
                let len = src.split('.').next().unwrap().len();
                expr.span.with_lo(expr.span.lo() + BytePos(len as u32))
            } else {
                let len = src.trim_end_matches(&checked_ty.to_string()).len();
                expr.span.with_lo(expr.span.lo() + BytePos(len as u32))
            },
            if expr.precedence().order() < PREC_POSTFIX {
                // Readd `)`
                format!("{expected_ty})")
            } else {
                expected_ty.to_string()
            },
        ));
        let literal_is_ty_suffixed = |expr: &hir::Expr<'_>| {
            if let hir::ExprKind::Lit(lit) = &expr.kind { lit.node.is_suffixed() } else { false }
        };
        let is_negative_int =
            |expr: &hir::Expr<'_>| matches!(expr.kind, hir::ExprKind::Unary(hir::UnOp::Neg, ..));
        let is_uint = |ty: Ty<'_>| matches!(ty.kind(), ty::Uint(..));

        let in_const_context = self.tcx.hir().is_inside_const_context(expr.hir_id);

        let suggest_fallible_into_or_lhs_from =
            |err: &mut Diagnostic, exp_to_found_is_fallible: bool| {
                // If we know the expression the expected type is derived from, we might be able
                // to suggest a widening conversion rather than a narrowing one (which may
                // panic). For example, given x: u8 and y: u32, if we know the span of "x",
                //   x > y
                // can be given the suggestion "u32::from(x) > y" rather than
                // "x > y.try_into().unwrap()".
                let lhs_expr_and_src = expected_ty_expr.and_then(|expr| {
                    self.tcx
                        .sess
                        .source_map()
                        .span_to_snippet(expr.span)
                        .ok()
                        .map(|src| (expr, src))
                });
                let (msg, suggestion) = if let (Some((lhs_expr, lhs_src)), false) =
                    (lhs_expr_and_src, exp_to_found_is_fallible)
                {
                    let msg = format!(
                        "you can convert `{lhs_src}` from `{expected_ty}` to `{checked_ty}`, matching the type of `{src}`",
                    );
                    let suggestion = vec![
                        (lhs_expr.span.shrink_to_lo(), format!("{checked_ty}::from(")),
                        (lhs_expr.span.shrink_to_hi(), ")".to_string()),
                    ];
                    (msg, suggestion)
                } else {
                    let msg =
                        format!("{} and panic if the converted value doesn't fit", msg.clone());
                    let mut suggestion = sugg.clone();
                    suggestion.push((
                        expr.span.shrink_to_hi(),
                        format!("{close_paren}.try_into().unwrap()"),
                    ));
                    (msg, suggestion)
                };
                err.multipart_suggestion_verbose(msg, suggestion, Applicability::MachineApplicable);
            };

        let suggest_to_change_suffix_or_into =
            |err: &mut Diagnostic,
             found_to_exp_is_fallible: bool,
             exp_to_found_is_fallible: bool| {
                let exp_is_lhs = expected_ty_expr.is_some_and(|e| self.tcx.hir().is_lhs(e.hir_id));

                if exp_is_lhs {
                    return;
                }

                let always_fallible = found_to_exp_is_fallible
                    && (exp_to_found_is_fallible || expected_ty_expr.is_none());
                let msg = if literal_is_ty_suffixed(expr) {
                    lit_msg.clone()
                } else if always_fallible && (is_negative_int(expr) && is_uint(expected_ty)) {
                    // We now know that converting either the lhs or rhs is fallible. Before we
                    // suggest a fallible conversion, check if the value can never fit in the
                    // expected type.
                    let msg = format!("`{src}` cannot fit into type `{expected_ty}`");
                    err.note(msg);
                    return;
                } else if in_const_context {
                    // Do not recommend `into` or `try_into` in const contexts.
                    return;
                } else if found_to_exp_is_fallible {
                    return suggest_fallible_into_or_lhs_from(err, exp_to_found_is_fallible);
                } else {
                    msg.clone()
                };
                let suggestion = if literal_is_ty_suffixed(expr) {
                    suffix_suggestion.clone()
                } else {
                    into_suggestion.clone()
                };
                err.multipart_suggestion_verbose(msg, suggestion, Applicability::MachineApplicable);
            };

        match (&expected_ty.kind(), &checked_ty.kind()) {
            (ty::Int(exp), ty::Int(found)) => {
                let (f2e_is_fallible, e2f_is_fallible) = match (exp.bit_width(), found.bit_width())
                {
                    (Some(exp), Some(found)) if exp < found => (true, false),
                    (Some(exp), Some(found)) if exp > found => (false, true),
                    (None, Some(8 | 16)) => (false, true),
                    (Some(8 | 16), None) => (true, false),
                    (None, _) | (_, None) => (true, true),
                    _ => (false, false),
                };
                suggest_to_change_suffix_or_into(err, f2e_is_fallible, e2f_is_fallible);
                true
            }
            (ty::Uint(exp), ty::Uint(found)) => {
                let (f2e_is_fallible, e2f_is_fallible) = match (exp.bit_width(), found.bit_width())
                {
                    (Some(exp), Some(found)) if exp < found => (true, false),
                    (Some(exp), Some(found)) if exp > found => (false, true),
                    (None, Some(8 | 16)) => (false, true),
                    (Some(8 | 16), None) => (true, false),
                    (None, _) | (_, None) => (true, true),
                    _ => (false, false),
                };
                suggest_to_change_suffix_or_into(err, f2e_is_fallible, e2f_is_fallible);
                true
            }
            (&ty::Int(exp), &ty::Uint(found)) => {
                let (f2e_is_fallible, e2f_is_fallible) = match (exp.bit_width(), found.bit_width())
                {
                    (Some(exp), Some(found)) if found < exp => (false, true),
                    (None, Some(8)) => (false, true),
                    _ => (true, true),
                };
                suggest_to_change_suffix_or_into(err, f2e_is_fallible, e2f_is_fallible);
                true
            }
            (&ty::Uint(exp), &ty::Int(found)) => {
                let (f2e_is_fallible, e2f_is_fallible) = match (exp.bit_width(), found.bit_width())
                {
                    (Some(exp), Some(found)) if found > exp => (true, false),
                    (Some(8), None) => (true, false),
                    _ => (true, true),
                };
                suggest_to_change_suffix_or_into(err, f2e_is_fallible, e2f_is_fallible);
                true
            }
            (ty::Float(exp), ty::Float(found)) => {
                if found.bit_width() < exp.bit_width() {
                    suggest_to_change_suffix_or_into(err, false, true);
                } else if literal_is_ty_suffixed(expr) {
                    err.multipart_suggestion_verbose(
                        lit_msg,
                        suffix_suggestion,
                        Applicability::MachineApplicable,
                    );
                } else if can_cast {
                    // Missing try_into implementation for `f64` to `f32`
                    err.multipart_suggestion_verbose(
                        format!("{cast_msg}, producing the closest possible value"),
                        cast_suggestion,
                        Applicability::MaybeIncorrect, // lossy conversion
                    );
                }
                true
            }
            (&ty::Uint(_) | &ty::Int(_), &ty::Float(_)) => {
                if literal_is_ty_suffixed(expr) {
                    err.multipart_suggestion_verbose(
                        lit_msg,
                        suffix_suggestion,
                        Applicability::MachineApplicable,
                    );
                } else if can_cast {
                    // Missing try_into implementation for `{float}` to `{integer}`
                    err.multipart_suggestion_verbose(
                        format!("{msg}, rounding the float towards zero"),
                        cast_suggestion,
                        Applicability::MaybeIncorrect, // lossy conversion
                    );
                }
                true
            }
            (ty::Float(exp), ty::Uint(found)) => {
                // if `found` is `None` (meaning found is `usize`), don't suggest `.into()`
                if exp.bit_width() > found.bit_width().unwrap_or(256) {
                    err.multipart_suggestion_verbose(
                        format!(
                            "{msg}, producing the floating point representation of the integer",
                        ),
                        into_suggestion,
                        Applicability::MachineApplicable,
                    );
                } else if literal_is_ty_suffixed(expr) {
                    err.multipart_suggestion_verbose(
                        lit_msg,
                        suffix_suggestion,
                        Applicability::MachineApplicable,
                    );
                } else {
                    // Missing try_into implementation for `{integer}` to `{float}`
                    err.multipart_suggestion_verbose(
                        format!(
                            "{cast_msg}, producing the floating point representation of the integer, \
                                 rounded if necessary",
                        ),
                        cast_suggestion,
                        Applicability::MaybeIncorrect, // lossy conversion
                    );
                }
                true
            }
            (ty::Float(exp), ty::Int(found)) => {
                // if `found` is `None` (meaning found is `isize`), don't suggest `.into()`
                if exp.bit_width() > found.bit_width().unwrap_or(256) {
                    err.multipart_suggestion_verbose(
                        format!(
                            "{}, producing the floating point representation of the integer",
                            msg.clone(),
                        ),
                        into_suggestion,
                        Applicability::MachineApplicable,
                    );
                } else if literal_is_ty_suffixed(expr) {
                    err.multipart_suggestion_verbose(
                        lit_msg,
                        suffix_suggestion,
                        Applicability::MachineApplicable,
                    );
                } else {
                    // Missing try_into implementation for `{integer}` to `{float}`
                    err.multipart_suggestion_verbose(
                        format!(
                            "{}, producing the floating point representation of the integer, \
                                rounded if necessary",
                            &msg,
                        ),
                        cast_suggestion,
                        Applicability::MaybeIncorrect, // lossy conversion
                    );
                }
                true
            }
            (
                &ty::Uint(ty::UintTy::U32 | ty::UintTy::U64 | ty::UintTy::U128)
                | &ty::Int(ty::IntTy::I32 | ty::IntTy::I64 | ty::IntTy::I128),
                &ty::Char,
            ) => {
                err.multipart_suggestion_verbose(
                    format!("{cast_msg}, since a `char` always occupies 4 bytes"),
                    cast_suggestion,
                    Applicability::MachineApplicable,
                );
                true
            }
            _ => false,
        }
    }

    /// Identify when the user has written `foo..bar()` instead of `foo.bar()`.
    pub fn suggest_method_call_on_range_literal(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'tcx>,
        checked_ty: Ty<'tcx>,
        expected_ty: Ty<'tcx>,
    ) {
        if !hir::is_range_literal(expr) {
            return;
        }
        let hir::ExprKind::Struct(hir::QPath::LangItem(LangItem::Range, ..), [start, end], _) =
            expr.kind
        else {
            return;
        };
        let parent = self.tcx.hir().parent_id(expr.hir_id);
        if let Some(hir::Node::ExprField(_)) = self.tcx.hir().find(parent) {
            // Ignore `Foo { field: a..Default::default() }`
            return;
        }
        let mut expr = end.expr;
        let mut expectation = Some(expected_ty);
        while let hir::ExprKind::MethodCall(_, rcvr, ..) = expr.kind {
            // Getting to the root receiver and asserting it is a fn call let's us ignore cases in
            // `tests/ui/methods/issues/issue-90315.stderr`.
            expr = rcvr;
            // If we have more than one layer of calls, then the expected ty
            // cannot guide the method probe.
            expectation = None;
        }
        let hir::ExprKind::Call(method_name, _) = expr.kind else {
            return;
        };
        let ty::Adt(adt, _) = checked_ty.kind() else {
            return;
        };
        if self.tcx.lang_items().range_struct() != Some(adt.did()) {
            return;
        }
        if let ty::Adt(adt, _) = expected_ty.kind()
            && self.tcx.lang_items().range_struct() == Some(adt.did())
        {
            return;
        }
        // Check if start has method named end.
        let hir::ExprKind::Path(hir::QPath::Resolved(None, p)) = method_name.kind else {
            return;
        };
        let [hir::PathSegment { ident, .. }] = p.segments else {
            return;
        };
        let self_ty = self.typeck_results.borrow().expr_ty(start.expr);
        let Ok(_pick) = self.lookup_probe_for_diagnostic(
            *ident,
            self_ty,
            expr,
            probe::ProbeScope::AllTraits,
            expectation,
        ) else {
            return;
        };
        let mut sugg = ".";
        let mut span = start.expr.span.between(end.expr.span);
        if span.lo() + BytePos(2) == span.hi() {
            // There's no space between the start, the range op and the end, suggest removal which
            // will be more noticeable than the replacement of `..` with `.`.
            span = span.with_lo(span.lo() + BytePos(1));
            sugg = "";
        }
        err.span_suggestion_verbose(
            span,
            "you likely meant to write a method call instead of a range",
            sugg,
            Applicability::MachineApplicable,
        );
    }

    /// Identify when the type error is because `()` is found in a binding that was assigned a
    /// block without a tail expression.
    fn suggest_return_binding_for_missing_tail_expr(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        checked_ty: Ty<'tcx>,
        expected_ty: Ty<'tcx>,
    ) {
        if !checked_ty.is_unit() {
            return;
        }
        let hir::ExprKind::Path(hir::QPath::Resolved(None, path)) = expr.kind else {
            return;
        };
        let hir::def::Res::Local(hir_id) = path.res else {
            return;
        };
        let Some(hir::Node::Pat(pat)) = self.tcx.hir().find(hir_id) else {
            return;
        };
        let Some(hir::Node::Local(hir::Local { ty: None, init: Some(init), .. })) =
            self.tcx.hir().find_parent(pat.hir_id)
        else {
            return;
        };
        let hir::ExprKind::Block(block, None) = init.kind else {
            return;
        };
        if block.expr.is_some() {
            return;
        }
        let [.., stmt] = block.stmts else {
            err.span_label(block.span, "this empty block is missing a tail expression");
            return;
        };
        let hir::StmtKind::Semi(tail_expr) = stmt.kind else {
            return;
        };
        let Some(ty) = self.node_ty_opt(tail_expr.hir_id) else {
            return;
        };
        if self.can_eq(self.param_env, expected_ty, ty) {
            err.span_suggestion_short(
                stmt.span.with_lo(tail_expr.span.hi()),
                "remove this semicolon",
                "",
                Applicability::MachineApplicable,
            );
        } else {
            err.span_label(block.span, "this block is missing a tail expression");
        }
    }

    fn note_wrong_return_ty_due_to_generic_arg(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        checked_ty: Ty<'tcx>,
    ) {
        let Some(hir::Node::Expr(parent_expr)) = self.tcx.hir().find_parent(expr.hir_id) else {
            return;
        };
        enum CallableKind {
            Function,
            Method,
            Constructor,
        }
        let mut maybe_emit_help = |def_id: hir::def_id::DefId,
                                   callable: rustc_span::symbol::Ident,
                                   args: &[hir::Expr<'_>],
                                   kind: CallableKind| {
            let arg_idx = args.iter().position(|a| a.hir_id == expr.hir_id).unwrap();
            let fn_ty = self.tcx.type_of(def_id).skip_binder();
            if !fn_ty.is_fn() {
                return;
            }
            let fn_sig = fn_ty.fn_sig(self.tcx).skip_binder();
            let Some(&arg) = fn_sig
                .inputs()
                .get(arg_idx + if matches!(kind, CallableKind::Method) { 1 } else { 0 })
            else {
                return;
            };
            if matches!(arg.kind(), ty::Param(_))
                && fn_sig.output().contains(arg)
                && self.node_ty(args[arg_idx].hir_id) == checked_ty
            {
                let mut multi_span: MultiSpan = parent_expr.span.into();
                multi_span.push_span_label(
                    args[arg_idx].span,
                    format!(
                        "this argument influences the {} of `{}`",
                        if matches!(kind, CallableKind::Constructor) {
                            "type"
                        } else {
                            "return type"
                        },
                        callable
                    ),
                );
                err.span_help(
                    multi_span,
                    format!(
                        "the {} `{}` due to the type of the argument passed",
                        match kind {
                            CallableKind::Function => "return type of this call is",
                            CallableKind::Method => "return type of this call is",
                            CallableKind::Constructor => "type constructed contains",
                        },
                        checked_ty
                    ),
                );
            }
        };
        match parent_expr.kind {
            hir::ExprKind::Call(fun, args) => {
                let hir::ExprKind::Path(hir::QPath::Resolved(_, path)) = fun.kind else {
                    return;
                };
                let hir::def::Res::Def(kind, def_id) = path.res else {
                    return;
                };
                let callable_kind = if matches!(kind, hir::def::DefKind::Ctor(_, _)) {
                    CallableKind::Constructor
                } else {
                    CallableKind::Function
                };
                maybe_emit_help(def_id, path.segments[0].ident, args, callable_kind);
            }
            hir::ExprKind::MethodCall(method, _receiver, args, _span) => {
                let Some(def_id) =
                    self.typeck_results.borrow().type_dependent_def_id(parent_expr.hir_id)
                else {
                    return;
                };
                maybe_emit_help(def_id, method.ident, args, CallableKind::Method)
            }
            _ => return,
        }
    }
}

pub enum TypeMismatchSource<'tcx> {
    /// Expected the binding to have the given type, but it was found to have
    /// a different type. Find out when that type first became incompatible.
    Ty(Ty<'tcx>),
    /// When we fail during method argument checking, try to find out if a previous
    /// expression has constrained the method's receiver in a way that makes the
    /// argument's type incompatible.
    Arg { call_expr: &'tcx hir::Expr<'tcx>, incompatible_arg: usize },
}
