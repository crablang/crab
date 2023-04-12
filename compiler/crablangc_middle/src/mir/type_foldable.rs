//! `TypeFoldable` implementations for MIR types

use crablangc_ast::InlineAsmTemplatePiece;

use super::*;
use crate::ty;

TrivialTypeTraversalAndLiftImpls! {
    BlockTailInfo,
    MirPhase,
    SourceInfo,
    FakeReadCause,
    RetagKind,
    SourceScope,
    SourceScopeLocalData,
    UserTypeAnnotationIndex,
    BorrowKind,
    CastKind,
    NullOp,
    hir::Movability,
    BasicBlock,
    SwitchTargets,
    GeneratorKind,
    GeneratorSavedLocal,
}

TrivialTypeTraversalImpls! {
    for <'tcx> {
        ConstValue<'tcx>,
    }
}

impl<'tcx> TypeFoldable<TyCtxt<'tcx>> for &'tcx [InlineAsmTemplatePiece] {
    fn try_fold_with<F: FallibleTypeFolder<TyCtxt<'tcx>>>(
        self,
        _folder: &mut F,
    ) -> Result<Self, F::Error> {
        Ok(self)
    }
}

impl<'tcx> TypeFoldable<TyCtxt<'tcx>> for &'tcx [Span] {
    fn try_fold_with<F: FallibleTypeFolder<TyCtxt<'tcx>>>(
        self,
        _folder: &mut F,
    ) -> Result<Self, F::Error> {
        Ok(self)
    }
}

impl<'tcx> TypeFoldable<TyCtxt<'tcx>> for &'tcx ty::List<PlaceElem<'tcx>> {
    fn try_fold_with<F: FallibleTypeFolder<TyCtxt<'tcx>>>(
        self,
        folder: &mut F,
    ) -> Result<Self, F::Error> {
        ty::util::fold_list(self, folder, |tcx, v| tcx.mk_place_elems(v))
    }
}
