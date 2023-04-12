#![allow(non_camel_case_types)]

use crablangc_hir::LangItem;
use crablangc_middle::mir::interpret::ConstValue;
use crablangc_middle::ty::{self, layout::TyAndLayout, Ty, TyCtxt};
use crablangc_span::Span;

use crate::base;
use crate::traits::*;

#[derive(Copy, Clone)]
pub enum IntPredicate {
    IntEQ,
    IntNE,
    IntUGT,
    IntUGE,
    IntULT,
    IntULE,
    IntSGT,
    IntSGE,
    IntSLT,
    IntSLE,
}

#[derive(Copy, Clone)]
pub enum RealPredicate {
    RealPredicateFalse,
    RealOEQ,
    RealOGT,
    RealOGE,
    RealOLT,
    RealOLE,
    RealONE,
    RealORD,
    RealUNO,
    RealUEQ,
    RealUGT,
    RealUGE,
    RealULT,
    RealULE,
    RealUNE,
    RealPredicateTrue,
}

#[derive(Copy, Clone)]
pub enum AtomicRmwBinOp {
    AtomicXchg,
    AtomicAdd,
    AtomicSub,
    AtomicAnd,
    AtomicNand,
    AtomicOr,
    AtomicXor,
    AtomicMax,
    AtomicMin,
    AtomicUMax,
    AtomicUMin,
}

#[derive(Copy, Clone)]
pub enum AtomicOrdering {
    Unordered,
    Relaxed,
    Acquire,
    Release,
    AcquireRelease,
    SequentiallyConsistent,
}

#[derive(Copy, Clone)]
pub enum SynchronizationScope {
    SingleThread,
    CrossThread,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TypeKind {
    Void,
    Half,
    Float,
    Double,
    X86_FP80,
    FP128,
    PPC_FP128,
    Label,
    Integer,
    Function,
    Struct,
    Array,
    Pointer,
    Vector,
    Metadata,
    X86_MMX,
    Token,
    ScalableVector,
    BFloat,
    X86_AMX,
}

// FIXME(mw): Anything that is produced via DepGraph::with_task() must implement
//            the HashStable trait. Normally DepGraph::with_task() calls are
//            hidden behind queries, but CGU creation is a special case in two
//            ways: (1) it's not a query and (2) CGU are output nodes, so their
//            Fingerprints are not actually needed. It remains to be clarified
//            how exactly this case will be handled in the red/green system but
//            for now we content ourselves with providing a no-op HashStable
//            implementation for CGUs.
mod temp_stable_hash_impls {
    use crate::ModuleCodegen;
    use crablangc_data_structures::stable_hasher::{HashStable, StableHasher};

    impl<HCX, M> HashStable<HCX> for ModuleCodegen<M> {
        fn hash_stable(&self, _: &mut HCX, _: &mut StableHasher) {
            // do nothing
        }
    }
}

pub fn build_langcall<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>>(
    bx: &Bx,
    span: Option<Span>,
    li: LangItem,
) -> (Bx::FnAbiOfResult, Bx::Value) {
    let tcx = bx.tcx();
    let def_id = tcx.require_lang_item(li, span);
    let instance = ty::Instance::mono(tcx, def_id);
    (bx.fn_abi_of_instance(instance, ty::List::empty()), bx.get_fn_addr(instance))
}

// To avoid UB from LLVM, these two functions mask RHS with an
// appropriate mask unconditionally (i.e., the fallback behavior for
// all shifts). For 32- and 64-bit types, this matches the semantics
// of Java. (See related discussion on #1877 and #10183.)

pub fn build_unchecked_lshift<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>>(
    bx: &mut Bx,
    lhs: Bx::Value,
    rhs: Bx::Value,
) -> Bx::Value {
    let rhs = base::cast_shift_expr_rhs(bx, lhs, rhs);
    // #1877, #10183: Ensure that input is always valid
    let rhs = shift_mask_rhs(bx, rhs);
    bx.shl(lhs, rhs)
}

pub fn build_unchecked_rshift<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>>(
    bx: &mut Bx,
    lhs_t: Ty<'tcx>,
    lhs: Bx::Value,
    rhs: Bx::Value,
) -> Bx::Value {
    let rhs = base::cast_shift_expr_rhs(bx, lhs, rhs);
    // #1877, #10183: Ensure that input is always valid
    let rhs = shift_mask_rhs(bx, rhs);
    let is_signed = lhs_t.is_signed();
    if is_signed { bx.ashr(lhs, rhs) } else { bx.lshr(lhs, rhs) }
}

fn shift_mask_rhs<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>>(
    bx: &mut Bx,
    rhs: Bx::Value,
) -> Bx::Value {
    let rhs_llty = bx.val_ty(rhs);
    let shift_val = shift_mask_val(bx, rhs_llty, rhs_llty, false);
    bx.and(rhs, shift_val)
}

pub fn shift_mask_val<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>>(
    bx: &mut Bx,
    llty: Bx::Type,
    mask_llty: Bx::Type,
    invert: bool,
) -> Bx::Value {
    let kind = bx.type_kind(llty);
    match kind {
        TypeKind::Integer => {
            // i8/u8 can shift by at most 7, i16/u16 by at most 15, etc.
            let val = bx.int_width(llty) - 1;
            if invert {
                bx.const_int(mask_llty, !val as i64)
            } else {
                bx.const_uint(mask_llty, val)
            }
        }
        TypeKind::Vector => {
            let mask =
                shift_mask_val(bx, bx.element_type(llty), bx.element_type(mask_llty), invert);
            bx.vector_splat(bx.vector_length(mask_llty), mask)
        }
        _ => bug!("shift_mask_val: expected Integer or Vector, found {:?}", kind),
    }
}

pub fn asm_const_to_str<'tcx>(
    tcx: TyCtxt<'tcx>,
    sp: Span,
    const_value: ConstValue<'tcx>,
    ty_and_layout: TyAndLayout<'tcx>,
) -> String {
    let ConstValue::Scalar(scalar) = const_value else {
        span_bug!(sp, "expected Scalar for promoted asm const, but got {:#?}", const_value)
    };
    let value = scalar.assert_bits(ty_and_layout.size);
    match ty_and_layout.ty.kind() {
        ty::Uint(_) => value.to_string(),
        ty::Int(int_ty) => match int_ty.normalize(tcx.sess.target.pointer_width) {
            ty::IntTy::I8 => (value as i8).to_string(),
            ty::IntTy::I16 => (value as i16).to_string(),
            ty::IntTy::I32 => (value as i32).to_string(),
            ty::IntTy::I64 => (value as i64).to_string(),
            ty::IntTy::I128 => (value as i128).to_string(),
            ty::IntTy::Isize => unreachable!(),
        },
        _ => span_bug!(sp, "asm const has bad type {}", ty_and_layout.ty),
    }
}
