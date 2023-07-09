/*!
 * Methods for the various MIR types. These are intended for use after
 * building is complete.
 */

use crate::mir::*;
use crate::ty::{self, Ty, TyCtxt};
use rustc_hir as hir;
use rustc_target::abi::{FieldIdx, VariantIdx};

#[derive(Copy, Clone, Debug, TypeFoldable, TypeVisitable)]
pub struct PlaceTy<'tcx> {
    pub ty: Ty<'tcx>,
    /// Downcast to a particular variant of an enum or a generator, if included.
    pub variant_index: Option<VariantIdx>,
}

// At least on 64 bit systems, `PlaceTy` should not be larger than two or three pointers.
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
static_assert_size!(PlaceTy<'_>, 16);

impl<'tcx> PlaceTy<'tcx> {
    #[inline]
    pub fn from_ty(ty: Ty<'tcx>) -> PlaceTy<'tcx> {
        PlaceTy { ty, variant_index: None }
    }

    /// `place_ty.field_ty(tcx, f)` computes the type at a given field
    /// of a record or enum-variant. (Most clients of `PlaceTy` can
    /// instead just extract the relevant type directly from their
    /// `PlaceElem`, but some instances of `ProjectionElem<V, T>` do
    /// not carry a `Ty` for `T`.)
    ///
    /// Note that the resulting type has not been normalized.
    #[instrument(level = "debug", skip(tcx), ret)]
    pub fn field_ty(self, tcx: TyCtxt<'tcx>, f: FieldIdx) -> Ty<'tcx> {
        match self.ty.kind() {
            ty::Adt(adt_def, substs) => {
                let variant_def = match self.variant_index {
                    None => adt_def.non_enum_variant(),
                    Some(variant_index) => {
                        assert!(adt_def.is_enum());
                        &adt_def.variant(variant_index)
                    }
                };
                let field_def = &variant_def.fields[f];
                field_def.ty(tcx, substs)
            }
            ty::Tuple(tys) => tys[f.index()],
            _ => bug!("extracting field of non-tuple non-adt: {:?}", self),
        }
    }

    /// Convenience wrapper around `projection_ty_core` for
    /// `PlaceElem`, where we can just use the `Ty` that is already
    /// stored inline on field projection elems.
    pub fn projection_ty(self, tcx: TyCtxt<'tcx>, elem: PlaceElem<'tcx>) -> PlaceTy<'tcx> {
        self.projection_ty_core(tcx, ty::ParamEnv::empty(), &elem, |_, _, ty| ty, |_, ty| ty)
    }

    /// `place_ty.projection_ty_core(tcx, elem, |...| { ... })`
    /// projects `place_ty` onto `elem`, returning the appropriate
    /// `Ty` or downcast variant corresponding to that projection.
    /// The `handle_field` callback must map a `FieldIdx` to its `Ty`,
    /// (which should be trivial when `T` = `Ty`).
    pub fn projection_ty_core<V, T>(
        self,
        tcx: TyCtxt<'tcx>,
        param_env: ty::ParamEnv<'tcx>,
        elem: &ProjectionElem<V, T>,
        mut handle_field: impl FnMut(&Self, FieldIdx, T) -> Ty<'tcx>,
        mut handle_opaque_cast: impl FnMut(&Self, T) -> Ty<'tcx>,
    ) -> PlaceTy<'tcx>
    where
        V: ::std::fmt::Debug,
        T: ::std::fmt::Debug + Copy,
    {
        if self.variant_index.is_some() && !matches!(elem, ProjectionElem::Field(..)) {
            bug!("cannot use non field projection on downcasted place")
        }
        let answer = match *elem {
            ProjectionElem::Deref => {
                let ty = self
                    .ty
                    .builtin_deref(true)
                    .unwrap_or_else(|| {
                        bug!("deref projection of non-dereferenceable ty {:?}", self)
                    })
                    .ty;
                PlaceTy::from_ty(ty)
            }
            ProjectionElem::Index(_) | ProjectionElem::ConstantIndex { .. } => {
                PlaceTy::from_ty(self.ty.builtin_index().unwrap())
            }
            ProjectionElem::Subslice { from, to, from_end } => {
                PlaceTy::from_ty(match self.ty.kind() {
                    ty::Slice(..) => self.ty,
                    ty::Array(inner, _) if !from_end => {
                        Ty::new_array(tcx, *inner, (to - from) as u64)
                    }
                    ty::Array(inner, size) if from_end => {
                        let size = size.eval_target_usize(tcx, param_env);
                        let len = size - from - to;
                        Ty::new_array(tcx, *inner, len)
                    }
                    _ => bug!("cannot subslice non-array type: `{:?}`", self),
                })
            }
            ProjectionElem::Downcast(_name, index) => {
                PlaceTy { ty: self.ty, variant_index: Some(index) }
            }
            ProjectionElem::Field(f, fty) => PlaceTy::from_ty(handle_field(&self, f, fty)),
            ProjectionElem::OpaqueCast(ty) => PlaceTy::from_ty(handle_opaque_cast(&self, ty)),
        };
        debug!("projection_ty self: {:?} elem: {:?} yields: {:?}", self, elem, answer);
        answer
    }
}

impl<'tcx> Place<'tcx> {
    pub fn ty_from<D: ?Sized>(
        local: Local,
        projection: &[PlaceElem<'tcx>],
        local_decls: &D,
        tcx: TyCtxt<'tcx>,
    ) -> PlaceTy<'tcx>
    where
        D: HasLocalDecls<'tcx>,
    {
        projection
            .iter()
            .fold(PlaceTy::from_ty(local_decls.local_decls()[local].ty), |place_ty, &elem| {
                place_ty.projection_ty(tcx, elem)
            })
    }

    pub fn ty<D: ?Sized>(&self, local_decls: &D, tcx: TyCtxt<'tcx>) -> PlaceTy<'tcx>
    where
        D: HasLocalDecls<'tcx>,
    {
        Place::ty_from(self.local, &self.projection, local_decls, tcx)
    }
}

impl<'tcx> PlaceRef<'tcx> {
    pub fn ty<D: ?Sized>(&self, local_decls: &D, tcx: TyCtxt<'tcx>) -> PlaceTy<'tcx>
    where
        D: HasLocalDecls<'tcx>,
    {
        Place::ty_from(self.local, &self.projection, local_decls, tcx)
    }
}

pub enum RvalueInitializationState {
    Shallow,
    Deep,
}

impl<'tcx> Rvalue<'tcx> {
    pub fn ty<D: ?Sized>(&self, local_decls: &D, tcx: TyCtxt<'tcx>) -> Ty<'tcx>
    where
        D: HasLocalDecls<'tcx>,
    {
        match *self {
            Rvalue::Use(ref operand) => operand.ty(local_decls, tcx),
            Rvalue::Repeat(ref operand, count) => {
                Ty::new_array_with_const_len(tcx, operand.ty(local_decls, tcx), count)
            }
            Rvalue::ThreadLocalRef(did) => tcx.thread_local_ptr_ty(did),
            Rvalue::Ref(reg, bk, ref place) => {
                let place_ty = place.ty(local_decls, tcx).ty;
                Ty::new_ref(tcx, reg, ty::TypeAndMut { ty: place_ty, mutbl: bk.to_mutbl_lossy() })
            }
            Rvalue::AddressOf(mutability, ref place) => {
                let place_ty = place.ty(local_decls, tcx).ty;
                Ty::new_ptr(tcx, ty::TypeAndMut { ty: place_ty, mutbl: mutability })
            }
            Rvalue::Len(..) => tcx.types.usize,
            Rvalue::Cast(.., ty) => ty,
            Rvalue::BinaryOp(op, box (ref lhs, ref rhs)) => {
                let lhs_ty = lhs.ty(local_decls, tcx);
                let rhs_ty = rhs.ty(local_decls, tcx);
                op.ty(tcx, lhs_ty, rhs_ty)
            }
            Rvalue::CheckedBinaryOp(op, box (ref lhs, ref rhs)) => {
                let lhs_ty = lhs.ty(local_decls, tcx);
                let rhs_ty = rhs.ty(local_decls, tcx);
                let ty = op.ty(tcx, lhs_ty, rhs_ty);
                Ty::new_tup(tcx, &[ty, tcx.types.bool])
            }
            Rvalue::UnaryOp(UnOp::Not | UnOp::Neg, ref operand) => operand.ty(local_decls, tcx),
            Rvalue::Discriminant(ref place) => place.ty(local_decls, tcx).ty.discriminant_ty(tcx),
            Rvalue::NullaryOp(NullOp::SizeOf | NullOp::AlignOf | NullOp::OffsetOf(..), _) => {
                tcx.types.usize
            }
            Rvalue::Aggregate(ref ak, ref ops) => match **ak {
                AggregateKind::Array(ty) => Ty::new_array(tcx, ty, ops.len() as u64),
                AggregateKind::Tuple => {
                    Ty::new_tup_from_iter(tcx, ops.iter().map(|op| op.ty(local_decls, tcx)))
                }
                AggregateKind::Adt(did, _, substs, _, _) => tcx.type_of(did).subst(tcx, substs),
                AggregateKind::Closure(did, substs) => Ty::new_closure(tcx, did, substs),
                AggregateKind::Generator(did, substs, movability) => {
                    Ty::new_generator(tcx, did, substs, movability)
                }
            },
            Rvalue::ShallowInitBox(_, ty) => Ty::new_box(tcx, ty),
            Rvalue::CopyForDeref(ref place) => place.ty(local_decls, tcx).ty,
        }
    }

    #[inline]
    /// Returns `true` if this rvalue is deeply initialized (most rvalues) or
    /// whether its only shallowly initialized (`Rvalue::Box`).
    pub fn initialization_state(&self) -> RvalueInitializationState {
        match *self {
            Rvalue::ShallowInitBox(_, _) => RvalueInitializationState::Shallow,
            _ => RvalueInitializationState::Deep,
        }
    }
}

impl<'tcx> Operand<'tcx> {
    pub fn ty<D: ?Sized>(&self, local_decls: &D, tcx: TyCtxt<'tcx>) -> Ty<'tcx>
    where
        D: HasLocalDecls<'tcx>,
    {
        match self {
            &Operand::Copy(ref l) | &Operand::Move(ref l) => l.ty(local_decls, tcx).ty,
            Operand::Constant(c) => c.literal.ty(),
        }
    }
}

impl<'tcx> BinOp {
    pub fn ty(&self, tcx: TyCtxt<'tcx>, lhs_ty: Ty<'tcx>, rhs_ty: Ty<'tcx>) -> Ty<'tcx> {
        // FIXME: handle SIMD correctly
        match self {
            &BinOp::Add
            | &BinOp::AddUnchecked
            | &BinOp::Sub
            | &BinOp::SubUnchecked
            | &BinOp::Mul
            | &BinOp::MulUnchecked
            | &BinOp::Div
            | &BinOp::Rem
            | &BinOp::BitXor
            | &BinOp::BitAnd
            | &BinOp::BitOr => {
                // these should be integers or floats of the same size.
                assert_eq!(lhs_ty, rhs_ty);
                lhs_ty
            }
            &BinOp::Shl
            | &BinOp::ShlUnchecked
            | &BinOp::Shr
            | &BinOp::ShrUnchecked
            | &BinOp::Offset => {
                lhs_ty // lhs_ty can be != rhs_ty
            }
            &BinOp::Eq | &BinOp::Lt | &BinOp::Le | &BinOp::Ne | &BinOp::Ge | &BinOp::Gt => {
                tcx.types.bool
            }
        }
    }
}

impl BorrowKind {
    pub fn to_mutbl_lossy(self) -> hir::Mutability {
        match self {
            BorrowKind::Mut { .. } => hir::Mutability::Mut,
            BorrowKind::Shared => hir::Mutability::Not,

            // We have no type corresponding to a shallow borrow, so use
            // `&` as an approximation.
            BorrowKind::Shallow => hir::Mutability::Not,
        }
    }
}

impl BinOp {
    pub fn to_hir_binop(self) -> hir::BinOpKind {
        match self {
            BinOp::Add => hir::BinOpKind::Add,
            BinOp::Sub => hir::BinOpKind::Sub,
            BinOp::Mul => hir::BinOpKind::Mul,
            BinOp::Div => hir::BinOpKind::Div,
            BinOp::Rem => hir::BinOpKind::Rem,
            BinOp::BitXor => hir::BinOpKind::BitXor,
            BinOp::BitAnd => hir::BinOpKind::BitAnd,
            BinOp::BitOr => hir::BinOpKind::BitOr,
            BinOp::Shl => hir::BinOpKind::Shl,
            BinOp::Shr => hir::BinOpKind::Shr,
            BinOp::Eq => hir::BinOpKind::Eq,
            BinOp::Ne => hir::BinOpKind::Ne,
            BinOp::Lt => hir::BinOpKind::Lt,
            BinOp::Gt => hir::BinOpKind::Gt,
            BinOp::Le => hir::BinOpKind::Le,
            BinOp::Ge => hir::BinOpKind::Ge,
            BinOp::AddUnchecked
            | BinOp::SubUnchecked
            | BinOp::MulUnchecked
            | BinOp::ShlUnchecked
            | BinOp::ShrUnchecked
            | BinOp::Offset => {
                unreachable!()
            }
        }
    }
}
