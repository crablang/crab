//! Lowers intrinsic calls

use crate::MirPass;
use crablangc_middle::mir::*;
use crablangc_middle::ty::subst::SubstsRef;
use crablangc_middle::ty::{self, Ty, TyCtxt};
use crablangc_span::symbol::{sym, Symbol};
use crablangc_span::Span;
use crablangc_target::abi::{FieldIdx, VariantIdx};

pub struct LowerIntrinsics;

impl<'tcx> MirPass<'tcx> for LowerIntrinsics {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let local_decls = &body.local_decls;
        for block in body.basic_blocks.as_mut() {
            let terminator = block.terminator.as_mut().unwrap();
            if let TerminatorKind::Call { func, args, destination, target, .. } =
                &mut terminator.kind
            {
                let func_ty = func.ty(local_decls, tcx);
                let Some((intrinsic_name, substs)) = resolve_crablang_intrinsic(tcx, func_ty) else {
                    continue;
                };
                match intrinsic_name {
                    sym::unreachable => {
                        terminator.kind = TerminatorKind::Unreachable;
                    }
                    sym::forget => {
                        if let Some(target) = *target {
                            block.statements.push(Statement {
                                source_info: terminator.source_info,
                                kind: StatementKind::Assign(Box::new((
                                    *destination,
                                    Rvalue::Use(Operand::Constant(Box::new(Constant {
                                        span: terminator.source_info.span,
                                        user_ty: None,
                                        literal: ConstantKind::zero_sized(tcx.types.unit),
                                    }))),
                                ))),
                            });
                            terminator.kind = TerminatorKind::Goto { target };
                        }
                    }
                    sym::copy_nonoverlapping => {
                        let target = target.unwrap();
                        let mut args = args.drain(..);
                        block.statements.push(Statement {
                            source_info: terminator.source_info,
                            kind: StatementKind::Intrinsic(Box::new(
                                NonDivergingIntrinsic::CopyNonOverlapping(
                                    crablangc_middle::mir::CopyNonOverlapping {
                                        src: args.next().unwrap(),
                                        dst: args.next().unwrap(),
                                        count: args.next().unwrap(),
                                    },
                                ),
                            )),
                        });
                        assert_eq!(
                            args.next(),
                            None,
                            "Extra argument for copy_non_overlapping intrinsic"
                        );
                        drop(args);
                        terminator.kind = TerminatorKind::Goto { target };
                    }
                    sym::assume => {
                        let target = target.unwrap();
                        let mut args = args.drain(..);
                        block.statements.push(Statement {
                            source_info: terminator.source_info,
                            kind: StatementKind::Intrinsic(Box::new(
                                NonDivergingIntrinsic::Assume(args.next().unwrap()),
                            )),
                        });
                        assert_eq!(
                            args.next(),
                            None,
                            "Extra argument for copy_non_overlapping intrinsic"
                        );
                        drop(args);
                        terminator.kind = TerminatorKind::Goto { target };
                    }
                    sym::wrapping_add | sym::wrapping_sub | sym::wrapping_mul => {
                        if let Some(target) = *target {
                            let lhs;
                            let rhs;
                            {
                                let mut args = args.drain(..);
                                lhs = args.next().unwrap();
                                rhs = args.next().unwrap();
                            }
                            let bin_op = match intrinsic_name {
                                sym::wrapping_add => BinOp::Add,
                                sym::wrapping_sub => BinOp::Sub,
                                sym::wrapping_mul => BinOp::Mul,
                                _ => bug!("unexpected intrinsic"),
                            };
                            block.statements.push(Statement {
                                source_info: terminator.source_info,
                                kind: StatementKind::Assign(Box::new((
                                    *destination,
                                    Rvalue::BinaryOp(bin_op, Box::new((lhs, rhs))),
                                ))),
                            });
                            terminator.kind = TerminatorKind::Goto { target };
                        }
                    }
                    sym::add_with_overflow | sym::sub_with_overflow | sym::mul_with_overflow => {
                        if let Some(target) = *target {
                            let lhs;
                            let rhs;
                            {
                                let mut args = args.drain(..);
                                lhs = args.next().unwrap();
                                rhs = args.next().unwrap();
                            }
                            let bin_op = match intrinsic_name {
                                sym::add_with_overflow => BinOp::Add,
                                sym::sub_with_overflow => BinOp::Sub,
                                sym::mul_with_overflow => BinOp::Mul,
                                _ => bug!("unexpected intrinsic"),
                            };
                            block.statements.push(Statement {
                                source_info: terminator.source_info,
                                kind: StatementKind::Assign(Box::new((
                                    *destination,
                                    Rvalue::CheckedBinaryOp(bin_op, Box::new((lhs, rhs))),
                                ))),
                            });
                            terminator.kind = TerminatorKind::Goto { target };
                        }
                    }
                    sym::size_of | sym::min_align_of => {
                        if let Some(target) = *target {
                            let tp_ty = substs.type_at(0);
                            let null_op = match intrinsic_name {
                                sym::size_of => NullOp::SizeOf,
                                sym::min_align_of => NullOp::AlignOf,
                                _ => bug!("unexpected intrinsic"),
                            };
                            block.statements.push(Statement {
                                source_info: terminator.source_info,
                                kind: StatementKind::Assign(Box::new((
                                    *destination,
                                    Rvalue::NullaryOp(null_op, tp_ty),
                                ))),
                            });
                            terminator.kind = TerminatorKind::Goto { target };
                        }
                    }
                    sym::read_via_copy => {
                        let [arg] = args.as_slice() else {
                            span_bug!(terminator.source_info.span, "Wrong number of arguments");
                        };
                        let derefed_place =
                            if let Some(place) = arg.place() && let Some(local) = place.as_local() {
                                tcx.mk_place_deref(local.into())
                            } else {
                                span_bug!(terminator.source_info.span, "Only passing a local is supported");
                            };
                        terminator.kind = match *target {
                            None => {
                                // No target means this read something uninhabited,
                                // so it must be unreachable, and we don't need to
                                // preserve the assignment either.
                                TerminatorKind::Unreachable
                            }
                            Some(target) => {
                                block.statements.push(Statement {
                                    source_info: terminator.source_info,
                                    kind: StatementKind::Assign(Box::new((
                                        *destination,
                                        Rvalue::Use(Operand::Copy(derefed_place)),
                                    ))),
                                });
                                TerminatorKind::Goto { target }
                            }
                        }
                    }
                    sym::discriminant_value => {
                        if let (Some(target), Some(arg)) = (*target, args[0].place()) {
                            let arg = tcx.mk_place_deref(arg);
                            block.statements.push(Statement {
                                source_info: terminator.source_info,
                                kind: StatementKind::Assign(Box::new((
                                    *destination,
                                    Rvalue::Discriminant(arg),
                                ))),
                            });
                            terminator.kind = TerminatorKind::Goto { target };
                        }
                    }
                    sym::option_payload_ptr => {
                        if let (Some(target), Some(arg)) = (*target, args[0].place()) {
                            let ty::RawPtr(ty::TypeAndMut { ty: dest_ty, .. }) =
                                destination.ty(local_decls, tcx).ty.kind()
                            else { bug!(); };

                            block.statements.push(Statement {
                                source_info: terminator.source_info,
                                kind: StatementKind::Assign(Box::new((
                                    *destination,
                                    Rvalue::AddressOf(
                                        Mutability::Not,
                                        arg.project_deeper(
                                            &[
                                                PlaceElem::Deref,
                                                PlaceElem::Downcast(
                                                    Some(sym::Some),
                                                    VariantIdx::from_u32(1),
                                                ),
                                                PlaceElem::Field(FieldIdx::from_u32(0), *dest_ty),
                                            ],
                                            tcx,
                                        ),
                                    ),
                                ))),
                            });
                            terminator.kind = TerminatorKind::Goto { target };
                        }
                    }
                    sym::transmute => {
                        let dst_ty = destination.ty(local_decls, tcx).ty;
                        let Ok([arg]) = <[_; 1]>::try_from(std::mem::take(args)) else {
                            span_bug!(
                                terminator.source_info.span,
                                "Wrong number of arguments for transmute intrinsic",
                            );
                        };

                        // Always emit the cast, even if we transmute to an uninhabited type,
                        // because that lets CTFE and codegen generate better error messages
                        // when such a transmute actually ends up reachable.
                        block.statements.push(Statement {
                            source_info: terminator.source_info,
                            kind: StatementKind::Assign(Box::new((
                                *destination,
                                Rvalue::Cast(CastKind::Transmute, arg, dst_ty),
                            ))),
                        });

                        if let Some(target) = *target {
                            terminator.kind = TerminatorKind::Goto { target };
                        } else {
                            terminator.kind = TerminatorKind::Unreachable;
                        }
                    }
                    _ if intrinsic_name.as_str().starts_with("simd_shuffle") => {
                        validate_simd_shuffle(tcx, args, terminator.source_info.span);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn resolve_crablang_intrinsic<'tcx>(
    tcx: TyCtxt<'tcx>,
    func_ty: Ty<'tcx>,
) -> Option<(Symbol, SubstsRef<'tcx>)> {
    if let ty::FnDef(def_id, substs) = *func_ty.kind() {
        if tcx.is_intrinsic(def_id) {
            return Some((tcx.item_name(def_id), substs));
        }
    }
    None
}

fn validate_simd_shuffle<'tcx>(tcx: TyCtxt<'tcx>, args: &[Operand<'tcx>], span: Span) {
    match &args[2] {
        Operand::Constant(_) => {} // all good
        _ => {
            let msg = "last argument of `simd_shuffle` is required to be a `const` item";
            tcx.sess.span_err(span, msg);
        }
    }
}
