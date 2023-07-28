//! Defines the various compiler queries.
//!
//! For more information on the query system, see
//! ["Queries: demand-driven compilation"](https://rustc-dev-guide.rust-lang.org/query.html).
//! This chapter includes instructions for adding new queries.

#![allow(unused_parens)]

use crate::dep_graph;
use crate::dep_graph::DepKind;
use crate::infer::canonical::{self, Canonical};
use crate::lint::LintExpectation;
use crate::metadata::ModChild;
use crate::middle::codegen_fn_attrs::CodegenFnAttrs;
use crate::middle::debugger_visualizer::DebuggerVisualizerFile;
use crate::middle::exported_symbols::{ExportedSymbol, SymbolExportInfo};
use crate::middle::lib_features::LibFeatures;
use crate::middle::privacy::EffectiveVisibilities;
use crate::middle::resolve_bound_vars::{ObjectLifetimeDefault, ResolveBoundVars, ResolvedArg};
use crate::middle::stability::{self, DeprecationEntry};
use crate::mir;
use crate::mir::interpret::GlobalId;
use crate::mir::interpret::{
    ConstValue, EvalToAllocationRawResult, EvalToConstValueResult, EvalToValTreeResult,
};
use crate::mir::interpret::{LitToConstError, LitToConstInput};
use crate::mir::mono::CodegenUnit;
use crate::query::erase::{erase, restore, Erase};
use crate::query::plumbing::{query_ensure, query_get_at, DynamicQuery};
use crate::thir;
use crate::traits::query::{
    CanonicalPredicateGoal, CanonicalProjectionGoal, CanonicalTyGoal,
    CanonicalTypeOpAscribeUserTypeGoal, CanonicalTypeOpEqGoal, CanonicalTypeOpNormalizeGoal,
    CanonicalTypeOpProvePredicateGoal, CanonicalTypeOpSubtypeGoal, NoSolution,
};
use crate::traits::query::{
    DropckConstraint, DropckOutlivesResult, MethodAutoderefStepsResult, NormalizationResult,
    OutlivesBound,
};
use crate::traits::specialization_graph;
use crate::traits::{
    CodegenObligationError, EvaluationResult, ImplSource, ObjectSafetyViolation, ObligationCause,
    OverflowError, WellFormedLoc,
};
use crate::ty::fast_reject::SimplifiedType;
use crate::ty::layout::ValidityRequirement;
use crate::ty::util::AlwaysRequiresDrop;
use crate::ty::GeneratorDiagnosticData;
use crate::ty::TyCtxtFeed;
use crate::ty::{
    self, print::describe_as_module, CrateInherentImpls, ParamEnvAnd, Ty, TyCtxt,
    UnusedGenericParams,
};
use crate::ty::{GenericArg, GenericArgsRef};
use rustc_arena::TypedArena;
use rustc_ast as ast;
use rustc_ast::expand::{allocator::AllocatorKind, StrippedCfgItem};
use rustc_attr as attr;
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::{FxHashMap, FxIndexMap, FxIndexSet};
use rustc_data_structures::steal::Steal;
use rustc_data_structures::svh::Svh;
use rustc_data_structures::sync::Lrc;
use rustc_data_structures::sync::WorkerLocal;
use rustc_data_structures::unord::UnordSet;
use rustc_errors::ErrorGuaranteed;
use rustc_hir as hir;
use rustc_hir::def::{DefKind, DocLinkResMap};
use rustc_hir::def_id::{
    CrateNum, DefId, DefIdMap, DefIdSet, LocalDefId, LocalDefIdMap, LocalDefIdSet,
};
use rustc_hir::lang_items::{LangItem, LanguageItems};
use rustc_hir::{Crate, ItemLocalId, TraitCandidate};
use rustc_index::IndexVec;
use rustc_query_system::ich::StableHashingContext;
use rustc_query_system::query::{try_get_cached, CacheSelector, QueryCache, QueryMode, QueryState};
use rustc_session::config::{EntryFnType, OptLevel, OutputFilenames, SymbolManglingVersion};
use rustc_session::cstore::{CrateDepKind, CrateSource};
use rustc_session::cstore::{ExternCrate, ForeignModule, LinkagePreference, NativeLib};
use rustc_session::lint::LintExpectationId;
use rustc_session::Limits;
use rustc_span::def_id::LOCAL_CRATE;
use rustc_span::symbol::Symbol;
use rustc_span::{Span, DUMMY_SP};
use rustc_target::abi;
use rustc_target::spec::PanicStrategy;
use std::mem;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

pub mod erase;
mod keys;
pub use keys::{AsLocalKey, Key, LocalCrate};
pub mod on_disk_cache;
#[macro_use]
pub mod plumbing;
pub use plumbing::{IntoQueryParam, TyCtxtAt, TyCtxtEnsure, TyCtxtEnsureWithValue};

// Each of these queries corresponds to a function pointer field in the
// `Providers` struct for requesting a value of that type, and a method
// on `tcx: TyCtxt` (and `tcx.at(span)`) for doing that request in a way
// which memoizes and does dep-graph tracking, wrapping around the actual
// `Providers` that the driver creates (using several `rustc_*` crates).
//
// The result type of each query must implement `Clone`, and additionally
// `ty::query::values::Value`, which produces an appropriate placeholder
// (error) value if the query resulted in a query cycle.
// Queries marked with `fatal_cycle` do not need the latter implementation,
// as they will raise an fatal error on query cycles instead.
rustc_queries! {
    query trigger_delay_span_bug(key: DefId) -> () {
        desc { "triggering a delay span bug" }
    }

    query registered_tools(_: ()) -> &'tcx ty::RegisteredTools {
        arena_cache
        desc { "compute registered tools for crate" }
    }

    query early_lint_checks(_: ()) -> () {
        desc { "perform lints prior to macro expansion" }
    }

    query resolutions(_: ()) -> &'tcx ty::ResolverGlobalCtxt {
        feedable
        no_hash
        desc { "getting the resolver outputs" }
    }

    query resolver_for_lowering(_: ()) -> &'tcx Steal<(ty::ResolverAstLowering, Lrc<ast::Crate>)> {
        eval_always
        no_hash
        desc { "getting the resolver for lowering" }
    }

    /// Return the span for a definition.
    /// Contrary to `def_span` below, this query returns the full absolute span of the definition.
    /// This span is meant for dep-tracking rather than diagnostics. It should not be used outside
    /// of rustc_middle::hir::source_map.
    query source_span(key: LocalDefId) -> Span {
        // Accesses untracked data
        eval_always
        desc { "getting the source span" }
    }

    /// Represents crate as a whole (as distinct from the top-level crate module).
    /// If you call `hir_crate` (e.g., indirectly by calling `tcx.hir().krate()`),
    /// we will have to assume that any change means that you need to be recompiled.
    /// This is because the `hir_crate` query gives you access to all other items.
    /// To avoid this fate, do not call `tcx.hir().krate()`; instead,
    /// prefer wrappers like `tcx.visit_all_items_in_krate()`.
    query hir_crate(key: ()) -> &'tcx Crate<'tcx> {
        arena_cache
        eval_always
        desc { "getting the crate HIR" }
    }

    /// All items in the crate.
    query hir_crate_items(_: ()) -> &'tcx rustc_middle::hir::ModuleItems {
        arena_cache
        eval_always
        desc { "getting HIR crate items" }
    }

    /// The items in a module.
    ///
    /// This can be conveniently accessed by `tcx.hir().visit_item_likes_in_module`.
    /// Avoid calling this query directly.
    query hir_module_items(key: LocalDefId) -> &'tcx rustc_middle::hir::ModuleItems {
        arena_cache
        desc { |tcx| "getting HIR module items in `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { true }
    }

    /// Gives access to the HIR node for the HIR owner `key`.
    ///
    /// This can be conveniently accessed by methods on `tcx.hir()`.
    /// Avoid calling this query directly.
    query hir_owner(key: hir::OwnerId) -> Option<crate::hir::Owner<'tcx>> {
        desc { |tcx| "getting HIR owner of `{}`", tcx.def_path_str(key) }
    }

    /// Gives access to the HIR ID for the given `LocalDefId` owner `key` if any.
    ///
    /// Definitions that were generated with no HIR, would be fed to return `None`.
    query opt_local_def_id_to_hir_id(key: LocalDefId) -> Option<hir::HirId>{
        desc { |tcx| "getting HIR ID of `{}`", tcx.def_path_str(key) }
        feedable
    }

    /// Gives access to the HIR node's parent for the HIR owner `key`.
    ///
    /// This can be conveniently accessed by methods on `tcx.hir()`.
    /// Avoid calling this query directly.
    query hir_owner_parent(key: hir::OwnerId) -> hir::HirId {
        desc { |tcx| "getting HIR parent of `{}`", tcx.def_path_str(key) }
    }

    /// Gives access to the HIR nodes and bodies inside the HIR owner `key`.
    ///
    /// This can be conveniently accessed by methods on `tcx.hir()`.
    /// Avoid calling this query directly.
    query hir_owner_nodes(key: hir::OwnerId) -> hir::MaybeOwner<&'tcx hir::OwnerNodes<'tcx>> {
        desc { |tcx| "getting HIR owner items in `{}`", tcx.def_path_str(key) }
    }

    /// Gives access to the HIR attributes inside the HIR owner `key`.
    ///
    /// This can be conveniently accessed by methods on `tcx.hir()`.
    /// Avoid calling this query directly.
    query hir_attrs(key: hir::OwnerId) -> &'tcx hir::AttributeMap<'tcx> {
        desc { |tcx| "getting HIR owner attributes in `{}`", tcx.def_path_str(key) }
    }

    /// Given the def_id of a const-generic parameter, computes the associated default const
    /// parameter. e.g. `fn example<const N: usize=3>` called on `N` would return `3`.
    query const_param_default(param: DefId) -> ty::EarlyBinder<ty::Const<'tcx>> {
        desc { |tcx| "computing const default for a given parameter `{}`", tcx.def_path_str(param)  }
        cache_on_disk_if { param.is_local() }
        separate_provide_extern
    }

    /// Returns the [`Ty`][rustc_middle::ty::Ty] of the given [`DefId`]. If the [`DefId`] points
    /// to an alias, it will "skip" this alias to return the aliased type.
    ///
    /// [`DefId`]: rustc_hir::def_id::DefId
    query type_of(key: DefId) -> ty::EarlyBinder<Ty<'tcx>> {
        desc { |tcx|
            "{action} `{path}`",
            action = {
                use rustc_hir::def::DefKind;
                match tcx.def_kind(key) {
                    DefKind::TyAlias => "expanding type alias",
                    DefKind::TraitAlias => "expanding trait alias",
                    _ => "computing type of",
                }
            },
            path = tcx.def_path_str(key),
        }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
        feedable
    }

    query collect_return_position_impl_trait_in_trait_tys(key: DefId)
        -> Result<&'tcx FxHashMap<DefId, ty::EarlyBinder<Ty<'tcx>>>, ErrorGuaranteed>
    {
        desc { "comparing an impl and trait method signature, inferring any hidden `impl Trait` types in the process" }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    query is_type_alias_impl_trait(key: DefId) -> bool
    {
        desc { "determine whether the opaque is a type-alias impl trait" }
        separate_provide_extern
        feedable
    }

    query unsizing_params_for_adt(key: DefId) -> &'tcx rustc_index::bit_set::BitSet<u32>
    {
        arena_cache
        desc { |tcx|
            "determining what parameters of `{}` can participate in unsizing",
            tcx.def_path_str(key),
        }
    }

    query analysis(key: ()) -> Result<(), ErrorGuaranteed> {
        eval_always
        desc { "running analysis passes on this crate" }
    }

    /// This query checks the fulfillment of collected lint expectations.
    /// All lint emitting queries have to be done before this is executed
    /// to ensure that all expectations can be fulfilled.
    ///
    /// This is an extra query to enable other drivers (like rustdoc) to
    /// only execute a small subset of the `analysis` query, while allowing
    /// lints to be expected. In rustc, this query will be executed as part of
    /// the `analysis` query and doesn't have to be called a second time.
    ///
    /// Tools can additionally pass in a tool filter. That will restrict the
    /// expectations to only trigger for lints starting with the listed tool
    /// name. This is useful for cases were not all linting code from rustc
    /// was called. With the default `None` all registered lints will also
    /// be checked for expectation fulfillment.
    query check_expectations(key: Option<Symbol>) -> () {
        eval_always
        desc { "checking lint expectations (RFC 2383)" }
    }

    /// Maps from the `DefId` of an item (trait/struct/enum/fn) to its
    /// associated generics.
    query generics_of(key: DefId) -> &'tcx ty::Generics {
        desc { |tcx| "computing generics of `{}`", tcx.def_path_str(key) }
        arena_cache
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
        feedable
    }

    /// Maps from the `DefId` of an item (trait/struct/enum/fn) to the
    /// predicates (where-clauses) that must be proven true in order
    /// to reference it. This is almost always the "predicates query"
    /// that you want.
    ///
    /// `predicates_of` builds on `predicates_defined_on` -- in fact,
    /// it is almost always the same as that query, except for the
    /// case of traits. For traits, `predicates_of` contains
    /// an additional `Self: Trait<...>` predicate that users don't
    /// actually write. This reflects the fact that to invoke the
    /// trait (e.g., via `Default::default`) you must supply types
    /// that actually implement the trait. (However, this extra
    /// predicate gets in the way of some checks, which are intended
    /// to operate over only the actual where-clauses written by the
    /// user.)
    query predicates_of(key: DefId) -> ty::GenericPredicates<'tcx> {
        desc { |tcx| "computing predicates of `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
    }

    query opaque_types_defined_by(
        key: LocalDefId
    ) -> &'tcx [LocalDefId] {
        desc {
            |tcx| "computing the opaque types defined by `{}`",
            tcx.def_path_str(key.to_def_id())
        }
    }

    /// Returns the list of bounds that can be used for
    /// `SelectionCandidate::ProjectionCandidate(_)` and
    /// `ProjectionTyCandidate::TraitDef`.
    /// Specifically this is the bounds written on the trait's type
    /// definition, or those after the `impl` keyword
    ///
    /// ```ignore (incomplete)
    /// type X: Bound + 'lt
    /// //      ^^^^^^^^^^^
    /// impl Debug + Display
    /// //   ^^^^^^^^^^^^^^^
    /// ```
    ///
    /// `key` is the `DefId` of the associated type or opaque type.
    ///
    /// Bounds from the parent (e.g. with nested impl trait) are not included.
    query explicit_item_bounds(key: DefId) -> ty::EarlyBinder<&'tcx [(ty::Clause<'tcx>, Span)]> {
        desc { |tcx| "finding item bounds for `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
        feedable
    }

    /// Elaborated version of the predicates from `explicit_item_bounds`.
    ///
    /// For example:
    ///
    /// ```
    /// trait MyTrait {
    ///     type MyAType: Eq + ?Sized;
    /// }
    /// ```
    ///
    /// `explicit_item_bounds` returns `[<Self as MyTrait>::MyAType: Eq]`,
    /// and `item_bounds` returns
    /// ```text
    /// [
    ///     <Self as Trait>::MyAType: Eq,
    ///     <Self as Trait>::MyAType: PartialEq<<Self as Trait>::MyAType>
    /// ]
    /// ```
    ///
    /// Bounds from the parent (e.g. with nested impl trait) are not included.
    query item_bounds(key: DefId) -> ty::EarlyBinder<&'tcx ty::List<ty::Clause<'tcx>>> {
        desc { |tcx| "elaborating item bounds for `{}`", tcx.def_path_str(key) }
    }

    /// Look up all native libraries this crate depends on.
    /// These are assembled from the following places:
    /// - `extern` blocks (depending on their `link` attributes)
    /// - the `libs` (`-l`) option
    query native_libraries(_: CrateNum) -> &'tcx Vec<NativeLib> {
        arena_cache
        desc { "looking up the native libraries of a linked crate" }
        separate_provide_extern
    }

    query shallow_lint_levels_on(key: hir::OwnerId) -> &'tcx rustc_middle::lint::ShallowLintLevelMap {
        eval_always // fetches `resolutions`
        arena_cache
        desc { |tcx| "looking up lint levels for `{}`", tcx.def_path_str(key) }
    }

    query lint_expectations(_: ()) -> &'tcx Vec<(LintExpectationId, LintExpectation)> {
        arena_cache
        desc { "computing `#[expect]`ed lints in this crate" }
    }

    query parent_module_from_def_id(key: LocalDefId) -> LocalDefId {
        eval_always
        desc { |tcx| "getting the parent module of `{}`", tcx.def_path_str(key) }
    }

    query expn_that_defined(key: DefId) -> rustc_span::ExpnId {
        desc { |tcx| "getting the expansion that defined `{}`", tcx.def_path_str(key) }
        separate_provide_extern
    }

    query is_panic_runtime(_: CrateNum) -> bool {
        fatal_cycle
        desc { "checking if the crate is_panic_runtime" }
        separate_provide_extern
    }

    /// Checks whether a type is representable or infinitely sized
    query representability(_: LocalDefId) -> rustc_middle::ty::Representability {
        desc { "checking if `{}` is representable", tcx.def_path_str(key) }
        // infinitely sized types will cause a cycle
        cycle_delay_bug
        // we don't want recursive representability calls to be forced with
        // incremental compilation because, if a cycle occurs, we need the
        // entire cycle to be in memory for diagnostics
        anon
    }

    /// An implementation detail for the `representability` query
    query representability_adt_ty(_: Ty<'tcx>) -> rustc_middle::ty::Representability {
        desc { "checking if `{}` is representable", key }
        cycle_delay_bug
        anon
    }

    /// Set of param indexes for type params that are in the type's representation
    query params_in_repr(key: DefId) -> &'tcx rustc_index::bit_set::BitSet<u32> {
        desc { "finding type parameters in the representation" }
        arena_cache
        no_hash
        separate_provide_extern
    }

    /// Fetch the THIR for a given body. If typeck for that body failed, returns an empty `Thir`.
    query thir_body(key: LocalDefId) -> Result<(&'tcx Steal<thir::Thir<'tcx>>, thir::ExprId), ErrorGuaranteed> {
        // Perf tests revealed that hashing THIR is inefficient (see #85729).
        no_hash
        desc { |tcx| "building THIR for `{}`", tcx.def_path_str(key) }
    }

    /// Create a THIR tree for debugging.
    query thir_tree(key: LocalDefId) -> &'tcx String {
        no_hash
        arena_cache
        desc { |tcx| "constructing THIR tree for `{}`", tcx.def_path_str(key) }
    }

    /// Create a list-like THIR representation for debugging.
    query thir_flat(key: LocalDefId) -> &'tcx String {
        no_hash
        arena_cache
        desc { |tcx| "constructing flat THIR representation for `{}`", tcx.def_path_str(key) }
    }

    /// Set of all the `DefId`s in this crate that have MIR associated with
    /// them. This includes all the body owners, but also things like struct
    /// constructors.
    query mir_keys(_: ()) -> &'tcx rustc_data_structures::fx::FxIndexSet<LocalDefId> {
        arena_cache
        desc { "getting a list of all mir_keys" }
    }

    /// Maps DefId's that have an associated `mir::Body` to the result
    /// of the MIR const-checking pass. This is the set of qualifs in
    /// the final value of a `const`.
    query mir_const_qualif(key: DefId) -> mir::ConstQualifs {
        desc { |tcx| "const checking `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    /// Fetch the MIR for a given `DefId` right after it's built - this includes
    /// unreachable code.
    query mir_built(key: LocalDefId) -> &'tcx Steal<mir::Body<'tcx>> {
        desc { |tcx| "building MIR for `{}`", tcx.def_path_str(key) }
    }

    /// Fetch the MIR for a given `DefId` up till the point where it is
    /// ready for const qualification.
    ///
    /// See the README for the `mir` module for details.
    query mir_const(key: LocalDefId) -> &'tcx Steal<mir::Body<'tcx>> {
        desc { |tcx| "preparing `{}` for borrow checking", tcx.def_path_str(key) }
        no_hash
    }

    /// Try to build an abstract representation of the given constant.
    query thir_abstract_const(
        key: DefId
    ) -> Result<Option<ty::EarlyBinder<ty::Const<'tcx>>>, ErrorGuaranteed> {
        desc {
            |tcx| "building an abstract representation for `{}`", tcx.def_path_str(key),
        }
        separate_provide_extern
    }

    query mir_drops_elaborated_and_const_checked(key: LocalDefId) -> &'tcx Steal<mir::Body<'tcx>> {
        no_hash
        desc { |tcx| "elaborating drops for `{}`", tcx.def_path_str(key) }
    }

    query mir_for_ctfe(
        key: DefId
    ) -> &'tcx mir::Body<'tcx> {
        desc { |tcx| "caching mir of `{}` for CTFE", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    query mir_promoted(key: LocalDefId) -> (
        &'tcx Steal<mir::Body<'tcx>>,
        &'tcx Steal<IndexVec<mir::Promoted, mir::Body<'tcx>>>
    ) {
        no_hash
        desc { |tcx| "promoting constants in MIR for `{}`", tcx.def_path_str(key) }
    }

    query closure_typeinfo(key: LocalDefId) -> ty::ClosureTypeInfo<'tcx> {
        desc {
            |tcx| "finding symbols for captures of closure `{}`",
            tcx.def_path_str(key)
        }
    }

    /// Returns names of captured upvars for closures and generators.
    ///
    /// Here are some examples:
    ///  - `name__field1__field2` when the upvar is captured by value.
    ///  - `_ref__name__field` when the upvar is captured by reference.
    ///
    /// For generators this only contains upvars that are shared by all states.
    query closure_saved_names_of_captured_variables(def_id: DefId) -> &'tcx IndexVec<abi::FieldIdx, Symbol> {
        arena_cache
        desc { |tcx| "computing debuginfo for closure `{}`", tcx.def_path_str(def_id) }
        separate_provide_extern
    }

    query mir_generator_witnesses(key: DefId) -> &'tcx Option<mir::GeneratorLayout<'tcx>> {
        arena_cache
        desc { |tcx| "generator witness types for `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    query check_generator_obligations(key: LocalDefId) {
        desc { |tcx| "verify auto trait bounds for generator interior type `{}`", tcx.def_path_str(key) }
    }

    /// MIR after our optimization passes have run. This is MIR that is ready
    /// for codegen. This is also the only query that can fetch non-local MIR, at present.
    query optimized_mir(key: DefId) -> &'tcx mir::Body<'tcx> {
        desc { |tcx| "optimizing MIR for `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    /// Returns coverage summary info for a function, after executing the `InstrumentCoverage`
    /// MIR pass (assuming the -Cinstrument-coverage option is enabled).
    query coverageinfo(key: ty::InstanceDef<'tcx>) -> &'tcx mir::CoverageInfo {
        desc { |tcx| "retrieving coverage info from MIR for `{}`", tcx.def_path_str(key.def_id()) }
        arena_cache
    }

    /// Returns the `CodeRegions` for a function that has instrumented coverage, in case the
    /// function was optimized out before codegen, and before being added to the Coverage Map.
    query covered_code_regions(key: DefId) -> &'tcx Vec<&'tcx mir::coverage::CodeRegion> {
        desc {
            |tcx| "retrieving the covered `CodeRegion`s, if instrumented, for `{}`",
            tcx.def_path_str(key)
        }
        arena_cache
        cache_on_disk_if { key.is_local() }
    }

    /// The `DefId` is the `DefId` of the containing MIR body. Promoteds do not have their own
    /// `DefId`. This function returns all promoteds in the specified body. The body references
    /// promoteds by the `DefId` and the `mir::Promoted` index. This is necessary, because
    /// after inlining a body may refer to promoteds from other bodies. In that case you still
    /// need to use the `DefId` of the original body.
    query promoted_mir(key: DefId) -> &'tcx IndexVec<mir::Promoted, mir::Body<'tcx>> {
        desc { |tcx| "optimizing promoted MIR for `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    /// Erases regions from `ty` to yield a new type.
    /// Normally you would just use `tcx.erase_regions(value)`,
    /// however, which uses this query as a kind of cache.
    query erase_regions_ty(ty: Ty<'tcx>) -> Ty<'tcx> {
        // This query is not expected to have input -- as a result, it
        // is not a good candidates for "replay" because it is essentially a
        // pure function of its input (and hence the expectation is that
        // no caller would be green **apart** from just these
        // queries). Making it anonymous avoids hashing the result, which
        // may save a bit of time.
        anon
        desc { "erasing regions from `{}`", ty }
    }

    query wasm_import_module_map(_: CrateNum) -> &'tcx FxHashMap<DefId, String> {
        arena_cache
        desc { "getting wasm import module map" }
    }

    /// Maps from the `DefId` of an item (trait/struct/enum/fn) to the
    /// predicates (where-clauses) directly defined on it. This is
    /// equal to the `explicit_predicates_of` predicates plus the
    /// `inferred_outlives_of` predicates.
    query predicates_defined_on(key: DefId) -> ty::GenericPredicates<'tcx> {
        desc { |tcx| "computing predicates of `{}`", tcx.def_path_str(key) }
    }

    /// Returns everything that looks like a predicate written explicitly
    /// by the user on a trait item.
    ///
    /// Traits are unusual, because predicates on associated types are
    /// converted into bounds on that type for backwards compatibility:
    ///
    /// trait X where Self::U: Copy { type U; }
    ///
    /// becomes
    ///
    /// trait X { type U: Copy; }
    ///
    /// `explicit_predicates_of` and `explicit_item_bounds` will then take
    /// the appropriate subsets of the predicates here.
    query trait_explicit_predicates_and_bounds(key: LocalDefId) -> ty::GenericPredicates<'tcx> {
        desc { |tcx| "computing explicit predicates of trait `{}`", tcx.def_path_str(key) }
    }

    /// Returns the predicates written explicitly by the user.
    query explicit_predicates_of(key: DefId) -> ty::GenericPredicates<'tcx> {
        desc { |tcx| "computing explicit predicates of `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
        feedable
    }

    /// Returns the inferred outlives predicates (e.g., for `struct
    /// Foo<'a, T> { x: &'a T }`, this would return `T: 'a`).
    query inferred_outlives_of(key: DefId) -> &'tcx [(ty::Clause<'tcx>, Span)] {
        desc { |tcx| "computing inferred outlives predicates of `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
        feedable
    }

    /// Maps from the `DefId` of a trait to the list of
    /// super-predicates. This is a subset of the full list of
    /// predicates. We store these in a separate map because we must
    /// evaluate them even during type conversion, often before the
    /// full predicates are available (note that supertraits have
    /// additional acyclicity requirements).
    query super_predicates_of(key: DefId) -> ty::GenericPredicates<'tcx> {
        desc { |tcx| "computing the super predicates of `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    query implied_predicates_of(key: DefId) -> ty::GenericPredicates<'tcx> {
        desc { |tcx| "computing the implied predicates of `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    /// The `Option<Ident>` is the name of an associated type. If it is `None`, then this query
    /// returns the full set of predicates. If `Some<Ident>`, then the query returns only the
    /// subset of super-predicates that reference traits that define the given associated type.
    /// This is used to avoid cycles in resolving types like `T::Item`.
    query super_predicates_that_define_assoc_item(key: (DefId, rustc_span::symbol::Ident)) -> ty::GenericPredicates<'tcx> {
        desc { |tcx| "computing the super traits of `{}` with associated type name `{}`",
            tcx.def_path_str(key.0),
            key.1
        }
    }

    /// To avoid cycles within the predicates of a single item we compute
    /// per-type-parameter predicates for resolving `T::AssocTy`.
    query type_param_predicates(key: (LocalDefId, LocalDefId, rustc_span::symbol::Ident)) -> ty::GenericPredicates<'tcx> {
        desc { |tcx| "computing the bounds for type parameter `{}`", tcx.hir().ty_param_name(key.1) }
    }

    query trait_def(key: DefId) -> &'tcx ty::TraitDef {
        desc { |tcx| "computing trait definition for `{}`", tcx.def_path_str(key) }
        arena_cache
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }
    query adt_def(key: DefId) -> ty::AdtDef<'tcx> {
        desc { |tcx| "computing ADT definition for `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }
    query adt_destructor(key: DefId) -> Option<ty::Destructor> {
        desc { |tcx| "computing `Drop` impl for `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    query adt_sized_constraint(key: DefId) -> &'tcx [Ty<'tcx>] {
        desc { |tcx| "computing `Sized` constraints for `{}`", tcx.def_path_str(key) }
    }

    query adt_dtorck_constraint(
        key: DefId
    ) -> Result<&'tcx DropckConstraint<'tcx>, NoSolution> {
        desc { |tcx| "computing drop-check constraints for `{}`", tcx.def_path_str(key) }
    }

    /// Returns `true` if this is a const fn, use the `is_const_fn` to know whether your crate
    /// actually sees it as const fn (e.g., the const-fn-ness might be unstable and you might
    /// not have the feature gate active).
    ///
    /// **Do not call this function manually.** It is only meant to cache the base data for the
    /// `is_const_fn` function. Consider using `is_const_fn` or `is_const_fn_raw` instead.
    query constness(key: DefId) -> hir::Constness {
        desc { |tcx| "checking if item is const: `{}`", tcx.def_path_str(key) }
        separate_provide_extern
    }

    query asyncness(key: DefId) -> hir::IsAsync {
        desc { |tcx| "checking if the function is async: `{}`", tcx.def_path_str(key) }
        separate_provide_extern
    }

    /// Returns `true` if calls to the function may be promoted.
    ///
    /// This is either because the function is e.g., a tuple-struct or tuple-variant
    /// constructor, or because it has the `#[rustc_promotable]` attribute. The attribute should
    /// be removed in the future in favour of some form of check which figures out whether the
    /// function does not inspect the bits of any of its arguments (so is essentially just a
    /// constructor function).
    query is_promotable_const_fn(key: DefId) -> bool {
        desc { |tcx| "checking if item is promotable: `{}`", tcx.def_path_str(key) }
    }

    /// Returns `Some(generator_kind)` if the node pointed to by `def_id` is a generator.
    query generator_kind(def_id: DefId) -> Option<hir::GeneratorKind> {
        desc { |tcx| "looking up generator kind of `{}`", tcx.def_path_str(def_id) }
        separate_provide_extern
    }

    /// Gets a map with the variance of every item; use `item_variance` instead.
    query crate_variances(_: ()) -> &'tcx ty::CrateVariancesMap<'tcx> {
        arena_cache
        desc { "computing the variances for items in this crate" }
    }

    /// Maps from the `DefId` of a type or region parameter to its (inferred) variance.
    query variances_of(def_id: DefId) -> &'tcx [ty::Variance] {
        desc { |tcx| "computing the variances of `{}`", tcx.def_path_str(def_id) }
        cache_on_disk_if { def_id.is_local() }
        separate_provide_extern
    }

    /// Maps from thee `DefId` of a type to its (inferred) outlives.
    query inferred_outlives_crate(_: ()) -> &'tcx ty::CratePredicatesMap<'tcx> {
        arena_cache
        desc { "computing the inferred outlives predicates for items in this crate" }
    }

    /// Maps from an impl/trait or struct/variant `DefId`
    /// to a list of the `DefId`s of its associated items or fields.
    query associated_item_def_ids(key: DefId) -> &'tcx [DefId] {
        desc { |tcx| "collecting associated items or fields of `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    /// Maps from a trait item to the trait item "descriptor".
    query associated_item(key: DefId) -> ty::AssocItem {
        desc { |tcx| "computing associated item data for `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
        feedable
    }

    /// Collects the associated items defined on a trait or impl.
    query associated_items(key: DefId) -> &'tcx ty::AssocItems {
        arena_cache
        desc { |tcx| "collecting associated items of `{}`", tcx.def_path_str(key) }
    }

    /// Maps from associated items on a trait to the corresponding associated
    /// item on the impl specified by `impl_id`.
    ///
    /// For example, with the following code
    ///
    /// ```
    /// struct Type {}
    ///                         // DefId
    /// trait Trait {           // trait_id
    ///     fn f();             // trait_f
    ///     fn g() {}           // trait_g
    /// }
    ///
    /// impl Trait for Type {   // impl_id
    ///     fn f() {}           // impl_f
    ///     fn g() {}           // impl_g
    /// }
    /// ```
    ///
    /// The map returned for `tcx.impl_item_implementor_ids(impl_id)` would be
    ///`{ trait_f: impl_f, trait_g: impl_g }`
    query impl_item_implementor_ids(impl_id: DefId) -> &'tcx DefIdMap<DefId> {
        arena_cache
        desc { |tcx| "comparing impl items against trait for `{}`", tcx.def_path_str(impl_id) }
    }

    /// Given `fn_def_id` of a trait or of an impl that implements a given trait:
    /// if `fn_def_id` is the def id of a function defined inside a trait, then it creates and returns
    /// the associated items that correspond to each impl trait in return position for that trait.
    /// if `fn_def_id` is the def id of a function defined inside an impl that implements a trait, then it
    /// creates and returns the associated items that correspond to each impl trait in return position
    /// of the implemented trait.
    query associated_types_for_impl_traits_in_associated_fn(fn_def_id: DefId) -> &'tcx [DefId] {
        desc { |tcx| "creating associated items for impl trait in trait returned by `{}`", tcx.def_path_str(fn_def_id) }
        cache_on_disk_if { fn_def_id.is_local() }
        separate_provide_extern
    }

    /// Given an impl trait in trait `opaque_ty_def_id`, create and return the corresponding
    /// associated item.
    query associated_type_for_impl_trait_in_trait(opaque_ty_def_id: LocalDefId) -> LocalDefId {
        desc { |tcx| "creates the associated item corresponding to the opaque type `{}`", tcx.def_path_str(opaque_ty_def_id.to_def_id()) }
        cache_on_disk_if { true }
    }

    /// Given an `impl_id`, return the trait it implements.
    /// Return `None` if this is an inherent impl.
    query impl_trait_ref(impl_id: DefId) -> Option<ty::EarlyBinder<ty::TraitRef<'tcx>>> {
        desc { |tcx| "computing trait implemented by `{}`", tcx.def_path_str(impl_id) }
        cache_on_disk_if { impl_id.is_local() }
        separate_provide_extern
    }
    query impl_polarity(impl_id: DefId) -> ty::ImplPolarity {
        desc { |tcx| "computing implementation polarity of `{}`", tcx.def_path_str(impl_id) }
        separate_provide_extern
    }

    query issue33140_self_ty(key: DefId) -> Option<ty::EarlyBinder<ty::Ty<'tcx>>> {
        desc { |tcx| "computing Self type wrt issue #33140 `{}`", tcx.def_path_str(key) }
    }

    /// Maps a `DefId` of a type to a list of its inherent impls.
    /// Contains implementations of methods that are inherent to a type.
    /// Methods in these implementations don't need to be exported.
    query inherent_impls(key: DefId) -> &'tcx [DefId] {
        desc { |tcx| "collecting inherent impls for `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    query incoherent_impls(key: SimplifiedType) -> &'tcx [DefId] {
        desc { |tcx| "collecting all inherent impls for `{:?}`", key }
    }

    /// The result of unsafety-checking this `LocalDefId`.
    query unsafety_check_result(key: LocalDefId) -> &'tcx mir::UnsafetyCheckResult {
        desc { |tcx| "unsafety-checking `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { true }
    }

    /// Unsafety-check this `LocalDefId` with THIR unsafeck. This should be
    /// used with `-Zthir-unsafeck`.
    query thir_check_unsafety(key: LocalDefId) {
        desc { |tcx| "unsafety-checking `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { true }
    }

    /// Returns the types assumed to be well formed while "inside" of the given item.
    ///
    /// Note that we've liberated the late bound regions of function signatures, so
    /// this can not be used to check whether these types are well formed.
    query assumed_wf_types(key: LocalDefId) -> &'tcx [(Ty<'tcx>, Span)] {
        desc { |tcx| "computing the implied bounds of `{}`", tcx.def_path_str(key) }
    }

    /// Computes the signature of the function.
    query fn_sig(key: DefId) -> ty::EarlyBinder<ty::PolyFnSig<'tcx>> {
        desc { |tcx| "computing function signature of `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
        cycle_delay_bug
    }

    /// Performs lint checking for the module.
    query lint_mod(key: LocalDefId) -> () {
        desc { |tcx| "linting {}", describe_as_module(key, tcx) }
    }

    query check_unused_traits(_: ()) -> () {
        desc { "checking unused trait imports in crate" }
    }

    /// Checks the attributes in the module.
    query check_mod_attrs(key: LocalDefId) -> () {
        desc { |tcx| "checking attributes in {}", describe_as_module(key, tcx) }
    }

    /// Checks for uses of unstable APIs in the module.
    query check_mod_unstable_api_usage(key: LocalDefId) -> () {
        desc { |tcx| "checking for unstable API usage in {}", describe_as_module(key, tcx) }
    }

    /// Checks the const bodies in the module for illegal operations (e.g. `if` or `loop`).
    query check_mod_const_bodies(key: LocalDefId) -> () {
        desc { |tcx| "checking consts in {}", describe_as_module(key, tcx) }
    }

    /// Checks the loops in the module.
    query check_mod_loops(key: LocalDefId) -> () {
        desc { |tcx| "checking loops in {}", describe_as_module(key, tcx) }
    }

    query check_mod_naked_functions(key: LocalDefId) -> () {
        desc { |tcx| "checking naked functions in {}", describe_as_module(key, tcx) }
    }

    query check_mod_item_types(key: LocalDefId) -> () {
        desc { |tcx| "checking item types in {}", describe_as_module(key, tcx) }
    }

    query check_mod_privacy(key: LocalDefId) -> () {
        desc { |tcx| "checking privacy in {}", describe_as_module(key, tcx) }
    }

    query check_liveness(key: LocalDefId) {
        desc { |tcx| "checking liveness of variables in `{}`", tcx.def_path_str(key) }
    }

    /// Return the live symbols in the crate for dead code check.
    ///
    /// The second return value maps from ADTs to ignored derived traits (e.g. Debug and Clone) and
    /// their respective impl (i.e., part of the derive macro)
    query live_symbols_and_ignored_derived_traits(_: ()) -> &'tcx (
        LocalDefIdSet,
        LocalDefIdMap<Vec<(DefId, DefId)>>
    ) {
        arena_cache
        desc { "finding live symbols in crate" }
    }

    query check_mod_deathness(key: LocalDefId) -> () {
        desc { |tcx| "checking deathness of variables in {}", describe_as_module(key, tcx) }
    }

    query check_mod_impl_wf(key: LocalDefId) -> () {
        desc { |tcx| "checking that impls are well-formed in {}", describe_as_module(key, tcx) }
    }

    query check_mod_type_wf(key: LocalDefId) -> () {
        desc { |tcx| "checking that types are well-formed in {}", describe_as_module(key, tcx) }
    }

    query collect_mod_item_types(key: LocalDefId) -> () {
        desc { |tcx| "collecting item types in {}", describe_as_module(key, tcx) }
    }

    /// Caches `CoerceUnsized` kinds for impls on custom types.
    query coerce_unsized_info(key: DefId) -> ty::adjustment::CoerceUnsizedInfo {
        desc { |tcx| "computing CoerceUnsized info for `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    query typeck(key: LocalDefId) -> &'tcx ty::TypeckResults<'tcx> {
        desc { |tcx| "type-checking `{}`", tcx.def_path_str(key) }
        cache_on_disk_if(tcx) { !tcx.is_typeck_child(key.to_def_id()) }
    }
    query diagnostic_only_typeck(key: LocalDefId) -> &'tcx ty::TypeckResults<'tcx> {
        desc { |tcx| "type-checking `{}`", tcx.def_path_str(key) }
    }

    query used_trait_imports(key: LocalDefId) -> &'tcx UnordSet<LocalDefId> {
        desc { |tcx| "finding used_trait_imports `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { true }
    }

    query has_typeck_results(def_id: DefId) -> bool {
        desc { |tcx| "checking whether `{}` has a body", tcx.def_path_str(def_id) }
    }

    query coherent_trait(def_id: DefId) -> () {
        desc { |tcx| "coherence checking all impls of trait `{}`", tcx.def_path_str(def_id) }
    }

    /// Borrow-checks the function body. If this is a closure, returns
    /// additional requirements that the closure's creator must verify.
    query mir_borrowck(key: LocalDefId) -> &'tcx mir::BorrowCheckResult<'tcx> {
        desc { |tcx| "borrow-checking `{}`", tcx.def_path_str(key) }
        cache_on_disk_if(tcx) { tcx.is_typeck_child(key.to_def_id()) }
    }

    /// Gets a complete map from all types to their inherent impls.
    /// Not meant to be used directly outside of coherence.
    query crate_inherent_impls(k: ()) -> &'tcx CrateInherentImpls {
        arena_cache
        desc { "finding all inherent impls defined in crate" }
    }

    /// Checks all types in the crate for overlap in their inherent impls. Reports errors.
    /// Not meant to be used directly outside of coherence.
    query crate_inherent_impls_overlap_check(_: ()) -> () {
        desc { "check for overlap between inherent impls defined in this crate" }
    }

    /// Checks whether all impls in the crate pass the overlap check, returning
    /// which impls fail it. If all impls are correct, the returned slice is empty.
    query orphan_check_impl(key: LocalDefId) -> Result<(), ErrorGuaranteed> {
        desc { |tcx|
            "checking whether impl `{}` follows the orphan rules",
            tcx.def_path_str(key),
        }
    }

    /// Check whether the function has any recursion that could cause the inliner to trigger
    /// a cycle. Returns the call stack causing the cycle. The call stack does not contain the
    /// current function, just all intermediate functions.
    query mir_callgraph_reachable(key: (ty::Instance<'tcx>, LocalDefId)) -> bool {
        fatal_cycle
        desc { |tcx|
            "computing if `{}` (transitively) calls `{}`",
            key.0,
            tcx.def_path_str(key.1),
        }
    }

    /// Obtain all the calls into other local functions
    query mir_inliner_callees(key: ty::InstanceDef<'tcx>) -> &'tcx [(DefId, GenericArgsRef<'tcx>)] {
        fatal_cycle
        desc { |tcx|
            "computing all local function calls in `{}`",
            tcx.def_path_str(key.def_id()),
        }
    }

    /// Evaluates a constant and returns the computed allocation.
    ///
    /// **Do not use this** directly, use the `tcx.eval_static_initializer` wrapper.
    query eval_to_allocation_raw(key: ty::ParamEnvAnd<'tcx, GlobalId<'tcx>>)
        -> EvalToAllocationRawResult<'tcx> {
        desc { |tcx|
            "const-evaluating + checking `{}`",
            key.value.display(tcx)
        }
        cache_on_disk_if { true }
    }

    /// Evaluates const items or anonymous constants
    /// (such as enum variant explicit discriminants or array lengths)
    /// into a representation suitable for the type system and const generics.
    ///
    /// **Do not use this** directly, use one of the following wrappers: `tcx.const_eval_poly`,
    /// `tcx.const_eval_resolve`, `tcx.const_eval_instance`, or `tcx.const_eval_global_id`.
    query eval_to_const_value_raw(key: ty::ParamEnvAnd<'tcx, GlobalId<'tcx>>)
        -> EvalToConstValueResult<'tcx> {
        desc { |tcx|
            "simplifying constant for the type system `{}`",
            key.value.display(tcx)
        }
        cache_on_disk_if { true }
    }

    /// Evaluate a constant and convert it to a type level constant or
    /// return `None` if that is not possible.
    query eval_to_valtree(
        key: ty::ParamEnvAnd<'tcx, GlobalId<'tcx>>
    ) -> EvalToValTreeResult<'tcx> {
        desc { "evaluating type-level constant" }
    }

    /// Converts a type level constant value into `ConstValue`
    query valtree_to_const_val(key: (Ty<'tcx>, ty::ValTree<'tcx>)) -> ConstValue<'tcx> {
        desc { "converting type-level constant value to mir constant value"}
    }

    /// Destructures array, ADT or tuple constants into the constants
    /// of their fields.
    query destructure_const(key: ty::Const<'tcx>) -> ty::DestructuredConst<'tcx> {
        desc { "destructuring type level constant"}
    }

    /// Tries to destructure an `mir::ConstantKind` ADT or array into its variant index
    /// and its field values. This should only be used for pretty printing.
    query try_destructure_mir_constant_for_diagnostics(
        key: (ConstValue<'tcx>, Ty<'tcx>)
    ) -> Option<mir::DestructuredConstant<'tcx>> {
        desc { "destructuring MIR constant"}
        no_hash
        eval_always
    }

    query const_caller_location(key: (rustc_span::Symbol, u32, u32)) -> ConstValue<'tcx> {
        desc { "getting a &core::panic::Location referring to a span" }
    }

    // FIXME get rid of this with valtrees
    query lit_to_const(
        key: LitToConstInput<'tcx>
    ) -> Result<ty::Const<'tcx>, LitToConstError> {
        desc { "converting literal to const" }
    }

    query check_match(key: LocalDefId) -> Result<(), rustc_errors::ErrorGuaranteed> {
        desc { |tcx| "match-checking `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { true }
    }

    /// Performs part of the privacy check and computes effective visibilities.
    query effective_visibilities(_: ()) -> &'tcx EffectiveVisibilities {
        eval_always
        desc { "checking effective visibilities" }
    }
    query check_private_in_public(_: ()) -> () {
        eval_always
        desc { "checking for private elements in public interfaces" }
    }

    query reachable_set(_: ()) -> &'tcx LocalDefIdSet {
        arena_cache
        desc { "reachability" }
    }

    /// Per-body `region::ScopeTree`. The `DefId` should be the owner `DefId` for the body;
    /// in the case of closures, this will be redirected to the enclosing function.
    query region_scope_tree(def_id: DefId) -> &'tcx crate::middle::region::ScopeTree {
        desc { |tcx| "computing drop scopes for `{}`", tcx.def_path_str(def_id) }
    }

    /// Generates a MIR body for the shim.
    query mir_shims(key: ty::InstanceDef<'tcx>) -> &'tcx mir::Body<'tcx> {
        arena_cache
        desc { |tcx| "generating MIR shim for `{}`", tcx.def_path_str(key.def_id()) }
    }

    /// The `symbol_name` query provides the symbol name for calling a
    /// given instance from the local crate. In particular, it will also
    /// look up the correct symbol name of instances from upstream crates.
    query symbol_name(key: ty::Instance<'tcx>) -> ty::SymbolName<'tcx> {
        desc { "computing the symbol for `{}`", key }
        cache_on_disk_if { true }
    }

    query opt_def_kind(def_id: DefId) -> Option<DefKind> {
        desc { |tcx| "looking up definition kind of `{}`", tcx.def_path_str(def_id) }
        cache_on_disk_if { def_id.is_local() }
        separate_provide_extern
        feedable
    }

    /// Gets the span for the definition.
    query def_span(def_id: DefId) -> Span {
        desc { |tcx| "looking up span for `{}`", tcx.def_path_str(def_id) }
        cache_on_disk_if { def_id.is_local() }
        separate_provide_extern
        feedable
    }

    /// Gets the span for the identifier of the definition.
    query def_ident_span(def_id: DefId) -> Option<Span> {
        desc { |tcx| "looking up span for `{}`'s identifier", tcx.def_path_str(def_id) }
        cache_on_disk_if { def_id.is_local() }
        separate_provide_extern
        feedable
    }

    query lookup_stability(def_id: DefId) -> Option<attr::Stability> {
        desc { |tcx| "looking up stability of `{}`", tcx.def_path_str(def_id) }
        cache_on_disk_if { def_id.is_local() }
        separate_provide_extern
    }

    query lookup_const_stability(def_id: DefId) -> Option<attr::ConstStability> {
        desc { |tcx| "looking up const stability of `{}`", tcx.def_path_str(def_id) }
        cache_on_disk_if { def_id.is_local() }
        separate_provide_extern
    }

    query lookup_default_body_stability(def_id: DefId) -> Option<attr::DefaultBodyStability> {
        desc { |tcx| "looking up default body stability of `{}`", tcx.def_path_str(def_id) }
        separate_provide_extern
    }

    query should_inherit_track_caller(def_id: DefId) -> bool {
        desc { |tcx| "computing should_inherit_track_caller of `{}`", tcx.def_path_str(def_id) }
    }

    query lookup_deprecation_entry(def_id: DefId) -> Option<DeprecationEntry> {
        desc { |tcx| "checking whether `{}` is deprecated", tcx.def_path_str(def_id) }
        cache_on_disk_if { def_id.is_local() }
        separate_provide_extern
    }

    /// Determines whether an item is annotated with `doc(hidden)`.
    query is_doc_hidden(def_id: DefId) -> bool {
        desc { |tcx| "checking whether `{}` is `doc(hidden)`", tcx.def_path_str(def_id) }
        separate_provide_extern
    }

    /// Determines whether an item is annotated with `doc(notable_trait)`.
    query is_doc_notable_trait(def_id: DefId) -> bool {
        desc { |tcx| "checking whether `{}` is `doc(notable_trait)`", tcx.def_path_str(def_id) }
    }

    /// Returns the attributes on the item at `def_id`.
    ///
    /// Do not use this directly, use `tcx.get_attrs` instead.
    query item_attrs(def_id: DefId) -> &'tcx [ast::Attribute] {
        desc { |tcx| "collecting attributes of `{}`", tcx.def_path_str(def_id) }
        separate_provide_extern
    }

    query codegen_fn_attrs(def_id: DefId) -> &'tcx CodegenFnAttrs {
        desc { |tcx| "computing codegen attributes of `{}`", tcx.def_path_str(def_id) }
        arena_cache
        cache_on_disk_if { def_id.is_local() }
        separate_provide_extern
    }

    query asm_target_features(def_id: DefId) -> &'tcx FxIndexSet<Symbol> {
        desc { |tcx| "computing target features for inline asm of `{}`", tcx.def_path_str(def_id) }
    }

    query fn_arg_names(def_id: DefId) -> &'tcx [rustc_span::symbol::Ident] {
        desc { |tcx| "looking up function parameter names for `{}`", tcx.def_path_str(def_id) }
        separate_provide_extern
    }
    /// Gets the rendered value of the specified constant or associated constant.
    /// Used by rustdoc.
    query rendered_const(def_id: DefId) -> &'tcx String {
        arena_cache
        desc { |tcx| "rendering constant initializer of `{}`", tcx.def_path_str(def_id) }
        separate_provide_extern
    }
    query impl_parent(def_id: DefId) -> Option<DefId> {
        desc { |tcx| "computing specialization parent impl of `{}`", tcx.def_path_str(def_id) }
        separate_provide_extern
    }

    query is_ctfe_mir_available(key: DefId) -> bool {
        desc { |tcx| "checking if item has CTFE MIR available: `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }
    query is_mir_available(key: DefId) -> bool {
        desc { |tcx| "checking if item has MIR available: `{}`", tcx.def_path_str(key) }
        cache_on_disk_if { key.is_local() }
        separate_provide_extern
    }

    query own_existential_vtable_entries(
        key: DefId
    ) -> &'tcx [DefId] {
        desc { |tcx| "finding all existential vtable entries for trait `{}`", tcx.def_path_str(key) }
    }

    query vtable_entries(key: ty::PolyTraitRef<'tcx>)
                        -> &'tcx [ty::VtblEntry<'tcx>] {
        desc { |tcx| "finding all vtable entries for trait `{}`", tcx.def_path_str(key.def_id()) }
    }

    query vtable_trait_upcasting_coercion_new_vptr_slot(key: (Ty<'tcx>, Ty<'tcx>)) -> Option<usize> {
        desc { |tcx| "finding the slot within vtable for trait object `{}` vtable ptr during trait upcasting coercion from `{}` vtable",
            key.1, key.0 }
    }

    query vtable_allocation(key: (Ty<'tcx>, Option<ty::PolyExistentialTraitRef<'tcx>>)) -> mir::interpret::AllocId {
        desc { |tcx| "vtable const allocation for <{} as {}>",
            key.0,
            key.1.map(|trait_ref| format!("{}", trait_ref)).unwrap_or("_".to_owned())
        }
    }

    query codegen_select_candidate(
        key: (ty::ParamEnv<'tcx>, ty::TraitRef<'tcx>)
    ) -> Result<&'tcx ImplSource<'tcx, ()>, CodegenObligationError> {
        cache_on_disk_if { true }
        desc { |tcx| "computing candidate for `{}`", key.1 }
    }

    /// Return all `impl` blocks in the current crate.
    query all_local_trait_impls(_: ()) -> &'tcx rustc_data_structures::fx::FxIndexMap<DefId, Vec<LocalDefId>> {
        desc { "finding local trait impls" }
    }

    /// Given a trait `trait_id`, return all known `impl` blocks.
    query trait_impls_of(trait_id: DefId) -> &'tcx ty::trait_def::TraitImpls {
        arena_cache
        desc { |tcx| "finding trait impls of `{}`", tcx.def_path_str(trait_id) }
    }

    query specialization_graph_of(trait_id: DefId) -> &'tcx specialization_graph::Graph {
        arena_cache
        desc { |tcx| "building specialization graph of trait `{}`", tcx.def_path_str(trait_id) }
        cache_on_disk_if { true }
    }
    query object_safety_violations(trait_id: DefId) -> &'tcx [ObjectSafetyViolation] {
        desc { |tcx| "determining object safety of trait `{}`", tcx.def_path_str(trait_id) }
    }
    query check_is_object_safe(trait_id: DefId) -> bool {
        desc { |tcx| "checking if trait `{}` is object safe", tcx.def_path_str(trait_id) }
    }

    /// Gets the ParameterEnvironment for a given item; this environment
    /// will be in "user-facing" mode, meaning that it is suitable for
    /// type-checking etc, and it does not normalize specializable
    /// associated types. This is almost always what you want,
    /// unless you are doing MIR optimizations, in which case you
    /// might want to use `reveal_all()` method to change modes.
    query param_env(def_id: DefId) -> ty::ParamEnv<'tcx> {
        desc { |tcx| "computing normalized predicates of `{}`", tcx.def_path_str(def_id) }
        feedable
    }

    /// Like `param_env`, but returns the `ParamEnv` in `Reveal::All` mode.
    /// Prefer this over `tcx.param_env(def_id).with_reveal_all_normalized(tcx)`,
    /// as this method is more efficient.
    query param_env_reveal_all_normalized(def_id: DefId) -> ty::ParamEnv<'tcx> {
        desc { |tcx| "computing revealed normalized predicates of `{}`", tcx.def_path_str(def_id) }
    }

    /// Trait selection queries. These are best used by invoking `ty.is_copy_modulo_regions()`,
    /// `ty.is_copy()`, etc, since that will prune the environment where possible.
    query is_copy_raw(env: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
        desc { "computing whether `{}` is `Copy`", env.value }
    }
    /// Query backing `Ty::is_sized`.
    query is_sized_raw(env: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
        desc { "computing whether `{}` is `Sized`", env.value }
    }
    /// Query backing `Ty::is_freeze`.
    query is_freeze_raw(env: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
        desc { "computing whether `{}` is freeze", env.value }
    }
    /// Query backing `Ty::is_unpin`.
    query is_unpin_raw(env: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
        desc { "computing whether `{}` is `Unpin`", env.value }
    }
    /// Query backing `Ty::needs_drop`.
    query needs_drop_raw(env: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
        desc { "computing whether `{}` needs drop", env.value }
    }
    /// Query backing `Ty::has_significant_drop_raw`.
    query has_significant_drop_raw(env: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
        desc { "computing whether `{}` has a significant drop", env.value }
    }

    /// Query backing `Ty::is_structural_eq_shallow`.
    ///
    /// This is only correct for ADTs. Call `is_structural_eq_shallow` to handle all types
    /// correctly.
    query has_structural_eq_impls(ty: Ty<'tcx>) -> bool {
        desc {
            "computing whether `{}` implements `PartialStructuralEq` and `StructuralEq`",
            ty
        }
    }

    /// A list of types where the ADT requires drop if and only if any of
    /// those types require drop. If the ADT is known to always need drop
    /// then `Err(AlwaysRequiresDrop)` is returned.
    query adt_drop_tys(def_id: DefId) -> Result<&'tcx ty::List<Ty<'tcx>>, AlwaysRequiresDrop> {
        desc { |tcx| "computing when `{}` needs drop", tcx.def_path_str(def_id) }
        cache_on_disk_if { true }
    }

    /// A list of types where the ADT requires drop if and only if any of those types
    /// has significant drop. A type marked with the attribute `rustc_insignificant_dtor`
    /// is considered to not be significant. A drop is significant if it is implemented
    /// by the user or does anything that will have any observable behavior (other than
    /// freeing up memory). If the ADT is known to have a significant destructor then
    /// `Err(AlwaysRequiresDrop)` is returned.
    query adt_significant_drop_tys(def_id: DefId) -> Result<&'tcx ty::List<Ty<'tcx>>, AlwaysRequiresDrop> {
        desc { |tcx| "computing when `{}` has a significant destructor", tcx.def_path_str(def_id) }
        cache_on_disk_if { false }
    }

    /// Computes the layout of a type. Note that this implicitly
    /// executes in "reveal all" mode, and will normalize the input type.
    query layout_of(
        key: ty::ParamEnvAnd<'tcx, Ty<'tcx>>
    ) -> Result<ty::layout::TyAndLayout<'tcx>, &'tcx ty::layout::LayoutError<'tcx>> {
        depth_limit
        desc { "computing layout of `{}`", key.value }
    }

    /// Compute a `FnAbi` suitable for indirect calls, i.e. to `fn` pointers.
    ///
    /// NB: this doesn't handle virtual calls - those should use `fn_abi_of_instance`
    /// instead, where the instance is an `InstanceDef::Virtual`.
    query fn_abi_of_fn_ptr(
        key: ty::ParamEnvAnd<'tcx, (ty::PolyFnSig<'tcx>, &'tcx ty::List<Ty<'tcx>>)>
    ) -> Result<&'tcx abi::call::FnAbi<'tcx, Ty<'tcx>>, &'tcx ty::layout::FnAbiError<'tcx>> {
        desc { "computing call ABI of `{}` function pointers", key.value.0 }
    }

    /// Compute a `FnAbi` suitable for declaring/defining an `fn` instance, and for
    /// direct calls to an `fn`.
    ///
    /// NB: that includes virtual calls, which are represented by "direct calls"
    /// to an `InstanceDef::Virtual` instance (of `<dyn Trait as Trait>::fn`).
    query fn_abi_of_instance(
        key: ty::ParamEnvAnd<'tcx, (ty::Instance<'tcx>, &'tcx ty::List<Ty<'tcx>>)>
    ) -> Result<&'tcx abi::call::FnAbi<'tcx, Ty<'tcx>>, &'tcx ty::layout::FnAbiError<'tcx>> {
        desc { "computing call ABI of `{}`", key.value.0 }
    }

    query dylib_dependency_formats(_: CrateNum)
                                    -> &'tcx [(CrateNum, LinkagePreference)] {
        desc { "getting dylib dependency formats of crate" }
        separate_provide_extern
    }

    query dependency_formats(_: ()) -> &'tcx Lrc<crate::middle::dependency_format::Dependencies> {
        arena_cache
        desc { "getting the linkage format of all dependencies" }
    }

    query is_compiler_builtins(_: CrateNum) -> bool {
        fatal_cycle
        desc { "checking if the crate is_compiler_builtins" }
        separate_provide_extern
    }
    query has_global_allocator(_: CrateNum) -> bool {
        // This query depends on untracked global state in CStore
        eval_always
        fatal_cycle
        desc { "checking if the crate has_global_allocator" }
        separate_provide_extern
    }
    query has_alloc_error_handler(_: CrateNum) -> bool {
        // This query depends on untracked global state in CStore
        eval_always
        fatal_cycle
        desc { "checking if the crate has_alloc_error_handler" }
        separate_provide_extern
    }
    query has_panic_handler(_: CrateNum) -> bool {
        fatal_cycle
        desc { "checking if the crate has_panic_handler" }
        separate_provide_extern
    }
    query is_profiler_runtime(_: CrateNum) -> bool {
        fatal_cycle
        desc { "checking if a crate is `#![profiler_runtime]`" }
        separate_provide_extern
    }
    query has_ffi_unwind_calls(key: LocalDefId) -> bool {
        desc { |tcx| "checking if `{}` contains FFI-unwind calls", tcx.def_path_str(key) }
        cache_on_disk_if { true }
    }
    query required_panic_strategy(_: CrateNum) -> Option<PanicStrategy> {
        fatal_cycle
        desc { "getting a crate's required panic strategy" }
        separate_provide_extern
    }
    query panic_in_drop_strategy(_: CrateNum) -> PanicStrategy {
        fatal_cycle
        desc { "getting a crate's configured panic-in-drop strategy" }
        separate_provide_extern
    }
    query is_no_builtins(_: CrateNum) -> bool {
        fatal_cycle
        desc { "getting whether a crate has `#![no_builtins]`" }
        separate_provide_extern
    }
    query symbol_mangling_version(_: CrateNum) -> SymbolManglingVersion {
        fatal_cycle
        desc { "getting a crate's symbol mangling version" }
        separate_provide_extern
    }

    query extern_crate(def_id: DefId) -> Option<&'tcx ExternCrate> {
        eval_always
        desc { "getting crate's ExternCrateData" }
        separate_provide_extern
    }

    query specializes(_: (DefId, DefId)) -> bool {
        desc { "computing whether impls specialize one another" }
    }
    query in_scope_traits_map(_: hir::OwnerId)
        -> Option<&'tcx FxHashMap<ItemLocalId, Box<[TraitCandidate]>>> {
        desc { "getting traits in scope at a block" }
    }

    /// Returns whether the impl or associated function has the `default` keyword.
    query defaultness(def_id: DefId) -> hir::Defaultness {
        desc { |tcx| "looking up whether `{}` has `default`", tcx.def_path_str(def_id) }
        separate_provide_extern
        feedable
    }

    query check_well_formed(key: hir::OwnerId) -> () {
        desc { |tcx| "checking that `{}` is well-formed", tcx.def_path_str(key) }
    }

    // The `DefId`s of all non-generic functions and statics in the given crate
    // that can be reached from outside the crate.
    //
    // We expect this items to be available for being linked to.
    //
    // This query can also be called for `LOCAL_CRATE`. In this case it will
    // compute which items will be reachable to other crates, taking into account
    // the kind of crate that is currently compiled. Crates with only a
    // C interface have fewer reachable things.
    //
    // Does not include external symbols that don't have a corresponding DefId,
    // like the compiler-generated `main` function and so on.
    query reachable_non_generics(_: CrateNum)
        -> &'tcx DefIdMap<SymbolExportInfo> {
        arena_cache
        desc { "looking up the exported symbols of a crate" }
        separate_provide_extern
    }
    query is_reachable_non_generic(def_id: DefId) -> bool {
        desc { |tcx| "checking whether `{}` is an exported symbol", tcx.def_path_str(def_id) }
        cache_on_disk_if { def_id.is_local() }
        separate_provide_extern
    }
    query is_unreachable_local_definition(def_id: LocalDefId) -> bool {
        desc { |tcx|
            "checking whether `{}` is reachable from outside the crate",
            tcx.def_path_str(def_id),
        }
    }

    /// The entire set of monomorphizations the local crate can safely link
    /// to because they are exported from upstream crates. Do not depend on
    /// this directly, as its value changes anytime a monomorphization gets
    /// added or removed in any upstream crate. Instead use the narrower
    /// `upstream_monomorphizations_for`, `upstream_drop_glue_for`, or, even
    /// better, `Instance::upstream_monomorphization()`.
    query upstream_monomorphizations(_: ()) -> &'tcx DefIdMap<FxHashMap<GenericArgsRef<'tcx>, CrateNum>> {
        arena_cache
        desc { "collecting available upstream monomorphizations" }
    }

    /// Returns the set of upstream monomorphizations available for the
    /// generic function identified by the given `def_id`. The query makes
    /// sure to make a stable selection if the same monomorphization is
    /// available in multiple upstream crates.
    ///
    /// You likely want to call `Instance::upstream_monomorphization()`
    /// instead of invoking this query directly.
    query upstream_monomorphizations_for(def_id: DefId)
        -> Option<&'tcx FxHashMap<GenericArgsRef<'tcx>, CrateNum>>
    {
        desc { |tcx|
            "collecting available upstream monomorphizations for `{}`",
            tcx.def_path_str(def_id),
        }
        separate_provide_extern
    }

    /// Returns the upstream crate that exports drop-glue for the given
    /// type (`args` is expected to be a single-item list containing the
    /// type one wants drop-glue for).
    ///
    /// This is a subset of `upstream_monomorphizations_for` in order to
    /// increase dep-tracking granularity. Otherwise adding or removing any
    /// type with drop-glue in any upstream crate would invalidate all
    /// functions calling drop-glue of an upstream type.
    ///
    /// You likely want to call `Instance::upstream_monomorphization()`
    /// instead of invoking this query directly.
    ///
    /// NOTE: This query could easily be extended to also support other
    ///       common functions that have are large set of monomorphizations
    ///       (like `Clone::clone` for example).
    query upstream_drop_glue_for(args: GenericArgsRef<'tcx>) -> Option<CrateNum> {
        desc { "available upstream drop-glue for `{:?}`", args }
    }

    /// Returns a list of all `extern` blocks of a crate.
    query foreign_modules(_: CrateNum) -> &'tcx FxIndexMap<DefId, ForeignModule> {
        arena_cache
        desc { "looking up the foreign modules of a linked crate" }
        separate_provide_extern
    }

    /// Identifies the entry-point (e.g., the `main` function) for a given
    /// crate, returning `None` if there is no entry point (such as for library crates).
    query entry_fn(_: ()) -> Option<(DefId, EntryFnType)> {
        desc { "looking up the entry function of a crate" }
    }

    /// Finds the `rustc_proc_macro_decls` item of a crate.
    query proc_macro_decls_static(_: ()) -> Option<LocalDefId> {
        desc { "looking up the proc macro declarations for a crate" }
    }

    // The macro which defines `rustc_metadata::provide_extern` depends on this query's name.
    // Changing the name should cause a compiler error, but in case that changes, be aware.
    query crate_hash(_: CrateNum) -> Svh {
        eval_always
        desc { "looking up the hash a crate" }
        separate_provide_extern
    }

    /// Gets the hash for the host proc macro. Used to support -Z dual-proc-macro.
    query crate_host_hash(_: CrateNum) -> Option<Svh> {
        eval_always
        desc { "looking up the hash of a host version of a crate" }
        separate_provide_extern
    }

    /// Gets the extra data to put in each output filename for a crate.
    /// For example, compiling the `foo` crate with `extra-filename=-a` creates a `libfoo-b.rlib` file.
    query extra_filename(_: CrateNum) -> &'tcx String {
        arena_cache
        eval_always
        desc { "looking up the extra filename for a crate" }
        separate_provide_extern
    }

    /// Gets the paths where the crate came from in the file system.
    query crate_extern_paths(_: CrateNum) -> &'tcx Vec<PathBuf> {
        arena_cache
        eval_always
        desc { "looking up the paths for extern crates" }
        separate_provide_extern
    }

    /// Given a crate and a trait, look up all impls of that trait in the crate.
    /// Return `(impl_id, self_ty)`.
    query implementations_of_trait(_: (CrateNum, DefId)) -> &'tcx [(DefId, Option<SimplifiedType>)] {
        desc { "looking up implementations of a trait in a crate" }
        separate_provide_extern
    }

    /// Collects all incoherent impls for the given crate and type.
    ///
    /// Do not call this directly, but instead use the `incoherent_impls` query.
    /// This query is only used to get the data necessary for that query.
    query crate_incoherent_impls(key: (CrateNum, SimplifiedType)) -> &'tcx [DefId] {
        desc { |tcx| "collecting all impls for a type in a crate" }
        separate_provide_extern
    }

    /// Get the corresponding native library from the `native_libraries` query
    query native_library(def_id: DefId) -> Option<&'tcx NativeLib> {
        desc { |tcx| "getting the native library for `{}`", tcx.def_path_str(def_id) }
    }

    /// Does lifetime resolution on items. Importantly, we can't resolve
    /// lifetimes directly on things like trait methods, because of trait params.
    /// See `rustc_resolve::late::lifetimes` for details.
    query resolve_bound_vars(_: hir::OwnerId) -> &'tcx ResolveBoundVars {
        arena_cache
        desc { "resolving lifetimes" }
    }
    query named_variable_map(_: hir::OwnerId) ->
        Option<&'tcx FxHashMap<ItemLocalId, ResolvedArg>> {
        desc { "looking up a named region" }
    }
    query is_late_bound_map(_: hir::OwnerId) -> Option<&'tcx FxIndexSet<ItemLocalId>> {
        desc { "testing if a region is late bound" }
    }
    /// For a given item's generic parameter, gets the default lifetimes to be used
    /// for each parameter if a trait object were to be passed for that parameter.
    /// For example, for `T` in `struct Foo<'a, T>`, this would be `'static`.
    /// For `T` in `struct Foo<'a, T: 'a>`, this would instead be `'a`.
    /// This query will panic if passed something that is not a type parameter.
    query object_lifetime_default(key: DefId) -> ObjectLifetimeDefault {
        desc { "looking up lifetime defaults for generic parameter `{}`", tcx.def_path_str(key) }
        separate_provide_extern
    }
    query late_bound_vars_map(_: hir::OwnerId)
        -> Option<&'tcx FxHashMap<ItemLocalId, Vec<ty::BoundVariableKind>>> {
        desc { "looking up late bound vars" }
    }

    /// Computes the visibility of the provided `def_id`.
    ///
    /// If the item from the `def_id` doesn't have a visibility, it will panic. For example
    /// a generic type parameter will panic if you call this method on it:
    ///
    /// ```
    /// use std::fmt::Debug;
    ///
    /// pub trait Foo<T: Debug> {}
    /// ```
    ///
    /// In here, if you call `visibility` on `T`, it'll panic.
    query visibility(def_id: DefId) -> ty::Visibility<DefId> {
        desc { |tcx| "computing visibility of `{}`", tcx.def_path_str(def_id) }
        separate_provide_extern
        feedable
    }

    query inhabited_predicate_adt(key: DefId) -> ty::inhabitedness::InhabitedPredicate<'tcx> {
        desc { "computing the uninhabited predicate of `{:?}`", key }
    }

    /// Do not call this query directly: invoke `Ty::inhabited_predicate` instead.
    query inhabited_predicate_type(key: Ty<'tcx>) -> ty::inhabitedness::InhabitedPredicate<'tcx> {
        desc { "computing the uninhabited predicate of `{}`", key }
    }

    query dep_kind(_: CrateNum) -> CrateDepKind {
        eval_always
        desc { "fetching what a dependency looks like" }
        separate_provide_extern
    }

    /// Gets the name of the crate.
    query crate_name(_: CrateNum) -> Symbol {
        feedable
        desc { "fetching what a crate is named" }
        separate_provide_extern
    }
    query module_children(def_id: DefId) -> &'tcx [ModChild] {
        desc { |tcx| "collecting child items of module `{}`", tcx.def_path_str(def_id) }
        separate_provide_extern
    }
    query extern_mod_stmt_cnum(def_id: LocalDefId) -> Option<CrateNum> {
        desc { |tcx| "computing crate imported by `{}`", tcx.def_path_str(def_id) }
    }

    query lib_features(_: ()) -> &'tcx LibFeatures {
        arena_cache
        desc { "calculating the lib features map" }
    }
    query defined_lib_features(_: CrateNum) -> &'tcx [(Symbol, Option<Symbol>)] {
        desc { "calculating the lib features defined in a crate" }
        separate_provide_extern
    }
    query stability_implications(_: CrateNum) -> &'tcx FxHashMap<Symbol, Symbol> {
        arena_cache
        desc { "calculating the implications between `#[unstable]` features defined in a crate" }
        separate_provide_extern
    }
    /// Whether the function is an intrinsic
    query is_intrinsic(def_id: DefId) -> bool {
        desc { |tcx| "checking whether `{}` is an intrinsic", tcx.def_path_str(def_id) }
        separate_provide_extern
    }
    /// Returns the lang items defined in another crate by loading it from metadata.
    query get_lang_items(_: ()) -> &'tcx LanguageItems {
        arena_cache
        eval_always
        desc { "calculating the lang items map" }
    }

    /// Returns all diagnostic items defined in all crates.
    query all_diagnostic_items(_: ()) -> &'tcx rustc_hir::diagnostic_items::DiagnosticItems {
        arena_cache
        eval_always
        desc { "calculating the diagnostic items map" }
    }

    /// Returns the lang items defined in another crate by loading it from metadata.
    query defined_lang_items(_: CrateNum) -> &'tcx [(DefId, LangItem)] {
        desc { "calculating the lang items defined in a crate" }
        separate_provide_extern
    }

    /// Returns the diagnostic items defined in a crate.
    query diagnostic_items(_: CrateNum) -> &'tcx rustc_hir::diagnostic_items::DiagnosticItems {
        arena_cache
        desc { "calculating the diagnostic items map in a crate" }
        separate_provide_extern
    }

    query missing_lang_items(_: CrateNum) -> &'tcx [LangItem] {
        desc { "calculating the missing lang items in a crate" }
        separate_provide_extern
    }
    query visible_parent_map(_: ()) -> &'tcx DefIdMap<DefId> {
        arena_cache
        desc { "calculating the visible parent map" }
    }
    query trimmed_def_paths(_: ()) -> &'tcx FxHashMap<DefId, Symbol> {
        arena_cache
        desc { "calculating trimmed def paths" }
    }
    query missing_extern_crate_item(_: CrateNum) -> bool {
        eval_always
        desc { "seeing if we're missing an `extern crate` item for this crate" }
        separate_provide_extern
    }
    query used_crate_source(_: CrateNum) -> &'tcx Lrc<CrateSource> {
        arena_cache
        eval_always
        desc { "looking at the source for a crate" }
        separate_provide_extern
    }

    /// Returns the debugger visualizers defined for this crate.
    /// NOTE: This query has to be marked `eval_always` because it reads data
    ///       directly from disk that is not tracked anywhere else. I.e. it
    ///       represents a genuine input to the query system.
    query debugger_visualizers(_: CrateNum) -> &'tcx Vec<DebuggerVisualizerFile> {
        arena_cache
        desc { "looking up the debugger visualizers for this crate" }
        separate_provide_extern
        eval_always
    }

    query postorder_cnums(_: ()) -> &'tcx [CrateNum] {
        eval_always
        desc { "generating a postorder list of CrateNums" }
    }
    /// Returns whether or not the crate with CrateNum 'cnum'
    /// is marked as a private dependency
    query is_private_dep(c: CrateNum) -> bool {
        eval_always
        desc { "checking whether crate `{}` is a private dependency", c }
        separate_provide_extern
    }
    query allocator_kind(_: ()) -> Option<AllocatorKind> {
        eval_always
        desc { "getting the allocator kind for the current crate" }
    }
    query alloc_error_handler_kind(_: ()) -> Option<AllocatorKind> {
        eval_always
        desc { "alloc error handler kind for the current crate" }
    }

    query upvars_mentioned(def_id: DefId) -> Option<&'tcx FxIndexMap<hir::HirId, hir::Upvar>> {
        desc { |tcx| "collecting upvars mentioned in `{}`", tcx.def_path_str(def_id) }
    }
    query maybe_unused_trait_imports(_: ()) -> &'tcx FxIndexSet<LocalDefId> {
        desc { "fetching potentially unused trait imports" }
    }
    query names_imported_by_glob_use(def_id: LocalDefId) -> &'tcx UnordSet<Symbol> {
        desc { |tcx| "finding names imported by glob use for `{}`", tcx.def_path_str(def_id) }
    }

    query stability_index(_: ()) -> &'tcx stability::Index {
        arena_cache
        eval_always
        desc { "calculating the stability index for the local crate" }
    }
    query crates(_: ()) -> &'tcx [CrateNum] {
        eval_always
        desc { "fetching all foreign CrateNum instances" }
    }

    /// A list of all traits in a crate, used by rustdoc and error reporting.
    query traits(_: CrateNum) -> &'tcx [DefId] {
        desc { "fetching all traits in a crate" }
        separate_provide_extern
    }

    query trait_impls_in_crate(_: CrateNum) -> &'tcx [DefId] {
        desc { "fetching all trait impls in a crate" }
        separate_provide_extern
    }

    /// The list of symbols exported from the given crate.
    ///
    /// - All names contained in `exported_symbols(cnum)` are guaranteed to
    ///   correspond to a publicly visible symbol in `cnum` machine code.
    /// - The `exported_symbols` sets of different crates do not intersect.
    query exported_symbols(cnum: CrateNum) -> &'tcx [(ExportedSymbol<'tcx>, SymbolExportInfo)] {
        desc { "collecting exported symbols for crate `{}`", cnum}
        cache_on_disk_if { *cnum == LOCAL_CRATE }
        separate_provide_extern
    }

    query collect_and_partition_mono_items(_: ()) -> (&'tcx DefIdSet, &'tcx [CodegenUnit<'tcx>]) {
        eval_always
        desc { "collect_and_partition_mono_items" }
    }

    query is_codegened_item(def_id: DefId) -> bool {
        desc { |tcx| "determining whether `{}` needs codegen", tcx.def_path_str(def_id) }
    }

    /// All items participating in code generation together with items inlined into them.
    query codegened_and_inlined_items(_: ()) -> &'tcx DefIdSet {
        eval_always
        desc { "collecting codegened and inlined items" }
    }

    query codegen_unit(sym: Symbol) -> &'tcx CodegenUnit<'tcx> {
        desc { "getting codegen unit `{sym}`" }
    }

    query unused_generic_params(key: ty::InstanceDef<'tcx>) -> UnusedGenericParams {
        cache_on_disk_if { key.def_id().is_local() }
        desc {
            |tcx| "determining which generic parameters are unused by `{}`",
                tcx.def_path_str(key.def_id())
        }
        separate_provide_extern
    }

    query backend_optimization_level(_: ()) -> OptLevel {
        desc { "optimization level used by backend" }
    }

    /// Return the filenames where output artefacts shall be stored.
    ///
    /// This query returns an `&Arc` because codegen backends need the value even after the `TyCtxt`
    /// has been destroyed.
    query output_filenames(_: ()) -> &'tcx Arc<OutputFilenames> {
        feedable
        desc { "getting output filenames" }
        arena_cache
    }

    /// Do not call this query directly: invoke `normalize` instead.
    query normalize_projection_ty(
        goal: CanonicalProjectionGoal<'tcx>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, NormalizationResult<'tcx>>>,
        NoSolution,
    > {
        desc { "normalizing `{}`", goal.value.value }
    }

    /// Do not call this query directly: invoke `normalize` instead.
    query normalize_weak_ty(
        goal: CanonicalProjectionGoal<'tcx>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, NormalizationResult<'tcx>>>,
        NoSolution,
    > {
        desc { "normalizing `{}`", goal.value.value }
    }

    /// Do not call this query directly: invoke `normalize` instead.
    query normalize_inherent_projection_ty(
        goal: CanonicalProjectionGoal<'tcx>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, NormalizationResult<'tcx>>>,
        NoSolution,
    > {
        desc { "normalizing `{}`", goal.value.value }
    }

    /// Do not call this query directly: invoke `try_normalize_erasing_regions` instead.
    query try_normalize_generic_arg_after_erasing_regions(
        goal: ParamEnvAnd<'tcx, GenericArg<'tcx>>
    ) -> Result<GenericArg<'tcx>, NoSolution> {
        desc { "normalizing `{}`", goal.value }
    }

    query implied_outlives_bounds(
        goal: CanonicalTyGoal<'tcx>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, Vec<OutlivesBound<'tcx>>>>,
        NoSolution,
    > {
        desc { "computing implied outlives bounds for `{}`", goal.value.value }
    }

    /// Do not call this query directly:
    /// invoke `DropckOutlives::new(dropped_ty)).fully_perform(typeck.infcx)` instead.
    query dropck_outlives(
        goal: CanonicalTyGoal<'tcx>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, DropckOutlivesResult<'tcx>>>,
        NoSolution,
    > {
        desc { "computing dropck types for `{}`", goal.value.value }
    }

    /// Do not call this query directly: invoke `infcx.predicate_may_hold()` or
    /// `infcx.predicate_must_hold()` instead.
    query evaluate_obligation(
        goal: CanonicalPredicateGoal<'tcx>
    ) -> Result<EvaluationResult, OverflowError> {
        desc { "evaluating trait selection obligation `{}`", goal.value.value }
    }

    /// Do not call this query directly: part of the `Eq` type-op
    query type_op_ascribe_user_type(
        goal: CanonicalTypeOpAscribeUserTypeGoal<'tcx>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, ()>>,
        NoSolution,
    > {
        desc { "evaluating `type_op_ascribe_user_type` `{:?}`", goal.value.value }
    }

    /// Do not call this query directly: part of the `Eq` type-op
    query type_op_eq(
        goal: CanonicalTypeOpEqGoal<'tcx>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, ()>>,
        NoSolution,
    > {
        desc { "evaluating `type_op_eq` `{:?}`", goal.value.value }
    }

    /// Do not call this query directly: part of the `Subtype` type-op
    query type_op_subtype(
        goal: CanonicalTypeOpSubtypeGoal<'tcx>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, ()>>,
        NoSolution,
    > {
        desc { "evaluating `type_op_subtype` `{:?}`", goal.value.value }
    }

    /// Do not call this query directly: part of the `ProvePredicate` type-op
    query type_op_prove_predicate(
        goal: CanonicalTypeOpProvePredicateGoal<'tcx>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, ()>>,
        NoSolution,
    > {
        desc { "evaluating `type_op_prove_predicate` `{:?}`", goal.value.value }
    }

    /// Do not call this query directly: part of the `Normalize` type-op
    query type_op_normalize_ty(
        goal: CanonicalTypeOpNormalizeGoal<'tcx, Ty<'tcx>>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, Ty<'tcx>>>,
        NoSolution,
    > {
        desc { "normalizing `{}`", goal.value.value.value }
    }

    /// Do not call this query directly: part of the `Normalize` type-op
    query type_op_normalize_clause(
        goal: CanonicalTypeOpNormalizeGoal<'tcx, ty::Clause<'tcx>>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, ty::Clause<'tcx>>>,
        NoSolution,
    > {
        desc { "normalizing `{:?}`", goal.value.value.value }
    }

    /// Do not call this query directly: part of the `Normalize` type-op
    query type_op_normalize_poly_fn_sig(
        goal: CanonicalTypeOpNormalizeGoal<'tcx, ty::PolyFnSig<'tcx>>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, ty::PolyFnSig<'tcx>>>,
        NoSolution,
    > {
        desc { "normalizing `{:?}`", goal.value.value.value }
    }

    /// Do not call this query directly: part of the `Normalize` type-op
    query type_op_normalize_fn_sig(
        goal: CanonicalTypeOpNormalizeGoal<'tcx, ty::FnSig<'tcx>>
    ) -> Result<
        &'tcx Canonical<'tcx, canonical::QueryResponse<'tcx, ty::FnSig<'tcx>>>,
        NoSolution,
    > {
        desc { "normalizing `{:?}`", goal.value.value.value }
    }

    query subst_and_check_impossible_predicates(key: (DefId, GenericArgsRef<'tcx>)) -> bool {
        desc { |tcx|
            "checking impossible substituted predicates: `{}`",
            tcx.def_path_str(key.0)
        }
    }

    query is_impossible_associated_item(key: (DefId, DefId)) -> bool {
        desc { |tcx|
            "checking if `{}` is impossible to reference within `{}`",
            tcx.def_path_str(key.1),
            tcx.def_path_str(key.0),
        }
    }

    query method_autoderef_steps(
        goal: CanonicalTyGoal<'tcx>
    ) -> MethodAutoderefStepsResult<'tcx> {
        desc { "computing autoderef types for `{}`", goal.value.value }
    }

    query supported_target_features(_: CrateNum) -> &'tcx FxHashMap<String, Option<Symbol>> {
        arena_cache
        eval_always
        desc { "looking up supported target features" }
    }

    query features_query(_: ()) -> &'tcx rustc_feature::Features {
        feedable
        desc { "looking up enabled feature gates" }
    }

    query metadata_loader((): ()) -> &'tcx Steal<Box<rustc_session::cstore::MetadataLoaderDyn>> {
        feedable
        no_hash
        desc { "raw operations for metadata file access" }
    }

    query crate_for_resolver((): ()) -> &'tcx Steal<(rustc_ast::Crate, rustc_ast::AttrVec)> {
        feedable
        no_hash
        desc { "the ast before macro expansion and name resolution" }
    }

    /// Attempt to resolve the given `DefId` to an `Instance`, for the
    /// given generics args (`GenericArgsRef`), returning one of:
    ///  * `Ok(Some(instance))` on success
    ///  * `Ok(None)` when the `GenericArgsRef` are still too generic,
    ///    and therefore don't allow finding the final `Instance`
    ///  * `Err(ErrorGuaranteed)` when the `Instance` resolution process
    ///    couldn't complete due to errors elsewhere - this is distinct
    ///    from `Ok(None)` to avoid misleading diagnostics when an error
    ///    has already been/will be emitted, for the original cause
    query resolve_instance(
        key: ty::ParamEnvAnd<'tcx, (DefId, GenericArgsRef<'tcx>)>
    ) -> Result<Option<ty::Instance<'tcx>>, ErrorGuaranteed> {
        desc { "resolving instance `{}`", ty::Instance::new(key.value.0, key.value.1) }
    }

    query reveal_opaque_types_in_bounds(key: &'tcx ty::List<ty::Clause<'tcx>>) -> &'tcx ty::List<ty::Clause<'tcx>> {
        desc { "revealing opaque types in `{:?}`", key }
    }

    query limits(key: ()) -> Limits {
        desc { "looking up limits" }
    }

    /// Performs an HIR-based well-formed check on the item with the given `HirId`. If
    /// we get an `Unimplemented` error that matches the provided `Predicate`, return
    /// the cause of the newly created obligation.
    ///
    /// This is only used by error-reporting code to get a better cause (in particular, a better
    /// span) for an *existing* error. Therefore, it is best-effort, and may never handle
    /// all of the cases that the normal `ty::Ty`-based wfcheck does. This is fine,
    /// because the `ty::Ty`-based wfcheck is always run.
    query diagnostic_hir_wf_check(
        key: (ty::Predicate<'tcx>, WellFormedLoc)
    ) -> &'tcx Option<ObligationCause<'tcx>> {
        arena_cache
        eval_always
        no_hash
        desc { "performing HIR wf-checking for predicate `{:?}` at item `{:?}`", key.0, key.1 }
    }

    /// The list of backend features computed from CLI flags (`-Ctarget-cpu`, `-Ctarget-feature`,
    /// `--target` and similar).
    query global_backend_features(_: ()) -> &'tcx Vec<String> {
        arena_cache
        eval_always
        desc { "computing the backend features for CLI flags" }
    }

    query generator_diagnostic_data(key: DefId) -> &'tcx Option<GeneratorDiagnosticData<'tcx>> {
        arena_cache
        desc { |tcx| "looking up generator diagnostic data of `{}`", tcx.def_path_str(key) }
        separate_provide_extern
    }

    query check_validity_requirement(key: (ValidityRequirement, ty::ParamEnvAnd<'tcx, Ty<'tcx>>)) -> Result<bool, &'tcx ty::layout::LayoutError<'tcx>> {
        desc { "checking validity requirement for `{}`: {}", key.1.value, key.0 }
    }

    query compare_impl_const(
        key: (LocalDefId, DefId)
    ) -> Result<(), ErrorGuaranteed> {
        desc { |tcx| "checking assoc const `{}` has the same type as trait item", tcx.def_path_str(key.0) }
    }

    query deduced_param_attrs(def_id: DefId) -> &'tcx [ty::DeducedParamAttrs] {
        desc { |tcx| "deducing parameter attributes for {}", tcx.def_path_str(def_id) }
        separate_provide_extern
    }

    query doc_link_resolutions(def_id: DefId) -> &'tcx DocLinkResMap {
        eval_always
        desc { "resolutions for documentation links for a module" }
        separate_provide_extern
    }

    query doc_link_traits_in_scope(def_id: DefId) -> &'tcx [DefId] {
        eval_always
        desc { "traits in scope for documentation links for a module" }
        separate_provide_extern
    }

    /// Used in `super_combine_consts` to ICE if the type of the two consts are definitely not going to end up being
    /// equal to eachother. This might return `Ok` even if the types are not equal, but will never return `Err` if
    /// the types might be equal.
    query check_tys_might_be_eq(arg: Canonical<'tcx, (ty::ParamEnv<'tcx>, Ty<'tcx>, Ty<'tcx>)>) -> Result<(), NoSolution> {
        desc { "check whether two const param are definitely not equal to eachother"}
    }

    /// Get all item paths that were stripped by a `#[cfg]` in a particular crate.
    /// Should not be called for the local crate before the resolver outputs are created, as it
    /// is only fed there.
    query stripped_cfg_items(cnum: CrateNum) -> &'tcx [StrippedCfgItem] {
        feedable
        desc { "getting cfg-ed out item names" }
        separate_provide_extern
    }

    query generics_require_sized_self(def_id: DefId) -> bool {
        desc { "check whether the item has a `where Self: Sized` bound" }
    }
}

rustc_query_append! { define_callbacks! }
rustc_feedable_queries! { define_feedable! }
