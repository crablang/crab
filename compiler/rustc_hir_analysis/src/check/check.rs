use crate::check::intrinsicck::InlineAsmCtxt;
use crate::errors::{self, LinkageType};

use super::compare_impl_item::check_type_bounds;
use super::compare_impl_item::{compare_impl_method, compare_impl_ty};
use super::*;
use rustc_attr as attr;
use rustc_errors::{Applicability, ErrorGuaranteed, MultiSpan};
use rustc_hir as hir;
use rustc_hir::def::{CtorKind, DefKind, Res};
use rustc_hir::def_id::{DefId, LocalDefId, CRATE_DEF_ID};
use rustc_hir::intravisit::Visitor;
use rustc_hir::{ItemKind, Node, PathSegment};
use rustc_infer::infer::opaque_types::ConstrainOpaqueTypeRegionVisitor;
use rustc_infer::infer::outlives::env::OutlivesEnvironment;
use rustc_infer::infer::{RegionVariableOrigin, TyCtxtInferExt};
use rustc_infer::traits::{Obligation, TraitEngineExt as _};
use rustc_lint_defs::builtin::REPR_TRANSPARENT_EXTERNAL_PRIVATE_FIELDS;
use rustc_middle::hir::nested_filter;
use rustc_middle::middle::stability::EvalResult;
use rustc_middle::traits::DefiningAnchor;
use rustc_middle::ty::fold::BottomUpFolder;
use rustc_middle::ty::layout::{LayoutError, MAX_SIMD_LANES};
use rustc_middle::ty::util::{Discr, IntTypeExt};
use rustc_middle::ty::GenericArgKind;
use rustc_middle::ty::{
    self, AdtDef, ParamEnv, RegionKind, Ty, TyCtxt, TypeSuperVisitable, TypeVisitable,
    TypeVisitableExt,
};
use rustc_session::lint::builtin::{UNINHABITED_STATIC, UNSUPPORTED_CALLING_CONVENTIONS};
use rustc_span::symbol::sym;
use rustc_span::{self, Span};
use rustc_target::abi::FieldIdx;
use rustc_target::spec::abi::Abi;
use rustc_trait_selection::traits::error_reporting::on_unimplemented::OnUnimplementedDirective;
use rustc_trait_selection::traits::error_reporting::TypeErrCtxtExt as _;
use rustc_trait_selection::traits::outlives_bounds::InferCtxtExt as _;
use rustc_trait_selection::traits::{self, ObligationCtxt, TraitEngine, TraitEngineExt as _};
use rustc_type_ir::fold::TypeFoldable;

use std::ops::ControlFlow;

pub fn check_abi(tcx: TyCtxt<'_>, hir_id: hir::HirId, span: Span, abi: Abi) {
    match tcx.sess.target.is_abi_supported(abi) {
        Some(true) => (),
        Some(false) => {
            struct_span_err!(
                tcx.sess,
                span,
                E0570,
                "`{abi}` is not a supported ABI for the current target",
            )
            .emit();
        }
        None => {
            tcx.struct_span_lint_hir(
                UNSUPPORTED_CALLING_CONVENTIONS,
                hir_id,
                span,
                "use of calling convention not supported on this target",
                |lint| lint,
            );
        }
    }

    // This ABI is only allowed on function pointers
    if abi == Abi::CCmseNonSecureCall {
        struct_span_err!(
            tcx.sess,
            span,
            E0781,
            "the `\"C-cmse-nonsecure-call\"` ABI is only allowed on function pointers"
        )
        .emit();
    }
}

fn check_struct(tcx: TyCtxt<'_>, def_id: LocalDefId) {
    let def = tcx.adt_def(def_id);
    let span = tcx.def_span(def_id);
    def.destructor(tcx); // force the destructor to be evaluated

    if def.repr().simd() {
        check_simd(tcx, span, def_id);
    }

    check_transparent(tcx, def);
    check_packed(tcx, span, def);
}

fn check_union(tcx: TyCtxt<'_>, def_id: LocalDefId) {
    let def = tcx.adt_def(def_id);
    let span = tcx.def_span(def_id);
    def.destructor(tcx); // force the destructor to be evaluated
    check_transparent(tcx, def);
    check_union_fields(tcx, span, def_id);
    check_packed(tcx, span, def);
}

/// Check that the fields of the `union` do not need dropping.
fn check_union_fields(tcx: TyCtxt<'_>, span: Span, item_def_id: LocalDefId) -> bool {
    let item_type = tcx.type_of(item_def_id).instantiate_identity();
    if let ty::Adt(def, args) = item_type.kind() {
        assert!(def.is_union());

        fn allowed_union_field<'tcx>(
            ty: Ty<'tcx>,
            tcx: TyCtxt<'tcx>,
            param_env: ty::ParamEnv<'tcx>,
        ) -> bool {
            // We don't just accept all !needs_drop fields, due to semver concerns.
            match ty.kind() {
                ty::Ref(..) => true, // references never drop (even mutable refs, which are non-Copy and hence fail the later check)
                ty::Tuple(tys) => {
                    // allow tuples of allowed types
                    tys.iter().all(|ty| allowed_union_field(ty, tcx, param_env))
                }
                ty::Array(elem, _len) => {
                    // Like `Copy`, we do *not* special-case length 0.
                    allowed_union_field(*elem, tcx, param_env)
                }
                _ => {
                    // Fallback case: allow `ManuallyDrop` and things that are `Copy`,
                    // also no need to report an error if the type is unresolved.
                    ty.ty_adt_def().is_some_and(|adt_def| adt_def.is_manually_drop())
                        || ty.is_copy_modulo_regions(tcx, param_env)
                        || ty.references_error()
                }
            }
        }

        let param_env = tcx.param_env(item_def_id);
        for field in &def.non_enum_variant().fields {
            let field_ty = tcx.normalize_erasing_regions(param_env, field.ty(tcx, args));

            if !allowed_union_field(field_ty, tcx, param_env) {
                let (field_span, ty_span) = match tcx.hir().get_if_local(field.did) {
                    // We are currently checking the type this field came from, so it must be local.
                    Some(Node::Field(field)) => (field.span, field.ty.span),
                    _ => unreachable!("mir field has to correspond to hir field"),
                };
                tcx.sess.emit_err(errors::InvalidUnionField {
                    field_span,
                    sugg: errors::InvalidUnionFieldSuggestion {
                        lo: ty_span.shrink_to_lo(),
                        hi: ty_span.shrink_to_hi(),
                    },
                    note: (),
                });
                return false;
            } else if field_ty.needs_drop(tcx, param_env) {
                // This should never happen. But we can get here e.g. in case of name resolution errors.
                tcx.sess.delay_span_bug(span, "we should never accept maybe-dropping union fields");
            }
        }
    } else {
        span_bug!(span, "unions must be ty::Adt, but got {:?}", item_type.kind());
    }
    true
}

/// Check that a `static` is inhabited.
fn check_static_inhabited(tcx: TyCtxt<'_>, def_id: LocalDefId) {
    // Make sure statics are inhabited.
    // Other parts of the compiler assume that there are no uninhabited places. In principle it
    // would be enough to check this for `extern` statics, as statics with an initializer will
    // have UB during initialization if they are uninhabited, but there also seems to be no good
    // reason to allow any statics to be uninhabited.
    let ty = tcx.type_of(def_id).instantiate_identity();
    let span = tcx.def_span(def_id);
    let layout = match tcx.layout_of(ParamEnv::reveal_all().and(ty)) {
        Ok(l) => l,
        // Foreign statics that overflow their allowed size should emit an error
        Err(LayoutError::SizeOverflow(_))
            if matches!(tcx.def_kind(def_id), DefKind::Static(_)
                if tcx.def_kind(tcx.local_parent(def_id)) == DefKind::ForeignMod) =>
        {
            tcx.sess.emit_err(errors::TooLargeStatic { span });
            return;
        }
        // Generic statics are rejected, but we still reach this case.
        Err(e) => {
            tcx.sess.delay_span_bug(span, format!("{e:?}"));
            return;
        }
    };
    if layout.abi.is_uninhabited() {
        tcx.struct_span_lint_hir(
            UNINHABITED_STATIC,
            tcx.hir().local_def_id_to_hir_id(def_id),
            span,
            "static of uninhabited type",
            |lint| {
                lint
                .note("uninhabited statics cannot be initialized, and any access would be an immediate error")
            },
        );
    }
}

/// Checks that an opaque type does not contain cycles and does not use `Self` or `T::Foo`
/// projections that would result in "inheriting lifetimes".
fn check_opaque(tcx: TyCtxt<'_>, id: hir::ItemId) {
    let item = tcx.hir().item(id);
    let hir::ItemKind::OpaqueTy(hir::OpaqueTy { origin, .. }) = item.kind else {
        tcx.sess.delay_span_bug(item.span, "expected opaque item");
        return;
    };

    // HACK(jynelson): trying to infer the type of `impl trait` breaks documenting
    // `async-std` (and `pub async fn` in general).
    // Since rustdoc doesn't care about the concrete type behind `impl Trait`, just don't look at it!
    // See https://github.com/rust-lang/rust/issues/75100
    if tcx.sess.opts.actually_rustdoc {
        return;
    }

    let args = GenericArgs::identity_for_item(tcx, item.owner_id);
    let span = tcx.def_span(item.owner_id.def_id);

    if !tcx.features().impl_trait_projections {
        check_opaque_for_inheriting_lifetimes(tcx, item.owner_id.def_id, span);
    }
    if tcx.type_of(item.owner_id.def_id).instantiate_identity().references_error() {
        return;
    }
    if check_opaque_for_cycles(tcx, item.owner_id.def_id, args, span, &origin).is_err() {
        return;
    }

    let _ = check_opaque_meets_bounds(tcx, item.owner_id.def_id, span, &origin);
}

/// Checks that an opaque type does not use `Self` or `T::Foo` projections that would result
/// in "inheriting lifetimes".
#[instrument(level = "debug", skip(tcx, span))]
pub(super) fn check_opaque_for_inheriting_lifetimes(
    tcx: TyCtxt<'_>,
    def_id: LocalDefId,
    span: Span,
) {
    let item = tcx.hir().expect_item(def_id);
    debug!(?item, ?span);

    struct ProhibitOpaqueVisitor<'tcx> {
        tcx: TyCtxt<'tcx>,
        opaque_identity_ty: Ty<'tcx>,
        parent_count: u32,
        references_parent_regions: bool,
        selftys: Vec<(Span, Option<String>)>,
    }

    impl<'tcx> ty::visit::TypeVisitor<TyCtxt<'tcx>> for ProhibitOpaqueVisitor<'tcx> {
        type BreakTy = Ty<'tcx>;

        fn visit_ty(&mut self, t: Ty<'tcx>) -> ControlFlow<Self::BreakTy> {
            debug!(?t, "root_visit_ty");
            if t == self.opaque_identity_ty {
                ControlFlow::Continue(())
            } else {
                t.visit_with(&mut ConstrainOpaqueTypeRegionVisitor {
                    tcx: self.tcx,
                    op: |region| {
                        if let ty::ReEarlyBound(ty::EarlyBoundRegion { index, .. }) = *region
                            && index < self.parent_count
                        {
                            self.references_parent_regions= true;
                        }
                    },
                });
                if self.references_parent_regions {
                    ControlFlow::Break(t)
                } else {
                    ControlFlow::Continue(())
                }
            }
        }
    }

    impl<'tcx> Visitor<'tcx> for ProhibitOpaqueVisitor<'tcx> {
        type NestedFilter = nested_filter::OnlyBodies;

        fn nested_visit_map(&mut self) -> Self::Map {
            self.tcx.hir()
        }

        fn visit_ty(&mut self, arg: &'tcx hir::Ty<'tcx>) {
            match arg.kind {
                hir::TyKind::Path(hir::QPath::Resolved(None, path)) => match &path.segments {
                    [PathSegment { res: Res::SelfTyParam { .. }, .. }] => {
                        let impl_ty_name = None;
                        self.selftys.push((path.span, impl_ty_name));
                    }
                    [PathSegment { res: Res::SelfTyAlias { alias_to: def_id, .. }, .. }] => {
                        let impl_ty_name = Some(self.tcx.def_path_str(*def_id));
                        self.selftys.push((path.span, impl_ty_name));
                    }
                    _ => {}
                },
                _ => {}
            }
            hir::intravisit::walk_ty(self, arg);
        }
    }

    if let ItemKind::OpaqueTy(&hir::OpaqueTy {
        origin: hir::OpaqueTyOrigin::AsyncFn(..) | hir::OpaqueTyOrigin::FnReturn(..),
        ..
    }) = item.kind
    {
        let args = GenericArgs::identity_for_item(tcx, def_id);
        let opaque_identity_ty = Ty::new_opaque(tcx, def_id.to_def_id(), args);
        let mut visitor = ProhibitOpaqueVisitor {
            opaque_identity_ty,
            parent_count: tcx.generics_of(def_id).parent_count as u32,
            references_parent_regions: false,
            tcx,
            selftys: vec![],
        };
        let prohibit_opaque = tcx
            .explicit_item_bounds(def_id)
            .instantiate_identity_iter_copied()
            .try_for_each(|(predicate, _)| predicate.visit_with(&mut visitor));

        if let Some(ty) = prohibit_opaque.break_value() {
            visitor.visit_item(&item);
            let is_async = match item.kind {
                ItemKind::OpaqueTy(hir::OpaqueTy { origin, .. }) => {
                    matches!(origin, hir::OpaqueTyOrigin::AsyncFn(..))
                }
                _ => unreachable!(),
            };

            let mut err = feature_err(
                &tcx.sess.parse_sess,
                sym::impl_trait_projections,
                span,
                format!(
                    "`{}` return type cannot contain a projection or `Self` that references \
                    lifetimes from a parent scope",
                    if is_async { "async fn" } else { "impl Trait" },
                ),
            );
            for (span, name) in visitor.selftys {
                err.span_suggestion(
                    span,
                    "consider spelling out the type instead",
                    name.unwrap_or_else(|| format!("{ty:?}")),
                    Applicability::MaybeIncorrect,
                );
            }
            err.emit();
        }
    }
}

/// Checks that an opaque type does not contain cycles.
pub(super) fn check_opaque_for_cycles<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    args: GenericArgsRef<'tcx>,
    span: Span,
    origin: &hir::OpaqueTyOrigin,
) -> Result<(), ErrorGuaranteed> {
    if tcx.try_expand_impl_trait_type(def_id.to_def_id(), args).is_err() {
        let reported = match origin {
            hir::OpaqueTyOrigin::AsyncFn(..) => async_opaque_type_cycle_error(tcx, span),
            _ => opaque_type_cycle_error(tcx, def_id, span),
        };
        Err(reported)
    } else {
        Ok(())
    }
}

/// Check that the concrete type behind `impl Trait` actually implements `Trait`.
///
/// This is mostly checked at the places that specify the opaque type, but we
/// check those cases in the `param_env` of that function, which may have
/// bounds not on this opaque type:
///
/// ```ignore (illustrative)
/// type X<T> = impl Clone;
/// fn f<T: Clone>(t: T) -> X<T> {
///     t
/// }
/// ```
///
/// Without this check the above code is incorrectly accepted: we would ICE if
/// some tried, for example, to clone an `Option<X<&mut ()>>`.
#[instrument(level = "debug", skip(tcx))]
fn check_opaque_meets_bounds<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    span: Span,
    origin: &hir::OpaqueTyOrigin,
) -> Result<(), ErrorGuaranteed> {
    let defining_use_anchor = match *origin {
        hir::OpaqueTyOrigin::FnReturn(did) | hir::OpaqueTyOrigin::AsyncFn(did) => did,
        hir::OpaqueTyOrigin::TyAlias { .. } => tcx.impl_trait_parent(def_id),
    };
    let param_env = tcx.param_env(defining_use_anchor);

    let infcx = tcx
        .infer_ctxt()
        .with_opaque_type_inference(DefiningAnchor::Bind(defining_use_anchor))
        .build();
    let ocx = ObligationCtxt::new(&infcx);

    let args = GenericArgs::identity_for_item(tcx, def_id.to_def_id());
    let opaque_ty = Ty::new_opaque(tcx, def_id.to_def_id(), args);

    // `ReErased` regions appear in the "parent_args" of closures/generators.
    // We're ignoring them here and replacing them with fresh region variables.
    // See tests in ui/type-alias-impl-trait/closure_{parent_args,wf_outlives}.rs.
    //
    // FIXME: Consider wrapping the hidden type in an existential `Binder` and instantiating it
    // here rather than using ReErased.
    let hidden_ty = tcx.type_of(def_id.to_def_id()).instantiate(tcx, args);
    let hidden_ty = tcx.fold_regions(hidden_ty, |re, _dbi| match re.kind() {
        ty::ReErased => infcx.next_region_var(RegionVariableOrigin::MiscVariable(span)),
        _ => re,
    });

    let misc_cause = traits::ObligationCause::misc(span, def_id);

    match ocx.eq(&misc_cause, param_env, opaque_ty, hidden_ty) {
        Ok(()) => {}
        Err(ty_err) => {
            let ty_err = ty_err.to_string(tcx);
            return Err(tcx.sess.delay_span_bug(
                span,
                format!("could not unify `{hidden_ty}` with revealed type:\n{ty_err}"),
            ));
        }
    }

    // Additionally require the hidden type to be well-formed with only the generics of the opaque type.
    // Defining use functions may have more bounds than the opaque type, which is ok, as long as the
    // hidden type is well formed even without those bounds.
    let predicate =
        ty::Binder::dummy(ty::PredicateKind::Clause(ty::ClauseKind::WellFormed(hidden_ty.into())));
    ocx.register_obligation(Obligation::new(tcx, misc_cause.clone(), param_env, predicate));

    // Check that all obligations are satisfied by the implementation's
    // version.
    let errors = ocx.select_all_or_error();
    if !errors.is_empty() {
        let guar = infcx.err_ctxt().report_fulfillment_errors(&errors);
        return Err(guar);
    }
    match origin {
        // Checked when type checking the function containing them.
        hir::OpaqueTyOrigin::FnReturn(..) | hir::OpaqueTyOrigin::AsyncFn(..) => {}
        // Nested opaque types occur only in associated types:
        // ` type Opaque<T> = impl Trait<&'static T, AssocTy = impl Nested>; `
        // They can only be referenced as `<Opaque<T> as Trait<&'static T>>::AssocTy`.
        // We don't have to check them here because their well-formedness follows from the WF of
        // the projection input types in the defining- and use-sites.
        hir::OpaqueTyOrigin::TyAlias { .. }
            if tcx.def_kind(tcx.parent(def_id.to_def_id())) == DefKind::OpaqueTy => {}
        // Can have different predicates to their defining use
        hir::OpaqueTyOrigin::TyAlias { .. } => {
            let wf_tys = ocx.assumed_wf_types_and_report_errors(param_env, def_id)?;
            let implied_bounds = infcx.implied_bounds_tys(param_env, def_id, wf_tys);
            let outlives_env = OutlivesEnvironment::with_bounds(param_env, implied_bounds);
            ocx.resolve_regions_and_report_errors(defining_use_anchor, &outlives_env)?;
        }
    }
    // Check that any hidden types found during wf checking match the hidden types that `type_of` sees.
    for (key, mut ty) in infcx.take_opaque_types() {
        ty.hidden_type.ty = infcx.resolve_vars_if_possible(ty.hidden_type.ty);
        sanity_check_found_hidden_type(tcx, key, ty.hidden_type, defining_use_anchor, origin)?;
    }
    Ok(())
}

fn sanity_check_found_hidden_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    key: ty::OpaqueTypeKey<'tcx>,
    mut ty: ty::OpaqueHiddenType<'tcx>,
    defining_use_anchor: LocalDefId,
    origin: &hir::OpaqueTyOrigin,
) -> Result<(), ErrorGuaranteed> {
    if ty.ty.is_ty_var() {
        // Nothing was actually constrained.
        return Ok(());
    }
    if let ty::Alias(ty::Opaque, alias) = ty.ty.kind() {
        if alias.def_id == key.def_id.to_def_id() && alias.args == key.args {
            // Nothing was actually constrained, this is an opaque usage that was
            // only discovered to be opaque after inference vars resolved.
            return Ok(());
        }
    }
    // Closures frequently end up containing erased lifetimes in their final representation.
    // These correspond to lifetime variables that never got resolved, so we patch this up here.
    ty.ty = ty.ty.fold_with(&mut BottomUpFolder {
        tcx,
        ty_op: |t| t,
        ct_op: |c| c,
        lt_op: |l| match l.kind() {
            RegionKind::ReVar(_) => tcx.lifetimes.re_erased,
            _ => l,
        },
    });
    // Get the hidden type.
    let mut hidden_ty = tcx.type_of(key.def_id).instantiate(tcx, key.args);
    if let hir::OpaqueTyOrigin::FnReturn(..) | hir::OpaqueTyOrigin::AsyncFn(..) = origin {
        if hidden_ty != ty.ty {
            hidden_ty = find_and_apply_rpit_args(
                tcx,
                hidden_ty,
                defining_use_anchor.to_def_id(),
                key.def_id.to_def_id(),
            )?;
        }
    }

    // If the hidden types differ, emit a type mismatch diagnostic.
    if hidden_ty == ty.ty {
        Ok(())
    } else {
        let span = tcx.def_span(key.def_id);
        let other = ty::OpaqueHiddenType { ty: hidden_ty, span };
        Err(ty.report_mismatch(&other, key.def_id, tcx).emit())
    }
}

/// In case it is in a nested opaque type, find that opaque type's
/// usage in the function signature and use the generic arguments from the usage site.
/// We need to do because RPITs ignore the lifetimes of the function,
/// as they have their own copies of all the lifetimes they capture.
/// So the only way to get the lifetimes represented in terms of the function,
/// is to look how they are used in the function signature (or do some other fancy
/// recording of this mapping at ast -> hir lowering time).
///
/// As an example:
/// ```text
/// trait Id {
///     type Assoc;
/// }
/// impl<'a> Id for &'a () {
///     type Assoc = &'a ();
/// }
/// fn func<'a>(x: &'a ()) -> impl Id<Assoc = impl Sized + 'a> { x }
/// // desugared to
/// fn func<'a>(x: &'a () -> Outer<'a> where <Outer<'a> as Id>::Assoc = Inner<'a> {
///     // Note that in contrast to other nested items, RPIT type aliases can
///     // access their parents' generics.
///
///     // hidden type is `&'aDupOuter ()`
///     // During wfcheck the hidden type of `Inner<'aDupOuter>` is `&'a ()`, but
///     // `typeof(Inner<'aDupOuter>) = &'aDupOuter ()`.
///     // So we walk the signature of `func` to find the use of `Inner<'a>`
///     // and then use that to replace the lifetimes in the hidden type, obtaining
///     // `&'a ()`.
///     type Outer<'aDupOuter> = impl Id<Assoc = Inner<'aDupOuter>>;
///
///     // hidden type is `&'aDupInner ()`
///     type Inner<'aDupInner> = impl Sized + 'aDupInner;
///
///     x
/// }
/// ```
fn find_and_apply_rpit_args<'tcx>(
    tcx: TyCtxt<'tcx>,
    mut hidden_ty: Ty<'tcx>,
    function: DefId,
    opaque: DefId,
) -> Result<Ty<'tcx>, ErrorGuaranteed> {
    // Find use of the RPIT in the function signature and thus find the right args to
    // convert it into the parameter space of the function signature. This is needed,
    // because that's what `type_of` returns, against which we compare later.
    let ret = tcx.fn_sig(function).instantiate_identity().output();
    struct Visitor<'tcx> {
        tcx: TyCtxt<'tcx>,
        opaque: DefId,
        function: DefId,
        seen: FxHashSet<DefId>,
    }
    impl<'tcx> ty::TypeVisitor<TyCtxt<'tcx>> for Visitor<'tcx> {
        type BreakTy = GenericArgsRef<'tcx>;

        #[instrument(level = "trace", skip(self), ret)]
        fn visit_ty(&mut self, t: Ty<'tcx>) -> ControlFlow<Self::BreakTy> {
            trace!("{:#?}", t.kind());
            match t.kind() {
                ty::Alias(ty::Opaque, alias) => {
                    trace!(?alias.def_id);
                    if alias.def_id == self.opaque {
                        return ControlFlow::Break(alias.args);
                    } else if self.seen.insert(alias.def_id) {
                        for clause in self
                            .tcx
                            .explicit_item_bounds(alias.def_id)
                            .iter_instantiated_copied(self.tcx, alias.args)
                        {
                            trace!(?clause);
                            clause.visit_with(self)?;
                        }
                    }
                }
                ty::Alias(ty::Projection, alias) => {
                    if self.tcx.is_impl_trait_in_trait(alias.def_id)
                        && self.tcx.impl_trait_in_trait_parent_fn(alias.def_id) == self.function
                    {
                        // If we're lowering to associated item, install the opaque type which is just
                        // the `type_of` of the trait's associated item. If we're using the old lowering
                        // strategy, then just reinterpret the associated type like an opaque :^)
                        self.tcx
                            .type_of(alias.def_id)
                            .instantiate(self.tcx, alias.args)
                            .visit_with(self)?;
                    }
                }
                ty::Alias(ty::Weak, alias) => {
                    self.tcx
                        .type_of(alias.def_id)
                        .instantiate(self.tcx, alias.args)
                        .visit_with(self)?;
                }
                _ => (),
            }

            t.super_visit_with(self)
        }
    }
    if let ControlFlow::Break(args) =
        ret.visit_with(&mut Visitor { tcx, function, opaque, seen: Default::default() })
    {
        trace!(?args);
        trace!("expected: {hidden_ty:#?}");
        hidden_ty = ty::EarlyBinder::bind(hidden_ty).instantiate(tcx, args);
        trace!("expected: {hidden_ty:#?}");
    } else {
        tcx.sess
            .delay_span_bug(tcx.def_span(function), format!("{ret:?} does not contain {opaque:?}"));
    }
    Ok(hidden_ty)
}

fn is_enum_of_nonnullable_ptr<'tcx>(
    tcx: TyCtxt<'tcx>,
    adt_def: AdtDef<'tcx>,
    args: GenericArgsRef<'tcx>,
) -> bool {
    if adt_def.repr().inhibit_enum_layout_opt() {
        return false;
    }

    let [var_one, var_two] = &adt_def.variants().raw[..] else {
        return false;
    };
    let (([], [field]) | ([field], [])) = (&var_one.fields.raw[..], &var_two.fields.raw[..]) else {
        return false;
    };
    matches!(field.ty(tcx, args).kind(), ty::FnPtr(..) | ty::Ref(..))
}

fn check_static_linkage(tcx: TyCtxt<'_>, def_id: LocalDefId) {
    if tcx.codegen_fn_attrs(def_id).import_linkage.is_some() {
        if match tcx.type_of(def_id).instantiate_identity().kind() {
            ty::RawPtr(_) => false,
            ty::Adt(adt_def, args) => !is_enum_of_nonnullable_ptr(tcx, *adt_def, *args),
            _ => true,
        } {
            tcx.sess.emit_err(LinkageType { span: tcx.def_span(def_id) });
        }
    }
}

fn check_item_type(tcx: TyCtxt<'_>, id: hir::ItemId) {
    debug!(
        "check_item_type(it.def_id={:?}, it.name={})",
        id.owner_id,
        tcx.def_path_str(id.owner_id)
    );
    let _indenter = indenter();
    match tcx.def_kind(id.owner_id) {
        DefKind::Static(..) => {
            tcx.ensure().typeck(id.owner_id.def_id);
            maybe_check_static_with_link_section(tcx, id.owner_id.def_id);
            check_static_inhabited(tcx, id.owner_id.def_id);
            check_static_linkage(tcx, id.owner_id.def_id);
        }
        DefKind::Const => {
            tcx.ensure().typeck(id.owner_id.def_id);
        }
        DefKind::Enum => {
            check_enum(tcx, id.owner_id.def_id);
        }
        DefKind::Fn => {} // entirely within check_item_body
        DefKind::Impl { of_trait } => {
            if of_trait && let Some(impl_trait_ref) = tcx.impl_trait_ref(id.owner_id) {
                check_impl_items_against_trait(
                    tcx,
                    id.owner_id.def_id,
                    impl_trait_ref.instantiate_identity(),
                );
                check_on_unimplemented(tcx, id);
            }
        }
        DefKind::Trait => {
            let assoc_items = tcx.associated_items(id.owner_id);
            check_on_unimplemented(tcx, id);

            for &assoc_item in assoc_items.in_definition_order() {
                match assoc_item.kind {
                    ty::AssocKind::Fn => {
                        let abi = tcx.fn_sig(assoc_item.def_id).skip_binder().abi();
                        fn_maybe_err(tcx, assoc_item.ident(tcx).span, abi);
                    }
                    ty::AssocKind::Type if assoc_item.defaultness(tcx).has_value() => {
                        let trait_args =
                            GenericArgs::identity_for_item(tcx, id.owner_id);
                        let _: Result<_, rustc_errors::ErrorGuaranteed> = check_type_bounds(
                            tcx,
                            assoc_item,
                            assoc_item,
                            ty::TraitRef::new(tcx, id.owner_id.to_def_id(), trait_args),
                        );
                    }
                    _ => {}
                }
            }
        }
        DefKind::Struct => {
            check_struct(tcx, id.owner_id.def_id);
        }
        DefKind::Union => {
            check_union(tcx, id.owner_id.def_id);
        }
        DefKind::OpaqueTy => {
            let origin = tcx.opaque_type_origin(id.owner_id.def_id);
            if let hir::OpaqueTyOrigin::FnReturn(fn_def_id) | hir::OpaqueTyOrigin::AsyncFn(fn_def_id) = origin
                && let hir::Node::TraitItem(trait_item) = tcx.hir().get_by_def_id(fn_def_id)
                && let (_, hir::TraitFn::Required(..)) = trait_item.expect_fn()
            {
                // Skip opaques from RPIT in traits with no default body.
            } else {
                check_opaque(tcx, id);
            }
        }
        DefKind::TyAlias => {
            let pty_ty = tcx.type_of(id.owner_id).instantiate_identity();
            let generics = tcx.generics_of(id.owner_id);
            check_type_params_are_used(tcx, &generics, pty_ty);
        }
        DefKind::ForeignMod => {
            let it = tcx.hir().item(id);
            let hir::ItemKind::ForeignMod { abi, items } = it.kind else {
                return;
            };
            check_abi(tcx, it.hir_id(), it.span, abi);

            match abi {
                Abi::RustIntrinsic => {
                    for item in items {
                        let item = tcx.hir().foreign_item(item.id);
                        intrinsic::check_intrinsic_type(tcx, item);
                    }
                }

                Abi::PlatformIntrinsic => {
                    for item in items {
                        let item = tcx.hir().foreign_item(item.id);
                        intrinsic::check_platform_intrinsic_type(tcx, item);
                    }
                }

                _ => {
                    for item in items {
                        let def_id = item.id.owner_id.def_id;
                        let generics = tcx.generics_of(def_id);
                        let own_counts = generics.own_counts();
                        if generics.params.len() - own_counts.lifetimes != 0 {
                            let (kinds, kinds_pl, egs) = match (own_counts.types, own_counts.consts)
                            {
                                (_, 0) => ("type", "types", Some("u32")),
                                // We don't specify an example value, because we can't generate
                                // a valid value for any type.
                                (0, _) => ("const", "consts", None),
                                _ => ("type or const", "types or consts", None),
                            };
                            struct_span_err!(
                                tcx.sess,
                                item.span,
                                E0044,
                                "foreign items may not have {kinds} parameters",
                            )
                            .span_label(item.span, format!("can't have {kinds} parameters"))
                            .help(
                                // FIXME: once we start storing spans for type arguments, turn this
                                // into a suggestion.
                                format!(
                                    "replace the {} parameters with concrete {}{}",
                                    kinds,
                                    kinds_pl,
                                    egs.map(|egs| format!(" like `{egs}`")).unwrap_or_default(),
                                ),
                            )
                            .emit();
                        }

                        let item = tcx.hir().foreign_item(item.id);
                        match &item.kind {
                            hir::ForeignItemKind::Fn(fn_decl, _, _) => {
                                require_c_abi_if_c_variadic(tcx, fn_decl, abi, item.span);
                            }
                            hir::ForeignItemKind::Static(..) => {
                                check_static_inhabited(tcx, def_id);
                                check_static_linkage(tcx, def_id);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        DefKind::GlobalAsm => {
            let it = tcx.hir().item(id);
            let hir::ItemKind::GlobalAsm(asm) = it.kind else { span_bug!(it.span, "DefKind::GlobalAsm but got {:#?}", it) };
            InlineAsmCtxt::new_global_asm(tcx).check_asm(asm, id.owner_id.def_id);
        }
        _ => {}
    }
}

pub(super) fn check_on_unimplemented(tcx: TyCtxt<'_>, item: hir::ItemId) {
    // an error would be reported if this fails.
    let _ = OnUnimplementedDirective::of_item(tcx, item.owner_id.to_def_id());
}

pub(super) fn check_specialization_validity<'tcx>(
    tcx: TyCtxt<'tcx>,
    trait_def: &ty::TraitDef,
    trait_item: ty::AssocItem,
    impl_id: DefId,
    impl_item: DefId,
) {
    let Ok(ancestors) = trait_def.ancestors(tcx, impl_id) else { return };
    let mut ancestor_impls = ancestors.skip(1).filter_map(|parent| {
        if parent.is_from_trait() {
            None
        } else {
            Some((parent, parent.item(tcx, trait_item.def_id)))
        }
    });

    let opt_result = ancestor_impls.find_map(|(parent_impl, parent_item)| {
        match parent_item {
            // Parent impl exists, and contains the parent item we're trying to specialize, but
            // doesn't mark it `default`.
            Some(parent_item) if traits::impl_item_is_final(tcx, &parent_item) => {
                Some(Err(parent_impl.def_id()))
            }

            // Parent impl contains item and makes it specializable.
            Some(_) => Some(Ok(())),

            // Parent impl doesn't mention the item. This means it's inherited from the
            // grandparent. In that case, if parent is a `default impl`, inherited items use the
            // "defaultness" from the grandparent, else they are final.
            None => {
                if tcx.defaultness(parent_impl.def_id()).is_default() {
                    None
                } else {
                    Some(Err(parent_impl.def_id()))
                }
            }
        }
    });

    // If `opt_result` is `None`, we have only encountered `default impl`s that don't contain the
    // item. This is allowed, the item isn't actually getting specialized here.
    let result = opt_result.unwrap_or(Ok(()));

    if let Err(parent_impl) = result {
        if !tcx.is_impl_trait_in_trait(impl_item) {
            report_forbidden_specialization(tcx, impl_item, parent_impl);
        } else {
            tcx.sess.delay_span_bug(
                DUMMY_SP,
                format!("parent item: {parent_impl:?} not marked as default"),
            );
        }
    }
}

fn check_impl_items_against_trait<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_id: LocalDefId,
    impl_trait_ref: ty::TraitRef<'tcx>,
) {
    // If the trait reference itself is erroneous (so the compilation is going
    // to fail), skip checking the items here -- the `impl_item` table in `tcx`
    // isn't populated for such impls.
    if impl_trait_ref.references_error() {
        return;
    }

    let impl_item_refs = tcx.associated_item_def_ids(impl_id);

    // Negative impls are not expected to have any items
    match tcx.impl_polarity(impl_id) {
        ty::ImplPolarity::Reservation | ty::ImplPolarity::Positive => {}
        ty::ImplPolarity::Negative => {
            if let [first_item_ref, ..] = impl_item_refs {
                let first_item_span = tcx.def_span(first_item_ref);
                struct_span_err!(
                    tcx.sess,
                    first_item_span,
                    E0749,
                    "negative impls cannot have any items"
                )
                .emit();
            }
            return;
        }
    }

    let trait_def = tcx.trait_def(impl_trait_ref.def_id);

    for &impl_item in impl_item_refs {
        let ty_impl_item = tcx.associated_item(impl_item);
        let ty_trait_item = if let Some(trait_item_id) = ty_impl_item.trait_item_def_id {
            tcx.associated_item(trait_item_id)
        } else {
            // Checked in `associated_item`.
            tcx.sess.delay_span_bug(tcx.def_span(impl_item), "missing associated item in trait");
            continue;
        };
        match ty_impl_item.kind {
            ty::AssocKind::Const => {
                let _ = tcx.compare_impl_const((
                    impl_item.expect_local(),
                    ty_impl_item.trait_item_def_id.unwrap(),
                ));
            }
            ty::AssocKind::Fn => {
                compare_impl_method(tcx, ty_impl_item, ty_trait_item, impl_trait_ref);
            }
            ty::AssocKind::Type => {
                compare_impl_ty(tcx, ty_impl_item, ty_trait_item, impl_trait_ref);
            }
        }

        check_specialization_validity(
            tcx,
            trait_def,
            ty_trait_item,
            impl_id.to_def_id(),
            impl_item,
        );
    }

    if let Ok(ancestors) = trait_def.ancestors(tcx, impl_id.to_def_id()) {
        // Check for missing items from trait
        let mut missing_items = Vec::new();

        let mut must_implement_one_of: Option<&[Ident]> =
            trait_def.must_implement_one_of.as_deref();

        for &trait_item_id in tcx.associated_item_def_ids(impl_trait_ref.def_id) {
            let leaf_def = ancestors.leaf_def(tcx, trait_item_id);

            let is_implemented = leaf_def
                .as_ref()
                .is_some_and(|node_item| node_item.item.defaultness(tcx).has_value());

            if !is_implemented && tcx.defaultness(impl_id).is_final() {
                missing_items.push(tcx.associated_item(trait_item_id));
            }

            // true if this item is specifically implemented in this impl
            let is_implemented_here =
                leaf_def.as_ref().is_some_and(|node_item| !node_item.defining_node.is_from_trait());

            if !is_implemented_here {
                let full_impl_span =
                    tcx.hir().span_with_body(tcx.hir().local_def_id_to_hir_id(impl_id));
                match tcx.eval_default_body_stability(trait_item_id, full_impl_span) {
                    EvalResult::Deny { feature, reason, issue, .. } => default_body_is_unstable(
                        tcx,
                        full_impl_span,
                        trait_item_id,
                        feature,
                        reason,
                        issue,
                    ),

                    // Unmarked default bodies are considered stable (at least for now).
                    EvalResult::Allow | EvalResult::Unmarked => {}
                }
            }

            if let Some(required_items) = &must_implement_one_of {
                if is_implemented_here {
                    let trait_item = tcx.associated_item(trait_item_id);
                    if required_items.contains(&trait_item.ident(tcx)) {
                        must_implement_one_of = None;
                    }
                }
            }

            if let Some(leaf_def) = &leaf_def
                && !leaf_def.is_final()
                && let def_id = leaf_def.item.def_id
                && tcx.impl_method_has_trait_impl_trait_tys(def_id)
            {
                let def_kind = tcx.def_kind(def_id);
                let descr = tcx.def_kind_descr(def_kind, def_id);
                let (msg, feature) = if tcx.asyncness(def_id).is_async() {
                    (
                        format!("async {descr} in trait cannot be specialized"),
                        sym::async_fn_in_trait,
                    )
                } else {
                    (
                        format!(
                            "{descr} with return-position `impl Trait` in trait cannot be specialized"
                        ),
                        sym::return_position_impl_trait_in_trait,
                    )
                };
                tcx.sess
                    .struct_span_err(tcx.def_span(def_id), msg)
                    .note(format!(
                        "specialization behaves in inconsistent and \
                        surprising ways with `#![feature({feature})]`, \
                        and for now is disallowed"
                    ))
                    .emit();
            }
        }

        if !missing_items.is_empty() {
            let full_impl_span =
                tcx.hir().span_with_body(tcx.hir().local_def_id_to_hir_id(impl_id));
            missing_items_err(tcx, impl_id, &missing_items, full_impl_span);
        }

        if let Some(missing_items) = must_implement_one_of {
            let attr_span = tcx
                .get_attr(impl_trait_ref.def_id, sym::rustc_must_implement_one_of)
                .map(|attr| attr.span);

            missing_items_must_implement_one_of_err(
                tcx,
                tcx.def_span(impl_id),
                missing_items,
                attr_span,
            );
        }
    }
}

pub fn check_simd(tcx: TyCtxt<'_>, sp: Span, def_id: LocalDefId) {
    let t = tcx.type_of(def_id).instantiate_identity();
    if let ty::Adt(def, args) = t.kind()
        && def.is_struct()
    {
        let fields = &def.non_enum_variant().fields;
        if fields.is_empty() {
            struct_span_err!(tcx.sess, sp, E0075, "SIMD vector cannot be empty").emit();
            return;
        }
        let e = fields[FieldIdx::from_u32(0)].ty(tcx, args);
        if !fields.iter().all(|f| f.ty(tcx, args) == e) {
            struct_span_err!(tcx.sess, sp, E0076, "SIMD vector should be homogeneous")
                .span_label(sp, "SIMD elements must have the same type")
                .emit();
            return;
        }

        let len = if let ty::Array(_ty, c) = e.kind() {
            c.try_eval_target_usize(tcx, tcx.param_env(def.did()))
        } else {
            Some(fields.len() as u64)
        };
        if let Some(len) = len {
            if len == 0 {
                struct_span_err!(tcx.sess, sp, E0075, "SIMD vector cannot be empty").emit();
                return;
            } else if len > MAX_SIMD_LANES {
                struct_span_err!(
                    tcx.sess,
                    sp,
                    E0075,
                    "SIMD vector cannot have more than {MAX_SIMD_LANES} elements",
                )
                .emit();
                return;
            }
        }

        // Check that we use types valid for use in the lanes of a SIMD "vector register"
        // These are scalar types which directly match a "machine" type
        // Yes: Integers, floats, "thin" pointers
        // No: char, "fat" pointers, compound types
        match e.kind() {
            ty::Param(_) => (), // pass struct<T>(T, T, T, T) through, let monomorphization catch errors
            ty::Int(_) | ty::Uint(_) | ty::Float(_) | ty::RawPtr(_) => (), // struct(u8, u8, u8, u8) is ok
            ty::Array(t, _) if matches!(t.kind(), ty::Param(_)) => (), // pass struct<T>([T; N]) through, let monomorphization catch errors
            ty::Array(t, _clen)
                if matches!(
                    t.kind(),
                    ty::Int(_) | ty::Uint(_) | ty::Float(_) | ty::RawPtr(_)
                ) =>
            { /* struct([f32; 4]) is ok */ }
            _ => {
                struct_span_err!(
                    tcx.sess,
                    sp,
                    E0077,
                    "SIMD vector element type should be a \
                        primitive scalar (integer/float/pointer) type"
                )
                .emit();
                return;
            }
        }
    }
}

pub(super) fn check_packed(tcx: TyCtxt<'_>, sp: Span, def: ty::AdtDef<'_>) {
    let repr = def.repr();
    if repr.packed() {
        for attr in tcx.get_attrs(def.did(), sym::repr) {
            for r in attr::parse_repr_attr(&tcx.sess, attr) {
                if let attr::ReprPacked(pack) = r
                && let Some(repr_pack) = repr.pack
                && pack as u64 != repr_pack.bytes()
            {
                        struct_span_err!(
                            tcx.sess,
                            sp,
                            E0634,
                            "type has conflicting packed representation hints"
                        )
                        .emit();
            }
            }
        }
        if repr.align.is_some() {
            struct_span_err!(
                tcx.sess,
                sp,
                E0587,
                "type has conflicting packed and align representation hints"
            )
            .emit();
        } else {
            if let Some(def_spans) = check_packed_inner(tcx, def.did(), &mut vec![]) {
                let mut err = struct_span_err!(
                    tcx.sess,
                    sp,
                    E0588,
                    "packed type cannot transitively contain a `#[repr(align)]` type"
                );

                err.span_note(
                    tcx.def_span(def_spans[0].0),
                    format!("`{}` has a `#[repr(align)]` attribute", tcx.item_name(def_spans[0].0)),
                );

                if def_spans.len() > 2 {
                    let mut first = true;
                    for (adt_def, span) in def_spans.iter().skip(1).rev() {
                        let ident = tcx.item_name(*adt_def);
                        err.span_note(
                            *span,
                            if first {
                                format!(
                                    "`{}` contains a field of type `{}`",
                                    tcx.type_of(def.did()).instantiate_identity(),
                                    ident
                                )
                            } else {
                                format!("...which contains a field of type `{ident}`")
                            },
                        );
                        first = false;
                    }
                }

                err.emit();
            }
        }
    }
}

pub(super) fn check_packed_inner(
    tcx: TyCtxt<'_>,
    def_id: DefId,
    stack: &mut Vec<DefId>,
) -> Option<Vec<(DefId, Span)>> {
    if let ty::Adt(def, args) = tcx.type_of(def_id).instantiate_identity().kind() {
        if def.is_struct() || def.is_union() {
            if def.repr().align.is_some() {
                return Some(vec![(def.did(), DUMMY_SP)]);
            }

            stack.push(def_id);
            for field in &def.non_enum_variant().fields {
                if let ty::Adt(def, _) = field.ty(tcx, args).kind()
                    && !stack.contains(&def.did())
                    && let Some(mut defs) = check_packed_inner(tcx, def.did(), stack)
                {
                    defs.push((def.did(), field.ident(tcx).span));
                    return Some(defs);
                }
            }
            stack.pop();
        }
    }

    None
}

pub(super) fn check_transparent<'tcx>(tcx: TyCtxt<'tcx>, adt: ty::AdtDef<'tcx>) {
    if !adt.repr().transparent() {
        return;
    }

    if adt.is_union() && !tcx.features().transparent_unions {
        feature_err(
            &tcx.sess.parse_sess,
            sym::transparent_unions,
            tcx.def_span(adt.did()),
            "transparent unions are unstable",
        )
        .emit();
    }

    if adt.variants().len() != 1 {
        bad_variant_count(tcx, adt, tcx.def_span(adt.did()), adt.did());
        // Don't bother checking the fields.
        return;
    }

    // For each field, figure out if it's known to be a ZST and align(1), with "known"
    // respecting #[non_exhaustive] attributes.
    let field_infos = adt.all_fields().map(|field| {
        let ty = field.ty(tcx, GenericArgs::identity_for_item(tcx, field.did));
        let param_env = tcx.param_env(field.did);
        let layout = tcx.layout_of(param_env.and(ty));
        // We are currently checking the type this field came from, so it must be local
        let span = tcx.hir().span_if_local(field.did).unwrap();
        let zst = layout.is_ok_and(|layout| layout.is_zst());
        let align = layout.ok().map(|layout| layout.align.abi.bytes());
        if !zst {
            return (span, zst, align, None);
        }

        fn check_non_exhaustive<'tcx>(
            tcx: TyCtxt<'tcx>,
            t: Ty<'tcx>,
        ) -> ControlFlow<(&'static str, DefId, GenericArgsRef<'tcx>, bool)> {
            match t.kind() {
                ty::Tuple(list) => list.iter().try_for_each(|t| check_non_exhaustive(tcx, t)),
                ty::Array(ty, _) => check_non_exhaustive(tcx, *ty),
                ty::Adt(def, subst) => {
                    if !def.did().is_local() {
                        let non_exhaustive = def.is_variant_list_non_exhaustive()
                            || def
                                .variants()
                                .iter()
                                .any(ty::VariantDef::is_field_list_non_exhaustive);
                        let has_priv = def.all_fields().any(|f| !f.vis.is_public());
                        if non_exhaustive || has_priv {
                            return ControlFlow::Break((
                                def.descr(),
                                def.did(),
                                subst,
                                non_exhaustive,
                            ));
                        }
                    }
                    def.all_fields()
                        .map(|field| field.ty(tcx, subst))
                        .try_for_each(|t| check_non_exhaustive(tcx, t))
                }
                _ => ControlFlow::Continue(()),
            }
        }

        (span, zst, align, check_non_exhaustive(tcx, ty).break_value())
    });

    let non_zst_fields = field_infos
        .clone()
        .filter_map(|(span, zst, _align, _non_exhaustive)| if !zst { Some(span) } else { None });
    let non_zst_count = non_zst_fields.clone().count();
    if non_zst_count >= 2 {
        bad_non_zero_sized_fields(tcx, adt, non_zst_count, non_zst_fields, tcx.def_span(adt.did()));
    }
    let incompatible_zst_fields =
        field_infos.clone().filter(|(_, _, _, opt)| opt.is_some()).count();
    let incompat = incompatible_zst_fields + non_zst_count >= 2 && non_zst_count < 2;
    for (span, zst, align, non_exhaustive) in field_infos {
        if zst && align != Some(1) {
            let mut err = struct_span_err!(
                tcx.sess,
                span,
                E0691,
                "zero-sized field in transparent {} has alignment larger than 1",
                adt.descr(),
            );

            if let Some(align_bytes) = align {
                err.span_label(
                    span,
                    format!("has alignment of {align_bytes}, which is larger than 1"),
                );
            } else {
                err.span_label(span, "may have alignment larger than 1");
            }

            err.emit();
        }
        if incompat && let Some((descr, def_id, args, non_exhaustive)) = non_exhaustive {
            tcx.struct_span_lint_hir(
                REPR_TRANSPARENT_EXTERNAL_PRIVATE_FIELDS,
                tcx.hir().local_def_id_to_hir_id(adt.did().expect_local()),
                span,
                "zero-sized fields in `repr(transparent)` cannot contain external non-exhaustive types",
                |lint| {
                    let note = if non_exhaustive {
                        "is marked with `#[non_exhaustive]`"
                    } else {
                        "contains private fields"
                    };
                    let field_ty = tcx.def_path_str_with_args(def_id, args);
                    lint
                        .note(format!("this {descr} contains `{field_ty}`, which {note}, \
                            and makes it not a breaking change to become non-zero-sized in the future."))
                },
            )
        }
    }
}

#[allow(trivial_numeric_casts)]
fn check_enum(tcx: TyCtxt<'_>, def_id: LocalDefId) {
    let def = tcx.adt_def(def_id);
    def.destructor(tcx); // force the destructor to be evaluated

    if def.variants().is_empty() {
        if let Some(attr) = tcx.get_attrs(def_id, sym::repr).next() {
            struct_span_err!(
                tcx.sess,
                attr.span,
                E0084,
                "unsupported representation for zero-variant enum"
            )
            .span_label(tcx.def_span(def_id), "zero-variant enum")
            .emit();
        }
    }

    let repr_type_ty = def.repr().discr_type().to_ty(tcx);
    if repr_type_ty == tcx.types.i128 || repr_type_ty == tcx.types.u128 {
        if !tcx.features().repr128 {
            feature_err(
                &tcx.sess.parse_sess,
                sym::repr128,
                tcx.def_span(def_id),
                "repr with 128-bit type is unstable",
            )
            .emit();
        }
    }

    for v in def.variants() {
        if let ty::VariantDiscr::Explicit(discr_def_id) = v.discr {
            tcx.ensure().typeck(discr_def_id.expect_local());
        }
    }

    if def.repr().int.is_none() {
        let is_unit = |var: &ty::VariantDef| matches!(var.ctor_kind(), Some(CtorKind::Const));
        let has_disr = |var: &ty::VariantDef| matches!(var.discr, ty::VariantDiscr::Explicit(_));

        let has_non_units = def.variants().iter().any(|var| !is_unit(var));
        let disr_units = def.variants().iter().any(|var| is_unit(&var) && has_disr(&var));
        let disr_non_unit = def.variants().iter().any(|var| !is_unit(&var) && has_disr(&var));

        if disr_non_unit || (disr_units && has_non_units) {
            let mut err = struct_span_err!(
                tcx.sess,
                tcx.def_span(def_id),
                E0732,
                "`#[repr(inttype)]` must be specified"
            );
            err.emit();
        }
    }

    detect_discriminant_duplicate(tcx, def);
    check_transparent(tcx, def);
}

/// Part of enum check. Given the discriminants of an enum, errors if two or more discriminants are equal
fn detect_discriminant_duplicate<'tcx>(tcx: TyCtxt<'tcx>, adt: ty::AdtDef<'tcx>) {
    // Helper closure to reduce duplicate code. This gets called everytime we detect a duplicate.
    // Here `idx` refers to the order of which the discriminant appears, and its index in `vs`
    let report = |dis: Discr<'tcx>, idx, err: &mut Diagnostic| {
        let var = adt.variant(idx); // HIR for the duplicate discriminant
        let (span, display_discr) = match var.discr {
            ty::VariantDiscr::Explicit(discr_def_id) => {
                // In the case the discriminant is both a duplicate and overflowed, let the user know
                if let hir::Node::AnonConst(expr) = tcx.hir().get_by_def_id(discr_def_id.expect_local())
                    && let hir::ExprKind::Lit(lit) = &tcx.hir().body(expr.body).value.kind
                    && let rustc_ast::LitKind::Int(lit_value, _int_kind) = &lit.node
                    && *lit_value != dis.val
                {
                    (tcx.def_span(discr_def_id), format!("`{dis}` (overflowed from `{lit_value}`)"))
                } else {
                    // Otherwise, format the value as-is
                    (tcx.def_span(discr_def_id), format!("`{dis}`"))
                }
            }
            // This should not happen.
            ty::VariantDiscr::Relative(0) => (tcx.def_span(var.def_id), format!("`{dis}`")),
            ty::VariantDiscr::Relative(distance_to_explicit) => {
                // At this point we know this discriminant is a duplicate, and was not explicitly
                // assigned by the user. Here we iterate backwards to fetch the HIR for the last
                // explicitly assigned discriminant, and letting the user know that this was the
                // increment startpoint, and how many steps from there leading to the duplicate
                if let Some(explicit_idx) =
                    idx.as_u32().checked_sub(distance_to_explicit).map(VariantIdx::from_u32)
                {
                    let explicit_variant = adt.variant(explicit_idx);
                    let ve_ident = var.name;
                    let ex_ident = explicit_variant.name;
                    let sp = if distance_to_explicit > 1 { "variants" } else { "variant" };

                    err.span_label(
                        tcx.def_span(explicit_variant.def_id),
                        format!(
                            "discriminant for `{ve_ident}` incremented from this startpoint \
                            (`{ex_ident}` + {distance_to_explicit} {sp} later \
                             => `{ve_ident}` = {dis})"
                        ),
                    );
                }

                (tcx.def_span(var.def_id), format!("`{dis}`"))
            }
        };

        err.span_label(span, format!("{display_discr} assigned here"));
    };

    let mut discrs = adt.discriminants(tcx).collect::<Vec<_>>();

    // Here we loop through the discriminants, comparing each discriminant to another.
    // When a duplicate is detected, we instantiate an error and point to both
    // initial and duplicate value. The duplicate discriminant is then discarded by swapping
    // it with the last element and decrementing the `vec.len` (which is why we have to evaluate
    // `discrs.len()` anew every iteration, and why this could be tricky to do in a functional
    // style as we are mutating `discrs` on the fly).
    let mut i = 0;
    while i < discrs.len() {
        let var_i_idx = discrs[i].0;
        let mut error: Option<DiagnosticBuilder<'_, _>> = None;

        let mut o = i + 1;
        while o < discrs.len() {
            let var_o_idx = discrs[o].0;

            if discrs[i].1.val == discrs[o].1.val {
                let err = error.get_or_insert_with(|| {
                    let mut ret = struct_span_err!(
                        tcx.sess,
                        tcx.def_span(adt.did()),
                        E0081,
                        "discriminant value `{}` assigned more than once",
                        discrs[i].1,
                    );

                    report(discrs[i].1, var_i_idx, &mut ret);

                    ret
                });

                report(discrs[o].1, var_o_idx, err);

                // Safe to unwrap here, as we wouldn't reach this point if `discrs` was empty
                discrs[o] = *discrs.last().unwrap();
                discrs.pop();
            } else {
                o += 1;
            }
        }

        if let Some(mut e) = error {
            e.emit();
        }

        i += 1;
    }
}

pub(super) fn check_type_params_are_used<'tcx>(
    tcx: TyCtxt<'tcx>,
    generics: &ty::Generics,
    ty: Ty<'tcx>,
) {
    debug!("check_type_params_are_used(generics={:?}, ty={:?})", generics, ty);

    assert_eq!(generics.parent, None);

    if generics.own_counts().types == 0 {
        return;
    }

    let mut params_used = BitSet::new_empty(generics.params.len());

    if ty.references_error() {
        // If there is already another error, do not emit
        // an error for not using a type parameter.
        assert!(tcx.sess.has_errors().is_some());
        return;
    }

    for leaf in ty.walk() {
        if let GenericArgKind::Type(leaf_ty) = leaf.unpack()
            && let ty::Param(param) = leaf_ty.kind()
        {
            debug!("found use of ty param {:?}", param);
            params_used.insert(param.index);
        }
    }

    for param in &generics.params {
        if !params_used.contains(param.index)
            && let ty::GenericParamDefKind::Type { .. } = param.kind
        {
            let span = tcx.def_span(param.def_id);
            struct_span_err!(
                tcx.sess,
                span,
                E0091,
                "type parameter `{}` is unused",
                param.name,
            )
            .span_label(span, "unused type parameter")
            .emit();
        }
    }
}

pub(super) fn check_mod_item_types(tcx: TyCtxt<'_>, module_def_id: LocalDefId) {
    let module = tcx.hir_module_items(module_def_id);
    for id in module.items() {
        check_item_type(tcx, id);
    }
    if module_def_id == CRATE_DEF_ID {
        super::entry::check_for_entry_fn(tcx);
    }
}

fn async_opaque_type_cycle_error(tcx: TyCtxt<'_>, span: Span) -> ErrorGuaranteed {
    struct_span_err!(tcx.sess, span, E0733, "recursion in an `async fn` requires boxing")
        .span_label(span, "recursive `async fn`")
        .note("a recursive `async fn` must be rewritten to return a boxed `dyn Future`")
        .note(
            "consider using the `async_recursion` crate: https://crates.io/crates/async_recursion",
        )
        .emit()
}

/// Emit an error for recursive opaque types.
///
/// If this is a return `impl Trait`, find the item's return expressions and point at them. For
/// direct recursion this is enough, but for indirect recursion also point at the last intermediary
/// `impl Trait`.
///
/// If all the return expressions evaluate to `!`, then we explain that the error will go away
/// after changing it. This can happen when a user uses `panic!()` or similar as a placeholder.
fn opaque_type_cycle_error(
    tcx: TyCtxt<'_>,
    opaque_def_id: LocalDefId,
    span: Span,
) -> ErrorGuaranteed {
    let mut err = struct_span_err!(tcx.sess, span, E0720, "cannot resolve opaque type");

    let mut label = false;
    if let Some((def_id, visitor)) = get_owner_return_paths(tcx, opaque_def_id) {
        let typeck_results = tcx.typeck(def_id);
        if visitor
            .returns
            .iter()
            .filter_map(|expr| typeck_results.node_type_opt(expr.hir_id))
            .all(|ty| matches!(ty.kind(), ty::Never))
        {
            let spans = visitor
                .returns
                .iter()
                .filter(|expr| typeck_results.node_type_opt(expr.hir_id).is_some())
                .map(|expr| expr.span)
                .collect::<Vec<Span>>();
            let span_len = spans.len();
            if span_len == 1 {
                err.span_label(spans[0], "this returned value is of `!` type");
            } else {
                let mut multispan: MultiSpan = spans.clone().into();
                for span in spans {
                    multispan.push_span_label(span, "this returned value is of `!` type");
                }
                err.span_note(multispan, "these returned values have a concrete \"never\" type");
            }
            err.help("this error will resolve once the item's body returns a concrete type");
        } else {
            let mut seen = FxHashSet::default();
            seen.insert(span);
            err.span_label(span, "recursive opaque type");
            label = true;
            for (sp, ty) in visitor
                .returns
                .iter()
                .filter_map(|e| typeck_results.node_type_opt(e.hir_id).map(|t| (e.span, t)))
                .filter(|(_, ty)| !matches!(ty.kind(), ty::Never))
            {
                #[derive(Default)]
                struct OpaqueTypeCollector {
                    opaques: Vec<DefId>,
                    closures: Vec<DefId>,
                }
                impl<'tcx> ty::visit::TypeVisitor<TyCtxt<'tcx>> for OpaqueTypeCollector {
                    fn visit_ty(&mut self, t: Ty<'tcx>) -> ControlFlow<Self::BreakTy> {
                        match *t.kind() {
                            ty::Alias(ty::Opaque, ty::AliasTy { def_id: def, .. }) => {
                                self.opaques.push(def);
                                ControlFlow::Continue(())
                            }
                            ty::Closure(def_id, ..) | ty::Generator(def_id, ..) => {
                                self.closures.push(def_id);
                                t.super_visit_with(self)
                            }
                            _ => t.super_visit_with(self),
                        }
                    }
                }

                let mut visitor = OpaqueTypeCollector::default();
                ty.visit_with(&mut visitor);
                for def_id in visitor.opaques {
                    let ty_span = tcx.def_span(def_id);
                    if !seen.contains(&ty_span) {
                        let descr = if ty.is_impl_trait() { "opaque " } else { "" };
                        err.span_label(ty_span, format!("returning this {descr}type `{ty}`"));
                        seen.insert(ty_span);
                    }
                    err.span_label(sp, format!("returning here with type `{ty}`"));
                }

                for closure_def_id in visitor.closures {
                    let Some(closure_local_did) = closure_def_id.as_local() else {
                        continue;
                    };
                    let typeck_results = tcx.typeck(closure_local_did);

                    let mut label_match = |ty: Ty<'_>, span| {
                        for arg in ty.walk() {
                            if let ty::GenericArgKind::Type(ty) = arg.unpack()
                                && let ty::Alias(ty::Opaque, ty::AliasTy { def_id: captured_def_id, .. }) = *ty.kind()
                                && captured_def_id == opaque_def_id.to_def_id()
                            {
                                err.span_label(
                                    span,
                                    format!(
                                        "{} captures itself here",
                                        tcx.def_descr(closure_def_id)
                                    ),
                                );
                            }
                        }
                    };

                    // Label any closure upvars that capture the opaque
                    for capture in typeck_results.closure_min_captures_flattened(closure_local_did)
                    {
                        label_match(capture.place.ty(), capture.get_path_span(tcx));
                    }
                    // Label any generator locals that capture the opaque
                    for interior_ty in
                        typeck_results.generator_interior_types.as_ref().skip_binder()
                    {
                        label_match(interior_ty.ty, interior_ty.span);
                    }
                    if tcx.sess.opts.unstable_opts.drop_tracking_mir
                        && let DefKind::Generator = tcx.def_kind(closure_def_id)
                        && let Some(generator_layout) = tcx.mir_generator_witnesses(closure_def_id)
                    {
                        for interior_ty in &generator_layout.field_tys {
                            label_match(interior_ty.ty, interior_ty.source_info.span);
                        }
                    }
                }
            }
        }
    }
    if !label {
        err.span_label(span, "cannot resolve opaque type");
    }
    err.emit()
}

pub(super) fn check_generator_obligations(tcx: TyCtxt<'_>, def_id: LocalDefId) {
    debug_assert!(tcx.sess.opts.unstable_opts.drop_tracking_mir);
    debug_assert!(matches!(tcx.def_kind(def_id), DefKind::Generator));

    let typeck = tcx.typeck(def_id);
    let param_env = tcx.param_env(def_id);

    let generator_interior_predicates = &typeck.generator_interior_predicates[&def_id];
    debug!(?generator_interior_predicates);

    let infcx = tcx
        .infer_ctxt()
        // typeck writeback gives us predicates with their regions erased.
        // As borrowck already has checked lifetimes, we do not need to do it again.
        .ignoring_regions()
        // Bind opaque types to `def_id` as they should have been checked by borrowck.
        .with_opaque_type_inference(DefiningAnchor::Bind(def_id))
        .build();

    let mut fulfillment_cx = <dyn TraitEngine<'_>>::new(&infcx);
    for (predicate, cause) in generator_interior_predicates {
        let obligation = Obligation::new(tcx, cause.clone(), param_env, *predicate);
        fulfillment_cx.register_predicate_obligation(&infcx, obligation);
    }
    let errors = fulfillment_cx.select_all_or_error(&infcx);
    debug!(?errors);
    if !errors.is_empty() {
        infcx.err_ctxt().report_fulfillment_errors(&errors);
    }
}
