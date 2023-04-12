use super::*;

use crablangc_middle::mir::coverage::*;
use crablangc_middle::mir::{self, Body, Coverage, CoverageInfo};
use crablangc_middle::ty::query::Providers;
use crablangc_middle::ty::{self, TyCtxt};
use crablangc_span::def_id::DefId;

/// A `query` provider for retrieving coverage information injected into MIR.
pub(crate) fn provide(providers: &mut Providers) {
    providers.coverageinfo = |tcx, def_id| coverageinfo(tcx, def_id);
    providers.covered_code_regions = |tcx, def_id| covered_code_regions(tcx, def_id);
}

/// The `num_counters` argument to `llvm.instrprof.increment` is the max counter_id + 1, or in
/// other words, the number of counter value references injected into the MIR (plus 1 for the
/// reserved `ZERO` counter, which uses counter ID `0` when included in an expression). Injected
/// counters have a counter ID from `1..num_counters-1`.
///
/// `num_expressions` is the number of counter expressions added to the MIR body.
///
/// Both `num_counters` and `num_expressions` are used to initialize new vectors, during backend
/// code generate, to lookup counters and expressions by simple u32 indexes.
///
/// MIR optimization may split and duplicate some BasicBlock sequences, or optimize out some code
/// including injected counters. (It is OK if some counters are optimized out, but those counters
/// are still included in the total `num_counters` or `num_expressions`.) Simply counting the
/// calls may not work; but computing the number of counters or expressions by adding `1` to the
/// highest ID (for a given instrumented function) is valid.
///
/// This visitor runs twice, first with `add_missing_operands` set to `false`, to find the maximum
/// counter ID and maximum expression ID based on their enum variant `id` fields; then, as a
/// safeguard, with `add_missing_operands` set to `true`, to find any other counter or expression
/// IDs referenced by expression operands, if not already seen.
///
/// Ideally, each operand ID in a MIR `CoverageKind::Expression` will have a separate MIR `Coverage`
/// statement for the `Counter` or `Expression` with the referenced ID. but since current or future
/// MIR optimizations can theoretically optimize out segments of a MIR, it may not be possible to
/// guarantee this, so the second pass ensures the `CoverageInfo` counts include all referenced IDs.
struct CoverageVisitor {
    info: CoverageInfo,
    add_missing_operands: bool,
}

impl CoverageVisitor {
    /// Updates `num_counters` to the maximum encountered zero-based counter_id plus 1. Note the
    /// final computed number of counters should be the number of all `CoverageKind::Counter`
    /// statements in the MIR *plus one* for the implicit `ZERO` counter.
    #[inline(always)]
    fn update_num_counters(&mut self, counter_id: u32) {
        self.info.num_counters = std::cmp::max(self.info.num_counters, counter_id + 1);
    }

    /// Computes an expression index for each expression ID, and updates `num_expressions` to the
    /// maximum encountered index plus 1.
    #[inline(always)]
    fn update_num_expressions(&mut self, expression_id: u32) {
        let expression_index = u32::MAX - expression_id;
        self.info.num_expressions = std::cmp::max(self.info.num_expressions, expression_index + 1);
    }

    fn update_from_expression_operand(&mut self, operand_id: u32) {
        if operand_id >= self.info.num_counters {
            let operand_as_expression_index = u32::MAX - operand_id;
            if operand_as_expression_index >= self.info.num_expressions {
                // The operand ID is outside the known range of counter IDs and also outside the
                // known range of expression IDs. In either case, the result of a missing operand
                // (if and when used in an expression) will be zero, so from a computation
                // perspective, it doesn't matter whether it is interpreted as a counter or an
                // expression.
                //
                // However, the `num_counters` and `num_expressions` query results are used to
                // allocate arrays when generating the coverage map (during codegen), so choose
                // the type that grows either `num_counters` or `num_expressions` the least.
                if operand_id - self.info.num_counters
                    < operand_as_expression_index - self.info.num_expressions
                {
                    self.update_num_counters(operand_id)
                } else {
                    self.update_num_expressions(operand_id)
                }
            }
        }
    }

    fn visit_body(&mut self, body: &Body<'_>) {
        for bb_data in body.basic_blocks.iter() {
            for statement in bb_data.statements.iter() {
                if let StatementKind::Coverage(box ref coverage) = statement.kind {
                    if is_inlined(body, statement) {
                        continue;
                    }
                    self.visit_coverage(coverage);
                }
            }
        }
    }

    fn visit_coverage(&mut self, coverage: &Coverage) {
        if self.add_missing_operands {
            match coverage.kind {
                CoverageKind::Expression { lhs, rhs, .. } => {
                    self.update_from_expression_operand(u32::from(lhs));
                    self.update_from_expression_operand(u32::from(rhs));
                }
                _ => {}
            }
        } else {
            match coverage.kind {
                CoverageKind::Counter { id, .. } => {
                    self.update_num_counters(u32::from(id));
                }
                CoverageKind::Expression { id, .. } => {
                    self.update_num_expressions(u32::from(id));
                }
                _ => {}
            }
        }
    }
}

fn coverageinfo<'tcx>(tcx: TyCtxt<'tcx>, instance_def: ty::InstanceDef<'tcx>) -> CoverageInfo {
    let mir_body = tcx.instance_mir(instance_def);

    let mut coverage_visitor = CoverageVisitor {
        // num_counters always has at least the `ZERO` counter.
        info: CoverageInfo { num_counters: 1, num_expressions: 0 },
        add_missing_operands: false,
    };

    coverage_visitor.visit_body(mir_body);

    coverage_visitor.add_missing_operands = true;
    coverage_visitor.visit_body(mir_body);

    coverage_visitor.info
}

fn covered_code_regions(tcx: TyCtxt<'_>, def_id: DefId) -> Vec<&CodeRegion> {
    let body = mir_body(tcx, def_id);
    body.basic_blocks
        .iter()
        .flat_map(|data| {
            data.statements.iter().filter_map(|statement| match statement.kind {
                StatementKind::Coverage(box ref coverage) => {
                    if is_inlined(body, statement) {
                        None
                    } else {
                        coverage.code_region.as_ref() // may be None
                    }
                }
                _ => None,
            })
        })
        .collect()
}

fn is_inlined(body: &Body<'_>, statement: &Statement<'_>) -> bool {
    let scope_data = &body.source_scopes[statement.source_info.scope];
    scope_data.inlined.is_some() || scope_data.inlined_parent_scope.is_some()
}

/// This function ensures we obtain the correct MIR for the given item irrespective of
/// whether that means const mir or runtime mir. For `const fn` this opts for runtime
/// mir.
fn mir_body(tcx: TyCtxt<'_>, def_id: DefId) -> &mir::Body<'_> {
    let id = ty::WithOptConstParam::unknown(def_id);
    let def = ty::InstanceDef::Item(id);
    tcx.instance_mir(def)
}
