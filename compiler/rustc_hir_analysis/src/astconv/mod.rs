//! Conversion from AST representation of types to the `ty.rs` representation.
//! The main routine here is `ast_ty_to_ty()`; each use is parameterized by an
//! instance of `AstConv`.

mod bounds;
mod errors;
pub mod generics;
mod lint;
mod object_safety;

use crate::astconv::errors::prohibit_assoc_ty_binding;
use crate::astconv::generics::{check_generic_arg_count, create_args_for_parent_generic_args};
use crate::bounds::Bounds;
use crate::collect::HirPlaceholderCollector;
use crate::errors::{AmbiguousLifetimeBound, TypeofReservedKeywordUsed};
use crate::middle::resolve_bound_vars as rbv;
use crate::require_c_abi_if_c_variadic;
use rustc_ast::TraitObjectSyntax;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_errors::{
    struct_span_err, Applicability, Diagnostic, DiagnosticBuilder, ErrorGuaranteed, FatalError,
    MultiSpan,
};
use rustc_hir as hir;
use rustc_hir::def::{CtorOf, DefKind, Namespace, Res};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_hir::intravisit::{walk_generics, Visitor as _};
use rustc_hir::{GenericArg, GenericArgs, OpaqueTyOrigin};
use rustc_infer::infer::{InferCtxt, InferOk, TyCtxtInferExt};
use rustc_infer::traits::ObligationCause;
use rustc_middle::middle::stability::AllowUnstable;
use rustc_middle::ty::GenericParamDefKind;
use rustc_middle::ty::{
    self, Const, GenericArgKind, GenericArgsRef, IsSuggestable, Ty, TyCtxt, TypeVisitableExt,
};
use rustc_session::lint::builtin::AMBIGUOUS_ASSOCIATED_ITEMS;
use rustc_span::edit_distance::find_best_match_for_name;
use rustc_span::symbol::{kw, Ident, Symbol};
use rustc_span::{sym, Span, DUMMY_SP};
use rustc_target::spec::abi;
use rustc_trait_selection::traits::wf::object_region_bounds;
use rustc_trait_selection::traits::{self, NormalizeExt, ObligationCtxt};
use rustc_type_ir::fold::{TypeFoldable, TypeFolder, TypeSuperFoldable};

use std::fmt::Display;
use std::slice;

#[derive(Debug)]
pub struct PathSeg(pub DefId, pub usize);

#[derive(Copy, Clone, Debug)]
pub struct OnlySelfBounds(pub bool);

#[derive(Copy, Clone, Debug)]
pub enum PredicateFilter {
    /// All predicates may be implied by the trait.
    All,

    /// Only traits that reference `Self: ..` are implied by the trait.
    SelfOnly,

    /// Only traits that reference `Self: ..` and define an associated type
    /// with the given ident are implied by the trait.
    SelfThatDefines(Ident),

    /// Only traits that reference `Self: ..` and their associated type bounds.
    /// For example, given `Self: Tr<A: B>`, this would expand to `Self: Tr`
    /// and `<Self as Tr>::A: B`.
    SelfAndAssociatedTypeBounds,
}

pub trait AstConv<'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx>;

    fn item_def_id(&self) -> DefId;

    /// Returns predicates in scope of the form `X: Foo<T>`, where `X`
    /// is a type parameter `X` with the given id `def_id` and T
    /// matches `assoc_name`. This is a subset of the full set of
    /// predicates.
    ///
    /// This is used for one specific purpose: resolving "short-hand"
    /// associated type references like `T::Item`. In principle, we
    /// would do that by first getting the full set of predicates in
    /// scope and then filtering down to find those that apply to `T`,
    /// but this can lead to cycle errors. The problem is that we have
    /// to do this resolution *in order to create the predicates in
    /// the first place*. Hence, we have this "special pass".
    fn get_type_parameter_bounds(
        &self,
        span: Span,
        def_id: LocalDefId,
        assoc_name: Ident,
    ) -> ty::GenericPredicates<'tcx>;

    /// Returns the lifetime to use when a lifetime is omitted (and not elided).
    fn re_infer(&self, param: Option<&ty::GenericParamDef>, span: Span)
    -> Option<ty::Region<'tcx>>;

    /// Returns the type to use when a type is omitted.
    fn ty_infer(&self, param: Option<&ty::GenericParamDef>, span: Span) -> Ty<'tcx>;

    /// Returns `true` if `_` is allowed in type signatures in the current context.
    fn allow_ty_infer(&self) -> bool;

    /// Returns the const to use when a const is omitted.
    fn ct_infer(
        &self,
        ty: Ty<'tcx>,
        param: Option<&ty::GenericParamDef>,
        span: Span,
    ) -> Const<'tcx>;

    /// Projecting an associated type from a (potentially)
    /// higher-ranked trait reference is more complicated, because of
    /// the possibility of late-bound regions appearing in the
    /// associated type binding. This is not legal in function
    /// signatures for that reason. In a function body, we can always
    /// handle it because we can use inference variables to remove the
    /// late-bound regions.
    fn projected_ty_from_poly_trait_ref(
        &self,
        span: Span,
        item_def_id: DefId,
        item_segment: &hir::PathSegment<'_>,
        poly_trait_ref: ty::PolyTraitRef<'tcx>,
    ) -> Ty<'tcx>;

    /// Returns `AdtDef` if `ty` is an ADT.
    /// Note that `ty` might be a projection type that needs normalization.
    /// This used to get the enum variants in scope of the type.
    /// For example, `Self::A` could refer to an associated type
    /// or to an enum variant depending on the result of this function.
    fn probe_adt(&self, span: Span, ty: Ty<'tcx>) -> Option<ty::AdtDef<'tcx>>;

    /// Invoked when we encounter an error from some prior pass
    /// (e.g., resolve) that is translated into a ty-error. This is
    /// used to help suppress derived errors typeck might otherwise
    /// report.
    fn set_tainted_by_errors(&self, e: ErrorGuaranteed);

    fn record_ty(&self, hir_id: hir::HirId, ty: Ty<'tcx>, span: Span);

    fn astconv(&self) -> &dyn AstConv<'tcx>
    where
        Self: Sized,
    {
        self
    }

    fn infcx(&self) -> Option<&InferCtxt<'tcx>>;
}

#[derive(Debug)]
struct ConvertedBinding<'a, 'tcx> {
    hir_id: hir::HirId,
    item_name: Ident,
    kind: ConvertedBindingKind<'a, 'tcx>,
    gen_args: &'a GenericArgs<'a>,
    span: Span,
}

#[derive(Debug)]
enum ConvertedBindingKind<'a, 'tcx> {
    Equality(ty::Term<'tcx>),
    Constraint(&'a [hir::GenericBound<'a>]),
}

/// New-typed boolean indicating whether explicit late-bound lifetimes
/// are present in a set of generic arguments.
///
/// For example if we have some method `fn f<'a>(&'a self)` implemented
/// for some type `T`, although `f` is generic in the lifetime `'a`, `'a`
/// is late-bound so should not be provided explicitly. Thus, if `f` is
/// instantiated with some generic arguments providing `'a` explicitly,
/// we taint those arguments with `ExplicitLateBound::Yes` so that we
/// can provide an appropriate diagnostic later.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ExplicitLateBound {
    Yes,
    No,
}

#[derive(Copy, Clone, PartialEq)]
pub enum IsMethodCall {
    Yes,
    No,
}

/// Denotes the "position" of a generic argument, indicating if it is a generic type,
/// generic function or generic method call.
#[derive(Copy, Clone, PartialEq)]
pub(crate) enum GenericArgPosition {
    Type,
    Value, // e.g., functions
    MethodCall,
}

/// A marker denoting that the generic arguments that were
/// provided did not match the respective generic parameters.
#[derive(Clone, Default, Debug)]
pub struct GenericArgCountMismatch {
    /// Indicates whether a fatal error was reported (`Some`), or just a lint (`None`).
    pub reported: Option<ErrorGuaranteed>,
    /// A list of spans of arguments provided that were not valid.
    pub invalid_args: Vec<Span>,
}

/// Decorates the result of a generic argument count mismatch
/// check with whether explicit late bounds were provided.
#[derive(Clone, Debug)]
pub struct GenericArgCountResult {
    pub explicit_late_bound: ExplicitLateBound,
    pub correct: Result<(), GenericArgCountMismatch>,
}

pub trait CreateSubstsForGenericArgsCtxt<'a, 'tcx> {
    fn args_for_def_id(&mut self, def_id: DefId) -> (Option<&'a GenericArgs<'a>>, bool);

    fn provided_kind(
        &mut self,
        param: &ty::GenericParamDef,
        arg: &GenericArg<'_>,
    ) -> ty::GenericArg<'tcx>;

    fn inferred_kind(
        &mut self,
        args: Option<&[ty::GenericArg<'tcx>]>,
        param: &ty::GenericParamDef,
        infer_args: bool,
    ) -> ty::GenericArg<'tcx>;
}

impl<'o, 'tcx> dyn AstConv<'tcx> + 'o {
    #[instrument(level = "debug", skip(self), ret)]
    pub fn ast_region_to_region(
        &self,
        lifetime: &hir::Lifetime,
        def: Option<&ty::GenericParamDef>,
    ) -> ty::Region<'tcx> {
        let tcx = self.tcx();
        let lifetime_name = |def_id| tcx.hir().name(tcx.hir().local_def_id_to_hir_id(def_id));

        match tcx.named_bound_var(lifetime.hir_id) {
            Some(rbv::ResolvedArg::StaticLifetime) => tcx.lifetimes.re_static,

            Some(rbv::ResolvedArg::LateBound(debruijn, index, def_id)) => {
                let name = lifetime_name(def_id.expect_local());
                let br = ty::BoundRegion {
                    var: ty::BoundVar::from_u32(index),
                    kind: ty::BrNamed(def_id, name),
                };
                ty::Region::new_late_bound(tcx, debruijn, br)
            }

            Some(rbv::ResolvedArg::EarlyBound(def_id)) => {
                let name = tcx.hir().ty_param_name(def_id.expect_local());
                let item_def_id = tcx.hir().ty_param_owner(def_id.expect_local());
                let generics = tcx.generics_of(item_def_id);
                let index = generics.param_def_id_to_index[&def_id];
                ty::Region::new_early_bound(tcx, ty::EarlyBoundRegion { def_id, index, name })
            }

            Some(rbv::ResolvedArg::Free(scope, id)) => {
                let name = lifetime_name(id.expect_local());
                ty::Region::new_free(tcx, scope, ty::BrNamed(id, name))

                // (*) -- not late-bound, won't change
            }

            Some(rbv::ResolvedArg::Error(_)) => {
                bug!("only ty/ct should resolve as ResolvedArg::Error")
            }

            None => {
                self.re_infer(def, lifetime.ident.span).unwrap_or_else(|| {
                    debug!(?lifetime, "unelided lifetime in signature");

                    // This indicates an illegal lifetime
                    // elision. `resolve_lifetime` should have
                    // reported an error in this case -- but if
                    // not, let's error out.
                    ty::Region::new_error_with_message(
                        tcx,
                        lifetime.ident.span,
                        "unelided lifetime in signature",
                    )
                })
            }
        }
    }

    /// Given a path `path` that refers to an item `I` with the declared generics `decl_generics`,
    /// returns an appropriate set of substitutions for this particular reference to `I`.
    pub fn ast_path_args_for_ty(
        &self,
        span: Span,
        def_id: DefId,
        item_segment: &hir::PathSegment<'_>,
    ) -> GenericArgsRef<'tcx> {
        let (args, _) = self.create_args_for_ast_path(
            span,
            def_id,
            &[],
            item_segment,
            item_segment.args(),
            item_segment.infer_args,
            None,
            ty::BoundConstness::NotConst,
        );
        if let Some(b) = item_segment.args().bindings.first() {
            prohibit_assoc_ty_binding(self.tcx(), b.span, Some((item_segment, span)));
        }

        args
    }

    /// Given the type/lifetime/const arguments provided to some path (along with
    /// an implicit `Self`, if this is a trait reference), returns the complete
    /// set of substitutions. This may involve applying defaulted type parameters.
    /// Constraints on associated types are created from `create_assoc_bindings_for_generic_args`.
    ///
    /// Example:
    ///
    /// ```ignore (illustrative)
    ///    T: std::ops::Index<usize, Output = u32>
    /// // ^1 ^^^^^^^^^^^^^^2 ^^^^3  ^^^^^^^^^^^4
    /// ```
    ///
    /// 1. The `self_ty` here would refer to the type `T`.
    /// 2. The path in question is the path to the trait `std::ops::Index`,
    ///    which will have been resolved to a `def_id`
    /// 3. The `generic_args` contains info on the `<...>` contents. The `usize` type
    ///    parameters are returned in the `GenericArgsRef`, the associated type bindings like
    ///    `Output = u32` are returned from `create_assoc_bindings_for_generic_args`.
    ///
    /// Note that the type listing given here is *exactly* what the user provided.
    ///
    /// For (generic) associated types
    ///
    /// ```ignore (illustrative)
    /// <Vec<u8> as Iterable<u8>>::Iter::<'a>
    /// ```
    ///
    /// We have the parent args are the args for the parent trait:
    /// `[Vec<u8>, u8]` and `generic_args` are the arguments for the associated
    /// type itself: `['a]`. The returned `GenericArgsRef` concatenates these two
    /// lists: `[Vec<u8>, u8, 'a]`.
    #[instrument(level = "debug", skip(self, span), ret)]
    fn create_args_for_ast_path<'a>(
        &self,
        span: Span,
        def_id: DefId,
        parent_args: &[ty::GenericArg<'tcx>],
        seg: &hir::PathSegment<'_>,
        generic_args: &'a hir::GenericArgs<'_>,
        infer_args: bool,
        self_ty: Option<Ty<'tcx>>,
        constness: ty::BoundConstness,
    ) -> (GenericArgsRef<'tcx>, GenericArgCountResult) {
        // If the type is parameterized by this region, then replace this
        // region with the current anon region binding (in other words,
        // whatever & would get replaced with).

        let tcx = self.tcx();
        let generics = tcx.generics_of(def_id);
        debug!("generics: {:?}", generics);

        if generics.has_self {
            if generics.parent.is_some() {
                // The parent is a trait so it should have at least one subst
                // for the `Self` type.
                assert!(!parent_args.is_empty())
            } else {
                // This item (presumably a trait) needs a self-type.
                assert!(self_ty.is_some());
            }
        } else {
            assert!(self_ty.is_none());
        }

        let arg_count = check_generic_arg_count(
            tcx,
            span,
            def_id,
            seg,
            generics,
            generic_args,
            GenericArgPosition::Type,
            self_ty.is_some(),
            infer_args,
        );

        // Skip processing if type has no generic parameters.
        // Traits always have `Self` as a generic parameter, which means they will not return early
        // here and so associated type bindings will be handled regardless of whether there are any
        // non-`Self` generic parameters.
        if generics.params.is_empty() {
            return (tcx.mk_args(parent_args), arg_count);
        }

        struct SubstsForAstPathCtxt<'a, 'tcx> {
            astconv: &'a (dyn AstConv<'tcx> + 'a),
            def_id: DefId,
            generic_args: &'a GenericArgs<'a>,
            span: Span,
            inferred_params: Vec<Span>,
            infer_args: bool,
        }

        impl<'a, 'tcx> CreateSubstsForGenericArgsCtxt<'a, 'tcx> for SubstsForAstPathCtxt<'a, 'tcx> {
            fn args_for_def_id(&mut self, did: DefId) -> (Option<&'a GenericArgs<'a>>, bool) {
                if did == self.def_id {
                    (Some(self.generic_args), self.infer_args)
                } else {
                    // The last component of this tuple is unimportant.
                    (None, false)
                }
            }

            fn provided_kind(
                &mut self,
                param: &ty::GenericParamDef,
                arg: &GenericArg<'_>,
            ) -> ty::GenericArg<'tcx> {
                let tcx = self.astconv.tcx();

                let mut handle_ty_args = |has_default, ty: &hir::Ty<'_>| {
                    if has_default {
                        tcx.check_optional_stability(
                            param.def_id,
                            Some(arg.hir_id()),
                            arg.span(),
                            None,
                            AllowUnstable::No,
                            |_, _| {
                                // Default generic parameters may not be marked
                                // with stability attributes, i.e. when the
                                // default parameter was defined at the same time
                                // as the rest of the type. As such, we ignore missing
                                // stability attributes.
                            },
                        );
                    }
                    if let (hir::TyKind::Infer, false) = (&ty.kind, self.astconv.allow_ty_infer()) {
                        self.inferred_params.push(ty.span);
                        Ty::new_misc_error(tcx).into()
                    } else {
                        self.astconv.ast_ty_to_ty(ty).into()
                    }
                };

                match (&param.kind, arg) {
                    (GenericParamDefKind::Lifetime, GenericArg::Lifetime(lt)) => {
                        self.astconv.ast_region_to_region(lt, Some(param)).into()
                    }
                    (&GenericParamDefKind::Type { has_default, .. }, GenericArg::Type(ty)) => {
                        handle_ty_args(has_default, ty)
                    }
                    (&GenericParamDefKind::Type { has_default, .. }, GenericArg::Infer(inf)) => {
                        handle_ty_args(has_default, &inf.to_ty())
                    }
                    (GenericParamDefKind::Const { .. }, GenericArg::Const(ct)) => {
                        let did = ct.value.def_id;
                        tcx.feed_anon_const_type(did, tcx.type_of(param.def_id));
                        ty::Const::from_anon_const(tcx, did).into()
                    }
                    (&GenericParamDefKind::Const { .. }, hir::GenericArg::Infer(inf)) => {
                        let ty = tcx
                            .at(self.span)
                            .type_of(param.def_id)
                            .no_bound_vars()
                            .expect("const parameter types cannot be generic");
                        if self.astconv.allow_ty_infer() {
                            self.astconv.ct_infer(ty, Some(param), inf.span).into()
                        } else {
                            self.inferred_params.push(inf.span);
                            ty::Const::new_misc_error(tcx, ty).into()
                        }
                    }
                    _ => unreachable!(),
                }
            }

            fn inferred_kind(
                &mut self,
                args: Option<&[ty::GenericArg<'tcx>]>,
                param: &ty::GenericParamDef,
                infer_args: bool,
            ) -> ty::GenericArg<'tcx> {
                let tcx = self.astconv.tcx();
                match param.kind {
                    GenericParamDefKind::Lifetime => self
                        .astconv
                        .re_infer(Some(param), self.span)
                        .unwrap_or_else(|| {
                            debug!(?param, "unelided lifetime in signature");

                            // This indicates an illegal lifetime in a non-assoc-trait position
                            ty::Region::new_error_with_message(
                                tcx,
                                self.span,
                                "unelided lifetime in signature",
                            )
                        })
                        .into(),
                    GenericParamDefKind::Type { has_default, .. } => {
                        if !infer_args && has_default {
                            // No type parameter provided, but a default exists.
                            let args = args.unwrap();
                            if args.iter().any(|arg| match arg.unpack() {
                                GenericArgKind::Type(ty) => ty.references_error(),
                                _ => false,
                            }) {
                                // Avoid ICE #86756 when type error recovery goes awry.
                                return Ty::new_misc_error(tcx).into();
                            }
                            tcx.at(self.span).type_of(param.def_id).instantiate(tcx, args).into()
                        } else if infer_args {
                            self.astconv.ty_infer(Some(param), self.span).into()
                        } else {
                            // We've already errored above about the mismatch.
                            Ty::new_misc_error(tcx).into()
                        }
                    }
                    GenericParamDefKind::Const { has_default } => {
                        let ty = tcx
                            .at(self.span)
                            .type_of(param.def_id)
                            .no_bound_vars()
                            .expect("const parameter types cannot be generic");
                        if let Err(guar) = ty.error_reported() {
                            return ty::Const::new_error(tcx, guar, ty).into();
                        }
                        if !infer_args && has_default {
                            tcx.const_param_default(param.def_id)
                                .instantiate(tcx, args.unwrap())
                                .into()
                        } else {
                            if infer_args {
                                self.astconv.ct_infer(ty, Some(param), self.span).into()
                            } else {
                                // We've already errored above about the mismatch.
                                ty::Const::new_misc_error(tcx, ty).into()
                            }
                        }
                    }
                }
            }
        }

        let mut args_ctx = SubstsForAstPathCtxt {
            astconv: self,
            def_id,
            span,
            generic_args,
            inferred_params: vec![],
            infer_args,
        };
        let args = create_args_for_parent_generic_args(
            tcx,
            def_id,
            parent_args,
            self_ty.is_some(),
            self_ty,
            &arg_count,
            &mut args_ctx,
        );

        if let ty::BoundConstness::ConstIfConst = constness
            && generics.has_self && !tcx.has_attr(def_id, sym::const_trait)
        {
            tcx.sess.emit_err(crate::errors::ConstBoundForNonConstTrait { span } );
        }

        (args, arg_count)
    }

    fn create_assoc_bindings_for_generic_args<'a>(
        &self,
        generic_args: &'a hir::GenericArgs<'_>,
    ) -> Vec<ConvertedBinding<'a, 'tcx>> {
        // Convert associated-type bindings or constraints into a separate vector.
        // Example: Given this:
        //
        //     T: Iterator<Item = u32>
        //
        // The `T` is passed in as a self-type; the `Item = u32` is
        // not a "type parameter" of the `Iterator` trait, but rather
        // a restriction on `<T as Iterator>::Item`, so it is passed
        // back separately.
        let assoc_bindings = generic_args
            .bindings
            .iter()
            .map(|binding| {
                let kind = match &binding.kind {
                    hir::TypeBindingKind::Equality { term } => match term {
                        hir::Term::Ty(ty) => {
                            ConvertedBindingKind::Equality(self.ast_ty_to_ty(ty).into())
                        }
                        hir::Term::Const(c) => {
                            let c = Const::from_anon_const(self.tcx(), c.def_id);
                            ConvertedBindingKind::Equality(c.into())
                        }
                    },
                    hir::TypeBindingKind::Constraint { bounds } => {
                        ConvertedBindingKind::Constraint(bounds)
                    }
                };
                ConvertedBinding {
                    hir_id: binding.hir_id,
                    item_name: binding.ident,
                    kind,
                    gen_args: binding.gen_args,
                    span: binding.span,
                }
            })
            .collect();

        assoc_bindings
    }

    pub fn create_args_for_associated_item(
        &self,
        span: Span,
        item_def_id: DefId,
        item_segment: &hir::PathSegment<'_>,
        parent_args: GenericArgsRef<'tcx>,
    ) -> GenericArgsRef<'tcx> {
        debug!(
            "create_args_for_associated_item(span: {:?}, item_def_id: {:?}, item_segment: {:?}",
            span, item_def_id, item_segment
        );
        let (args, _) = self.create_args_for_ast_path(
            span,
            item_def_id,
            parent_args,
            item_segment,
            item_segment.args(),
            item_segment.infer_args,
            None,
            ty::BoundConstness::NotConst,
        );

        if let Some(b) = item_segment.args().bindings.first() {
            prohibit_assoc_ty_binding(self.tcx(), b.span, Some((item_segment, span)));
        }

        args
    }

    /// Instantiates the path for the given trait reference, assuming that it's
    /// bound to a valid trait type. Returns the `DefId` of the defining trait.
    /// The type _cannot_ be a type other than a trait type.
    ///
    /// If the `projections` argument is `None`, then assoc type bindings like `Foo<T = X>`
    /// are disallowed. Otherwise, they are pushed onto the vector given.
    pub fn instantiate_mono_trait_ref(
        &self,
        trait_ref: &hir::TraitRef<'_>,
        self_ty: Ty<'tcx>,
        constness: ty::BoundConstness,
    ) -> ty::TraitRef<'tcx> {
        self.prohibit_generics(trait_ref.path.segments.split_last().unwrap().1.iter(), |_| {});

        self.ast_path_to_mono_trait_ref(
            trait_ref.path.span,
            trait_ref.trait_def_id().unwrap_or_else(|| FatalError.raise()),
            self_ty,
            trait_ref.path.segments.last().unwrap(),
            true,
            constness,
        )
    }

    fn instantiate_poly_trait_ref_inner(
        &self,
        hir_id: hir::HirId,
        span: Span,
        binding_span: Option<Span>,
        constness: ty::BoundConstness,
        polarity: ty::ImplPolarity,
        bounds: &mut Bounds<'tcx>,
        speculative: bool,
        trait_ref_span: Span,
        trait_def_id: DefId,
        trait_segment: &hir::PathSegment<'_>,
        args: &GenericArgs<'_>,
        infer_args: bool,
        self_ty: Ty<'tcx>,
        only_self_bounds: OnlySelfBounds,
    ) -> GenericArgCountResult {
        let (generic_args, arg_count) = self.create_args_for_ast_path(
            trait_ref_span,
            trait_def_id,
            &[],
            trait_segment,
            args,
            infer_args,
            Some(self_ty),
            constness,
        );

        let tcx = self.tcx();
        let bound_vars = tcx.late_bound_vars(hir_id);
        debug!(?bound_vars);

        let assoc_bindings = self.create_assoc_bindings_for_generic_args(args);

        let poly_trait_ref = ty::Binder::bind_with_vars(
            ty::TraitRef::new(tcx, trait_def_id, generic_args),
            bound_vars,
        );

        debug!(?poly_trait_ref, ?assoc_bindings);
        bounds.push_trait_bound(tcx, poly_trait_ref, span, constness, polarity);

        let mut dup_bindings = FxHashMap::default();
        for binding in &assoc_bindings {
            // Don't register additional associated type bounds for negative bounds,
            // since we should have emitten an error for them earlier, and they will
            // not be well-formed!
            if polarity == ty::ImplPolarity::Negative {
                self.tcx()
                    .sess
                    .delay_span_bug(binding.span, "negative trait bounds should not have bindings");
                continue;
            }

            // Specify type to assert that error was already reported in `Err` case.
            let _: Result<_, ErrorGuaranteed> = self.add_predicates_for_ast_type_binding(
                hir_id,
                poly_trait_ref,
                binding,
                bounds,
                speculative,
                &mut dup_bindings,
                binding_span.unwrap_or(binding.span),
                constness,
                only_self_bounds,
                polarity,
            );
            // Okay to ignore `Err` because of `ErrorGuaranteed` (see above).
        }

        arg_count
    }

    /// Given a trait bound like `Debug`, applies that trait bound the given self-type to construct
    /// a full trait reference. The resulting trait reference is returned. This may also generate
    /// auxiliary bounds, which are added to `bounds`.
    ///
    /// Example:
    ///
    /// ```ignore (illustrative)
    /// poly_trait_ref = Iterator<Item = u32>
    /// self_ty = Foo
    /// ```
    ///
    /// this would return `Foo: Iterator` and add `<Foo as Iterator>::Item = u32` into `bounds`.
    ///
    /// **A note on binders:** against our usual convention, there is an implied bounder around
    /// the `self_ty` and `poly_trait_ref` parameters here. So they may reference bound regions.
    /// If for example you had `for<'a> Foo<'a>: Bar<'a>`, then the `self_ty` would be `Foo<'a>`
    /// where `'a` is a bound region at depth 0. Similarly, the `poly_trait_ref` would be
    /// `Bar<'a>`. The returned poly-trait-ref will have this binder instantiated explicitly,
    /// however.
    #[instrument(level = "debug", skip(self, span, constness, bounds, speculative))]
    pub(crate) fn instantiate_poly_trait_ref(
        &self,
        trait_ref: &hir::TraitRef<'_>,
        span: Span,
        constness: ty::BoundConstness,
        polarity: ty::ImplPolarity,
        self_ty: Ty<'tcx>,
        bounds: &mut Bounds<'tcx>,
        speculative: bool,
        only_self_bounds: OnlySelfBounds,
    ) -> GenericArgCountResult {
        let hir_id = trait_ref.hir_ref_id;
        let binding_span = None;
        let trait_ref_span = trait_ref.path.span;
        let trait_def_id = trait_ref.trait_def_id().unwrap_or_else(|| FatalError.raise());
        let trait_segment = trait_ref.path.segments.last().unwrap();
        let args = trait_segment.args();
        let infer_args = trait_segment.infer_args;

        self.prohibit_generics(trait_ref.path.segments.split_last().unwrap().1.iter(), |_| {});
        self.complain_about_internal_fn_trait(span, trait_def_id, trait_segment, false);

        self.instantiate_poly_trait_ref_inner(
            hir_id,
            span,
            binding_span,
            constness,
            polarity,
            bounds,
            speculative,
            trait_ref_span,
            trait_def_id,
            trait_segment,
            args,
            infer_args,
            self_ty,
            only_self_bounds,
        )
    }

    pub(crate) fn instantiate_lang_item_trait_ref(
        &self,
        lang_item: hir::LangItem,
        span: Span,
        hir_id: hir::HirId,
        args: &GenericArgs<'_>,
        self_ty: Ty<'tcx>,
        bounds: &mut Bounds<'tcx>,
        only_self_bounds: OnlySelfBounds,
    ) {
        let binding_span = Some(span);
        let constness = ty::BoundConstness::NotConst;
        let speculative = false;
        let trait_ref_span = span;
        let trait_def_id = self.tcx().require_lang_item(lang_item, Some(span));
        let trait_segment = &hir::PathSegment::invalid();
        let infer_args = false;

        self.instantiate_poly_trait_ref_inner(
            hir_id,
            span,
            binding_span,
            constness,
            ty::ImplPolarity::Positive,
            bounds,
            speculative,
            trait_ref_span,
            trait_def_id,
            trait_segment,
            args,
            infer_args,
            self_ty,
            only_self_bounds,
        );
    }

    fn ast_path_to_mono_trait_ref(
        &self,
        span: Span,
        trait_def_id: DefId,
        self_ty: Ty<'tcx>,
        trait_segment: &hir::PathSegment<'_>,
        is_impl: bool,
        constness: ty::BoundConstness,
    ) -> ty::TraitRef<'tcx> {
        let (generic_args, _) = self.create_args_for_ast_trait_ref(
            span,
            trait_def_id,
            self_ty,
            trait_segment,
            is_impl,
            constness,
        );
        if let Some(b) = trait_segment.args().bindings.first() {
            prohibit_assoc_ty_binding(self.tcx(), b.span, Some((trait_segment, span)));
        }
        ty::TraitRef::new(self.tcx(), trait_def_id, generic_args)
    }

    #[instrument(level = "debug", skip(self, span))]
    fn create_args_for_ast_trait_ref<'a>(
        &self,
        span: Span,
        trait_def_id: DefId,
        self_ty: Ty<'tcx>,
        trait_segment: &'a hir::PathSegment<'a>,
        is_impl: bool,
        constness: ty::BoundConstness,
    ) -> (GenericArgsRef<'tcx>, GenericArgCountResult) {
        self.complain_about_internal_fn_trait(span, trait_def_id, trait_segment, is_impl);

        self.create_args_for_ast_path(
            span,
            trait_def_id,
            &[],
            trait_segment,
            trait_segment.args(),
            trait_segment.infer_args,
            Some(self_ty),
            constness,
        )
    }

    fn trait_defines_associated_item_named(
        &self,
        trait_def_id: DefId,
        assoc_kind: ty::AssocKind,
        assoc_name: Ident,
    ) -> bool {
        self.tcx()
            .associated_items(trait_def_id)
            .find_by_name_and_kind(self.tcx(), assoc_name, assoc_kind, trait_def_id)
            .is_some()
    }

    fn ast_path_to_ty(
        &self,
        span: Span,
        did: DefId,
        item_segment: &hir::PathSegment<'_>,
    ) -> Ty<'tcx> {
        let args = self.ast_path_args_for_ty(span, did, item_segment);
        let ty = self.tcx().at(span).type_of(did);

        if matches!(self.tcx().def_kind(did), DefKind::TyAlias)
            && (ty.skip_binder().has_opaque_types() || self.tcx().features().lazy_type_alias)
        {
            // Type aliases referring to types that contain opaque types (but aren't just directly
            // referencing a single opaque type) get encoded as a type alias that normalization will
            // then actually instantiate the where bounds of.
            let alias_ty = self.tcx().mk_alias_ty(did, args);
            Ty::new_alias(self.tcx(), ty::Weak, alias_ty)
        } else {
            ty.instantiate(self.tcx(), args)
        }
    }

    fn report_ambiguous_associated_type(
        &self,
        span: Span,
        types: &[String],
        traits: &[String],
        name: Symbol,
    ) -> ErrorGuaranteed {
        let mut err = struct_span_err!(self.tcx().sess, span, E0223, "ambiguous associated type");
        if self
            .tcx()
            .resolutions(())
            .confused_type_with_std_module
            .keys()
            .any(|full_span| full_span.contains(span))
        {
            err.span_suggestion_verbose(
                span.shrink_to_lo(),
                "you are looking for the module in `std`, not the primitive type",
                "std::",
                Applicability::MachineApplicable,
            );
        } else {
            match (types, traits) {
                ([], []) => {
                    err.span_suggestion_verbose(
                        span,
                        format!(
                            "if there were a type named `Type` that implements a trait named \
                             `Trait` with associated type `{name}`, you could use the \
                             fully-qualified path",
                        ),
                        format!("<Type as Trait>::{name}"),
                        Applicability::HasPlaceholders,
                    );
                }
                ([], [trait_str]) => {
                    err.span_suggestion_verbose(
                        span,
                        format!(
                            "if there were a type named `Example` that implemented `{trait_str}`, \
                             you could use the fully-qualified path",
                        ),
                        format!("<Example as {trait_str}>::{name}"),
                        Applicability::HasPlaceholders,
                    );
                }
                ([], traits) => {
                    err.span_suggestions(
                        span,
                        format!(
                            "if there were a type named `Example` that implemented one of the \
                             traits with associated type `{name}`, you could use the \
                             fully-qualified path",
                        ),
                        traits
                            .iter()
                            .map(|trait_str| format!("<Example as {trait_str}>::{name}"))
                            .collect::<Vec<_>>(),
                        Applicability::HasPlaceholders,
                    );
                }
                ([type_str], []) => {
                    err.span_suggestion_verbose(
                        span,
                        format!(
                            "if there were a trait named `Example` with associated type `{name}` \
                             implemented for `{type_str}`, you could use the fully-qualified path",
                        ),
                        format!("<{type_str} as Example>::{name}"),
                        Applicability::HasPlaceholders,
                    );
                }
                (types, []) => {
                    err.span_suggestions(
                        span,
                        format!(
                            "if there were a trait named `Example` with associated type `{name}` \
                             implemented for one of the types, you could use the fully-qualified \
                             path",
                        ),
                        types
                            .into_iter()
                            .map(|type_str| format!("<{type_str} as Example>::{name}")),
                        Applicability::HasPlaceholders,
                    );
                }
                (types, traits) => {
                    let mut suggestions = vec![];
                    for type_str in types {
                        for trait_str in traits {
                            suggestions.push(format!("<{type_str} as {trait_str}>::{name}"));
                        }
                    }
                    err.span_suggestions(
                        span,
                        "use the fully-qualified path",
                        suggestions,
                        Applicability::MachineApplicable,
                    );
                }
            }
        }
        err.emit()
    }

    // Search for a bound on a type parameter which includes the associated item
    // given by `assoc_name`. `ty_param_def_id` is the `DefId` of the type parameter
    // This function will fail if there are no suitable bounds or there is
    // any ambiguity.
    fn find_bound_for_assoc_item(
        &self,
        ty_param_def_id: LocalDefId,
        assoc_name: Ident,
        span: Span,
    ) -> Result<ty::PolyTraitRef<'tcx>, ErrorGuaranteed> {
        let tcx = self.tcx();

        debug!(
            "find_bound_for_assoc_item(ty_param_def_id={:?}, assoc_name={:?}, span={:?})",
            ty_param_def_id, assoc_name, span,
        );

        let predicates =
            &self.get_type_parameter_bounds(span, ty_param_def_id, assoc_name).predicates;

        debug!("find_bound_for_assoc_item: predicates={:#?}", predicates);

        let param_name = tcx.hir().ty_param_name(ty_param_def_id);
        self.one_bound_for_assoc_type(
            || {
                traits::transitive_bounds_that_define_assoc_item(
                    tcx,
                    predicates
                        .iter()
                        .filter_map(|(p, _)| Some(p.as_trait_clause()?.map_bound(|t| t.trait_ref))),
                    assoc_name,
                )
            },
            param_name,
            assoc_name,
            span,
            None,
        )
    }

    // Checks that `bounds` contains exactly one element and reports appropriate
    // errors otherwise.
    #[instrument(level = "debug", skip(self, all_candidates, ty_param_name, is_equality), ret)]
    fn one_bound_for_assoc_type<I>(
        &self,
        all_candidates: impl Fn() -> I,
        ty_param_name: impl Display,
        assoc_name: Ident,
        span: Span,
        is_equality: Option<ty::Term<'tcx>>,
    ) -> Result<ty::PolyTraitRef<'tcx>, ErrorGuaranteed>
    where
        I: Iterator<Item = ty::PolyTraitRef<'tcx>>,
    {
        let mut matching_candidates = all_candidates().filter(|r| {
            self.trait_defines_associated_item_named(r.def_id(), ty::AssocKind::Type, assoc_name)
        });
        let mut const_candidates = all_candidates().filter(|r| {
            self.trait_defines_associated_item_named(r.def_id(), ty::AssocKind::Const, assoc_name)
        });

        let (bound, next_cand) = match (matching_candidates.next(), const_candidates.next()) {
            (Some(bound), _) => (bound, matching_candidates.next()),
            (None, Some(bound)) => (bound, const_candidates.next()),
            (None, None) => {
                let reported = self.complain_about_assoc_type_not_found(
                    all_candidates,
                    &ty_param_name.to_string(),
                    assoc_name,
                    span,
                );
                return Err(reported);
            }
        };
        debug!(?bound);

        if let Some(bound2) = next_cand {
            debug!(?bound2);

            let bounds = IntoIterator::into_iter([bound, bound2]).chain(matching_candidates);
            let mut err = if is_equality.is_some() {
                // More specific Error Index entry.
                struct_span_err!(
                    self.tcx().sess,
                    span,
                    E0222,
                    "ambiguous associated type `{}` in bounds of `{}`",
                    assoc_name,
                    ty_param_name
                )
            } else {
                struct_span_err!(
                    self.tcx().sess,
                    span,
                    E0221,
                    "ambiguous associated type `{}` in bounds of `{}`",
                    assoc_name,
                    ty_param_name
                )
            };
            err.span_label(span, format!("ambiguous associated type `{assoc_name}`"));

            let mut where_bounds = vec![];
            for bound in bounds {
                let bound_id = bound.def_id();
                let bound_span = self
                    .tcx()
                    .associated_items(bound_id)
                    .find_by_name_and_kind(self.tcx(), assoc_name, ty::AssocKind::Type, bound_id)
                    .and_then(|item| self.tcx().hir().span_if_local(item.def_id));

                if let Some(bound_span) = bound_span {
                    err.span_label(
                        bound_span,
                        format!(
                            "ambiguous `{}` from `{}`",
                            assoc_name,
                            bound.print_only_trait_path(),
                        ),
                    );
                    if let Some(constraint) = &is_equality {
                        where_bounds.push(format!(
                            "        T: {trait}::{assoc} = {constraint}",
                            trait=bound.print_only_trait_path(),
                            assoc=assoc_name,
                            constraint=constraint,
                        ));
                    } else {
                        err.span_suggestion_verbose(
                            span.with_hi(assoc_name.span.lo()),
                            "use fully qualified syntax to disambiguate",
                            format!("<{} as {}>::", ty_param_name, bound.print_only_trait_path()),
                            Applicability::MaybeIncorrect,
                        );
                    }
                } else {
                    err.note(format!(
                        "associated type `{}` could derive from `{}`",
                        ty_param_name,
                        bound.print_only_trait_path(),
                    ));
                }
            }
            if !where_bounds.is_empty() {
                err.help(format!(
                    "consider introducing a new type parameter `T` and adding `where` constraints:\
                     \n    where\n        T: {},\n{}",
                    ty_param_name,
                    where_bounds.join(",\n"),
                ));
            }
            let reported = err.emit();
            if !where_bounds.is_empty() {
                return Err(reported);
            }
        }

        Ok(bound)
    }

    #[instrument(level = "debug", skip(self, all_candidates, ty_name), ret)]
    fn one_bound_for_assoc_method(
        &self,
        all_candidates: impl Iterator<Item = ty::PolyTraitRef<'tcx>>,
        ty_name: impl Display,
        assoc_name: Ident,
        span: Span,
    ) -> Result<ty::PolyTraitRef<'tcx>, ErrorGuaranteed> {
        let mut matching_candidates = all_candidates.filter(|r| {
            self.trait_defines_associated_item_named(r.def_id(), ty::AssocKind::Fn, assoc_name)
        });

        let candidate = match matching_candidates.next() {
            Some(candidate) => candidate,
            None => {
                return Err(self.tcx().sess.emit_err(
                    crate::errors::ReturnTypeNotationMissingMethod {
                        span,
                        ty_name: ty_name.to_string(),
                        assoc_name: assoc_name.name,
                    },
                ));
            }
        };

        if let Some(conflicting_candidate) = matching_candidates.next() {
            return Err(self.tcx().sess.emit_err(
                crate::errors::ReturnTypeNotationConflictingBound {
                    span,
                    ty_name: ty_name.to_string(),
                    assoc_name: assoc_name.name,
                    first_bound: candidate.print_only_trait_path(),
                    second_bound: conflicting_candidate.print_only_trait_path(),
                },
            ));
        }

        Ok(candidate)
    }

    // Create a type from a path to an associated type or to an enum variant.
    // For a path `A::B::C::D`, `qself_ty` and `qself_def` are the type and def for `A::B::C`
    // and item_segment is the path segment for `D`. We return a type and a def for
    // the whole path.
    // Will fail except for `T::A` and `Self::A`; i.e., if `qself_ty`/`qself_def` are not a type
    // parameter or `Self`.
    // NOTE: When this function starts resolving `Trait::AssocTy` successfully
    // it should also start reporting the `BARE_TRAIT_OBJECTS` lint.
    #[instrument(level = "debug", skip(self, hir_ref_id, span, qself, assoc_segment), fields(assoc_ident=?assoc_segment.ident), ret)]
    pub fn associated_path_to_ty(
        &self,
        hir_ref_id: hir::HirId,
        span: Span,
        qself_ty: Ty<'tcx>,
        qself: &hir::Ty<'_>,
        assoc_segment: &hir::PathSegment<'_>,
        permit_variants: bool,
    ) -> Result<(Ty<'tcx>, DefKind, DefId), ErrorGuaranteed> {
        let tcx = self.tcx();
        let assoc_ident = assoc_segment.ident;
        let qself_res = if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &qself.kind {
            path.res
        } else {
            Res::Err
        };

        // Check if we have an enum variant or an inherent associated type.
        let mut variant_resolution = None;
        if let Some(adt_def) = self.probe_adt(span, qself_ty) {
            if adt_def.is_enum() {
                let variant_def = adt_def
                    .variants()
                    .iter()
                    .find(|vd| tcx.hygienic_eq(assoc_ident, vd.ident(tcx), adt_def.did()));
                if let Some(variant_def) = variant_def {
                    if permit_variants {
                        tcx.check_stability(variant_def.def_id, Some(hir_ref_id), span, None);
                        self.prohibit_generics(slice::from_ref(assoc_segment).iter(), |err| {
                            err.note("enum variants can't have type parameters");
                            let type_name = tcx.item_name(adt_def.did());
                            let msg = format!(
                                "you might have meant to specify type parameters on enum \
                                 `{type_name}`"
                            );
                            let Some(args) = assoc_segment.args else {
                                return;
                            };
                            // Get the span of the generics args *including* the leading `::`.
                            let args_span =
                                assoc_segment.ident.span.shrink_to_hi().to(args.span_ext);
                            if tcx.generics_of(adt_def.did()).count() == 0 {
                                // FIXME(estebank): we could also verify that the arguments being
                                // work for the `enum`, instead of just looking if it takes *any*.
                                err.span_suggestion_verbose(
                                    args_span,
                                    format!("{type_name} doesn't have generic parameters"),
                                    "",
                                    Applicability::MachineApplicable,
                                );
                                return;
                            }
                            let Ok(snippet) = tcx.sess.source_map().span_to_snippet(args_span)
                            else {
                                err.note(msg);
                                return;
                            };
                            let (qself_sugg_span, is_self) =
                                if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) =
                                    &qself.kind
                                {
                                    // If the path segment already has type params, we want to overwrite
                                    // them.
                                    match &path.segments {
                                        // `segment` is the previous to last element on the path,
                                        // which would normally be the `enum` itself, while the last
                                        // `_` `PathSegment` corresponds to the variant.
                                        [
                                            ..,
                                            hir::PathSegment {
                                                ident,
                                                args,
                                                res: Res::Def(DefKind::Enum, _),
                                                ..
                                            },
                                            _,
                                        ] => (
                                            // We need to include the `::` in `Type::Variant::<Args>`
                                            // to point the span to `::<Args>`, not just `<Args>`.
                                            ident.span.shrink_to_hi().to(args
                                                .map_or(ident.span.shrink_to_hi(), |a| a.span_ext)),
                                            false,
                                        ),
                                        [segment] => (
                                            // We need to include the `::` in `Type::Variant::<Args>`
                                            // to point the span to `::<Args>`, not just `<Args>`.
                                            segment.ident.span.shrink_to_hi().to(segment
                                                .args
                                                .map_or(segment.ident.span.shrink_to_hi(), |a| {
                                                    a.span_ext
                                                })),
                                            kw::SelfUpper == segment.ident.name,
                                        ),
                                        _ => {
                                            err.note(msg);
                                            return;
                                        }
                                    }
                                } else {
                                    err.note(msg);
                                    return;
                                };
                            let suggestion = vec![
                                if is_self {
                                    // Account for people writing `Self::Variant::<Args>`, where
                                    // `Self` is the enum, and suggest replacing `Self` with the
                                    // appropriate type: `Type::<Args>::Variant`.
                                    (qself.span, format!("{type_name}{snippet}"))
                                } else {
                                    (qself_sugg_span, snippet)
                                },
                                (args_span, String::new()),
                            ];
                            err.multipart_suggestion_verbose(
                                msg,
                                suggestion,
                                Applicability::MaybeIncorrect,
                            );
                        });
                        return Ok((qself_ty, DefKind::Variant, variant_def.def_id));
                    } else {
                        variant_resolution = Some(variant_def.def_id);
                    }
                }
            }

            if let Some((ty, did)) = self.lookup_inherent_assoc_ty(
                assoc_ident,
                assoc_segment,
                adt_def.did(),
                qself_ty,
                hir_ref_id,
                span,
            )? {
                return Ok((ty, DefKind::AssocTy, did));
            }
        }

        // Find the type of the associated item, and the trait where the associated
        // item is declared.
        let bound = match (&qself_ty.kind(), qself_res) {
            (_, Res::SelfTyAlias { alias_to: impl_def_id, is_trait_impl: true, .. }) => {
                // `Self` in an impl of a trait -- we have a concrete self type and a
                // trait reference.
                let Some(trait_ref) = tcx.impl_trait_ref(impl_def_id) else {
                    // A cycle error occurred, most likely.
                    let guar = tcx.sess.delay_span_bug(span, "expected cycle error");
                    return Err(guar);
                };

                self.one_bound_for_assoc_type(
                    || {
                        traits::supertraits(
                            tcx,
                            ty::Binder::dummy(trait_ref.instantiate_identity()),
                        )
                    },
                    kw::SelfUpper,
                    assoc_ident,
                    span,
                    None,
                )?
            }
            (
                &ty::Param(_),
                Res::SelfTyParam { trait_: param_did } | Res::Def(DefKind::TyParam, param_did),
            ) => self.find_bound_for_assoc_item(param_did.expect_local(), assoc_ident, span)?,
            _ => {
                let reported = if variant_resolution.is_some() {
                    // Variant in type position
                    let msg = format!("expected type, found variant `{assoc_ident}`");
                    tcx.sess.span_err(span, msg)
                } else if qself_ty.is_enum() {
                    let mut err = struct_span_err!(
                        tcx.sess,
                        assoc_ident.span,
                        E0599,
                        "no variant named `{}` found for enum `{}`",
                        assoc_ident,
                        qself_ty,
                    );

                    let adt_def = qself_ty.ty_adt_def().expect("enum is not an ADT");
                    if let Some(suggested_name) = find_best_match_for_name(
                        &adt_def
                            .variants()
                            .iter()
                            .map(|variant| variant.name)
                            .collect::<Vec<Symbol>>(),
                        assoc_ident.name,
                        None,
                    ) {
                        err.span_suggestion(
                            assoc_ident.span,
                            "there is a variant with a similar name",
                            suggested_name,
                            Applicability::MaybeIncorrect,
                        );
                    } else {
                        err.span_label(
                            assoc_ident.span,
                            format!("variant not found in `{qself_ty}`"),
                        );
                    }

                    if let Some(sp) = tcx.hir().span_if_local(adt_def.did()) {
                        err.span_label(sp, format!("variant `{assoc_ident}` not found here"));
                    }

                    err.emit()
                } else if let Err(reported) = qself_ty.error_reported() {
                    reported
                } else if let ty::Alias(ty::Opaque, alias_ty) = qself_ty.kind() {
                    // `<impl Trait as OtherTrait>::Assoc` makes no sense.
                    struct_span_err!(
                        tcx.sess,
                        tcx.def_span(alias_ty.def_id),
                        E0667,
                        "`impl Trait` is not allowed in path parameters"
                    )
                    .emit() // Already reported in an earlier stage.
                } else {
                    let traits: Vec<_> =
                        self.probe_traits_that_match_assoc_ty(qself_ty, assoc_ident);

                    // Don't print `TyErr` to the user.
                    self.report_ambiguous_associated_type(
                        span,
                        &[qself_ty.to_string()],
                        &traits,
                        assoc_ident.name,
                    )
                };
                return Err(reported);
            }
        };

        let trait_did = bound.def_id();
        let Some(assoc_ty_did) = self.lookup_assoc_ty(assoc_ident, hir_ref_id, span, trait_did)
        else {
            // Assume that if it's not matched, there must be a const defined with the same name
            // but it was used in a type position.
            let msg = format!("found associated const `{assoc_ident}` when type was expected");
            let guar = tcx.sess.struct_span_err(span, msg).emit();
            return Err(guar);
        };

        let ty = self.projected_ty_from_poly_trait_ref(span, assoc_ty_did, assoc_segment, bound);

        if let Some(variant_def_id) = variant_resolution {
            tcx.struct_span_lint_hir(
                AMBIGUOUS_ASSOCIATED_ITEMS,
                hir_ref_id,
                span,
                "ambiguous associated item",
                |lint| {
                    let mut could_refer_to = |kind: DefKind, def_id, also| {
                        let note_msg = format!(
                            "`{}` could{} refer to the {} defined here",
                            assoc_ident,
                            also,
                            tcx.def_kind_descr(kind, def_id)
                        );
                        lint.span_note(tcx.def_span(def_id), note_msg);
                    };

                    could_refer_to(DefKind::Variant, variant_def_id, "");
                    could_refer_to(DefKind::AssocTy, assoc_ty_did, " also");

                    lint.span_suggestion(
                        span,
                        "use fully-qualified syntax",
                        format!("<{} as {}>::{}", qself_ty, tcx.item_name(trait_did), assoc_ident),
                        Applicability::MachineApplicable,
                    );

                    lint
                },
            );
        }
        Ok((ty, DefKind::AssocTy, assoc_ty_did))
    }

    fn lookup_inherent_assoc_ty(
        &self,
        name: Ident,
        segment: &hir::PathSegment<'_>,
        adt_did: DefId,
        self_ty: Ty<'tcx>,
        block: hir::HirId,
        span: Span,
    ) -> Result<Option<(Ty<'tcx>, DefId)>, ErrorGuaranteed> {
        let tcx = self.tcx();

        // Don't attempt to look up inherent associated types when the feature is not enabled.
        // Theoretically it'd be fine to do so since we feature-gate their definition site.
        // However, due to current limitations of the implementation (caused by us performing
        // selection in AstConv), IATs can lead to cycle errors (#108491, #110106) which mask the
        // feature-gate error, needlessly confusing users that use IATs by accident (#113265).
        if !tcx.features().inherent_associated_types {
            return Ok(None);
        }

        let candidates: Vec<_> = tcx
            .inherent_impls(adt_did)
            .iter()
            .filter_map(|&impl_| Some((impl_, self.lookup_assoc_ty_unchecked(name, block, impl_)?)))
            .collect();

        if candidates.is_empty() {
            return Ok(None);
        }

        //
        // Select applicable inherent associated type candidates modulo regions.
        //

        // In contexts that have no inference context, just make a new one.
        // We do need a local variable to store it, though.
        let infcx_;
        let infcx = match self.infcx() {
            Some(infcx) => infcx,
            None => {
                assert!(!self_ty.has_infer());
                infcx_ = tcx.infer_ctxt().ignoring_regions().build();
                &infcx_
            }
        };

        // FIXME(inherent_associated_types): Acquiring the ParamEnv this early leads to cycle errors
        // when inside of an ADT (#108491) or where clause.
        let param_env = tcx.param_env(block.owner);
        let cause = ObligationCause::misc(span, block.owner.def_id);

        let mut fulfillment_errors = Vec::new();
        let mut applicable_candidates: Vec<_> = infcx.probe(|_| {
            // Regions are not considered during selection.
            let self_ty = self_ty
                .fold_with(&mut BoundVarEraser { tcx, universe: infcx.create_next_universe() });

            struct BoundVarEraser<'tcx> {
                tcx: TyCtxt<'tcx>,
                universe: ty::UniverseIndex,
            }

            // FIXME(non_lifetime_binders): Don't assign the same universe to each placeholder.
            impl<'tcx> TypeFolder<TyCtxt<'tcx>> for BoundVarEraser<'tcx> {
                fn interner(&self) -> TyCtxt<'tcx> {
                    self.tcx
                }

                fn fold_region(&mut self, r: ty::Region<'tcx>) -> ty::Region<'tcx> {
                    if r.is_late_bound() { self.tcx.lifetimes.re_erased } else { r }
                }

                fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
                    match *ty.kind() {
                        ty::Bound(_, bv) => Ty::new_placeholder(
                            self.tcx,
                            ty::PlaceholderType { universe: self.universe, bound: bv },
                        ),
                        _ => ty.super_fold_with(self),
                    }
                }

                fn fold_const(
                    &mut self,
                    ct: ty::Const<'tcx>,
                ) -> <TyCtxt<'tcx> as rustc_type_ir::Interner>::Const {
                    assert!(!ct.ty().has_escaping_bound_vars());

                    match ct.kind() {
                        ty::ConstKind::Bound(_, bv) => ty::Const::new_placeholder(
                            self.tcx,
                            ty::PlaceholderConst { universe: self.universe, bound: bv },
                            ct.ty(),
                        ),
                        _ => ct.super_fold_with(self),
                    }
                }
            }

            let InferOk { value: self_ty, obligations } =
                infcx.at(&cause, param_env).normalize(self_ty);

            candidates
                .iter()
                .copied()
                .filter(|&(impl_, _)| {
                    infcx.probe(|_| {
                        let ocx = ObligationCtxt::new(&infcx);
                        ocx.register_obligations(obligations.clone());

                        let impl_args = infcx.fresh_args_for_item(span, impl_);
                        let impl_ty = tcx.type_of(impl_).instantiate(tcx, impl_args);
                        let impl_ty = ocx.normalize(&cause, param_env, impl_ty);

                        // Check that the self types can be related.
                        // FIXME(inherent_associated_types): Should we use `eq` here? Method probing uses
                        // `sup` for this situtation, too. What for? To constrain inference variables?
                        if ocx.sup(&ObligationCause::dummy(), param_env, impl_ty, self_ty).is_err()
                        {
                            return false;
                        }

                        // Check whether the impl imposes obligations we have to worry about.
                        let impl_bounds = tcx.predicates_of(impl_).instantiate(tcx, impl_args);
                        let impl_bounds = ocx.normalize(&cause, param_env, impl_bounds);
                        let impl_obligations = traits::predicates_for_generics(
                            |_, _| cause.clone(),
                            param_env,
                            impl_bounds,
                        );
                        ocx.register_obligations(impl_obligations);

                        let mut errors = ocx.select_where_possible();
                        if !errors.is_empty() {
                            fulfillment_errors.append(&mut errors);
                            return false;
                        }

                        true
                    })
                })
                .collect()
        });

        if applicable_candidates.len() > 1 {
            return Err(self.complain_about_ambiguous_inherent_assoc_type(
                name,
                applicable_candidates.into_iter().map(|(_, (candidate, _))| candidate).collect(),
                span,
            ));
        }

        if let Some((impl_, (assoc_item, def_scope))) = applicable_candidates.pop() {
            self.check_assoc_ty(assoc_item, name, def_scope, block, span);

            // FIXME(fmease): Currently creating throwaway `parent_args` to please
            // `create_args_for_associated_item`. Modify the latter instead (or sth. similar) to
            // not require the parent args logic.
            let parent_args = ty::GenericArgs::identity_for_item(tcx, impl_);
            let args = self.create_args_for_associated_item(span, assoc_item, segment, parent_args);
            let args = tcx.mk_args_from_iter(
                std::iter::once(ty::GenericArg::from(self_ty))
                    .chain(args.into_iter().skip(parent_args.len())),
            );

            let ty = Ty::new_alias(tcx, ty::Inherent, tcx.mk_alias_ty(assoc_item, args));

            return Ok(Some((ty, assoc_item)));
        }

        Err(self.complain_about_inherent_assoc_type_not_found(
            name,
            self_ty,
            candidates,
            fulfillment_errors,
            span,
        ))
    }

    fn lookup_assoc_ty(
        &self,
        name: Ident,
        block: hir::HirId,
        span: Span,
        scope: DefId,
    ) -> Option<DefId> {
        let (item, def_scope) = self.lookup_assoc_ty_unchecked(name, block, scope)?;
        self.check_assoc_ty(item, name, def_scope, block, span);
        Some(item)
    }

    fn lookup_assoc_ty_unchecked(
        &self,
        name: Ident,
        block: hir::HirId,
        scope: DefId,
    ) -> Option<(DefId, DefId)> {
        let tcx = self.tcx();
        let (ident, def_scope) = tcx.adjust_ident_and_get_scope(name, scope, block);

        // We have already adjusted the item name above, so compare with `ident.normalize_to_macros_2_0()` instead
        // of calling `find_by_name_and_kind`.
        let item = tcx.associated_items(scope).in_definition_order().find(|i| {
            i.kind.namespace() == Namespace::TypeNS
                && i.ident(tcx).normalize_to_macros_2_0() == ident
        })?;

        Some((item.def_id, def_scope))
    }

    fn check_assoc_ty(
        &self,
        item: DefId,
        name: Ident,
        def_scope: DefId,
        block: hir::HirId,
        span: Span,
    ) {
        let tcx = self.tcx();
        let kind = DefKind::AssocTy;

        if !tcx.visibility(item).is_accessible_from(def_scope, tcx) {
            let kind = tcx.def_kind_descr(kind, item);
            let msg = format!("{kind} `{name}` is private");
            let def_span = tcx.def_span(item);
            tcx.sess
                .struct_span_err_with_code(span, msg, rustc_errors::error_code!(E0624))
                .span_label(span, format!("private {kind}"))
                .span_label(def_span, format!("{kind} defined here"))
                .emit();
        }
        tcx.check_stability(item, Some(block), span, None);
    }

    fn probe_traits_that_match_assoc_ty(
        &self,
        qself_ty: Ty<'tcx>,
        assoc_ident: Ident,
    ) -> Vec<String> {
        let tcx = self.tcx();

        // In contexts that have no inference context, just make a new one.
        // We do need a local variable to store it, though.
        let infcx_;
        let infcx = if let Some(infcx) = self.infcx() {
            infcx
        } else {
            assert!(!qself_ty.has_infer());
            infcx_ = tcx.infer_ctxt().build();
            &infcx_
        };

        tcx.all_traits()
            .filter(|trait_def_id| {
                // Consider only traits with the associated type
                tcx.associated_items(*trait_def_id)
                        .in_definition_order()
                        .any(|i| {
                            i.kind.namespace() == Namespace::TypeNS
                                && i.ident(tcx).normalize_to_macros_2_0() == assoc_ident
                                && matches!(i.kind, ty::AssocKind::Type)
                        })
                    // Consider only accessible traits
                    && tcx.visibility(*trait_def_id)
                        .is_accessible_from(self.item_def_id(), tcx)
                    && tcx.all_impls(*trait_def_id)
                        .any(|impl_def_id| {
                            let trait_ref = tcx.impl_trait_ref(impl_def_id);
                            trait_ref.is_some_and(|trait_ref| {
                                let impl_ = trait_ref.instantiate(
                                    tcx,
                                    infcx.fresh_args_for_item(DUMMY_SP, impl_def_id),
                                );
                                let value = tcx.fold_regions(qself_ty, |_, _| tcx.lifetimes.re_erased);
                                // FIXME: Don't bother dealing with non-lifetime binders here...
                                if value.has_escaping_bound_vars() {
                                    return false;
                                }
                                infcx
                                    .can_eq(
                                        ty::ParamEnv::empty(),
                                        impl_.self_ty(),
                                        value,
                                    )
                            })
                            && tcx.impl_polarity(impl_def_id) != ty::ImplPolarity::Negative
                        })
            })
            .map(|trait_def_id| tcx.def_path_str(trait_def_id))
            .collect()
    }

    fn qpath_to_ty(
        &self,
        span: Span,
        opt_self_ty: Option<Ty<'tcx>>,
        item_def_id: DefId,
        trait_segment: &hir::PathSegment<'_>,
        item_segment: &hir::PathSegment<'_>,
        constness: ty::BoundConstness,
    ) -> Ty<'tcx> {
        let tcx = self.tcx();

        let trait_def_id = tcx.parent(item_def_id);

        debug!("qpath_to_ty: trait_def_id={:?}", trait_def_id);

        let Some(self_ty) = opt_self_ty else {
            let path_str = tcx.def_path_str(trait_def_id);

            let def_id = self.item_def_id();

            debug!("qpath_to_ty: self.item_def_id()={:?}", def_id);

            let parent_def_id = def_id
                .as_local()
                .map(|def_id| tcx.hir().local_def_id_to_hir_id(def_id))
                .map(|hir_id| tcx.hir().get_parent_item(hir_id).to_def_id());

            debug!("qpath_to_ty: parent_def_id={:?}", parent_def_id);

            // If the trait in segment is the same as the trait defining the item,
            // use the `<Self as ..>` syntax in the error.
            let is_part_of_self_trait_constraints = def_id == trait_def_id;
            let is_part_of_fn_in_self_trait = parent_def_id == Some(trait_def_id);

            let type_names = if is_part_of_self_trait_constraints || is_part_of_fn_in_self_trait {
                vec!["Self".to_string()]
            } else {
                // Find all the types that have an `impl` for the trait.
                tcx.all_impls(trait_def_id)
                    .filter(|impl_def_id| {
                        // Consider only accessible traits
                        tcx.visibility(trait_def_id).is_accessible_from(self.item_def_id(), tcx)
                            && tcx.impl_polarity(impl_def_id) != ty::ImplPolarity::Negative
                    })
                    .filter_map(|impl_def_id| tcx.impl_trait_ref(impl_def_id))
                    .map(|impl_| impl_.instantiate_identity().self_ty())
                    // We don't care about blanket impls.
                    .filter(|self_ty| !self_ty.has_non_region_param())
                    .map(|self_ty| tcx.erase_regions(self_ty).to_string())
                    .collect()
            };
            // FIXME: also look at `tcx.generics_of(self.item_def_id()).params` any that
            // references the trait. Relevant for the first case in
            // `src/test/ui/associated-types/associated-types-in-ambiguous-context.rs`
            let reported = self.report_ambiguous_associated_type(
                span,
                &type_names,
                &[path_str],
                item_segment.ident.name,
            );
            return Ty::new_error(tcx, reported);
        };

        debug!("qpath_to_ty: self_type={:?}", self_ty);

        let trait_ref = self.ast_path_to_mono_trait_ref(
            span,
            trait_def_id,
            self_ty,
            trait_segment,
            false,
            constness,
        );

        let item_args =
            self.create_args_for_associated_item(span, item_def_id, item_segment, trait_ref.args);

        debug!("qpath_to_ty: trait_ref={:?}", trait_ref);

        Ty::new_projection(tcx, item_def_id, item_args)
    }

    pub fn prohibit_generics<'a>(
        &self,
        segments: impl Iterator<Item = &'a hir::PathSegment<'a>> + Clone,
        extend: impl Fn(&mut Diagnostic),
    ) -> bool {
        let args = segments.clone().flat_map(|segment| segment.args().args);

        let (lt, ty, ct, inf) =
            args.clone().fold((false, false, false, false), |(lt, ty, ct, inf), arg| match arg {
                hir::GenericArg::Lifetime(_) => (true, ty, ct, inf),
                hir::GenericArg::Type(_) => (lt, true, ct, inf),
                hir::GenericArg::Const(_) => (lt, ty, true, inf),
                hir::GenericArg::Infer(_) => (lt, ty, ct, true),
            });
        let mut emitted = false;
        if lt || ty || ct || inf {
            let types_and_spans: Vec<_> = segments
                .clone()
                .flat_map(|segment| {
                    if segment.args().args.is_empty() {
                        None
                    } else {
                        Some((
                            match segment.res {
                                Res::PrimTy(ty) => format!("{} `{}`", segment.res.descr(), ty.name()),
                                Res::Def(_, def_id)
                                if let Some(name) = self.tcx().opt_item_name(def_id) => {
                                    format!("{} `{name}`", segment.res.descr())
                                }
                                Res::Err => "this type".to_string(),
                                _ => segment.res.descr().to_string(),
                            },
                            segment.ident.span,
                        ))
                    }
                })
                .collect();
            let this_type = match &types_and_spans[..] {
                [.., _, (last, _)] => format!(
                    "{} and {last}",
                    types_and_spans[..types_and_spans.len() - 1]
                        .iter()
                        .map(|(x, _)| x.as_str())
                        .intersperse(&", ")
                        .collect::<String>()
                ),
                [(only, _)] => only.to_string(),
                [] => "this type".to_string(),
            };

            let arg_spans: Vec<Span> = args.map(|arg| arg.span()).collect();

            let mut kinds = Vec::with_capacity(4);
            if lt {
                kinds.push("lifetime");
            }
            if ty {
                kinds.push("type");
            }
            if ct {
                kinds.push("const");
            }
            if inf {
                kinds.push("generic");
            }
            let (kind, s) = match kinds[..] {
                [.., _, last] => (
                    format!(
                        "{} and {last}",
                        kinds[..kinds.len() - 1]
                            .iter()
                            .map(|&x| x)
                            .intersperse(", ")
                            .collect::<String>()
                    ),
                    "s",
                ),
                [only] => (only.to_string(), ""),
                [] => unreachable!(),
            };
            let last_span = *arg_spans.last().unwrap();
            let span: MultiSpan = arg_spans.into();
            let mut err = struct_span_err!(
                self.tcx().sess,
                span,
                E0109,
                "{kind} arguments are not allowed on {this_type}",
            );
            err.span_label(last_span, format!("{kind} argument{s} not allowed"));
            for (what, span) in types_and_spans {
                err.span_label(span, format!("not allowed on {what}"));
            }
            extend(&mut err);
            err.emit();
            emitted = true;
        }

        for segment in segments {
            // Only emit the first error to avoid overloading the user with error messages.
            if let Some(b) = segment.args().bindings.first() {
                prohibit_assoc_ty_binding(self.tcx(), b.span, None);
                return true;
            }
        }
        emitted
    }

    // FIXME(eddyb, varkor) handle type paths here too, not just value ones.
    pub fn def_ids_for_value_path_segments(
        &self,
        segments: &[hir::PathSegment<'_>],
        self_ty: Option<Ty<'tcx>>,
        kind: DefKind,
        def_id: DefId,
        span: Span,
    ) -> Vec<PathSeg> {
        // We need to extract the type parameters supplied by the user in
        // the path `path`. Due to the current setup, this is a bit of a
        // tricky-process; the problem is that resolve only tells us the
        // end-point of the path resolution, and not the intermediate steps.
        // Luckily, we can (at least for now) deduce the intermediate steps
        // just from the end-point.
        //
        // There are basically five cases to consider:
        //
        // 1. Reference to a constructor of a struct:
        //
        //        struct Foo<T>(...)
        //
        //    In this case, the parameters are declared in the type space.
        //
        // 2. Reference to a constructor of an enum variant:
        //
        //        enum E<T> { Foo(...) }
        //
        //    In this case, the parameters are defined in the type space,
        //    but may be specified either on the type or the variant.
        //
        // 3. Reference to a fn item or a free constant:
        //
        //        fn foo<T>() { }
        //
        //    In this case, the path will again always have the form
        //    `a::b::foo::<T>` where only the final segment should have
        //    type parameters. However, in this case, those parameters are
        //    declared on a value, and hence are in the `FnSpace`.
        //
        // 4. Reference to a method or an associated constant:
        //
        //        impl<A> SomeStruct<A> {
        //            fn foo<B>(...)
        //        }
        //
        //    Here we can have a path like
        //    `a::b::SomeStruct::<A>::foo::<B>`, in which case parameters
        //    may appear in two places. The penultimate segment,
        //    `SomeStruct::<A>`, contains parameters in TypeSpace, and the
        //    final segment, `foo::<B>` contains parameters in fn space.
        //
        // The first step then is to categorize the segments appropriately.

        let tcx = self.tcx();

        assert!(!segments.is_empty());
        let last = segments.len() - 1;

        let mut path_segs = vec![];

        match kind {
            // Case 1. Reference to a struct constructor.
            DefKind::Ctor(CtorOf::Struct, ..) => {
                // Everything but the final segment should have no
                // parameters at all.
                let generics = tcx.generics_of(def_id);
                // Variant and struct constructors use the
                // generics of their parent type definition.
                let generics_def_id = generics.parent.unwrap_or(def_id);
                path_segs.push(PathSeg(generics_def_id, last));
            }

            // Case 2. Reference to a variant constructor.
            DefKind::Ctor(CtorOf::Variant, ..) | DefKind::Variant => {
                let (generics_def_id, index) = if let Some(self_ty) = self_ty {
                    let adt_def = self.probe_adt(span, self_ty).unwrap();
                    debug_assert!(adt_def.is_enum());
                    (adt_def.did(), last)
                } else if last >= 1 && segments[last - 1].args.is_some() {
                    // Everything but the penultimate segment should have no
                    // parameters at all.
                    let mut def_id = def_id;

                    // `DefKind::Ctor` -> `DefKind::Variant`
                    if let DefKind::Ctor(..) = kind {
                        def_id = tcx.parent(def_id);
                    }

                    // `DefKind::Variant` -> `DefKind::Enum`
                    let enum_def_id = tcx.parent(def_id);
                    (enum_def_id, last - 1)
                } else {
                    // FIXME: lint here recommending `Enum::<...>::Variant` form
                    // instead of `Enum::Variant::<...>` form.

                    // Everything but the final segment should have no
                    // parameters at all.
                    let generics = tcx.generics_of(def_id);
                    // Variant and struct constructors use the
                    // generics of their parent type definition.
                    (generics.parent.unwrap_or(def_id), last)
                };
                path_segs.push(PathSeg(generics_def_id, index));
            }

            // Case 3. Reference to a top-level value.
            DefKind::Fn | DefKind::Const | DefKind::ConstParam | DefKind::Static(_) => {
                path_segs.push(PathSeg(def_id, last));
            }

            // Case 4. Reference to a method or associated const.
            DefKind::AssocFn | DefKind::AssocConst => {
                if segments.len() >= 2 {
                    let generics = tcx.generics_of(def_id);
                    path_segs.push(PathSeg(generics.parent.unwrap(), last - 1));
                }
                path_segs.push(PathSeg(def_id, last));
            }

            kind => bug!("unexpected definition kind {:?} for {:?}", kind, def_id),
        }

        debug!("path_segs = {:?}", path_segs);

        path_segs
    }

    /// Check a type `Path` and convert it to a `Ty`.
    pub fn res_to_ty(
        &self,
        opt_self_ty: Option<Ty<'tcx>>,
        path: &hir::Path<'_>,
        hir_id: hir::HirId,
        permit_variants: bool,
    ) -> Ty<'tcx> {
        let tcx = self.tcx();

        debug!(
            "res_to_ty(res={:?}, opt_self_ty={:?}, path_segments={:?})",
            path.res, opt_self_ty, path.segments
        );

        let span = path.span;
        match path.res {
            Res::Def(DefKind::OpaqueTy, did) => {
                // Check for desugared `impl Trait`.
                assert!(tcx.is_type_alias_impl_trait(did));
                let item_segment = path.segments.split_last().unwrap();
                self.prohibit_generics(item_segment.1.iter(), |err| {
                    err.note("`impl Trait` types can't have type parameters");
                });
                let args = self.ast_path_args_for_ty(span, did, item_segment.0);
                Ty::new_opaque(tcx, did, args)
            }
            Res::Def(
                DefKind::Enum
                | DefKind::TyAlias
                | DefKind::Struct
                | DefKind::Union
                | DefKind::ForeignTy,
                did,
            ) => {
                assert_eq!(opt_self_ty, None);
                self.prohibit_generics(path.segments.split_last().unwrap().1.iter(), |_| {});
                self.ast_path_to_ty(span, did, path.segments.last().unwrap())
            }
            Res::Def(kind @ DefKind::Variant, def_id) if permit_variants => {
                // Convert "variant type" as if it were a real type.
                // The resulting `Ty` is type of the variant's enum for now.
                assert_eq!(opt_self_ty, None);

                let path_segs =
                    self.def_ids_for_value_path_segments(path.segments, None, kind, def_id, span);
                let generic_segs: FxHashSet<_> =
                    path_segs.iter().map(|PathSeg(_, index)| index).collect();
                self.prohibit_generics(
                    path.segments.iter().enumerate().filter_map(|(index, seg)| {
                        if !generic_segs.contains(&index) { Some(seg) } else { None }
                    }),
                    |err| {
                        err.note("enum variants can't have type parameters");
                    },
                );

                let PathSeg(def_id, index) = path_segs.last().unwrap();
                self.ast_path_to_ty(span, *def_id, &path.segments[*index])
            }
            Res::Def(DefKind::TyParam, def_id) => {
                assert_eq!(opt_self_ty, None);
                self.prohibit_generics(path.segments.iter(), |err| {
                    if let Some(span) = tcx.def_ident_span(def_id) {
                        let name = tcx.item_name(def_id);
                        err.span_note(span, format!("type parameter `{name}` defined here"));
                    }
                });

                match tcx.named_bound_var(hir_id) {
                    Some(rbv::ResolvedArg::LateBound(debruijn, index, _)) => {
                        let name =
                            tcx.hir().name(tcx.hir().local_def_id_to_hir_id(def_id.expect_local()));
                        let br = ty::BoundTy {
                            var: ty::BoundVar::from_u32(index),
                            kind: ty::BoundTyKind::Param(def_id, name),
                        };
                        Ty::new_bound(tcx, debruijn, br)
                    }
                    Some(rbv::ResolvedArg::EarlyBound(_)) => {
                        let def_id = def_id.expect_local();
                        let item_def_id = tcx.hir().ty_param_owner(def_id);
                        let generics = tcx.generics_of(item_def_id);
                        let index = generics.param_def_id_to_index[&def_id.to_def_id()];
                        Ty::new_param(tcx, index, tcx.hir().ty_param_name(def_id))
                    }
                    Some(rbv::ResolvedArg::Error(guar)) => Ty::new_error(tcx, guar),
                    arg => bug!("unexpected bound var resolution for {hir_id:?}: {arg:?}"),
                }
            }
            Res::SelfTyParam { .. } => {
                // `Self` in trait or type alias.
                assert_eq!(opt_self_ty, None);
                self.prohibit_generics(path.segments.iter(), |err| {
                    if let [hir::PathSegment { args: Some(args), ident, .. }] = &path.segments {
                        err.span_suggestion_verbose(
                            ident.span.shrink_to_hi().to(args.span_ext),
                            "the `Self` type doesn't accept type parameters",
                            "",
                            Applicability::MaybeIncorrect,
                        );
                    }
                });
                tcx.types.self_param
            }
            Res::SelfTyAlias { alias_to: def_id, forbid_generic, .. } => {
                // `Self` in impl (we know the concrete type).
                assert_eq!(opt_self_ty, None);
                // Try to evaluate any array length constants.
                let ty = tcx.at(span).type_of(def_id).instantiate_identity();
                let span_of_impl = tcx.span_of_impl(def_id);
                self.prohibit_generics(path.segments.iter(), |err| {
                    let def_id = match *ty.kind() {
                        ty::Adt(self_def, _) => self_def.did(),
                        _ => return,
                    };

                    let type_name = tcx.item_name(def_id);
                    let span_of_ty = tcx.def_ident_span(def_id);
                    let generics = tcx.generics_of(def_id).count();

                    let msg = format!("`Self` is of type `{ty}`");
                    if let (Ok(i_sp), Some(t_sp)) = (span_of_impl, span_of_ty) {
                        let mut span: MultiSpan = vec![t_sp].into();
                        span.push_span_label(
                            i_sp,
                            format!("`Self` is on type `{type_name}` in this `impl`"),
                        );
                        let mut postfix = "";
                        if generics == 0 {
                            postfix = ", which doesn't have generic parameters";
                        }
                        span.push_span_label(
                            t_sp,
                            format!("`Self` corresponds to this type{postfix}"),
                        );
                        err.span_note(span, msg);
                    } else {
                        err.note(msg);
                    }
                    for segment in path.segments {
                        if let Some(args) = segment.args && segment.ident.name == kw::SelfUpper {
                            if generics == 0 {
                                // FIXME(estebank): we could also verify that the arguments being
                                // work for the `enum`, instead of just looking if it takes *any*.
                                err.span_suggestion_verbose(
                                    segment.ident.span.shrink_to_hi().to(args.span_ext),
                                    "the `Self` type doesn't accept type parameters",
                                    "",
                                    Applicability::MachineApplicable,
                                );
                                return;
                            } else {
                                err.span_suggestion_verbose(
                                    segment.ident.span,
                                    format!(
                                        "the `Self` type doesn't accept type parameters, use the \
                                        concrete type's name `{type_name}` instead if you want to \
                                        specify its type parameters"
                                    ),
                                    type_name,
                                    Applicability::MaybeIncorrect,
                                );
                            }
                        }
                    }
                });
                // HACK(min_const_generics): Forbid generic `Self` types
                // here as we can't easily do that during nameres.
                //
                // We do this before normalization as we otherwise allow
                // ```rust
                // trait AlwaysApplicable { type Assoc; }
                // impl<T: ?Sized> AlwaysApplicable for T { type Assoc = usize; }
                //
                // trait BindsParam<T> {
                //     type ArrayTy;
                // }
                // impl<T> BindsParam<T> for <T as AlwaysApplicable>::Assoc {
                //    type ArrayTy = [u8; Self::MAX];
                // }
                // ```
                // Note that the normalization happens in the param env of
                // the anon const, which is empty. This is why the
                // `AlwaysApplicable` impl needs a `T: ?Sized` bound for
                // this to compile if we were to normalize here.
                if forbid_generic && ty.has_param() {
                    let mut err = tcx.sess.struct_span_err(
                        path.span,
                        "generic `Self` types are currently not permitted in anonymous constants",
                    );
                    if let Some(hir::Node::Item(&hir::Item {
                        kind: hir::ItemKind::Impl(impl_),
                        ..
                    })) = tcx.hir().get_if_local(def_id)
                    {
                        err.span_note(impl_.self_ty.span, "not a concrete type");
                    }
                    Ty::new_error(tcx, err.emit())
                } else {
                    ty
                }
            }
            Res::Def(DefKind::AssocTy, def_id) => {
                debug_assert!(path.segments.len() >= 2);
                self.prohibit_generics(path.segments[..path.segments.len() - 2].iter(), |_| {});
                // HACK: until we support `<Type as ~const Trait>`, assume all of them are.
                let constness = if tcx.has_attr(tcx.parent(def_id), sym::const_trait) {
                    ty::BoundConstness::ConstIfConst
                } else {
                    ty::BoundConstness::NotConst
                };
                self.qpath_to_ty(
                    span,
                    opt_self_ty,
                    def_id,
                    &path.segments[path.segments.len() - 2],
                    path.segments.last().unwrap(),
                    constness,
                )
            }
            Res::PrimTy(prim_ty) => {
                assert_eq!(opt_self_ty, None);
                self.prohibit_generics(path.segments.iter(), |err| {
                    let name = prim_ty.name_str();
                    for segment in path.segments {
                        if let Some(args) = segment.args {
                            err.span_suggestion_verbose(
                                segment.ident.span.shrink_to_hi().to(args.span_ext),
                                format!("primitive type `{name}` doesn't have generic parameters"),
                                "",
                                Applicability::MaybeIncorrect,
                            );
                        }
                    }
                });
                match prim_ty {
                    hir::PrimTy::Bool => tcx.types.bool,
                    hir::PrimTy::Char => tcx.types.char,
                    hir::PrimTy::Int(it) => Ty::new_int(tcx, ty::int_ty(it)),
                    hir::PrimTy::Uint(uit) => Ty::new_uint(tcx, ty::uint_ty(uit)),
                    hir::PrimTy::Float(ft) => Ty::new_float(tcx, ty::float_ty(ft)),
                    hir::PrimTy::Str => tcx.types.str_,
                }
            }
            Res::Err => {
                let e = self
                    .tcx()
                    .sess
                    .delay_span_bug(path.span, "path with `Res::Err` but no error emitted");
                self.set_tainted_by_errors(e);
                Ty::new_error(self.tcx(), e)
            }
            _ => span_bug!(span, "unexpected resolution: {:?}", path.res),
        }
    }

    /// Parses the programmer's textual representation of a type into our
    /// internal notion of a type.
    pub fn ast_ty_to_ty(&self, ast_ty: &hir::Ty<'_>) -> Ty<'tcx> {
        self.ast_ty_to_ty_inner(ast_ty, false, false)
    }

    /// Parses the programmer's textual representation of a type into our
    /// internal notion of a type. This is meant to be used within a path.
    pub fn ast_ty_to_ty_in_path(&self, ast_ty: &hir::Ty<'_>) -> Ty<'tcx> {
        self.ast_ty_to_ty_inner(ast_ty, false, true)
    }

    /// Turns a `hir::Ty` into a `Ty`. For diagnostics' purposes we keep track of whether trait
    /// objects are borrowed like `&dyn Trait` to avoid emitting redundant errors.
    #[instrument(level = "debug", skip(self), ret)]
    fn ast_ty_to_ty_inner(&self, ast_ty: &hir::Ty<'_>, borrowed: bool, in_path: bool) -> Ty<'tcx> {
        let tcx = self.tcx();

        let result_ty = match &ast_ty.kind {
            hir::TyKind::Slice(ty) => Ty::new_slice(tcx, self.ast_ty_to_ty(ty)),
            hir::TyKind::Ptr(mt) => {
                Ty::new_ptr(tcx, ty::TypeAndMut { ty: self.ast_ty_to_ty(mt.ty), mutbl: mt.mutbl })
            }
            hir::TyKind::Ref(region, mt) => {
                let r = self.ast_region_to_region(region, None);
                debug!(?r);
                let t = self.ast_ty_to_ty_inner(mt.ty, true, false);
                Ty::new_ref(tcx, r, ty::TypeAndMut { ty: t, mutbl: mt.mutbl })
            }
            hir::TyKind::Never => tcx.types.never,
            hir::TyKind::Tup(fields) => {
                Ty::new_tup_from_iter(tcx, fields.iter().map(|t| self.ast_ty_to_ty(t)))
            }
            hir::TyKind::BareFn(bf) => {
                require_c_abi_if_c_variadic(tcx, bf.decl, bf.abi, ast_ty.span);

                Ty::new_fn_ptr(
                    tcx,
                    self.ty_of_fn(ast_ty.hir_id, bf.unsafety, bf.abi, bf.decl, None, Some(ast_ty)),
                )
            }
            hir::TyKind::TraitObject(bounds, lifetime, repr) => {
                self.maybe_lint_bare_trait(ast_ty, in_path);
                let repr = match repr {
                    TraitObjectSyntax::Dyn | TraitObjectSyntax::None => ty::Dyn,
                    TraitObjectSyntax::DynStar => ty::DynStar,
                };

                self.conv_object_ty_poly_trait_ref(
                    ast_ty.span,
                    ast_ty.hir_id,
                    bounds,
                    lifetime,
                    borrowed,
                    repr,
                )
            }
            hir::TyKind::Path(hir::QPath::Resolved(maybe_qself, path)) => {
                debug!(?maybe_qself, ?path);
                let opt_self_ty = maybe_qself.as_ref().map(|qself| self.ast_ty_to_ty(qself));
                self.res_to_ty(opt_self_ty, path, ast_ty.hir_id, false)
            }
            &hir::TyKind::OpaqueDef(item_id, lifetimes, in_trait) => {
                let opaque_ty = tcx.hir().item(item_id);

                match opaque_ty.kind {
                    hir::ItemKind::OpaqueTy(&hir::OpaqueTy { origin, .. }) => {
                        let local_def_id = item_id.owner_id.def_id;
                        // If this is an RPITIT and we are using the new RPITIT lowering scheme, we
                        // generate the def_id of an associated type for the trait and return as
                        // type a projection.
                        let def_id = if in_trait {
                            tcx.associated_type_for_impl_trait_in_trait(local_def_id).to_def_id()
                        } else {
                            local_def_id.to_def_id()
                        };
                        self.impl_trait_ty_to_ty(def_id, lifetimes, origin, in_trait)
                    }
                    ref i => bug!("`impl Trait` pointed to non-opaque type?? {:#?}", i),
                }
            }
            hir::TyKind::Path(hir::QPath::TypeRelative(qself, segment)) => {
                debug!(?qself, ?segment);
                let ty = self.ast_ty_to_ty_inner(qself, false, true);
                self.associated_path_to_ty(ast_ty.hir_id, ast_ty.span, ty, qself, segment, false)
                    .map(|(ty, _, _)| ty)
                    .unwrap_or_else(|guar| Ty::new_error(tcx, guar))
            }
            &hir::TyKind::Path(hir::QPath::LangItem(lang_item, span, _)) => {
                let def_id = tcx.require_lang_item(lang_item, Some(span));
                let (args, _) = self.create_args_for_ast_path(
                    span,
                    def_id,
                    &[],
                    &hir::PathSegment::invalid(),
                    &GenericArgs::none(),
                    true,
                    None,
                    ty::BoundConstness::NotConst,
                );
                tcx.at(span).type_of(def_id).instantiate(tcx, args)
            }
            hir::TyKind::Array(ty, length) => {
                let length = match length {
                    &hir::ArrayLen::Infer(_, span) => self.ct_infer(tcx.types.usize, None, span),
                    hir::ArrayLen::Body(constant) => {
                        ty::Const::from_anon_const(tcx, constant.def_id)
                    }
                };

                Ty::new_array_with_const_len(tcx, self.ast_ty_to_ty(ty), length)
            }
            hir::TyKind::Typeof(e) => {
                let ty_erased = tcx.type_of(e.def_id).instantiate_identity();
                let ty = tcx.fold_regions(ty_erased, |r, _| {
                    if r.is_erased() { tcx.lifetimes.re_static } else { r }
                });
                let span = ast_ty.span;
                let (ty, opt_sugg) = if let Some(ty) = ty.make_suggestable(tcx, false) {
                    (ty, Some((span, Applicability::MachineApplicable)))
                } else {
                    (ty, None)
                };
                tcx.sess.emit_err(TypeofReservedKeywordUsed { span, ty, opt_sugg });

                ty
            }
            hir::TyKind::Infer => {
                // Infer also appears as the type of arguments or return
                // values in an ExprKind::Closure, or as
                // the type of local variables. Both of these cases are
                // handled specially and will not descend into this routine.
                self.ty_infer(None, ast_ty.span)
            }
            hir::TyKind::Err(guar) => Ty::new_error(tcx, *guar),
        };

        self.record_ty(ast_ty.hir_id, result_ty, ast_ty.span);
        result_ty
    }

    #[instrument(level = "debug", skip(self), ret)]
    fn impl_trait_ty_to_ty(
        &self,
        def_id: DefId,
        lifetimes: &[hir::GenericArg<'_>],
        origin: OpaqueTyOrigin,
        in_trait: bool,
    ) -> Ty<'tcx> {
        debug!("impl_trait_ty_to_ty(def_id={:?}, lifetimes={:?})", def_id, lifetimes);
        let tcx = self.tcx();

        let generics = tcx.generics_of(def_id);

        debug!("impl_trait_ty_to_ty: generics={:?}", generics);
        let args = ty::GenericArgs::for_item(tcx, def_id, |param, _| {
            // We use `generics.count() - lifetimes.len()` here instead of `generics.parent_count`
            // since return-position impl trait in trait squashes all of the generics from its source fn
            // into its own generics, so the opaque's "own" params isn't always just lifetimes.
            if let Some(i) = (param.index as usize).checked_sub(generics.count() - lifetimes.len())
            {
                // Resolve our own lifetime parameters.
                let GenericParamDefKind::Lifetime { .. } = param.kind else { bug!() };
                let hir::GenericArg::Lifetime(lifetime) = &lifetimes[i] else { bug!() };
                self.ast_region_to_region(lifetime, None).into()
            } else {
                tcx.mk_param_from_def(param)
            }
        });
        debug!("impl_trait_ty_to_ty: args={:?}", args);

        if in_trait {
            Ty::new_projection(tcx, def_id, args)
        } else {
            Ty::new_opaque(tcx, def_id, args)
        }
    }

    pub fn ty_of_arg(&self, ty: &hir::Ty<'_>, expected_ty: Option<Ty<'tcx>>) -> Ty<'tcx> {
        match ty.kind {
            hir::TyKind::Infer if expected_ty.is_some() => {
                self.record_ty(ty.hir_id, expected_ty.unwrap(), ty.span);
                expected_ty.unwrap()
            }
            _ => self.ast_ty_to_ty(ty),
        }
    }

    #[instrument(level = "debug", skip(self, hir_id, unsafety, abi, decl, generics, hir_ty), ret)]
    pub fn ty_of_fn(
        &self,
        hir_id: hir::HirId,
        unsafety: hir::Unsafety,
        abi: abi::Abi,
        decl: &hir::FnDecl<'_>,
        generics: Option<&hir::Generics<'_>>,
        hir_ty: Option<&hir::Ty<'_>>,
    ) -> ty::PolyFnSig<'tcx> {
        let tcx = self.tcx();
        let bound_vars = tcx.late_bound_vars(hir_id);
        debug!(?bound_vars);

        // We proactively collect all the inferred type params to emit a single error per fn def.
        let mut visitor = HirPlaceholderCollector::default();
        let mut infer_replacements = vec![];

        if let Some(generics) = generics {
            walk_generics(&mut visitor, generics);
        }

        let input_tys: Vec<_> = decl
            .inputs
            .iter()
            .enumerate()
            .map(|(i, a)| {
                if let hir::TyKind::Infer = a.kind && !self.allow_ty_infer() {
                    if let Some(suggested_ty) =
                        self.suggest_trait_fn_ty_for_impl_fn_infer(hir_id, Some(i))
                    {
                        infer_replacements.push((a.span, suggested_ty.to_string()));
                        return suggested_ty;
                    }
                }

                // Only visit the type looking for `_` if we didn't fix the type above
                visitor.visit_ty(a);
                self.ty_of_arg(a, None)
            })
            .collect();

        let output_ty = match decl.output {
            hir::FnRetTy::Return(output) => {
                if let hir::TyKind::Infer = output.kind
                    && !self.allow_ty_infer()
                    && let Some(suggested_ty) =
                        self.suggest_trait_fn_ty_for_impl_fn_infer(hir_id, None)
                {
                    infer_replacements.push((output.span, suggested_ty.to_string()));
                    suggested_ty
                } else {
                    visitor.visit_ty(output);
                    self.ast_ty_to_ty(output)
                }
            }
            hir::FnRetTy::DefaultReturn(..) => Ty::new_unit(tcx,),
        };

        debug!(?output_ty);

        let fn_ty = tcx.mk_fn_sig(input_tys, output_ty, decl.c_variadic, unsafety, abi);
        let bare_fn_ty = ty::Binder::bind_with_vars(fn_ty, bound_vars);

        if !self.allow_ty_infer() && !(visitor.0.is_empty() && infer_replacements.is_empty()) {
            // We always collect the spans for placeholder types when evaluating `fn`s, but we
            // only want to emit an error complaining about them if infer types (`_`) are not
            // allowed. `allow_ty_infer` gates this behavior. We check for the presence of
            // `ident_span` to not emit an error twice when we have `fn foo(_: fn() -> _)`.

            let mut diag = crate::collect::placeholder_type_error_diag(
                tcx,
                generics,
                visitor.0,
                infer_replacements.iter().map(|(s, _)| *s).collect(),
                true,
                hir_ty,
                "function",
            );

            if !infer_replacements.is_empty() {
                diag.multipart_suggestion(
                    format!(
                    "try replacing `_` with the type{} in the corresponding trait method signature",
                    rustc_errors::pluralize!(infer_replacements.len()),
                ),
                    infer_replacements,
                    Applicability::MachineApplicable,
                );
            }

            diag.emit();
        }

        // Find any late-bound regions declared in return type that do
        // not appear in the arguments. These are not well-formed.
        //
        // Example:
        //     for<'a> fn() -> &'a str <-- 'a is bad
        //     for<'a> fn(&'a String) -> &'a str <-- 'a is ok
        let inputs = bare_fn_ty.inputs();
        let late_bound_in_args =
            tcx.collect_constrained_late_bound_regions(&inputs.map_bound(|i| i.to_owned()));
        let output = bare_fn_ty.output();
        let late_bound_in_ret = tcx.collect_referenced_late_bound_regions(&output);

        self.validate_late_bound_regions(late_bound_in_args, late_bound_in_ret, |br_name| {
            struct_span_err!(
                tcx.sess,
                decl.output.span(),
                E0581,
                "return type references {}, which is not constrained by the fn input types",
                br_name
            )
        });

        bare_fn_ty
    }

    /// Given a fn_hir_id for a impl function, suggest the type that is found on the
    /// corresponding function in the trait that the impl implements, if it exists.
    /// If arg_idx is Some, then it corresponds to an input type index, otherwise it
    /// corresponds to the return type.
    fn suggest_trait_fn_ty_for_impl_fn_infer(
        &self,
        fn_hir_id: hir::HirId,
        arg_idx: Option<usize>,
    ) -> Option<Ty<'tcx>> {
        let tcx = self.tcx();
        let hir = tcx.hir();

        let hir::Node::ImplItem(hir::ImplItem { kind: hir::ImplItemKind::Fn(..), ident, .. }) =
            hir.get(fn_hir_id)
        else {
            return None;
        };
        let i = hir.get_parent(fn_hir_id).expect_item().expect_impl();

        let trait_ref = self.instantiate_mono_trait_ref(
            i.of_trait.as_ref()?,
            self.ast_ty_to_ty(i.self_ty),
            ty::BoundConstness::NotConst,
        );

        let assoc = tcx.associated_items(trait_ref.def_id).find_by_name_and_kind(
            tcx,
            *ident,
            ty::AssocKind::Fn,
            trait_ref.def_id,
        )?;

        let fn_sig = tcx.fn_sig(assoc.def_id).instantiate(
            tcx,
            trait_ref.args.extend_to(tcx, assoc.def_id, |param, _| tcx.mk_param_from_def(param)),
        );
        let fn_sig = tcx.liberate_late_bound_regions(fn_hir_id.expect_owner().to_def_id(), fn_sig);

        Some(if let Some(arg_idx) = arg_idx {
            *fn_sig.inputs().get(arg_idx)?
        } else {
            fn_sig.output()
        })
    }

    #[instrument(level = "trace", skip(self, generate_err))]
    fn validate_late_bound_regions(
        &self,
        constrained_regions: FxHashSet<ty::BoundRegionKind>,
        referenced_regions: FxHashSet<ty::BoundRegionKind>,
        generate_err: impl Fn(&str) -> DiagnosticBuilder<'tcx, ErrorGuaranteed>,
    ) {
        for br in referenced_regions.difference(&constrained_regions) {
            let br_name = match *br {
                ty::BrNamed(_, kw::UnderscoreLifetime) | ty::BrAnon(..) | ty::BrEnv => {
                    "an anonymous lifetime".to_string()
                }
                ty::BrNamed(_, name) => format!("lifetime `{name}`"),
            };

            let mut err = generate_err(&br_name);

            if let ty::BrNamed(_, kw::UnderscoreLifetime) | ty::BrAnon(..) = *br {
                // The only way for an anonymous lifetime to wind up
                // in the return type but **also** be unconstrained is
                // if it only appears in "associated types" in the
                // input. See #47511 and #62200 for examples. In this case,
                // though we can easily give a hint that ought to be
                // relevant.
                err.note(
                    "lifetimes appearing in an associated or opaque type are not considered constrained",
                );
                err.note("consider introducing a named lifetime parameter");
            }

            err.emit();
        }
    }

    /// Given the bounds on an object, determines what single region bound (if any) we can
    /// use to summarize this type. The basic idea is that we will use the bound the user
    /// provided, if they provided one, and otherwise search the supertypes of trait bounds
    /// for region bounds. It may be that we can derive no bound at all, in which case
    /// we return `None`.
    fn compute_object_lifetime_bound(
        &self,
        span: Span,
        existential_predicates: &'tcx ty::List<ty::PolyExistentialPredicate<'tcx>>,
    ) -> Option<ty::Region<'tcx>> // if None, use the default
    {
        let tcx = self.tcx();

        debug!("compute_opt_region_bound(existential_predicates={:?})", existential_predicates);

        // No explicit region bound specified. Therefore, examine trait
        // bounds and see if we can derive region bounds from those.
        let derived_region_bounds = object_region_bounds(tcx, existential_predicates);

        // If there are no derived region bounds, then report back that we
        // can find no region bound. The caller will use the default.
        if derived_region_bounds.is_empty() {
            return None;
        }

        // If any of the derived region bounds are 'static, that is always
        // the best choice.
        if derived_region_bounds.iter().any(|r| r.is_static()) {
            return Some(tcx.lifetimes.re_static);
        }

        // Determine whether there is exactly one unique region in the set
        // of derived region bounds. If so, use that. Otherwise, report an
        // error.
        let r = derived_region_bounds[0];
        if derived_region_bounds[1..].iter().any(|r1| r != *r1) {
            tcx.sess.emit_err(AmbiguousLifetimeBound { span });
        }
        Some(r)
    }
}
