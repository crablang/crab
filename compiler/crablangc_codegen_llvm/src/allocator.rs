use crate::attributes;
use libc::c_uint;
use crablangc_ast::expand::allocator::{AllocatorKind, AllocatorTy, ALLOCATOR_METHODS};
use crablangc_middle::bug;
use crablangc_middle::ty::TyCtxt;
use crablangc_session::config::{DebugInfo, OomStrategy};
use crablangc_span::symbol::sym;

use crate::debuginfo;
use crate::llvm::{self, False, True};
use crate::ModuleLlvm;

pub(crate) unsafe fn codegen(
    tcx: TyCtxt<'_>,
    module_llvm: &mut ModuleLlvm,
    module_name: &str,
    kind: AllocatorKind,
    alloc_error_handler_kind: AllocatorKind,
) {
    let llcx = &*module_llvm.llcx;
    let llmod = module_llvm.llmod();
    let usize = match tcx.sess.target.pointer_width {
        16 => llvm::LLVMInt16TypeInContext(llcx),
        32 => llvm::LLVMInt32TypeInContext(llcx),
        64 => llvm::LLVMInt64TypeInContext(llcx),
        tws => bug!("Unsupported target word size for int: {}", tws),
    };
    let i8 = llvm::LLVMInt8TypeInContext(llcx);
    let i8p = llvm::LLVMPointerType(i8, 0);
    let void = llvm::LLVMVoidTypeInContext(llcx);

    for method in ALLOCATOR_METHODS {
        let mut args = Vec::with_capacity(method.inputs.len());
        for ty in method.inputs.iter() {
            match *ty {
                AllocatorTy::Layout => {
                    args.push(usize); // size
                    args.push(usize); // align
                }
                AllocatorTy::Ptr => args.push(i8p),
                AllocatorTy::Usize => args.push(usize),

                AllocatorTy::ResultPtr | AllocatorTy::Unit => panic!("invalid allocator arg"),
            }
        }
        let output = match method.output {
            AllocatorTy::ResultPtr => Some(i8p),
            AllocatorTy::Unit => None,

            AllocatorTy::Layout | AllocatorTy::Usize | AllocatorTy::Ptr => {
                panic!("invalid allocator output")
            }
        };
        let ty = llvm::LLVMFunctionType(
            output.unwrap_or(void),
            args.as_ptr(),
            args.len() as c_uint,
            False,
        );
        let name = format!("__crablang_{}", method.name);
        let llfn = llvm::LLVMCrabLangGetOrInsertFunction(llmod, name.as_ptr().cast(), name.len(), ty);

        if tcx.sess.target.default_hidden_visibility {
            llvm::LLVMCrabLangSetVisibility(llfn, llvm::Visibility::Hidden);
        }
        if tcx.sess.must_emit_unwind_tables() {
            let uwtable = attributes::uwtable_attr(llcx);
            attributes::apply_to_llfn(llfn, llvm::AttributePlace::Function, &[uwtable]);
        }

        let callee = kind.fn_name(method.name);
        let callee =
            llvm::LLVMCrabLangGetOrInsertFunction(llmod, callee.as_ptr().cast(), callee.len(), ty);
        llvm::LLVMCrabLangSetVisibility(callee, llvm::Visibility::Hidden);

        let llbb = llvm::LLVMAppendBasicBlockInContext(llcx, llfn, "entry\0".as_ptr().cast());

        let llbuilder = llvm::LLVMCreateBuilderInContext(llcx);
        llvm::LLVMPositionBuilderAtEnd(llbuilder, llbb);
        let args = args
            .iter()
            .enumerate()
            .map(|(i, _)| llvm::LLVMGetParam(llfn, i as c_uint))
            .collect::<Vec<_>>();
        let ret = llvm::LLVMCrabLangBuildCall(
            llbuilder,
            ty,
            callee,
            args.as_ptr(),
            args.len() as c_uint,
            [].as_ptr(),
            0 as c_uint,
        );
        llvm::LLVMSetTailCall(ret, True);
        if output.is_some() {
            llvm::LLVMBuildRet(llbuilder, ret);
        } else {
            llvm::LLVMBuildRetVoid(llbuilder);
        }
        llvm::LLVMDisposeBuilder(llbuilder);
    }

    // crablang alloc error handler
    let args = [usize, usize]; // size, align

    let ty = llvm::LLVMFunctionType(void, args.as_ptr(), args.len() as c_uint, False);
    let name = "__crablang_alloc_error_handler";
    let llfn = llvm::LLVMCrabLangGetOrInsertFunction(llmod, name.as_ptr().cast(), name.len(), ty);
    // -> ! DIFlagNoReturn
    let no_return = llvm::AttributeKind::NoReturn.create_attr(llcx);
    attributes::apply_to_llfn(llfn, llvm::AttributePlace::Function, &[no_return]);

    if tcx.sess.target.default_hidden_visibility {
        llvm::LLVMCrabLangSetVisibility(llfn, llvm::Visibility::Hidden);
    }
    if tcx.sess.must_emit_unwind_tables() {
        let uwtable = attributes::uwtable_attr(llcx);
        attributes::apply_to_llfn(llfn, llvm::AttributePlace::Function, &[uwtable]);
    }

    let callee = alloc_error_handler_kind.fn_name(sym::oom);
    let callee = llvm::LLVMCrabLangGetOrInsertFunction(llmod, callee.as_ptr().cast(), callee.len(), ty);
    // -> ! DIFlagNoReturn
    attributes::apply_to_llfn(callee, llvm::AttributePlace::Function, &[no_return]);
    llvm::LLVMCrabLangSetVisibility(callee, llvm::Visibility::Hidden);

    let llbb = llvm::LLVMAppendBasicBlockInContext(llcx, llfn, "entry\0".as_ptr().cast());

    let llbuilder = llvm::LLVMCreateBuilderInContext(llcx);
    llvm::LLVMPositionBuilderAtEnd(llbuilder, llbb);
    let args = args
        .iter()
        .enumerate()
        .map(|(i, _)| llvm::LLVMGetParam(llfn, i as c_uint))
        .collect::<Vec<_>>();
    let ret = llvm::LLVMCrabLangBuildCall(
        llbuilder,
        ty,
        callee,
        args.as_ptr(),
        args.len() as c_uint,
        [].as_ptr(),
        0 as c_uint,
    );
    llvm::LLVMSetTailCall(ret, True);
    llvm::LLVMBuildRetVoid(llbuilder);
    llvm::LLVMDisposeBuilder(llbuilder);

    // __crablang_alloc_error_handler_should_panic
    let name = OomStrategy::SYMBOL;
    let ll_g = llvm::LLVMCrabLangGetOrInsertGlobal(llmod, name.as_ptr().cast(), name.len(), i8);
    if tcx.sess.target.default_hidden_visibility {
        llvm::LLVMCrabLangSetVisibility(ll_g, llvm::Visibility::Hidden);
    }
    let val = tcx.sess.opts.unstable_opts.oom.should_panic();
    let llval = llvm::LLVMConstInt(i8, val as u64, False);
    llvm::LLVMSetInitializer(ll_g, llval);

    if tcx.sess.opts.debuginfo != DebugInfo::None {
        let dbg_cx = debuginfo::CodegenUnitDebugContext::new(llmod);
        debuginfo::metadata::build_compile_unit_di_node(tcx, module_name, &dbg_cx);
        dbg_cx.finalize(tcx.sess);
    }
}
