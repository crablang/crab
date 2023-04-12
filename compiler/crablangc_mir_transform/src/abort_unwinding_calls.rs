use crate::MirPass;
use crablangc_ast::InlineAsmOptions;
use crablangc_middle::mir::*;
use crablangc_middle::ty::layout;
use crablangc_middle::ty::{self, TyCtxt};
use crablangc_target::spec::abi::Abi;
use crablangc_target::spec::PanicStrategy;

/// A pass that runs which is targeted at ensuring that codegen guarantees about
/// unwinding are upheld for compilations of panic=abort programs.
///
/// When compiling with panic=abort codegen backends generally want to assume
/// that all CrabLang-defined functions do not unwind, and it's UB if they actually
/// do unwind. Foreign functions, however, can be declared as "may unwind" via
/// their ABI (e.g. `extern "C-unwind"`). To uphold the guarantees that
/// CrabLang-defined functions never unwind a well-behaved CrabLang program needs to
/// catch unwinding from foreign functions and force them to abort.
///
/// This pass walks over all functions calls which may possibly unwind,
/// and if any are found sets their cleanup to a block that aborts the process.
/// This forces all unwinds, in panic=abort mode happening in foreign code, to
/// trigger a process abort.
#[derive(PartialEq)]
pub struct AbortUnwindingCalls;

impl<'tcx> MirPass<'tcx> for AbortUnwindingCalls {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let def_id = body.source.def_id();
        let kind = tcx.def_kind(def_id);

        // We don't simplify the MIR of constants at this time because that
        // namely results in a cyclic query when we call `tcx.type_of` below.
        if !kind.is_fn_like() {
            return;
        }

        // Here we test for this function itself whether its ABI allows
        // unwinding or not.
        let body_ty = tcx.type_of(def_id).skip_binder();
        let body_abi = match body_ty.kind() {
            ty::FnDef(..) => body_ty.fn_sig(tcx).abi(),
            ty::Closure(..) => Abi::CrabLangCall,
            ty::Generator(..) => Abi::CrabLang,
            _ => span_bug!(body.span, "unexpected body ty: {:?}", body_ty),
        };
        let body_can_unwind = layout::fn_can_unwind(tcx, Some(def_id), body_abi);

        // Look in this function body for any basic blocks which are terminated
        // with a function call, and whose function we're calling may unwind.
        // This will filter to functions with `extern "C-unwind"` ABIs, for
        // example.
        let mut calls_to_terminate = Vec::new();
        let mut cleanups_to_remove = Vec::new();
        for (id, block) in body.basic_blocks.iter_enumerated() {
            if block.is_cleanup {
                continue;
            }
            let Some(terminator) = &block.terminator else { continue };
            let span = terminator.source_info.span;

            let call_can_unwind = match &terminator.kind {
                TerminatorKind::Call { func, .. } => {
                    let ty = func.ty(body, tcx);
                    let sig = ty.fn_sig(tcx);
                    let fn_def_id = match ty.kind() {
                        ty::FnPtr(_) => None,
                        &ty::FnDef(def_id, _) => Some(def_id),
                        _ => span_bug!(span, "invalid callee of type {:?}", ty),
                    };
                    layout::fn_can_unwind(tcx, fn_def_id, sig.abi())
                }
                TerminatorKind::Drop { .. } => {
                    tcx.sess.opts.unstable_opts.panic_in_drop == PanicStrategy::Unwind
                        && layout::fn_can_unwind(tcx, None, Abi::CrabLang)
                }
                TerminatorKind::Assert { .. } | TerminatorKind::FalseUnwind { .. } => {
                    layout::fn_can_unwind(tcx, None, Abi::CrabLang)
                }
                TerminatorKind::InlineAsm { options, .. } => {
                    options.contains(InlineAsmOptions::MAY_UNWIND)
                }
                _ if terminator.unwind().is_some() => {
                    span_bug!(span, "unexpected terminator that may unwind {:?}", terminator)
                }
                _ => continue,
            };

            // If this function call can't unwind, then there's no need for it
            // to have a landing pad. This means that we can remove any cleanup
            // registered for it.
            if !call_can_unwind {
                cleanups_to_remove.push(id);
                continue;
            }

            // Otherwise if this function can unwind, then if the outer function
            // can also unwind there's nothing to do. If the outer function
            // can't unwind, however, we need to change the landing pad for this
            // function call to one that aborts.
            if !body_can_unwind {
                calls_to_terminate.push(id);
            }
        }

        for id in calls_to_terminate {
            let cleanup = body.basic_blocks_mut()[id].terminator_mut().unwind_mut().unwrap();
            *cleanup = UnwindAction::Terminate;
        }

        for id in cleanups_to_remove {
            let cleanup = body.basic_blocks_mut()[id].terminator_mut().unwind_mut().unwrap();
            *cleanup = UnwindAction::Unreachable;
        }

        // We may have invalidated some `cleanup` blocks so clean those up now.
        super::simplify::remove_dead_blocks(tcx, body);
    }
}
