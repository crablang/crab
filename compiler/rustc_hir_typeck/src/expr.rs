//! Type checking expressions.
//!
//! See `mod.rs` for more context on type checking in general.

use crate::cast;
use crate::coercion::CoerceMany;
use crate::coercion::DynamicCoerceMany;
use crate::errors::ReturnLikeStatementKind;
use crate::errors::TypeMismatchFruTypo;
use crate::errors::{AddressOfTemporaryTaken, ReturnStmtOutsideOfFnBody, StructExprNonExhaustive};
use crate::errors::{
    FieldMultiplySpecifiedInInitializer, FunctionalRecordUpdateOnNonStruct, HelpUseLatestEdition,
    YieldExprOutsideOfGenerator,
};
use crate::fatally_break_rust;
use crate::method::SelfSource;
use crate::type_error_struct;
use crate::Expectation::{self, ExpectCastableToType, ExpectHasType, NoExpectation};
use crate::{
    report_unexpected_variant_res, BreakableCtxt, Diverges, FnCtxt, Needs,
    TupleArgumentsFlag::DontTupleArguments,
};
use rustc_ast as ast;
use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::stack::ensure_sufficient_stack;
use rustc_errors::{
    pluralize, struct_span_err, AddToDiagnostic, Applicability, Diagnostic, DiagnosticBuilder,
    DiagnosticId, ErrorGuaranteed, StashKey,
};
use rustc_hir as hir;
use rustc_hir::def::{CtorKind, DefKind, Res};
use rustc_hir::def_id::DefId;
use rustc_hir::intravisit::Visitor;
use rustc_hir::lang_items::LangItem;
use rustc_hir::{ExprKind, HirId, QPath};
use rustc_hir_analysis::astconv::AstConv as _;
use rustc_hir_analysis::check::ty_kind_suggestion;
use rustc_infer::infer;
use rustc_infer::infer::type_variable::{TypeVariableOrigin, TypeVariableOriginKind};
use rustc_infer::infer::DefineOpaqueTypes;
use rustc_infer::infer::InferOk;
use rustc_infer::traits::query::NoSolution;
use rustc_infer::traits::ObligationCause;
use rustc_middle::middle::stability;
use rustc_middle::ty::adjustment::{Adjust, Adjustment, AllowTwoPhase};
use rustc_middle::ty::error::TypeError::FieldMisMatch;
use rustc_middle::ty::subst::SubstsRef;
use rustc_middle::ty::{self, AdtKind, Ty, TypeVisitableExt};
use rustc_session::errors::ExprParenthesesNeeded;
use rustc_session::parse::feature_err;
use rustc_span::edit_distance::find_best_match_for_name;
use rustc_span::hygiene::DesugaringKind;
use rustc_span::source_map::{Span, Spanned};
use rustc_span::symbol::{kw, sym, Ident, Symbol};
use rustc_target::abi::FieldIdx;
use rustc_target::spec::abi::Abi::RustIntrinsic;
use rustc_trait_selection::infer::InferCtxtExt;
use rustc_trait_selection::traits::error_reporting::TypeErrCtxtExt;
use rustc_trait_selection::traits::ObligationCtxt;
use rustc_trait_selection::traits::{self, ObligationCauseCode};

impl<'a, 'tcx> FnCtxt<'a, 'tcx> {
    fn check_expr_eq_type(&self, expr: &'tcx hir::Expr<'tcx>, expected: Ty<'tcx>) {
        let ty = self.check_expr_with_hint(expr, expected);
        self.demand_eqtype(expr.span, expected, ty);
    }

    pub fn check_expr_has_type_or_error(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        expected: Ty<'tcx>,
        extend_err: impl FnMut(&mut Diagnostic),
    ) -> Ty<'tcx> {
        self.check_expr_meets_expectation_or_error(expr, ExpectHasType(expected), extend_err)
    }

    fn check_expr_meets_expectation_or_error(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        expected: Expectation<'tcx>,
        mut extend_err: impl FnMut(&mut Diagnostic),
    ) -> Ty<'tcx> {
        let expected_ty = expected.to_option(&self).unwrap_or(self.tcx.types.bool);
        let mut ty = self.check_expr_with_expectation(expr, expected);

        // While we don't allow *arbitrary* coercions here, we *do* allow
        // coercions from ! to `expected`.
        if ty.is_never() {
            if let Some(adjustments) = self.typeck_results.borrow().adjustments().get(expr.hir_id) {
                let reported = self.tcx().sess.delay_span_bug(
                    expr.span,
                    "expression with never type wound up being adjusted",
                );
                return if let [Adjustment { kind: Adjust::NeverToAny, target }] = &adjustments[..] {
                    target.to_owned()
                } else {
                    Ty::new_error(self.tcx(), reported)
                };
            }

            let adj_ty = self.next_ty_var(TypeVariableOrigin {
                kind: TypeVariableOriginKind::AdjustmentType,
                span: expr.span,
            });
            self.apply_adjustments(
                expr,
                vec![Adjustment { kind: Adjust::NeverToAny, target: adj_ty }],
            );
            ty = adj_ty;
        }

        if let Some(mut err) = self.demand_suptype_diag(expr.span, expected_ty, ty) {
            let _ = self.emit_type_mismatch_suggestions(
                &mut err,
                expr.peel_drop_temps(),
                ty,
                expected_ty,
                None,
                None,
            );
            extend_err(&mut err);
            err.emit();
        }
        ty
    }

    pub(super) fn check_expr_coercible_to_type(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        expected: Ty<'tcx>,
        expected_ty_expr: Option<&'tcx hir::Expr<'tcx>>,
    ) -> Ty<'tcx> {
        let ty = self.check_expr_with_hint(expr, expected);
        // checks don't need two phase
        self.demand_coerce(expr, ty, expected, expected_ty_expr, AllowTwoPhase::No)
    }

    pub(super) fn check_expr_with_hint(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        expected: Ty<'tcx>,
    ) -> Ty<'tcx> {
        self.check_expr_with_expectation(expr, ExpectHasType(expected))
    }

    fn check_expr_with_expectation_and_needs(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        expected: Expectation<'tcx>,
        needs: Needs,
    ) -> Ty<'tcx> {
        let ty = self.check_expr_with_expectation(expr, expected);

        // If the expression is used in a place whether mutable place is required
        // e.g. LHS of assignment, perform the conversion.
        if let Needs::MutPlace = needs {
            self.convert_place_derefs_to_mutable(expr);
        }

        ty
    }

    pub(super) fn check_expr(&self, expr: &'tcx hir::Expr<'tcx>) -> Ty<'tcx> {
        self.check_expr_with_expectation(expr, NoExpectation)
    }

    pub(super) fn check_expr_with_needs(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        needs: Needs,
    ) -> Ty<'tcx> {
        self.check_expr_with_expectation_and_needs(expr, NoExpectation, needs)
    }

    /// Invariant:
    /// If an expression has any sub-expressions that result in a type error,
    /// inspecting that expression's type with `ty.references_error()` will return
    /// true. Likewise, if an expression is known to diverge, inspecting its
    /// type with `ty::type_is_bot` will return true (n.b.: since Rust is
    /// strict, _|_ can appear in the type of an expression that does not,
    /// itself, diverge: for example, fn() -> _|_.)
    /// Note that inspecting a type's structure *directly* may expose the fact
    /// that there are actually multiple representations for `Error`, so avoid
    /// that when err needs to be handled differently.
    #[instrument(skip(self, expr), level = "debug")]
    pub(super) fn check_expr_with_expectation(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        expected: Expectation<'tcx>,
    ) -> Ty<'tcx> {
        self.check_expr_with_expectation_and_args(expr, expected, &[])
    }

    /// Same as `check_expr_with_expectation`, but allows us to pass in the arguments of a
    /// `ExprKind::Call` when evaluating its callee when it is an `ExprKind::Path`.
    pub(super) fn check_expr_with_expectation_and_args(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        expected: Expectation<'tcx>,
        args: &'tcx [hir::Expr<'tcx>],
    ) -> Ty<'tcx> {
        if self.tcx().sess.verbose() {
            // make this code only run with -Zverbose because it is probably slow
            if let Ok(lint_str) = self.tcx.sess.source_map().span_to_snippet(expr.span) {
                if !lint_str.contains('\n') {
                    debug!("expr text: {lint_str}");
                } else {
                    let mut lines = lint_str.lines();
                    if let Some(line0) = lines.next() {
                        let remaining_lines = lines.count();
                        debug!("expr text: {line0}");
                        debug!("expr text: ...(and {remaining_lines} more lines)");
                    }
                }
            }
        }

        // True if `expr` is a `Try::from_ok(())` that is a result of desugaring a try block
        // without the final expr (e.g. `try { return; }`). We don't want to generate an
        // unreachable_code lint for it since warnings for autogenerated code are confusing.
        let is_try_block_generated_unit_expr = match expr.kind {
            ExprKind::Call(_, args) if expr.span.is_desugaring(DesugaringKind::TryBlock) => {
                args.len() == 1 && args[0].span.is_desugaring(DesugaringKind::TryBlock)
            }

            _ => false,
        };

        // Warn for expressions after diverging siblings.
        if !is_try_block_generated_unit_expr {
            self.warn_if_unreachable(expr.hir_id, expr.span, "expression");
        }

        // Hide the outer diverging and has_errors flags.
        let old_diverges = self.diverges.replace(Diverges::Maybe);

        let ty = ensure_sufficient_stack(|| match &expr.kind {
            hir::ExprKind::Path(
                qpath @ (hir::QPath::Resolved(..) | hir::QPath::TypeRelative(..)),
            ) => self.check_expr_path(qpath, expr, args),
            _ => self.check_expr_kind(expr, expected),
        });
        let ty = self.resolve_vars_if_possible(ty);

        // Warn for non-block expressions with diverging children.
        match expr.kind {
            ExprKind::Block(..)
            | ExprKind::If(..)
            | ExprKind::Let(..)
            | ExprKind::Loop(..)
            | ExprKind::Match(..) => {}
            // If `expr` is a result of desugaring the try block and is an ok-wrapped
            // diverging expression (e.g. it arose from desugaring of `try { return }`),
            // we skip issuing a warning because it is autogenerated code.
            ExprKind::Call(..) if expr.span.is_desugaring(DesugaringKind::TryBlock) => {}
            ExprKind::Call(callee, _) => self.warn_if_unreachable(expr.hir_id, callee.span, "call"),
            ExprKind::MethodCall(segment, ..) => {
                self.warn_if_unreachable(expr.hir_id, segment.ident.span, "call")
            }
            _ => self.warn_if_unreachable(expr.hir_id, expr.span, "expression"),
        }

        // Any expression that produces a value of type `!` must have diverged
        if ty.is_never() {
            self.diverges.set(self.diverges.get() | Diverges::always(expr.span));
        }

        // Record the type, which applies it effects.
        // We need to do this after the warning above, so that
        // we don't warn for the diverging expression itself.
        self.write_ty(expr.hir_id, ty);

        // Combine the diverging and has_error flags.
        self.diverges.set(self.diverges.get() | old_diverges);

        debug!("type of {} is...", self.tcx.hir().node_to_string(expr.hir_id));
        debug!("... {:?}, expected is {:?}", ty, expected);

        ty
    }

    #[instrument(skip(self, expr), level = "debug")]
    fn check_expr_kind(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        expected: Expectation<'tcx>,
    ) -> Ty<'tcx> {
        trace!("expr={:#?}", expr);

        let tcx = self.tcx;
        match expr.kind {
            ExprKind::Lit(ref lit) => self.check_lit(&lit, expected),
            ExprKind::Binary(op, lhs, rhs) => self.check_binop(expr, op, lhs, rhs, expected),
            ExprKind::Assign(lhs, rhs, span) => {
                self.check_expr_assign(expr, expected, lhs, rhs, span)
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                self.check_binop_assign(expr, op, lhs, rhs, expected)
            }
            ExprKind::Unary(unop, oprnd) => self.check_expr_unary(unop, oprnd, expected, expr),
            ExprKind::AddrOf(kind, mutbl, oprnd) => {
                self.check_expr_addr_of(kind, mutbl, oprnd, expected, expr)
            }
            ExprKind::Path(QPath::LangItem(lang_item, _, hir_id)) => {
                self.check_lang_item_path(lang_item, expr, hir_id)
            }
            ExprKind::Path(ref qpath) => self.check_expr_path(qpath, expr, &[]),
            ExprKind::InlineAsm(asm) => {
                // We defer some asm checks as we may not have resolved the input and output types yet (they may still be infer vars).
                self.deferred_asm_checks.borrow_mut().push((asm, expr.hir_id));
                self.check_expr_asm(asm)
            }
            ExprKind::OffsetOf(container, ref fields) => {
                self.check_offset_of(container, fields, expr)
            }
            ExprKind::Break(destination, ref expr_opt) => {
                self.check_expr_break(destination, expr_opt.as_deref(), expr)
            }
            ExprKind::Continue(destination) => {
                if destination.target_id.is_ok() {
                    tcx.types.never
                } else {
                    // There was an error; make type-check fail.
                    Ty::new_misc_error(tcx)
                }
            }
            ExprKind::Ret(ref expr_opt) => self.check_expr_return(expr_opt.as_deref(), expr),
            ExprKind::Become(call) => self.check_expr_become(call, expr),
            ExprKind::Let(let_expr) => self.check_expr_let(let_expr),
            ExprKind::Loop(body, _, source, _) => {
                self.check_expr_loop(body, source, expected, expr)
            }
            ExprKind::Match(discrim, arms, match_src) => {
                self.check_match(expr, &discrim, arms, expected, match_src)
            }
            ExprKind::Closure(closure) => self.check_expr_closure(closure, expr.span, expected),
            ExprKind::Block(body, _) => self.check_block_with_expected(&body, expected),
            ExprKind::Call(callee, args) => self.check_call(expr, &callee, args, expected),
            ExprKind::MethodCall(segment, receiver, args, _) => {
                self.check_method_call(expr, segment, receiver, args, expected)
            }
            ExprKind::Cast(e, t) => self.check_expr_cast(e, t, expr),
            ExprKind::Type(e, t) => {
                let ty = self.to_ty_saving_user_provided_ty(&t);
                self.check_expr_eq_type(&e, ty);
                ty
            }
            ExprKind::If(cond, then_expr, opt_else_expr) => {
                self.check_then_else(cond, then_expr, opt_else_expr, expr.span, expected)
            }
            ExprKind::DropTemps(e) => self.check_expr_with_expectation(e, expected),
            ExprKind::Array(args) => self.check_expr_array(args, expected, expr),
            ExprKind::ConstBlock(ref block) => self.check_expr_const_block(block, expected, expr),
            ExprKind::Repeat(element, ref count) => {
                self.check_expr_repeat(element, count, expected, expr)
            }
            ExprKind::Tup(elts) => self.check_expr_tuple(elts, expected, expr),
            ExprKind::Struct(qpath, fields, ref base_expr) => {
                self.check_expr_struct(expr, expected, qpath, fields, base_expr)
            }
            ExprKind::Field(base, field) => self.check_field(expr, &base, field, expected),
            ExprKind::Index(base, idx) => self.check_expr_index(base, idx, expr),
            ExprKind::Yield(value, ref src) => self.check_expr_yield(value, expr, src),
            hir::ExprKind::Err(guar) => Ty::new_error(tcx, guar),
        }
    }

    fn check_expr_unary(
        &self,
        unop: hir::UnOp,
        oprnd: &'tcx hir::Expr<'tcx>,
        expected: Expectation<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let tcx = self.tcx;
        let expected_inner = match unop {
            hir::UnOp::Not | hir::UnOp::Neg => expected,
            hir::UnOp::Deref => NoExpectation,
        };
        let mut oprnd_t = self.check_expr_with_expectation(&oprnd, expected_inner);

        if !oprnd_t.references_error() {
            oprnd_t = self.structurally_resolve_type(expr.span, oprnd_t);
            match unop {
                hir::UnOp::Deref => {
                    if let Some(ty) = self.lookup_derefing(expr, oprnd, oprnd_t) {
                        oprnd_t = ty;
                    } else {
                        let mut err = type_error_struct!(
                            tcx.sess,
                            expr.span,
                            oprnd_t,
                            E0614,
                            "type `{oprnd_t}` cannot be dereferenced",
                        );
                        let sp = tcx.sess.source_map().start_point(expr.span).with_parent(None);
                        if let Some(sp) =
                            tcx.sess.parse_sess.ambiguous_block_expr_parse.borrow().get(&sp)
                        {
                            err.subdiagnostic(ExprParenthesesNeeded::surrounding(*sp));
                        }
                        oprnd_t = Ty::new_error(tcx, err.emit());
                    }
                }
                hir::UnOp::Not => {
                    let result = self.check_user_unop(expr, oprnd_t, unop, expected_inner);
                    // If it's builtin, we can reuse the type, this helps inference.
                    if !(oprnd_t.is_integral() || *oprnd_t.kind() == ty::Bool) {
                        oprnd_t = result;
                    }
                }
                hir::UnOp::Neg => {
                    let result = self.check_user_unop(expr, oprnd_t, unop, expected_inner);
                    // If it's builtin, we can reuse the type, this helps inference.
                    if !oprnd_t.is_numeric() {
                        oprnd_t = result;
                    }
                }
            }
        }
        oprnd_t
    }

    fn check_expr_addr_of(
        &self,
        kind: hir::BorrowKind,
        mutbl: hir::Mutability,
        oprnd: &'tcx hir::Expr<'tcx>,
        expected: Expectation<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let hint = expected.only_has_type(self).map_or(NoExpectation, |ty| {
            match ty.kind() {
                ty::Ref(_, ty, _) | ty::RawPtr(ty::TypeAndMut { ty, .. }) => {
                    if oprnd.is_syntactic_place_expr() {
                        // Places may legitimately have unsized types.
                        // For example, dereferences of a fat pointer and
                        // the last field of a struct can be unsized.
                        ExpectHasType(*ty)
                    } else {
                        Expectation::rvalue_hint(self, *ty)
                    }
                }
                _ => NoExpectation,
            }
        });
        let ty =
            self.check_expr_with_expectation_and_needs(&oprnd, hint, Needs::maybe_mut_place(mutbl));

        let tm = ty::TypeAndMut { ty, mutbl };
        match kind {
            _ if tm.ty.references_error() => Ty::new_misc_error(self.tcx),
            hir::BorrowKind::Raw => {
                self.check_named_place_expr(oprnd);
                Ty::new_ptr(self.tcx, tm)
            }
            hir::BorrowKind::Ref => {
                // Note: at this point, we cannot say what the best lifetime
                // is to use for resulting pointer. We want to use the
                // shortest lifetime possible so as to avoid spurious borrowck
                // errors. Moreover, the longest lifetime will depend on the
                // precise details of the value whose address is being taken
                // (and how long it is valid), which we don't know yet until
                // type inference is complete.
                //
                // Therefore, here we simply generate a region variable. The
                // region inferencer will then select a suitable value.
                // Finally, borrowck will infer the value of the region again,
                // this time with enough precision to check that the value
                // whose address was taken can actually be made to live as long
                // as it needs to live.
                let region = self.next_region_var(infer::AddrOfRegion(expr.span));
                Ty::new_ref(self.tcx, region, tm)
            }
        }
    }

    /// Does this expression refer to a place that either:
    /// * Is based on a local or static.
    /// * Contains a dereference
    /// Note that the adjustments for the children of `expr` should already
    /// have been resolved.
    fn check_named_place_expr(&self, oprnd: &'tcx hir::Expr<'tcx>) {
        let is_named = oprnd.is_place_expr(|base| {
            // Allow raw borrows if there are any deref adjustments.
            //
            // const VAL: (i32,) = (0,);
            // const REF: &(i32,) = &(0,);
            //
            // &raw const VAL.0;            // ERROR
            // &raw const REF.0;            // OK, same as &raw const (*REF).0;
            //
            // This is maybe too permissive, since it allows
            // `let u = &raw const Box::new((1,)).0`, which creates an
            // immediately dangling raw pointer.
            self.typeck_results
                .borrow()
                .adjustments()
                .get(base.hir_id)
                .is_some_and(|x| x.iter().any(|adj| matches!(adj.kind, Adjust::Deref(_))))
        });
        if !is_named {
            self.tcx.sess.emit_err(AddressOfTemporaryTaken { span: oprnd.span });
        }
    }

    fn check_lang_item_path(
        &self,
        lang_item: hir::LangItem,
        expr: &'tcx hir::Expr<'tcx>,
        hir_id: Option<hir::HirId>,
    ) -> Ty<'tcx> {
        self.resolve_lang_item_path(lang_item, expr.span, expr.hir_id, hir_id).1
    }

    pub(crate) fn check_expr_path(
        &self,
        qpath: &'tcx hir::QPath<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
        args: &'tcx [hir::Expr<'tcx>],
    ) -> Ty<'tcx> {
        let tcx = self.tcx;
        let (res, opt_ty, segs) =
            self.resolve_ty_and_res_fully_qualified_call(qpath, expr.hir_id, expr.span);
        let ty = match res {
            Res::Err => {
                self.suggest_assoc_method_call(segs);
                let e =
                    self.tcx.sess.delay_span_bug(qpath.span(), "`Res::Err` but no error emitted");
                self.set_tainted_by_errors(e);
                Ty::new_error(tcx, e)
            }
            Res::Def(DefKind::Variant, _) => {
                let e = report_unexpected_variant_res(tcx, res, qpath, expr.span, "E0533", "value");
                Ty::new_error(tcx, e)
            }
            _ => self.instantiate_value_path(segs, opt_ty, res, expr.span, expr.hir_id).0,
        };

        if let ty::FnDef(did, ..) = *ty.kind() {
            let fn_sig = ty.fn_sig(tcx);
            if tcx.fn_sig(did).skip_binder().abi() == RustIntrinsic
                && tcx.item_name(did) == sym::transmute
            {
                let from = fn_sig.inputs().skip_binder()[0];
                let to = fn_sig.output().skip_binder();
                // We defer the transmute to the end of typeck, once all inference vars have
                // been resolved or we errored. This is important as we can only check transmute
                // on concrete types, but the output type may not be known yet (it would only
                // be known if explicitly specified via turbofish).
                self.deferred_transmute_checks.borrow_mut().push((from, to, expr.hir_id));
            }
            if !tcx.features().unsized_fn_params {
                // We want to remove some Sized bounds from std functions,
                // but don't want to expose the removal to stable Rust.
                // i.e., we don't want to allow
                //
                // ```rust
                // drop as fn(str);
                // ```
                //
                // to work in stable even if the Sized bound on `drop` is relaxed.
                for i in 0..fn_sig.inputs().skip_binder().len() {
                    // We just want to check sizedness, so instead of introducing
                    // placeholder lifetimes with probing, we just replace higher lifetimes
                    // with fresh vars.
                    let span = args.get(i).map(|a| a.span).unwrap_or(expr.span);
                    let input = self.instantiate_binder_with_fresh_vars(
                        span,
                        infer::LateBoundRegionConversionTime::FnCall,
                        fn_sig.input(i),
                    );
                    self.require_type_is_sized_deferred(
                        input,
                        span,
                        traits::SizedArgumentType(None),
                    );
                }
            }
            // Here we want to prevent struct constructors from returning unsized types.
            // There were two cases this happened: fn pointer coercion in stable
            // and usual function call in presence of unsized_locals.
            // Also, as we just want to check sizedness, instead of introducing
            // placeholder lifetimes with probing, we just replace higher lifetimes
            // with fresh vars.
            let output = self.instantiate_binder_with_fresh_vars(
                expr.span,
                infer::LateBoundRegionConversionTime::FnCall,
                fn_sig.output(),
            );
            self.require_type_is_sized_deferred(output, expr.span, traits::SizedReturnType);
        }

        // We always require that the type provided as the value for
        // a type parameter outlives the moment of instantiation.
        let substs = self.typeck_results.borrow().node_substs(expr.hir_id);
        self.add_wf_bounds(substs, expr);

        ty
    }

    fn check_expr_break(
        &self,
        destination: hir::Destination,
        expr_opt: Option<&'tcx hir::Expr<'tcx>>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let tcx = self.tcx;
        if let Ok(target_id) = destination.target_id {
            let (e_ty, cause);
            if let Some(e) = expr_opt {
                // If this is a break with a value, we need to type-check
                // the expression. Get an expected type from the loop context.
                let opt_coerce_to = {
                    // We should release `enclosing_breakables` before the `check_expr_with_hint`
                    // below, so can't move this block of code to the enclosing scope and share
                    // `ctxt` with the second `enclosing_breakables` borrow below.
                    let mut enclosing_breakables = self.enclosing_breakables.borrow_mut();
                    match enclosing_breakables.opt_find_breakable(target_id) {
                        Some(ctxt) => ctxt.coerce.as_ref().map(|coerce| coerce.expected_ty()),
                        None => {
                            // Avoid ICE when `break` is inside a closure (#65383).
                            return Ty::new_error_with_message(
                                tcx,
                                expr.span,
                                "break was outside loop, but no error was emitted",
                            );
                        }
                    }
                };

                // If the loop context is not a `loop { }`, then break with
                // a value is illegal, and `opt_coerce_to` will be `None`.
                // Just set expectation to error in that case.
                let coerce_to = opt_coerce_to.unwrap_or_else(|| Ty::new_misc_error(tcx));

                // Recurse without `enclosing_breakables` borrowed.
                e_ty = self.check_expr_with_hint(e, coerce_to);
                cause = self.misc(e.span);
            } else {
                // Otherwise, this is a break *without* a value. That's
                // always legal, and is equivalent to `break ()`.
                e_ty = Ty::new_unit(tcx);
                cause = self.misc(expr.span);
            }

            // Now that we have type-checked `expr_opt`, borrow
            // the `enclosing_loops` field and let's coerce the
            // type of `expr_opt` into what is expected.
            let mut enclosing_breakables = self.enclosing_breakables.borrow_mut();
            let Some(ctxt) = enclosing_breakables.opt_find_breakable(target_id) else {
                // Avoid ICE when `break` is inside a closure (#65383).
                return Ty::new_error_with_message(tcx,
                    expr.span,
                    "break was outside loop, but no error was emitted",
                );
            };

            if let Some(ref mut coerce) = ctxt.coerce {
                if let Some(ref e) = expr_opt {
                    coerce.coerce(self, &cause, e, e_ty);
                } else {
                    assert!(e_ty.is_unit());
                    let ty = coerce.expected_ty();
                    coerce.coerce_forced_unit(
                        self,
                        &cause,
                        &mut |mut err| {
                            self.suggest_mismatched_types_on_tail(
                                &mut err, expr, ty, e_ty, target_id,
                            );
                            if let Some(val) = ty_kind_suggestion(ty) {
                                let label = destination
                                    .label
                                    .map(|l| format!(" {}", l.ident))
                                    .unwrap_or_else(String::new);
                                err.span_suggestion(
                                    expr.span,
                                    "give it a value of the expected type",
                                    format!("break{label} {val}"),
                                    Applicability::HasPlaceholders,
                                );
                            }
                        },
                        false,
                    );
                }
            } else {
                // If `ctxt.coerce` is `None`, we can just ignore
                // the type of the expression. This is because
                // either this was a break *without* a value, in
                // which case it is always a legal type (`()`), or
                // else an error would have been flagged by the
                // `loops` pass for using break with an expression
                // where you are not supposed to.
                assert!(expr_opt.is_none() || self.tcx.sess.has_errors().is_some());
            }

            // If we encountered a `break`, then (no surprise) it may be possible to break from the
            // loop... unless the value being returned from the loop diverges itself, e.g.
            // `break return 5` or `break loop {}`.
            ctxt.may_break |= !self.diverges.get().is_always();

            // the type of a `break` is always `!`, since it diverges
            tcx.types.never
        } else {
            // Otherwise, we failed to find the enclosing loop;
            // this can only happen if the `break` was not
            // inside a loop at all, which is caught by the
            // loop-checking pass.
            let err = Ty::new_error_with_message(
                self.tcx,
                expr.span,
                "break was outside loop, but no error was emitted",
            );

            // We still need to assign a type to the inner expression to
            // prevent the ICE in #43162.
            if let Some(e) = expr_opt {
                self.check_expr_with_hint(e, err);

                // ... except when we try to 'break rust;'.
                // ICE this expression in particular (see #43162).
                if let ExprKind::Path(QPath::Resolved(_, path)) = e.kind {
                    if path.segments.len() == 1 && path.segments[0].ident.name == sym::rust {
                        fatally_break_rust(self.tcx);
                    }
                }
            }

            // There was an error; make type-check fail.
            err
        }
    }

    fn check_expr_return(
        &self,
        expr_opt: Option<&'tcx hir::Expr<'tcx>>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        if self.ret_coercion.is_none() {
            self.emit_return_outside_of_fn_body(expr, ReturnLikeStatementKind::Return);

            if let Some(e) = expr_opt {
                // We still have to type-check `e` (issue #86188), but calling
                // `check_return_expr` only works inside fn bodies.
                self.check_expr(e);
            }
        } else if let Some(e) = expr_opt {
            if self.ret_coercion_span.get().is_none() {
                self.ret_coercion_span.set(Some(e.span));
            }
            self.check_return_expr(e, true);
        } else {
            let mut coercion = self.ret_coercion.as_ref().unwrap().borrow_mut();
            if self.ret_coercion_span.get().is_none() {
                self.ret_coercion_span.set(Some(expr.span));
            }
            let cause = self.cause(expr.span, ObligationCauseCode::ReturnNoExpression);
            if let Some((_, fn_decl, _)) = self.get_fn_decl(expr.hir_id) {
                coercion.coerce_forced_unit(
                    self,
                    &cause,
                    &mut |db| {
                        let span = fn_decl.output.span();
                        if let Ok(snippet) = self.tcx.sess.source_map().span_to_snippet(span) {
                            db.span_label(
                                span,
                                format!("expected `{snippet}` because of this return type"),
                            );
                        }
                    },
                    true,
                );
            } else {
                coercion.coerce_forced_unit(self, &cause, &mut |_| (), true);
            }
        }
        self.tcx.types.never
    }

    fn check_expr_become(
        &self,
        call: &'tcx hir::Expr<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        match &self.ret_coercion {
            Some(ret_coercion) => {
                let ret_ty = ret_coercion.borrow().expected_ty();
                let call_expr_ty = self.check_expr_with_hint(call, ret_ty);

                // N.B. don't coerce here, as tail calls can't support most/all coercions
                // FIXME(explicit_tail_calls): add a diagnostic note that `become` doesn't allow coercions
                self.demand_suptype(expr.span, ret_ty, call_expr_ty);
            }
            None => {
                self.emit_return_outside_of_fn_body(expr, ReturnLikeStatementKind::Become);

                // Fallback to simply type checking `call` without hint/demanding the right types.
                // Best effort to highlight more errors.
                self.check_expr(call);
            }
        }

        self.tcx.types.never
    }

    /// Check an expression that _is being returned_.
    /// For example, this is called with `return_expr: $expr` when `return $expr`
    /// is encountered.
    ///
    /// Note that this function must only be called in function bodies.
    ///
    /// `explicit_return` is `true` if we're checking an explicit `return expr`,
    /// and `false` if we're checking a trailing expression.
    pub(super) fn check_return_expr(
        &self,
        return_expr: &'tcx hir::Expr<'tcx>,
        explicit_return: bool,
    ) {
        let ret_coercion = self.ret_coercion.as_ref().unwrap_or_else(|| {
            span_bug!(return_expr.span, "check_return_expr called outside fn body")
        });

        let ret_ty = ret_coercion.borrow().expected_ty();
        let return_expr_ty = self.check_expr_with_hint(return_expr, ret_ty);
        let mut span = return_expr.span;
        // Use the span of the trailing expression for our cause,
        // not the span of the entire function
        if !explicit_return
            && let ExprKind::Block(body, _) = return_expr.kind
            && let Some(last_expr) = body.expr
        {
                span = last_expr.span;
        }
        ret_coercion.borrow_mut().coerce(
            self,
            &self.cause(span, ObligationCauseCode::ReturnValue(return_expr.hir_id)),
            return_expr,
            return_expr_ty,
        );

        if let Some(fn_sig) = self.body_fn_sig()
            && fn_sig.output().has_opaque_types()
        {
            // Point any obligations that were registered due to opaque type
            // inference at the return expression.
            self.select_obligations_where_possible(|errors| {
                self.point_at_return_for_opaque_ty_error(errors, span, return_expr_ty, return_expr.span);
            });
        }
    }

    /// Emit an error because `return` or `become` is used outside of a function body.
    ///
    /// `expr` is the `return` (`become`) "statement", `kind` is the kind of the statement
    /// either `Return` or `Become`.
    fn emit_return_outside_of_fn_body(&self, expr: &hir::Expr<'_>, kind: ReturnLikeStatementKind) {
        let mut err = ReturnStmtOutsideOfFnBody {
            span: expr.span,
            encl_body_span: None,
            encl_fn_span: None,
            statement_kind: kind,
        };

        let encl_item_id = self.tcx.hir().get_parent_item(expr.hir_id);

        if let Some(hir::Node::Item(hir::Item {
            kind: hir::ItemKind::Fn(..),
            span: encl_fn_span,
            ..
        }))
        | Some(hir::Node::TraitItem(hir::TraitItem {
            kind: hir::TraitItemKind::Fn(_, hir::TraitFn::Provided(_)),
            span: encl_fn_span,
            ..
        }))
        | Some(hir::Node::ImplItem(hir::ImplItem {
            kind: hir::ImplItemKind::Fn(..),
            span: encl_fn_span,
            ..
        })) = self.tcx.hir().find_by_def_id(encl_item_id.def_id)
        {
            // We are inside a function body, so reporting "return statement
            // outside of function body" needs an explanation.

            let encl_body_owner_id = self.tcx.hir().enclosing_body_owner(expr.hir_id);

            // If this didn't hold, we would not have to report an error in
            // the first place.
            assert_ne!(encl_item_id.def_id, encl_body_owner_id);

            let encl_body_id = self.tcx.hir().body_owned_by(encl_body_owner_id);
            let encl_body = self.tcx.hir().body(encl_body_id);

            err.encl_body_span = Some(encl_body.value.span);
            err.encl_fn_span = Some(*encl_fn_span);
        }

        self.tcx.sess.emit_err(err);
    }

    fn point_at_return_for_opaque_ty_error(
        &self,
        errors: &mut Vec<traits::FulfillmentError<'tcx>>,
        span: Span,
        return_expr_ty: Ty<'tcx>,
        return_span: Span,
    ) {
        // Don't point at the whole block if it's empty
        if span == return_span {
            return;
        }
        for err in errors {
            let cause = &mut err.obligation.cause;
            if let ObligationCauseCode::OpaqueReturnType(None) = cause.code() {
                let new_cause = ObligationCause::new(
                    cause.span,
                    cause.body_id,
                    ObligationCauseCode::OpaqueReturnType(Some((return_expr_ty, span))),
                );
                *cause = new_cause;
            }
        }
    }

    pub(crate) fn check_lhs_assignable(
        &self,
        lhs: &'tcx hir::Expr<'tcx>,
        err_code: &'static str,
        op_span: Span,
        adjust_err: impl FnOnce(&mut Diagnostic),
    ) {
        if lhs.is_syntactic_place_expr() {
            return;
        }

        // FIXME: Make this use Diagnostic once error codes can be dynamically set.
        let mut err = self.tcx.sess.struct_span_err_with_code(
            op_span,
            "invalid left-hand side of assignment",
            DiagnosticId::Error(err_code.into()),
        );
        err.span_label(lhs.span, "cannot assign to this expression");

        self.comes_from_while_condition(lhs.hir_id, |expr| {
            err.span_suggestion_verbose(
                expr.span.shrink_to_lo(),
                "you might have meant to use pattern destructuring",
                "let ",
                Applicability::MachineApplicable,
            );
        });

        adjust_err(&mut err);

        err.emit();
    }

    // Check if an expression `original_expr_id` comes from the condition of a while loop,
    /// as opposed from the body of a while loop, which we can naively check by iterating
    /// parents until we find a loop...
    pub(super) fn comes_from_while_condition(
        &self,
        original_expr_id: HirId,
        then: impl FnOnce(&hir::Expr<'_>),
    ) {
        let mut parent = self.tcx.hir().parent_id(original_expr_id);
        while let Some(node) = self.tcx.hir().find(parent) {
            match node {
                hir::Node::Expr(hir::Expr {
                    kind:
                        hir::ExprKind::Loop(
                            hir::Block {
                                expr:
                                    Some(hir::Expr {
                                        kind:
                                            hir::ExprKind::Match(expr, ..) | hir::ExprKind::If(expr, ..),
                                        ..
                                    }),
                                ..
                            },
                            _,
                            hir::LoopSource::While,
                            _,
                        ),
                    ..
                }) => {
                    // Check if our original expression is a child of the condition of a while loop
                    let expr_is_ancestor = std::iter::successors(Some(original_expr_id), |id| {
                        self.tcx.hir().opt_parent_id(*id)
                    })
                    .take_while(|id| *id != parent)
                    .any(|id| id == expr.hir_id);
                    // if it is, then we have a situation like `while Some(0) = value.get(0) {`,
                    // where `while let` was more likely intended.
                    if expr_is_ancestor {
                        then(expr);
                    }
                    break;
                }
                hir::Node::Item(_)
                | hir::Node::ImplItem(_)
                | hir::Node::TraitItem(_)
                | hir::Node::Crate(_) => break,
                _ => {
                    parent = self.tcx.hir().parent_id(parent);
                }
            }
        }
    }

    // A generic function for checking the 'then' and 'else' clauses in an 'if'
    // or 'if-else' expression.
    fn check_then_else(
        &self,
        cond_expr: &'tcx hir::Expr<'tcx>,
        then_expr: &'tcx hir::Expr<'tcx>,
        opt_else_expr: Option<&'tcx hir::Expr<'tcx>>,
        sp: Span,
        orig_expected: Expectation<'tcx>,
    ) -> Ty<'tcx> {
        let cond_ty = self.check_expr_has_type_or_error(cond_expr, self.tcx.types.bool, |_| {});

        self.warn_if_unreachable(
            cond_expr.hir_id,
            then_expr.span,
            "block in `if` or `while` expression",
        );

        let cond_diverges = self.diverges.get();
        self.diverges.set(Diverges::Maybe);

        let expected = orig_expected.adjust_for_branches(self);
        let then_ty = self.check_expr_with_expectation(then_expr, expected);
        let then_diverges = self.diverges.get();
        self.diverges.set(Diverges::Maybe);

        // We've already taken the expected type's preferences
        // into account when typing the `then` branch. To figure
        // out the initial shot at a LUB, we thus only consider
        // `expected` if it represents a *hard* constraint
        // (`only_has_type`); otherwise, we just go with a
        // fresh type variable.
        let coerce_to_ty = expected.coercion_target_type(self, sp);
        let mut coerce: DynamicCoerceMany<'_> = CoerceMany::new(coerce_to_ty);

        coerce.coerce(self, &self.misc(sp), then_expr, then_ty);

        if let Some(else_expr) = opt_else_expr {
            let else_ty = self.check_expr_with_expectation(else_expr, expected);
            let else_diverges = self.diverges.get();

            let opt_suggest_box_span = self.opt_suggest_box_span(then_ty, else_ty, orig_expected);
            let if_cause = self.if_cause(
                sp,
                cond_expr.span,
                then_expr,
                else_expr,
                then_ty,
                else_ty,
                opt_suggest_box_span,
            );

            coerce.coerce(self, &if_cause, else_expr, else_ty);

            // We won't diverge unless both branches do (or the condition does).
            self.diverges.set(cond_diverges | then_diverges & else_diverges);
        } else {
            self.if_fallback_coercion(sp, then_expr, &mut coerce);

            // If the condition is false we can't diverge.
            self.diverges.set(cond_diverges);
        }

        let result_ty = coerce.complete(self);
        if let Err(guar) = cond_ty.error_reported() {
            Ty::new_error(self.tcx, guar)
        } else {
            result_ty
        }
    }

    /// Type check assignment expression `expr` of form `lhs = rhs`.
    /// The expected type is `()` and is passed to the function for the purposes of diagnostics.
    fn check_expr_assign(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        expected: Expectation<'tcx>,
        lhs: &'tcx hir::Expr<'tcx>,
        rhs: &'tcx hir::Expr<'tcx>,
        span: Span,
    ) -> Ty<'tcx> {
        let expected_ty = expected.coercion_target_type(self, expr.span);
        if expected_ty == self.tcx.types.bool {
            // The expected type is `bool` but this will result in `()` so we can reasonably
            // say that the user intended to write `lhs == rhs` instead of `lhs = rhs`.
            // The likely cause of this is `if foo = bar { .. }`.
            let actual_ty = Ty::new_unit(self.tcx);
            let mut err = self.demand_suptype_diag(expr.span, expected_ty, actual_ty).unwrap();
            let lhs_ty = self.check_expr(&lhs);
            let rhs_ty = self.check_expr(&rhs);
            let (applicability, eq) = if self.can_coerce(rhs_ty, lhs_ty) {
                (Applicability::MachineApplicable, true)
            } else if let ExprKind::Binary(
                Spanned { node: hir::BinOpKind::And | hir::BinOpKind::Or, .. },
                _,
                rhs_expr,
            ) = lhs.kind
            {
                // if x == 1 && y == 2 { .. }
                //                 +
                let actual_lhs_ty = self.check_expr(&rhs_expr);
                (Applicability::MaybeIncorrect, self.can_coerce(rhs_ty, actual_lhs_ty))
            } else if let ExprKind::Binary(
                Spanned { node: hir::BinOpKind::And | hir::BinOpKind::Or, .. },
                lhs_expr,
                _,
            ) = rhs.kind
            {
                // if x == 1 && y == 2 { .. }
                //       +
                let actual_rhs_ty = self.check_expr(&lhs_expr);
                (Applicability::MaybeIncorrect, self.can_coerce(actual_rhs_ty, lhs_ty))
            } else {
                (Applicability::MaybeIncorrect, false)
            };
            if !lhs.is_syntactic_place_expr()
                && lhs.is_approximately_pattern()
                && !matches!(lhs.kind, hir::ExprKind::Lit(_))
            {
                // Do not suggest `if let x = y` as `==` is way more likely to be the intention.
                let hir = self.tcx.hir();
                if let hir::Node::Expr(hir::Expr { kind: ExprKind::If { .. }, .. }) =
                    hir.get_parent(hir.parent_id(expr.hir_id))
                {
                    err.span_suggestion_verbose(
                        expr.span.shrink_to_lo(),
                        "you might have meant to use pattern matching",
                        "let ",
                        applicability,
                    );
                };
            }
            if eq {
                err.span_suggestion_verbose(
                    span.shrink_to_hi(),
                    "you might have meant to compare for equality",
                    '=',
                    applicability,
                );
            }

            // If the assignment expression itself is ill-formed, don't
            // bother emitting another error
            let reported = err.emit_unless(lhs_ty.references_error() || rhs_ty.references_error());
            return Ty::new_error(self.tcx, reported);
        }

        let lhs_ty = self.check_expr_with_needs(&lhs, Needs::MutPlace);

        let suggest_deref_binop = |err: &mut Diagnostic, rhs_ty: Ty<'tcx>| {
            if let Some(lhs_deref_ty) = self.deref_once_mutably_for_diagnostic(lhs_ty) {
                // Can only assign if the type is sized, so if `DerefMut` yields a type that is
                // unsized, do not suggest dereferencing it.
                let lhs_deref_ty_is_sized = self
                    .infcx
                    .type_implements_trait(
                        self.tcx.require_lang_item(LangItem::Sized, None),
                        [lhs_deref_ty],
                        self.param_env,
                    )
                    .may_apply();
                if lhs_deref_ty_is_sized && self.can_coerce(rhs_ty, lhs_deref_ty) {
                    err.span_suggestion_verbose(
                        lhs.span.shrink_to_lo(),
                        "consider dereferencing here to assign to the mutably borrowed value",
                        "*",
                        Applicability::MachineApplicable,
                    );
                }
            }
        };

        // This is (basically) inlined `check_expr_coercible_to_type`, but we want
        // to suggest an additional fixup here in `suggest_deref_binop`.
        let rhs_ty = self.check_expr_with_hint(&rhs, lhs_ty);
        if let (_, Some(mut diag)) =
            self.demand_coerce_diag(rhs, rhs_ty, lhs_ty, Some(lhs), AllowTwoPhase::No)
        {
            suggest_deref_binop(&mut diag, rhs_ty);
            diag.emit();
        }

        self.check_lhs_assignable(lhs, "E0070", span, |err| {
            if let Some(rhs_ty) = self.typeck_results.borrow().expr_ty_opt(rhs) {
                suggest_deref_binop(err, rhs_ty);
            }
        });

        self.require_type_is_sized(lhs_ty, lhs.span, traits::AssignmentLhsSized);

        if let Err(guar) = (lhs_ty, rhs_ty).error_reported() {
            Ty::new_error(self.tcx, guar)
        } else {
            Ty::new_unit(self.tcx)
        }
    }

    pub(super) fn check_expr_let(&self, let_expr: &'tcx hir::Let<'tcx>) -> Ty<'tcx> {
        // for let statements, this is done in check_stmt
        let init = let_expr.init;
        self.warn_if_unreachable(init.hir_id, init.span, "block in `let` expression");
        // otherwise check exactly as a let statement
        self.check_decl(let_expr.into());
        // but return a bool, for this is a boolean expression
        self.tcx.types.bool
    }

    fn check_expr_loop(
        &self,
        body: &'tcx hir::Block<'tcx>,
        source: hir::LoopSource,
        expected: Expectation<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let coerce = match source {
            // you can only use break with a value from a normal `loop { }`
            hir::LoopSource::Loop => {
                let coerce_to = expected.coercion_target_type(self, body.span);
                Some(CoerceMany::new(coerce_to))
            }

            hir::LoopSource::While | hir::LoopSource::ForLoop => None,
        };

        let ctxt = BreakableCtxt {
            coerce,
            may_break: false, // Will get updated if/when we find a `break`.
        };

        let (ctxt, ()) = self.with_breakable_ctxt(expr.hir_id, ctxt, || {
            self.check_block_no_value(&body);
        });

        if ctxt.may_break {
            // No way to know whether it's diverging because
            // of a `break` or an outer `break` or `return`.
            self.diverges.set(Diverges::Maybe);
        }

        // If we permit break with a value, then result type is
        // the LUB of the breaks (possibly ! if none); else, it
        // is nil. This makes sense because infinite loops
        // (which would have type !) are only possible iff we
        // permit break with a value [1].
        if ctxt.coerce.is_none() && !ctxt.may_break {
            // [1]
            self.tcx.sess.delay_span_bug(body.span, "no coercion, but loop may not break");
        }
        ctxt.coerce.map(|c| c.complete(self)).unwrap_or_else(|| Ty::new_unit(self.tcx))
    }

    /// Checks a method call.
    fn check_method_call(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        segment: &hir::PathSegment<'_>,
        rcvr: &'tcx hir::Expr<'tcx>,
        args: &'tcx [hir::Expr<'tcx>],
        expected: Expectation<'tcx>,
    ) -> Ty<'tcx> {
        let rcvr_t = self.check_expr(&rcvr);
        // no need to check for bot/err -- callee does that
        let rcvr_t = self.structurally_resolve_type(rcvr.span, rcvr_t);
        let span = segment.ident.span;

        let method = match self.lookup_method(rcvr_t, segment, span, expr, rcvr, args) {
            Ok(method) => {
                // We could add a "consider `foo::<params>`" suggestion here, but I wasn't able to
                // trigger this codepath causing `structurally_resolve_type` to emit an error.

                self.enforce_context_effects(expr.hir_id, expr.span, method.def_id, method.substs);
                self.write_method_call(expr.hir_id, method);
                Ok(method)
            }
            Err(error) => {
                if segment.ident.name != kw::Empty {
                    if let Some(mut err) = self.report_method_error(
                        span,
                        rcvr_t,
                        segment.ident,
                        SelfSource::MethodCall(rcvr),
                        error,
                        Some((rcvr, args)),
                        expected,
                        false,
                    ) {
                        err.emit();
                    }
                }
                Err(())
            }
        };

        // Call the generic checker.
        self.check_method_argument_types(span, expr, method, &args, DontTupleArguments, expected)
    }

    fn check_expr_cast(
        &self,
        e: &'tcx hir::Expr<'tcx>,
        t: &'tcx hir::Ty<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        // Find the type of `e`. Supply hints based on the type we are casting to,
        // if appropriate.
        let t_cast = self.to_ty_saving_user_provided_ty(t);
        let t_cast = self.resolve_vars_if_possible(t_cast);
        let t_expr = self.check_expr_with_expectation(e, ExpectCastableToType(t_cast));
        let t_expr = self.resolve_vars_if_possible(t_expr);

        // Eagerly check for some obvious errors.
        if let Err(guar) = (t_expr, t_cast).error_reported() {
            Ty::new_error(self.tcx, guar)
        } else {
            // Defer other checks until we're done type checking.
            let mut deferred_cast_checks = self.deferred_cast_checks.borrow_mut();
            match cast::CastCheck::new(
                self,
                e,
                t_expr,
                t_cast,
                t.span,
                expr.span,
                self.param_env.constness(),
            ) {
                Ok(cast_check) => {
                    debug!(
                        "check_expr_cast: deferring cast from {:?} to {:?}: {:?}",
                        t_cast, t_expr, cast_check,
                    );
                    deferred_cast_checks.push(cast_check);
                    t_cast
                }
                Err(guar) => Ty::new_error(self.tcx, guar),
            }
        }
    }

    fn check_expr_array(
        &self,
        args: &'tcx [hir::Expr<'tcx>],
        expected: Expectation<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let element_ty = if !args.is_empty() {
            let coerce_to = expected
                .to_option(self)
                .and_then(|uty| match *uty.kind() {
                    ty::Array(ty, _) | ty::Slice(ty) => Some(ty),
                    _ => None,
                })
                .unwrap_or_else(|| {
                    self.next_ty_var(TypeVariableOrigin {
                        kind: TypeVariableOriginKind::TypeInference,
                        span: expr.span,
                    })
                });
            let mut coerce = CoerceMany::with_coercion_sites(coerce_to, args);
            assert_eq!(self.diverges.get(), Diverges::Maybe);
            for e in args {
                let e_ty = self.check_expr_with_hint(e, coerce_to);
                let cause = self.misc(e.span);
                coerce.coerce(self, &cause, e, e_ty);
            }
            coerce.complete(self)
        } else {
            self.next_ty_var(TypeVariableOrigin {
                kind: TypeVariableOriginKind::TypeInference,
                span: expr.span,
            })
        };
        let array_len = args.len() as u64;
        self.suggest_array_len(expr, array_len);
        Ty::new_array(self.tcx, element_ty, array_len)
    }

    fn suggest_array_len(&self, expr: &'tcx hir::Expr<'tcx>, array_len: u64) {
        let parent_node = self.tcx.hir().parent_iter(expr.hir_id).find(|(_, node)| {
            !matches!(node, hir::Node::Expr(hir::Expr { kind: hir::ExprKind::AddrOf(..), .. }))
        });
        let Some((_,
            hir::Node::Local(hir::Local { ty: Some(ty), .. })
            | hir::Node::Item(hir::Item { kind: hir::ItemKind::Const(ty, _), .. }))
        ) = parent_node else {
            return
        };
        if let hir::TyKind::Array(_, length) = ty.peel_refs().kind
            && let hir::ArrayLen::Body(hir::AnonConst { hir_id, .. }) = length
            && let Some(span) = self.tcx.hir().opt_span(hir_id)
        {
            match self.tcx.sess.diagnostic().steal_diagnostic(span, StashKey::UnderscoreForArrayLengths) {
                Some(mut err) => {
                    err.span_suggestion(
                        span,
                        "consider specifying the array length",
                        array_len,
                        Applicability::MaybeIncorrect,
                    );
                    err.emit();
                }
                None => ()
            }
        }
    }

    fn check_expr_const_block(
        &self,
        block: &'tcx hir::ConstBlock,
        expected: Expectation<'tcx>,
        _expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let body = self.tcx.hir().body(block.body);

        // Create a new function context.
        let def_id = block.def_id;
        let fcx = FnCtxt::new(self, self.param_env.with_const(), def_id);
        crate::GatherLocalsVisitor::new(&fcx).visit_body(body);

        let ty = fcx.check_expr_with_expectation(&body.value, expected);
        fcx.require_type_is_sized(ty, body.value.span, traits::ConstSized);
        fcx.write_ty(block.hir_id, ty);
        ty
    }

    fn check_expr_repeat(
        &self,
        element: &'tcx hir::Expr<'tcx>,
        count: &'tcx hir::ArrayLen,
        expected: Expectation<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let tcx = self.tcx;
        let count = self.array_length_to_const(count);
        if let Some(count) = count.try_eval_target_usize(tcx, self.param_env) {
            self.suggest_array_len(expr, count);
        }

        let uty = match expected {
            ExpectHasType(uty) => match *uty.kind() {
                ty::Array(ty, _) | ty::Slice(ty) => Some(ty),
                _ => None,
            },
            _ => None,
        };

        let (element_ty, t) = match uty {
            Some(uty) => {
                self.check_expr_coercible_to_type(&element, uty, None);
                (uty, uty)
            }
            None => {
                let ty = self.next_ty_var(TypeVariableOrigin {
                    kind: TypeVariableOriginKind::MiscVariable,
                    span: element.span,
                });
                let element_ty = self.check_expr_has_type_or_error(&element, ty, |_| {});
                (element_ty, ty)
            }
        };

        if let Err(guar) = element_ty.error_reported() {
            return Ty::new_error(tcx, guar);
        }

        self.check_repeat_element_needs_copy_bound(element, count, element_ty);

        self.register_wf_obligation(
            Ty::new_array_with_const_len(tcx, t, count).into(),
            expr.span,
            traits::WellFormed(None),
        );

        Ty::new_array_with_const_len(tcx, t, count)
    }

    fn check_repeat_element_needs_copy_bound(
        &self,
        element: &hir::Expr<'_>,
        count: ty::Const<'tcx>,
        element_ty: Ty<'tcx>,
    ) {
        let tcx = self.tcx;
        // Actual constants as the repeat element get inserted repeatedly instead of getting copied via Copy.
        match &element.kind {
            hir::ExprKind::ConstBlock(..) => return,
            hir::ExprKind::Path(qpath) => {
                let res = self.typeck_results.borrow().qpath_res(qpath, element.hir_id);
                if let Res::Def(DefKind::Const | DefKind::AssocConst | DefKind::AnonConst, _) = res
                {
                    return;
                }
            }
            _ => {}
        }
        // If someone calls a const fn, they can extract that call out into a separate constant (or a const
        // block in the future), so we check that to tell them that in the diagnostic. Does not affect typeck.
        let is_const_fn = match element.kind {
            hir::ExprKind::Call(func, _args) => match *self.node_ty(func.hir_id).kind() {
                ty::FnDef(def_id, _) => tcx.is_const_fn(def_id),
                _ => false,
            },
            _ => false,
        };

        // If the length is 0, we don't create any elements, so we don't copy any. If the length is 1, we
        // don't copy that one element, we move it. Only check for Copy if the length is larger.
        if count.try_eval_target_usize(tcx, self.param_env).map_or(true, |len| len > 1) {
            let lang_item = self.tcx.require_lang_item(LangItem::Copy, None);
            let code = traits::ObligationCauseCode::RepeatElementCopy { is_const_fn };
            self.require_type_meets(element_ty, element.span, code, lang_item);
        }
    }

    fn check_expr_tuple(
        &self,
        elts: &'tcx [hir::Expr<'tcx>],
        expected: Expectation<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let flds = expected.only_has_type(self).and_then(|ty| {
            let ty = self.resolve_vars_with_obligations(ty);
            match ty.kind() {
                ty::Tuple(flds) => Some(&flds[..]),
                _ => None,
            }
        });

        let elt_ts_iter = elts.iter().enumerate().map(|(i, e)| match flds {
            Some(fs) if i < fs.len() => {
                let ety = fs[i];
                self.check_expr_coercible_to_type(&e, ety, None);
                ety
            }
            _ => self.check_expr_with_expectation(&e, NoExpectation),
        });
        let tuple = Ty::new_tup_from_iter(self.tcx, elt_ts_iter);
        if let Err(guar) = tuple.error_reported() {
            Ty::new_error(self.tcx, guar)
        } else {
            self.require_type_is_sized(tuple, expr.span, traits::TupleInitializerSized);
            tuple
        }
    }

    fn check_expr_struct(
        &self,
        expr: &hir::Expr<'_>,
        expected: Expectation<'tcx>,
        qpath: &QPath<'_>,
        fields: &'tcx [hir::ExprField<'tcx>],
        base_expr: &'tcx Option<&'tcx hir::Expr<'tcx>>,
    ) -> Ty<'tcx> {
        // Find the relevant variant
        let (variant, adt_ty) = match self.check_struct_path(qpath, expr.hir_id) {
            Ok(data) => data,
            Err(guar) => {
                self.check_struct_fields_on_error(fields, base_expr);
                return Ty::new_error(self.tcx, guar);
            }
        };

        // Prohibit struct expressions when non-exhaustive flag is set.
        let adt = adt_ty.ty_adt_def().expect("`check_struct_path` returned non-ADT type");
        if !adt.did().is_local() && variant.is_field_list_non_exhaustive() {
            self.tcx
                .sess
                .emit_err(StructExprNonExhaustive { span: expr.span, what: adt.variant_descr() });
        }

        self.check_expr_struct_fields(
            adt_ty,
            expected,
            expr.hir_id,
            qpath.span(),
            variant,
            fields,
            base_expr,
            expr.span,
        );

        self.require_type_is_sized(adt_ty, expr.span, traits::StructInitializerSized);
        adt_ty
    }

    fn check_expr_struct_fields(
        &self,
        adt_ty: Ty<'tcx>,
        expected: Expectation<'tcx>,
        expr_id: hir::HirId,
        span: Span,
        variant: &'tcx ty::VariantDef,
        ast_fields: &'tcx [hir::ExprField<'tcx>],
        base_expr: &'tcx Option<&'tcx hir::Expr<'tcx>>,
        expr_span: Span,
    ) {
        let tcx = self.tcx;

        let expected_inputs =
            self.expected_inputs_for_expected_output(span, expected, adt_ty, &[adt_ty]);
        let adt_ty_hint = if let Some(expected_inputs) = expected_inputs {
            expected_inputs.get(0).cloned().unwrap_or(adt_ty)
        } else {
            adt_ty
        };
        // re-link the regions that EIfEO can erase.
        self.demand_eqtype(span, adt_ty_hint, adt_ty);

        let ty::Adt(adt, substs) = adt_ty.kind() else {
            span_bug!(span, "non-ADT passed to check_expr_struct_fields");
        };
        let adt_kind = adt.adt_kind();

        let mut remaining_fields = variant
            .fields
            .iter_enumerated()
            .map(|(i, field)| (field.ident(tcx).normalize_to_macros_2_0(), (i, field)))
            .collect::<FxHashMap<_, _>>();

        let mut seen_fields = FxHashMap::default();

        let mut error_happened = false;

        // Type-check each field.
        for (idx, field) in ast_fields.iter().enumerate() {
            let ident = tcx.adjust_ident(field.ident, variant.def_id);
            let field_type = if let Some((i, v_field)) = remaining_fields.remove(&ident) {
                seen_fields.insert(ident, field.span);
                self.write_field_index(field.hir_id, i);

                // We don't look at stability attributes on
                // struct-like enums (yet...), but it's definitely not
                // a bug to have constructed one.
                if adt_kind != AdtKind::Enum {
                    tcx.check_stability(v_field.did, Some(expr_id), field.span, None);
                }

                self.field_ty(field.span, v_field, substs)
            } else {
                error_happened = true;
                let guar = if let Some(prev_span) = seen_fields.get(&ident) {
                    tcx.sess.emit_err(FieldMultiplySpecifiedInInitializer {
                        span: field.ident.span,
                        prev_span: *prev_span,
                        ident,
                    })
                } else {
                    self.report_unknown_field(
                        adt_ty,
                        variant,
                        field,
                        ast_fields,
                        adt.variant_descr(),
                        expr_span,
                    )
                };

                Ty::new_error(tcx, guar)
            };

            // Make sure to give a type to the field even if there's
            // an error, so we can continue type-checking.
            let ty = self.check_expr_with_hint(&field.expr, field_type);
            let (_, diag) =
                self.demand_coerce_diag(&field.expr, ty, field_type, None, AllowTwoPhase::No);

            if let Some(mut diag) = diag {
                if idx == ast_fields.len() - 1 {
                    if remaining_fields.is_empty() {
                        self.suggest_fru_from_range(field, variant, substs, &mut diag);
                        diag.emit();
                    } else {
                        diag.stash(field.span, StashKey::MaybeFruTypo);
                    }
                } else {
                    diag.emit();
                }
            }
        }

        // Make sure the programmer specified correct number of fields.
        if adt_kind == AdtKind::Union {
            if ast_fields.len() != 1 {
                struct_span_err!(
                    tcx.sess,
                    span,
                    E0784,
                    "union expressions should have exactly one field",
                )
                .emit();
            }
        }

        // If check_expr_struct_fields hit an error, do not attempt to populate
        // the fields with the base_expr. This could cause us to hit errors later
        // when certain fields are assumed to exist that in fact do not.
        if error_happened {
            if let Some(base_expr) = base_expr {
                self.check_expr(base_expr);
            }
            return;
        }

        if let Some(base_expr) = base_expr {
            // FIXME: We are currently creating two branches here in order to maintain
            // consistency. But they should be merged as much as possible.
            let fru_tys = if self.tcx.features().type_changing_struct_update {
                if adt.is_struct() {
                    // Make some fresh substitutions for our ADT type.
                    let fresh_substs = self.fresh_substs_for_item(base_expr.span, adt.did());
                    // We do subtyping on the FRU fields first, so we can
                    // learn exactly what types we expect the base expr
                    // needs constrained to be compatible with the struct
                    // type we expect from the expectation value.
                    let fru_tys = variant
                        .fields
                        .iter()
                        .map(|f| {
                            let fru_ty = self.normalize(
                                expr_span,
                                self.field_ty(base_expr.span, f, fresh_substs),
                            );
                            let ident = self.tcx.adjust_ident(f.ident(self.tcx), variant.def_id);
                            if let Some(_) = remaining_fields.remove(&ident) {
                                let target_ty = self.field_ty(base_expr.span, f, substs);
                                let cause = self.misc(base_expr.span);
                                match self.at(&cause, self.param_env).sup(
                                    DefineOpaqueTypes::No,
                                    target_ty,
                                    fru_ty,
                                ) {
                                    Ok(InferOk { obligations, value: () }) => {
                                        self.register_predicates(obligations)
                                    }
                                    Err(_) => {
                                        // This should never happen, since we're just subtyping the
                                        // remaining_fields, but it's fine to emit this, I guess.
                                        self.err_ctxt()
                                            .report_mismatched_types(
                                                &cause,
                                                target_ty,
                                                fru_ty,
                                                FieldMisMatch(variant.name, ident.name),
                                            )
                                            .emit();
                                    }
                                }
                            }
                            self.resolve_vars_if_possible(fru_ty)
                        })
                        .collect();
                    // The use of fresh substs that we have subtyped against
                    // our base ADT type's fields allows us to guide inference
                    // along so that, e.g.
                    // ```
                    // MyStruct<'a, F1, F2, const C: usize> {
                    //     f: F1,
                    //     // Other fields that reference `'a`, `F2`, and `C`
                    // }
                    //
                    // let x = MyStruct {
                    //    f: 1usize,
                    //    ..other_struct
                    // };
                    // ```
                    // will have the `other_struct` expression constrained to
                    // `MyStruct<'a, _, F2, C>`, as opposed to just `_`...
                    // This is important to allow coercions to happen in
                    // `other_struct` itself. See `coerce-in-base-expr.rs`.
                    let fresh_base_ty = Ty::new_adt(self.tcx, *adt, fresh_substs);
                    self.check_expr_has_type_or_error(
                        base_expr,
                        self.resolve_vars_if_possible(fresh_base_ty),
                        |_| {},
                    );
                    fru_tys
                } else {
                    // Check the base_expr, regardless of a bad expected adt_ty, so we can get
                    // type errors on that expression, too.
                    self.check_expr(base_expr);
                    self.tcx
                        .sess
                        .emit_err(FunctionalRecordUpdateOnNonStruct { span: base_expr.span });
                    return;
                }
            } else {
                self.check_expr_has_type_or_error(base_expr, adt_ty, |_| {
                    let base_ty = self.typeck_results.borrow().expr_ty(*base_expr);
                    let same_adt = matches!((adt_ty.kind(), base_ty.kind()),
                        (ty::Adt(adt, _), ty::Adt(base_adt, _)) if adt == base_adt);
                    if self.tcx.sess.is_nightly_build() && same_adt {
                        feature_err(
                            &self.tcx.sess.parse_sess,
                            sym::type_changing_struct_update,
                            base_expr.span,
                            "type changing struct updating is experimental",
                        )
                        .emit();
                    }
                });
                match adt_ty.kind() {
                    ty::Adt(adt, substs) if adt.is_struct() => variant
                        .fields
                        .iter()
                        .map(|f| self.normalize(expr_span, f.ty(self.tcx, substs)))
                        .collect(),
                    _ => {
                        self.tcx
                            .sess
                            .emit_err(FunctionalRecordUpdateOnNonStruct { span: base_expr.span });
                        return;
                    }
                }
            };
            self.typeck_results.borrow_mut().fru_field_types_mut().insert(expr_id, fru_tys);
        } else if adt_kind != AdtKind::Union && !remaining_fields.is_empty() {
            debug!(?remaining_fields);
            let private_fields: Vec<&ty::FieldDef> = variant
                .fields
                .iter()
                .filter(|field| !field.vis.is_accessible_from(tcx.parent_module(expr_id), tcx))
                .collect();

            if !private_fields.is_empty() {
                self.report_private_fields(adt_ty, span, private_fields, ast_fields);
            } else {
                self.report_missing_fields(
                    adt_ty,
                    span,
                    remaining_fields,
                    variant,
                    ast_fields,
                    substs,
                );
            }
        }
    }

    fn check_struct_fields_on_error(
        &self,
        fields: &'tcx [hir::ExprField<'tcx>],
        base_expr: &'tcx Option<&'tcx hir::Expr<'tcx>>,
    ) {
        for field in fields {
            self.check_expr(&field.expr);
        }
        if let Some(base) = *base_expr {
            self.check_expr(&base);
        }
    }

    /// Report an error for a struct field expression when there are fields which aren't provided.
    ///
    /// ```text
    /// error: missing field `you_can_use_this_field` in initializer of `foo::Foo`
    ///  --> src/main.rs:8:5
    ///   |
    /// 8 |     foo::Foo {};
    ///   |     ^^^^^^^^ missing `you_can_use_this_field`
    ///
    /// error: aborting due to previous error
    /// ```
    fn report_missing_fields(
        &self,
        adt_ty: Ty<'tcx>,
        span: Span,
        remaining_fields: FxHashMap<Ident, (FieldIdx, &ty::FieldDef)>,
        variant: &'tcx ty::VariantDef,
        ast_fields: &'tcx [hir::ExprField<'tcx>],
        substs: SubstsRef<'tcx>,
    ) {
        let len = remaining_fields.len();

        let mut displayable_field_names: Vec<&str> =
            remaining_fields.keys().map(|ident| ident.as_str()).collect();
        // sorting &str primitives here, sort_unstable is ok
        displayable_field_names.sort_unstable();

        let mut truncated_fields_error = String::new();
        let remaining_fields_names = match &displayable_field_names[..] {
            [field1] => format!("`{}`", field1),
            [field1, field2] => format!("`{field1}` and `{field2}`"),
            [field1, field2, field3] => format!("`{field1}`, `{field2}` and `{field3}`"),
            _ => {
                truncated_fields_error =
                    format!(" and {} other field{}", len - 3, pluralize!(len - 3));
                displayable_field_names
                    .iter()
                    .take(3)
                    .map(|n| format!("`{n}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        };

        let mut err = struct_span_err!(
            self.tcx.sess,
            span,
            E0063,
            "missing field{} {}{} in initializer of `{}`",
            pluralize!(len),
            remaining_fields_names,
            truncated_fields_error,
            adt_ty
        );
        err.span_label(span, format!("missing {remaining_fields_names}{truncated_fields_error}"));

        if let Some(last) = ast_fields.last() {
            self.suggest_fru_from_range(last, variant, substs, &mut err);
        }

        err.emit();
    }

    /// If the last field is a range literal, but it isn't supposed to be, then they probably
    /// meant to use functional update syntax.
    fn suggest_fru_from_range(
        &self,
        last_expr_field: &hir::ExprField<'tcx>,
        variant: &ty::VariantDef,
        substs: SubstsRef<'tcx>,
        err: &mut Diagnostic,
    ) {
        // I don't use 'is_range_literal' because only double-sided, half-open ranges count.
        if let ExprKind::Struct(
                QPath::LangItem(LangItem::Range, ..),
                [range_start, range_end],
                _,
            ) = last_expr_field.expr.kind
            && let variant_field =
                variant.fields.iter().find(|field| field.ident(self.tcx) == last_expr_field.ident)
            && let range_def_id = self.tcx.lang_items().range_struct()
            && variant_field
                .and_then(|field| field.ty(self.tcx, substs).ty_adt_def())
                .map(|adt| adt.did())
                != range_def_id
        {
            // Suppress any range expr type mismatches
            if let Some(mut diag) = self
                .tcx
                .sess
                .diagnostic()
                .steal_diagnostic(last_expr_field.span, StashKey::MaybeFruTypo)
            {
                diag.delay_as_bug();
            }

            // Use a (somewhat arbitrary) filtering heuristic to avoid printing
            // expressions that are either too long, or have control character
            //such as newlines in them.
            let expr = self
                .tcx
                .sess
                .source_map()
                .span_to_snippet(range_end.expr.span)
                .ok()
                .filter(|s| s.len() < 25 && !s.contains(|c: char| c.is_control()));

            let fru_span = self
                .tcx
                .sess
                .source_map()
                .span_extend_while(range_start.span, |c| c.is_whitespace())
                .unwrap_or(range_start.span).shrink_to_hi().to(range_end.span);

            err.subdiagnostic(TypeMismatchFruTypo {
                expr_span: range_start.span,
                fru_span,
                expr,
            });
        }
    }

    /// Report an error for a struct field expression when there are invisible fields.
    ///
    /// ```text
    /// error: cannot construct `Foo` with struct literal syntax due to private fields
    ///  --> src/main.rs:8:5
    ///   |
    /// 8 |     foo::Foo {};
    ///   |     ^^^^^^^^
    ///
    /// error: aborting due to previous error
    /// ```
    fn report_private_fields(
        &self,
        adt_ty: Ty<'tcx>,
        span: Span,
        private_fields: Vec<&ty::FieldDef>,
        used_fields: &'tcx [hir::ExprField<'tcx>],
    ) {
        let mut err =
            self.tcx.sess.struct_span_err(
                span,
                format!(
                    "cannot construct `{adt_ty}` with struct literal syntax due to private fields",
                ),
            );
        let (used_private_fields, remaining_private_fields): (
            Vec<(Symbol, Span, bool)>,
            Vec<(Symbol, Span, bool)>,
        ) = private_fields
            .iter()
            .map(|field| {
                match used_fields.iter().find(|used_field| field.name == used_field.ident.name) {
                    Some(used_field) => (field.name, used_field.span, true),
                    None => (field.name, self.tcx.def_span(field.did), false),
                }
            })
            .partition(|field| field.2);
        err.span_labels(used_private_fields.iter().map(|(_, span, _)| *span), "private field");
        if !remaining_private_fields.is_empty() {
            let remaining_private_fields_len = remaining_private_fields.len();
            let names = match &remaining_private_fields
                .iter()
                .map(|(name, _, _)| name)
                .collect::<Vec<_>>()[..]
            {
                _ if remaining_private_fields_len > 6 => String::new(),
                [name] => format!("`{name}` "),
                [names @ .., last] => {
                    let names = names.iter().map(|name| format!("`{name}`")).collect::<Vec<_>>();
                    format!("{} and `{last}` ", names.join(", "))
                }
                [] => unreachable!(),
            };
            err.note(format!(
                "... and other private field{s} {names}that {were} not provided",
                s = pluralize!(remaining_private_fields_len),
                were = pluralize!("was", remaining_private_fields_len),
            ));
        }
        err.emit();
    }

    fn report_unknown_field(
        &self,
        ty: Ty<'tcx>,
        variant: &'tcx ty::VariantDef,
        field: &hir::ExprField<'_>,
        skip_fields: &[hir::ExprField<'_>],
        kind_name: &str,
        expr_span: Span,
    ) -> ErrorGuaranteed {
        if variant.is_recovered() {
            let guar = self
                .tcx
                .sess
                .delay_span_bug(expr_span, "parser recovered but no error was emitted");
            self.set_tainted_by_errors(guar);
            return guar;
        }
        let mut err = self.err_ctxt().type_error_struct_with_diag(
            field.ident.span,
            |actual| match ty.kind() {
                ty::Adt(adt, ..) if adt.is_enum() => struct_span_err!(
                    self.tcx.sess,
                    field.ident.span,
                    E0559,
                    "{} `{}::{}` has no field named `{}`",
                    kind_name,
                    actual,
                    variant.name,
                    field.ident
                ),
                _ => struct_span_err!(
                    self.tcx.sess,
                    field.ident.span,
                    E0560,
                    "{} `{}` has no field named `{}`",
                    kind_name,
                    actual,
                    field.ident
                ),
            },
            ty,
        );

        let variant_ident_span = self.tcx.def_ident_span(variant.def_id).unwrap();
        match variant.ctor_kind() {
            Some(CtorKind::Fn) => match ty.kind() {
                ty::Adt(adt, ..) if adt.is_enum() => {
                    err.span_label(
                        variant_ident_span,
                        format!(
                            "`{adt}::{variant}` defined here",
                            adt = ty,
                            variant = variant.name,
                        ),
                    );
                    err.span_label(field.ident.span, "field does not exist");
                    err.span_suggestion_verbose(
                        expr_span,
                        format!(
                            "`{adt}::{variant}` is a tuple {kind_name}, use the appropriate syntax",
                            adt = ty,
                            variant = variant.name,
                        ),
                        format!(
                            "{adt}::{variant}(/* fields */)",
                            adt = ty,
                            variant = variant.name,
                        ),
                        Applicability::HasPlaceholders,
                    );
                }
                _ => {
                    err.span_label(variant_ident_span, format!("`{adt}` defined here", adt = ty));
                    err.span_label(field.ident.span, "field does not exist");
                    err.span_suggestion_verbose(
                        expr_span,
                        format!(
                            "`{adt}` is a tuple {kind_name}, use the appropriate syntax",
                            adt = ty,
                            kind_name = kind_name,
                        ),
                        format!("{adt}(/* fields */)", adt = ty),
                        Applicability::HasPlaceholders,
                    );
                }
            },
            _ => {
                // prevent all specified fields from being suggested
                let skip_fields: Vec<_> = skip_fields.iter().map(|x| x.ident.name).collect();
                if let Some(field_name) =
                    self.suggest_field_name(variant, field.ident.name, &skip_fields, expr_span)
                {
                    err.span_suggestion(
                        field.ident.span,
                        "a field with a similar name exists",
                        field_name,
                        Applicability::MaybeIncorrect,
                    );
                } else {
                    match ty.kind() {
                        ty::Adt(adt, ..) => {
                            if adt.is_enum() {
                                err.span_label(
                                    field.ident.span,
                                    format!("`{}::{}` does not have this field", ty, variant.name),
                                );
                            } else {
                                err.span_label(
                                    field.ident.span,
                                    format!("`{ty}` does not have this field"),
                                );
                            }
                            let mut available_field_names =
                                self.available_field_names(variant, expr_span);
                            available_field_names
                                .retain(|name| skip_fields.iter().all(|skip| name != skip));
                            if available_field_names.is_empty() {
                                err.note("all struct fields are already assigned");
                            } else {
                                err.note(format!(
                                    "available fields are: {}",
                                    self.name_series_display(available_field_names)
                                ));
                            }
                        }
                        _ => bug!("non-ADT passed to report_unknown_field"),
                    }
                };
            }
        }
        err.emit()
    }

    // Return a hint about the closest match in field names
    fn suggest_field_name(
        &self,
        variant: &'tcx ty::VariantDef,
        field: Symbol,
        skip: &[Symbol],
        // The span where stability will be checked
        span: Span,
    ) -> Option<Symbol> {
        let names = variant
            .fields
            .iter()
            .filter_map(|field| {
                // ignore already set fields and private fields from non-local crates
                // and unstable fields.
                if skip.iter().any(|&x| x == field.name)
                    || (!variant.def_id.is_local() && !field.vis.is_public())
                    || matches!(
                        self.tcx.eval_stability(field.did, None, span, None),
                        stability::EvalResult::Deny { .. }
                    )
                {
                    None
                } else {
                    Some(field.name)
                }
            })
            .collect::<Vec<Symbol>>();

        find_best_match_for_name(&names, field, None)
    }

    fn available_field_names(
        &self,
        variant: &'tcx ty::VariantDef,
        access_span: Span,
    ) -> Vec<Symbol> {
        let body_owner_hir_id = self.tcx.hir().local_def_id_to_hir_id(self.body_id);
        variant
            .fields
            .iter()
            .filter(|field| {
                let def_scope = self
                    .tcx
                    .adjust_ident_and_get_scope(
                        field.ident(self.tcx),
                        variant.def_id,
                        body_owner_hir_id,
                    )
                    .1;
                field.vis.is_accessible_from(def_scope, self.tcx)
                    && !matches!(
                        self.tcx.eval_stability(field.did, None, access_span, None),
                        stability::EvalResult::Deny { .. }
                    )
            })
            .filter(|field| !self.tcx.is_doc_hidden(field.did))
            .map(|field| field.name)
            .collect()
    }

    fn name_series_display(&self, names: Vec<Symbol>) -> String {
        // dynamic limit, to never omit just one field
        let limit = if names.len() == 6 { 6 } else { 5 };
        let mut display =
            names.iter().take(limit).map(|n| format!("`{}`", n)).collect::<Vec<_>>().join(", ");
        if names.len() > limit {
            display = format!("{} ... and {} others", display, names.len() - limit);
        }
        display
    }

    // Check field access expressions
    fn check_field(
        &self,
        expr: &'tcx hir::Expr<'tcx>,
        base: &'tcx hir::Expr<'tcx>,
        field: Ident,
        expected: Expectation<'tcx>,
    ) -> Ty<'tcx> {
        debug!("check_field(expr: {:?}, base: {:?}, field: {:?})", expr, base, field);
        let base_ty = self.check_expr(base);
        let base_ty = self.structurally_resolve_type(base.span, base_ty);
        let mut private_candidate = None;
        let mut autoderef = self.autoderef(expr.span, base_ty);
        while let Some((deref_base_ty, _)) = autoderef.next() {
            debug!("deref_base_ty: {:?}", deref_base_ty);
            match deref_base_ty.kind() {
                ty::Adt(base_def, substs) if !base_def.is_enum() => {
                    debug!("struct named {:?}", deref_base_ty);
                    let body_hir_id = self.tcx.hir().local_def_id_to_hir_id(self.body_id);
                    let (ident, def_scope) =
                        self.tcx.adjust_ident_and_get_scope(field, base_def.did(), body_hir_id);
                    let fields = &base_def.non_enum_variant().fields;
                    if let Some((index, field)) = fields
                        .iter_enumerated()
                        .find(|(_, f)| f.ident(self.tcx).normalize_to_macros_2_0() == ident)
                    {
                        let field_ty = self.field_ty(expr.span, field, substs);
                        // Save the index of all fields regardless of their visibility in case
                        // of error recovery.
                        self.write_field_index(expr.hir_id, index);
                        let adjustments = self.adjust_steps(&autoderef);
                        if field.vis.is_accessible_from(def_scope, self.tcx) {
                            self.apply_adjustments(base, adjustments);
                            self.register_predicates(autoderef.into_obligations());

                            self.tcx.check_stability(field.did, Some(expr.hir_id), expr.span, None);
                            return field_ty;
                        }
                        private_candidate = Some((adjustments, base_def.did()));
                    }
                }
                ty::Tuple(tys) => {
                    if let Ok(index) = field.as_str().parse::<usize>() {
                        if field.name == sym::integer(index) {
                            if let Some(&field_ty) = tys.get(index) {
                                let adjustments = self.adjust_steps(&autoderef);
                                self.apply_adjustments(base, adjustments);
                                self.register_predicates(autoderef.into_obligations());

                                self.write_field_index(expr.hir_id, FieldIdx::from_usize(index));
                                return field_ty;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        self.structurally_resolve_type(autoderef.span(), autoderef.final_ty(false));

        if let Some((adjustments, did)) = private_candidate {
            // (#90483) apply adjustments to avoid ExprUseVisitor from
            // creating erroneous projection.
            self.apply_adjustments(base, adjustments);
            let guar = self.ban_private_field_access(
                expr,
                base_ty,
                field,
                did,
                expected.only_has_type(self),
            );
            return Ty::new_error(self.tcx(), guar);
        }

        let guar = if field.name == kw::Empty {
            self.tcx.sess.delay_span_bug(field.span, "field name with no name")
        } else if self.method_exists(
            field,
            base_ty,
            expr.hir_id,
            true,
            expected.only_has_type(self),
        ) {
            self.ban_take_value_of_method(expr, base_ty, field)
        } else if !base_ty.is_primitive_ty() {
            self.ban_nonexisting_field(field, base, expr, base_ty)
        } else {
            let field_name = field.to_string();
            let mut err = type_error_struct!(
                self.tcx().sess,
                field.span,
                base_ty,
                E0610,
                "`{base_ty}` is a primitive type and therefore doesn't have fields",
            );
            let is_valid_suffix = |field: &str| {
                if field == "f32" || field == "f64" {
                    return true;
                }
                let mut chars = field.chars().peekable();
                match chars.peek() {
                    Some('e') | Some('E') => {
                        chars.next();
                        if let Some(c) = chars.peek()
                            && !c.is_numeric() && *c != '-' && *c != '+'
                        {
                            return false;
                        }
                        while let Some(c) = chars.peek() {
                            if !c.is_numeric() {
                                break;
                            }
                            chars.next();
                        }
                    }
                    _ => (),
                }
                let suffix = chars.collect::<String>();
                suffix.is_empty() || suffix == "f32" || suffix == "f64"
            };
            let maybe_partial_suffix = |field: &str| -> Option<&str> {
                let first_chars = ['f', 'l'];
                if field.len() >= 1
                    && field.to_lowercase().starts_with(first_chars)
                    && field[1..].chars().all(|c| c.is_ascii_digit())
                {
                    if field.to_lowercase().starts_with(['f']) { Some("f32") } else { Some("f64") }
                } else {
                    None
                }
            };
            if let ty::Infer(ty::IntVar(_)) = base_ty.kind()
                && let ExprKind::Lit(Spanned {
                    node: ast::LitKind::Int(_, ast::LitIntType::Unsuffixed),
                    ..
                }) = base.kind
                && !base.span.from_expansion()
            {
                if is_valid_suffix(&field_name) {
                    err.span_suggestion_verbose(
                        field.span.shrink_to_lo(),
                        "if intended to be a floating point literal, consider adding a `0` after the period",
                        '0',
                        Applicability::MaybeIncorrect,
                    );
                } else if let Some(correct_suffix) = maybe_partial_suffix(&field_name) {
                    err.span_suggestion_verbose(
                        field.span,
                        format!("if intended to be a floating point literal, consider adding a `0` after the period and a `{correct_suffix}` suffix"),
                        format!("0{correct_suffix}"),
                        Applicability::MaybeIncorrect,
                    );
                }
            }
            err.emit()
        };

        Ty::new_error(self.tcx(), guar)
    }

    fn suggest_await_on_field_access(
        &self,
        err: &mut Diagnostic,
        field_ident: Ident,
        base: &'tcx hir::Expr<'tcx>,
        ty: Ty<'tcx>,
    ) {
        let Some(output_ty) = self.get_impl_future_output_ty(ty) else { return; };
        let mut add_label = true;
        if let ty::Adt(def, _) = output_ty.kind() {
            // no field access on enum type
            if !def.is_enum() {
                if def
                    .non_enum_variant()
                    .fields
                    .iter()
                    .any(|field| field.ident(self.tcx) == field_ident)
                {
                    add_label = false;
                    err.span_label(
                        field_ident.span,
                        "field not available in `impl Future`, but it is available in its `Output`",
                    );
                    err.span_suggestion_verbose(
                        base.span.shrink_to_hi(),
                        "consider `await`ing on the `Future` and access the field of its `Output`",
                        ".await",
                        Applicability::MaybeIncorrect,
                    );
                }
            }
        }
        if add_label {
            err.span_label(field_ident.span, format!("field not found in `{ty}`"));
        }
    }

    fn ban_nonexisting_field(
        &self,
        ident: Ident,
        base: &'tcx hir::Expr<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
        base_ty: Ty<'tcx>,
    ) -> ErrorGuaranteed {
        debug!(
            "ban_nonexisting_field: field={:?}, base={:?}, expr={:?}, base_ty={:?}",
            ident, base, expr, base_ty
        );
        let mut err = self.no_such_field_err(ident, base_ty, base.hir_id);

        match *base_ty.peel_refs().kind() {
            ty::Array(_, len) => {
                self.maybe_suggest_array_indexing(&mut err, expr, base, ident, len);
            }
            ty::RawPtr(..) => {
                self.suggest_first_deref_field(&mut err, expr, base, ident);
            }
            ty::Adt(def, _) if !def.is_enum() => {
                self.suggest_fields_on_recordish(&mut err, def, ident, expr.span);
            }
            ty::Param(param_ty) => {
                self.point_at_param_definition(&mut err, param_ty);
            }
            ty::Alias(ty::Opaque, _) => {
                self.suggest_await_on_field_access(&mut err, ident, base, base_ty.peel_refs());
            }
            _ => {}
        }

        self.suggest_fn_call(&mut err, base, base_ty, |output_ty| {
            if let ty::Adt(def, _) = output_ty.kind() && !def.is_enum() {
                def.non_enum_variant().fields.iter().any(|field| {
                    field.ident(self.tcx) == ident
                        && field.vis.is_accessible_from(expr.hir_id.owner.def_id, self.tcx)
                })
            } else if let ty::Tuple(tys) = output_ty.kind()
                && let Ok(idx) = ident.as_str().parse::<usize>()
            {
                idx < tys.len()
            } else {
                false
            }
        });

        if ident.name == kw::Await {
            // We know by construction that `<expr>.await` is either on Rust 2015
            // or results in `ExprKind::Await`. Suggest switching the edition to 2018.
            err.note("to `.await` a `Future`, switch to Rust 2018 or later");
            HelpUseLatestEdition::new().add_to_diagnostic(&mut err);
        }

        err.emit()
    }

    fn ban_private_field_access(
        &self,
        expr: &hir::Expr<'tcx>,
        expr_t: Ty<'tcx>,
        field: Ident,
        base_did: DefId,
        return_ty: Option<Ty<'tcx>>,
    ) -> ErrorGuaranteed {
        let mut err = self.private_field_err(field, base_did);

        // Also check if an accessible method exists, which is often what is meant.
        if self.method_exists(field, expr_t, expr.hir_id, false, return_ty)
            && !self.expr_in_place(expr.hir_id)
        {
            self.suggest_method_call(
                &mut err,
                format!("a method `{field}` also exists, call it with parentheses"),
                field,
                expr_t,
                expr,
                None,
            );
        }
        err.emit()
    }

    fn ban_take_value_of_method(
        &self,
        expr: &hir::Expr<'tcx>,
        expr_t: Ty<'tcx>,
        field: Ident,
    ) -> ErrorGuaranteed {
        let mut err = type_error_struct!(
            self.tcx().sess,
            field.span,
            expr_t,
            E0615,
            "attempted to take value of method `{field}` on type `{expr_t}`",
        );
        err.span_label(field.span, "method, not a field");
        let expr_is_call =
            if let hir::Node::Expr(hir::Expr { kind: ExprKind::Call(callee, _args), .. }) =
                self.tcx.hir().get_parent(expr.hir_id)
            {
                expr.hir_id == callee.hir_id
            } else {
                false
            };
        let expr_snippet =
            self.tcx.sess.source_map().span_to_snippet(expr.span).unwrap_or_default();
        let is_wrapped = expr_snippet.starts_with('(') && expr_snippet.ends_with(')');
        let after_open = expr.span.lo() + rustc_span::BytePos(1);
        let before_close = expr.span.hi() - rustc_span::BytePos(1);

        if expr_is_call && is_wrapped {
            err.multipart_suggestion(
                "remove wrapping parentheses to call the method",
                vec![
                    (expr.span.with_hi(after_open), String::new()),
                    (expr.span.with_lo(before_close), String::new()),
                ],
                Applicability::MachineApplicable,
            );
        } else if !self.expr_in_place(expr.hir_id) {
            // Suggest call parentheses inside the wrapping parentheses
            let span = if is_wrapped {
                expr.span.with_lo(after_open).with_hi(before_close)
            } else {
                expr.span
            };
            self.suggest_method_call(
                &mut err,
                "use parentheses to call the method",
                field,
                expr_t,
                expr,
                Some(span),
            );
        } else if let ty::RawPtr(ty_and_mut) = expr_t.kind()
            && let ty::Adt(adt_def, _) = ty_and_mut.ty.kind()
            && let ExprKind::Field(base_expr, _) = expr.kind
            && adt_def.variants().len() == 1
            && adt_def
                .variants()
                .iter()
                .next()
                .unwrap()
                .fields
                .iter()
                .any(|f| f.ident(self.tcx) == field)
        {
            err.multipart_suggestion(
                "to access the field, dereference first",
                vec![
                    (base_expr.span.shrink_to_lo(), "(*".to_string()),
                    (base_expr.span.shrink_to_hi(), ")".to_string()),
                ],
                Applicability::MaybeIncorrect,
            );
        } else {
            err.help("methods are immutable and cannot be assigned to");
        }

        err.emit()
    }

    fn point_at_param_definition(&self, err: &mut Diagnostic, param: ty::ParamTy) {
        let generics = self.tcx.generics_of(self.body_id);
        let generic_param = generics.type_param(&param, self.tcx);
        if let ty::GenericParamDefKind::Type { synthetic: true, .. } = generic_param.kind {
            return;
        }
        let param_def_id = generic_param.def_id;
        let param_hir_id = match param_def_id.as_local() {
            Some(x) => self.tcx.hir().local_def_id_to_hir_id(x),
            None => return,
        };
        let param_span = self.tcx.hir().span(param_hir_id);
        let param_name = self.tcx.hir().ty_param_name(param_def_id.expect_local());

        err.span_label(param_span, format!("type parameter '{param_name}' declared here"));
    }

    fn suggest_fields_on_recordish(
        &self,
        err: &mut Diagnostic,
        def: ty::AdtDef<'tcx>,
        field: Ident,
        access_span: Span,
    ) {
        if let Some(suggested_field_name) =
            self.suggest_field_name(def.non_enum_variant(), field.name, &[], access_span)
        {
            err.span_suggestion(
                field.span,
                "a field with a similar name exists",
                suggested_field_name,
                Applicability::MaybeIncorrect,
            );
        } else {
            err.span_label(field.span, "unknown field");
            let struct_variant_def = def.non_enum_variant();
            let field_names = self.available_field_names(struct_variant_def, access_span);
            if !field_names.is_empty() {
                err.note(format!(
                    "available fields are: {}",
                    self.name_series_display(field_names),
                ));
            }
        }
    }

    fn maybe_suggest_array_indexing(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        base: &hir::Expr<'_>,
        field: Ident,
        len: ty::Const<'tcx>,
    ) {
        if let (Some(len), Ok(user_index)) =
            (len.try_eval_target_usize(self.tcx, self.param_env), field.as_str().parse::<u64>())
            && let Ok(base) = self.tcx.sess.source_map().span_to_snippet(base.span)
        {
            let help = "instead of using tuple indexing, use array indexing";
            let suggestion = format!("{base}[{field}]");
            let applicability = if len < user_index {
                Applicability::MachineApplicable
            } else {
                Applicability::MaybeIncorrect
            };
            err.span_suggestion(expr.span, help, suggestion, applicability);
        }
    }

    fn suggest_first_deref_field(
        &self,
        err: &mut Diagnostic,
        expr: &hir::Expr<'_>,
        base: &hir::Expr<'_>,
        field: Ident,
    ) {
        if let Ok(base) = self.tcx.sess.source_map().span_to_snippet(base.span) {
            let msg = format!("`{base}` is a raw pointer; try dereferencing it");
            let suggestion = format!("(*{base}).{field}");
            err.span_suggestion(expr.span, msg, suggestion, Applicability::MaybeIncorrect);
        }
    }

    fn no_such_field_err(
        &self,
        field: Ident,
        expr_t: Ty<'tcx>,
        id: HirId,
    ) -> DiagnosticBuilder<'_, ErrorGuaranteed> {
        let span = field.span;
        debug!("no_such_field_err(span: {:?}, field: {:?}, expr_t: {:?})", span, field, expr_t);

        let mut err = type_error_struct!(
            self.tcx().sess,
            field.span,
            expr_t,
            E0609,
            "no field `{field}` on type `{expr_t}`",
        );

        // try to add a suggestion in case the field is a nested field of a field of the Adt
        let mod_id = self.tcx.parent_module(id).to_def_id();
        if let Some((fields, substs)) =
            self.get_field_candidates_considering_privacy(span, expr_t, mod_id)
        {
            let candidate_fields: Vec<_> = fields
                .filter_map(|candidate_field| {
                    self.check_for_nested_field_satisfying(
                        span,
                        &|candidate_field, _| candidate_field.ident(self.tcx()) == field,
                        candidate_field,
                        substs,
                        vec![],
                        mod_id,
                    )
                })
                .map(|mut field_path| {
                    field_path.pop();
                    field_path
                        .iter()
                        .map(|id| id.name.to_ident_string())
                        .collect::<Vec<String>>()
                        .join(".")
                })
                .collect::<Vec<_>>();

            let len = candidate_fields.len();
            if len > 0 {
                err.span_suggestions(
                    field.span.shrink_to_lo(),
                    format!(
                        "{} of the expressions' fields {} a field of the same name",
                        if len > 1 { "some" } else { "one" },
                        if len > 1 { "have" } else { "has" },
                    ),
                    candidate_fields.iter().map(|path| format!("{path}.")),
                    Applicability::MaybeIncorrect,
                );
            }
        }
        err
    }

    fn private_field_err(
        &self,
        field: Ident,
        base_did: DefId,
    ) -> DiagnosticBuilder<'_, ErrorGuaranteed> {
        let struct_path = self.tcx().def_path_str(base_did);
        let kind_name = self.tcx().def_descr(base_did);
        let mut err = struct_span_err!(
            self.tcx().sess,
            field.span,
            E0616,
            "field `{field}` of {kind_name} `{struct_path}` is private",
        );
        err.span_label(field.span, "private field");

        err
    }

    pub(crate) fn get_field_candidates_considering_privacy(
        &self,
        span: Span,
        base_ty: Ty<'tcx>,
        mod_id: DefId,
    ) -> Option<(impl Iterator<Item = &'tcx ty::FieldDef> + 'tcx, SubstsRef<'tcx>)> {
        debug!("get_field_candidates(span: {:?}, base_t: {:?}", span, base_ty);

        for (base_t, _) in self.autoderef(span, base_ty) {
            match base_t.kind() {
                ty::Adt(base_def, substs) if !base_def.is_enum() => {
                    let tcx = self.tcx;
                    let fields = &base_def.non_enum_variant().fields;
                    // Some struct, e.g. some that impl `Deref`, have all private fields
                    // because you're expected to deref them to access the _real_ fields.
                    // This, for example, will help us suggest accessing a field through a `Box<T>`.
                    if fields.iter().all(|field| !field.vis.is_accessible_from(mod_id, tcx)) {
                        continue;
                    }
                    return Some((
                        fields
                            .iter()
                            .filter(move |field| field.vis.is_accessible_from(mod_id, tcx))
                            // For compile-time reasons put a limit on number of fields we search
                            .take(100),
                        substs,
                    ));
                }
                _ => {}
            }
        }
        None
    }

    /// This method is called after we have encountered a missing field error to recursively
    /// search for the field
    pub(crate) fn check_for_nested_field_satisfying(
        &self,
        span: Span,
        matches: &impl Fn(&ty::FieldDef, Ty<'tcx>) -> bool,
        candidate_field: &ty::FieldDef,
        subst: SubstsRef<'tcx>,
        mut field_path: Vec<Ident>,
        mod_id: DefId,
    ) -> Option<Vec<Ident>> {
        debug!(
            "check_for_nested_field_satisfying(span: {:?}, candidate_field: {:?}, field_path: {:?}",
            span, candidate_field, field_path
        );

        if field_path.len() > 3 {
            // For compile-time reasons and to avoid infinite recursion we only check for fields
            // up to a depth of three
            None
        } else {
            field_path.push(candidate_field.ident(self.tcx).normalize_to_macros_2_0());
            let field_ty = candidate_field.ty(self.tcx, subst);
            if matches(candidate_field, field_ty) {
                return Some(field_path);
            } else if let Some((nested_fields, subst)) =
                self.get_field_candidates_considering_privacy(span, field_ty, mod_id)
            {
                // recursively search fields of `candidate_field` if it's a ty::Adt
                for field in nested_fields {
                    if let Some(field_path) = self.check_for_nested_field_satisfying(
                        span,
                        matches,
                        field,
                        subst,
                        field_path.clone(),
                        mod_id,
                    ) {
                        return Some(field_path);
                    }
                }
            }
            None
        }
    }

    fn check_expr_index(
        &self,
        base: &'tcx hir::Expr<'tcx>,
        idx: &'tcx hir::Expr<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let base_t = self.check_expr(&base);
        let idx_t = self.check_expr(&idx);

        if base_t.references_error() {
            base_t
        } else if idx_t.references_error() {
            idx_t
        } else {
            let base_t = self.structurally_resolve_type(base.span, base_t);
            match self.lookup_indexing(expr, base, base_t, idx, idx_t) {
                Some((index_ty, element_ty)) => {
                    // two-phase not needed because index_ty is never mutable
                    self.demand_coerce(idx, idx_t, index_ty, None, AllowTwoPhase::No);
                    self.select_obligations_where_possible(|errors| {
                        self.point_at_index_if_possible(errors, idx.span)
                    });
                    element_ty
                }
                None => {
                    // Attempt to *shallowly* search for an impl which matches,
                    // but has nested obligations which are unsatisfied.
                    for (base_t, _) in self.autoderef(base.span, base_t).silence_errors() {
                        if let Some((_, index_ty, element_ty)) =
                            self.find_and_report_unsatisfied_index_impl(base, base_t)
                        {
                            self.demand_coerce(idx, idx_t, index_ty, None, AllowTwoPhase::No);
                            return element_ty;
                        }
                    }

                    let mut err = type_error_struct!(
                        self.tcx.sess,
                        expr.span,
                        base_t,
                        E0608,
                        "cannot index into a value of type `{base_t}`",
                    );
                    // Try to give some advice about indexing tuples.
                    if let ty::Tuple(types) = base_t.kind() {
                        let mut needs_note = true;
                        // If the index is an integer, we can show the actual
                        // fixed expression:
                        if let ExprKind::Lit(ref lit) = idx.kind
                            && let ast::LitKind::Int(i, ast::LitIntType::Unsuffixed) = lit.node
                            && i < types.len().try_into().expect("expected tuple index to be < usize length")
                        {
                            let snip = self.tcx.sess.source_map().span_to_snippet(base.span);
                            if let Ok(snip) = snip {
                                err.span_suggestion(
                                    expr.span,
                                    "to access tuple elements, use",
                                    format!("{snip}.{i}"),
                                    Applicability::MachineApplicable,
                                );
                                needs_note = false;
                            }
                        } else if let ExprKind::Path(..) = idx.peel_borrows().kind {
                            err.span_label(idx.span, "cannot access tuple elements at a variable index");
                        }
                        if needs_note {
                            err.help(
                                "to access tuple elements, use tuple indexing \
                                        syntax (e.g., `tuple.0`)",
                            );
                        }
                    }

                    if base_t.is_unsafe_ptr() && idx_t.is_integral() {
                        err.multipart_suggestion(
                            "consider using `wrapping_add` or `add` for indexing into raw pointer",
                            vec![
                                (base.span.between(idx.span), ".wrapping_add(".to_owned()),
                                (
                                    idx.span.shrink_to_hi().until(expr.span.shrink_to_hi()),
                                    ")".to_owned(),
                                ),
                            ],
                            Applicability::MaybeIncorrect,
                        );
                    }

                    let reported = err.emit();
                    Ty::new_error(self.tcx, reported)
                }
            }
        }
    }

    /// Try to match an implementation of `Index` against a self type, and report
    /// the unsatisfied predicates that result from confirming this impl.
    ///
    /// Given an index expression, sometimes the `Self` type shallowly but does not
    /// deeply satisfy an impl predicate. Instead of simply saying that the type
    /// does not support being indexed, we want to point out exactly what nested
    /// predicates cause this to be, so that the user can add them to fix their code.
    fn find_and_report_unsatisfied_index_impl(
        &self,
        base_expr: &hir::Expr<'_>,
        base_ty: Ty<'tcx>,
    ) -> Option<(ErrorGuaranteed, Ty<'tcx>, Ty<'tcx>)> {
        let index_trait_def_id = self.tcx.lang_items().index_trait()?;
        let index_trait_output_def_id = self.tcx.get_diagnostic_item(sym::IndexOutput)?;

        let mut relevant_impls = vec![];
        self.tcx.for_each_relevant_impl(index_trait_def_id, base_ty, |impl_def_id| {
            relevant_impls.push(impl_def_id);
        });
        let [impl_def_id] = relevant_impls[..] else {
            // Only report unsatisfied impl predicates if there's one impl
            return None;
        };

        self.commit_if_ok(|_| {
            let ocx = ObligationCtxt::new(self);
            let impl_substs = self.fresh_substs_for_item(base_expr.span, impl_def_id);
            let impl_trait_ref =
                self.tcx.impl_trait_ref(impl_def_id).unwrap().subst(self.tcx, impl_substs);
            let cause = self.misc(base_expr.span);

            // Match the impl self type against the base ty. If this fails,
            // we just skip this impl, since it's not particularly useful.
            let impl_trait_ref = ocx.normalize(&cause, self.param_env, impl_trait_ref);
            ocx.eq(&cause, self.param_env, impl_trait_ref.self_ty(), base_ty)?;

            // Register the impl's predicates. One of these predicates
            // must be unsatisfied, or else we wouldn't have gotten here
            // in the first place.
            ocx.register_obligations(traits::predicates_for_generics(
                |idx, span| {
                    cause.clone().derived_cause(
                        ty::Binder::dummy(ty::TraitPredicate {
                            trait_ref: impl_trait_ref,
                            polarity: ty::ImplPolarity::Positive,
                            constness: ty::BoundConstness::NotConst,
                        }),
                        |derived| {
                            traits::ImplDerivedObligation(Box::new(
                                traits::ImplDerivedObligationCause {
                                    derived,
                                    impl_or_alias_def_id: impl_def_id,
                                    impl_def_predicate_index: Some(idx),
                                    span,
                                },
                            ))
                        },
                    )
                },
                self.param_env,
                self.tcx.predicates_of(impl_def_id).instantiate(self.tcx, impl_substs),
            ));

            // Normalize the output type, which we can use later on as the
            // return type of the index expression...
            let element_ty = ocx.normalize(
                &cause,
                self.param_env,
                Ty::new_projection(self.tcx, index_trait_output_def_id, impl_trait_ref.substs),
            );

            let errors = ocx.select_where_possible();
            // There should be at least one error reported. If not, we
            // will still delay a span bug in `report_fulfillment_errors`.
            Ok::<_, NoSolution>((
                self.err_ctxt().report_fulfillment_errors(&errors),
                impl_trait_ref.substs.type_at(1),
                element_ty,
            ))
        })
        .ok()
    }

    fn point_at_index_if_possible(
        &self,
        errors: &mut Vec<traits::FulfillmentError<'tcx>>,
        span: Span,
    ) {
        for error in errors {
            match error.obligation.predicate.kind().skip_binder() {
                ty::PredicateKind::Clause(ty::ClauseKind::Trait(predicate))
                    if self.tcx.is_diagnostic_item(sym::SliceIndex, predicate.trait_ref.def_id) => {
                }
                _ => continue,
            }
            error.obligation.cause.span = span;
        }
    }

    fn check_expr_yield(
        &self,
        value: &'tcx hir::Expr<'tcx>,
        expr: &'tcx hir::Expr<'tcx>,
        src: &'tcx hir::YieldSource,
    ) -> Ty<'tcx> {
        match self.resume_yield_tys {
            Some((resume_ty, yield_ty)) => {
                self.check_expr_coercible_to_type(&value, yield_ty, None);

                resume_ty
            }
            // Given that this `yield` expression was generated as a result of lowering a `.await`,
            // we know that the yield type must be `()`; however, the context won't contain this
            // information. Hence, we check the source of the yield expression here and check its
            // value's type against `()` (this check should always hold).
            None if src.is_await() => {
                self.check_expr_coercible_to_type(&value, Ty::new_unit(self.tcx), None);
                Ty::new_unit(self.tcx)
            }
            _ => {
                self.tcx.sess.emit_err(YieldExprOutsideOfGenerator { span: expr.span });
                // Avoid expressions without types during writeback (#78653).
                self.check_expr(value);
                Ty::new_unit(self.tcx)
            }
        }
    }

    fn check_expr_asm_operand(&self, expr: &'tcx hir::Expr<'tcx>, is_input: bool) {
        let needs = if is_input { Needs::None } else { Needs::MutPlace };
        let ty = self.check_expr_with_needs(expr, needs);
        self.require_type_is_sized(ty, expr.span, traits::InlineAsmSized);

        if !is_input && !expr.is_syntactic_place_expr() {
            let mut err = self.tcx.sess.struct_span_err(expr.span, "invalid asm output");
            err.span_label(expr.span, "cannot assign to this expression");
            err.emit();
        }

        // If this is an input value, we require its type to be fully resolved
        // at this point. This allows us to provide helpful coercions which help
        // pass the type candidate list in a later pass.
        //
        // We don't require output types to be resolved at this point, which
        // allows them to be inferred based on how they are used later in the
        // function.
        if is_input {
            let ty = self.structurally_resolve_type(expr.span, ty);
            match *ty.kind() {
                ty::FnDef(..) => {
                    let fnptr_ty = Ty::new_fn_ptr(self.tcx, ty.fn_sig(self.tcx));
                    self.demand_coerce(expr, ty, fnptr_ty, None, AllowTwoPhase::No);
                }
                ty::Ref(_, base_ty, mutbl) => {
                    let ptr_ty = Ty::new_ptr(self.tcx, ty::TypeAndMut { ty: base_ty, mutbl });
                    self.demand_coerce(expr, ty, ptr_ty, None, AllowTwoPhase::No);
                }
                _ => {}
            }
        }
    }

    fn check_expr_asm(&self, asm: &'tcx hir::InlineAsm<'tcx>) -> Ty<'tcx> {
        for (op, _op_sp) in asm.operands {
            match op {
                hir::InlineAsmOperand::In { expr, .. } => {
                    self.check_expr_asm_operand(expr, true);
                }
                hir::InlineAsmOperand::Out { expr: Some(expr), .. }
                | hir::InlineAsmOperand::InOut { expr, .. } => {
                    self.check_expr_asm_operand(expr, false);
                }
                hir::InlineAsmOperand::Out { expr: None, .. } => {}
                hir::InlineAsmOperand::SplitInOut { in_expr, out_expr, .. } => {
                    self.check_expr_asm_operand(in_expr, true);
                    if let Some(out_expr) = out_expr {
                        self.check_expr_asm_operand(out_expr, false);
                    }
                }
                // `AnonConst`s have their own body and is type-checked separately.
                // As they don't flow into the type system we don't need them to
                // be well-formed.
                hir::InlineAsmOperand::Const { .. } | hir::InlineAsmOperand::SymFn { .. } => {}
                hir::InlineAsmOperand::SymStatic { .. } => {}
            }
        }
        if asm.options.contains(ast::InlineAsmOptions::NORETURN) {
            self.tcx.types.never
        } else {
            Ty::new_unit(self.tcx)
        }
    }

    fn check_offset_of(
        &self,
        container: &'tcx hir::Ty<'tcx>,
        fields: &[Ident],
        expr: &'tcx hir::Expr<'tcx>,
    ) -> Ty<'tcx> {
        let container = self.to_ty(container).normalized;

        let mut field_indices = Vec::with_capacity(fields.len());
        let mut current_container = container;

        for &field in fields {
            let container = self.structurally_resolve_type(expr.span, current_container);

            match container.kind() {
                ty::Adt(container_def, substs) if !container_def.is_enum() => {
                    let block = self.tcx.hir().local_def_id_to_hir_id(self.body_id);
                    let (ident, def_scope) =
                        self.tcx.adjust_ident_and_get_scope(field, container_def.did(), block);

                    let fields = &container_def.non_enum_variant().fields;
                    if let Some((index, field)) = fields
                        .iter_enumerated()
                        .find(|(_, f)| f.ident(self.tcx).normalize_to_macros_2_0() == ident)
                    {
                        let field_ty = self.field_ty(expr.span, field, substs);

                        // FIXME: DSTs with static alignment should be allowed
                        self.require_type_is_sized(field_ty, expr.span, traits::MiscObligation);

                        if field.vis.is_accessible_from(def_scope, self.tcx) {
                            self.tcx.check_stability(field.did, Some(expr.hir_id), expr.span, None);
                        } else {
                            self.private_field_err(ident, container_def.did()).emit();
                        }

                        // Save the index of all fields regardless of their visibility in case
                        // of error recovery.
                        field_indices.push(index);
                        current_container = field_ty;

                        continue;
                    }
                }
                ty::Tuple(tys) => {
                    if let Ok(index) = field.as_str().parse::<usize>()
                        && field.name == sym::integer(index)
                    {
                        for ty in tys.iter().take(index + 1) {
                            self.require_type_is_sized(ty, expr.span, traits::MiscObligation);
                        }
                        if let Some(&field_ty) = tys.get(index) {
                            field_indices.push(index.into());
                            current_container = field_ty;

                            continue;
                        }
                    }
                }
                _ => (),
            };

            self.no_such_field_err(field, container, expr.hir_id).emit();

            break;
        }

        self.typeck_results
            .borrow_mut()
            .offset_of_data_mut()
            .insert(expr.hir_id, (container, field_indices));

        self.tcx.types.usize
    }
}
