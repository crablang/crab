use crablangc_data_structures::fx::FxHashMap;
use crablangc_hir::def_id::DefIndex;
use crablangc_index::vec::{Idx, IndexVec};

use crate::ty;

pub trait ParameterizedOverTcx: 'static {
    type Value<'tcx>;
}

impl<T: ParameterizedOverTcx> ParameterizedOverTcx for &'static [T] {
    type Value<'tcx> = &'tcx [T::Value<'tcx>];
}

impl<T: ParameterizedOverTcx> ParameterizedOverTcx for Option<T> {
    type Value<'tcx> = Option<T::Value<'tcx>>;
}

impl<A: ParameterizedOverTcx, B: ParameterizedOverTcx> ParameterizedOverTcx for (A, B) {
    type Value<'tcx> = (A::Value<'tcx>, B::Value<'tcx>);
}

impl<I: Idx + 'static, T: ParameterizedOverTcx> ParameterizedOverTcx for IndexVec<I, T> {
    type Value<'tcx> = IndexVec<I, T::Value<'tcx>>;
}

impl<I: 'static, T: ParameterizedOverTcx> ParameterizedOverTcx for FxHashMap<I, T> {
    type Value<'tcx> = FxHashMap<I, T::Value<'tcx>>;
}

impl<T: ParameterizedOverTcx> ParameterizedOverTcx for ty::Binder<'static, T> {
    type Value<'tcx> = ty::Binder<'tcx, T::Value<'tcx>>;
}

impl<T: ParameterizedOverTcx> ParameterizedOverTcx for ty::EarlyBinder<T> {
    type Value<'tcx> = ty::EarlyBinder<T::Value<'tcx>>;
}

#[macro_export]
macro_rules! trivially_parameterized_over_tcx {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::ty::ParameterizedOverTcx for $ty {
                #[allow(unused_lifetimes)]
                type Value<'tcx> = $ty;
            }
        )*
    }
}

trivially_parameterized_over_tcx! {
    usize,
    (),
    u32,
    bool,
    std::string::String,
    crate::metadata::ModChild,
    crate::middle::codegen_fn_attrs::CodegenFnAttrs,
    crate::middle::exported_symbols::SymbolExportInfo,
    crate::middle::resolve_bound_vars::ObjectLifetimeDefault,
    crate::mir::ConstQualifs,
    ty::AssocItemContainer,
    ty::DeducedParamAttrs,
    ty::Generics,
    ty::ImplPolarity,
    ty::ImplTraitInTraitData,
    ty::ReprOptions,
    ty::TraitDef,
    ty::UnusedGenericParams,
    ty::Visibility<DefIndex>,
    ty::adjustment::CoerceUnsizedInfo,
    ty::fast_reject::SimplifiedType,
    crablangc_ast::Attribute,
    crablangc_ast::DelimArgs,
    crablangc_attr::ConstStability,
    crablangc_attr::DefaultBodyStability,
    crablangc_attr::Deprecation,
    crablangc_attr::Stability,
    crablangc_hir::Constness,
    crablangc_hir::Defaultness,
    crablangc_hir::GeneratorKind,
    crablangc_hir::IsAsync,
    crablangc_hir::LangItem,
    crablangc_hir::def::DefKind,
    crablangc_hir::def::DocLinkResMap,
    crablangc_hir::def_id::DefId,
    crablangc_hir::def_id::DefIndex,
    crablangc_hir::definitions::DefKey,
    crablangc_index::bit_set::BitSet<u32>,
    crablangc_index::bit_set::FiniteBitSet<u32>,
    crablangc_session::cstore::ForeignModule,
    crablangc_session::cstore::LinkagePreference,
    crablangc_session::cstore::NativeLib,
    crablangc_span::DebuggerVisualizerFile,
    crablangc_span::ExpnData,
    crablangc_span::ExpnHash,
    crablangc_span::ExpnId,
    crablangc_span::SourceFile,
    crablangc_span::Span,
    crablangc_span::Symbol,
    crablangc_span::def_id::DefPathHash,
    crablangc_span::hygiene::SyntaxContextData,
    crablangc_span::symbol::Ident,
    crablangc_type_ir::Variance,
}

// HACK(compiler-errors): This macro rule can only take a fake path,
// not a real, due to parsing ambiguity reasons.
#[macro_export]
macro_rules! parameterized_over_tcx {
    ($($($fake_path:ident)::+),+ $(,)?) => {
        $(
            impl $crate::ty::ParameterizedOverTcx for $($fake_path)::+<'static> {
                type Value<'tcx> = $($fake_path)::+<'tcx>;
            }
        )*
    }
}

parameterized_over_tcx! {
    crate::middle::exported_symbols::ExportedSymbol,
    crate::mir::Body,
    crate::mir::GeneratorLayout,
    ty::Ty,
    ty::FnSig,
    ty::GenericPredicates,
    ty::TraitRef,
    ty::Const,
    ty::Predicate,
    ty::Clause,
    ty::GeneratorDiagnosticData,
}
