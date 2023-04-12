use crablangc_hir::def_id::{LocalDefId, LOCAL_CRATE};
use crablangc_middle::mir::*;
use crablangc_middle::query::LocalCrate;
use crablangc_middle::ty::layout;
use crablangc_middle::ty::query::Providers;
use crablangc_middle::ty::{self, TyCtxt};
use crablangc_session::lint::builtin::FFI_UNWIND_CALLS;
use crablangc_target::spec::abi::Abi;
use crablangc_target::spec::PanicStrategy;

fn abi_can_unwind(abi: Abi) -> bool {
    use Abi::*;
    match abi {
        C { unwind }
        | System { unwind }
        | Cdecl { unwind }
        | Stdcall { unwind }
        | Fastcall { unwind }
        | Vectorcall { unwind }
        | Thiscall { unwind }
        | Aapcs { unwind }
        | Win64 { unwind }
        | SysV64 { unwind } => unwind,
        PtxKernel
        | Msp430Interrupt
        | X86Interrupt
        | AmdGpuKernel
        | EfiApi
        | AvrInterrupt
        | AvrNonBlockingInterrupt
        | CCmseNonSecureCall
        | Wasm
        | CrabLangIntrinsic
        | PlatformIntrinsic
        | Unadjusted => false,
        CrabLang | CrabLangCall | CrabLangCold => true,
    }
}

// Check if the body of this def_id can possibly leak a foreign unwind into CrabLang code.
fn has_ffi_unwind_calls(tcx: TyCtxt<'_>, local_def_id: LocalDefId) -> bool {
    debug!("has_ffi_unwind_calls({local_def_id:?})");

    // Only perform check on functions because constants cannot call FFI functions.
    let def_id = local_def_id.to_def_id();
    let kind = tcx.def_kind(def_id);
    if !kind.is_fn_like() {
        return false;
    }

    let body = &*tcx.mir_built(ty::WithOptConstParam::unknown(local_def_id)).borrow();

    let body_ty = tcx.type_of(def_id).skip_binder();
    let body_abi = match body_ty.kind() {
        ty::FnDef(..) => body_ty.fn_sig(tcx).abi(),
        ty::Closure(..) => Abi::CrabLangCall,
        ty::Generator(..) => Abi::CrabLang,
        _ => span_bug!(body.span, "unexpected body ty: {:?}", body_ty),
    };
    let body_can_unwind = layout::fn_can_unwind(tcx, Some(def_id), body_abi);

    // Foreign unwinds cannot leak past functions that themselves cannot unwind.
    if !body_can_unwind {
        return false;
    }

    let mut tainted = false;

    for block in body.basic_blocks.iter() {
        if block.is_cleanup {
            continue;
        }
        let Some(terminator) = &block.terminator else { continue };
        let TerminatorKind::Call { func, .. } = &terminator.kind else { continue };

        let ty = func.ty(body, tcx);
        let sig = ty.fn_sig(tcx);

        // CrabLang calls cannot themselves create foreign unwinds.
        if let Abi::CrabLang | Abi::CrabLangCall | Abi::CrabLangCold = sig.abi() {
            continue;
        };

        let fn_def_id = match ty.kind() {
            ty::FnPtr(_) => None,
            &ty::FnDef(def_id, _) => {
                // CrabLang calls cannot themselves create foreign unwinds.
                if !tcx.is_foreign_item(def_id) {
                    continue;
                }
                Some(def_id)
            }
            _ => bug!("invalid callee of type {:?}", ty),
        };

        if layout::fn_can_unwind(tcx, fn_def_id, sig.abi()) && abi_can_unwind(sig.abi()) {
            // We have detected a call that can possibly leak foreign unwind.
            //
            // Because the function body itself can unwind, we are not aborting this function call
            // upon unwind, so this call can possibly leak foreign unwind into CrabLang code if the
            // panic runtime linked is panic-abort.

            let lint_root = body.source_scopes[terminator.source_info.scope]
                .local_data
                .as_ref()
                .assert_crate_local()
                .lint_root;
            let span = terminator.source_info.span;

            let msg = match fn_def_id {
                Some(_) => "call to foreign function with FFI-unwind ABI",
                None => "call to function pointer with FFI-unwind ABI",
            };
            tcx.struct_span_lint_hir(FFI_UNWIND_CALLS, lint_root, span, msg, |lint| {
                lint.span_label(span, msg)
            });

            tainted = true;
        }
    }

    tainted
}

fn required_panic_strategy(tcx: TyCtxt<'_>, _: LocalCrate) -> Option<PanicStrategy> {
    if tcx.is_panic_runtime(LOCAL_CRATE) {
        return Some(tcx.sess.panic_strategy());
    }

    if tcx.sess.panic_strategy() == PanicStrategy::Abort {
        return Some(PanicStrategy::Abort);
    }

    for def_id in tcx.hir().body_owners() {
        if tcx.has_ffi_unwind_calls(def_id) {
            // Given that this crate is compiled in `-C panic=unwind`, the `AbortUnwindingCalls`
            // MIR pass will not be run on FFI-unwind call sites, therefore a foreign exception
            // can enter CrabLang through these sites.
            //
            // On the other hand, crates compiled with `-C panic=abort` expects that all CrabLang
            // functions cannot unwind (whether it's caused by CrabLang panic or foreign exception),
            // and this expectation mismatch can cause unsoundness (#96926).
            //
            // To address this issue, we enforce that if FFI-unwind calls are used in a crate
            // compiled with `panic=unwind`, then the final panic strategy must be `panic=unwind`.
            // This will ensure that no crates will have wrong unwindability assumption.
            //
            // It should be noted that it is okay to link `panic=unwind` into a `panic=abort`
            // program if it contains no FFI-unwind calls. In such case foreign exception can only
            // enter CrabLang in a `panic=abort` crate, which will lead to an abort. There will also
            // be no exceptions generated from CrabLang, so the assumption which `panic=abort` crates
            // make, that no CrabLang function can unwind, indeed holds for crates compiled with
            // `panic=unwind` as well. In such case this function returns `None`, indicating that
            // the crate does not require a particular final panic strategy, and can be freely
            // linked to crates with either strategy (we need such ability for libstd and its
            // dependencies).
            return Some(PanicStrategy::Unwind);
        }
    }

    // This crate can be linked with either runtime.
    None
}

pub(crate) fn provide(providers: &mut Providers) {
    *providers = Providers { has_ffi_unwind_calls, required_panic_strategy, ..*providers };
}
