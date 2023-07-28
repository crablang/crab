//! A different sort of visitor for walking fn bodies. Unlike the
//! normal visitor, which just walks the entire body in one shot, the
//! `ExprUseVisitor` determines how expressions are being used.

use std::slice::from_ref;

use hir::def::DefKind;
use hir::Expr;
// Export these here so that Clippy can use them.
pub use rustc_middle::hir::place::{Place, PlaceBase, PlaceWithHirId, Projection};

use rustc_data_structures::fx::FxIndexMap;
use rustc_hir as hir;
use rustc_hir::def::Res;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::PatKind;
use rustc_infer::infer::InferCtxt;
use rustc_middle::hir::place::ProjectionKind;
use rustc_middle::mir::FakeReadCause;
use rustc_middle::ty::{self, adjustment, AdtKind, Ty, TyCtxt};
use rustc_target::abi::FIRST_VARIANT;
use ty::BorrowKind::ImmBorrow;

use crate::mem_categorization as mc;

/// This trait defines the callbacks you can expect to receive when
/// employing the ExprUseVisitor.
pub trait Delegate<'tcx> {
    /// The value found at `place` is moved, depending
    /// on `mode`. Where `diag_expr_id` is the id used for diagnostics for `place`.
    ///
    /// Use of a `Copy` type in a ByValue context is considered a use
    /// by `ImmBorrow` and `borrow` is called instead. This is because
    /// a shared borrow is the "minimum access" that would be needed
    /// to perform a copy.
    ///
    ///
    /// The parameter `diag_expr_id` indicates the HIR id that ought to be used for
    /// diagnostics. Around pattern matching such as `let pat = expr`, the diagnostic
    /// id will be the id of the expression `expr` but the place itself will have
    /// the id of the binding in the pattern `pat`.
    fn consume(&mut self, place_with_id: &PlaceWithHirId<'tcx>, diag_expr_id: hir::HirId);

    /// The value found at `place` is being borrowed with kind `bk`.
    /// `diag_expr_id` is the id used for diagnostics (see `consume` for more details).
    fn borrow(
        &mut self,
        place_with_id: &PlaceWithHirId<'tcx>,
        diag_expr_id: hir::HirId,
        bk: ty::BorrowKind,
    );

    /// The value found at `place` is being copied.
    /// `diag_expr_id` is the id used for diagnostics (see `consume` for more details).
    fn copy(&mut self, place_with_id: &PlaceWithHirId<'tcx>, diag_expr_id: hir::HirId) {
        // In most cases, copying data from `x` is equivalent to doing `*&x`, so by default
        // we treat a copy of `x` as a borrow of `x`.
        self.borrow(place_with_id, diag_expr_id, ty::BorrowKind::ImmBorrow)
    }

    /// The path at `assignee_place` is being assigned to.
    /// `diag_expr_id` is the id used for diagnostics (see `consume` for more details).
    fn mutate(&mut self, assignee_place: &PlaceWithHirId<'tcx>, diag_expr_id: hir::HirId);

    /// The path at `binding_place` is a binding that is being initialized.
    ///
    /// This covers cases such as `let x = 42;`
    fn bind(&mut self, binding_place: &PlaceWithHirId<'tcx>, diag_expr_id: hir::HirId) {
        // Bindings can normally be treated as a regular assignment, so by default we
        // forward this to the mutate callback.
        self.mutate(binding_place, diag_expr_id)
    }

    /// The `place` should be a fake read because of specified `cause`.
    fn fake_read(
        &mut self,
        place_with_id: &PlaceWithHirId<'tcx>,
        cause: FakeReadCause,
        diag_expr_id: hir::HirId,
    );
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ConsumeMode {
    /// reference to x where x has a type that copies
    Copy,
    /// reference to x where x has a type that moves
    Move,
}

/// The ExprUseVisitor type
///
/// This is the code that actually walks the tree.
pub struct ExprUseVisitor<'a, 'tcx> {
    mc: mc::MemCategorizationContext<'a, 'tcx>,
    body_owner: LocalDefId,
    delegate: &'a mut dyn Delegate<'tcx>,
}

/// If the MC results in an error, it's because the type check
/// failed (or will fail, when the error is uncovered and reported
/// during writeback). In this case, we just ignore this part of the
/// code.
///
/// Note that this macro appears similar to try!(), but, unlike try!(),
/// it does not propagate the error.
macro_rules! return_if_err {
    ($inp: expr) => {
        match $inp {
            Ok(v) => v,
            Err(()) => {
                debug!("mc reported err");
                return;
            }
        }
    };
}

impl<'a, 'tcx> ExprUseVisitor<'a, 'tcx> {
    /// Creates the ExprUseVisitor, configuring it with the various options provided:
    ///
    /// - `delegate` -- who receives the callbacks
    /// - `param_env` --- parameter environment for trait lookups (esp. pertaining to `Copy`)
    /// - `typeck_results` --- typeck results for the code being analyzed
    pub fn new(
        delegate: &'a mut (dyn Delegate<'tcx> + 'a),
        infcx: &'a InferCtxt<'tcx>,
        body_owner: LocalDefId,
        param_env: ty::ParamEnv<'tcx>,
        typeck_results: &'a ty::TypeckResults<'tcx>,
    ) -> Self {
        ExprUseVisitor {
            mc: mc::MemCategorizationContext::new(infcx, param_env, body_owner, typeck_results),
            body_owner,
            delegate,
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn consume_body(&mut self, body: &hir::Body<'_>) {
        for param in body.params {
            let param_ty = return_if_err!(self.mc.pat_ty_adjusted(param.pat));
            debug!("consume_body: param_ty = {:?}", param_ty);

            let param_place = self.mc.cat_rvalue(param.hir_id, param.pat.span, param_ty);

            self.walk_irrefutable_pat(&param_place, param.pat);
        }

        self.consume_expr(&body.value);
    }

    fn tcx(&self) -> TyCtxt<'tcx> {
        self.mc.tcx()
    }

    fn delegate_consume(&mut self, place_with_id: &PlaceWithHirId<'tcx>, diag_expr_id: hir::HirId) {
        delegate_consume(&self.mc, self.delegate, place_with_id, diag_expr_id)
    }

    fn consume_exprs(&mut self, exprs: &[hir::Expr<'_>]) {
        for expr in exprs {
            self.consume_expr(expr);
        }
    }

    pub fn consume_expr(&mut self, expr: &hir::Expr<'_>) {
        debug!("consume_expr(expr={:?})", expr);

        let place_with_id = return_if_err!(self.mc.cat_expr(expr));
        self.delegate_consume(&place_with_id, place_with_id.hir_id);
        self.walk_expr(expr);
    }

    fn mutate_expr(&mut self, expr: &hir::Expr<'_>) {
        let place_with_id = return_if_err!(self.mc.cat_expr(expr));
        self.delegate.mutate(&place_with_id, place_with_id.hir_id);
        self.walk_expr(expr);
    }

    fn borrow_expr(&mut self, expr: &hir::Expr<'_>, bk: ty::BorrowKind) {
        debug!("borrow_expr(expr={:?}, bk={:?})", expr, bk);

        let place_with_id = return_if_err!(self.mc.cat_expr(expr));
        self.delegate.borrow(&place_with_id, place_with_id.hir_id, bk);

        self.walk_expr(expr)
    }

    fn select_from_expr(&mut self, expr: &hir::Expr<'_>) {
        self.walk_expr(expr)
    }

    pub fn walk_expr(&mut self, expr: &hir::Expr<'_>) {
        debug!("walk_expr(expr={:?})", expr);

        self.walk_adjustment(expr);

        match expr.kind {
            hir::ExprKind::Path(_) => {}

            hir::ExprKind::Type(subexpr, _) => self.walk_expr(subexpr),

            hir::ExprKind::Unary(hir::UnOp::Deref, base) => {
                // *base
                self.select_from_expr(base);
            }

            hir::ExprKind::Field(base, _) => {
                // base.f
                self.select_from_expr(base);
            }

            hir::ExprKind::Index(lhs, rhs) => {
                // lhs[rhs]
                self.select_from_expr(lhs);
                self.consume_expr(rhs);
            }

            hir::ExprKind::Call(callee, args) => {
                // callee(args)
                self.consume_expr(callee);
                self.consume_exprs(args);
            }

            hir::ExprKind::MethodCall(.., receiver, args, _) => {
                // callee.m(args)
                self.consume_expr(receiver);
                self.consume_exprs(args);
            }

            hir::ExprKind::Struct(_, fields, ref opt_with) => {
                self.walk_struct_expr(fields, opt_with);
            }

            hir::ExprKind::Tup(exprs) => {
                self.consume_exprs(exprs);
            }

            hir::ExprKind::If(ref cond_expr, ref then_expr, ref opt_else_expr) => {
                self.consume_expr(cond_expr);
                self.consume_expr(then_expr);
                if let Some(ref else_expr) = *opt_else_expr {
                    self.consume_expr(else_expr);
                }
            }

            hir::ExprKind::Let(hir::Let { pat, init, .. }) => {
                self.walk_local(init, pat, None, |t| t.borrow_expr(init, ty::ImmBorrow))
            }

            hir::ExprKind::Match(ref discr, arms, _) => {
                let discr_place = return_if_err!(self.mc.cat_expr(discr));
                return_if_err!(self.maybe_read_scrutinee(
                    discr,
                    discr_place.clone(),
                    arms.iter().map(|arm| arm.pat),
                ));

                // treatment of the discriminant is handled while walking the arms.
                for arm in arms {
                    self.walk_arm(&discr_place, arm);
                }
            }

            hir::ExprKind::Array(exprs) => {
                self.consume_exprs(exprs);
            }

            hir::ExprKind::AddrOf(_, m, ref base) => {
                // &base
                // make sure that the thing we are pointing out stays valid
                // for the lifetime `scope_r` of the resulting ptr:
                let bk = ty::BorrowKind::from_mutbl(m);
                self.borrow_expr(base, bk);
            }

            hir::ExprKind::InlineAsm(asm) => {
                for (op, _op_sp) in asm.operands {
                    match op {
                        hir::InlineAsmOperand::In { expr, .. } => self.consume_expr(expr),
                        hir::InlineAsmOperand::Out { expr: Some(expr), .. }
                        | hir::InlineAsmOperand::InOut { expr, .. } => {
                            self.mutate_expr(expr);
                        }
                        hir::InlineAsmOperand::SplitInOut { in_expr, out_expr, .. } => {
                            self.consume_expr(in_expr);
                            if let Some(out_expr) = out_expr {
                                self.mutate_expr(out_expr);
                            }
                        }
                        hir::InlineAsmOperand::Out { expr: None, .. }
                        | hir::InlineAsmOperand::Const { .. }
                        | hir::InlineAsmOperand::SymFn { .. }
                        | hir::InlineAsmOperand::SymStatic { .. } => {}
                    }
                }
            }

            hir::ExprKind::Continue(..)
            | hir::ExprKind::Lit(..)
            | hir::ExprKind::ConstBlock(..)
            | hir::ExprKind::OffsetOf(..)
            | hir::ExprKind::Err(_) => {}

            hir::ExprKind::Loop(blk, ..) => {
                self.walk_block(blk);
            }

            hir::ExprKind::Unary(_, lhs) => {
                self.consume_expr(lhs);
            }

            hir::ExprKind::Binary(_, lhs, rhs) => {
                self.consume_expr(lhs);
                self.consume_expr(rhs);
            }

            hir::ExprKind::Block(blk, _) => {
                self.walk_block(blk);
            }

            hir::ExprKind::Break(_, ref opt_expr) | hir::ExprKind::Ret(ref opt_expr) => {
                if let Some(expr) = *opt_expr {
                    self.consume_expr(expr);
                }
            }

            hir::ExprKind::Become(call) => {
                self.consume_expr(call);
            }

            hir::ExprKind::Assign(lhs, rhs, _) => {
                self.mutate_expr(lhs);
                self.consume_expr(rhs);
            }

            hir::ExprKind::Cast(base, _) => {
                self.consume_expr(base);
            }

            hir::ExprKind::DropTemps(expr) => {
                self.consume_expr(expr);
            }

            hir::ExprKind::AssignOp(_, lhs, rhs) => {
                if self.mc.typeck_results.is_method_call(expr) {
                    self.consume_expr(lhs);
                } else {
                    self.mutate_expr(lhs);
                }
                self.consume_expr(rhs);
            }

            hir::ExprKind::Repeat(base, _) => {
                self.consume_expr(base);
            }

            hir::ExprKind::Closure(closure) => {
                self.walk_captures(closure);
            }

            hir::ExprKind::Yield(value, _) => {
                self.consume_expr(value);
            }
        }
    }

    fn walk_stmt(&mut self, stmt: &hir::Stmt<'_>) {
        match stmt.kind {
            hir::StmtKind::Local(hir::Local { pat, init: Some(expr), els, .. }) => {
                self.walk_local(expr, pat, *els, |_| {})
            }

            hir::StmtKind::Local(_) => {}

            hir::StmtKind::Item(_) => {
                // We don't visit nested items in this visitor,
                // only the fn body we were given.
            }

            hir::StmtKind::Expr(ref expr) | hir::StmtKind::Semi(ref expr) => {
                self.consume_expr(expr);
            }
        }
    }

    fn maybe_read_scrutinee<'t>(
        &mut self,
        discr: &Expr<'_>,
        discr_place: PlaceWithHirId<'tcx>,
        pats: impl Iterator<Item = &'t hir::Pat<'t>>,
    ) -> Result<(), ()> {
        // Matching should not always be considered a use of the place, hence
        // discr does not necessarily need to be borrowed.
        // We only want to borrow discr if the pattern contain something other
        // than wildcards.
        let ExprUseVisitor { ref mc, body_owner: _, delegate: _ } = *self;
        let mut needs_to_be_read = false;
        for pat in pats {
            mc.cat_pattern(discr_place.clone(), pat, |place, pat| {
                match &pat.kind {
                    PatKind::Binding(.., opt_sub_pat) => {
                        // If the opt_sub_pat is None, than the binding does not count as
                        // a wildcard for the purpose of borrowing discr.
                        if opt_sub_pat.is_none() {
                            needs_to_be_read = true;
                        }
                    }
                    PatKind::Path(qpath) => {
                        // A `Path` pattern is just a name like `Foo`. This is either a
                        // named constant or else it refers to an ADT variant

                        let res = self.mc.typeck_results.qpath_res(qpath, pat.hir_id);
                        match res {
                            Res::Def(DefKind::Const, _) | Res::Def(DefKind::AssocConst, _) => {
                                // Named constants have to be equated with the value
                                // being matched, so that's a read of the value being matched.
                                //
                                // FIXME: We don't actually reads for ZSTs.
                                needs_to_be_read = true;
                            }
                            _ => {
                                // Otherwise, this is a struct/enum variant, and so it's
                                // only a read if we need to read the discriminant.
                                needs_to_be_read |= is_multivariant_adt(place.place.ty());
                            }
                        }
                    }
                    PatKind::TupleStruct(..) | PatKind::Struct(..) | PatKind::Tuple(..) => {
                        // For `Foo(..)`, `Foo { ... }` and `(...)` patterns, check if we are matching
                        // against a multivariant enum or struct. In that case, we have to read
                        // the discriminant. Otherwise this kind of pattern doesn't actually
                        // read anything (we'll get invoked for the `...`, which may indeed
                        // perform some reads).

                        let place_ty = place.place.ty();
                        needs_to_be_read |= is_multivariant_adt(place_ty);
                    }
                    PatKind::Lit(_) | PatKind::Range(..) => {
                        // If the PatKind is a Lit or a Range then we want
                        // to borrow discr.
                        needs_to_be_read = true;
                    }
                    PatKind::Slice(lhs, wild, rhs) => {
                        // We don't need to test the length if the pattern is `[..]`
                        if matches!((lhs, wild, rhs), (&[], Some(_), &[]))
                            // Arrays have a statically known size, so
                            // there is no need to read their length
                            || place.place.ty().peel_refs().is_array()
                        {
                        } else {
                            needs_to_be_read = true;
                        }
                    }
                    PatKind::Or(_) | PatKind::Box(_) | PatKind::Ref(..) | PatKind::Wild => {
                        // If the PatKind is Or, Box, or Ref, the decision is made later
                        // as these patterns contains subpatterns
                        // If the PatKind is Wild, the decision is made based on the other patterns being
                        // examined
                    }
                }
            })?
        }

        if needs_to_be_read {
            self.borrow_expr(discr, ty::ImmBorrow);
        } else {
            let closure_def_id = match discr_place.place.base {
                PlaceBase::Upvar(upvar_id) => Some(upvar_id.closure_expr_id),
                _ => None,
            };

            self.delegate.fake_read(
                &discr_place,
                FakeReadCause::ForMatchedPlace(closure_def_id),
                discr_place.hir_id,
            );

            // We always want to walk the discriminant. We want to make sure, for instance,
            // that the discriminant has been initialized.
            self.walk_expr(discr);
        }
        Ok(())
    }

    fn walk_local<F>(
        &mut self,
        expr: &hir::Expr<'_>,
        pat: &hir::Pat<'_>,
        els: Option<&hir::Block<'_>>,
        mut f: F,
    ) where
        F: FnMut(&mut Self),
    {
        self.walk_expr(expr);
        let expr_place = return_if_err!(self.mc.cat_expr(expr));
        f(self);
        if let Some(els) = els {
            // borrowing because we need to test the discriminant
            return_if_err!(self.maybe_read_scrutinee(
                expr,
                expr_place.clone(),
                from_ref(pat).iter()
            ));
            self.walk_block(els)
        }
        self.walk_irrefutable_pat(&expr_place, &pat);
    }

    /// Indicates that the value of `blk` will be consumed, meaning either copied or moved
    /// depending on its type.
    fn walk_block(&mut self, blk: &hir::Block<'_>) {
        debug!("walk_block(blk.hir_id={})", blk.hir_id);

        for stmt in blk.stmts {
            self.walk_stmt(stmt);
        }

        if let Some(ref tail_expr) = blk.expr {
            self.consume_expr(tail_expr);
        }
    }

    fn walk_struct_expr<'hir>(
        &mut self,
        fields: &[hir::ExprField<'_>],
        opt_with: &Option<&'hir hir::Expr<'_>>,
    ) {
        // Consume the expressions supplying values for each field.
        for field in fields {
            self.consume_expr(field.expr);

            // The struct path probably didn't resolve
            if self.mc.typeck_results.opt_field_index(field.hir_id).is_none() {
                self.tcx().sess.delay_span_bug(field.span, "couldn't resolve index for field");
            }
        }

        let with_expr = match *opt_with {
            Some(w) => &*w,
            None => {
                return;
            }
        };

        let with_place = return_if_err!(self.mc.cat_expr(with_expr));

        // Select just those fields of the `with`
        // expression that will actually be used
        match with_place.place.ty().kind() {
            ty::Adt(adt, args) if adt.is_struct() => {
                // Consume those fields of the with expression that are needed.
                for (f_index, with_field) in adt.non_enum_variant().fields.iter_enumerated() {
                    let is_mentioned = fields
                        .iter()
                        .any(|f| self.mc.typeck_results.opt_field_index(f.hir_id) == Some(f_index));
                    if !is_mentioned {
                        let field_place = self.mc.cat_projection(
                            &*with_expr,
                            with_place.clone(),
                            with_field.ty(self.tcx(), args),
                            ProjectionKind::Field(f_index, FIRST_VARIANT),
                        );
                        self.delegate_consume(&field_place, field_place.hir_id);
                    }
                }
            }
            _ => {
                // the base expression should always evaluate to a
                // struct; however, when EUV is run during typeck, it
                // may not. This will generate an error earlier in typeck,
                // so we can just ignore it.
                if self.tcx().sess.has_errors().is_none() {
                    span_bug!(with_expr.span, "with expression doesn't evaluate to a struct");
                }
            }
        }

        // walk the with expression so that complex expressions
        // are properly handled.
        self.walk_expr(with_expr);
    }

    /// Invoke the appropriate delegate calls for anything that gets
    /// consumed or borrowed as part of the automatic adjustment
    /// process.
    fn walk_adjustment(&mut self, expr: &hir::Expr<'_>) {
        let adjustments = self.mc.typeck_results.expr_adjustments(expr);
        let mut place_with_id = return_if_err!(self.mc.cat_expr_unadjusted(expr));
        for adjustment in adjustments {
            debug!("walk_adjustment expr={:?} adj={:?}", expr, adjustment);
            match adjustment.kind {
                adjustment::Adjust::NeverToAny
                | adjustment::Adjust::Pointer(_)
                | adjustment::Adjust::DynStar => {
                    // Creating a closure/fn-pointer or unsizing consumes
                    // the input and stores it into the resulting rvalue.
                    self.delegate_consume(&place_with_id, place_with_id.hir_id);
                }

                adjustment::Adjust::Deref(None) => {}

                // Autoderefs for overloaded Deref calls in fact reference
                // their receiver. That is, if we have `(*x)` where `x`
                // is of type `Rc<T>`, then this in fact is equivalent to
                // `x.deref()`. Since `deref()` is declared with `&self`,
                // this is an autoref of `x`.
                adjustment::Adjust::Deref(Some(ref deref)) => {
                    let bk = ty::BorrowKind::from_mutbl(deref.mutbl);
                    self.delegate.borrow(&place_with_id, place_with_id.hir_id, bk);
                }

                adjustment::Adjust::Borrow(ref autoref) => {
                    self.walk_autoref(expr, &place_with_id, autoref);
                }
            }
            place_with_id =
                return_if_err!(self.mc.cat_expr_adjusted(expr, place_with_id, adjustment));
        }
    }

    /// Walks the autoref `autoref` applied to the autoderef'd
    /// `expr`. `base_place` is the mem-categorized form of `expr`
    /// after all relevant autoderefs have occurred.
    fn walk_autoref(
        &mut self,
        expr: &hir::Expr<'_>,
        base_place: &PlaceWithHirId<'tcx>,
        autoref: &adjustment::AutoBorrow<'tcx>,
    ) {
        debug!(
            "walk_autoref(expr.hir_id={} base_place={:?} autoref={:?})",
            expr.hir_id, base_place, autoref
        );

        match *autoref {
            adjustment::AutoBorrow::Ref(_, m) => {
                self.delegate.borrow(
                    base_place,
                    base_place.hir_id,
                    ty::BorrowKind::from_mutbl(m.into()),
                );
            }

            adjustment::AutoBorrow::RawPtr(m) => {
                debug!("walk_autoref: expr.hir_id={} base_place={:?}", expr.hir_id, base_place);

                self.delegate.borrow(base_place, base_place.hir_id, ty::BorrowKind::from_mutbl(m));
            }
        }
    }

    fn walk_arm(&mut self, discr_place: &PlaceWithHirId<'tcx>, arm: &hir::Arm<'_>) {
        let closure_def_id = match discr_place.place.base {
            PlaceBase::Upvar(upvar_id) => Some(upvar_id.closure_expr_id),
            _ => None,
        };

        self.delegate.fake_read(
            discr_place,
            FakeReadCause::ForMatchedPlace(closure_def_id),
            discr_place.hir_id,
        );
        self.walk_pat(discr_place, arm.pat, arm.guard.is_some());

        if let Some(hir::Guard::If(e)) = arm.guard {
            self.consume_expr(e)
        } else if let Some(hir::Guard::IfLet(ref l)) = arm.guard {
            self.consume_expr(l.init)
        }

        self.consume_expr(arm.body);
    }

    /// Walks a pat that occurs in isolation (i.e., top-level of fn argument or
    /// let binding, and *not* a match arm or nested pat.)
    fn walk_irrefutable_pat(&mut self, discr_place: &PlaceWithHirId<'tcx>, pat: &hir::Pat<'_>) {
        let closure_def_id = match discr_place.place.base {
            PlaceBase::Upvar(upvar_id) => Some(upvar_id.closure_expr_id),
            _ => None,
        };

        self.delegate.fake_read(
            discr_place,
            FakeReadCause::ForLet(closure_def_id),
            discr_place.hir_id,
        );
        self.walk_pat(discr_place, pat, false);
    }

    /// The core driver for walking a pattern
    fn walk_pat(
        &mut self,
        discr_place: &PlaceWithHirId<'tcx>,
        pat: &hir::Pat<'_>,
        has_guard: bool,
    ) {
        debug!("walk_pat(discr_place={:?}, pat={:?}, has_guard={:?})", discr_place, pat, has_guard);

        let tcx = self.tcx();
        let ExprUseVisitor { ref mc, body_owner: _, ref mut delegate } = *self;
        return_if_err!(mc.cat_pattern(discr_place.clone(), pat, |place, pat| {
            if let PatKind::Binding(_, canonical_id, ..) = pat.kind {
                debug!("walk_pat: binding place={:?} pat={:?}", place, pat);
                if let Some(bm) =
                    mc.typeck_results.extract_binding_mode(tcx.sess, pat.hir_id, pat.span)
                {
                    debug!("walk_pat: pat.hir_id={:?} bm={:?}", pat.hir_id, bm);

                    // pat_ty: the type of the binding being produced.
                    let pat_ty = return_if_err!(mc.node_ty(pat.hir_id));
                    debug!("walk_pat: pat_ty={:?}", pat_ty);

                    let def = Res::Local(canonical_id);
                    if let Ok(ref binding_place) = mc.cat_res(pat.hir_id, pat.span, pat_ty, def) {
                        delegate.bind(binding_place, binding_place.hir_id);
                    }

                    // Subtle: MIR desugaring introduces immutable borrows for each pattern
                    // binding when lowering pattern guards to ensure that the guard does not
                    // modify the scrutinee.
                    if has_guard {
                        delegate.borrow(place, discr_place.hir_id, ImmBorrow);
                    }

                    // It is also a borrow or copy/move of the value being matched.
                    // In a cases of pattern like `let pat = upvar`, don't use the span
                    // of the pattern, as this just looks confusing, instead use the span
                    // of the discriminant.
                    match bm {
                        ty::BindByReference(m) => {
                            let bk = ty::BorrowKind::from_mutbl(m);
                            delegate.borrow(place, discr_place.hir_id, bk);
                        }
                        ty::BindByValue(..) => {
                            debug!("walk_pat binding consuming pat");
                            delegate_consume(mc, *delegate, place, discr_place.hir_id);
                        }
                    }
                }
            }
        }));
    }

    /// Handle the case where the current body contains a closure.
    ///
    /// When the current body being handled is a closure, then we must make sure that
    /// - The parent closure only captures Places from the nested closure that are not local to it.
    ///
    /// In the following example the closures `c` only captures `p.x` even though `incr`
    /// is a capture of the nested closure
    ///
    /// ```
    /// struct P { x: i32 }
    /// let mut p = P { x: 4 };
    /// let c = || {
    ///    let incr = 10;
    ///    let nested = || p.x += incr;
    /// };
    /// ```
    ///
    /// - When reporting the Place back to the Delegate, ensure that the UpvarId uses the enclosing
    /// closure as the DefId.
    fn walk_captures(&mut self, closure_expr: &hir::Closure<'_>) {
        fn upvar_is_local_variable(
            upvars: Option<&FxIndexMap<hir::HirId, hir::Upvar>>,
            upvar_id: hir::HirId,
            body_owner_is_closure: bool,
        ) -> bool {
            upvars.map(|upvars| !upvars.contains_key(&upvar_id)).unwrap_or(body_owner_is_closure)
        }

        debug!("walk_captures({:?})", closure_expr);

        let tcx = self.tcx();
        let closure_def_id = closure_expr.def_id;
        let upvars = tcx.upvars_mentioned(self.body_owner);

        // For purposes of this function, generator and closures are equivalent.
        let body_owner_is_closure =
            matches!(tcx.hir().body_owner_kind(self.body_owner), hir::BodyOwnerKind::Closure,);

        // If we have a nested closure, we want to include the fake reads present in the nested closure.
        if let Some(fake_reads) = self.mc.typeck_results.closure_fake_reads.get(&closure_def_id) {
            for (fake_read, cause, hir_id) in fake_reads.iter() {
                match fake_read.base {
                    PlaceBase::Upvar(upvar_id) => {
                        if upvar_is_local_variable(
                            upvars,
                            upvar_id.var_path.hir_id,
                            body_owner_is_closure,
                        ) {
                            // The nested closure might be fake reading the current (enclosing) closure's local variables.
                            // The only places we want to fake read before creating the parent closure are the ones that
                            // are not local to it/ defined by it.
                            //
                            // ```rust,ignore(cannot-test-this-because-pseudo-code)
                            // let v1 = (0, 1);
                            // let c = || { // fake reads: v1
                            //    let v2 = (0, 1);
                            //    let e = || { // fake reads: v1, v2
                            //       let (_, t1) = v1;
                            //       let (_, t2) = v2;
                            //    }
                            // }
                            // ```
                            // This check is performed when visiting the body of the outermost closure (`c`) and ensures
                            // that we don't add a fake read of v2 in c.
                            continue;
                        }
                    }
                    _ => {
                        bug!(
                            "Do not know how to get HirId out of Rvalue and StaticItem {:?}",
                            fake_read.base
                        );
                    }
                };
                self.delegate.fake_read(
                    &PlaceWithHirId { place: fake_read.clone(), hir_id: *hir_id },
                    *cause,
                    *hir_id,
                );
            }
        }

        if let Some(min_captures) = self.mc.typeck_results.closure_min_captures.get(&closure_def_id)
        {
            for (var_hir_id, min_list) in min_captures.iter() {
                if upvars.map_or(body_owner_is_closure, |upvars| !upvars.contains_key(var_hir_id)) {
                    // The nested closure might be capturing the current (enclosing) closure's local variables.
                    // We check if the root variable is ever mentioned within the enclosing closure, if not
                    // then for the current body (if it's a closure) these aren't captures, we will ignore them.
                    continue;
                }
                for captured_place in min_list {
                    let place = &captured_place.place;
                    let capture_info = captured_place.info;

                    let place_base = if body_owner_is_closure {
                        // Mark the place to be captured by the enclosing closure
                        PlaceBase::Upvar(ty::UpvarId::new(*var_hir_id, self.body_owner))
                    } else {
                        // If the body owner isn't a closure then the variable must
                        // be a local variable
                        PlaceBase::Local(*var_hir_id)
                    };
                    let closure_hir_id = tcx.hir().local_def_id_to_hir_id(closure_def_id);
                    let place_with_id = PlaceWithHirId::new(
                        capture_info
                            .path_expr_id
                            .unwrap_or(capture_info.capture_kind_expr_id.unwrap_or(closure_hir_id)),
                        place.base_ty,
                        place_base,
                        place.projections.clone(),
                    );

                    match capture_info.capture_kind {
                        ty::UpvarCapture::ByValue => {
                            self.delegate_consume(&place_with_id, place_with_id.hir_id);
                        }
                        ty::UpvarCapture::ByRef(upvar_borrow) => {
                            self.delegate.borrow(
                                &place_with_id,
                                place_with_id.hir_id,
                                upvar_borrow,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn copy_or_move<'a, 'tcx>(
    mc: &mc::MemCategorizationContext<'a, 'tcx>,
    place_with_id: &PlaceWithHirId<'tcx>,
) -> ConsumeMode {
    if !mc.type_is_copy_modulo_regions(place_with_id.place.ty()) {
        ConsumeMode::Move
    } else {
        ConsumeMode::Copy
    }
}

// - If a place is used in a `ByValue` context then move it if it's not a `Copy` type.
// - If the place that is a `Copy` type consider it an `ImmBorrow`.
fn delegate_consume<'a, 'tcx>(
    mc: &mc::MemCategorizationContext<'a, 'tcx>,
    delegate: &mut (dyn Delegate<'tcx> + 'a),
    place_with_id: &PlaceWithHirId<'tcx>,
    diag_expr_id: hir::HirId,
) {
    debug!("delegate_consume(place_with_id={:?})", place_with_id);

    let mode = copy_or_move(mc, place_with_id);

    match mode {
        ConsumeMode::Move => delegate.consume(place_with_id, diag_expr_id),
        ConsumeMode::Copy => delegate.copy(place_with_id, diag_expr_id),
    }
}

fn is_multivariant_adt(ty: Ty<'_>) -> bool {
    if let ty::Adt(def, _) = ty.kind() {
        // Note that if a non-exhaustive SingleVariant is defined in another crate, we need
        // to assume that more cases will be added to the variant in the future. This mean
        // that we should handle non-exhaustive SingleVariant the same way we would handle
        // a MultiVariant.
        // If the variant is not local it must be defined in another crate.
        let is_non_exhaustive = match def.adt_kind() {
            AdtKind::Struct | AdtKind::Union => {
                def.non_enum_variant().is_field_list_non_exhaustive()
            }
            AdtKind::Enum => def.is_variant_list_non_exhaustive(),
        };
        def.variants().len() > 1 || (!def.did().is_local() && is_non_exhaustive)
    } else {
        false
    }
}
