#![feature(associated_type_defaults)]
#![feature(box_patterns)]
#![feature(exact_size_is_empty)]
#![feature(let_chains)]
#![feature(min_specialization)]
#![feature(stmt_expr_attributes)]
#![feature(tcrablanged_step)]
#![recursion_limit = "256"]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate crablangc_middle;

use crablangc_ast::MetaItem;
use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_hir::def_id::DefId;
use crablangc_macros::fluent_messages;
use crablangc_middle::ty::{self, TyCtxt};
use crablangc_span::symbol::{sym, Symbol};

pub use self::drop_flag_effects::{
    drop_flag_effects_for_function_entry, drop_flag_effects_for_location,
    move_path_children_matching, on_all_children_bits, on_all_drop_children_bits,
    on_lookup_result_bits,
};
pub use self::framework::{
    fmt, graphviz, lattice, visit_results, Analysis, AnalysisDomain, Backward, CallReturnPlaces,
    Direction, Engine, Forward, GenKill, GenKillAnalysis, JoinSemiLattice, Results, ResultsCursor,
    ResultsRefCursor, ResultsVisitable, ResultsVisitor, SwitchIntEdgeEffects,
};

use self::move_paths::MoveData;

pub mod drop_flag_effects;
pub mod elaborate_drops;
mod errors;
mod framework;
pub mod impls;
pub mod move_paths;
pub mod crablangc_peek;
pub mod storage;
pub mod un_derefer;
pub mod value_analysis;

fluent_messages! { "../messages.ftl" }

pub(crate) mod indexes {
    pub(crate) use super::move_paths::MovePathIndex;
}

pub struct MoveDataParamEnv<'tcx> {
    pub move_data: MoveData<'tcx>,
    pub param_env: ty::ParamEnv<'tcx>,
}

pub fn has_crablangc_mir_with(tcx: TyCtxt<'_>, def_id: DefId, name: Symbol) -> Option<MetaItem> {
    for attr in tcx.get_attrs(def_id, sym::crablangc_mir) {
        let items = attr.meta_item_list();
        for item in items.iter().flat_map(|l| l.iter()) {
            match item.meta_item() {
                Some(mi) if mi.has_name(name) => return Some(mi.clone()),
                _ => continue,
            }
        }
    }
    None
}
