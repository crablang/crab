//! Allocator shim
// Adapted from crablangc

use crate::prelude::*;

use crablangc_ast::expand::allocator::{AllocatorKind, AllocatorTy, ALLOCATOR_METHODS};
use crablangc_codegen_ssa::base::allocator_kind_for_codegen;
use crablangc_session::config::OomStrategy;
use crablangc_span::symbol::sym;

/// Returns whether an allocator shim was created
pub(crate) fn codegen(
    tcx: TyCtxt<'_>,
    module: &mut impl Module,
    unwind_context: &mut UnwindContext,
) -> bool {
    let Some(kind) = allocator_kind_for_codegen(tcx) else { return false };
    codegen_inner(
        module,
        unwind_context,
        kind,
        tcx.alloc_error_handler_kind(()).unwrap(),
        tcx.sess.opts.unstable_opts.oom,
    );
    true
}

fn codegen_inner(
    module: &mut impl Module,
    unwind_context: &mut UnwindContext,
    kind: AllocatorKind,
    alloc_error_handler_kind: AllocatorKind,
    oom_strategy: OomStrategy,
) {
    let usize_ty = module.target_config().pointer_type();

    for method in ALLOCATOR_METHODS {
        let mut arg_tys = Vec::with_capacity(method.inputs.len());
        for ty in method.inputs.iter() {
            match *ty {
                AllocatorTy::Layout => {
                    arg_tys.push(usize_ty); // size
                    arg_tys.push(usize_ty); // align
                }
                AllocatorTy::Ptr => arg_tys.push(usize_ty),
                AllocatorTy::Usize => arg_tys.push(usize_ty),

                AllocatorTy::ResultPtr | AllocatorTy::Unit => panic!("invalid allocator arg"),
            }
        }
        let output = match method.output {
            AllocatorTy::ResultPtr => Some(usize_ty),
            AllocatorTy::Unit => None,

            AllocatorTy::Layout | AllocatorTy::Usize | AllocatorTy::Ptr => {
                panic!("invalid allocator output")
            }
        };

        let sig = Signature {
            call_conv: module.target_config().default_call_conv,
            params: arg_tys.iter().cloned().map(AbiParam::new).collect(),
            returns: output.into_iter().map(AbiParam::new).collect(),
        };
        crate::common::create_wrapper_function(
            module,
            unwind_context,
            sig,
            &format!("__crablang_{}", method.name),
            &kind.fn_name(method.name),
        );
    }

    let sig = Signature {
        call_conv: module.target_config().default_call_conv,
        params: vec![AbiParam::new(usize_ty), AbiParam::new(usize_ty)],
        returns: vec![],
    };
    crate::common::create_wrapper_function(
        module,
        unwind_context,
        sig,
        "__crablang_alloc_error_handler",
        &alloc_error_handler_kind.fn_name(sym::oom),
    );

    let data_id = module.declare_data(OomStrategy::SYMBOL, Linkage::Export, false, false).unwrap();
    let mut data_ctx = DataContext::new();
    data_ctx.set_align(1);
    let val = oom_strategy.should_panic();
    data_ctx.define(Box::new([val]));
    module.define_data(data_id, &data_ctx).unwrap();
}
