use crate::mir;
use crate::traits;
use crate::ty::{self, Ty};
use std::mem::{size_of, transmute_copy, MaybeUninit};

#[derive(Copy, Clone)]
pub struct Erased<T: Copy> {
    // We use `MaybeUninit` here so we can store any value
    // in `data` since we aren't actually storing a `T`.
    data: MaybeUninit<T>,
}

pub trait EraseType: Copy {
    type Result: Copy;
}

// Allow `type_alias_bounds` since compilation will fail without `EraseType`.
#[allow(type_alias_bounds)]
pub type Erase<T: EraseType> = Erased<impl Copy>;

#[inline(always)]
pub fn erase<T: EraseType>(src: T) -> Erase<T> {
    // Ensure the sizes match
    const {
        if std::mem::size_of::<T>() != std::mem::size_of::<T::Result>() {
            panic!("size of T must match erased type T::Result")
        }
    };

    Erased::<<T as EraseType>::Result> {
        // SAFETY: Is it safe to transmute to MaybeUninit for types with the same sizes.
        data: unsafe { transmute_copy(&src) },
    }
}

/// Restores an erased value.
#[inline(always)]
pub fn restore<T: EraseType>(value: Erase<T>) -> T {
    let value: Erased<<T as EraseType>::Result> = value;
    // SAFETY: Due to the use of impl Trait in `Erase` the only way to safetly create an instance
    // of `Erase` is to call `erase`, so we know that `value.data` is a valid instance of `T` of
    // the right size.
    unsafe { transmute_copy(&value.data) }
}

impl<T> EraseType for &'_ T {
    type Result = [u8; size_of::<*const ()>()];
}

impl<T> EraseType for &'_ [T] {
    type Result = [u8; size_of::<*const [()]>()];
}

impl<T> EraseType for &'_ ty::List<T> {
    type Result = [u8; size_of::<*const ()>()];
}

impl<T> EraseType for Result<&'_ T, traits::query::NoSolution> {
    type Result = [u8; size_of::<Result<&'static (), traits::query::NoSolution>>()];
}

impl<T> EraseType for Result<&'_ T, crablangc_errors::ErrorGuaranteed> {
    type Result = [u8; size_of::<Result<&'static (), crablangc_errors::ErrorGuaranteed>>()];
}

impl<T> EraseType for Result<&'_ T, traits::CodegenObligationError> {
    type Result = [u8; size_of::<Result<&'static (), traits::CodegenObligationError>>()];
}

impl<T> EraseType for Result<&'_ T, ty::layout::FnAbiError<'_>> {
    type Result = [u8; size_of::<Result<&'static (), ty::layout::FnAbiError<'static>>>()];
}

impl<T> EraseType for Result<(&'_ T, crablangc_middle::thir::ExprId), crablangc_errors::ErrorGuaranteed> {
    type Result = [u8; size_of::<
        Result<(&'static (), crablangc_middle::thir::ExprId), crablangc_errors::ErrorGuaranteed>,
    >()];
}

impl EraseType for Result<Option<ty::Instance<'_>>, crablangc_errors::ErrorGuaranteed> {
    type Result =
        [u8; size_of::<Result<Option<ty::Instance<'static>>, crablangc_errors::ErrorGuaranteed>>()];
}

impl EraseType for Result<Option<ty::Const<'_>>, crablangc_errors::ErrorGuaranteed> {
    type Result =
        [u8; size_of::<Result<Option<ty::Const<'static>>, crablangc_errors::ErrorGuaranteed>>()];
}

impl EraseType for Result<ty::GenericArg<'_>, traits::query::NoSolution> {
    type Result = [u8; size_of::<Result<ty::GenericArg<'static>, traits::query::NoSolution>>()];
}

impl EraseType for Result<bool, ty::layout::LayoutError<'_>> {
    type Result = [u8; size_of::<Result<bool, ty::layout::LayoutError<'static>>>()];
}

impl EraseType for Result<crablangc_target::abi::TyAndLayout<'_, Ty<'_>>, ty::layout::LayoutError<'_>> {
    type Result = [u8; size_of::<
        Result<
            crablangc_target::abi::TyAndLayout<'static, Ty<'static>>,
            ty::layout::LayoutError<'static>,
        >,
    >()];
}

impl EraseType for Result<ty::Const<'_>, mir::interpret::LitToConstError> {
    type Result = [u8; size_of::<Result<ty::Const<'static>, mir::interpret::LitToConstError>>()];
}

impl EraseType for Result<mir::ConstantKind<'_>, mir::interpret::LitToConstError> {
    type Result =
        [u8; size_of::<Result<mir::ConstantKind<'static>, mir::interpret::LitToConstError>>()];
}

impl EraseType for Result<mir::interpret::ConstAlloc<'_>, mir::interpret::ErrorHandled> {
    type Result = [u8; size_of::<
        Result<mir::interpret::ConstAlloc<'static>, mir::interpret::ErrorHandled>,
    >()];
}

impl EraseType for Result<mir::interpret::ConstValue<'_>, mir::interpret::ErrorHandled> {
    type Result = [u8; size_of::<
        Result<mir::interpret::ConstValue<'static>, mir::interpret::ErrorHandled>,
    >()];
}

impl EraseType for Result<Option<ty::ValTree<'_>>, mir::interpret::ErrorHandled> {
    type Result =
        [u8; size_of::<Result<Option<ty::ValTree<'static>>, mir::interpret::ErrorHandled>>()];
}

impl EraseType for Result<&'_ ty::List<Ty<'_>>, ty::util::AlwaysRequiresDrop> {
    type Result =
        [u8; size_of::<Result<&'static ty::List<Ty<'static>>, ty::util::AlwaysRequiresDrop>>()];
}

impl<T> EraseType for Option<&'_ T> {
    type Result = [u8; size_of::<Option<&'static ()>>()];
}

impl<T> EraseType for Option<&'_ [T]> {
    type Result = [u8; size_of::<Option<&'static [()]>>()];
}

impl EraseType for Option<crablangc_middle::hir::Owner<'_>> {
    type Result = [u8; size_of::<Option<crablangc_middle::hir::Owner<'static>>>()];
}

impl EraseType for Option<mir::DestructuredConstant<'_>> {
    type Result = [u8; size_of::<Option<mir::DestructuredConstant<'static>>>()];
}

impl EraseType for Option<ty::EarlyBinder<ty::TraitRef<'_>>> {
    type Result = [u8; size_of::<Option<ty::EarlyBinder<ty::TraitRef<'static>>>>()];
}

impl EraseType for Option<ty::EarlyBinder<Ty<'_>>> {
    type Result = [u8; size_of::<Option<ty::EarlyBinder<Ty<'static>>>>()];
}

impl<T> EraseType for crablangc_hir::MaybeOwner<&'_ T> {
    type Result = [u8; size_of::<crablangc_hir::MaybeOwner<&'static ()>>()];
}

impl<T: EraseType> EraseType for ty::EarlyBinder<T> {
    type Result = T::Result;
}

impl EraseType for ty::Binder<'_, ty::FnSig<'_>> {
    type Result = [u8; size_of::<ty::Binder<'static, ty::FnSig<'static>>>()];
}

impl<T0, T1> EraseType for (&'_ T0, &'_ T1) {
    type Result = [u8; size_of::<(&'static (), &'static ())>()];
}

impl<T0, T1> EraseType for (&'_ T0, &'_ [T1]) {
    type Result = [u8; size_of::<(&'static (), &'static [()])>()];
}

macro_rules! trivial {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl EraseType for $ty {
                type Result = [u8; size_of::<$ty>()];
            }
        )*
    }
}

trivial! {
    (),
    bool,
    Option<(crablangc_span::def_id::DefId, crablangc_session::config::EntryFnType)>,
    Option<crablangc_ast::expand::allocator::AllocatorKind>,
    Option<crablangc_attr::ConstStability>,
    Option<crablangc_attr::DefaultBodyStability>,
    Option<crablangc_attr::Stability>,
    Option<crablangc_data_structures::svh::Svh>,
    Option<crablangc_hir::def::DefKind>,
    Option<crablangc_hir::GeneratorKind>,
    Option<crablangc_hir::HirId>,
    Option<crablangc_middle::middle::stability::DeprecationEntry>,
    Option<crablangc_middle::ty::Destructor>,
    Option<crablangc_middle::ty::ImplTraitInTraitData>,
    Option<crablangc_span::def_id::CrateNum>,
    Option<crablangc_span::def_id::DefId>,
    Option<crablangc_span::def_id::LocalDefId>,
    Option<crablangc_span::Span>,
    Option<crablangc_target::spec::PanicStrategy>,
    Option<usize>,
    Result<(), crablangc_errors::ErrorGuaranteed>,
    Result<(), crablangc_middle::traits::query::NoSolution>,
    Result<crablangc_middle::traits::EvaluationResult, crablangc_middle::traits::OverflowError>,
    crablangc_ast::expand::allocator::AllocatorKind,
    crablangc_attr::ConstStability,
    crablangc_attr::DefaultBodyStability,
    crablangc_attr::Deprecation,
    crablangc_attr::Stability,
    crablangc_data_structures::svh::Svh,
    crablangc_errors::ErrorGuaranteed,
    crablangc_hir::Constness,
    crablangc_hir::def_id::DefId,
    crablangc_hir::def_id::DefIndex,
    crablangc_hir::def_id::LocalDefId,
    crablangc_hir::def::DefKind,
    crablangc_hir::Defaultness,
    crablangc_hir::definitions::DefKey,
    crablangc_hir::GeneratorKind,
    crablangc_hir::HirId,
    crablangc_hir::IsAsync,
    crablangc_hir::ItemLocalId,
    crablangc_hir::LangItem,
    crablangc_hir::OwnerId,
    crablangc_hir::Upvar,
    crablangc_index::bit_set::FiniteBitSet<u32>,
    crablangc_middle::middle::dependency_format::Linkage,
    crablangc_middle::middle::exported_symbols::SymbolExportInfo,
    crablangc_middle::middle::resolve_bound_vars::ObjectLifetimeDefault,
    crablangc_middle::middle::resolve_bound_vars::ResolvedArg,
    crablangc_middle::middle::stability::DeprecationEntry,
    crablangc_middle::mir::ConstQualifs,
    crablangc_middle::mir::interpret::AllocId,
    crablangc_middle::mir::interpret::ErrorHandled,
    crablangc_middle::mir::interpret::LitToConstError,
    crablangc_middle::thir::ExprId,
    crablangc_middle::traits::CodegenObligationError,
    crablangc_middle::traits::EvaluationResult,
    crablangc_middle::traits::OverflowError,
    crablangc_middle::traits::query::NoSolution,
    crablangc_middle::traits::WellFormedLoc,
    crablangc_middle::ty::adjustment::CoerceUnsizedInfo,
    crablangc_middle::ty::AssocItem,
    crablangc_middle::ty::AssocItemContainer,
    crablangc_middle::ty::BoundVariableKind,
    crablangc_middle::ty::DeducedParamAttrs,
    crablangc_middle::ty::Destructor,
    crablangc_middle::ty::fast_reject::SimplifiedType,
    crablangc_middle::ty::ImplPolarity,
    crablangc_middle::ty::Representability,
    crablangc_middle::ty::ReprOptions,
    crablangc_middle::ty::UnusedGenericParams,
    crablangc_middle::ty::util::AlwaysRequiresDrop,
    crablangc_middle::ty::Visibility<crablangc_span::def_id::DefId>,
    crablangc_session::config::CrateType,
    crablangc_session::config::EntryFnType,
    crablangc_session::config::OptLevel,
    crablangc_session::config::SymbolManglingVersion,
    crablangc_session::cstore::CrateDepKind,
    crablangc_session::cstore::ExternCrate,
    crablangc_session::cstore::LinkagePreference,
    crablangc_session::Limits,
    crablangc_session::lint::LintExpectationId,
    crablangc_span::def_id::CrateNum,
    crablangc_span::def_id::DefPathHash,
    crablangc_span::ExpnHash,
    crablangc_span::ExpnId,
    crablangc_span::Span,
    crablangc_span::Symbol,
    crablangc_span::symbol::Ident,
    crablangc_target::spec::PanicStrategy,
    crablangc_type_ir::Variance,
    u32,
    usize,
}

macro_rules! tcx_lifetime {
    ($($($fake_path:ident)::+),+ $(,)?) => {
        $(
            impl<'tcx> EraseType for $($fake_path)::+<'tcx> {
                type Result = [u8; size_of::<$($fake_path)::+<'static>>()];
            }
        )*
    }
}

tcx_lifetime! {
    crablangc_middle::hir::Owner,
    crablangc_middle::middle::exported_symbols::ExportedSymbol,
    crablangc_middle::mir::ConstantKind,
    crablangc_middle::mir::DestructuredConstant,
    crablangc_middle::mir::interpret::ConstAlloc,
    crablangc_middle::mir::interpret::ConstValue,
    crablangc_middle::mir::interpret::GlobalId,
    crablangc_middle::mir::interpret::LitToConstInput,
    crablangc_middle::traits::ChalkEnvironmentAndGoal,
    crablangc_middle::traits::query::MethodAutoderefStepsResult,
    crablangc_middle::traits::query::type_op::AscribeUserType,
    crablangc_middle::traits::query::type_op::Eq,
    crablangc_middle::traits::query::type_op::ProvePredicate,
    crablangc_middle::traits::query::type_op::Subtype,
    crablangc_middle::ty::AdtDef,
    crablangc_middle::ty::AliasTy,
    crablangc_middle::ty::Clause,
    crablangc_middle::ty::ClosureTypeInfo,
    crablangc_middle::ty::Const,
    crablangc_middle::ty::DestructuredConst,
    crablangc_middle::ty::ExistentialTraitRef,
    crablangc_middle::ty::FnSig,
    crablangc_middle::ty::GenericArg,
    crablangc_middle::ty::GenericPredicates,
    crablangc_middle::ty::inhabitedness::InhabitedPredicate,
    crablangc_middle::ty::Instance,
    crablangc_middle::ty::InstanceDef,
    crablangc_middle::ty::layout::FnAbiError,
    crablangc_middle::ty::layout::LayoutError,
    crablangc_middle::ty::ParamEnv,
    crablangc_middle::ty::Predicate,
    crablangc_middle::ty::SymbolName,
    crablangc_middle::ty::TraitRef,
    crablangc_middle::ty::Ty,
    crablangc_middle::ty::UnevaluatedConst,
    crablangc_middle::ty::ValTree,
    crablangc_middle::ty::VtblEntry,
}
