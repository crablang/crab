//! Type-checking for the rust-intrinsic and platform-intrinsic
//! intrinsics that the compiler exposes.

use crate::errors::{
    UnrecognizedAtomicOperation, UnrecognizedIntrinsicFunction,
    WrongNumberOfGenericArgumentsToIntrinsic,
};
use crate::require_same_types;

use hir::def_id::DefId;
use rustc_errors::{struct_span_err, DiagnosticMessage};
use rustc_hir as hir;
use rustc_middle::traits::{ObligationCause, ObligationCauseCode};
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::symbol::{kw, sym, Symbol};
use rustc_target::spec::abi::Abi;

fn equate_intrinsic_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    it: &hir::ForeignItem<'_>,
    n_tps: usize,
    n_lts: usize,
    sig: ty::PolyFnSig<'tcx>,
) {
    let (own_counts, span) = match &it.kind {
        hir::ForeignItemKind::Fn(.., generics) => {
            let own_counts = tcx.generics_of(it.owner_id.to_def_id()).own_counts();
            (own_counts, generics.span)
        }
        _ => {
            struct_span_err!(tcx.sess, it.span, E0622, "intrinsic must be a function")
                .span_label(it.span, "expected a function")
                .emit();
            return;
        }
    };

    let gen_count_ok = |found: usize, expected: usize, descr: &str| -> bool {
        if found != expected {
            tcx.sess.emit_err(WrongNumberOfGenericArgumentsToIntrinsic {
                span,
                found,
                expected,
                descr,
            });
            false
        } else {
            true
        }
    };

    if gen_count_ok(own_counts.lifetimes, n_lts, "lifetime")
        && gen_count_ok(own_counts.types, n_tps, "type")
        && gen_count_ok(own_counts.consts, 0, "const")
    {
        let fty = Ty::new_fn_ptr(tcx, sig);
        let it_def_id = it.owner_id.def_id;
        let cause = ObligationCause::new(it.span, it_def_id, ObligationCauseCode::IntrinsicType);
        require_same_types(
            tcx,
            &cause,
            ty::ParamEnv::empty(), // FIXME: do all intrinsics have an empty param env?
            Ty::new_fn_ptr(tcx, tcx.fn_sig(it.owner_id).subst_identity()),
            fty,
        );
    }
}

/// Returns the unsafety of the given intrinsic.
pub fn intrinsic_operation_unsafety(tcx: TyCtxt<'_>, intrinsic_id: DefId) -> hir::Unsafety {
    let has_safe_attr = match tcx.has_attr(intrinsic_id, sym::rustc_safe_intrinsic) {
        true => hir::Unsafety::Normal,
        false => hir::Unsafety::Unsafe,
    };
    let is_in_list = match tcx.item_name(intrinsic_id) {
        // When adding a new intrinsic to this list,
        // it's usually worth updating that intrinsic's documentation
        // to note that it's safe to call, since
        // safe extern fns are otherwise unprecedented.
        sym::abort
        | sym::assert_inhabited
        | sym::assert_zero_valid
        | sym::assert_mem_uninitialized_valid
        | sym::size_of
        | sym::min_align_of
        | sym::needs_drop
        | sym::caller_location
        | sym::add_with_overflow
        | sym::sub_with_overflow
        | sym::mul_with_overflow
        | sym::wrapping_add
        | sym::wrapping_sub
        | sym::wrapping_mul
        | sym::saturating_add
        | sym::saturating_sub
        | sym::rotate_left
        | sym::rotate_right
        | sym::ctpop
        | sym::ctlz
        | sym::cttz
        | sym::bswap
        | sym::bitreverse
        | sym::discriminant_value
        | sym::type_id
        | sym::likely
        | sym::unlikely
        | sym::ptr_guaranteed_cmp
        | sym::minnumf32
        | sym::minnumf64
        | sym::maxnumf32
        | sym::rustc_peek
        | sym::maxnumf64
        | sym::type_name
        | sym::forget
        | sym::black_box
        | sym::variant_count
        | sym::ptr_mask => hir::Unsafety::Normal,
        _ => hir::Unsafety::Unsafe,
    };

    if has_safe_attr != is_in_list {
        tcx.sess.struct_span_err(
            tcx.def_span(intrinsic_id),
            DiagnosticMessage::from(format!(
                "intrinsic safety mismatch between list of intrinsics within the compiler and core library intrinsics for intrinsic `{}`",
                tcx.item_name(intrinsic_id)
            )
        )).emit();
    }

    is_in_list
}

/// Remember to add all intrinsics here, in `compiler/rustc_codegen_llvm/src/intrinsic.rs`,
/// and in `library/core/src/intrinsics.rs`.
pub fn check_intrinsic_type(tcx: TyCtxt<'_>, it: &hir::ForeignItem<'_>) {
    let param = |n| Ty::new_param(tcx, n, Symbol::intern(&format!("P{}", n)));
    let intrinsic_id = it.owner_id.to_def_id();
    let intrinsic_name = tcx.item_name(intrinsic_id);
    let name_str = intrinsic_name.as_str();

    let bound_vars = tcx.mk_bound_variable_kinds(&[
        ty::BoundVariableKind::Region(ty::BrAnon(None)),
        ty::BoundVariableKind::Region(ty::BrEnv),
    ]);
    let mk_va_list_ty = |mutbl| {
        tcx.lang_items().va_list().map(|did| {
            let region = ty::Region::new_late_bound(
                tcx,
                ty::INNERMOST,
                ty::BoundRegion { var: ty::BoundVar::from_u32(0), kind: ty::BrAnon(None) },
            );
            let env_region = ty::Region::new_late_bound(
                tcx,
                ty::INNERMOST,
                ty::BoundRegion { var: ty::BoundVar::from_u32(1), kind: ty::BrEnv },
            );
            let va_list_ty = tcx.type_of(did).subst(tcx, &[region.into()]);
            (Ty::new_ref(tcx, env_region, ty::TypeAndMut { ty: va_list_ty, mutbl }), va_list_ty)
        })
    };

    let (n_tps, n_lts, inputs, output, unsafety) = if name_str.starts_with("atomic_") {
        let split: Vec<&str> = name_str.split('_').collect();
        assert!(split.len() >= 2, "Atomic intrinsic in an incorrect format");

        //We only care about the operation here
        let (n_tps, inputs, output) = match split[1] {
            "cxchg" | "cxchgweak" => (
                1,
                vec![Ty::new_mut_ptr(tcx, param(0)), param(0), param(0)],
                Ty::new_tup(tcx, &[param(0), tcx.types.bool]),
            ),
            "load" => (1, vec![Ty::new_imm_ptr(tcx, param(0))], param(0)),
            "store" => (1, vec![Ty::new_mut_ptr(tcx, param(0)), param(0)], Ty::new_unit(tcx)),

            "xchg" | "xadd" | "xsub" | "and" | "nand" | "or" | "xor" | "max" | "min" | "umax"
            | "umin" => (1, vec![Ty::new_mut_ptr(tcx, param(0)), param(0)], param(0)),
            "fence" | "singlethreadfence" => (0, Vec::new(), Ty::new_unit(tcx)),
            op => {
                tcx.sess.emit_err(UnrecognizedAtomicOperation { span: it.span, op });
                return;
            }
        };
        (n_tps, 0, inputs, output, hir::Unsafety::Unsafe)
    } else {
        let unsafety = intrinsic_operation_unsafety(tcx, intrinsic_id);
        let (n_tps, inputs, output) = match intrinsic_name {
            sym::abort => (0, Vec::new(), tcx.types.never),
            sym::unreachable => (0, Vec::new(), tcx.types.never),
            sym::breakpoint => (0, Vec::new(), Ty::new_unit(tcx)),
            sym::size_of | sym::pref_align_of | sym::min_align_of | sym::variant_count => {
                (1, Vec::new(), tcx.types.usize)
            }
            sym::size_of_val | sym::min_align_of_val => {
                (1, vec![Ty::new_imm_ptr(tcx, param(0))], tcx.types.usize)
            }
            sym::rustc_peek => (1, vec![param(0)], param(0)),
            sym::caller_location => (0, vec![], tcx.caller_location_ty()),
            sym::assert_inhabited
            | sym::assert_zero_valid
            | sym::assert_mem_uninitialized_valid => (1, Vec::new(), Ty::new_unit(tcx)),
            sym::forget => (1, vec![param(0)], Ty::new_unit(tcx)),
            sym::transmute | sym::transmute_unchecked => (2, vec![param(0)], param(1)),
            sym::prefetch_read_data
            | sym::prefetch_write_data
            | sym::prefetch_read_instruction
            | sym::prefetch_write_instruction => (
                1,
                vec![
                    Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Not }),
                    tcx.types.i32,
                ],
                Ty::new_unit(tcx),
            ),
            sym::drop_in_place => (1, vec![Ty::new_mut_ptr(tcx, param(0))], Ty::new_unit(tcx)),
            sym::needs_drop => (1, Vec::new(), tcx.types.bool),

            sym::type_name => (1, Vec::new(), Ty::new_static_str(tcx)),
            sym::type_id => (1, Vec::new(), tcx.types.u128),
            sym::offset => (2, vec![param(0), param(1)], param(0)),
            sym::arith_offset => (
                1,
                vec![
                    Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Not }),
                    tcx.types.isize,
                ],
                Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Not }),
            ),
            sym::option_payload_ptr => {
                let option_def_id = tcx.require_lang_item(hir::LangItem::Option, None);
                let p0 = param(0);
                (
                    1,
                    vec![Ty::new_ptr(
                        tcx,
                        ty::TypeAndMut {
                            ty: Ty::new_adt(
                                tcx,
                                tcx.adt_def(option_def_id),
                                tcx.mk_substs_from_iter([ty::GenericArg::from(p0)].into_iter()),
                            ),
                            mutbl: hir::Mutability::Not,
                        },
                    )],
                    Ty::new_ptr(tcx, ty::TypeAndMut { ty: p0, mutbl: hir::Mutability::Not }),
                )
            }
            sym::ptr_mask => (
                1,
                vec![
                    Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Not }),
                    tcx.types.usize,
                ],
                Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Not }),
            ),

            sym::copy | sym::copy_nonoverlapping => (
                1,
                vec![
                    Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Not }),
                    Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Mut }),
                    tcx.types.usize,
                ],
                Ty::new_unit(tcx),
            ),
            sym::volatile_copy_memory | sym::volatile_copy_nonoverlapping_memory => (
                1,
                vec![
                    Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Mut }),
                    Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Not }),
                    tcx.types.usize,
                ],
                Ty::new_unit(tcx),
            ),
            sym::write_bytes | sym::volatile_set_memory => (
                1,
                vec![
                    Ty::new_ptr(tcx, ty::TypeAndMut { ty: param(0), mutbl: hir::Mutability::Mut }),
                    tcx.types.u8,
                    tcx.types.usize,
                ],
                Ty::new_unit(tcx),
            ),
            sym::sqrtf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::sqrtf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::powif32 => (0, vec![tcx.types.f32, tcx.types.i32], tcx.types.f32),
            sym::powif64 => (0, vec![tcx.types.f64, tcx.types.i32], tcx.types.f64),
            sym::sinf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::sinf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::cosf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::cosf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::powf32 => (0, vec![tcx.types.f32, tcx.types.f32], tcx.types.f32),
            sym::powf64 => (0, vec![tcx.types.f64, tcx.types.f64], tcx.types.f64),
            sym::expf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::expf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::exp2f32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::exp2f64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::logf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::logf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::log10f32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::log10f64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::log2f32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::log2f64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::fmaf32 => (0, vec![tcx.types.f32, tcx.types.f32, tcx.types.f32], tcx.types.f32),
            sym::fmaf64 => (0, vec![tcx.types.f64, tcx.types.f64, tcx.types.f64], tcx.types.f64),
            sym::fabsf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::fabsf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::minnumf32 => (0, vec![tcx.types.f32, tcx.types.f32], tcx.types.f32),
            sym::minnumf64 => (0, vec![tcx.types.f64, tcx.types.f64], tcx.types.f64),
            sym::maxnumf32 => (0, vec![tcx.types.f32, tcx.types.f32], tcx.types.f32),
            sym::maxnumf64 => (0, vec![tcx.types.f64, tcx.types.f64], tcx.types.f64),
            sym::copysignf32 => (0, vec![tcx.types.f32, tcx.types.f32], tcx.types.f32),
            sym::copysignf64 => (0, vec![tcx.types.f64, tcx.types.f64], tcx.types.f64),
            sym::floorf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::floorf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::ceilf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::ceilf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::truncf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::truncf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::rintf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::rintf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::nearbyintf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::nearbyintf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::roundf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::roundf64 => (0, vec![tcx.types.f64], tcx.types.f64),
            sym::roundevenf32 => (0, vec![tcx.types.f32], tcx.types.f32),
            sym::roundevenf64 => (0, vec![tcx.types.f64], tcx.types.f64),

            sym::volatile_load | sym::unaligned_volatile_load => {
                (1, vec![Ty::new_imm_ptr(tcx, param(0))], param(0))
            }
            sym::volatile_store | sym::unaligned_volatile_store => {
                (1, vec![Ty::new_mut_ptr(tcx, param(0)), param(0)], Ty::new_unit(tcx))
            }

            sym::ctpop
            | sym::ctlz
            | sym::ctlz_nonzero
            | sym::cttz
            | sym::cttz_nonzero
            | sym::bswap
            | sym::bitreverse => (1, vec![param(0)], param(0)),

            sym::add_with_overflow | sym::sub_with_overflow | sym::mul_with_overflow => {
                (1, vec![param(0), param(0)], Ty::new_tup(tcx, &[param(0), tcx.types.bool]))
            }

            sym::ptr_guaranteed_cmp => (
                1,
                vec![Ty::new_imm_ptr(tcx, param(0)), Ty::new_imm_ptr(tcx, param(0))],
                tcx.types.u8,
            ),

            sym::const_allocate => {
                (0, vec![tcx.types.usize, tcx.types.usize], Ty::new_mut_ptr(tcx, tcx.types.u8))
            }
            sym::const_deallocate => (
                0,
                vec![Ty::new_mut_ptr(tcx, tcx.types.u8), tcx.types.usize, tcx.types.usize],
                Ty::new_unit(tcx),
            ),

            sym::ptr_offset_from => (
                1,
                vec![Ty::new_imm_ptr(tcx, param(0)), Ty::new_imm_ptr(tcx, param(0))],
                tcx.types.isize,
            ),
            sym::ptr_offset_from_unsigned => (
                1,
                vec![Ty::new_imm_ptr(tcx, param(0)), Ty::new_imm_ptr(tcx, param(0))],
                tcx.types.usize,
            ),
            sym::unchecked_div | sym::unchecked_rem | sym::exact_div => {
                (1, vec![param(0), param(0)], param(0))
            }
            sym::unchecked_shl | sym::unchecked_shr | sym::rotate_left | sym::rotate_right => {
                (1, vec![param(0), param(0)], param(0))
            }
            sym::unchecked_add | sym::unchecked_sub | sym::unchecked_mul => {
                (1, vec![param(0), param(0)], param(0))
            }
            sym::wrapping_add | sym::wrapping_sub | sym::wrapping_mul => {
                (1, vec![param(0), param(0)], param(0))
            }
            sym::saturating_add | sym::saturating_sub => (1, vec![param(0), param(0)], param(0)),
            sym::fadd_fast | sym::fsub_fast | sym::fmul_fast | sym::fdiv_fast | sym::frem_fast => {
                (1, vec![param(0), param(0)], param(0))
            }
            sym::float_to_int_unchecked => (2, vec![param(0)], param(1)),

            sym::assume => (0, vec![tcx.types.bool], Ty::new_unit(tcx)),
            sym::likely => (0, vec![tcx.types.bool], tcx.types.bool),
            sym::unlikely => (0, vec![tcx.types.bool], tcx.types.bool),

            sym::read_via_copy => (1, vec![Ty::new_imm_ptr(tcx, param(0))], param(0)),
            sym::write_via_move => {
                (1, vec![Ty::new_mut_ptr(tcx, param(0)), param(0)], Ty::new_unit(tcx))
            }

            sym::discriminant_value => {
                let assoc_items = tcx.associated_item_def_ids(
                    tcx.require_lang_item(hir::LangItem::DiscriminantKind, None),
                );
                let discriminant_def_id = assoc_items[0];

                let br = ty::BoundRegion { var: ty::BoundVar::from_u32(0), kind: ty::BrAnon(None) };
                (
                    1,
                    vec![Ty::new_imm_ref(
                        tcx,
                        ty::Region::new_late_bound(tcx, ty::INNERMOST, br),
                        param(0),
                    )],
                    Ty::new_projection(tcx, discriminant_def_id, tcx.mk_substs(&[param(0).into()])),
                )
            }

            kw::Try => {
                let mut_u8 = Ty::new_mut_ptr(tcx, tcx.types.u8);
                let try_fn_ty = ty::Binder::dummy(tcx.mk_fn_sig(
                    [mut_u8],
                    Ty::new_unit(tcx),
                    false,
                    hir::Unsafety::Normal,
                    Abi::Rust,
                ));
                let catch_fn_ty = ty::Binder::dummy(tcx.mk_fn_sig(
                    [mut_u8, mut_u8],
                    Ty::new_unit(tcx),
                    false,
                    hir::Unsafety::Normal,
                    Abi::Rust,
                ));
                (
                    0,
                    vec![Ty::new_fn_ptr(tcx, try_fn_ty), mut_u8, Ty::new_fn_ptr(tcx, catch_fn_ty)],
                    tcx.types.i32,
                )
            }

            sym::va_start | sym::va_end => match mk_va_list_ty(hir::Mutability::Mut) {
                Some((va_list_ref_ty, _)) => (0, vec![va_list_ref_ty], Ty::new_unit(tcx)),
                None => bug!("`va_list` language item needed for C-variadic intrinsics"),
            },

            sym::va_copy => match mk_va_list_ty(hir::Mutability::Not) {
                Some((va_list_ref_ty, va_list_ty)) => {
                    let va_list_ptr_ty = Ty::new_mut_ptr(tcx, va_list_ty);
                    (0, vec![va_list_ptr_ty, va_list_ref_ty], Ty::new_unit(tcx))
                }
                None => bug!("`va_list` language item needed for C-variadic intrinsics"),
            },

            sym::va_arg => match mk_va_list_ty(hir::Mutability::Mut) {
                Some((va_list_ref_ty, _)) => (1, vec![va_list_ref_ty], param(0)),
                None => bug!("`va_list` language item needed for C-variadic intrinsics"),
            },

            sym::nontemporal_store => {
                (1, vec![Ty::new_mut_ptr(tcx, param(0)), param(0)], Ty::new_unit(tcx))
            }

            sym::raw_eq => {
                let br = ty::BoundRegion { var: ty::BoundVar::from_u32(0), kind: ty::BrAnon(None) };
                let param_ty = Ty::new_imm_ref(
                    tcx,
                    ty::Region::new_late_bound(tcx, ty::INNERMOST, br),
                    param(0),
                );
                (1, vec![param_ty; 2], tcx.types.bool)
            }

            sym::black_box => (1, vec![param(0)], param(0)),

            sym::const_eval_select => (4, vec![param(0), param(1), param(2)], param(3)),

            sym::vtable_size | sym::vtable_align => {
                (0, vec![Ty::new_imm_ptr(tcx, Ty::new_unit(tcx))], tcx.types.usize)
            }

            other => {
                tcx.sess.emit_err(UnrecognizedIntrinsicFunction { span: it.span, name: other });
                return;
            }
        };
        (n_tps, 0, inputs, output, unsafety)
    };
    let sig = tcx.mk_fn_sig(inputs, output, false, unsafety, Abi::RustIntrinsic);
    let sig = ty::Binder::bind_with_vars(sig, bound_vars);
    equate_intrinsic_type(tcx, it, n_tps, n_lts, sig)
}

/// Type-check `extern "platform-intrinsic" { ... }` functions.
pub fn check_platform_intrinsic_type(tcx: TyCtxt<'_>, it: &hir::ForeignItem<'_>) {
    let param = |n| {
        let name = Symbol::intern(&format!("P{}", n));
        Ty::new_param(tcx, n, name)
    };

    let name = it.ident.name;

    let (n_tps, inputs, output) = match name {
        sym::simd_eq | sym::simd_ne | sym::simd_lt | sym::simd_le | sym::simd_gt | sym::simd_ge => {
            (2, vec![param(0), param(0)], param(1))
        }
        sym::simd_add
        | sym::simd_sub
        | sym::simd_mul
        | sym::simd_rem
        | sym::simd_div
        | sym::simd_shl
        | sym::simd_shr
        | sym::simd_and
        | sym::simd_or
        | sym::simd_xor
        | sym::simd_fmin
        | sym::simd_fmax
        | sym::simd_fpow
        | sym::simd_saturating_add
        | sym::simd_saturating_sub => (1, vec![param(0), param(0)], param(0)),
        sym::simd_arith_offset => (2, vec![param(0), param(1)], param(0)),
        sym::simd_neg
        | sym::simd_fsqrt
        | sym::simd_fsin
        | sym::simd_fcos
        | sym::simd_fexp
        | sym::simd_fexp2
        | sym::simd_flog2
        | sym::simd_flog10
        | sym::simd_flog
        | sym::simd_fabs
        | sym::simd_ceil
        | sym::simd_floor
        | sym::simd_round
        | sym::simd_trunc => (1, vec![param(0)], param(0)),
        sym::simd_fpowi => (1, vec![param(0), tcx.types.i32], param(0)),
        sym::simd_fma => (1, vec![param(0), param(0), param(0)], param(0)),
        sym::simd_gather => (3, vec![param(0), param(1), param(2)], param(0)),
        sym::simd_scatter => (3, vec![param(0), param(1), param(2)], Ty::new_unit(tcx)),
        sym::simd_insert => (2, vec![param(0), tcx.types.u32, param(1)], param(0)),
        sym::simd_extract => (2, vec![param(0), tcx.types.u32], param(1)),
        sym::simd_cast
        | sym::simd_as
        | sym::simd_cast_ptr
        | sym::simd_expose_addr
        | sym::simd_from_exposed_addr => (2, vec![param(0)], param(1)),
        sym::simd_bitmask => (2, vec![param(0)], param(1)),
        sym::simd_select | sym::simd_select_bitmask => {
            (2, vec![param(0), param(1), param(1)], param(1))
        }
        sym::simd_reduce_all | sym::simd_reduce_any => (1, vec![param(0)], tcx.types.bool),
        sym::simd_reduce_add_ordered | sym::simd_reduce_mul_ordered => {
            (2, vec![param(0), param(1)], param(1))
        }
        sym::simd_reduce_add_unordered
        | sym::simd_reduce_mul_unordered
        | sym::simd_reduce_and
        | sym::simd_reduce_or
        | sym::simd_reduce_xor
        | sym::simd_reduce_min
        | sym::simd_reduce_max
        | sym::simd_reduce_min_nanless
        | sym::simd_reduce_max_nanless => (2, vec![param(0)], param(1)),
        sym::simd_shuffle => (3, vec![param(0), param(0), param(1)], param(2)),
        name if name.as_str().starts_with("simd_shuffle") => {
            match name.as_str()["simd_shuffle".len()..].parse() {
                Ok(n) => {
                    let params = vec![param(0), param(0), Ty::new_array(tcx, tcx.types.u32, n)];
                    (2, params, param(1))
                }
                Err(_) => {
                    let msg =
                        format!("unrecognized platform-specific intrinsic function: `{name}`");
                    tcx.sess.struct_span_err(it.span, msg).emit();
                    return;
                }
            }
        }
        _ => {
            let msg = format!("unrecognized platform-specific intrinsic function: `{name}`");
            tcx.sess.struct_span_err(it.span, msg).emit();
            return;
        }
    };

    let sig = tcx.mk_fn_sig(inputs, output, false, hir::Unsafety::Unsafe, Abi::PlatformIntrinsic);
    let sig = ty::Binder::dummy(sig);
    equate_intrinsic_type(tcx, it, n_tps, 0, sig)
}
