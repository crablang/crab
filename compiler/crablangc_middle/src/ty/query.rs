#![allow(unused_parens)]

use crate::dep_graph;
use crate::infer::canonical::{self, Canonical};
use crate::lint::LintExpectation;
use crate::metadata::ModChild;
use crate::middle::codegen_fn_attrs::CodegenFnAttrs;
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
use crate::query::{AsLocalKey, Key};
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
use crate::traits::{self, ImplSource};
use crate::ty::context::TyCtxtFeed;
use crate::ty::fast_reject::SimplifiedType;
use crate::ty::layout::ValidityRequirement;
use crate::ty::subst::{GenericArg, SubstsRef};
use crate::ty::util::AlwaysRequiresDrop;
use crate::ty::GeneratorDiagnosticData;
use crate::ty::{self, CrateInherentImpls, ParamEnvAnd, Ty, TyCtxt, UnusedGenericParams};
use crablangc_arena::TypedArena;
use crablangc_ast as ast;
use crablangc_ast::expand::allocator::AllocatorKind;
use crablangc_attr as attr;
use crablangc_data_structures::fx::{FxHashMap, FxIndexMap, FxIndexSet};
use crablangc_data_structures::steal::Steal;
use crablangc_data_structures::svh::Svh;
use crablangc_data_structures::sync::Lrc;
use crablangc_data_structures::sync::WorkerLocal;
use crablangc_data_structures::unord::UnordSet;
use crablangc_errors::ErrorGuaranteed;
use crablangc_hir as hir;
use crablangc_hir::def::{DefKind, DocLinkResMap};
use crablangc_hir::def_id::{
    CrateNum, DefId, DefIdMap, DefIdSet, LocalDefId, LocalDefIdMap, LocalDefIdSet,
};
use crablangc_hir::hir_id::OwnerId;
use crablangc_hir::lang_items::{LangItem, LanguageItems};
use crablangc_hir::{Crate, ItemLocalId, TraitCandidate};
use crablangc_index::vec::IndexVec;
pub(crate) use crablangc_query_system::query::QueryJobId;
use crablangc_query_system::query::*;
use crablangc_session::config::{EntryFnType, OptLevel, OutputFilenames, SymbolManglingVersion};
use crablangc_session::cstore::{CrateDepKind, CrateSource};
use crablangc_session::cstore::{ExternCrate, ForeignModule, LinkagePreference, NativeLib};
use crablangc_session::lint::LintExpectationId;
use crablangc_session::Limits;
use crablangc_span::symbol::Symbol;
use crablangc_span::{Span, DUMMY_SP};
use crablangc_target::abi;
use crablangc_target::spec::PanicStrategy;

use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Default)]
pub struct QuerySystem<'tcx> {
    pub arenas: QueryArenas<'tcx>,
    pub caches: QueryCaches<'tcx>,
    // Since we erase query value types we tell the typesystem about them with `PhantomData`.
    _phantom_values: QueryPhantomValues<'tcx>,
}

#[derive(Copy, Clone)]
pub struct TyCtxtAt<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub span: Span,
}

impl<'tcx> Deref for TyCtxtAt<'tcx> {
    type Target = TyCtxt<'tcx>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.tcx
    }
}

#[derive(Copy, Clone)]
pub struct TyCtxtEnsure<'tcx> {
    pub tcx: TyCtxt<'tcx>,
}

#[derive(Copy, Clone)]
pub struct TyCtxtEnsureWithValue<'tcx> {
    pub tcx: TyCtxt<'tcx>,
}

impl<'tcx> TyCtxt<'tcx> {
    /// Returns a transparent wrapper for `TyCtxt`, which ensures queries
    /// are executed instead of just returning their results.
    #[inline(always)]
    pub fn ensure(self) -> TyCtxtEnsure<'tcx> {
        TyCtxtEnsure { tcx: self }
    }

    /// Returns a transparent wrapper for `TyCtxt`, which ensures queries
    /// are executed instead of just returning their results.
    ///
    /// This version verifies that the computed result exists in the cache before returning.
    #[inline(always)]
    pub fn ensure_with_value(self) -> TyCtxtEnsureWithValue<'tcx> {
        TyCtxtEnsureWithValue { tcx: self }
    }

    /// Returns a transparent wrapper for `TyCtxt` which uses
    /// `span` as the location of queries performed through it.
    #[inline(always)]
    pub fn at(self, span: Span) -> TyCtxtAt<'tcx> {
        TyCtxtAt { tcx: self, span }
    }

    pub fn try_mark_green(self, dep_node: &dep_graph::DepNode) -> bool {
        self.queries.try_mark_green(self, dep_node)
    }
}

macro_rules! query_helper_param_ty {
    (DefId) => { impl IntoQueryParam<DefId> };
    (LocalDefId) => { impl IntoQueryParam<LocalDefId> };
    ($K:ty) => { $K };
}

macro_rules! query_if_arena {
    ([] $arena:tt $no_arena:tt) => {
        $no_arena
    };
    ([(arena_cache) $($rest:tt)*] $arena:tt $no_arena:tt) => {
        $arena
    };
    ([$other:tt $($modifiers:tt)*]$($args:tt)*) => {
        query_if_arena!([$($modifiers)*]$($args)*)
    };
}

/// If `separate_provide_if_extern`, then the key can be projected to its
/// local key via `<$K as AsLocalKey>::LocalKey`.
macro_rules! local_key_if_separate_extern {
    ([] $($K:tt)*) => {
        $($K)*
    };
    ([(separate_provide_extern) $($rest:tt)*] $($K:tt)*) => {
        <$($K)* as AsLocalKey>::LocalKey
    };
    ([$other:tt $($modifiers:tt)*] $($K:tt)*) => {
        local_key_if_separate_extern!([$($modifiers)*] $($K)*)
    };
}

macro_rules! separate_provide_extern_decl {
    ([][$name:ident]) => {
        ()
    };
    ([(separate_provide_extern) $($rest:tt)*][$name:ident]) => {
        for<'tcx> fn(
            TyCtxt<'tcx>,
            query_keys::$name<'tcx>,
        ) -> query_provided::$name<'tcx>
    };
    ([$other:tt $($modifiers:tt)*][$($args:tt)*]) => {
        separate_provide_extern_decl!([$($modifiers)*][$($args)*])
    };
}

macro_rules! separate_provide_extern_default {
    ([][$name:ident]) => {
        ()
    };
    ([(separate_provide_extern) $($rest:tt)*][$name:ident]) => {
        |_, key| bug!(
            "`tcx.{}({:?})` unsupported by its crate; \
             perhaps the `{}` query was never assigned a provider function",
            stringify!($name),
            key,
            stringify!($name),
        )
    };
    ([$other:tt $($modifiers:tt)*][$($args:tt)*]) => {
        separate_provide_extern_default!([$($modifiers)*][$($args)*])
    };
}

macro_rules! opt_remap_env_constness {
    ([][$name:ident]) => {};
    ([(remap_env_constness) $($rest:tt)*][$name:ident]) => {
        let $name = $name.without_const();
    };
    ([$other:tt $($modifiers:tt)*][$name:ident]) => {
        opt_remap_env_constness!([$($modifiers)*][$name])
    };
}

macro_rules! define_callbacks {
    (
     $($(#[$attr:meta])*
        [$($modifiers:tt)*] fn $name:ident($($K:tt)*) -> $V:ty,)*) => {

        // HACK(eddyb) this is like the `impl QueryConfig for queries::$name`
        // below, but using type aliases instead of associated types, to bypass
        // the limitations around normalizing under HRTB - for example, this:
        // `for<'tcx> fn(...) -> <queries::$name<'tcx> as QueryConfig<TyCtxt<'tcx>>>::Value`
        // doesn't currently normalize to `for<'tcx> fn(...) -> query_values::$name<'tcx>`.
        // This is primarily used by the `provide!` macro in `crablangc_metadata`.
        #[allow(nonstandard_style, unused_lifetimes)]
        pub mod query_keys {
            use super::*;

            $(pub type $name<'tcx> = $($K)*;)*
        }
        #[allow(nonstandard_style, unused_lifetimes)]
        pub mod query_keys_local {
            use super::*;

            $(pub type $name<'tcx> = local_key_if_separate_extern!([$($modifiers)*] $($K)*);)*
        }
        #[allow(nonstandard_style, unused_lifetimes)]
        pub mod query_values {
            use super::*;

            $(pub type $name<'tcx> = $V;)*
        }

        /// This module specifies the type returned from query providers and the type used for
        /// decoding. For regular queries this is the declared returned type `V`, but
        /// `arena_cache` will use `<V as Deref>::Target` instead.
        #[allow(nonstandard_style, unused_lifetimes)]
        pub mod query_provided {
            use super::*;

            $(
                pub type $name<'tcx> = query_if_arena!([$($modifiers)*] (<$V as Deref>::Target) ($V));
            )*
        }

        /// This module has a function per query which takes a `query_provided` value and coverts
        /// it to a regular `V` value by allocating it on an arena if the query has the
        /// `arena_cache` modifier. This will happen when computing the query using a provider or
        /// decoding a stored result.
        #[allow(nonstandard_style, unused_lifetimes)]
        pub mod query_provided_to_value {
            use super::*;

            $(
                #[inline(always)]
                pub fn $name<'tcx>(
                    _tcx: TyCtxt<'tcx>,
                    value: query_provided::$name<'tcx>,
                ) -> Erase<query_values::$name<'tcx>> {
                    erase(query_if_arena!([$($modifiers)*]
                        {
                            if mem::needs_drop::<query_provided::$name<'tcx>>() {
                                &*_tcx.query_system.arenas.$name.alloc(value)
                            } else {
                                &*_tcx.arena.dropless.alloc(value)
                            }
                        }
                        (value)
                    ))
                }
            )*
        }
        #[allow(nonstandard_style, unused_lifetimes)]
        pub mod query_storage {
            use super::*;

            $(
                pub type $name<'tcx> = <<$($K)* as Key>::CacheSelector as CacheSelector<'tcx, Erase<$V>>>::Cache;
            )*
        }

        $(
            // Ensure that keys grow no larger than 64 bytes
            #[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
            const _: () = {
                if mem::size_of::<query_keys::$name<'static>>() > 64 {
                    panic!("{}", concat!(
                        "the query `",
                        stringify!($name),
                        "` has a key type `",
                        stringify!($($K)*),
                        "` that is too large"
                    ));
                }
            };

            // Ensure that values grow no larger than 64 bytes
            #[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
            const _: () = {
                if mem::size_of::<query_values::$name<'static>>() > 64 {
                    panic!("{}", concat!(
                        "the query `",
                        stringify!($name),
                        "` has a value type `",
                        stringify!($V),
                        "` that is too large"
                    ));
                }
            };
        )*

        pub struct QueryArenas<'tcx> {
            $($(#[$attr])* pub $name: query_if_arena!([$($modifiers)*]
                (WorkerLocal<TypedArena<<$V as Deref>::Target>>)
                ()
            ),)*
        }

        impl Default for QueryArenas<'_> {
            fn default() -> Self {
                Self {
                    $($name: query_if_arena!([$($modifiers)*]
                        (WorkerLocal::new(|_| Default::default()))
                        ()
                    ),)*
                }
            }
        }

        #[derive(Default)]
        pub struct QueryPhantomValues<'tcx> {
            $($(#[$attr])* pub $name: PhantomData<query_values::$name<'tcx>>,)*
        }

        #[derive(Default)]
        pub struct QueryCaches<'tcx> {
            $($(#[$attr])* pub $name: query_storage::$name<'tcx>,)*
        }

        impl<'tcx> TyCtxtEnsure<'tcx> {
            $($(#[$attr])*
            #[inline(always)]
            pub fn $name(self, key: query_helper_param_ty!($($K)*)) {
                let key = key.into_query_param();
                opt_remap_env_constness!([$($modifiers)*][key]);

                match try_get_cached(self.tcx, &self.tcx.query_system.caches.$name, &key) {
                    Some(_) => return,
                    None => self.tcx.queries.$name(
                        self.tcx,
                        DUMMY_SP,
                        key,
                        QueryMode::Ensure { check_cache: false },
                    ),
                };
            })*
        }

        impl<'tcx> TyCtxtEnsureWithValue<'tcx> {
            $($(#[$attr])*
            #[inline(always)]
            pub fn $name(self, key: query_helper_param_ty!($($K)*)) {
                let key = key.into_query_param();
                opt_remap_env_constness!([$($modifiers)*][key]);

                match try_get_cached(self.tcx, &self.tcx.query_system.caches.$name, &key) {
                    Some(_) => return,
                    None => self.tcx.queries.$name(
                        self.tcx,
                        DUMMY_SP,
                        key,
                        QueryMode::Ensure { check_cache: true },
                    ),
                };
            })*
        }

        impl<'tcx> TyCtxt<'tcx> {
            $($(#[$attr])*
            #[inline(always)]
            #[must_use]
            pub fn $name(self, key: query_helper_param_ty!($($K)*)) -> $V
            {
                self.at(DUMMY_SP).$name(key)
            })*
        }

        impl<'tcx> TyCtxtAt<'tcx> {
            $($(#[$attr])*
            #[inline(always)]
            pub fn $name(self, key: query_helper_param_ty!($($K)*)) -> $V
            {
                let key = key.into_query_param();
                opt_remap_env_constness!([$($modifiers)*][key]);

                restore::<$V>(match try_get_cached(self.tcx, &self.tcx.query_system.caches.$name, &key) {
                    Some(value) => value,
                    None => self.tcx.queries.$name(self.tcx, self.span, key, QueryMode::Get).unwrap(),
                })
            })*
        }

        pub struct Providers {
            $(pub $name: for<'tcx> fn(
                TyCtxt<'tcx>,
                query_keys_local::$name<'tcx>,
            ) -> query_provided::$name<'tcx>,)*
        }

        pub struct ExternProviders {
            $(pub $name: separate_provide_extern_decl!([$($modifiers)*][$name]),)*
        }

        impl Default for Providers {
            fn default() -> Self {
                Providers {
                    $($name: |_, key| bug!(
                        "`tcx.{}({:?})` is not supported for this key;\n\
                        hint: Queries can be either made to the local crate, or the external crate. \
                        This error means you tried to use it for one that's not supported.\n\
                        If that's not the case, {} was likely never assigned to a provider function.\n",
                        stringify!($name),
                        key,
                        stringify!($name),
                    ),)*
                }
            }
        }

        impl Default for ExternProviders {
            fn default() -> Self {
                ExternProviders {
                    $($name: separate_provide_extern_default!([$($modifiers)*][$name]),)*
                }
            }
        }

        impl Copy for Providers {}
        impl Clone for Providers {
            fn clone(&self) -> Self { *self }
        }

        impl Copy for ExternProviders {}
        impl Clone for ExternProviders {
            fn clone(&self) -> Self { *self }
        }

        pub trait QueryEngine<'tcx>: crablangc_data_structures::sync::Sync {
            fn as_any(&'tcx self) -> &'tcx dyn std::any::Any;

            fn try_mark_green(&'tcx self, tcx: TyCtxt<'tcx>, dep_node: &dep_graph::DepNode) -> bool;

            $($(#[$attr])*
            fn $name(
                &'tcx self,
                tcx: TyCtxt<'tcx>,
                span: Span,
                key: query_keys::$name<'tcx>,
                mode: QueryMode,
            ) -> Option<Erase<$V>>;)*
        }
    };
}

macro_rules! hash_result {
    ([]) => {{
        Some(dep_graph::hash_result)
    }};
    ([(no_hash) $($rest:tt)*]) => {{
        None
    }};
    ([$other:tt $($modifiers:tt)*]) => {
        hash_result!([$($modifiers)*])
    };
}

macro_rules! define_feedable {
    ($($(#[$attr:meta])* [$($modifiers:tt)*] fn $name:ident($($K:tt)*) -> $V:ty,)*) => {
        $(impl<'tcx, K: IntoQueryParam<$($K)*> + Copy> TyCtxtFeed<'tcx, K> {
            $(#[$attr])*
            #[inline(always)]
            pub fn $name(self, value: query_provided::$name<'tcx>) -> $V {
                let key = self.key().into_query_param();
                opt_remap_env_constness!([$($modifiers)*][key]);

                let tcx = self.tcx;
                let erased = query_provided_to_value::$name(tcx, value);
                let value = restore::<$V>(erased);
                let cache = &tcx.query_system.caches.$name;

                match try_get_cached(tcx, cache, &key) {
                    Some(old) => {
                        let old = restore::<$V>(old);
                        bug!(
                            "Trying to feed an already recorded value for query {} key={key:?}:\nold value: {old:?}\nnew value: {value:?}",
                            stringify!($name),
                        )
                    }
                    None => {
                        let dep_node = dep_graph::DepNode::construct(tcx, dep_graph::DepKind::$name, &key);
                        let dep_node_index = tcx.dep_graph.with_feed_task(
                            dep_node,
                            tcx,
                            key,
                            &value,
                            hash_result!([$($modifiers)*]),
                        );
                        cache.complete(key, erased, dep_node_index);
                        value
                    }
                }
            }
        })*
    }
}

// Each of these queries corresponds to a function pointer field in the
// `Providers` struct for requesting a value of that type, and a method
// on `tcx: TyCtxt` (and `tcx.at(span)`) for doing that request in a way
// which memoizes and does dep-graph tracking, wrapping around the actual
// `Providers` that the driver creates (using several `crablangc_*` crates).
//
// The result type of each query must implement `Clone`, and additionally
// `ty::query::values::Value`, which produces an appropriate placeholder
// (error) value if the query resulted in a query cycle.
// Queries marked with `fatal_cycle` do not need the latter implementation,
// as they will raise an fatal error on query cycles instead.

crablangc_query_append! { define_callbacks! }
crablangc_feedable_queries! { define_feedable! }

mod sealed {
    use super::{DefId, LocalDefId, OwnerId};

    /// An analogue of the `Into` trait that's intended only for query parameters.
    ///
    /// This exists to allow queries to accept either `DefId` or `LocalDefId` while requiring that the
    /// user call `to_def_id` to convert between them everywhere else.
    pub trait IntoQueryParam<P> {
        fn into_query_param(self) -> P;
    }

    impl<P> IntoQueryParam<P> for P {
        #[inline(always)]
        fn into_query_param(self) -> P {
            self
        }
    }

    impl<'a, P: Copy> IntoQueryParam<P> for &'a P {
        #[inline(always)]
        fn into_query_param(self) -> P {
            *self
        }
    }

    impl IntoQueryParam<LocalDefId> for OwnerId {
        #[inline(always)]
        fn into_query_param(self) -> LocalDefId {
            self.def_id
        }
    }

    impl IntoQueryParam<DefId> for LocalDefId {
        #[inline(always)]
        fn into_query_param(self) -> DefId {
            self.to_def_id()
        }
    }

    impl IntoQueryParam<DefId> for OwnerId {
        #[inline(always)]
        fn into_query_param(self) -> DefId {
            self.to_def_id()
        }
    }
}

use sealed::IntoQueryParam;

impl<'tcx> TyCtxt<'tcx> {
    pub fn def_kind(self, def_id: impl IntoQueryParam<DefId>) -> DefKind {
        let def_id = def_id.into_query_param();
        self.opt_def_kind(def_id)
            .unwrap_or_else(|| bug!("def_kind: unsupported node: {:?}", def_id))
    }
}

impl<'tcx> TyCtxtAt<'tcx> {
    pub fn def_kind(self, def_id: impl IntoQueryParam<DefId>) -> DefKind {
        let def_id = def_id.into_query_param();
        self.opt_def_kind(def_id)
            .unwrap_or_else(|| bug!("def_kind: unsupported node: {:?}", def_id))
    }
}
