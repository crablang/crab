use super::BackendTypes;
use crate::mir::operand::OperandRef;
use crate::mir::place::PlaceRef;
use crablangc_ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use crablangc_hir::def_id::DefId;
use crablangc_middle::ty::Instance;
use crablangc_span::Span;
use crablangc_target::asm::InlineAsmRegOrRegClass;

#[derive(Debug)]
pub enum InlineAsmOperandRef<'tcx, B: BackendTypes + ?Sized> {
    In {
        reg: InlineAsmRegOrRegClass,
        value: OperandRef<'tcx, B::Value>,
    },
    Out {
        reg: InlineAsmRegOrRegClass,
        late: bool,
        place: Option<PlaceRef<'tcx, B::Value>>,
    },
    InOut {
        reg: InlineAsmRegOrRegClass,
        late: bool,
        in_value: OperandRef<'tcx, B::Value>,
        out_place: Option<PlaceRef<'tcx, B::Value>>,
    },
    Const {
        string: String,
    },
    SymFn {
        instance: Instance<'tcx>,
    },
    SymStatic {
        def_id: DefId,
    },
}

#[derive(Debug)]
pub enum GlobalAsmOperandRef<'tcx> {
    Const { string: String },
    SymFn { instance: Instance<'tcx> },
    SymStatic { def_id: DefId },
}

pub trait AsmBuilderMethods<'tcx>: BackendTypes {
    /// Take an inline assembly expression and splat it out via LLVM
    fn codegen_inline_asm(
        &mut self,
        template: &[InlineAsmTemplatePiece],
        operands: &[InlineAsmOperandRef<'tcx, Self>],
        options: InlineAsmOptions,
        line_spans: &[Span],
        instance: Instance<'_>,
        dest_catch_funclet: Option<(Self::BasicBlock, Self::BasicBlock, Option<&Self::Funclet>)>,
    );
}

pub trait AsmMethods<'tcx> {
    fn codegen_global_asm(
        &self,
        template: &[InlineAsmTemplatePiece],
        operands: &[GlobalAsmOperandRef<'tcx>],
        options: InlineAsmOptions,
        line_spans: &[Span],
    );
}
