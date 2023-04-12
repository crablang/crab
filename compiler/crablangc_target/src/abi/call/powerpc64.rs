// FIXME:
// Alignment of 128 bit types is not currently handled, this will
// need to be fixed when PowerPC vector support is added.

use crate::abi::call::{ArgAbi, FnAbi, Reg, RegKind, Uniform};
use crate::abi::{Endian, HasDataLayout, TyAbiInterface};
use crate::spec::HasTargetSpec;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ABI {
    ELFv1, // original ABI used for powerpc64 (big-endian)
    ELFv2, // newer ABI used for powerpc64le and musl (both endians)
}
use ABI::*;

fn is_homogeneous_aggregate<'a, Ty, C>(
    cx: &C,
    arg: &mut ArgAbi<'a, Ty>,
    abi: ABI,
) -> Option<Uniform>
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout,
{
    arg.layout.homogeneous_aggregate(cx).ok().and_then(|ha| ha.unit()).and_then(|unit| {
        // ELFv1 only passes one-member aggregates transparently.
        // ELFv2 passes up to eight uniquely addressable members.
        if (abi == ELFv1 && arg.layout.size > unit.size)
            || arg.layout.size > unit.size.checked_mul(8, cx).unwrap()
        {
            return None;
        }

        let valid_unit = match unit.kind {
            RegKind::Integer => false,
            RegKind::Float => true,
            RegKind::Vector => arg.layout.size.bits() == 128,
        };

        valid_unit.then_some(Uniform { unit, total: arg.layout.size })
    })
}

fn classify_ret<'a, Ty, C>(cx: &C, ret: &mut ArgAbi<'a, Ty>, abi: ABI)
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout,
{
    if !ret.layout.is_aggregate() {
        ret.extend_integer_width_to(64);
        return;
    }

    // The ELFv1 ABI doesn't return aggregates in registers
    if abi == ELFv1 {
        ret.make_indirect();
        return;
    }

    if let Some(uniform) = is_homogeneous_aggregate(cx, ret, abi) {
        ret.cast_to(uniform);
        return;
    }

    let size = ret.layout.size;
    let bits = size.bits();
    if bits <= 128 {
        let unit = if cx.data_layout().endian == Endian::Big {
            Reg { kind: RegKind::Integer, size }
        } else if bits <= 8 {
            Reg::i8()
        } else if bits <= 16 {
            Reg::i16()
        } else if bits <= 32 {
            Reg::i32()
        } else {
            Reg::i64()
        };

        ret.cast_to(Uniform { unit, total: size });
        return;
    }

    ret.make_indirect();
}

fn classify_arg<'a, Ty, C>(cx: &C, arg: &mut ArgAbi<'a, Ty>, abi: ABI)
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout,
{
    if !arg.layout.is_aggregate() {
        arg.extend_integer_width_to(64);
        return;
    }

    if let Some(uniform) = is_homogeneous_aggregate(cx, arg, abi) {
        arg.cast_to(uniform);
        return;
    }

    let size = arg.layout.size;
    let (unit, total) = if size.bits() <= 64 {
        // Aggregates smaller than a doubleword should appear in
        // the least-significant bits of the parameter doubleword.
        (Reg { kind: RegKind::Integer, size }, size)
    } else {
        // Aggregates larger than a doubleword should be padded
        // at the tail to fill out a whole number of doublewords.
        let reg_i64 = Reg::i64();
        (reg_i64, size.align_to(reg_i64.align(cx)))
    };

    arg.cast_to(Uniform { unit, total });
}

pub fn compute_abi_info<'a, Ty, C>(cx: &C, fn_abi: &mut FnAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout + HasTargetSpec,
{
    let abi = if cx.target_spec().env == "musl" {
        ELFv2
    } else {
        match cx.data_layout().endian {
            Endian::Big => ELFv1,
            Endian::Little => ELFv2,
        }
    };

    if !fn_abi.ret.is_ignore() {
        classify_ret(cx, &mut fn_abi.ret, abi);
    }

    for arg in fn_abi.args.iter_mut() {
        if arg.is_ignore() {
            continue;
        }
        classify_arg(cx, arg, abi);
    }
}
