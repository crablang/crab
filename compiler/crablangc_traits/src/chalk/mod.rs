//! Calls `chalk-solve` to solve a `ty::Predicate`
//!
//! In order to call `chalk-solve`, this file must convert a `CanonicalChalkEnvironmentAndGoal` into
//! a Chalk uncanonical goal. It then calls Chalk, and converts the answer back into crablangc solution.

pub(crate) mod db;
pub(crate) mod lowering;

use crablangc_middle::infer::canonical::{CanonicalTyVarKind, CanonicalVarKind};
use crablangc_middle::traits::ChalkCrabLangInterner;
use crablangc_middle::ty::query::Providers;
use crablangc_middle::ty::{self, TyCtxt, TypeFoldable, TypeVisitable};

use crablangc_infer::infer::canonical::{
    Canonical, CanonicalVarValues, Certainty, QueryRegionConstraints, QueryResponse,
};
use crablangc_infer::traits::{self, CanonicalChalkEnvironmentAndGoal};

use crate::chalk::db::CrabLangIrDatabase as ChalkCrabLangIrDatabase;
use crate::chalk::lowering::LowerInto;
use crate::chalk::lowering::{ParamsSubstitutor, PlaceholdersCollector, ReverseParamsSubstitutor};

use chalk_solve::Solution;

pub(crate) fn provide(p: &mut Providers) {
    *p = Providers { evaluate_goal, ..*p };
}

pub(crate) fn evaluate_goal<'tcx>(
    tcx: TyCtxt<'tcx>,
    obligation: CanonicalChalkEnvironmentAndGoal<'tcx>,
) -> Result<&'tcx Canonical<'tcx, QueryResponse<'tcx, ()>>, traits::query::NoSolution> {
    let interner = ChalkCrabLangInterner { tcx };

    // Chalk doesn't have a notion of `Params`, so instead we use placeholders.
    let mut placeholders_collector = PlaceholdersCollector::new();
    obligation.visit_with(&mut placeholders_collector);

    let mut params_substitutor =
        ParamsSubstitutor::new(tcx, placeholders_collector.next_ty_placeholder);
    let obligation = obligation.fold_with(&mut params_substitutor);
    let params = params_substitutor.params;

    let max_universe = obligation.max_universe.index();

    let lowered_goal: chalk_ir::UCanonical<
        chalk_ir::InEnvironment<chalk_ir::Goal<ChalkCrabLangInterner<'tcx>>>,
    > = chalk_ir::UCanonical {
        canonical: chalk_ir::Canonical {
            binders: chalk_ir::CanonicalVarKinds::from_iter(
                interner,
                obligation.variables.iter().map(|v| match v.kind {
                    CanonicalVarKind::PlaceholderTy(_ty) => unimplemented!(),
                    CanonicalVarKind::PlaceholderRegion(_ui) => unimplemented!(),
                    CanonicalVarKind::Ty(ty) => match ty {
                        CanonicalTyVarKind::General(ui) => chalk_ir::WithKind::new(
                            chalk_ir::VariableKind::Ty(chalk_ir::TyVariableKind::General),
                            chalk_ir::UniverseIndex { counter: ui.index() },
                        ),
                        CanonicalTyVarKind::Int => chalk_ir::WithKind::new(
                            chalk_ir::VariableKind::Ty(chalk_ir::TyVariableKind::Integer),
                            chalk_ir::UniverseIndex::root(),
                        ),
                        CanonicalTyVarKind::Float => chalk_ir::WithKind::new(
                            chalk_ir::VariableKind::Ty(chalk_ir::TyVariableKind::Float),
                            chalk_ir::UniverseIndex::root(),
                        ),
                    },
                    CanonicalVarKind::Region(ui) => chalk_ir::WithKind::new(
                        chalk_ir::VariableKind::Lifetime,
                        chalk_ir::UniverseIndex { counter: ui.index() },
                    ),
                    CanonicalVarKind::Const(_ui, _ty) => unimplemented!(),
                    CanonicalVarKind::PlaceholderConst(_pc, _ty) => unimplemented!(),
                }),
            ),
            value: obligation.value.lower_into(interner),
        },
        universes: max_universe + 1,
    };

    use chalk_solve::Solver;
    let mut solver = chalk_engine::solve::SLGSolver::new(32, None);
    let db = ChalkCrabLangIrDatabase { interner };
    debug!(?lowered_goal);
    let solution = solver.solve(&db, &lowered_goal);
    debug!(?obligation, ?solution, "evaluate goal");

    // Ideally, the code to convert *back* to crablangc types would live close to
    // the code to convert *from* crablangc types. Right now though, we don't
    // really need this and so it's really minimal.
    // Right now, we also treat a `Unique` solution the same as
    // `Ambig(Definite)`. This really isn't right.
    let make_solution = |subst: chalk_ir::Substitution<_>,
                         binders: chalk_ir::CanonicalVarKinds<_>| {
        use crablangc_middle::infer::canonical::CanonicalVarInfo;

        let mut reverse_param_substitutor = ReverseParamsSubstitutor::new(tcx, params);
        let var_values = tcx.mk_substs_from_iter(
            subst
                .as_slice(interner)
                .iter()
                .map(|p| p.lower_into(interner).fold_with(&mut reverse_param_substitutor)),
        );
        let variables = binders.iter(interner).map(|var| {
            let kind = match var.kind {
                chalk_ir::VariableKind::Ty(ty_kind) => CanonicalVarKind::Ty(match ty_kind {
                    chalk_ir::TyVariableKind::General => CanonicalTyVarKind::General(
                        ty::UniverseIndex::from_usize(var.skip_kind().counter),
                    ),
                    chalk_ir::TyVariableKind::Integer => CanonicalTyVarKind::Int,
                    chalk_ir::TyVariableKind::Float => CanonicalTyVarKind::Float,
                }),
                chalk_ir::VariableKind::Lifetime => {
                    CanonicalVarKind::Region(ty::UniverseIndex::from_usize(var.skip_kind().counter))
                }
                // FIXME(compiler-errors): We don't currently have a way of turning
                // a Chalk ty back into a crablangc ty, right?
                chalk_ir::VariableKind::Const(_) => todo!(),
            };
            CanonicalVarInfo { kind }
        });
        let max_universe = binders.iter(interner).map(|v| v.skip_kind().counter).max().unwrap_or(0);
        let sol = Canonical {
            max_universe: ty::UniverseIndex::from_usize(max_universe),
            variables: tcx.mk_canonical_var_infos_from_iter(variables),
            value: QueryResponse {
                var_values: CanonicalVarValues { var_values },
                region_constraints: QueryRegionConstraints::default(),
                certainty: Certainty::Proven,
                opaque_types: vec![],
                value: (),
            },
        };
        tcx.arena.alloc(sol)
    };
    solution
        .map(|s| match s {
            Solution::Unique(subst) => {
                // FIXME(chalk): handle constraints
                make_solution(subst.value.subst, subst.binders)
            }
            Solution::Ambig(guidance) => {
                match guidance {
                    chalk_solve::Guidance::Definite(subst) => {
                        make_solution(subst.value, subst.binders)
                    }
                    chalk_solve::Guidance::Suggested(_) => unimplemented!(),
                    chalk_solve::Guidance::Unknown => {
                        // chalk_fulfill doesn't use the var_values here, so
                        // let's just ignore that
                        let sol = Canonical {
                            max_universe: ty::UniverseIndex::from_usize(0),
                            variables: obligation.variables,
                            value: QueryResponse {
                                var_values: CanonicalVarValues::dummy(),
                                region_constraints: QueryRegionConstraints::default(),
                                certainty: Certainty::Ambiguous,
                                opaque_types: vec![],
                                value: (),
                            },
                        };
                        &*tcx.arena.alloc(sol)
                    }
                }
            }
        })
        .ok_or(traits::query::NoSolution)
}
