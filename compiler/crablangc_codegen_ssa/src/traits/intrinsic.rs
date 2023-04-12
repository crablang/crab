use super::BackendTypes;
use crate::mir::operand::OperandRef;
use crablangc_middle::ty::{self, Ty};
use crablangc_span::Span;
use crablangc_target::abi::call::FnAbi;

pub trait IntrinsicCallMethods<'tcx>: BackendTypes {
    /// Remember to add all intrinsics here, in `compiler/crablangc_hir_analysis/src/check/mod.rs`,
    /// and in `library/core/src/intrinsics.rs`; if you need access to any LLVM intrinsics,
    /// add them to `compiler/crablangc_codegen_llvm/src/context.rs`.
    fn codegen_intrinsic_call(
        &mut self,
        instance: ty::Instance<'tcx>,
        fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        args: &[OperandRef<'tcx, Self::Value>],
        llresult: Self::Value,
        span: Span,
    );

    fn abort(&mut self);
    fn assume(&mut self, val: Self::Value);
    fn expect(&mut self, cond: Self::Value, expected: bool) -> Self::Value;
    /// Trait method used to test whether a given pointer is associated with a type identifier.
    fn type_test(&mut self, pointer: Self::Value, typeid: Self::Value) -> Self::Value;
    /// Trait method used to load a function while testing if it is associated with a type
    /// identifier.
    fn type_checked_load(
        &mut self,
        llvtable: Self::Value,
        vtable_byte_offset: u64,
        typeid: Self::Value,
    ) -> Self::Value;
    /// Trait method used to inject `va_start` on the "spoofed" `VaListImpl` in
    /// CrabLang defined C-variadic functions.
    fn va_start(&mut self, val: Self::Value) -> Self::Value;
    /// Trait method used to inject `va_end` on the "spoofed" `VaListImpl` before
    /// CrabLang defined C-variadic functions return.
    fn va_end(&mut self, val: Self::Value) -> Self::Value;
}
