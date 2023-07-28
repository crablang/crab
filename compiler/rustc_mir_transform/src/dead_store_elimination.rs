//! This module implements a dead store elimination (DSE) routine.
//!
//! This transformation was written specifically for the needs of dest prop. Although it is
//! perfectly sound to use it in any context that might need it, its behavior should not be changed
//! without analyzing the interaction this will have with dest prop. Specifically, in addition to
//! the soundness of this pass in general, dest prop needs it to satisfy two additional conditions:
//!
//!  1. It's idempotent, meaning that running this pass a second time immediately after running it a
//!     first time will not cause any further changes.
//!  2. This idempotence persists across dest prop's main transform, in other words inserting any
//!     number of iterations of dest prop between the first and second application of this transform
//!     will still not cause any further changes.
//!

use rustc_index::bit_set::BitSet;
use rustc_middle::mir::visit::Visitor;
use rustc_middle::mir::*;
use rustc_middle::ty::TyCtxt;
use rustc_mir_dataflow::impls::{
    borrowed_locals, LivenessTransferFunction, MaybeTransitiveLiveLocals,
};
use rustc_mir_dataflow::Analysis;

/// Performs the optimization on the body
///
/// The `borrowed` set must be a `BitSet` of all the locals that are ever borrowed in this body. It
/// can be generated via the [`borrowed_locals`] function.
pub fn eliminate<'tcx>(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>, borrowed: &BitSet<Local>) {
    let mut live = MaybeTransitiveLiveLocals::new(borrowed)
        .into_engine(tcx, body)
        .iterate_to_fixpoint()
        .into_results_cursor(body);

    // For blocks with a call terminator, if an argument copy can be turned into a move,
    // record it as (block, argument index).
    let mut call_operands_to_move = Vec::new();
    let mut patch = Vec::new();

    for (bb, bb_data) in traversal::preorder(body) {
        if let TerminatorKind::Call { ref args, .. } = bb_data.terminator().kind {
            let loc = Location { block: bb, statement_index: bb_data.statements.len() };

            // Position ourselves between the evaluation of `args` and the write to `destination`.
            live.seek_to_block_end(bb);
            let mut state = live.get().clone();

            for (index, arg) in args.iter().enumerate().rev() {
                if let Operand::Copy(place) = *arg
                    && !place.is_indirect()
                    && !borrowed.contains(place.local)
                    && !state.contains(place.local)
                {
                    call_operands_to_move.push((bb, index));
                }

                // Account that `arg` is read from, so we don't promote another argument to a move.
                LivenessTransferFunction(&mut state).visit_operand(arg, loc);
            }
        }

        for (statement_index, statement) in bb_data.statements.iter().enumerate().rev() {
            let loc = Location { block: bb, statement_index };
            if let StatementKind::Assign(assign) = &statement.kind {
                if !assign.1.is_safe_to_remove() {
                    continue;
                }
            }
            match &statement.kind {
                StatementKind::Assign(box (place, _))
                | StatementKind::SetDiscriminant { place: box place, .. }
                | StatementKind::Deinit(box place) => {
                    if !place.is_indirect() && !borrowed.contains(place.local) {
                        live.seek_before_primary_effect(loc);
                        if !live.get().contains(place.local) {
                            patch.push(loc);
                        }
                    }
                }
                StatementKind::Retag(_, _)
                | StatementKind::StorageLive(_)
                | StatementKind::StorageDead(_)
                | StatementKind::Coverage(_)
                | StatementKind::Intrinsic(_)
                | StatementKind::ConstEvalCounter
                | StatementKind::PlaceMention(_)
                | StatementKind::Nop => (),

                StatementKind::FakeRead(_) | StatementKind::AscribeUserType(_, _) => {
                    bug!("{:?} not found in this MIR phase!", &statement.kind)
                }
            }
        }
    }

    if patch.is_empty() && call_operands_to_move.is_empty() {
        return;
    }

    let bbs = body.basic_blocks.as_mut_preserves_cfg();
    for Location { block, statement_index } in patch {
        bbs[block].statements[statement_index].make_nop();
    }
    for (block, argument_index) in call_operands_to_move {
        let TerminatorKind::Call { ref mut args, .. } = bbs[block].terminator_mut().kind else {
            bug!()
        };
        let arg = &mut args[argument_index];
        let Operand::Copy(place) = *arg else { bug!() };
        *arg = Operand::Move(place);
    }

    crate::simplify::simplify_locals(body, tcx)
}

pub struct DeadStoreElimination;

impl<'tcx> MirPass<'tcx> for DeadStoreElimination {
    fn is_enabled(&self, sess: &rustc_session::Session) -> bool {
        sess.mir_opt_level() >= 2
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let borrowed = borrowed_locals(body);
        eliminate(tcx, body, &borrowed);
    }
}
