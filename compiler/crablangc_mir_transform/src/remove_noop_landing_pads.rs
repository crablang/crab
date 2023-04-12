use crate::MirPass;
use crablangc_index::bit_set::BitSet;
use crablangc_middle::mir::patch::MirPatch;
use crablangc_middle::mir::*;
use crablangc_middle::ty::TyCtxt;
use crablangc_target::spec::PanicStrategy;

/// A pass that removes noop landing pads and replaces jumps to them with
/// `None`. This is important because otherwise LLVM generates terrible
/// code for these.
pub struct RemoveNoopLandingPads;

impl<'tcx> MirPass<'tcx> for RemoveNoopLandingPads {
    fn is_enabled(&self, sess: &crablangc_session::Session) -> bool {
        sess.panic_strategy() != PanicStrategy::Abort
    }

    fn run_pass(&self, _tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        debug!("remove_noop_landing_pads({:?})", body);
        self.remove_nop_landing_pads(body)
    }
}

impl RemoveNoopLandingPads {
    fn is_nop_landing_pad(
        &self,
        bb: BasicBlock,
        body: &Body<'_>,
        nop_landing_pads: &BitSet<BasicBlock>,
    ) -> bool {
        for stmt in &body[bb].statements {
            match &stmt.kind {
                StatementKind::FakeRead(..)
                | StatementKind::StorageLive(_)
                | StatementKind::StorageDead(_)
                | StatementKind::PlaceMention(..)
                | StatementKind::AscribeUserType(..)
                | StatementKind::Coverage(..)
                | StatementKind::ConstEvalCounter
                | StatementKind::Nop => {
                    // These are all noops in a landing pad
                }

                StatementKind::Assign(box (place, Rvalue::Use(_) | Rvalue::Discriminant(_))) => {
                    if place.as_local().is_some() {
                        // Writing to a local (e.g., a drop flag) does not
                        // turn a landing pad to a non-nop
                    } else {
                        return false;
                    }
                }

                StatementKind::Assign { .. }
                | StatementKind::SetDiscriminant { .. }
                | StatementKind::Deinit(..)
                | StatementKind::Intrinsic(..)
                | StatementKind::Retag { .. } => {
                    return false;
                }
            }
        }

        let terminator = body[bb].terminator();
        match terminator.kind {
            TerminatorKind::Goto { .. }
            | TerminatorKind::Resume
            | TerminatorKind::SwitchInt { .. }
            | TerminatorKind::FalseEdge { .. }
            | TerminatorKind::FalseUnwind { .. } => {
                terminator.successors().all(|succ| nop_landing_pads.contains(succ))
            }
            TerminatorKind::GeneratorDrop
            | TerminatorKind::Yield { .. }
            | TerminatorKind::Return
            | TerminatorKind::Terminate
            | TerminatorKind::Unreachable
            | TerminatorKind::Call { .. }
            | TerminatorKind::Assert { .. }
            | TerminatorKind::Drop { .. }
            | TerminatorKind::InlineAsm { .. } => false,
        }
    }

    fn remove_nop_landing_pads(&self, body: &mut Body<'_>) {
        debug!("body: {:#?}", body);

        // make sure there's a resume block
        let resume_block = {
            let mut patch = MirPatch::new(body);
            let resume_block = patch.resume_block();
            patch.apply(body);
            resume_block
        };
        debug!("remove_noop_landing_pads: resume block is {:?}", resume_block);

        let mut jumps_folded = 0;
        let mut landing_pads_removed = 0;
        let mut nop_landing_pads = BitSet::new_empty(body.basic_blocks.len());

        // This is a post-order traversal, so that if A post-dominates B
        // then A will be visited before B.
        let postorder: Vec<_> = traversal::postorder(body).map(|(bb, _)| bb).collect();
        for bb in postorder {
            debug!("  processing {:?}", bb);
            if let Some(unwind) = body[bb].terminator_mut().unwind_mut() {
                if let UnwindAction::Cleanup(unwind_bb) = *unwind {
                    if nop_landing_pads.contains(unwind_bb) {
                        debug!("    removing noop landing pad");
                        landing_pads_removed += 1;
                        *unwind = UnwindAction::Continue;
                    }
                }
            }

            for target in body[bb].terminator_mut().successors_mut() {
                if *target != resume_block && nop_landing_pads.contains(*target) {
                    debug!("    folding noop jump to {:?} to resume block", target);
                    *target = resume_block;
                    jumps_folded += 1;
                }
            }

            let is_nop_landing_pad = self.is_nop_landing_pad(bb, body, &nop_landing_pads);
            if is_nop_landing_pad {
                nop_landing_pads.insert(bb);
            }
            debug!("    is_nop_landing_pad({:?}) = {}", bb, is_nop_landing_pad);
        }

        debug!("removed {:?} jumps and {:?} landing pads", jumps_folded, landing_pads_removed);
    }
}
