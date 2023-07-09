#![doc(html_root_url = "https://doc.rust-lang.org/nightly/nightly-rustc/")]
#![feature(associated_type_defaults)]
#![feature(rustc_private)]
#![feature(try_blocks)]
#![feature(let_chains)]
#![recursion_limit = "256"]
#![deny(rustc::untranslatable_diagnostic)]
#![deny(rustc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate tracing;

mod errors;

use rustc_ast::MacroDef;
use rustc_attr as attr;
use rustc_data_structures::fx::FxHashSet;
use rustc_data_structures::intern::Interned;
use rustc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use rustc_fluent_macro::fluent_messages;
use rustc_hir as hir;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, LocalDefId, CRATE_DEF_ID};
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{AssocItemKind, ForeignItemKind, HirIdSet, ItemId, Node, PatKind};
use rustc_middle::bug;
use rustc_middle::hir::nested_filter;
use rustc_middle::middle::privacy::{EffectiveVisibilities, EffectiveVisibility, Level};
use rustc_middle::query::Providers;
use rustc_middle::span_bug;
use rustc_middle::ty::subst::InternalSubsts;
use rustc_middle::ty::{self, Const, GenericParamDefKind};
use rustc_middle::ty::{TraitRef, Ty, TyCtxt, TypeSuperVisitable, TypeVisitable, TypeVisitor};
use rustc_session::lint;
use rustc_span::hygiene::Transparency;
use rustc_span::symbol::{kw, sym, Ident};
use rustc_span::Span;

use std::marker::PhantomData;
use std::ops::ControlFlow;
use std::{fmt, mem};

use errors::{
    FieldIsPrivate, FieldIsPrivateLabel, FromPrivateDependencyInPublicInterface, InPublicInterface,
    InPublicInterfaceTraits, ItemIsPrivate, PrivateInPublicLint, PrivateInterfacesOrBoundsLint,
    ReportEffectiveVisibility, UnnameableTypesLint, UnnamedItemIsPrivate,
};

fluent_messages! { "../messages.ftl" }

////////////////////////////////////////////////////////////////////////////////
/// Generic infrastructure used to implement specific visitors below.
////////////////////////////////////////////////////////////////////////////////

struct LazyDefPathStr<'tcx> {
    def_id: DefId,
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> fmt::Display for LazyDefPathStr<'tcx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tcx.def_path_str(self.def_id))
    }
}

/// Implemented to visit all `DefId`s in a type.
/// Visiting `DefId`s is useful because visibilities and reachabilities are attached to them.
/// The idea is to visit "all components of a type", as documented in
/// <https://github.com/rust-lang/rfcs/blob/master/text/2145-type-privacy.md#how-to-determine-visibility-of-a-type>.
/// The default type visitor (`TypeVisitor`) does most of the job, but it has some shortcomings.
/// First, it doesn't have overridable `fn visit_trait_ref`, so we have to catch trait `DefId`s
/// manually. Second, it doesn't visit some type components like signatures of fn types, or traits
/// in `impl Trait`, see individual comments in `DefIdVisitorSkeleton::visit_ty`.
trait DefIdVisitor<'tcx> {
    type BreakTy = ();
    const SHALLOW: bool = false;
    const SKIP_ASSOC_TYS: bool = false;

    fn tcx(&self) -> TyCtxt<'tcx>;
    fn visit_def_id(
        &mut self,
        def_id: DefId,
        kind: &str,
        descr: &dyn fmt::Display,
    ) -> ControlFlow<Self::BreakTy>;

    /// Not overridden, but used to actually visit types and traits.
    fn skeleton(&mut self) -> DefIdVisitorSkeleton<'_, 'tcx, Self> {
        DefIdVisitorSkeleton {
            def_id_visitor: self,
            visited_opaque_tys: Default::default(),
            dummy: Default::default(),
        }
    }
    fn visit(
        &mut self,
        ty_fragment: impl TypeVisitable<TyCtxt<'tcx>>,
    ) -> ControlFlow<Self::BreakTy> {
        ty_fragment.visit_with(&mut self.skeleton())
    }
    fn visit_trait(&mut self, trait_ref: TraitRef<'tcx>) -> ControlFlow<Self::BreakTy> {
        self.skeleton().visit_trait(trait_ref)
    }
    fn visit_projection_ty(&mut self, projection: ty::AliasTy<'tcx>) -> ControlFlow<Self::BreakTy> {
        self.skeleton().visit_projection_ty(projection)
    }
    fn visit_predicates(
        &mut self,
        predicates: ty::GenericPredicates<'tcx>,
    ) -> ControlFlow<Self::BreakTy> {
        self.skeleton().visit_clauses(predicates.predicates)
    }
    fn visit_clauses(
        &mut self,
        clauses: &[(ty::Clause<'tcx>, Span)],
    ) -> ControlFlow<Self::BreakTy> {
        self.skeleton().visit_clauses(clauses)
    }
}

struct DefIdVisitorSkeleton<'v, 'tcx, V: ?Sized> {
    def_id_visitor: &'v mut V,
    visited_opaque_tys: FxHashSet<DefId>,
    dummy: PhantomData<TyCtxt<'tcx>>,
}

impl<'tcx, V> DefIdVisitorSkeleton<'_, 'tcx, V>
where
    V: DefIdVisitor<'tcx> + ?Sized,
{
    fn visit_trait(&mut self, trait_ref: TraitRef<'tcx>) -> ControlFlow<V::BreakTy> {
        let TraitRef { def_id, substs, .. } = trait_ref;
        self.def_id_visitor.visit_def_id(def_id, "trait", &trait_ref.print_only_trait_path())?;
        if V::SHALLOW { ControlFlow::Continue(()) } else { substs.visit_with(self) }
    }

    fn visit_projection_ty(&mut self, projection: ty::AliasTy<'tcx>) -> ControlFlow<V::BreakTy> {
        let tcx = self.def_id_visitor.tcx();
        let (trait_ref, assoc_substs) = if tcx.def_kind(projection.def_id)
            != DefKind::ImplTraitPlaceholder
        {
            projection.trait_ref_and_own_substs(tcx)
        } else {
            // HACK(RPITIT): Remove this when RPITITs are lowered to regular assoc tys
            let def_id = tcx.impl_trait_in_trait_parent_fn(projection.def_id);
            let trait_generics = tcx.generics_of(def_id);
            (
                ty::TraitRef::new(tcx, def_id, projection.substs.truncate_to(tcx, trait_generics)),
                &projection.substs[trait_generics.count()..],
            )
        };
        self.visit_trait(trait_ref)?;
        if V::SHALLOW {
            ControlFlow::Continue(())
        } else {
            assoc_substs.iter().try_for_each(|subst| subst.visit_with(self))
        }
    }

    fn visit_clause(&mut self, clause: ty::Clause<'tcx>) -> ControlFlow<V::BreakTy> {
        match clause.kind().skip_binder() {
            ty::ClauseKind::Trait(ty::TraitPredicate { trait_ref, constness: _, polarity: _ }) => {
                self.visit_trait(trait_ref)
            }
            ty::ClauseKind::Projection(ty::ProjectionPredicate { projection_ty, term }) => {
                term.visit_with(self)?;
                self.visit_projection_ty(projection_ty)
            }
            ty::ClauseKind::TypeOutlives(ty::OutlivesPredicate(ty, _region)) => ty.visit_with(self),
            ty::ClauseKind::RegionOutlives(..) => ControlFlow::Continue(()),
            ty::ClauseKind::ConstArgHasType(ct, ty) => {
                ct.visit_with(self)?;
                ty.visit_with(self)
            }
            ty::ClauseKind::ConstEvaluatable(ct) => ct.visit_with(self),
            ty::ClauseKind::WellFormed(arg) => arg.visit_with(self),
        }
    }

    fn visit_clauses(&mut self, clauses: &[(ty::Clause<'tcx>, Span)]) -> ControlFlow<V::BreakTy> {
        clauses.into_iter().try_for_each(|&(clause, _span)| self.visit_clause(clause))
    }
}

impl<'tcx, V> TypeVisitor<TyCtxt<'tcx>> for DefIdVisitorSkeleton<'_, 'tcx, V>
where
    V: DefIdVisitor<'tcx> + ?Sized,
{
    type BreakTy = V::BreakTy;

    fn visit_ty(&mut self, ty: Ty<'tcx>) -> ControlFlow<V::BreakTy> {
        let tcx = self.def_id_visitor.tcx();
        // InternalSubsts are not visited here because they are visited below
        // in `super_visit_with`.
        match *ty.kind() {
            ty::Adt(ty::AdtDef(Interned(&ty::AdtDefData { did: def_id, .. }, _)), ..)
            | ty::Foreign(def_id)
            | ty::FnDef(def_id, ..)
            | ty::Closure(def_id, ..)
            | ty::Generator(def_id, ..) => {
                self.def_id_visitor.visit_def_id(def_id, "type", &ty)?;
                if V::SHALLOW {
                    return ControlFlow::Continue(());
                }
                // Default type visitor doesn't visit signatures of fn types.
                // Something like `fn() -> Priv {my_func}` is considered a private type even if
                // `my_func` is public, so we need to visit signatures.
                if let ty::FnDef(..) = ty.kind() {
                    // FIXME: this should probably use `substs` from `FnDef`
                    tcx.fn_sig(def_id).subst_identity().visit_with(self)?;
                }
                // Inherent static methods don't have self type in substs.
                // Something like `fn() {my_method}` type of the method
                // `impl Pub<Priv> { pub fn my_method() {} }` is considered a private type,
                // so we need to visit the self type additionally.
                if let Some(assoc_item) = tcx.opt_associated_item(def_id) {
                    if let Some(impl_def_id) = assoc_item.impl_container(tcx) {
                        tcx.type_of(impl_def_id).subst_identity().visit_with(self)?;
                    }
                }
            }
            ty::Alias(ty::Weak, alias) => {
                self.def_id_visitor.visit_def_id(alias.def_id, "type alias", &ty);
            }
            ty::Alias(ty::Projection, proj) => {
                if V::SKIP_ASSOC_TYS {
                    // Visitors searching for minimal visibility/reachability want to
                    // conservatively approximate associated types like `<Type as Trait>::Alias`
                    // as visible/reachable even if both `Type` and `Trait` are private.
                    // Ideally, associated types should be substituted in the same way as
                    // free type aliases, but this isn't done yet.
                    return ControlFlow::Continue(());
                }
                // This will also visit substs if necessary, so we don't need to recurse.
                return self.visit_projection_ty(proj);
            }
            ty::Alias(ty::Inherent, data) => {
                if V::SKIP_ASSOC_TYS {
                    // Visitors searching for minimal visibility/reachability want to
                    // conservatively approximate associated types like `Type::Alias`
                    // as visible/reachable even if `Type` is private.
                    // Ideally, associated types should be substituted in the same way as
                    // free type aliases, but this isn't done yet.
                    return ControlFlow::Continue(());
                }

                self.def_id_visitor.visit_def_id(
                    data.def_id,
                    "associated type",
                    &LazyDefPathStr { def_id: data.def_id, tcx },
                )?;

                // This will also visit substs if necessary, so we don't need to recurse.
                return if V::SHALLOW {
                    ControlFlow::Continue(())
                } else {
                    data.substs.iter().try_for_each(|subst| subst.visit_with(self))
                };
            }
            ty::Dynamic(predicates, ..) => {
                // All traits in the list are considered the "primary" part of the type
                // and are visited by shallow visitors.
                for predicate in predicates {
                    let trait_ref = match predicate.skip_binder() {
                        ty::ExistentialPredicate::Trait(trait_ref) => trait_ref,
                        ty::ExistentialPredicate::Projection(proj) => proj.trait_ref(tcx),
                        ty::ExistentialPredicate::AutoTrait(def_id) => {
                            ty::ExistentialTraitRef { def_id, substs: InternalSubsts::empty() }
                        }
                    };
                    let ty::ExistentialTraitRef { def_id, substs: _ } = trait_ref;
                    self.def_id_visitor.visit_def_id(def_id, "trait", &trait_ref)?;
                }
            }
            ty::Alias(ty::Opaque, ty::AliasTy { def_id, .. }) => {
                // Skip repeated `Opaque`s to avoid infinite recursion.
                if self.visited_opaque_tys.insert(def_id) {
                    // The intent is to treat `impl Trait1 + Trait2` identically to
                    // `dyn Trait1 + Trait2`. Therefore we ignore def-id of the opaque type itself
                    // (it either has no visibility, or its visibility is insignificant, like
                    // visibilities of type aliases) and recurse into bounds instead to go
                    // through the trait list (default type visitor doesn't visit those traits).
                    // All traits in the list are considered the "primary" part of the type
                    // and are visited by shallow visitors.
                    self.visit_clauses(tcx.explicit_item_bounds(def_id).skip_binder())?;
                }
            }
            // These types don't have their own def-ids (but may have subcomponents
            // with def-ids that should be visited recursively).
            ty::Bool
            | ty::Char
            | ty::Int(..)
            | ty::Uint(..)
            | ty::Float(..)
            | ty::Str
            | ty::Never
            | ty::Array(..)
            | ty::Slice(..)
            | ty::Tuple(..)
            | ty::RawPtr(..)
            | ty::Ref(..)
            | ty::FnPtr(..)
            | ty::Param(..)
            | ty::Bound(..)
            | ty::Error(_)
            | ty::GeneratorWitness(..)
            | ty::GeneratorWitnessMIR(..) => {}
            ty::Placeholder(..) | ty::Infer(..) => {
                bug!("unexpected type: {:?}", ty)
            }
        }

        if V::SHALLOW { ControlFlow::Continue(()) } else { ty.super_visit_with(self) }
    }

    fn visit_const(&mut self, c: Const<'tcx>) -> ControlFlow<Self::BreakTy> {
        let tcx = self.def_id_visitor.tcx();
        tcx.expand_abstract_consts(c).super_visit_with(self)
    }
}

fn min(vis1: ty::Visibility, vis2: ty::Visibility, tcx: TyCtxt<'_>) -> ty::Visibility {
    if vis1.is_at_least(vis2, tcx) { vis2 } else { vis1 }
}

////////////////////////////////////////////////////////////////////////////////
/// Visitor used to determine impl visibility and reachability.
////////////////////////////////////////////////////////////////////////////////

struct FindMin<'a, 'tcx, VL: VisibilityLike, const SHALLOW: bool> {
    tcx: TyCtxt<'tcx>,
    effective_visibilities: &'a EffectiveVisibilities,
    min: VL,
}

impl<'a, 'tcx, VL: VisibilityLike, const SHALLOW: bool> DefIdVisitor<'tcx>
    for FindMin<'a, 'tcx, VL, SHALLOW>
{
    const SHALLOW: bool = SHALLOW;
    const SKIP_ASSOC_TYS: bool = true;
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
    fn visit_def_id(
        &mut self,
        def_id: DefId,
        _kind: &str,
        _descr: &dyn fmt::Display,
    ) -> ControlFlow<Self::BreakTy> {
        if let Some(def_id) = def_id.as_local() {
            self.min = VL::new_min(self, def_id);
        }
        ControlFlow::Continue(())
    }
}

trait VisibilityLike: Sized {
    const MAX: Self;
    fn new_min<const SHALLOW: bool>(
        find: &FindMin<'_, '_, Self, SHALLOW>,
        def_id: LocalDefId,
    ) -> Self;

    // Returns an over-approximation (`SKIP_ASSOC_TYS` = true) of visibility due to
    // associated types for which we can't determine visibility precisely.
    fn of_impl<const SHALLOW: bool>(
        def_id: LocalDefId,
        tcx: TyCtxt<'_>,
        effective_visibilities: &EffectiveVisibilities,
    ) -> Self {
        let mut find = FindMin::<_, SHALLOW> { tcx, effective_visibilities, min: Self::MAX };
        find.visit(tcx.type_of(def_id).subst_identity());
        if let Some(trait_ref) = tcx.impl_trait_ref(def_id) {
            find.visit_trait(trait_ref.subst_identity());
        }
        find.min
    }
}
impl VisibilityLike for ty::Visibility {
    const MAX: Self = ty::Visibility::Public;
    fn new_min<const SHALLOW: bool>(
        find: &FindMin<'_, '_, Self, SHALLOW>,
        def_id: LocalDefId,
    ) -> Self {
        min(find.tcx.local_visibility(def_id), find.min, find.tcx)
    }
}

impl VisibilityLike for EffectiveVisibility {
    const MAX: Self = EffectiveVisibility::from_vis(ty::Visibility::Public);
    fn new_min<const SHALLOW: bool>(
        find: &FindMin<'_, '_, Self, SHALLOW>,
        def_id: LocalDefId,
    ) -> Self {
        let effective_vis =
            find.effective_visibilities.effective_vis(def_id).copied().unwrap_or_else(|| {
                let private_vis =
                    ty::Visibility::Restricted(find.tcx.parent_module_from_def_id(def_id));
                EffectiveVisibility::from_vis(private_vis)
            });

        effective_vis.min(find.min, find.tcx)
    }
}

////////////////////////////////////////////////////////////////////////////////
/// The embargo visitor, used to determine the exports of the AST.
////////////////////////////////////////////////////////////////////////////////

struct EmbargoVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,

    /// Effective visibilities for reachable nodes.
    effective_visibilities: EffectiveVisibilities,
    /// A set of pairs corresponding to modules, where the first module is
    /// reachable via a macro that's defined in the second module. This cannot
    /// be represented as reachable because it can't handle the following case:
    ///
    /// pub mod n {                         // Should be `Public`
    ///     pub(crate) mod p {              // Should *not* be accessible
    ///         pub fn f() -> i32 { 12 }    // Must be `Reachable`
    ///     }
    /// }
    /// pub macro m() {
    ///     n::p::f()
    /// }
    macro_reachable: FxHashSet<(LocalDefId, LocalDefId)>,
    /// Preliminary pass for marking all underlying types of `impl Trait`s as reachable.
    impl_trait_pass: bool,
    /// Has something changed in the level map?
    changed: bool,
}

struct ReachEverythingInTheInterfaceVisitor<'a, 'tcx> {
    effective_vis: EffectiveVisibility,
    item_def_id: LocalDefId,
    ev: &'a mut EmbargoVisitor<'tcx>,
    level: Level,
}

impl<'tcx> EmbargoVisitor<'tcx> {
    fn get(&self, def_id: LocalDefId) -> Option<EffectiveVisibility> {
        self.effective_visibilities.effective_vis(def_id).copied()
    }

    // Updates node effective visibility.
    fn update(
        &mut self,
        def_id: LocalDefId,
        inherited_effective_vis: EffectiveVisibility,
        level: Level,
    ) {
        let nominal_vis = self.tcx.local_visibility(def_id);
        self.update_eff_vis(def_id, inherited_effective_vis, Some(nominal_vis), level);
    }

    fn update_eff_vis(
        &mut self,
        def_id: LocalDefId,
        inherited_effective_vis: EffectiveVisibility,
        max_vis: Option<ty::Visibility>,
        level: Level,
    ) {
        let private_vis = ty::Visibility::Restricted(self.tcx.parent_module_from_def_id(def_id));
        if max_vis != Some(private_vis) {
            self.changed |= self.effective_visibilities.update(
                def_id,
                max_vis,
                || private_vis,
                inherited_effective_vis,
                level,
                self.tcx,
            );
        }
    }

    fn reach(
        &mut self,
        def_id: LocalDefId,
        effective_vis: EffectiveVisibility,
    ) -> ReachEverythingInTheInterfaceVisitor<'_, 'tcx> {
        ReachEverythingInTheInterfaceVisitor {
            effective_vis,
            item_def_id: def_id,
            ev: self,
            level: Level::Reachable,
        }
    }

    fn reach_through_impl_trait(
        &mut self,
        def_id: LocalDefId,
        effective_vis: EffectiveVisibility,
    ) -> ReachEverythingInTheInterfaceVisitor<'_, 'tcx> {
        ReachEverythingInTheInterfaceVisitor {
            effective_vis,
            item_def_id: def_id,
            ev: self,
            level: Level::ReachableThroughImplTrait,
        }
    }

    // We have to make sure that the items that macros might reference
    // are reachable, since they might be exported transitively.
    fn update_reachability_from_macro(
        &mut self,
        local_def_id: LocalDefId,
        md: &MacroDef,
        macro_ev: EffectiveVisibility,
    ) {
        // Non-opaque macros cannot make other items more accessible than they already are.
        let hir_id = self.tcx.hir().local_def_id_to_hir_id(local_def_id);
        let attrs = self.tcx.hir().attrs(hir_id);
        if attr::find_transparency(attrs, md.macro_rules).0 != Transparency::Opaque {
            return;
        }

        let macro_module_def_id = self.tcx.local_parent(local_def_id);
        if self.tcx.opt_def_kind(macro_module_def_id) != Some(DefKind::Mod) {
            // The macro's parent doesn't correspond to a `mod`, return early (#63164, #65252).
            return;
        }

        if self.effective_visibilities.public_at_level(local_def_id).is_none() {
            return;
        }

        // Since we are starting from an externally visible module,
        // all the parents in the loop below are also guaranteed to be modules.
        let mut module_def_id = macro_module_def_id;
        loop {
            let changed_reachability =
                self.update_macro_reachable(module_def_id, macro_module_def_id, macro_ev);
            if changed_reachability || module_def_id == CRATE_DEF_ID {
                break;
            }
            module_def_id = self.tcx.local_parent(module_def_id);
        }
    }

    /// Updates the item as being reachable through a macro defined in the given
    /// module. Returns `true` if the level has changed.
    fn update_macro_reachable(
        &mut self,
        module_def_id: LocalDefId,
        defining_mod: LocalDefId,
        macro_ev: EffectiveVisibility,
    ) -> bool {
        if self.macro_reachable.insert((module_def_id, defining_mod)) {
            self.update_macro_reachable_mod(module_def_id, defining_mod, macro_ev);
            true
        } else {
            false
        }
    }

    fn update_macro_reachable_mod(
        &mut self,
        module_def_id: LocalDefId,
        defining_mod: LocalDefId,
        macro_ev: EffectiveVisibility,
    ) {
        let module = self.tcx.hir().get_module(module_def_id).0;
        for item_id in module.item_ids {
            let def_kind = self.tcx.def_kind(item_id.owner_id);
            let vis = self.tcx.local_visibility(item_id.owner_id.def_id);
            self.update_macro_reachable_def(
                item_id.owner_id.def_id,
                def_kind,
                vis,
                defining_mod,
                macro_ev,
            );
        }
        for child in self.tcx.module_children_local(module_def_id) {
            // FIXME: Use module children for the logic above too.
            if !child.reexport_chain.is_empty()
                && child.vis.is_accessible_from(defining_mod, self.tcx)
                && let Res::Def(def_kind, def_id) = child.res
                && let Some(def_id) = def_id.as_local() {
                let vis = self.tcx.local_visibility(def_id);
                self.update_macro_reachable_def(def_id, def_kind, vis, defining_mod, macro_ev);
            }
        }
    }

    fn update_macro_reachable_def(
        &mut self,
        def_id: LocalDefId,
        def_kind: DefKind,
        vis: ty::Visibility,
        module: LocalDefId,
        macro_ev: EffectiveVisibility,
    ) {
        self.update(def_id, macro_ev, Level::Reachable);
        match def_kind {
            // No type privacy, so can be directly marked as reachable.
            DefKind::Const | DefKind::Static(_) | DefKind::TraitAlias | DefKind::TyAlias => {
                if vis.is_accessible_from(module, self.tcx) {
                    self.update(def_id, macro_ev, Level::Reachable);
                }
            }

            // Hygiene isn't really implemented for `macro_rules!` macros at the
            // moment. Accordingly, marking them as reachable is unwise. `macro` macros
            // have normal hygiene, so we can treat them like other items without type
            // privacy and mark them reachable.
            DefKind::Macro(_) => {
                let item = self.tcx.hir().expect_item(def_id);
                if let hir::ItemKind::Macro(MacroDef { macro_rules: false, .. }, _) = item.kind {
                    if vis.is_accessible_from(module, self.tcx) {
                        self.update(def_id, macro_ev, Level::Reachable);
                    }
                }
            }

            // We can't use a module name as the final segment of a path, except
            // in use statements. Since re-export checking doesn't consider
            // hygiene these don't need to be marked reachable. The contents of
            // the module, however may be reachable.
            DefKind::Mod => {
                if vis.is_accessible_from(module, self.tcx) {
                    self.update_macro_reachable(def_id, module, macro_ev);
                }
            }

            DefKind::Struct | DefKind::Union => {
                // While structs and unions have type privacy, their fields do not.
                let item = self.tcx.hir().expect_item(def_id);
                if let hir::ItemKind::Struct(ref struct_def, _)
                | hir::ItemKind::Union(ref struct_def, _) = item.kind
                {
                    for field in struct_def.fields() {
                        let field_vis = self.tcx.local_visibility(field.def_id);
                        if field_vis.is_accessible_from(module, self.tcx) {
                            self.reach(field.def_id, macro_ev).ty();
                        }
                    }
                } else {
                    bug!("item {:?} with DefKind {:?}", item, def_kind);
                }
            }

            // These have type privacy, so are not reachable unless they're
            // public, or are not namespaced at all.
            DefKind::AssocConst
            | DefKind::AssocTy
            | DefKind::ConstParam
            | DefKind::Ctor(_, _)
            | DefKind::Enum
            | DefKind::ForeignTy
            | DefKind::Fn
            | DefKind::OpaqueTy
            | DefKind::ImplTraitPlaceholder
            | DefKind::AssocFn
            | DefKind::Trait
            | DefKind::TyParam
            | DefKind::Variant
            | DefKind::LifetimeParam
            | DefKind::ExternCrate
            | DefKind::Use
            | DefKind::ForeignMod
            | DefKind::AnonConst
            | DefKind::InlineConst
            | DefKind::Field
            | DefKind::GlobalAsm
            | DefKind::Impl { .. }
            | DefKind::Closure
            | DefKind::Generator => (),
        }
    }
}

impl<'tcx> Visitor<'tcx> for EmbargoVisitor<'tcx> {
    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        if self.impl_trait_pass
            && let hir::ItemKind::OpaqueTy(ref opaque) = item.kind
            && !opaque.in_trait {
            // FIXME: This is some serious pessimization intended to workaround deficiencies
            // in the reachability pass (`middle/reachable.rs`). Types are marked as link-time
            // reachable if they are returned via `impl Trait`, even from private functions.
            let pub_ev = EffectiveVisibility::from_vis(ty::Visibility::Public);
            self.reach_through_impl_trait(item.owner_id.def_id, pub_ev)
                .generics()
                .predicates()
                .ty();
            return;
        }

        // Update levels of nested things and mark all items
        // in interfaces of reachable items as reachable.
        let item_ev = self.get(item.owner_id.def_id);
        match item.kind {
            // The interface is empty, and no nested items.
            hir::ItemKind::Use(..)
            | hir::ItemKind::ExternCrate(..)
            | hir::ItemKind::GlobalAsm(..) => {}
            // The interface is empty, and all nested items are processed by `visit_item`.
            hir::ItemKind::Mod(..) | hir::ItemKind::OpaqueTy(..) => {}
            hir::ItemKind::Macro(ref macro_def, _) => {
                if let Some(item_ev) = item_ev {
                    self.update_reachability_from_macro(item.owner_id.def_id, macro_def, item_ev);
                }
            }
            hir::ItemKind::Const(..)
            | hir::ItemKind::Static(..)
            | hir::ItemKind::Fn(..)
            | hir::ItemKind::TyAlias(..) => {
                if let Some(item_ev) = item_ev {
                    self.reach(item.owner_id.def_id, item_ev).generics().predicates().ty();
                }
            }
            hir::ItemKind::Trait(.., trait_item_refs) => {
                if let Some(item_ev) = item_ev {
                    self.reach(item.owner_id.def_id, item_ev).generics().predicates();

                    for trait_item_ref in trait_item_refs {
                        self.update(trait_item_ref.id.owner_id.def_id, item_ev, Level::Reachable);

                        let tcx = self.tcx;
                        let mut reach = self.reach(trait_item_ref.id.owner_id.def_id, item_ev);
                        reach.generics().predicates();

                        if trait_item_ref.kind == AssocItemKind::Type
                            && !tcx.defaultness(trait_item_ref.id.owner_id).has_value()
                        {
                            // No type to visit.
                        } else {
                            reach.ty();
                        }
                    }
                }
            }
            hir::ItemKind::TraitAlias(..) => {
                if let Some(item_ev) = item_ev {
                    self.reach(item.owner_id.def_id, item_ev).generics().predicates();
                }
            }
            hir::ItemKind::Impl(ref impl_) => {
                // Type inference is very smart sometimes. It can make an impl reachable even some
                // components of its type or trait are unreachable. E.g. methods of
                // `impl ReachableTrait<UnreachableTy> for ReachableTy<UnreachableTy> { ... }`
                // can be usable from other crates (#57264). So we skip substs when calculating
                // reachability and consider an impl reachable if its "shallow" type and trait are
                // reachable.
                //
                // The assumption we make here is that type-inference won't let you use an impl
                // without knowing both "shallow" version of its self type and "shallow" version of
                // its trait if it exists (which require reaching the `DefId`s in them).
                let item_ev = EffectiveVisibility::of_impl::<true>(
                    item.owner_id.def_id,
                    self.tcx,
                    &self.effective_visibilities,
                );

                self.update_eff_vis(item.owner_id.def_id, item_ev, None, Level::Direct);

                self.reach(item.owner_id.def_id, item_ev).generics().predicates().ty().trait_ref();

                for impl_item_ref in impl_.items {
                    let def_id = impl_item_ref.id.owner_id.def_id;
                    let max_vis =
                        impl_.of_trait.is_none().then(|| self.tcx.local_visibility(def_id));
                    self.update_eff_vis(def_id, item_ev, max_vis, Level::Direct);

                    if let Some(impl_item_ev) = self.get(def_id) {
                        self.reach(def_id, impl_item_ev).generics().predicates().ty();
                    }
                }
            }
            hir::ItemKind::Enum(ref def, _) => {
                if let Some(item_ev) = item_ev {
                    self.reach(item.owner_id.def_id, item_ev).generics().predicates();
                }
                for variant in def.variants {
                    if let Some(item_ev) = item_ev {
                        self.update(variant.def_id, item_ev, Level::Reachable);
                    }

                    if let Some(variant_ev) = self.get(variant.def_id) {
                        if let Some(ctor_def_id) = variant.data.ctor_def_id() {
                            self.update(ctor_def_id, variant_ev, Level::Reachable);
                        }
                        for field in variant.data.fields() {
                            self.update(field.def_id, variant_ev, Level::Reachable);
                            self.reach(field.def_id, variant_ev).ty();
                        }
                        // Corner case: if the variant is reachable, but its
                        // enum is not, make the enum reachable as well.
                        self.reach(item.owner_id.def_id, variant_ev).ty();
                    }
                    if let Some(ctor_def_id) = variant.data.ctor_def_id() {
                        if let Some(ctor_ev) = self.get(ctor_def_id) {
                            self.reach(item.owner_id.def_id, ctor_ev).ty();
                        }
                    }
                }
            }
            hir::ItemKind::ForeignMod { items, .. } => {
                for foreign_item in items {
                    if let Some(foreign_item_ev) = self.get(foreign_item.id.owner_id.def_id) {
                        self.reach(foreign_item.id.owner_id.def_id, foreign_item_ev)
                            .generics()
                            .predicates()
                            .ty();
                    }
                }
            }
            hir::ItemKind::Struct(ref struct_def, _) | hir::ItemKind::Union(ref struct_def, _) => {
                if let Some(item_ev) = item_ev {
                    self.reach(item.owner_id.def_id, item_ev).generics().predicates();
                    for field in struct_def.fields() {
                        self.update(field.def_id, item_ev, Level::Reachable);
                        if let Some(field_ev) = self.get(field.def_id) {
                            self.reach(field.def_id, field_ev).ty();
                        }
                    }
                }
                if let Some(ctor_def_id) = struct_def.ctor_def_id() {
                    if let Some(item_ev) = item_ev {
                        self.update(ctor_def_id, item_ev, Level::Reachable);
                    }
                    if let Some(ctor_ev) = self.get(ctor_def_id) {
                        self.reach(item.owner_id.def_id, ctor_ev).ty();
                    }
                }
            }
        }
    }
}

impl ReachEverythingInTheInterfaceVisitor<'_, '_> {
    fn generics(&mut self) -> &mut Self {
        for param in &self.ev.tcx.generics_of(self.item_def_id).params {
            match param.kind {
                GenericParamDefKind::Lifetime => {}
                GenericParamDefKind::Type { has_default, .. } => {
                    if has_default {
                        self.visit(self.ev.tcx.type_of(param.def_id).subst_identity());
                    }
                }
                GenericParamDefKind::Const { has_default } => {
                    self.visit(self.ev.tcx.type_of(param.def_id).subst_identity());
                    if has_default {
                        self.visit(self.ev.tcx.const_param_default(param.def_id).subst_identity());
                    }
                }
            }
        }
        self
    }

    fn predicates(&mut self) -> &mut Self {
        self.visit_predicates(self.ev.tcx.predicates_of(self.item_def_id));
        self
    }

    fn ty(&mut self) -> &mut Self {
        self.visit(self.ev.tcx.type_of(self.item_def_id).subst_identity());
        self
    }

    fn trait_ref(&mut self) -> &mut Self {
        if let Some(trait_ref) = self.ev.tcx.impl_trait_ref(self.item_def_id) {
            self.visit_trait(trait_ref.subst_identity());
        }
        self
    }
}

impl<'tcx> DefIdVisitor<'tcx> for ReachEverythingInTheInterfaceVisitor<'_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.ev.tcx
    }
    fn visit_def_id(
        &mut self,
        def_id: DefId,
        _kind: &str,
        _descr: &dyn fmt::Display,
    ) -> ControlFlow<Self::BreakTy> {
        if let Some(def_id) = def_id.as_local() {
            // All effective visibilities except `reachable_through_impl_trait` are limited to
            // nominal visibility. If any type or trait is leaked farther than that, it will
            // produce type privacy errors on any use, so we don't consider it leaked.
            let max_vis = (self.level != Level::ReachableThroughImplTrait)
                .then(|| self.ev.tcx.local_visibility(def_id));
            self.ev.update_eff_vis(def_id, self.effective_vis, max_vis, self.level);
        }
        ControlFlow::Continue(())
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Visitor, used for EffectiveVisibilities table checking
////////////////////////////////////////////////////////////////////////////////
pub struct TestReachabilityVisitor<'tcx, 'a> {
    tcx: TyCtxt<'tcx>,
    effective_visibilities: &'a EffectiveVisibilities,
}

fn vis_to_string<'tcx>(def_id: LocalDefId, vis: ty::Visibility, tcx: TyCtxt<'tcx>) -> String {
    match vis {
        ty::Visibility::Restricted(restricted_id) => {
            if restricted_id.is_top_level_module() {
                "pub(crate)".to_string()
            } else if restricted_id == tcx.parent_module_from_def_id(def_id) {
                "pub(self)".to_string()
            } else {
                format!("pub({})", tcx.item_name(restricted_id.to_def_id()))
            }
        }
        ty::Visibility::Public => "pub".to_string(),
    }
}

impl<'tcx, 'a> TestReachabilityVisitor<'tcx, 'a> {
    fn effective_visibility_diagnostic(&mut self, def_id: LocalDefId) {
        if self.tcx.has_attr(def_id, sym::rustc_effective_visibility) {
            let mut error_msg = String::new();
            let span = self.tcx.def_span(def_id.to_def_id());
            if let Some(effective_vis) = self.effective_visibilities.effective_vis(def_id) {
                for level in Level::all_levels() {
                    let vis_str = vis_to_string(def_id, *effective_vis.at_level(level), self.tcx);
                    if level != Level::Direct {
                        error_msg.push_str(", ");
                    }
                    error_msg.push_str(&format!("{level:?}: {vis_str}"));
                }
            } else {
                error_msg.push_str("not in the table");
            }
            self.tcx.sess.emit_err(ReportEffectiveVisibility { span, descr: error_msg });
        }
    }
}

impl<'tcx, 'a> Visitor<'tcx> for TestReachabilityVisitor<'tcx, 'a> {
    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        self.effective_visibility_diagnostic(item.owner_id.def_id);

        match item.kind {
            hir::ItemKind::Enum(ref def, _) => {
                for variant in def.variants.iter() {
                    self.effective_visibility_diagnostic(variant.def_id);
                    if let Some(ctor_def_id) = variant.data.ctor_def_id() {
                        self.effective_visibility_diagnostic(ctor_def_id);
                    }
                    for field in variant.data.fields() {
                        self.effective_visibility_diagnostic(field.def_id);
                    }
                }
            }
            hir::ItemKind::Struct(ref def, _) | hir::ItemKind::Union(ref def, _) => {
                if let Some(ctor_def_id) = def.ctor_def_id() {
                    self.effective_visibility_diagnostic(ctor_def_id);
                }
                for field in def.fields() {
                    self.effective_visibility_diagnostic(field.def_id);
                }
            }
            _ => {}
        }
    }

    fn visit_trait_item(&mut self, item: &'tcx hir::TraitItem<'tcx>) {
        self.effective_visibility_diagnostic(item.owner_id.def_id);
    }
    fn visit_impl_item(&mut self, item: &'tcx hir::ImplItem<'tcx>) {
        self.effective_visibility_diagnostic(item.owner_id.def_id);
    }
    fn visit_foreign_item(&mut self, item: &'tcx hir::ForeignItem<'tcx>) {
        self.effective_visibility_diagnostic(item.owner_id.def_id);
    }
}

//////////////////////////////////////////////////////////////////////////////////////
/// Name privacy visitor, checks privacy and reports violations.
/// Most of name privacy checks are performed during the main resolution phase,
/// or later in type checking when field accesses and associated items are resolved.
/// This pass performs remaining checks for fields in struct expressions and patterns.
//////////////////////////////////////////////////////////////////////////////////////

struct NamePrivacyVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    maybe_typeck_results: Option<&'tcx ty::TypeckResults<'tcx>>,
    current_item: LocalDefId,
}

impl<'tcx> NamePrivacyVisitor<'tcx> {
    /// Gets the type-checking results for the current body.
    /// As this will ICE if called outside bodies, only call when working with
    /// `Expr` or `Pat` nodes (they are guaranteed to be found only in bodies).
    #[track_caller]
    fn typeck_results(&self) -> &'tcx ty::TypeckResults<'tcx> {
        self.maybe_typeck_results
            .expect("`NamePrivacyVisitor::typeck_results` called outside of body")
    }

    // Checks that a field in a struct constructor (expression or pattern) is accessible.
    fn check_field(
        &mut self,
        use_ctxt: Span,        // syntax context of the field name at the use site
        span: Span,            // span of the field pattern, e.g., `x: 0`
        def: ty::AdtDef<'tcx>, // definition of the struct or enum
        field: &'tcx ty::FieldDef,
        in_update_syntax: bool,
    ) {
        if def.is_enum() {
            return;
        }

        // definition of the field
        let ident = Ident::new(kw::Empty, use_ctxt);
        let hir_id = self.tcx.hir().local_def_id_to_hir_id(self.current_item);
        let def_id = self.tcx.adjust_ident_and_get_scope(ident, def.did(), hir_id).1;
        if !field.vis.is_accessible_from(def_id, self.tcx) {
            self.tcx.sess.emit_err(FieldIsPrivate {
                span,
                field_name: field.name,
                variant_descr: def.variant_descr(),
                def_path_str: self.tcx.def_path_str(def.did()),
                label: if in_update_syntax {
                    FieldIsPrivateLabel::IsUpdateSyntax { span, field_name: field.name }
                } else {
                    FieldIsPrivateLabel::Other { span }
                },
            });
        }
    }
}

impl<'tcx> Visitor<'tcx> for NamePrivacyVisitor<'tcx> {
    type NestedFilter = nested_filter::All;

    /// We want to visit items in the context of their containing
    /// module and so forth, so supply a crate for doing a deep walk.
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_mod(&mut self, _m: &'tcx hir::Mod<'tcx>, _s: Span, _n: hir::HirId) {
        // Don't visit nested modules, since we run a separate visitor walk
        // for each module in `effective_visibilities`
    }

    fn visit_nested_body(&mut self, body: hir::BodyId) {
        let old_maybe_typeck_results =
            self.maybe_typeck_results.replace(self.tcx.typeck_body(body));
        let body = self.tcx.hir().body(body);
        self.visit_body(body);
        self.maybe_typeck_results = old_maybe_typeck_results;
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        let orig_current_item = mem::replace(&mut self.current_item, item.owner_id.def_id);
        intravisit::walk_item(self, item);
        self.current_item = orig_current_item;
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if let hir::ExprKind::Struct(qpath, fields, ref base) = expr.kind {
            let res = self.typeck_results().qpath_res(qpath, expr.hir_id);
            let adt = self.typeck_results().expr_ty(expr).ty_adt_def().unwrap();
            let variant = adt.variant_of_res(res);
            if let Some(base) = *base {
                // If the expression uses FRU we need to make sure all the unmentioned fields
                // are checked for privacy (RFC 736). Rather than computing the set of
                // unmentioned fields, just check them all.
                for (vf_index, variant_field) in variant.fields.iter_enumerated() {
                    let field = fields
                        .iter()
                        .find(|f| self.typeck_results().field_index(f.hir_id) == vf_index);
                    let (use_ctxt, span) = match field {
                        Some(field) => (field.ident.span, field.span),
                        None => (base.span, base.span),
                    };
                    self.check_field(use_ctxt, span, adt, variant_field, true);
                }
            } else {
                for field in fields {
                    let use_ctxt = field.ident.span;
                    let index = self.typeck_results().field_index(field.hir_id);
                    self.check_field(use_ctxt, field.span, adt, &variant.fields[index], false);
                }
            }
        }

        intravisit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &'tcx hir::Pat<'tcx>) {
        if let PatKind::Struct(ref qpath, fields, _) = pat.kind {
            let res = self.typeck_results().qpath_res(qpath, pat.hir_id);
            let adt = self.typeck_results().pat_ty(pat).ty_adt_def().unwrap();
            let variant = adt.variant_of_res(res);
            for field in fields {
                let use_ctxt = field.ident.span;
                let index = self.typeck_results().field_index(field.hir_id);
                self.check_field(use_ctxt, field.span, adt, &variant.fields[index], false);
            }
        }

        intravisit::walk_pat(self, pat);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////
/// Type privacy visitor, checks types for privacy and reports violations.
/// Both explicitly written types and inferred types of expressions and patterns are checked.
/// Checks are performed on "semantic" types regardless of names and their hygiene.
////////////////////////////////////////////////////////////////////////////////////////////

struct TypePrivacyVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    maybe_typeck_results: Option<&'tcx ty::TypeckResults<'tcx>>,
    current_item: LocalDefId,
    span: Span,
}

impl<'tcx> TypePrivacyVisitor<'tcx> {
    /// Gets the type-checking results for the current body.
    /// As this will ICE if called outside bodies, only call when working with
    /// `Expr` or `Pat` nodes (they are guaranteed to be found only in bodies).
    #[track_caller]
    fn typeck_results(&self) -> &'tcx ty::TypeckResults<'tcx> {
        self.maybe_typeck_results
            .expect("`TypePrivacyVisitor::typeck_results` called outside of body")
    }

    fn item_is_accessible(&self, did: DefId) -> bool {
        self.tcx.visibility(did).is_accessible_from(self.current_item, self.tcx)
    }

    // Take node-id of an expression or pattern and check its type for privacy.
    fn check_expr_pat_type(&mut self, id: hir::HirId, span: Span) -> bool {
        self.span = span;
        let typeck_results = self.typeck_results();
        let result: ControlFlow<()> = try {
            self.visit(typeck_results.node_type(id))?;
            self.visit(typeck_results.node_substs(id))?;
            if let Some(adjustments) = typeck_results.adjustments().get(id) {
                adjustments.iter().try_for_each(|adjustment| self.visit(adjustment.target))?;
            }
        };
        result.is_break()
    }

    fn check_def_id(&mut self, def_id: DefId, kind: &str, descr: &dyn fmt::Display) -> bool {
        let is_error = !self.item_is_accessible(def_id);
        if is_error {
            self.tcx.sess.emit_err(ItemIsPrivate { span: self.span, kind, descr: descr.into() });
        }
        is_error
    }
}

impl<'tcx> Visitor<'tcx> for TypePrivacyVisitor<'tcx> {
    type NestedFilter = nested_filter::All;

    /// We want to visit items in the context of their containing
    /// module and so forth, so supply a crate for doing a deep walk.
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_mod(&mut self, _m: &'tcx hir::Mod<'tcx>, _s: Span, _n: hir::HirId) {
        // Don't visit nested modules, since we run a separate visitor walk
        // for each module in `effective_visibilities`
    }

    fn visit_nested_body(&mut self, body: hir::BodyId) {
        let old_maybe_typeck_results =
            self.maybe_typeck_results.replace(self.tcx.typeck_body(body));
        let body = self.tcx.hir().body(body);
        self.visit_body(body);
        self.maybe_typeck_results = old_maybe_typeck_results;
    }

    fn visit_generic_arg(&mut self, generic_arg: &'tcx hir::GenericArg<'tcx>) {
        match generic_arg {
            hir::GenericArg::Type(t) => self.visit_ty(t),
            hir::GenericArg::Infer(inf) => self.visit_infer(inf),
            hir::GenericArg::Lifetime(_) | hir::GenericArg::Const(_) => {}
        }
    }

    fn visit_ty(&mut self, hir_ty: &'tcx hir::Ty<'tcx>) {
        self.span = hir_ty.span;
        if let Some(typeck_results) = self.maybe_typeck_results {
            // Types in bodies.
            if self.visit(typeck_results.node_type(hir_ty.hir_id)).is_break() {
                return;
            }
        } else {
            // Types in signatures.
            // FIXME: This is very ineffective. Ideally each HIR type should be converted
            // into a semantic type only once and the result should be cached somehow.
            if self.visit(rustc_hir_analysis::hir_ty_to_ty(self.tcx, hir_ty)).is_break() {
                return;
            }
        }

        intravisit::walk_ty(self, hir_ty);
    }

    fn visit_infer(&mut self, inf: &'tcx hir::InferArg) {
        self.span = inf.span;
        if let Some(typeck_results) = self.maybe_typeck_results {
            if let Some(ty) = typeck_results.node_type_opt(inf.hir_id) {
                if self.visit(ty).is_break() {
                    return;
                }
            } else {
                // We don't do anything for const infers here.
            }
        } else {
            bug!("visit_infer without typeck_results");
        }
        intravisit::walk_inf(self, inf);
    }

    fn visit_trait_ref(&mut self, trait_ref: &'tcx hir::TraitRef<'tcx>) {
        self.span = trait_ref.path.span;
        if self.maybe_typeck_results.is_none() {
            // Avoid calling `hir_trait_to_predicates` in bodies, it will ICE.
            // The traits' privacy in bodies is already checked as a part of trait object types.
            let bounds = rustc_hir_analysis::hir_trait_to_predicates(
                self.tcx,
                trait_ref,
                // NOTE: This isn't really right, but the actual type doesn't matter here. It's
                // just required by `ty::TraitRef`.
                self.tcx.types.never,
            );

            for (clause, _) in bounds.clauses() {
                match clause.kind().skip_binder() {
                    ty::ClauseKind::Trait(trait_predicate) => {
                        if self.visit_trait(trait_predicate.trait_ref).is_break() {
                            return;
                        }
                    }
                    ty::ClauseKind::Projection(proj_predicate) => {
                        let term = self.visit(proj_predicate.term);
                        if term.is_break()
                            || self.visit_projection_ty(proj_predicate.projection_ty).is_break()
                        {
                            return;
                        }
                    }
                    _ => {}
                }
            }
        }

        intravisit::walk_trait_ref(self, trait_ref);
    }

    // Check types of expressions
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.check_expr_pat_type(expr.hir_id, expr.span) {
            // Do not check nested expressions if the error already happened.
            return;
        }
        match expr.kind {
            hir::ExprKind::Assign(_, rhs, _) | hir::ExprKind::Match(rhs, ..) => {
                // Do not report duplicate errors for `x = y` and `match x { ... }`.
                if self.check_expr_pat_type(rhs.hir_id, rhs.span) {
                    return;
                }
            }
            hir::ExprKind::MethodCall(segment, ..) => {
                // Method calls have to be checked specially.
                self.span = segment.ident.span;
                if let Some(def_id) = self.typeck_results().type_dependent_def_id(expr.hir_id) {
                    if self.visit(self.tcx.type_of(def_id).subst_identity()).is_break() {
                        return;
                    }
                } else {
                    self.tcx
                        .sess
                        .delay_span_bug(expr.span, "no type-dependent def for method call");
                }
            }
            _ => {}
        }

        intravisit::walk_expr(self, expr);
    }

    // Prohibit access to associated items with insufficient nominal visibility.
    //
    // Additionally, until better reachability analysis for macros 2.0 is available,
    // we prohibit access to private statics from other crates, this allows to give
    // more code internal visibility at link time. (Access to private functions
    // is already prohibited by type privacy for function types.)
    fn visit_qpath(&mut self, qpath: &'tcx hir::QPath<'tcx>, id: hir::HirId, span: Span) {
        let def = match qpath {
            hir::QPath::Resolved(_, path) => match path.res {
                Res::Def(kind, def_id) => Some((kind, def_id)),
                _ => None,
            },
            hir::QPath::TypeRelative(..) | hir::QPath::LangItem(..) => self
                .maybe_typeck_results
                .and_then(|typeck_results| typeck_results.type_dependent_def(id)),
        };
        let def = def.filter(|(kind, _)| {
            matches!(
                kind,
                DefKind::AssocFn | DefKind::AssocConst | DefKind::AssocTy | DefKind::Static(_)
            )
        });
        if let Some((kind, def_id)) = def {
            let is_local_static =
                if let DefKind::Static(_) = kind { def_id.is_local() } else { false };
            if !self.item_is_accessible(def_id) && !is_local_static {
                let name = match *qpath {
                    hir::QPath::LangItem(it, ..) => {
                        self.tcx.lang_items().get(it).map(|did| self.tcx.def_path_str(did))
                    }
                    hir::QPath::Resolved(_, path) => Some(self.tcx.def_path_str(path.res.def_id())),
                    hir::QPath::TypeRelative(_, segment) => Some(segment.ident.to_string()),
                };
                let kind = self.tcx.def_descr(def_id);
                let sess = self.tcx.sess;
                let _ = match name {
                    Some(name) => {
                        sess.emit_err(ItemIsPrivate { span, kind, descr: (&name).into() })
                    }
                    None => sess.emit_err(UnnamedItemIsPrivate { span, kind }),
                };
                return;
            }
        }

        intravisit::walk_qpath(self, qpath, id);
    }

    // Check types of patterns.
    fn visit_pat(&mut self, pattern: &'tcx hir::Pat<'tcx>) {
        if self.check_expr_pat_type(pattern.hir_id, pattern.span) {
            // Do not check nested patterns if the error already happened.
            return;
        }

        intravisit::walk_pat(self, pattern);
    }

    fn visit_local(&mut self, local: &'tcx hir::Local<'tcx>) {
        if let Some(init) = local.init {
            if self.check_expr_pat_type(init.hir_id, init.span) {
                // Do not report duplicate errors for `let x = y`.
                return;
            }
        }

        intravisit::walk_local(self, local);
    }

    // Check types in item interfaces.
    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        let orig_current_item = mem::replace(&mut self.current_item, item.owner_id.def_id);
        let old_maybe_typeck_results = self.maybe_typeck_results.take();
        intravisit::walk_item(self, item);
        self.maybe_typeck_results = old_maybe_typeck_results;
        self.current_item = orig_current_item;
    }
}

impl<'tcx> DefIdVisitor<'tcx> for TypePrivacyVisitor<'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
    fn visit_def_id(
        &mut self,
        def_id: DefId,
        kind: &str,
        descr: &dyn fmt::Display,
    ) -> ControlFlow<Self::BreakTy> {
        if self.check_def_id(def_id, kind, descr) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Obsolete visitors for checking for private items in public interfaces.
/// These visitors are supposed to be kept in frozen state and produce an
/// "old error node set". For backward compatibility the new visitor reports
/// warnings instead of hard errors when the erroneous node is not in this old set.
///////////////////////////////////////////////////////////////////////////////

struct ObsoleteVisiblePrivateTypesVisitor<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    effective_visibilities: &'a EffectiveVisibilities,
    in_variant: bool,
    // Set of errors produced by this obsolete visitor.
    old_error_set: HirIdSet,
}

struct ObsoleteCheckTypeForPrivatenessVisitor<'a, 'b, 'tcx> {
    inner: &'a ObsoleteVisiblePrivateTypesVisitor<'b, 'tcx>,
    /// Whether the type refers to private types.
    contains_private: bool,
    /// Whether we've recurred at all (i.e., if we're pointing at the
    /// first type on which `visit_ty` was called).
    at_outer_type: bool,
    /// Whether that first type is a public path.
    outer_type_is_public_path: bool,
}

impl<'a, 'tcx> ObsoleteVisiblePrivateTypesVisitor<'a, 'tcx> {
    fn path_is_private_type(&self, path: &hir::Path<'_>) -> bool {
        let did = match path.res {
            Res::PrimTy(..) | Res::SelfTyParam { .. } | Res::SelfTyAlias { .. } | Res::Err => {
                return false;
            }
            res => res.def_id(),
        };

        // A path can only be private if:
        // it's in this crate...
        if let Some(did) = did.as_local() {
            // .. and it corresponds to a private type in the AST (this returns
            // `None` for type parameters).
            match self.tcx.hir().find(self.tcx.hir().local_def_id_to_hir_id(did)) {
                Some(Node::Item(_)) => !self.tcx.visibility(did).is_public(),
                Some(_) | None => false,
            }
        } else {
            false
        }
    }

    fn trait_is_public(&self, trait_id: LocalDefId) -> bool {
        // FIXME: this would preferably be using `exported_items`, but all
        // traits are exported currently (see `EmbargoVisitor.exported_trait`).
        self.effective_visibilities.is_directly_public(trait_id)
    }

    fn check_generic_bound(&mut self, bound: &hir::GenericBound<'_>) {
        if let hir::GenericBound::Trait(ref trait_ref, _) = *bound {
            if self.path_is_private_type(trait_ref.trait_ref.path) {
                self.old_error_set.insert(trait_ref.trait_ref.hir_ref_id);
            }
        }
    }

    fn item_is_public(&self, def_id: LocalDefId) -> bool {
        self.effective_visibilities.is_reachable(def_id) || self.tcx.visibility(def_id).is_public()
    }
}

impl<'a, 'b, 'tcx, 'v> Visitor<'v> for ObsoleteCheckTypeForPrivatenessVisitor<'a, 'b, 'tcx> {
    fn visit_generic_arg(&mut self, generic_arg: &'v hir::GenericArg<'v>) {
        match generic_arg {
            hir::GenericArg::Type(t) => self.visit_ty(t),
            hir::GenericArg::Infer(inf) => self.visit_ty(&inf.to_ty()),
            hir::GenericArg::Lifetime(_) | hir::GenericArg::Const(_) => {}
        }
    }

    fn visit_ty(&mut self, ty: &hir::Ty<'_>) {
        if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = ty.kind {
            if self.inner.path_is_private_type(path) {
                self.contains_private = true;
                // Found what we're looking for, so let's stop working.
                return;
            }
        }
        if let hir::TyKind::Path(_) = ty.kind {
            if self.at_outer_type {
                self.outer_type_is_public_path = true;
            }
        }
        self.at_outer_type = false;
        intravisit::walk_ty(self, ty)
    }

    // Don't want to recurse into `[, .. expr]`.
    fn visit_expr(&mut self, _: &hir::Expr<'_>) {}
}

impl<'a, 'tcx> Visitor<'tcx> for ObsoleteVisiblePrivateTypesVisitor<'a, 'tcx> {
    type NestedFilter = nested_filter::All;

    /// We want to visit items in the context of their containing
    /// module and so forth, so supply a crate for doing a deep walk.
    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        match item.kind {
            // Contents of a private mod can be re-exported, so we need
            // to check internals.
            hir::ItemKind::Mod(_) => {}

            // An `extern {}` doesn't introduce a new privacy
            // namespace (the contents have their own privacies).
            hir::ItemKind::ForeignMod { .. } => {}

            hir::ItemKind::Trait(.., bounds, _) => {
                if !self.trait_is_public(item.owner_id.def_id) {
                    return;
                }

                for bound in bounds.iter() {
                    self.check_generic_bound(bound)
                }
            }

            // Impls need some special handling to try to offer useful
            // error messages without (too many) false positives
            // (i.e., we could just return here to not check them at
            // all, or some worse estimation of whether an impl is
            // publicly visible).
            hir::ItemKind::Impl(ref impl_) => {
                // `impl [... for] Private` is never visible.
                let self_contains_private;
                // `impl [... for] Public<...>`, but not `impl [... for]
                // Vec<Public>` or `(Public,)`, etc.
                let self_is_public_path;

                // Check the properties of the `Self` type:
                {
                    let mut visitor = ObsoleteCheckTypeForPrivatenessVisitor {
                        inner: self,
                        contains_private: false,
                        at_outer_type: true,
                        outer_type_is_public_path: false,
                    };
                    visitor.visit_ty(impl_.self_ty);
                    self_contains_private = visitor.contains_private;
                    self_is_public_path = visitor.outer_type_is_public_path;
                }

                // Miscellaneous info about the impl:

                // `true` iff this is `impl Private for ...`.
                let not_private_trait = impl_.of_trait.as_ref().map_or(
                    true, // no trait counts as public trait
                    |tr| {
                        if let Some(def_id) = tr.path.res.def_id().as_local() {
                            self.trait_is_public(def_id)
                        } else {
                            true // external traits must be public
                        }
                    },
                );

                // `true` iff this is a trait impl or at least one method is public.
                //
                // `impl Public { $( fn ...() {} )* }` is not visible.
                //
                // This is required over just using the methods' privacy
                // directly because we might have `impl<T: Foo<Private>> ...`,
                // and we shouldn't warn about the generics if all the methods
                // are private (because `T` won't be visible externally).
                let trait_or_some_public_method = impl_.of_trait.is_some()
                    || impl_.items.iter().any(|impl_item_ref| {
                        let impl_item = self.tcx.hir().impl_item(impl_item_ref.id);
                        match impl_item.kind {
                            hir::ImplItemKind::Const(..) | hir::ImplItemKind::Fn(..) => self
                                .effective_visibilities
                                .is_reachable(impl_item_ref.id.owner_id.def_id),
                            hir::ImplItemKind::Type(_) => false,
                        }
                    });

                if !self_contains_private && not_private_trait && trait_or_some_public_method {
                    intravisit::walk_generics(self, &impl_.generics);

                    match impl_.of_trait {
                        None => {
                            for impl_item_ref in impl_.items {
                                // This is where we choose whether to walk down
                                // further into the impl to check its items. We
                                // should only walk into public items so that we
                                // don't erroneously report errors for private
                                // types in private items.
                                let impl_item = self.tcx.hir().impl_item(impl_item_ref.id);
                                match impl_item.kind {
                                    hir::ImplItemKind::Const(..) | hir::ImplItemKind::Fn(..)
                                        if self.item_is_public(impl_item.owner_id.def_id) =>
                                    {
                                        intravisit::walk_impl_item(self, impl_item)
                                    }
                                    hir::ImplItemKind::Type(..) => {
                                        intravisit::walk_impl_item(self, impl_item)
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Some(ref tr) => {
                            // Any private types in a trait impl fall into three
                            // categories.
                            // 1. mentioned in the trait definition
                            // 2. mentioned in the type params/generics
                            // 3. mentioned in the associated types of the impl
                            //
                            // Those in 1. can only occur if the trait is in
                            // this crate and will have been warned about on the
                            // trait definition (there's no need to warn twice
                            // so we don't check the methods).
                            //
                            // Those in 2. are warned via walk_generics and this
                            // call here.
                            intravisit::walk_path(self, tr.path);

                            // Those in 3. are warned with this call.
                            for impl_item_ref in impl_.items {
                                let impl_item = self.tcx.hir().impl_item(impl_item_ref.id);
                                if let hir::ImplItemKind::Type(ty) = impl_item.kind {
                                    self.visit_ty(ty);
                                }
                            }
                        }
                    }
                } else if impl_.of_trait.is_none() && self_is_public_path {
                    // `impl Public<Private> { ... }`. Any public static
                    // methods will be visible as `Public::foo`.
                    let mut found_pub_static = false;
                    for impl_item_ref in impl_.items {
                        if self
                            .effective_visibilities
                            .is_reachable(impl_item_ref.id.owner_id.def_id)
                            || self.tcx.visibility(impl_item_ref.id.owner_id).is_public()
                        {
                            let impl_item = self.tcx.hir().impl_item(impl_item_ref.id);
                            match impl_item_ref.kind {
                                AssocItemKind::Const => {
                                    found_pub_static = true;
                                    intravisit::walk_impl_item(self, impl_item);
                                }
                                AssocItemKind::Fn { has_self: false } => {
                                    found_pub_static = true;
                                    intravisit::walk_impl_item(self, impl_item);
                                }
                                _ => {}
                            }
                        }
                    }
                    if found_pub_static {
                        intravisit::walk_generics(self, &impl_.generics)
                    }
                }
                return;
            }

            // `type ... = ...;` can contain private types, because
            // we're introducing a new name.
            hir::ItemKind::TyAlias(..) => return,

            // Not at all public, so we don't care.
            _ if !self.item_is_public(item.owner_id.def_id) => {
                return;
            }

            _ => {}
        }

        // We've carefully constructed it so that if we're here, then
        // any `visit_ty`'s will be called on things that are in
        // public signatures, i.e., things that we're interested in for
        // this visitor.
        intravisit::walk_item(self, item);
    }

    fn visit_generics(&mut self, generics: &'tcx hir::Generics<'tcx>) {
        for predicate in generics.predicates {
            match predicate {
                hir::WherePredicate::BoundPredicate(bound_pred) => {
                    for bound in bound_pred.bounds.iter() {
                        self.check_generic_bound(bound)
                    }
                }
                hir::WherePredicate::RegionPredicate(_) => {}
                hir::WherePredicate::EqPredicate(eq_pred) => {
                    self.visit_ty(eq_pred.rhs_ty);
                }
            }
        }
    }

    fn visit_foreign_item(&mut self, item: &'tcx hir::ForeignItem<'tcx>) {
        if self.effective_visibilities.is_reachable(item.owner_id.def_id) {
            intravisit::walk_foreign_item(self, item)
        }
    }

    fn visit_ty(&mut self, t: &'tcx hir::Ty<'tcx>) {
        if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = t.kind {
            if self.path_is_private_type(path) {
                self.old_error_set.insert(t.hir_id);
            }
        }
        intravisit::walk_ty(self, t)
    }

    fn visit_variant(&mut self, v: &'tcx hir::Variant<'tcx>) {
        if self.effective_visibilities.is_reachable(v.def_id) {
            self.in_variant = true;
            intravisit::walk_variant(self, v);
            self.in_variant = false;
        }
    }

    fn visit_field_def(&mut self, s: &'tcx hir::FieldDef<'tcx>) {
        let vis = self.tcx.visibility(s.def_id);
        if vis.is_public() || self.in_variant {
            intravisit::walk_field_def(self, s);
        }
    }

    // We don't need to introspect into these at all: an
    // expression/block context can't possibly contain exported things.
    // (Making them no-ops stops us from traversing the whole AST without
    // having to be super careful about our `walk_...` calls above.)
    fn visit_block(&mut self, _: &'tcx hir::Block<'tcx>) {}
    fn visit_expr(&mut self, _: &'tcx hir::Expr<'tcx>) {}
}

///////////////////////////////////////////////////////////////////////////////
/// SearchInterfaceForPrivateItemsVisitor traverses an item's interface and
/// finds any private components in it.
/// PrivateItemsInPublicInterfacesVisitor ensures there are no private types
/// and traits in public interfaces.
///////////////////////////////////////////////////////////////////////////////

struct SearchInterfaceForPrivateItemsVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    item_def_id: LocalDefId,
    /// The visitor checks that each component type is at least this visible.
    required_visibility: ty::Visibility,
    required_effective_vis: Option<EffectiveVisibility>,
    has_old_errors: bool,
    in_assoc_ty: bool,
    in_primary_interface: bool,
}

impl SearchInterfaceForPrivateItemsVisitor<'_> {
    fn generics(&mut self) -> &mut Self {
        self.in_primary_interface = true;
        for param in &self.tcx.generics_of(self.item_def_id).params {
            match param.kind {
                GenericParamDefKind::Lifetime => {}
                GenericParamDefKind::Type { has_default, .. } => {
                    if has_default {
                        self.visit(self.tcx.type_of(param.def_id).subst_identity());
                    }
                }
                // FIXME(generic_const_exprs): May want to look inside const here
                GenericParamDefKind::Const { .. } => {
                    self.visit(self.tcx.type_of(param.def_id).subst_identity());
                }
            }
        }
        self
    }

    fn predicates(&mut self) -> &mut Self {
        self.in_primary_interface = false;
        // N.B., we use `explicit_predicates_of` and not `predicates_of`
        // because we don't want to report privacy errors due to where
        // clauses that the compiler inferred. We only want to
        // consider the ones that the user wrote. This is important
        // for the inferred outlives rules; see
        // `tests/ui/rfc-2093-infer-outlives/privacy.rs`.
        self.visit_predicates(self.tcx.explicit_predicates_of(self.item_def_id));
        self
    }

    fn bounds(&mut self) -> &mut Self {
        self.in_primary_interface = false;
        self.visit_clauses(self.tcx.explicit_item_bounds(self.item_def_id).skip_binder());
        self
    }

    fn ty(&mut self) -> &mut Self {
        self.in_primary_interface = true;
        self.visit(self.tcx.type_of(self.item_def_id).subst_identity());
        self
    }

    fn check_def_id(&mut self, def_id: DefId, kind: &str, descr: &dyn fmt::Display) -> bool {
        if self.leaks_private_dep(def_id) {
            self.tcx.emit_spanned_lint(
                lint::builtin::EXPORTED_PRIVATE_DEPENDENCIES,
                self.tcx.hir().local_def_id_to_hir_id(self.item_def_id),
                self.tcx.def_span(self.item_def_id.to_def_id()),
                FromPrivateDependencyInPublicInterface {
                    kind,
                    descr: descr.into(),
                    krate: self.tcx.crate_name(def_id.krate),
                },
            );
        }

        let Some(local_def_id) = def_id.as_local() else {
            return false;
        };

        let vis = self.tcx.local_visibility(local_def_id);
        let hir_id = self.tcx.hir().local_def_id_to_hir_id(local_def_id);
        let span = self.tcx.def_span(self.item_def_id.to_def_id());
        let vis_span = self.tcx.def_span(def_id);
        if !vis.is_at_least(self.required_visibility, self.tcx) {
            let vis_descr = match vis {
                ty::Visibility::Public => "public",
                ty::Visibility::Restricted(vis_def_id) => {
                    if vis_def_id == self.tcx.parent_module(hir_id) {
                        "private"
                    } else if vis_def_id.is_top_level_module() {
                        "crate-private"
                    } else {
                        "restricted"
                    }
                }
            };

            if self.has_old_errors
                || self.in_assoc_ty
                || self.tcx.resolutions(()).has_pub_restricted
            {
                if kind == "trait" {
                    self.tcx.sess.emit_err(InPublicInterfaceTraits {
                        span,
                        vis_descr,
                        kind,
                        descr: descr.into(),
                        vis_span,
                    });
                } else {
                    self.tcx.sess.emit_err(InPublicInterface {
                        span,
                        vis_descr,
                        kind,
                        descr: descr.into(),
                        vis_span,
                    });
                }
            } else {
                self.tcx.emit_spanned_lint(
                    lint::builtin::PRIVATE_IN_PUBLIC,
                    hir_id,
                    span,
                    PrivateInPublicLint { vis_descr, kind, descr: descr.into() },
                );
            }
        }

        let Some(effective_vis) = self.required_effective_vis else {
            return false;
        };

        let reachable_at_vis = *effective_vis.at_level(Level::Reachable);

        if !vis.is_at_least(reachable_at_vis, self.tcx) {
            let lint = if self.in_primary_interface {
                lint::builtin::PRIVATE_INTERFACES
            } else {
                lint::builtin::PRIVATE_BOUNDS
            };
            self.tcx.emit_spanned_lint(
                lint,
                hir_id,
                span,
                PrivateInterfacesOrBoundsLint {
                    item_span: span,
                    item_kind: self.tcx.def_descr(self.item_def_id.to_def_id()),
                    item_descr: (&LazyDefPathStr {
                        def_id: self.item_def_id.to_def_id(),
                        tcx: self.tcx,
                    })
                        .into(),
                    item_vis_descr: &vis_to_string(self.item_def_id, reachable_at_vis, self.tcx),
                    ty_span: vis_span,
                    ty_kind: kind,
                    ty_descr: descr.into(),
                    ty_vis_descr: &vis_to_string(local_def_id, vis, self.tcx),
                },
            );
        }

        false
    }

    /// An item is 'leaked' from a private dependency if all
    /// of the following are true:
    /// 1. It's contained within a public type
    /// 2. It comes from a private crate
    fn leaks_private_dep(&self, item_id: DefId) -> bool {
        let ret = self.required_visibility.is_public() && self.tcx.is_private_dep(item_id.krate);

        debug!("leaks_private_dep(item_id={:?})={}", item_id, ret);
        ret
    }
}

impl<'tcx> DefIdVisitor<'tcx> for SearchInterfaceForPrivateItemsVisitor<'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
    fn visit_def_id(
        &mut self,
        def_id: DefId,
        kind: &str,
        descr: &dyn fmt::Display,
    ) -> ControlFlow<Self::BreakTy> {
        if self.check_def_id(def_id, kind, descr) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

struct PrivateItemsInPublicInterfacesChecker<'tcx, 'a> {
    tcx: TyCtxt<'tcx>,
    old_error_set_ancestry: HirIdSet,
    effective_visibilities: &'a EffectiveVisibilities,
}

impl<'tcx> PrivateItemsInPublicInterfacesChecker<'tcx, '_> {
    fn check(
        &self,
        def_id: LocalDefId,
        required_visibility: ty::Visibility,
        required_effective_vis: Option<EffectiveVisibility>,
    ) -> SearchInterfaceForPrivateItemsVisitor<'tcx> {
        SearchInterfaceForPrivateItemsVisitor {
            tcx: self.tcx,
            item_def_id: def_id,
            required_visibility,
            required_effective_vis,
            has_old_errors: self
                .old_error_set_ancestry
                .contains(&self.tcx.hir().local_def_id_to_hir_id(def_id)),
            in_assoc_ty: false,
            in_primary_interface: true,
        }
    }

    fn check_unnameable(&self, def_id: LocalDefId, effective_vis: Option<EffectiveVisibility>) {
        let Some(effective_vis) = effective_vis else {
            return;
        };

        let reexported_at_vis = effective_vis.at_level(Level::Reexported);
        let reachable_at_vis = effective_vis.at_level(Level::Reachable);

        if reexported_at_vis != reachable_at_vis {
            let hir_id = self.tcx.hir().local_def_id_to_hir_id(def_id);
            let span = self.tcx.def_span(def_id.to_def_id());
            self.tcx.emit_spanned_lint(
                lint::builtin::UNNAMEABLE_TYPES,
                hir_id,
                span,
                UnnameableTypesLint {
                    span,
                    kind: self.tcx.def_descr(def_id.to_def_id()),
                    descr: (&LazyDefPathStr { def_id: def_id.to_def_id(), tcx: self.tcx }).into(),
                    reachable_vis: &vis_to_string(def_id, *reachable_at_vis, self.tcx),
                    reexported_vis: &vis_to_string(def_id, *reexported_at_vis, self.tcx),
                },
            );
        }
    }

    fn check_assoc_item(
        &self,
        def_id: LocalDefId,
        assoc_item_kind: AssocItemKind,
        vis: ty::Visibility,
        effective_vis: Option<EffectiveVisibility>,
    ) {
        let mut check = self.check(def_id, vis, effective_vis);

        let (check_ty, is_assoc_ty) = match assoc_item_kind {
            AssocItemKind::Const | AssocItemKind::Fn { .. } => (true, false),
            AssocItemKind::Type => (self.tcx.defaultness(def_id).has_value(), true),
        };

        if is_assoc_ty {
            self.check_unnameable(def_id, self.get(def_id));
        }

        check.in_assoc_ty = is_assoc_ty;
        check.generics().predicates();
        if check_ty {
            check.ty();
        }
    }

    fn get(&self, def_id: LocalDefId) -> Option<EffectiveVisibility> {
        self.effective_visibilities.effective_vis(def_id).copied()
    }

    pub fn check_item(&mut self, id: ItemId) {
        let tcx = self.tcx;
        let def_id = id.owner_id.def_id;
        let item_visibility = tcx.local_visibility(def_id);
        let effective_vis = self.get(def_id);
        let def_kind = tcx.def_kind(def_id);

        match def_kind {
            DefKind::Const | DefKind::Static(_) | DefKind::Fn | DefKind::TyAlias => {
                if let DefKind::TyAlias = def_kind {
                    self.check_unnameable(def_id, effective_vis);
                }
                self.check(def_id, item_visibility, effective_vis).generics().predicates().ty();
            }
            DefKind::OpaqueTy => {
                // `ty()` for opaque types is the underlying type,
                // it's not a part of interface, so we skip it.
                self.check(def_id, item_visibility, effective_vis).generics().bounds();
            }
            DefKind::Trait => {
                let item = tcx.hir().item(id);
                if let hir::ItemKind::Trait(.., trait_item_refs) = item.kind {
                    self.check_unnameable(item.owner_id.def_id, effective_vis);

                    self.check(item.owner_id.def_id, item_visibility, effective_vis)
                        .generics()
                        .predicates();

                    for trait_item_ref in trait_item_refs {
                        self.check_assoc_item(
                            trait_item_ref.id.owner_id.def_id,
                            trait_item_ref.kind,
                            item_visibility,
                            effective_vis,
                        );

                        if let AssocItemKind::Type = trait_item_ref.kind {
                            self.check(
                                trait_item_ref.id.owner_id.def_id,
                                item_visibility,
                                effective_vis,
                            )
                            .bounds();
                        }
                    }
                }
            }
            DefKind::TraitAlias => {
                self.check(def_id, item_visibility, effective_vis).generics().predicates();
            }
            DefKind::Enum => {
                let item = tcx.hir().item(id);
                if let hir::ItemKind::Enum(ref def, _) = item.kind {
                    self.check_unnameable(item.owner_id.def_id, effective_vis);

                    self.check(item.owner_id.def_id, item_visibility, effective_vis)
                        .generics()
                        .predicates();

                    for variant in def.variants {
                        for field in variant.data.fields() {
                            self.check(field.def_id, item_visibility, effective_vis).ty();
                        }
                    }
                }
            }
            // Subitems of foreign modules have their own publicity.
            DefKind::ForeignMod => {
                let item = tcx.hir().item(id);
                if let hir::ItemKind::ForeignMod { items, .. } = item.kind {
                    for foreign_item in items {
                        let foreign_item = tcx.hir().foreign_item(foreign_item.id);

                        let ev = self.get(foreign_item.owner_id.def_id);
                        let vis = tcx.local_visibility(foreign_item.owner_id.def_id);

                        if let ForeignItemKind::Type = foreign_item.kind {
                            self.check_unnameable(foreign_item.owner_id.def_id, ev);
                        }

                        self.check(foreign_item.owner_id.def_id, vis, ev)
                            .generics()
                            .predicates()
                            .ty();
                    }
                }
            }
            // Subitems of structs and unions have their own publicity.
            DefKind::Struct | DefKind::Union => {
                let item = tcx.hir().item(id);
                if let hir::ItemKind::Struct(ref struct_def, _)
                | hir::ItemKind::Union(ref struct_def, _) = item.kind
                {
                    self.check_unnameable(item.owner_id.def_id, effective_vis);
                    self.check(item.owner_id.def_id, item_visibility, effective_vis)
                        .generics()
                        .predicates();

                    for field in struct_def.fields() {
                        let field_visibility = tcx.local_visibility(field.def_id);
                        let field_ev = self.get(field.def_id);

                        self.check(
                            field.def_id,
                            min(item_visibility, field_visibility, tcx),
                            field_ev,
                        )
                        .ty();
                    }
                }
            }
            // An inherent impl is public when its type is public
            // Subitems of inherent impls have their own publicity.
            // A trait impl is public when both its type and its trait are public
            // Subitems of trait impls have inherited publicity.
            DefKind::Impl { .. } => {
                let item = tcx.hir().item(id);
                if let hir::ItemKind::Impl(ref impl_) = item.kind {
                    let impl_vis = ty::Visibility::of_impl::<false>(
                        item.owner_id.def_id,
                        tcx,
                        &Default::default(),
                    );

                    // We are using the non-shallow version here, unlike when building the
                    // effective visisibilities table to avoid large number of false positives.
                    // For example in
                    //
                    // impl From<Priv> for Pub {
                    //     fn from(_: Priv) -> Pub {...}
                    // }
                    //
                    // lints shouldn't be emmited even if `from` effective visibility
                    // is larger than `Priv` nominal visibility and if `Priv` can leak
                    // in some scenarios due to type inference.
                    let impl_ev = EffectiveVisibility::of_impl::<false>(
                        item.owner_id.def_id,
                        tcx,
                        self.effective_visibilities,
                    );

                    // check that private components do not appear in the generics or predicates of inherent impls
                    // this check is intentionally NOT performed for impls of traits, per #90586
                    if impl_.of_trait.is_none() {
                        self.check(item.owner_id.def_id, impl_vis, Some(impl_ev))
                            .generics()
                            .predicates();
                    }
                    for impl_item_ref in impl_.items {
                        let impl_item_vis = if impl_.of_trait.is_none() {
                            min(
                                tcx.local_visibility(impl_item_ref.id.owner_id.def_id),
                                impl_vis,
                                tcx,
                            )
                        } else {
                            impl_vis
                        };

                        let impl_item_ev = if impl_.of_trait.is_none() {
                            self.get(impl_item_ref.id.owner_id.def_id)
                                .map(|ev| ev.min(impl_ev, self.tcx))
                        } else {
                            Some(impl_ev)
                        };

                        self.check_assoc_item(
                            impl_item_ref.id.owner_id.def_id,
                            impl_item_ref.kind,
                            impl_item_vis,
                            impl_item_ev,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn provide(providers: &mut Providers) {
    *providers = Providers {
        visibility,
        effective_visibilities,
        check_private_in_public,
        check_mod_privacy,
        ..*providers
    };
}

fn visibility(tcx: TyCtxt<'_>, def_id: LocalDefId) -> ty::Visibility<DefId> {
    local_visibility(tcx, def_id).to_def_id()
}

fn local_visibility(tcx: TyCtxt<'_>, def_id: LocalDefId) -> ty::Visibility {
    match tcx.resolutions(()).visibilities.get(&def_id) {
        Some(vis) => *vis,
        None => {
            let hir_id = tcx.hir().local_def_id_to_hir_id(def_id);
            match tcx.hir().get(hir_id) {
                // Unique types created for closures participate in type privacy checking.
                // They have visibilities inherited from the module they are defined in.
                Node::Expr(hir::Expr { kind: hir::ExprKind::Closure{..}, .. })
                // - AST lowering creates dummy `use` items which don't
                //   get their entries in the resolver's visibility table.
                // - AST lowering also creates opaque type items with inherited visibilities.
                //   Visibility on them should have no effect, but to avoid the visibility
                //   query failing on some items, we provide it for opaque types as well.
                | Node::Item(hir::Item {
                    kind: hir::ItemKind::Use(_, hir::UseKind::ListStem)
                        | hir::ItemKind::OpaqueTy(..),
                    ..
                }) => ty::Visibility::Restricted(tcx.parent_module(hir_id)),
                // Visibilities of trait impl items are inherited from their traits
                // and are not filled in resolve.
                Node::ImplItem(impl_item) => {
                    match tcx.hir().get_by_def_id(tcx.hir().get_parent_item(hir_id).def_id) {
                        Node::Item(hir::Item {
                            kind: hir::ItemKind::Impl(hir::Impl { of_trait: Some(tr), .. }),
                            ..
                        }) => tr.path.res.opt_def_id().map_or_else(
                            || {
                                tcx.sess.delay_span_bug(tr.path.span, "trait without a def-id");
                                ty::Visibility::Public
                            },
                            |def_id| tcx.visibility(def_id).expect_local(),
                        ),
                        _ => span_bug!(impl_item.span, "the parent is not a trait impl"),
                    }
                }
                _ => span_bug!(
                    tcx.def_span(def_id),
                    "visibility table unexpectedly missing a def-id: {:?}",
                    def_id,
                ),
            }
        }
    }
}

fn check_mod_privacy(tcx: TyCtxt<'_>, module_def_id: LocalDefId) {
    // Check privacy of names not checked in previous compilation stages.
    let mut visitor =
        NamePrivacyVisitor { tcx, maybe_typeck_results: None, current_item: module_def_id };
    let (module, span, hir_id) = tcx.hir().get_module(module_def_id);

    intravisit::walk_mod(&mut visitor, module, hir_id);

    // Check privacy of explicitly written types and traits as well as
    // inferred types of expressions and patterns.
    let mut visitor =
        TypePrivacyVisitor { tcx, maybe_typeck_results: None, current_item: module_def_id, span };
    intravisit::walk_mod(&mut visitor, module, hir_id);
}

fn effective_visibilities(tcx: TyCtxt<'_>, (): ()) -> &EffectiveVisibilities {
    // Build up a set of all exported items in the AST. This is a set of all
    // items which are reachable from external crates based on visibility.
    let mut visitor = EmbargoVisitor {
        tcx,
        effective_visibilities: tcx.resolutions(()).effective_visibilities.clone(),
        macro_reachable: Default::default(),
        // HACK(jynelson): trying to infer the type of `impl Trait` breaks `async-std` (and
        // `pub async fn` in general). Since rustdoc never needs to do codegen and doesn't
        // care about link-time reachability, keep them unreachable (issue #75100).
        impl_trait_pass: !tcx.sess.opts.actually_rustdoc,
        changed: false,
    };

    visitor.effective_visibilities.check_invariants(tcx);
    if visitor.impl_trait_pass {
        // Underlying types of `impl Trait`s are marked as reachable unconditionally,
        // so this pass doesn't need to be a part of the fixed point iteration below.
        tcx.hir().visit_all_item_likes_in_crate(&mut visitor);
        visitor.impl_trait_pass = false;
        visitor.changed = false;
    }

    loop {
        tcx.hir().visit_all_item_likes_in_crate(&mut visitor);
        if visitor.changed {
            visitor.changed = false;
        } else {
            break;
        }
    }
    visitor.effective_visibilities.check_invariants(tcx);

    let mut check_visitor =
        TestReachabilityVisitor { tcx, effective_visibilities: &visitor.effective_visibilities };
    check_visitor.effective_visibility_diagnostic(CRATE_DEF_ID);
    tcx.hir().visit_all_item_likes_in_crate(&mut check_visitor);

    tcx.arena.alloc(visitor.effective_visibilities)
}

fn check_private_in_public(tcx: TyCtxt<'_>, (): ()) {
    let effective_visibilities = tcx.effective_visibilities(());

    let mut visitor = ObsoleteVisiblePrivateTypesVisitor {
        tcx,
        effective_visibilities,
        in_variant: false,
        old_error_set: Default::default(),
    };
    tcx.hir().walk_toplevel_module(&mut visitor);

    let mut old_error_set_ancestry = HirIdSet::default();
    for mut id in visitor.old_error_set.iter().copied() {
        loop {
            if !old_error_set_ancestry.insert(id) {
                break;
            }
            let parent = tcx.hir().parent_id(id);
            if parent == id {
                break;
            }
            id = parent;
        }
    }

    // Check for private types and traits in public interfaces.
    let mut checker = PrivateItemsInPublicInterfacesChecker {
        tcx,
        old_error_set_ancestry,
        effective_visibilities,
    };

    for id in tcx.hir().items() {
        checker.check_item(id);
    }
}
