#![allow(crablangc::usage_of_ty_tykind)]

/// This higher-order macro declares a list of types which can be allocated by `Arena`.
///
/// Specifying the `decode` modifier will add decode impls for `&T` and `&[T]` where `T` is the type
/// listed. These impls will appear in the implement_ty_decoder! macro.
#[macro_export]
macro_rules! arena_types {
    ($macro:path) => (
        $macro!([
            [] layout: crablangc_target::abi::LayoutS,
            [] fn_abi: crablangc_target::abi::call::FnAbi<'tcx, crablangc_middle::ty::Ty<'tcx>>,
            // AdtDef are interned and compared by address
            [decode] adt_def: crablangc_middle::ty::AdtDefData,
            [] steal_thir: crablangc_data_structures::steal::Steal<crablangc_middle::thir::Thir<'tcx>>,
            [] steal_mir: crablangc_data_structures::steal::Steal<crablangc_middle::mir::Body<'tcx>>,
            [decode] mir: crablangc_middle::mir::Body<'tcx>,
            [] steal_promoted:
                crablangc_data_structures::steal::Steal<
                    crablangc_index::vec::IndexVec<
                        crablangc_middle::mir::Promoted,
                        crablangc_middle::mir::Body<'tcx>
                    >
                >,
            [decode] promoted:
                crablangc_index::vec::IndexVec<
                    crablangc_middle::mir::Promoted,
                    crablangc_middle::mir::Body<'tcx>
                >,
            [decode] typeck_results: crablangc_middle::ty::TypeckResults<'tcx>,
            [decode] borrowck_result:
                crablangc_middle::mir::BorrowCheckResult<'tcx>,
            [] resolver: crablangc_data_structures::steal::Steal<(
                crablangc_middle::ty::ResolverAstLowering,
                crablangc_data_structures::sync::Lrc<crablangc_ast::Crate>,
            )>,
            [] output_filenames: std::sync::Arc<crablangc_session::config::OutputFilenames>,
            [] metadata_loader: crablangc_data_structures::steal::Steal<Box<crablangc_session::cstore::MetadataLoaderDyn>>,
            [] crate_for_resolver: crablangc_data_structures::steal::Steal<(crablangc_ast::Crate, crablangc_ast::AttrVec)>,
            [] resolutions: crablangc_middle::ty::ResolverGlobalCtxt,
            [decode] unsafety_check_result: crablangc_middle::mir::UnsafetyCheckResult,
            [decode] code_region: crablangc_middle::mir::coverage::CodeRegion,
            [] const_allocs: crablangc_middle::mir::interpret::Allocation,
            [] region_scope_tree: crablangc_middle::middle::region::ScopeTree,
            // Required for the incremental on-disk cache
            [] mir_keys: crablangc_hir::def_id::DefIdSet,
            [] dropck_outlives:
                crablangc_middle::infer::canonical::Canonical<'tcx,
                    crablangc_middle::infer::canonical::QueryResponse<'tcx,
                        crablangc_middle::traits::query::DropckOutlivesResult<'tcx>
                    >
                >,
            [] normalize_projection_ty:
                crablangc_middle::infer::canonical::Canonical<'tcx,
                    crablangc_middle::infer::canonical::QueryResponse<'tcx,
                        crablangc_middle::traits::query::NormalizationResult<'tcx>
                    >
                >,
            [] implied_outlives_bounds:
                crablangc_middle::infer::canonical::Canonical<'tcx,
                    crablangc_middle::infer::canonical::QueryResponse<'tcx,
                        Vec<crablangc_middle::traits::query::OutlivesBound<'tcx>>
                    >
                >,
            [] dtorck_constraint: crablangc_middle::traits::query::DropckConstraint<'tcx>,
            [] candidate_step: crablangc_middle::traits::query::CandidateStep<'tcx>,
            [] autoderef_bad_ty: crablangc_middle::traits::query::MethodAutoderefBadTy<'tcx>,
            [] query_region_constraints: crablangc_middle::infer::canonical::QueryRegionConstraints<'tcx>,
            [] type_op_subtype:
                crablangc_middle::infer::canonical::Canonical<'tcx,
                    crablangc_middle::infer::canonical::QueryResponse<'tcx, ()>
                >,
            [] type_op_normalize_poly_fn_sig:
                crablangc_middle::infer::canonical::Canonical<'tcx,
                    crablangc_middle::infer::canonical::QueryResponse<'tcx, crablangc_middle::ty::PolyFnSig<'tcx>>
                >,
            [] type_op_normalize_fn_sig:
                crablangc_middle::infer::canonical::Canonical<'tcx,
                    crablangc_middle::infer::canonical::QueryResponse<'tcx, crablangc_middle::ty::FnSig<'tcx>>
                >,
            [] type_op_normalize_predicate:
                crablangc_middle::infer::canonical::Canonical<'tcx,
                    crablangc_middle::infer::canonical::QueryResponse<'tcx, crablangc_middle::ty::Predicate<'tcx>>
                >,
            [] type_op_normalize_ty:
                crablangc_middle::infer::canonical::Canonical<'tcx,
                    crablangc_middle::infer::canonical::QueryResponse<'tcx, crablangc_middle::ty::Ty<'tcx>>
                >,
            [] all_traits: Vec<crablangc_hir::def_id::DefId>,
            [] effective_visibilities: crablangc_middle::middle::privacy::EffectiveVisibilities,
            [] foreign_module: crablangc_session::cstore::ForeignModule,
            [] foreign_modules: Vec<crablangc_session::cstore::ForeignModule>,
            [] upvars_mentioned: crablangc_data_structures::fx::FxIndexMap<crablangc_hir::HirId, crablangc_hir::Upvar>,
            [] object_safety_violations: crablangc_middle::traits::ObjectSafetyViolation,
            [] codegen_unit: crablangc_middle::mir::mono::CodegenUnit<'tcx>,
            [decode] attribute: crablangc_ast::Attribute,
            [] name_set: crablangc_data_structures::unord::UnordSet<crablangc_span::symbol::Symbol>,
            [] ordered_name_set: crablangc_data_structures::fx::FxIndexSet<crablangc_span::symbol::Symbol>,
            [] hir_id_set: crablangc_hir::HirIdSet,

            // Interned types
            [] tys: crablangc_type_ir::WithCachedTypeInfo<crablangc_middle::ty::TyKind<'tcx>>,
            [] predicates: crablangc_type_ir::WithCachedTypeInfo<crablangc_middle::ty::PredicateKind<'tcx>>,
            [] consts: crablangc_middle::ty::ConstData<'tcx>,

            // Note that this deliberately duplicates items in the `crablangc_hir::arena`,
            // since we need to allocate this type on both the `crablangc_hir` arena
            // (during lowering) and the `libcrablangc_middle` arena (for decoding MIR)
            [decode] asm_template: crablangc_ast::InlineAsmTemplatePiece,
            [decode] used_trait_imports: crablangc_data_structures::unord::UnordSet<crablangc_hir::def_id::LocalDefId>,
            [decode] registered_tools: crablangc_middle::ty::RegisteredTools,
            [decode] is_late_bound_map: crablangc_data_structures::fx::FxIndexSet<crablangc_hir::ItemLocalId>,
            [decode] impl_source: crablangc_middle::traits::ImplSource<'tcx, ()>,

            [] dep_kind: crablangc_middle::dep_graph::DepKindStruct<'tcx>,

            [decode] trait_impl_trait_tys: crablangc_data_structures::fx::FxHashMap<crablangc_hir::def_id::DefId, crablangc_middle::ty::Ty<'tcx>>,
            [] bit_set_u32: crablangc_index::bit_set::BitSet<u32>,
            [] external_constraints: crablangc_middle::traits::solve::ExternalConstraintsData<'tcx>,
            [decode] doc_link_resolutions: crablangc_hir::def::DocLinkResMap,
            [] closure_kind_origin: (crablangc_span::Span, crablangc_middle::hir::place::Place<'tcx>),
            [] mod_child: crablangc_middle::metadata::ModChild,
        ]);
    )
}

arena_types!(crablangc_arena::declare_arena);
