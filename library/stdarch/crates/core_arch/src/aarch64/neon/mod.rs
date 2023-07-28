//! ARMv8 ASIMD intrinsics

#![allow(non_camel_case_types)]

#[rustfmt::skip]
mod generated;
#[rustfmt::skip]
pub use self::generated::*;

// FIXME: replace neon with asimd

use crate::{
    core_arch::{arm_shared::*, simd::*, simd_llvm::*},
    hint::unreachable_unchecked,
    mem::{transmute, zeroed},
    ptr::{read_unaligned, write_unaligned},
};
#[cfg(test)]
use stdarch_test::assert_instr;

types! {
    /// ARM-specific 64-bit wide vector of one packed `f64`.
    #[stable(feature = "neon_intrinsics", since = "1.59.0")]
    pub struct float64x1_t(f64); // FIXME: check this!
    /// ARM-specific 128-bit wide vector of two packed `f64`.
    #[stable(feature = "neon_intrinsics", since = "1.59.0")]
    pub struct float64x2_t(f64, f64);
}

/// ARM-specific type containing two `float64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub struct float64x1x2_t(pub float64x1_t, pub float64x1_t);
/// ARM-specific type containing three `float64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub struct float64x1x3_t(pub float64x1_t, pub float64x1_t, pub float64x1_t);
/// ARM-specific type containing four `float64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub struct float64x1x4_t(
    pub float64x1_t,
    pub float64x1_t,
    pub float64x1_t,
    pub float64x1_t,
);

/// ARM-specific type containing two `float64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub struct float64x2x2_t(pub float64x2_t, pub float64x2_t);
/// ARM-specific type containing three `float64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub struct float64x2x3_t(pub float64x2_t, pub float64x2_t, pub float64x2_t);
/// ARM-specific type containing four `float64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub struct float64x2x4_t(
    pub float64x2_t,
    pub float64x2_t,
    pub float64x2_t,
    pub float64x2_t,
);

#[allow(improper_ctypes)]
extern "unadjusted" {
    // absolute value
    #[link_name = "llvm.aarch64.neon.abs.i64"]
    fn vabsd_s64_(a: i64) -> i64;
    #[link_name = "llvm.aarch64.neon.abs.v1i64"]
    fn vabs_s64_(a: int64x1_t) -> int64x1_t;
    #[link_name = "llvm.aarch64.neon.abs.v2i64"]
    fn vabsq_s64_(a: int64x2_t) -> int64x2_t;

    #[link_name = "llvm.aarch64.neon.suqadd.v8i8"]
    fn vuqadd_s8_(a: int8x8_t, b: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.suqadd.v16i8"]
    fn vuqaddq_s8_(a: int8x16_t, b: uint8x16_t) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.suqadd.v4i16"]
    fn vuqadd_s16_(a: int16x4_t, b: uint16x4_t) -> int16x4_t;
    #[link_name = "llvm.aarch64.neon.suqadd.v8i16"]
    fn vuqaddq_s16_(a: int16x8_t, b: uint16x8_t) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.suqadd.v2i32"]
    fn vuqadd_s32_(a: int32x2_t, b: uint32x2_t) -> int32x2_t;
    #[link_name = "llvm.aarch64.neon.suqadd.v4i32"]
    fn vuqaddq_s32_(a: int32x4_t, b: uint32x4_t) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.suqadd.v1i64"]
    fn vuqadd_s64_(a: int64x1_t, b: uint64x1_t) -> int64x1_t;
    #[link_name = "llvm.aarch64.neon.suqadd.v2i64"]
    fn vuqaddq_s64_(a: int64x2_t, b: uint64x2_t) -> int64x2_t;

    #[link_name = "llvm.aarch64.neon.usqadd.v8i8"]
    fn vsqadd_u8_(a: uint8x8_t, b: int8x8_t) -> uint8x8_t;
    #[link_name = "llvm.aarch64.neon.usqadd.v16i8"]
    fn vsqaddq_u8_(a: uint8x16_t, b: int8x16_t) -> uint8x16_t;
    #[link_name = "llvm.aarch64.neon.usqadd.v4i16"]
    fn vsqadd_u16_(a: uint16x4_t, b: int16x4_t) -> uint16x4_t;
    #[link_name = "llvm.aarch64.neon.usqadd.v8i16"]
    fn vsqaddq_u16_(a: uint16x8_t, b: int16x8_t) -> uint16x8_t;
    #[link_name = "llvm.aarch64.neon.usqadd.v2i32"]
    fn vsqadd_u32_(a: uint32x2_t, b: int32x2_t) -> uint32x2_t;
    #[link_name = "llvm.aarch64.neon.usqadd.v4i32"]
    fn vsqaddq_u32_(a: uint32x4_t, b: int32x4_t) -> uint32x4_t;
    #[link_name = "llvm.aarch64.neon.usqadd.v1i64"]
    fn vsqadd_u64_(a: uint64x1_t, b: int64x1_t) -> uint64x1_t;
    #[link_name = "llvm.aarch64.neon.usqadd.v2i64"]
    fn vsqaddq_u64_(a: uint64x2_t, b: int64x2_t) -> uint64x2_t;

    #[link_name = "llvm.aarch64.neon.addp.v8i16"]
    fn vpaddq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.addp.v4i32"]
    fn vpaddq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.addp.v2i64"]
    fn vpaddq_s64_(a: int64x2_t, b: int64x2_t) -> int64x2_t;
    #[link_name = "llvm.aarch64.neon.addp.v16i8"]
    fn vpaddq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.saddv.i32.v4i16"]
    fn vaddv_s16_(a: int16x4_t) -> i16;
    #[link_name = "llvm.aarch64.neon.saddv.i32.v2i32"]
    fn vaddv_s32_(a: int32x2_t) -> i32;
    #[link_name = "llvm.aarch64.neon.saddv.i32.v8i8"]
    fn vaddv_s8_(a: int8x8_t) -> i8;
    #[link_name = "llvm.aarch64.neon.uaddv.i32.v4i16"]
    fn vaddv_u16_(a: uint16x4_t) -> u16;
    #[link_name = "llvm.aarch64.neon.uaddv.i32.v2i32"]
    fn vaddv_u32_(a: uint32x2_t) -> u32;
    #[link_name = "llvm.aarch64.neon.uaddv.i32.v8i8"]
    fn vaddv_u8_(a: uint8x8_t) -> u8;
    #[link_name = "llvm.aarch64.neon.saddv.i32.v8i16"]
    fn vaddvq_s16_(a: int16x8_t) -> i16;
    #[link_name = "llvm.aarch64.neon.saddv.i32.v4i32"]
    fn vaddvq_s32_(a: int32x4_t) -> i32;
    #[link_name = "llvm.aarch64.neon.saddv.i32.v16i8"]
    fn vaddvq_s8_(a: int8x16_t) -> i8;
    #[link_name = "llvm.aarch64.neon.uaddv.i32.v8i16"]
    fn vaddvq_u16_(a: uint16x8_t) -> u16;
    #[link_name = "llvm.aarch64.neon.uaddv.i32.v4i32"]
    fn vaddvq_u32_(a: uint32x4_t) -> u32;
    #[link_name = "llvm.aarch64.neon.uaddv.i32.v16i8"]
    fn vaddvq_u8_(a: uint8x16_t) -> u8;
    #[link_name = "llvm.aarch64.neon.saddv.i64.v2i64"]
    fn vaddvq_s64_(a: int64x2_t) -> i64;
    #[link_name = "llvm.aarch64.neon.uaddv.i64.v2i64"]
    fn vaddvq_u64_(a: uint64x2_t) -> u64;

    #[link_name = "llvm.aarch64.neon.saddlv.i32.v8i8"]
    fn vaddlv_s8_(a: int8x8_t) -> i32;
    #[link_name = "llvm.aarch64.neon.uaddlv.i32.v8i8"]
    fn vaddlv_u8_(a: uint8x8_t) -> u32;
    #[link_name = "llvm.aarch64.neon.saddlv.i32.v16i8"]
    fn vaddlvq_s8_(a: int8x16_t) -> i32;
    #[link_name = "llvm.aarch64.neon.uaddlv.i32.v16i8"]
    fn vaddlvq_u8_(a: uint8x16_t) -> u32;

    #[link_name = "llvm.aarch64.neon.smaxv.i8.v8i8"]
    fn vmaxv_s8_(a: int8x8_t) -> i8;
    #[link_name = "llvm.aarch64.neon.smaxv.i8.v16i8"]
    fn vmaxvq_s8_(a: int8x16_t) -> i8;
    #[link_name = "llvm.aarch64.neon.smaxv.i16.v4i16"]
    fn vmaxv_s16_(a: int16x4_t) -> i16;
    #[link_name = "llvm.aarch64.neon.smaxv.i16.v8i16"]
    fn vmaxvq_s16_(a: int16x8_t) -> i16;
    #[link_name = "llvm.aarch64.neon.smaxv.i32.v2i32"]
    fn vmaxv_s32_(a: int32x2_t) -> i32;
    #[link_name = "llvm.aarch64.neon.smaxv.i32.v4i32"]
    fn vmaxvq_s32_(a: int32x4_t) -> i32;

    #[link_name = "llvm.aarch64.neon.umaxv.i8.v8i8"]
    fn vmaxv_u8_(a: uint8x8_t) -> u8;
    #[link_name = "llvm.aarch64.neon.umaxv.i8.v16i8"]
    fn vmaxvq_u8_(a: uint8x16_t) -> u8;
    #[link_name = "llvm.aarch64.neon.umaxv.i16.v4i16"]
    fn vmaxv_u16_(a: uint16x4_t) -> u16;
    #[link_name = "llvm.aarch64.neon.umaxv.i16.v8i16"]
    fn vmaxvq_u16_(a: uint16x8_t) -> u16;
    #[link_name = "llvm.aarch64.neon.umaxv.i32.v2i32"]
    fn vmaxv_u32_(a: uint32x2_t) -> u32;
    #[link_name = "llvm.aarch64.neon.umaxv.i32.v4i32"]
    fn vmaxvq_u32_(a: uint32x4_t) -> u32;

    #[link_name = "llvm.aarch64.neon.fmaxv.f32.v2f32"]
    fn vmaxv_f32_(a: float32x2_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fmaxv.f32.v4f32"]
    fn vmaxvq_f32_(a: float32x4_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fmaxv.f64.v2f64"]
    fn vmaxvq_f64_(a: float64x2_t) -> f64;

    #[link_name = "llvm.aarch64.neon.sminv.i8.v8i8"]
    fn vminv_s8_(a: int8x8_t) -> i8;
    #[link_name = "llvm.aarch64.neon.sminv.i8.v16i8"]
    fn vminvq_s8_(a: int8x16_t) -> i8;
    #[link_name = "llvm.aarch64.neon.sminv.i16.v4i16"]
    fn vminv_s16_(a: int16x4_t) -> i16;
    #[link_name = "llvm.aarch64.neon.sminv.i16.v8i16"]
    fn vminvq_s16_(a: int16x8_t) -> i16;
    #[link_name = "llvm.aarch64.neon.sminv.i32.v2i32"]
    fn vminv_s32_(a: int32x2_t) -> i32;
    #[link_name = "llvm.aarch64.neon.sminv.i32.v4i32"]
    fn vminvq_s32_(a: int32x4_t) -> i32;

    #[link_name = "llvm.aarch64.neon.uminv.i8.v8i8"]
    fn vminv_u8_(a: uint8x8_t) -> u8;
    #[link_name = "llvm.aarch64.neon.uminv.i8.v16i8"]
    fn vminvq_u8_(a: uint8x16_t) -> u8;
    #[link_name = "llvm.aarch64.neon.uminv.i16.v4i16"]
    fn vminv_u16_(a: uint16x4_t) -> u16;
    #[link_name = "llvm.aarch64.neon.uminv.i16.v8i16"]
    fn vminvq_u16_(a: uint16x8_t) -> u16;
    #[link_name = "llvm.aarch64.neon.uminv.i32.v2i32"]
    fn vminv_u32_(a: uint32x2_t) -> u32;
    #[link_name = "llvm.aarch64.neon.uminv.i32.v4i32"]
    fn vminvq_u32_(a: uint32x4_t) -> u32;

    #[link_name = "llvm.aarch64.neon.fminv.f32.v2f32"]
    fn vminv_f32_(a: float32x2_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fminv.f32.v4f32"]
    fn vminvq_f32_(a: float32x4_t) -> f32;
    #[link_name = "llvm.aarch64.neon.fminv.f64.v2f64"]
    fn vminvq_f64_(a: float64x2_t) -> f64;

    #[link_name = "llvm.aarch64.neon.sminp.v16i8"]
    fn vpminq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.sminp.v8i16"]
    fn vpminq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.sminp.v4i32"]
    fn vpminq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.uminp.v16i8"]
    fn vpminq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    #[link_name = "llvm.aarch64.neon.uminp.v8i16"]
    fn vpminq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    #[link_name = "llvm.aarch64.neon.uminp.v4i32"]
    fn vpminq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    #[link_name = "llvm.aarch64.neon.fminp.4f32"]
    fn vpminq_f32_(a: float32x4_t, b: float32x4_t) -> float32x4_t;
    #[link_name = "llvm.aarch64.neon.fminp.v2f64"]
    fn vpminq_f64_(a: float64x2_t, b: float64x2_t) -> float64x2_t;

    #[link_name = "llvm.aarch64.neon.smaxp.v16i8"]
    fn vpmaxq_s8_(a: int8x16_t, b: int8x16_t) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.smaxp.v8i16"]
    fn vpmaxq_s16_(a: int16x8_t, b: int16x8_t) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.smaxp.v4i32"]
    fn vpmaxq_s32_(a: int32x4_t, b: int32x4_t) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.umaxp.v16i8"]
    fn vpmaxq_u8_(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t;
    #[link_name = "llvm.aarch64.neon.umaxp.v8i16"]
    fn vpmaxq_u16_(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t;
    #[link_name = "llvm.aarch64.neon.umaxp.v4i32"]
    fn vpmaxq_u32_(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t;
    #[link_name = "llvm.aarch64.neon.fmaxp.4f32"]
    fn vpmaxq_f32_(a: float32x4_t, b: float32x4_t) -> float32x4_t;
    #[link_name = "llvm.aarch64.neon.fmaxp.v2f64"]
    fn vpmaxq_f64_(a: float64x2_t, b: float64x2_t) -> float64x2_t;

    #[link_name = "llvm.aarch64.neon.tbl1.v8i8"]
    fn vqtbl1(a: int8x16_t, b: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbl1.v16i8"]
    fn vqtbl1q(a: int8x16_t, b: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbx1.v8i8"]
    fn vqtbx1(a: int8x8_t, b: int8x16_t, c: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbx1.v16i8"]
    fn vqtbx1q(a: int8x16_t, b: int8x16_t, c: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbl2.v8i8"]
    fn vqtbl2(a0: int8x16_t, a1: int8x16_t, b: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbl2.v16i8"]
    fn vqtbl2q(a0: int8x16_t, a1: int8x16_t, b: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbx2.v8i8"]
    fn vqtbx2(a: int8x8_t, b0: int8x16_t, b1: int8x16_t, c: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbx2.v16i8"]
    fn vqtbx2q(a: int8x16_t, b0: int8x16_t, b1: int8x16_t, c: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbl3.v8i8"]
    fn vqtbl3(a0: int8x16_t, a1: int8x16_t, a2: int8x16_t, b: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbl3.v16i8"]
    fn vqtbl3q(a0: int8x16_t, a1: int8x16_t, a2: int8x16_t, b: uint8x16_t) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbx3.v8i8"]
    fn vqtbx3(a: int8x8_t, b0: int8x16_t, b1: int8x16_t, b2: int8x16_t, c: uint8x8_t) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbx3.v16i8"]
    fn vqtbx3q(
        a: int8x16_t,
        b0: int8x16_t,
        b1: int8x16_t,
        b2: int8x16_t,
        c: uint8x16_t,
    ) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbl4.v8i8"]
    fn vqtbl4(a0: int8x16_t, a1: int8x16_t, a2: int8x16_t, a3: int8x16_t, b: uint8x8_t)
        -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.tbl4.v16i8"]
    fn vqtbl4q(
        a0: int8x16_t,
        a1: int8x16_t,
        a2: int8x16_t,
        a3: int8x16_t,
        b: uint8x16_t,
    ) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.tbx4.v8i8"]
    fn vqtbx4(
        a: int8x8_t,
        b0: int8x16_t,
        b1: int8x16_t,
        b2: int8x16_t,
        b3: int8x16_t,
        c: uint8x8_t,
    ) -> int8x8_t;

    #[link_name = "llvm.aarch64.neon.tbx4.v16i8"]
    fn vqtbx4q(
        a: int8x16_t,
        b0: int8x16_t,
        b1: int8x16_t,
        b2: int8x16_t,
        b3: int8x16_t,
        c: uint8x16_t,
    ) -> int8x16_t;

    #[link_name = "llvm.aarch64.neon.vsli.v8i8"]
    fn vsli_n_s8_(a: int8x8_t, b: int8x8_t, n: i32) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.vsli.v16i8"]
    fn vsliq_n_s8_(a: int8x16_t, b: int8x16_t, n: i32) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.vsli.v4i16"]
    fn vsli_n_s16_(a: int16x4_t, b: int16x4_t, n: i32) -> int16x4_t;
    #[link_name = "llvm.aarch64.neon.vsli.v8i16"]
    fn vsliq_n_s16_(a: int16x8_t, b: int16x8_t, n: i32) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.vsli.v2i32"]
    fn vsli_n_s32_(a: int32x2_t, b: int32x2_t, n: i32) -> int32x2_t;
    #[link_name = "llvm.aarch64.neon.vsli.v4i32"]
    fn vsliq_n_s32_(a: int32x4_t, b: int32x4_t, n: i32) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.vsli.v1i64"]
    fn vsli_n_s64_(a: int64x1_t, b: int64x1_t, n: i32) -> int64x1_t;
    #[link_name = "llvm.aarch64.neon.vsli.v2i64"]
    fn vsliq_n_s64_(a: int64x2_t, b: int64x2_t, n: i32) -> int64x2_t;

    #[link_name = "llvm.aarch64.neon.vsri.v8i8"]
    fn vsri_n_s8_(a: int8x8_t, b: int8x8_t, n: i32) -> int8x8_t;
    #[link_name = "llvm.aarch64.neon.vsri.v16i8"]
    fn vsriq_n_s8_(a: int8x16_t, b: int8x16_t, n: i32) -> int8x16_t;
    #[link_name = "llvm.aarch64.neon.vsri.v4i16"]
    fn vsri_n_s16_(a: int16x4_t, b: int16x4_t, n: i32) -> int16x4_t;
    #[link_name = "llvm.aarch64.neon.vsri.v8i16"]
    fn vsriq_n_s16_(a: int16x8_t, b: int16x8_t, n: i32) -> int16x8_t;
    #[link_name = "llvm.aarch64.neon.vsri.v2i32"]
    fn vsri_n_s32_(a: int32x2_t, b: int32x2_t, n: i32) -> int32x2_t;
    #[link_name = "llvm.aarch64.neon.vsri.v4i32"]
    fn vsriq_n_s32_(a: int32x4_t, b: int32x4_t, n: i32) -> int32x4_t;
    #[link_name = "llvm.aarch64.neon.vsri.v1i64"]
    fn vsri_n_s64_(a: int64x1_t, b: int64x1_t, n: i32) -> int64x1_t;
    #[link_name = "llvm.aarch64.neon.vsri.v2i64"]
    fn vsriq_n_s64_(a: int64x2_t, b: int64x2_t, n: i32) -> int64x2_t;
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N1 = 0, N2 = 0))]
#[rustc_legacy_const_generics(1, 3)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcopy_lane_s64<const N1: i32, const N2: i32>(
    _a: int64x1_t,
    b: int64x1_t,
) -> int64x1_t {
    static_assert!(N1 == 0);
    static_assert!(N2 == 0);
    b
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N1 = 0, N2 = 0))]
#[rustc_legacy_const_generics(1, 3)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcopy_lane_u64<const N1: i32, const N2: i32>(
    _a: uint64x1_t,
    b: uint64x1_t,
) -> uint64x1_t {
    static_assert!(N1 == 0);
    static_assert!(N2 == 0);
    b
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N1 = 0, N2 = 0))]
#[rustc_legacy_const_generics(1, 3)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcopy_lane_p64<const N1: i32, const N2: i32>(
    _a: poly64x1_t,
    b: poly64x1_t,
) -> poly64x1_t {
    static_assert!(N1 == 0);
    static_assert!(N2 == 0);
    b
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N1 = 0, N2 = 0))]
#[rustc_legacy_const_generics(1, 3)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcopy_lane_f64<const N1: i32, const N2: i32>(
    _a: float64x1_t,
    b: float64x1_t,
) -> float64x1_t {
    static_assert!(N1 == 0);
    static_assert!(N2 == 0);
    b
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, LANE1 = 0, LANE2 = 1))]
#[rustc_legacy_const_generics(1, 3)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcopy_laneq_s64<const LANE1: i32, const LANE2: i32>(
    _a: int64x1_t,
    b: int64x2_t,
) -> int64x1_t {
    static_assert!(LANE1 == 0);
    static_assert_uimm_bits!(LANE2, 1);
    transmute::<i64, _>(simd_extract(b, LANE2 as u32))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, LANE1 = 0, LANE2 = 1))]
#[rustc_legacy_const_generics(1, 3)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcopy_laneq_u64<const LANE1: i32, const LANE2: i32>(
    _a: uint64x1_t,
    b: uint64x2_t,
) -> uint64x1_t {
    static_assert!(LANE1 == 0);
    static_assert_uimm_bits!(LANE2, 1);
    transmute::<u64, _>(simd_extract(b, LANE2 as u32))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, LANE1 = 0, LANE2 = 1))]
#[rustc_legacy_const_generics(1, 3)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcopy_laneq_p64<const LANE1: i32, const LANE2: i32>(
    _a: poly64x1_t,
    b: poly64x2_t,
) -> poly64x1_t {
    static_assert!(LANE1 == 0);
    static_assert_uimm_bits!(LANE2, 1);
    transmute::<u64, _>(simd_extract(b, LANE2 as u32))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, LANE1 = 0, LANE2 = 1))]
#[rustc_legacy_const_generics(1, 3)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcopy_laneq_f64<const LANE1: i32, const LANE2: i32>(
    _a: float64x1_t,
    b: float64x2_t,
) -> float64x1_t {
    static_assert!(LANE1 == 0);
    static_assert_uimm_bits!(LANE2, 1);
    transmute::<f64, _>(simd_extract(b, LANE2 as u32))
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_s8(ptr: *const i8) -> int8x8_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_s8(ptr: *const i8) -> int8x16_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_s16(ptr: *const i16) -> int16x4_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_s16(ptr: *const i16) -> int16x8_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_s32(ptr: *const i32) -> int32x2_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_s32(ptr: *const i32) -> int32x4_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_s64(ptr: *const i64) -> int64x1_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_s64(ptr: *const i64) -> int64x2_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_u8(ptr: *const u8) -> uint8x8_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_u8(ptr: *const u8) -> uint8x16_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_u16(ptr: *const u16) -> uint16x4_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_u16(ptr: *const u16) -> uint16x8_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_u32(ptr: *const u32) -> uint32x2_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_u32(ptr: *const u32) -> uint32x4_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_u64(ptr: *const u64) -> uint64x1_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_u64(ptr: *const u64) -> uint64x2_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_p8(ptr: *const p8) -> poly8x8_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_p8(ptr: *const p8) -> poly8x16_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_p16(ptr: *const p16) -> poly16x4_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_p16(ptr: *const p16) -> poly16x8_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vld1_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_p64(ptr: *const p64) -> poly64x1_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vld1q_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_p64(ptr: *const p64) -> poly64x2_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_f32(ptr: *const f32) -> float32x2_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_f32(ptr: *const f32) -> float32x4_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_f64(ptr: *const f64) -> float64x1_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_f64(ptr: *const f64) -> float64x2_t {
    read_unaligned(ptr.cast())
}

/// Load multiple single-element structures to one, two, three, or four registers
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ldr))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_dup_f64(ptr: *const f64) -> float64x1_t {
    vld1_f64(ptr)
}

/// Load multiple single-element structures to one, two, three, or four registers
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ld1r))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_dup_f64(ptr: *const f64) -> float64x2_t {
    let x = vld1q_lane_f64::<0>(ptr, transmute(f64x2::splat(0.)));
    simd_shuffle!(x, x, [0, 0])
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(test, assert_instr(ldr, LANE = 0))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1_lane_f64<const LANE: i32>(ptr: *const f64, src: float64x1_t) -> float64x1_t {
    static_assert!(LANE == 0);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(test, assert_instr(ld1, LANE = 1))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vld1q_lane_f64<const LANE: i32>(ptr: *const f64, src: float64x2_t) -> float64x2_t {
    static_assert_uimm_bits!(LANE, 1);
    simd_insert(src, LANE as u32, *ptr)
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_s8(ptr: *mut i8, a: int8x8_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_s8(ptr: *mut i8, a: int8x16_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_s16(ptr: *mut i16, a: int16x4_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_s16(ptr: *mut i16, a: int16x8_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_s32(ptr: *mut i32, a: int32x2_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_s32(ptr: *mut i32, a: int32x4_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_s64(ptr: *mut i64, a: int64x1_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_s64(ptr: *mut i64, a: int64x2_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_u8(ptr: *mut u8, a: uint8x8_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_u8(ptr: *mut u8, a: uint8x16_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_u16(ptr: *mut u16, a: uint16x4_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_u16(ptr: *mut u16, a: uint16x8_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_u32(ptr: *mut u32, a: uint32x2_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_u32(ptr: *mut u32, a: uint32x4_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_u64(ptr: *mut u64, a: uint64x1_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_u64(ptr: *mut u64, a: uint64x2_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_p8(ptr: *mut p8, a: poly8x8_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_p8(ptr: *mut p8, a: poly8x16_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_p16(ptr: *mut p16, a: poly16x4_t) {
    write_unaligned(ptr.cast(), a);
}

/// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_p16(ptr: *mut p16, a: poly16x8_t) {
    write_unaligned(ptr.cast(), a);
}

// Store multiple single-element structures from one, two, three, or four registers.
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vst1_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_p64(ptr: *mut p64, a: poly64x1_t) {
    write_unaligned(ptr.cast(), a);
}

// Store multiple single-element structures from one, two, three, or four registers.
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vst1q_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_p64(ptr: *mut p64, a: poly64x2_t) {
    write_unaligned(ptr.cast(), a);
}

// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_f32(ptr: *mut f32, a: float32x2_t) {
    write_unaligned(ptr.cast(), a);
}

// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_f32(ptr: *mut f32, a: float32x4_t) {
    write_unaligned(ptr.cast(), a);
}

// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1_f64(ptr: *mut f64, a: float64x1_t) {
    write_unaligned(ptr.cast(), a);
}

// Store multiple single-element structures from one, two, three, or four registers.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(str))]
#[allow(clippy::cast_ptr_alignment)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vst1q_f64(ptr: *mut f64, a: float64x2_t) {
    write_unaligned(ptr.cast(), a);
}

/// Absolute Value (wrapping).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(abs))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vabsd_s64(a: i64) -> i64 {
    vabsd_s64_(a)
}
/// Absolute Value (wrapping).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(abs))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vabs_s64(a: int64x1_t) -> int64x1_t {
    vabs_s64_(a)
}
/// Absolute Value (wrapping).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(abs))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vabsq_s64(a: int64x2_t) -> int64x2_t {
    vabsq_s64_(a)
}

/// Bitwise Select instructions. This instruction sets each bit in the destination SIMD&FP register
/// to the corresponding bit from the first source SIMD&FP register when the original
/// destination bit was 1, otherwise from the second source SIMD&FP register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(bsl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vbsl_f64(a: uint64x1_t, b: float64x1_t, c: float64x1_t) -> float64x1_t {
    let not = int64x1_t(-1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}
/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(bsl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vbsl_p64(a: poly64x1_t, b: poly64x1_t, c: poly64x1_t) -> poly64x1_t {
    let not = int64x1_t(-1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}
/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(bsl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vbslq_f64(a: uint64x2_t, b: float64x2_t, c: float64x2_t) -> float64x2_t {
    let not = int64x2_t(-1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}
/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(bsl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vbslq_p64(a: poly64x2_t, b: poly64x2_t, c: poly64x2_t) -> poly64x2_t {
    let not = int64x2_t(-1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Signed saturating Accumulate of Unsigned value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(suqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vuqadd_s8(a: int8x8_t, b: uint8x8_t) -> int8x8_t {
    vuqadd_s8_(a, b)
}
/// Signed saturating Accumulate of Unsigned value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(suqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vuqaddq_s8(a: int8x16_t, b: uint8x16_t) -> int8x16_t {
    vuqaddq_s8_(a, b)
}
/// Signed saturating Accumulate of Unsigned value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(suqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vuqadd_s16(a: int16x4_t, b: uint16x4_t) -> int16x4_t {
    vuqadd_s16_(a, b)
}
/// Signed saturating Accumulate of Unsigned value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(suqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vuqaddq_s16(a: int16x8_t, b: uint16x8_t) -> int16x8_t {
    vuqaddq_s16_(a, b)
}
/// Signed saturating Accumulate of Unsigned value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(suqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vuqadd_s32(a: int32x2_t, b: uint32x2_t) -> int32x2_t {
    vuqadd_s32_(a, b)
}
/// Signed saturating Accumulate of Unsigned value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(suqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vuqaddq_s32(a: int32x4_t, b: uint32x4_t) -> int32x4_t {
    vuqaddq_s32_(a, b)
}
/// Signed saturating Accumulate of Unsigned value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(suqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vuqadd_s64(a: int64x1_t, b: uint64x1_t) -> int64x1_t {
    vuqadd_s64_(a, b)
}
/// Signed saturating Accumulate of Unsigned value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(suqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vuqaddq_s64(a: int64x2_t, b: uint64x2_t) -> int64x2_t {
    vuqaddq_s64_(a, b)
}

/// Unsigned saturating Accumulate of Signed value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(usqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsqadd_u8(a: uint8x8_t, b: int8x8_t) -> uint8x8_t {
    vsqadd_u8_(a, b)
}
/// Unsigned saturating Accumulate of Signed value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(usqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsqaddq_u8(a: uint8x16_t, b: int8x16_t) -> uint8x16_t {
    vsqaddq_u8_(a, b)
}
/// Unsigned saturating Accumulate of Signed value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(usqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsqadd_u16(a: uint16x4_t, b: int16x4_t) -> uint16x4_t {
    vsqadd_u16_(a, b)
}
/// Unsigned saturating Accumulate of Signed value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(usqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsqaddq_u16(a: uint16x8_t, b: int16x8_t) -> uint16x8_t {
    vsqaddq_u16_(a, b)
}
/// Unsigned saturating Accumulate of Signed value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(usqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsqadd_u32(a: uint32x2_t, b: int32x2_t) -> uint32x2_t {
    vsqadd_u32_(a, b)
}
/// Unsigned saturating Accumulate of Signed value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(usqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsqaddq_u32(a: uint32x4_t, b: int32x4_t) -> uint32x4_t {
    vsqaddq_u32_(a, b)
}
/// Unsigned saturating Accumulate of Signed value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(usqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsqadd_u64(a: uint64x1_t, b: int64x1_t) -> uint64x1_t {
    vsqadd_u64_(a, b)
}
/// Unsigned saturating Accumulate of Signed value.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(usqadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsqaddq_u64(a: uint64x2_t, b: int64x2_t) -> uint64x2_t {
    vsqaddq_u64_(a, b)
}

/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    vpaddq_s16_(a, b)
}
/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    transmute(vpaddq_s16_(transmute(a), transmute(b)))
}
/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    vpaddq_s32_(a, b)
}
/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    transmute(vpaddq_s32_(transmute(a), transmute(b)))
}
/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddq_s64(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    vpaddq_s64_(a, b)
}
/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    transmute(vpaddq_s64_(transmute(a), transmute(b)))
}
/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vpaddq_s8_(a, b)
}
/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    transmute(vpaddq_s8_(transmute(a), transmute(b)))
}
/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddd_s64(a: int64x2_t) -> i64 {
    transmute(vaddvq_u64_(transmute(a)))
}
/// Add pairwise
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpaddd_u64(a: uint64x2_t) -> u64 {
    transmute(vaddvq_u64_(transmute(a)))
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddv_s16(a: int16x4_t) -> i16 {
    vaddv_s16_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddv_s32(a: int32x2_t) -> i32 {
    vaddv_s32_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddv_s8(a: int8x8_t) -> i8 {
    vaddv_s8_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddv_u16(a: uint16x4_t) -> u16 {
    vaddv_u16_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddv_u32(a: uint32x2_t) -> u32 {
    vaddv_u32_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddv_u8(a: uint8x8_t) -> u8 {
    vaddv_u8_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddvq_s16(a: int16x8_t) -> i16 {
    vaddvq_s16_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddvq_s32(a: int32x4_t) -> i32 {
    vaddvq_s32_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddvq_s8(a: int8x16_t) -> i8 {
    vaddvq_s8_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddvq_u16(a: uint16x8_t) -> u16 {
    vaddvq_u16_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddvq_u32(a: uint32x4_t) -> u32 {
    vaddvq_u32_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddvq_u8(a: uint8x16_t) -> u8 {
    vaddvq_u8_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddvq_s64(a: int64x2_t) -> i64 {
    vaddvq_s64_(a)
}

/// Add across vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(addp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddvq_u64(a: uint64x2_t) -> u64 {
    vaddvq_u64_(a)
}

/// Signed Add Long across Vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(saddlv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddlv_s8(a: int8x8_t) -> i16 {
    vaddlv_s8_(a) as i16
}

/// Signed Add Long across Vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(saddlv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddlvq_s8(a: int8x16_t) -> i16 {
    vaddlvq_s8_(a) as i16
}

/// Unsigned Add Long across Vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uaddlv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddlv_u8(a: uint8x8_t) -> u16 {
    vaddlv_u8_(a) as u16
}

/// Unsigned Add Long across Vector
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uaddlv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddlvq_u8(a: uint8x16_t) -> u16 {
    vaddlvq_u8_(a) as u16
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vadd_f64(a: float64x1_t, b: float64x1_t) -> float64x1_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fadd))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(add))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vadd_s64(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(add))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vadd_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(add))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddd_s64(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(add))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vaddd_u64(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxv_s8(a: int8x8_t) -> i8 {
    vmaxv_s8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxvq_s8(a: int8x16_t) -> i8 {
    vmaxvq_s8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxv_s16(a: int16x4_t) -> i16 {
    vmaxv_s16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxvq_s16(a: int16x8_t) -> i16 {
    vmaxvq_s16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxv_s32(a: int32x2_t) -> i32 {
    vmaxv_s32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxvq_s32(a: int32x4_t) -> i32 {
    vmaxvq_s32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxv_u8(a: uint8x8_t) -> u8 {
    vmaxv_u8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxvq_u8(a: uint8x16_t) -> u8 {
    vmaxvq_u8_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxv_u16(a: uint16x4_t) -> u16 {
    vmaxv_u16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxvq_u16(a: uint16x8_t) -> u16 {
    vmaxvq_u16_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxv_u32(a: uint32x2_t) -> u32 {
    vmaxv_u32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxvq_u32(a: uint32x4_t) -> u32 {
    vmaxvq_u32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxv_f32(a: float32x2_t) -> f32 {
    vmaxv_f32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxvq_f32(a: float32x4_t) -> f32 {
    vmaxvq_f32_(a)
}

/// Horizontal vector max.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmaxvq_f64(a: float64x2_t) -> f64 {
    vmaxvq_f64_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminv_s8(a: int8x8_t) -> i8 {
    vminv_s8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminvq_s8(a: int8x16_t) -> i8 {
    vminvq_s8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminv_s16(a: int16x4_t) -> i16 {
    vminv_s16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminvq_s16(a: int16x8_t) -> i16 {
    vminvq_s16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminv_s32(a: int32x2_t) -> i32 {
    vminv_s32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminvq_s32(a: int32x4_t) -> i32 {
    vminvq_s32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminv_u8(a: uint8x8_t) -> u8 {
    vminv_u8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminvq_u8(a: uint8x16_t) -> u8 {
    vminvq_u8_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminv_u16(a: uint16x4_t) -> u16 {
    vminv_u16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminvq_u16(a: uint16x8_t) -> u16 {
    vminvq_u16_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminv_u32(a: uint32x2_t) -> u32 {
    vminv_u32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminvq_u32(a: uint32x4_t) -> u32 {
    vminvq_u32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminv_f32(a: float32x2_t) -> f32 {
    vminv_f32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminv))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminvq_f32(a: float32x4_t) -> f32 {
    vminvq_f32_(a)
}

/// Horizontal vector min.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vminvq_f64(a: float64x2_t) -> f64 {
    vminvq_f64_(a)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpminq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vpminq_s8_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpminq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    vpminq_s16_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpminq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    vpminq_s32_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpminq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    vpminq_u8_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpminq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    vpminq_u16_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(uminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpminq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    vpminq_u32_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpminq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    vpminq_f32_(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fminp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpminq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    vpminq_f64_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpmaxq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vpmaxq_s8_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpmaxq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    vpmaxq_s16_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(smaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpmaxq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    vpmaxq_s32_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpmaxq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    vpmaxq_u8_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpmaxq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    vpmaxq_u16_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(umaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpmaxq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    vpmaxq_u32_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpmaxq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    vpmaxq_f32_(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmaxp))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vpmaxq_f64(a: float64x2_t, b: float64x2_t) -> float64x2_t {
    vpmaxq_f64_(a, b)
}

/// Extract vector from pair of vectors
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N = 0))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vext_p64<const N: i32>(a: poly64x1_t, _b: poly64x1_t) -> poly64x1_t {
    static_assert!(N == 0);
    a
}

/// Extract vector from pair of vectors
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N = 0))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vext_f64<const N: i32>(a: float64x1_t, _b: float64x1_t) -> float64x1_t {
    static_assert!(N == 0);
    a
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmov))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vdup_n_p64(value: p64) -> poly64x1_t {
    transmute(u64x1::new(value))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vdup_n_f64(value: f64) -> float64x1_t {
    float64x1_t(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(dup))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vdupq_n_p64(value: p64) -> poly64x2_t {
    transmute(u64x2::new(value, value))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(dup))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vdupq_n_f64(value: f64) -> float64x2_t {
    float64x2_t(value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(fmov))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmov_n_p64(value: p64) -> poly64x1_t {
    vdup_n_p64(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmov_n_f64(value: f64) -> float64x1_t {
    vdup_n_f64(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(dup))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmovq_n_p64(value: p64) -> poly64x2_t {
    vdupq_n_p64(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(dup))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vmovq_n_f64(value: f64) -> float64x2_t {
    vdupq_n_f64(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(mov))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vget_high_f64(a: float64x2_t) -> float64x1_t {
    float64x1_t(simd_extract(a, 1))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(ext))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vget_high_p64(a: poly64x2_t) -> poly64x1_t {
    transmute(u64x1::new(simd_extract(a, 1)))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vget_low_f64(a: float64x2_t) -> float64x1_t {
    float64x1_t(simd_extract(a, 0))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vget_low_p64(a: poly64x2_t) -> poly64x1_t {
    transmute(u64x1::new(simd_extract(a, 0)))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[rustc_legacy_const_generics(1)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(nop, IMM5 = 0))]
pub unsafe fn vget_lane_f64<const IMM5: i32>(v: float64x1_t) -> f64 {
    static_assert!(IMM5 == 0);
    simd_extract(v, IMM5 as u32)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[rustc_legacy_const_generics(1)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(nop, IMM5 = 0))]
pub unsafe fn vgetq_lane_f64<const IMM5: i32>(v: float64x2_t) -> f64 {
    static_assert_uimm_bits!(IMM5, 1);
    simd_extract(v, IMM5 as u32)
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(mov))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcombine_f64(low: float64x1_t, high: float64x1_t) -> float64x2_t {
    simd_shuffle!(low, high, [0, 1])
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl1_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    vqtbl1_s8(vcombine_s8(a, zeroed()), transmute(b))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl1_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    vqtbl1_u8(vcombine_u8(a, zeroed()), b)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl1_p8(a: poly8x8_t, b: uint8x8_t) -> poly8x8_t {
    vqtbl1_p8(vcombine_p8(a, zeroed()), b)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl2_s8(a: int8x8x2_t, b: int8x8_t) -> int8x8_t {
    vqtbl1_s8(vcombine_s8(a.0, a.1), transmute(b))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl2_u8(a: uint8x8x2_t, b: uint8x8_t) -> uint8x8_t {
    vqtbl1_u8(vcombine_u8(a.0, a.1), b)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl2_p8(a: poly8x8x2_t, b: uint8x8_t) -> poly8x8_t {
    vqtbl1_p8(vcombine_p8(a.0, a.1), b)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl3_s8(a: int8x8x3_t, b: int8x8_t) -> int8x8_t {
    vqtbl2_s8(
        int8x16x2_t(vcombine_s8(a.0, a.1), vcombine_s8(a.2, zeroed())),
        transmute(b),
    )
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl3_u8(a: uint8x8x3_t, b: uint8x8_t) -> uint8x8_t {
    vqtbl2_u8(
        uint8x16x2_t(vcombine_u8(a.0, a.1), vcombine_u8(a.2, zeroed())),
        b,
    )
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl3_p8(a: poly8x8x3_t, b: uint8x8_t) -> poly8x8_t {
    vqtbl2_p8(
        poly8x16x2_t(vcombine_p8(a.0, a.1), vcombine_p8(a.2, zeroed())),
        b,
    )
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl4_s8(a: int8x8x4_t, b: int8x8_t) -> int8x8_t {
    vqtbl2_s8(
        int8x16x2_t(vcombine_s8(a.0, a.1), vcombine_s8(a.2, a.3)),
        transmute(b),
    )
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl4_u8(a: uint8x8x4_t, b: uint8x8_t) -> uint8x8_t {
    vqtbl2_u8(
        uint8x16x2_t(vcombine_u8(a.0, a.1), vcombine_u8(a.2, a.3)),
        b,
    )
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbl4_p8(a: poly8x8x4_t, b: uint8x8_t) -> poly8x8_t {
    vqtbl2_p8(
        poly8x16x2_t(vcombine_p8(a.0, a.1), vcombine_p8(a.2, a.3)),
        b,
    )
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx1_s8(a: int8x8_t, b: int8x8_t, c: int8x8_t) -> int8x8_t {
    let r = vqtbx1_s8(a, vcombine_s8(b, zeroed()), transmute(c));
    let m: int8x8_t = simd_lt(c, transmute(i8x8::splat(8)));
    simd_select(m, r, a)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx1_u8(a: uint8x8_t, b: uint8x8_t, c: uint8x8_t) -> uint8x8_t {
    let r = vqtbx1_u8(a, vcombine_u8(b, zeroed()), c);
    let m: int8x8_t = simd_lt(c, transmute(u8x8::splat(8)));
    simd_select(m, r, a)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx1_p8(a: poly8x8_t, b: poly8x8_t, c: uint8x8_t) -> poly8x8_t {
    let r = vqtbx1_p8(a, vcombine_p8(b, zeroed()), c);
    let m: int8x8_t = simd_lt(c, transmute(u8x8::splat(8)));
    simd_select(m, r, a)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx2_s8(a: int8x8_t, b: int8x8x2_t, c: int8x8_t) -> int8x8_t {
    vqtbx1_s8(a, vcombine_s8(b.0, b.1), transmute(c))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx2_u8(a: uint8x8_t, b: uint8x8x2_t, c: uint8x8_t) -> uint8x8_t {
    vqtbx1_u8(a, vcombine_u8(b.0, b.1), c)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx2_p8(a: poly8x8_t, b: poly8x8x2_t, c: uint8x8_t) -> poly8x8_t {
    vqtbx1_p8(a, vcombine_p8(b.0, b.1), c)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx3_s8(a: int8x8_t, b: int8x8x3_t, c: int8x8_t) -> int8x8_t {
    let r = vqtbx2_s8(
        a,
        int8x16x2_t(vcombine_s8(b.0, b.1), vcombine_s8(b.2, zeroed())),
        transmute(c),
    );
    let m: int8x8_t = simd_lt(c, transmute(i8x8::splat(24)));
    simd_select(m, r, a)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx3_u8(a: uint8x8_t, b: uint8x8x3_t, c: uint8x8_t) -> uint8x8_t {
    let r = vqtbx2_u8(
        a,
        uint8x16x2_t(vcombine_u8(b.0, b.1), vcombine_u8(b.2, zeroed())),
        c,
    );
    let m: int8x8_t = simd_lt(c, transmute(u8x8::splat(24)));
    simd_select(m, r, a)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx3_p8(a: poly8x8_t, b: poly8x8x3_t, c: uint8x8_t) -> poly8x8_t {
    let r = vqtbx2_p8(
        a,
        poly8x16x2_t(vcombine_p8(b.0, b.1), vcombine_p8(b.2, zeroed())),
        c,
    );
    let m: int8x8_t = simd_lt(c, transmute(u8x8::splat(24)));
    simd_select(m, r, a)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx4_s8(a: int8x8_t, b: int8x8x4_t, c: int8x8_t) -> int8x8_t {
    vqtbx2_s8(
        a,
        int8x16x2_t(vcombine_s8(b.0, b.1), vcombine_s8(b.2, b.3)),
        transmute(c),
    )
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx4_u8(a: uint8x8_t, b: uint8x8x4_t, c: uint8x8_t) -> uint8x8_t {
    vqtbx2_u8(
        a,
        uint8x16x2_t(vcombine_u8(b.0, b.1), vcombine_u8(b.2, b.3)),
        c,
    )
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vtbx4_p8(a: poly8x8_t, b: poly8x8x4_t, c: uint8x8_t) -> poly8x8_t {
    vqtbx2_p8(
        a,
        poly8x16x2_t(vcombine_p8(b.0, b.1), vcombine_p8(b.2, b.3)),
        c,
    )
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl1_s8(t: int8x16_t, idx: uint8x8_t) -> int8x8_t {
    vqtbl1(t, idx)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl1q_s8(t: int8x16_t, idx: uint8x16_t) -> int8x16_t {
    vqtbl1q(t, idx)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl1_u8(t: uint8x16_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbl1(transmute(t), transmute(idx)))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl1q_u8(t: uint8x16_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbl1q(transmute(t), transmute(idx)))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl1_p8(t: poly8x16_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbl1(transmute(t), transmute(idx)))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl1q_p8(t: poly8x16_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbl1q(transmute(t), transmute(idx)))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx1_s8(a: int8x8_t, t: int8x16_t, idx: uint8x8_t) -> int8x8_t {
    vqtbx1(a, t, idx)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx1q_s8(a: int8x16_t, t: int8x16_t, idx: uint8x16_t) -> int8x16_t {
    vqtbx1q(a, t, idx)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx1_u8(a: uint8x8_t, t: uint8x16_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbx1(transmute(a), transmute(t), transmute(idx)))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx1q_u8(a: uint8x16_t, t: uint8x16_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbx1q(transmute(a), transmute(t), transmute(idx)))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx1_p8(a: poly8x8_t, t: poly8x16_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbx1(transmute(a), transmute(t), transmute(idx)))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx1q_p8(a: poly8x16_t, t: poly8x16_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbx1q(transmute(a), transmute(t), transmute(idx)))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl2_s8(t: int8x16x2_t, idx: uint8x8_t) -> int8x8_t {
    vqtbl2(t.0, t.1, idx)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl2q_s8(t: int8x16x2_t, idx: uint8x16_t) -> int8x16_t {
    vqtbl2q(t.0, t.1, idx)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl2_u8(t: uint8x16x2_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbl2(transmute(t.0), transmute(t.1), transmute(idx)))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl2q_u8(t: uint8x16x2_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbl2q(transmute(t.0), transmute(t.1), transmute(idx)))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl2_p8(t: poly8x16x2_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbl2(transmute(t.0), transmute(t.1), transmute(idx)))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl2q_p8(t: poly8x16x2_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbl2q(transmute(t.0), transmute(t.1), transmute(idx)))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx2_s8(a: int8x8_t, t: int8x16x2_t, idx: uint8x8_t) -> int8x8_t {
    vqtbx2(a, t.0, t.1, idx)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx2q_s8(a: int8x16_t, t: int8x16x2_t, idx: uint8x16_t) -> int8x16_t {
    vqtbx2q(a, t.0, t.1, idx)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx2_u8(a: uint8x8_t, t: uint8x16x2_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbx2(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx2q_u8(a: uint8x16_t, t: uint8x16x2_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbx2q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx2_p8(a: poly8x8_t, t: poly8x16x2_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbx2(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx2q_p8(a: poly8x16_t, t: poly8x16x2_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbx2q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl3_s8(t: int8x16x3_t, idx: uint8x8_t) -> int8x8_t {
    vqtbl3(t.0, t.1, t.2, idx)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl3q_s8(t: int8x16x3_t, idx: uint8x16_t) -> int8x16_t {
    vqtbl3q(t.0, t.1, t.2, idx)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl3_u8(t: uint8x16x3_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbl3(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl3q_u8(t: uint8x16x3_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbl3q(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl3_p8(t: poly8x16x3_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbl3(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl3q_p8(t: poly8x16x3_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbl3q(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx3_s8(a: int8x8_t, t: int8x16x3_t, idx: uint8x8_t) -> int8x8_t {
    vqtbx3(a, t.0, t.1, t.2, idx)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx3q_s8(a: int8x16_t, t: int8x16x3_t, idx: uint8x16_t) -> int8x16_t {
    vqtbx3q(a, t.0, t.1, t.2, idx)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx3_u8(a: uint8x8_t, t: uint8x16x3_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbx3(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx3q_u8(a: uint8x16_t, t: uint8x16x3_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbx3q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx3_p8(a: poly8x8_t, t: poly8x16x3_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbx3(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx3q_p8(a: poly8x16_t, t: poly8x16x3_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbx3q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl4_s8(t: int8x16x4_t, idx: uint8x8_t) -> int8x8_t {
    vqtbl4(t.0, t.1, t.2, t.3, idx)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl4q_s8(t: int8x16x4_t, idx: uint8x16_t) -> int8x16_t {
    vqtbl4q(t.0, t.1, t.2, t.3, idx)
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl4_u8(t: uint8x16x4_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbl4(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl4q_u8(t: uint8x16x4_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbl4q(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl4_p8(t: poly8x16x4_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbl4(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}

/// Table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbl))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbl4q_p8(t: poly8x16x4_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbl4q(
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx4_s8(a: int8x8_t, t: int8x16x4_t, idx: uint8x8_t) -> int8x8_t {
    vqtbx4(a, t.0, t.1, t.2, t.3, idx)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx4q_s8(a: int8x16_t, t: int8x16x4_t, idx: uint8x16_t) -> int8x16_t {
    vqtbx4q(a, t.0, t.1, t.2, t.3, idx)
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx4_u8(a: uint8x8_t, t: uint8x16x4_t, idx: uint8x8_t) -> uint8x8_t {
    transmute(vqtbx4(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx4q_u8(a: uint8x16_t, t: uint8x16x4_t, idx: uint8x16_t) -> uint8x16_t {
    transmute(vqtbx4q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx4_p8(a: poly8x8_t, t: poly8x16x4_t, idx: uint8x8_t) -> poly8x8_t {
    transmute(vqtbx4(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}

/// Extended table look-up
#[inline]
#[cfg(target_endian = "little")]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(tbx))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vqtbx4q_p8(a: poly8x16_t, t: poly8x16x4_t, idx: uint8x16_t) -> poly8x16_t {
    transmute(vqtbx4q(
        transmute(a),
        transmute(t.0),
        transmute(t.1),
        transmute(t.2),
        transmute(t.3),
        transmute(idx),
    ))
}

/// Shift left
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N = 2))]
#[rustc_legacy_const_generics(1)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vshld_n_s64<const N: i32>(a: i64) -> i64 {
    static_assert_uimm_bits!(N, 6);
    a << N
}

/// Shift left
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N = 2))]
#[rustc_legacy_const_generics(1)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vshld_n_u64<const N: i32>(a: u64) -> u64 {
    static_assert_uimm_bits!(N, 6);
    a << N
}

/// Signed shift right
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N = 2))]
#[rustc_legacy_const_generics(1)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vshrd_n_s64<const N: i32>(a: i64) -> i64 {
    static_assert!(N >= 1 && N <= 64);
    let n: i32 = if N == 64 { 63 } else { N };
    a >> n
}

/// Unsigned shift right
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N = 2))]
#[rustc_legacy_const_generics(1)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vshrd_n_u64<const N: i32>(a: u64) -> u64 {
    static_assert!(N >= 1 && N <= 64);
    let n: i32 = if N == 64 {
        return 0;
    } else {
        N
    };
    a >> n
}

/// Signed shift right and accumulate
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N = 2))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsrad_n_s64<const N: i32>(a: i64, b: i64) -> i64 {
    static_assert!(N >= 1 && N <= 64);
    a.wrapping_add(vshrd_n_s64::<N>(b))
}

/// Unsigned shift right and accumulate
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(nop, N = 2))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsrad_n_u64<const N: i32>(a: u64, b: u64) -> u64 {
    static_assert!(N >= 1 && N <= 64);
    a.wrapping_add(vshrd_n_u64::<N>(b))
}

/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_s8<const N: i32>(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    static_assert_uimm_bits!(N, 3);
    vsli_n_s8_(a, b, N)
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_s8<const N: i32>(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    static_assert_uimm_bits!(N, 3);
    vsliq_n_s8_(a, b, N)
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_s16<const N: i32>(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    static_assert_uimm_bits!(N, 4);
    vsli_n_s16_(a, b, N)
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_s16<const N: i32>(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    static_assert_uimm_bits!(N, 4);
    vsliq_n_s16_(a, b, N)
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_s32<const N: i32>(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    static_assert!(N >= 0 && N <= 31);
    vsli_n_s32_(a, b, N)
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_s32<const N: i32>(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    static_assert!(N >= 0 && N <= 31);
    vsliq_n_s32_(a, b, N)
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_s64<const N: i32>(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    static_assert!(N >= 0 && N <= 63);
    vsli_n_s64_(a, b, N)
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_s64<const N: i32>(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    static_assert!(N >= 0 && N <= 63);
    vsliq_n_s64_(a, b, N)
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_u8<const N: i32>(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    static_assert_uimm_bits!(N, 3);
    transmute(vsli_n_s8_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_u8<const N: i32>(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    static_assert_uimm_bits!(N, 3);
    transmute(vsliq_n_s8_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_u16<const N: i32>(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    static_assert_uimm_bits!(N, 4);
    transmute(vsli_n_s16_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_u16<const N: i32>(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    static_assert_uimm_bits!(N, 4);
    transmute(vsliq_n_s16_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_u32<const N: i32>(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    static_assert!(N >= 0 && N <= 31);
    transmute(vsli_n_s32_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_u32<const N: i32>(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    static_assert!(N >= 0 && N <= 31);
    transmute(vsliq_n_s32_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_u64<const N: i32>(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    static_assert!(N >= 0 && N <= 63);
    transmute(vsli_n_s64_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_u64<const N: i32>(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    static_assert!(N >= 0 && N <= 63);
    transmute(vsliq_n_s64_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_p8<const N: i32>(a: poly8x8_t, b: poly8x8_t) -> poly8x8_t {
    static_assert_uimm_bits!(N, 3);
    transmute(vsli_n_s8_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_p8<const N: i32>(a: poly8x16_t, b: poly8x16_t) -> poly8x16_t {
    static_assert_uimm_bits!(N, 3);
    transmute(vsliq_n_s8_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_p16<const N: i32>(a: poly16x4_t, b: poly16x4_t) -> poly16x4_t {
    static_assert_uimm_bits!(N, 4);
    transmute(vsli_n_s16_(transmute(a), transmute(b), N))
}
/// Shift Left and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_p16<const N: i32>(a: poly16x8_t, b: poly16x8_t) -> poly16x8_t {
    static_assert_uimm_bits!(N, 4);
    transmute(vsliq_n_s16_(transmute(a), transmute(b), N))
}

/// Shift Left and Insert (immediate)
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vsli_n_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsli_n_p64<const N: i32>(a: poly64x1_t, b: poly64x1_t) -> poly64x1_t {
    static_assert!(N >= 0 && N <= 63);
    transmute(vsli_n_s64_(transmute(a), transmute(b), N))
}

/// Shift Left and Insert (immediate)
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vsliq_n_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(test, assert_instr(sli, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsliq_n_p64<const N: i32>(a: poly64x2_t, b: poly64x2_t) -> poly64x2_t {
    static_assert!(N >= 0 && N <= 63);
    transmute(vsliq_n_s64_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_s8<const N: i32>(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    static_assert!(N >= 1 && N <= 8);
    vsri_n_s8_(a, b, N)
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_s8<const N: i32>(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    static_assert!(N >= 1 && N <= 8);
    vsriq_n_s8_(a, b, N)
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_s16<const N: i32>(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    static_assert!(N >= 1 && N <= 16);
    vsri_n_s16_(a, b, N)
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_s16<const N: i32>(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    static_assert!(N >= 1 && N <= 16);
    vsriq_n_s16_(a, b, N)
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_s32<const N: i32>(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    static_assert!(N >= 1 && N <= 32);
    vsri_n_s32_(a, b, N)
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_s32<const N: i32>(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    static_assert!(N >= 1 && N <= 32);
    vsriq_n_s32_(a, b, N)
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_s64<const N: i32>(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    static_assert!(N >= 1 && N <= 64);
    vsri_n_s64_(a, b, N)
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_s64<const N: i32>(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    static_assert!(N >= 1 && N <= 64);
    vsriq_n_s64_(a, b, N)
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_u8<const N: i32>(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    static_assert!(N >= 1 && N <= 8);
    transmute(vsri_n_s8_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_u8<const N: i32>(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    static_assert!(N >= 1 && N <= 8);
    transmute(vsriq_n_s8_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_u16<const N: i32>(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    static_assert!(N >= 1 && N <= 16);
    transmute(vsri_n_s16_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_u16<const N: i32>(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    static_assert!(N >= 1 && N <= 16);
    transmute(vsriq_n_s16_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_u32<const N: i32>(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    static_assert!(N >= 1 && N <= 32);
    transmute(vsri_n_s32_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_u32<const N: i32>(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    static_assert!(N >= 1 && N <= 32);
    transmute(vsriq_n_s32_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_u64<const N: i32>(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    static_assert!(N >= 1 && N <= 64);
    transmute(vsri_n_s64_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_u64<const N: i32>(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    static_assert!(N >= 1 && N <= 64);
    transmute(vsriq_n_s64_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_p8<const N: i32>(a: poly8x8_t, b: poly8x8_t) -> poly8x8_t {
    static_assert!(N >= 1 && N <= 8);
    transmute(vsri_n_s8_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_p8<const N: i32>(a: poly8x16_t, b: poly8x16_t) -> poly8x16_t {
    static_assert!(N >= 1 && N <= 8);
    transmute(vsriq_n_s8_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_p16<const N: i32>(a: poly16x4_t, b: poly16x4_t) -> poly16x4_t {
    static_assert!(N >= 1 && N <= 16);
    transmute(vsri_n_s16_(transmute(a), transmute(b), N))
}
/// Shift Right and Insert (immediate)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_p16<const N: i32>(a: poly16x8_t, b: poly16x8_t) -> poly16x8_t {
    static_assert!(N >= 1 && N <= 16);
    transmute(vsriq_n_s16_(transmute(a), transmute(b), N))
}

/// Shift Right and Insert (immediate)
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vsri_n_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsri_n_p64<const N: i32>(a: poly64x1_t, b: poly64x1_t) -> poly64x1_t {
    static_assert!(N >= 1 && N <= 64);
    transmute(vsri_n_s64_(transmute(a), transmute(b), N))
}

/// Shift Right and Insert (immediate)
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vsriq_n_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(test, assert_instr(sri, N = 1))]
#[rustc_legacy_const_generics(2)]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vsriq_n_p64<const N: i32>(a: poly64x2_t, b: poly64x2_t) -> poly64x2_t {
    static_assert!(N >= 1 && N <= 64);
    transmute(vsriq_n_s64_(transmute(a), transmute(b), N))
}

/// SM3TT1A
#[inline]
#[target_feature(enable = "neon,sm4")]
#[cfg_attr(test, assert_instr(sm3tt1a, IMM2 = 0))]
#[rustc_legacy_const_generics(3)]
pub unsafe fn vsm3tt1aq_u32<const IMM2: i32>(
    a: uint32x4_t,
    b: uint32x4_t,
    c: uint32x4_t,
) -> uint32x4_t {
    static_assert_uimm_bits!(IMM2, 2);
    #[allow(improper_ctypes)]
    extern "unadjusted" {
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.crypto.sm3tt1a")]
        fn vsm3tt1aq_u32_(a: uint32x4_t, b: uint32x4_t, c: uint32x4_t, imm2: i64) -> uint32x4_t;
    }
    vsm3tt1aq_u32_(a, b, c, IMM2 as i64)
}

/// SM3TT1B
#[inline]
#[target_feature(enable = "neon,sm4")]
#[cfg_attr(test, assert_instr(sm3tt1b, IMM2 = 0))]
#[rustc_legacy_const_generics(3)]
pub unsafe fn vsm3tt1bq_u32<const IMM2: i32>(
    a: uint32x4_t,
    b: uint32x4_t,
    c: uint32x4_t,
) -> uint32x4_t {
    static_assert_uimm_bits!(IMM2, 2);
    #[allow(improper_ctypes)]
    extern "unadjusted" {
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.crypto.sm3tt1b")]
        fn vsm3tt1bq_u32_(a: uint32x4_t, b: uint32x4_t, c: uint32x4_t, imm2: i64) -> uint32x4_t;
    }
    vsm3tt1bq_u32_(a, b, c, IMM2 as i64)
}

/// SM3TT2A
#[inline]
#[target_feature(enable = "neon,sm4")]
#[cfg_attr(test, assert_instr(sm3tt2a, IMM2 = 0))]
#[rustc_legacy_const_generics(3)]
pub unsafe fn vsm3tt2aq_u32<const IMM2: i32>(
    a: uint32x4_t,
    b: uint32x4_t,
    c: uint32x4_t,
) -> uint32x4_t {
    static_assert_uimm_bits!(IMM2, 2);
    #[allow(improper_ctypes)]
    extern "unadjusted" {
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.crypto.sm3tt2a")]
        fn vsm3tt2aq_u32_(a: uint32x4_t, b: uint32x4_t, c: uint32x4_t, imm2: i64) -> uint32x4_t;
    }
    vsm3tt2aq_u32_(a, b, c, IMM2 as i64)
}

/// SM3TT2B
#[inline]
#[target_feature(enable = "neon,sm4")]
#[cfg_attr(test, assert_instr(sm3tt2b, IMM2 = 0))]
#[rustc_legacy_const_generics(3)]
pub unsafe fn vsm3tt2bq_u32<const IMM2: i32>(
    a: uint32x4_t,
    b: uint32x4_t,
    c: uint32x4_t,
) -> uint32x4_t {
    static_assert_uimm_bits!(IMM2, 2);
    #[allow(improper_ctypes)]
    extern "unadjusted" {
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.crypto.sm3tt2b")]
        fn vsm3tt2bq_u32_(a: uint32x4_t, b: uint32x4_t, c: uint32x4_t, imm2: i64) -> uint32x4_t;
    }
    vsm3tt2bq_u32_(a, b, c, IMM2 as i64)
}

/// Exclusive OR and rotate
#[inline]
#[target_feature(enable = "neon,sha3")]
#[cfg_attr(test, assert_instr(xar, IMM6 = 0))]
#[rustc_legacy_const_generics(2)]
pub unsafe fn vxarq_u64<const IMM6: i32>(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    static_assert_uimm_bits!(IMM6, 6);
    #[allow(improper_ctypes)]
    extern "unadjusted" {
        #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.crypto.xar")]
        fn vxarq_u64_(a: uint64x2_t, b: uint64x2_t, n: i64) -> uint64x2_t;
    }
    vxarq_u64_(a, b, IMM6 as i64)
}

#[cfg(test)]
mod tests {
    use crate::core_arch::aarch64::test_support::*;
    use crate::core_arch::arm_shared::test_support::*;
    use crate::core_arch::{aarch64::neon::*, aarch64::*, simd::*};
    use std::mem::transmute;
    use stdarch_test::simd_test;

    #[simd_test(enable = "neon")]
    unsafe fn test_vuqadd_s8() {
        let a = i8x8::new(i8::MIN, -3, -2, -1, 0, 1, 2, i8::MAX);
        let b = u8x8::new(u8::MAX, 1, 2, 3, 4, 5, 6, 7);
        let e = i8x8::new(i8::MAX, -2, 0, 2, 4, 6, 8, i8::MAX);
        let r: i8x8 = transmute(vuqadd_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vuqaddq_s8() {
        let a = i8x16::new(
            i8::MIN,
            -7,
            -6,
            -5,
            -4,
            -3,
            -2,
            -1,
            0,
            1,
            2,
            3,
            4,
            5,
            6,
            i8::MAX,
        );
        let b = u8x16::new(u8::MAX, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let e = i8x16::new(
            i8::MAX,
            -6,
            -4,
            -2,
            0,
            2,
            4,
            6,
            8,
            10,
            12,
            14,
            16,
            18,
            20,
            i8::MAX,
        );
        let r: i8x16 = transmute(vuqaddq_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vuqadd_s16() {
        let a = i16x4::new(i16::MIN, -1, 0, i16::MAX);
        let b = u16x4::new(u16::MAX, 1, 2, 3);
        let e = i16x4::new(i16::MAX, 0, 2, i16::MAX);
        let r: i16x4 = transmute(vuqadd_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vuqaddq_s16() {
        let a = i16x8::new(i16::MIN, -3, -2, -1, 0, 1, 2, i16::MAX);
        let b = u16x8::new(u16::MAX, 1, 2, 3, 4, 5, 6, 7);
        let e = i16x8::new(i16::MAX, -2, 0, 2, 4, 6, 8, i16::MAX);
        let r: i16x8 = transmute(vuqaddq_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vuqadd_s32() {
        let a = i32x2::new(i32::MIN, i32::MAX);
        let b = u32x2::new(u32::MAX, 1);
        let e = i32x2::new(i32::MAX, i32::MAX);
        let r: i32x2 = transmute(vuqadd_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vuqaddq_s32() {
        let a = i32x4::new(i32::MIN, -1, 0, i32::MAX);
        let b = u32x4::new(u32::MAX, 1, 2, 3);
        let e = i32x4::new(i32::MAX, 0, 2, i32::MAX);
        let r: i32x4 = transmute(vuqaddq_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vuqadd_s64() {
        let a = i64x1::new(i64::MIN);
        let b = u64x1::new(u64::MAX);
        let e = i64x1::new(i64::MAX);
        let r: i64x1 = transmute(vuqadd_s64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vuqaddq_s64() {
        let a = i64x2::new(i64::MIN, i64::MAX);
        let b = u64x2::new(u64::MAX, 1);
        let e = i64x2::new(i64::MAX, i64::MAX);
        let r: i64x2 = transmute(vuqaddq_s64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vsqadd_u8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, u8::MAX);
        let b = i8x8::new(i8::MIN, -3, -2, -1, 0, 1, 2, 3);
        let e = u8x8::new(0, 0, 0, 2, 4, 6, 8, u8::MAX);
        let r: u8x8 = transmute(vsqadd_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsqaddq_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, u8::MAX);
        let b = i8x16::new(i8::MIN, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7);
        let e = u8x16::new(0, 0, 0, 0, 0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, u8::MAX);
        let r: u8x16 = transmute(vsqaddq_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsqadd_u16() {
        let a = u16x4::new(0, 1, 2, u16::MAX);
        let b = i16x4::new(i16::MIN, -1, 0, 1);
        let e = u16x4::new(0, 0, 2, u16::MAX);
        let r: u16x4 = transmute(vsqadd_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsqaddq_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, u16::MAX);
        let b = i16x8::new(i16::MIN, -3, -2, -1, 0, 1, 2, 3);
        let e = u16x8::new(0, 0, 0, 2, 4, 6, 8, u16::MAX);
        let r: u16x8 = transmute(vsqaddq_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsqadd_u32() {
        let a = u32x2::new(0, u32::MAX);
        let b = i32x2::new(i32::MIN, 1);
        let e = u32x2::new(0, u32::MAX);
        let r: u32x2 = transmute(vsqadd_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsqaddq_u32() {
        let a = u32x4::new(0, 1, 2, u32::MAX);
        let b = i32x4::new(i32::MIN, -1, 0, 1);
        let e = u32x4::new(0, 0, 2, u32::MAX);
        let r: u32x4 = transmute(vsqaddq_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsqadd_u64() {
        let a = u64x1::new(0);
        let b = i64x1::new(i64::MIN);
        let e = u64x1::new(0);
        let r: u64x1 = transmute(vsqadd_u64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsqaddq_u64() {
        let a = u64x2::new(0, u64::MAX);
        let b = i64x2::new(i64::MIN, 1);
        let e = u64x2::new(0, u64::MAX);
        let r: u64x2 = transmute(vsqaddq_u64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddq_s16() {
        let a = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = i16x8::new(0, -1, -2, -3, -4, -5, -6, -7);
        let r: i16x8 = transmute(vpaddq_s16(transmute(a), transmute(b)));
        let e = i16x8::new(3, 7, 11, 15, -1, -5, -9, -13);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddq_s32() {
        let a = i32x4::new(1, 2, 3, 4);
        let b = i32x4::new(0, -1, -2, -3);
        let r: i32x4 = transmute(vpaddq_s32(transmute(a), transmute(b)));
        let e = i32x4::new(3, 7, -1, -5);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddq_s64() {
        let a = i64x2::new(1, 2);
        let b = i64x2::new(0, -1);
        let r: i64x2 = transmute(vpaddq_s64(transmute(a), transmute(b)));
        let e = i64x2::new(3, -1);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddq_s8() {
        let a = i8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let b = i8x16::new(
            0, -1, -2, -3, -4, -5, -6, -7, -8, -8, -10, -11, -12, -13, -14, -15,
        );
        let r: i8x16 = transmute(vpaddq_s8(transmute(a), transmute(b)));
        let e = i8x16::new(
            3, 7, 11, 15, 19, 23, 27, 31, -1, -5, -9, -13, -16, -21, -25, -29,
        );
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddq_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let b = u16x8::new(17, 18, 19, 20, 20, 21, 22, 23);
        let r: u16x8 = transmute(vpaddq_u16(transmute(a), transmute(b)));
        let e = u16x8::new(1, 5, 9, 13, 35, 39, 41, 45);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddq_u32() {
        let a = u32x4::new(0, 1, 2, 3);
        let b = u32x4::new(17, 18, 19, 20);
        let r: u32x4 = transmute(vpaddq_u32(transmute(a), transmute(b)));
        let e = u32x4::new(1, 5, 35, 39);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddq_u64() {
        let a = u64x2::new(0, 1);
        let b = u64x2::new(17, 18);
        let r: u64x2 = transmute(vpaddq_u64(transmute(a), transmute(b)));
        let e = u64x2::new(1, 35);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddq_u8() {
        let a = i8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let b = i8x16::new(
            17, 18, 19, 20, 20, 21, 22, 23, 24, 25, 26, 27, 29, 29, 30, 31,
        );
        let r = i8x16(1, 5, 9, 13, 17, 21, 25, 29, 35, 39, 41, 45, 49, 53, 58, 61);
        let e: i8x16 = transmute(vpaddq_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddd_s64() {
        let a = i64x2::new(2, -3);
        let r: i64 = transmute(vpaddd_s64(transmute(a)));
        let e = -1_i64;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddd_u64() {
        let a = i64x2::new(2, 3);
        let r: u64 = transmute(vpaddd_u64(transmute(a)));
        let e = 5_u64;
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_f64() {
        let a = 1.;
        let b = 8.;
        let e = 9.;
        let r: f64 = transmute(vadd_f64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddq_f64() {
        let a = f64x2::new(1., 2.);
        let b = f64x2::new(8., 7.);
        let e = f64x2::new(9., 9.);
        let r: f64x2 = transmute(vaddq_f64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_s64() {
        let a = 1_i64;
        let b = 8_i64;
        let e = 9_i64;
        let r: i64 = transmute(vadd_s64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_u64() {
        let a = 1_u64;
        let b = 8_u64;
        let e = 9_u64;
        let r: u64 = transmute(vadd_u64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddd_s64() {
        let a = 1_i64;
        let b = 8_i64;
        let e = 9_i64;
        let r: i64 = transmute(vaddd_s64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddd_u64() {
        let a = 1_u64;
        let b = 8_u64;
        let e = 9_u64;
        let r: u64 = transmute(vaddd_u64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_s8() {
        let r = vmaxv_s8(transmute(i8x8::new(1, 2, 3, 4, -8, 6, 7, 5)));
        assert_eq!(r, 7_i8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_s8() {
        #[rustfmt::skip]
        let r = vmaxvq_s8(transmute(i8x16::new(
            1, 2, 3, 4,
            -16, 6, 7, 5,
            8, 1, 1, 1,
            1, 1, 1, 1,
        )));
        assert_eq!(r, 8_i8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_s16() {
        let r = vmaxv_s16(transmute(i16x4::new(1, 2, -4, 3)));
        assert_eq!(r, 3_i16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_s16() {
        let r = vmaxvq_s16(transmute(i16x8::new(1, 2, 7, 4, -16, 6, 7, 5)));
        assert_eq!(r, 7_i16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_s32() {
        let r = vmaxv_s32(transmute(i32x2::new(1, -4)));
        assert_eq!(r, 1_i32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_s32() {
        let r = vmaxvq_s32(transmute(i32x4::new(1, 2, -32, 4)));
        assert_eq!(r, 4_i32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_u8() {
        let r = vmaxv_u8(transmute(u8x8::new(1, 2, 3, 4, 8, 6, 7, 5)));
        assert_eq!(r, 8_u8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_u8() {
        #[rustfmt::skip]
        let r = vmaxvq_u8(transmute(u8x16::new(
            1, 2, 3, 4,
            16, 6, 7, 5,
            8, 1, 1, 1,
            1, 1, 1, 1,
        )));
        assert_eq!(r, 16_u8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_u16() {
        let r = vmaxv_u16(transmute(u16x4::new(1, 2, 4, 3)));
        assert_eq!(r, 4_u16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_u16() {
        let r = vmaxvq_u16(transmute(u16x8::new(1, 2, 7, 4, 16, 6, 7, 5)));
        assert_eq!(r, 16_u16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_u32() {
        let r = vmaxv_u32(transmute(u32x2::new(1, 4)));
        assert_eq!(r, 4_u32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_u32() {
        let r = vmaxvq_u32(transmute(u32x4::new(1, 2, 32, 4)));
        assert_eq!(r, 32_u32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxv_f32() {
        let r = vmaxv_f32(transmute(f32x2::new(1., 4.)));
        assert_eq!(r, 4_f32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_f32() {
        let r = vmaxvq_f32(transmute(f32x4::new(1., 2., 32., 4.)));
        assert_eq!(r, 32_f32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmaxvq_f64() {
        let r = vmaxvq_f64(transmute(f64x2::new(1., 4.)));
        assert_eq!(r, 4_f64);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_s8() {
        let r = vminv_s8(transmute(i8x8::new(1, 2, 3, 4, -8, 6, 7, 5)));
        assert_eq!(r, -8_i8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_s8() {
        #[rustfmt::skip]
        let r = vminvq_s8(transmute(i8x16::new(
            1, 2, 3, 4,
            -16, 6, 7, 5,
            8, 1, 1, 1,
            1, 1, 1, 1,
        )));
        assert_eq!(r, -16_i8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_s16() {
        let r = vminv_s16(transmute(i16x4::new(1, 2, -4, 3)));
        assert_eq!(r, -4_i16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_s16() {
        let r = vminvq_s16(transmute(i16x8::new(1, 2, 7, 4, -16, 6, 7, 5)));
        assert_eq!(r, -16_i16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_s32() {
        let r = vminv_s32(transmute(i32x2::new(1, -4)));
        assert_eq!(r, -4_i32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_s32() {
        let r = vminvq_s32(transmute(i32x4::new(1, 2, -32, 4)));
        assert_eq!(r, -32_i32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_u8() {
        let r = vminv_u8(transmute(u8x8::new(1, 2, 3, 4, 8, 6, 7, 5)));
        assert_eq!(r, 1_u8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_u8() {
        #[rustfmt::skip]
        let r = vminvq_u8(transmute(u8x16::new(
            1, 2, 3, 4,
            16, 6, 7, 5,
            8, 1, 1, 1,
            1, 1, 1, 1,
        )));
        assert_eq!(r, 1_u8);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_u16() {
        let r = vminv_u16(transmute(u16x4::new(1, 2, 4, 3)));
        assert_eq!(r, 1_u16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_u16() {
        let r = vminvq_u16(transmute(u16x8::new(1, 2, 7, 4, 16, 6, 7, 5)));
        assert_eq!(r, 1_u16);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_u32() {
        let r = vminv_u32(transmute(u32x2::new(1, 4)));
        assert_eq!(r, 1_u32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_u32() {
        let r = vminvq_u32(transmute(u32x4::new(1, 2, 32, 4)));
        assert_eq!(r, 1_u32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminv_f32() {
        let r = vminv_f32(transmute(f32x2::new(1., 4.)));
        assert_eq!(r, 1_f32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_f32() {
        let r = vminvq_f32(transmute(f32x4::new(1., 2., 32., 4.)));
        assert_eq!(r, 1_f32);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vminvq_f64() {
        let r = vminvq_f64(transmute(f64x2::new(1., 4.)));
        assert_eq!(r, 1_f64);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_s8() {
        #[cfg_attr(rustfmt, skip)]
        let a = i8x16::new(1, -2, 3, -4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8);
        #[cfg_attr(rustfmt, skip)]
        let b = i8x16::new(0, 3, 2, 5, 4, 7, 6, 9, 0, 3, 2, 5, 4, 7, 6, 9);
        #[cfg_attr(rustfmt, skip)]
        let e = i8x16::new(-2, -4, 5, 7, 1, 3, 5, 7, 0, 2, 4, 6, 0, 2, 4, 6);
        let r: i8x16 = transmute(vpminq_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_s16() {
        let a = i16x8::new(1, -2, 3, 4, 5, 6, 7, 8);
        let b = i16x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = i16x8::new(-2, 3, 5, 7, 0, 2, 4, 6);
        let r: i16x8 = transmute(vpminq_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_s32() {
        let a = i32x4::new(1, -2, 3, 4);
        let b = i32x4::new(0, 3, 2, 5);
        let e = i32x4::new(-2, 3, 0, 2);
        let r: i32x4 = transmute(vpminq_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_u8() {
        #[cfg_attr(rustfmt, skip)]
        let a = u8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8);
        #[cfg_attr(rustfmt, skip)]
        let b = u8x16::new(0, 3, 2, 5, 4, 7, 6, 9, 0, 3, 2, 5, 4, 7, 6, 9);
        #[cfg_attr(rustfmt, skip)]
        let e = u8x16::new(1, 3, 5, 7, 1, 3, 5, 7, 0, 2, 4, 6, 0, 2, 4, 6);
        let r: u8x16 = transmute(vpminq_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_u16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = u16x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = u16x8::new(1, 3, 5, 7, 0, 2, 4, 6);
        let r: u16x8 = transmute(vpminq_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpminq_u32() {
        let a = u32x4::new(1, 2, 3, 4);
        let b = u32x4::new(0, 3, 2, 5);
        let e = u32x4::new(1, 3, 0, 2);
        let r: u32x4 = transmute(vpminq_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_f32() {
        let a = f32x4::new(1., -2., 3., 4.);
        let b = f32x4::new(0., 3., 2., 5.);
        let e = f32x4::new(-2., 3., 0., 2.);
        let r: f32x4 = transmute(vpminq_f32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_f64() {
        let a = f64x2::new(1., -2.);
        let b = f64x2::new(0., 3.);
        let e = f64x2::new(-2., 0.);
        let r: f64x2 = transmute(vpminq_f64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_s8() {
        #[cfg_attr(rustfmt, skip)]
        let a = i8x16::new(1, -2, 3, -4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8);
        #[cfg_attr(rustfmt, skip)]
        let b = i8x16::new(0, 3, 2, 5, 4, 7, 6, 9, 0, 3, 2, 5, 4, 7, 6, 9);
        #[cfg_attr(rustfmt, skip)]
        let e = i8x16::new(1, 3, 6, 8, 2, 4, 6, 8, 3, 5, 7, 9, 3, 5, 7, 9);
        let r: i8x16 = transmute(vpmaxq_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_s16() {
        let a = i16x8::new(1, -2, 3, 4, 5, 6, 7, 8);
        let b = i16x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = i16x8::new(1, 4, 6, 8, 3, 5, 7, 9);
        let r: i16x8 = transmute(vpmaxq_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_s32() {
        let a = i32x4::new(1, -2, 3, 4);
        let b = i32x4::new(0, 3, 2, 5);
        let e = i32x4::new(1, 4, 3, 5);
        let r: i32x4 = transmute(vpmaxq_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_u8() {
        #[cfg_attr(rustfmt, skip)]
        let a = u8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8);
        #[cfg_attr(rustfmt, skip)]
        let b = u8x16::new(0, 3, 2, 5, 4, 7, 6, 9, 0, 3, 2, 5, 4, 7, 6, 9);
        #[cfg_attr(rustfmt, skip)]
        let e = u8x16::new(2, 4, 6, 8, 2, 4, 6, 8, 3, 5, 7, 9, 3, 5, 7, 9);
        let r: u8x16 = transmute(vpmaxq_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_u16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = u16x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = u16x8::new(2, 4, 6, 8, 3, 5, 7, 9);
        let r: u16x8 = transmute(vpmaxq_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmaxq_u32() {
        let a = u32x4::new(1, 2, 3, 4);
        let b = u32x4::new(0, 3, 2, 5);
        let e = u32x4::new(2, 4, 3, 5);
        let r: u32x4 = transmute(vpmaxq_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_f32() {
        let a = f32x4::new(1., -2., 3., 4.);
        let b = f32x4::new(0., 3., 2., 5.);
        let e = f32x4::new(1., 4., 3., 5.);
        let r: f32x4 = transmute(vpmaxq_f32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_f64() {
        let a = f64x2::new(1., -2.);
        let b = f64x2::new(0., 3.);
        let e = f64x2::new(1., 3.);
        let r: f64x2 = transmute(vpmaxq_f64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vext_p64() {
        let a: i64x1 = i64x1::new(0);
        let b: i64x1 = i64x1::new(1);
        let e: i64x1 = i64x1::new(0);
        let r: i64x1 = transmute(vext_p64::<0>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vext_f64() {
        let a: f64x1 = f64x1::new(0.);
        let b: f64x1 = f64x1::new(1.);
        let e: f64x1 = f64x1::new(0.);
        let r: f64x1 = transmute(vext_f64::<0>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vshld_n_s64() {
        let a: i64 = 1;
        let e: i64 = 4;
        let r: i64 = vshld_n_s64::<2>(a);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vshld_n_u64() {
        let a: u64 = 1;
        let e: u64 = 4;
        let r: u64 = vshld_n_u64::<2>(a);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vshrd_n_s64() {
        let a: i64 = 4;
        let e: i64 = 1;
        let r: i64 = vshrd_n_s64::<2>(a);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vshrd_n_u64() {
        let a: u64 = 4;
        let e: u64 = 1;
        let r: u64 = vshrd_n_u64::<2>(a);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vsrad_n_s64() {
        let a: i64 = 1;
        let b: i64 = 4;
        let e: i64 = 2;
        let r: i64 = vsrad_n_s64::<2>(a, b);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vsrad_n_u64() {
        let a: u64 = 1;
        let b: u64 = 4;
        let e: u64 = 2;
        let r: u64 = vsrad_n_u64::<2>(a, b);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_f64() {
        let a: f64 = 3.3;
        let e = f64x1::new(3.3);
        let r: f64x1 = transmute(vdup_n_f64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_p64() {
        let a: u64 = 3;
        let e = u64x1::new(3);
        let r: u64x1 = transmute(vdup_n_p64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_f64() {
        let a: f64 = 3.3;
        let e = f64x2::new(3.3, 3.3);
        let r: f64x2 = transmute(vdupq_n_f64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_p64() {
        let a: u64 = 3;
        let e = u64x2::new(3, 3);
        let r: u64x2 = transmute(vdupq_n_p64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_p64() {
        let a: u64 = 3;
        let e = u64x1::new(3);
        let r: u64x1 = transmute(vmov_n_p64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_f64() {
        let a: f64 = 3.3;
        let e = f64x1::new(3.3);
        let r: f64x1 = transmute(vmov_n_f64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_p64() {
        let a: u64 = 3;
        let e = u64x2::new(3, 3);
        let r: u64x2 = transmute(vmovq_n_p64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_f64() {
        let a: f64 = 3.3;
        let e = f64x2::new(3.3, 3.3);
        let r: f64x2 = transmute(vmovq_n_f64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_f64() {
        let a = f64x2::new(1.0, 2.0);
        let e = f64x1::new(2.0);
        let r: f64x1 = transmute(vget_high_f64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_p64() {
        let a = u64x2::new(1, 2);
        let e = u64x1::new(2);
        let r: u64x1 = transmute(vget_high_p64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_f64() {
        let a = f64x2::new(1.0, 2.0);
        let e = f64x1::new(1.0);
        let r: f64x1 = transmute(vget_low_f64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_p64() {
        let a = u64x2::new(1, 2);
        let e = u64x1::new(1);
        let r: u64x1 = transmute(vget_low_p64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_f64() {
        let v = f64x1::new(1.0);
        let r = vget_lane_f64::<0>(transmute(v));
        assert_eq!(r, 1.0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_f64() {
        let v = f64x2::new(0.0, 1.0);
        let r = vgetq_lane_f64::<1>(transmute(v));
        assert_eq!(r, 1.0);
        let r = vgetq_lane_f64::<0>(transmute(v));
        assert_eq!(r, 0.0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcopy_lane_s64() {
        let a: i64x1 = i64x1::new(1);
        let b: i64x1 = i64x1::new(0x7F_FF_FF_FF_FF_FF_FF_FF);
        let e: i64x1 = i64x1::new(0x7F_FF_FF_FF_FF_FF_FF_FF);
        let r: i64x1 = transmute(vcopy_lane_s64::<0, 0>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcopy_lane_u64() {
        let a: u64x1 = u64x1::new(1);
        let b: u64x1 = u64x1::new(0xFF_FF_FF_FF_FF_FF_FF_FF);
        let e: u64x1 = u64x1::new(0xFF_FF_FF_FF_FF_FF_FF_FF);
        let r: u64x1 = transmute(vcopy_lane_u64::<0, 0>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcopy_lane_p64() {
        let a: i64x1 = i64x1::new(1);
        let b: i64x1 = i64x1::new(0x7F_FF_FF_FF_FF_FF_FF_FF);
        let e: i64x1 = i64x1::new(0x7F_FF_FF_FF_FF_FF_FF_FF);
        let r: i64x1 = transmute(vcopy_lane_p64::<0, 0>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcopy_lane_f64() {
        let a: f64 = 1.;
        let b: f64 = 0.;
        let e: f64 = 0.;
        let r: f64 = transmute(vcopy_lane_f64::<0, 0>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcopy_laneq_s64() {
        let a: i64x1 = i64x1::new(1);
        let b: i64x2 = i64x2::new(0, 0x7F_FF_FF_FF_FF_FF_FF_FF);
        let e: i64x1 = i64x1::new(0x7F_FF_FF_FF_FF_FF_FF_FF);
        let r: i64x1 = transmute(vcopy_laneq_s64::<0, 1>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcopy_laneq_u64() {
        let a: u64x1 = u64x1::new(1);
        let b: u64x2 = u64x2::new(0, 0xFF_FF_FF_FF_FF_FF_FF_FF);
        let e: u64x1 = u64x1::new(0xFF_FF_FF_FF_FF_FF_FF_FF);
        let r: u64x1 = transmute(vcopy_laneq_u64::<0, 1>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcopy_laneq_p64() {
        let a: i64x1 = i64x1::new(1);
        let b: i64x2 = i64x2::new(0, 0x7F_FF_FF_FF_FF_FF_FF_FF);
        let e: i64x1 = i64x1::new(0x7F_FF_FF_FF_FF_FF_FF_FF);
        let r: i64x1 = transmute(vcopy_laneq_p64::<0, 1>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcopy_laneq_f64() {
        let a: f64 = 1.;
        let b: f64x2 = f64x2::new(0., 0.5);
        let e: f64 = 0.5;
        let r: f64 = transmute(vcopy_laneq_f64::<0, 1>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_u64() {
        test_cmp_u64(
            |i, j| vceq_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a == b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_u64() {
        testq_cmp_u64(
            |i, j| vceqq_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a == b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_s64() {
        test_cmp_s64(
            |i, j| vceq_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a == b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_s64() {
        testq_cmp_s64(
            |i, j| vceqq_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a == b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_p64() {
        test_cmp_p64(
            |i, j| vceq_p64(i, j),
            |a: u64, b: u64| -> u64 {
                if a == b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_p64() {
        testq_cmp_p64(
            |i, j| vceqq_p64(i, j),
            |a: u64, b: u64| -> u64 {
                if a == b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_f64() {
        test_cmp_f64(
            |i, j| vceq_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a == b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_f64() {
        testq_cmp_f64(
            |i, j| vceqq_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a == b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_s64() {
        test_cmp_s64(
            |i, j| vcgt_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a > b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_s64() {
        testq_cmp_s64(
            |i, j| vcgtq_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a > b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_u64() {
        test_cmp_u64(
            |i, j| vcgt_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a > b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_u64() {
        testq_cmp_u64(
            |i, j| vcgtq_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a > b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_f64() {
        test_cmp_f64(
            |i, j| vcgt_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a > b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_f64() {
        testq_cmp_f64(
            |i, j| vcgtq_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a > b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vclt_s64() {
        test_cmp_s64(
            |i, j| vclt_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a < b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_s64() {
        testq_cmp_s64(
            |i, j| vcltq_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a < b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vclt_u64() {
        test_cmp_u64(
            |i, j| vclt_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a < b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_u64() {
        testq_cmp_u64(
            |i, j| vcltq_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a < b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vltq_f64() {
        test_cmp_f64(
            |i, j| vclt_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a < b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_f64() {
        testq_cmp_f64(
            |i, j| vcltq_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a < b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcle_s64() {
        test_cmp_s64(
            |i, j| vcle_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a <= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_s64() {
        testq_cmp_s64(
            |i, j| vcleq_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a <= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcle_u64() {
        test_cmp_u64(
            |i, j| vcle_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a <= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_u64() {
        testq_cmp_u64(
            |i, j| vcleq_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a <= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vleq_f64() {
        test_cmp_f64(
            |i, j| vcle_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a <= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_f64() {
        testq_cmp_f64(
            |i, j| vcleq_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a <= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcge_s64() {
        test_cmp_s64(
            |i, j| vcge_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a >= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_s64() {
        testq_cmp_s64(
            |i, j| vcgeq_s64(i, j),
            |a: i64, b: i64| -> u64 {
                if a >= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcge_u64() {
        test_cmp_u64(
            |i, j| vcge_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a >= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_u64() {
        testq_cmp_u64(
            |i, j| vcgeq_u64(i, j),
            |a: u64, b: u64| -> u64 {
                if a >= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgeq_f64() {
        test_cmp_f64(
            |i, j| vcge_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a >= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_f64() {
        testq_cmp_f64(
            |i, j| vcgeq_f64(i, j),
            |a: f64, b: f64| -> u64 {
                if a >= b {
                    0xFFFFFFFFFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmul_f64() {
        test_ari_f64(|i, j| vmul_f64(i, j), |a: f64, b: f64| -> f64 { a * b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmulq_f64() {
        testq_ari_f64(|i, j| vmulq_f64(i, j), |a: f64, b: f64| -> f64 { a * b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vsub_f64() {
        test_ari_f64(|i, j| vsub_f64(i, j), |a: f64, b: f64| -> f64 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsubq_f64() {
        testq_ari_f64(|i, j| vsubq_f64(i, j), |a: f64, b: f64| -> f64 { a - b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vabsd_s64() {
        assert_eq!(vabsd_s64(-1), 1);
        assert_eq!(vabsd_s64(0), 0);
        assert_eq!(vabsd_s64(1), 1);
        assert_eq!(vabsd_s64(i64::MIN), i64::MIN);
        assert_eq!(vabsd_s64(i64::MIN + 1), i64::MAX);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabs_s64() {
        let a = i64x1::new(i64::MIN);
        let r: i64x1 = transmute(vabs_s64(transmute(a)));
        let e = i64x1::new(i64::MIN);
        assert_eq!(r, e);
        let a = i64x1::new(i64::MIN + 1);
        let r: i64x1 = transmute(vabs_s64(transmute(a)));
        let e = i64x1::new(i64::MAX);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabsq_s64() {
        let a = i64x2::new(i64::MIN, i64::MIN + 1);
        let r: i64x2 = transmute(vabsq_s64(transmute(a)));
        let e = i64x2::new(i64::MIN, i64::MAX);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_f64() {
        let a = u64x1::new(0x8000000000000000);
        let b = f64x1::new(-1.23f64);
        let c = f64x1::new(2.34f64);
        let e = f64x1::new(-2.34f64);
        let r: f64x1 = transmute(vbsl_f64(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_p64() {
        let a = u64x1::new(1);
        let b = u64x1::new(u64::MAX);
        let c = u64x1::new(u64::MIN);
        let e = u64x1::new(1);
        let r: u64x1 = transmute(vbsl_p64(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_f64() {
        let a = u64x2::new(1, 0x8000000000000000);
        let b = f64x2::new(f64::MAX, -1.23f64);
        let c = f64x2::new(f64::MIN, 2.34f64);
        let e = f64x2::new(f64::MIN, -2.34f64);
        let r: f64x2 = transmute(vbslq_f64(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_p64() {
        let a = u64x2::new(u64::MAX, 1);
        let b = u64x2::new(u64::MAX, u64::MAX);
        let c = u64x2::new(u64::MIN, u64::MIN);
        let e = u64x2::new(u64::MAX, 1);
        let r: u64x2 = transmute(vbslq_p64(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddv_s16() {
        let a = i16x4::new(1, 2, 3, -4);
        let r: i16 = transmute(vaddv_s16(transmute(a)));
        let e = 2_i16;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddv_u16() {
        let a = u16x4::new(1, 2, 3, 4);
        let r: u16 = transmute(vaddv_u16(transmute(a)));
        let e = 10_u16;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddv_s32() {
        let a = i32x2::new(1, -2);
        let r: i32 = transmute(vaddv_s32(transmute(a)));
        let e = -1_i32;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddv_u32() {
        let a = u32x2::new(1, 2);
        let r: u32 = transmute(vaddv_u32(transmute(a)));
        let e = 3_u32;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddv_s8() {
        let a = i8x8::new(1, 2, 3, 4, 5, 6, 7, -8);
        let r: i8 = transmute(vaddv_s8(transmute(a)));
        let e = 20_i8;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddv_u8() {
        let a = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: u8 = transmute(vaddv_u8(transmute(a)));
        let e = 36_u8;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddvq_s16() {
        let a = i16x8::new(1, 2, 3, 4, 5, 6, 7, -8);
        let r: i16 = transmute(vaddvq_s16(transmute(a)));
        let e = 20_i16;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddvq_u16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: u16 = transmute(vaddvq_u16(transmute(a)));
        let e = 36_u16;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddvq_s32() {
        let a = i32x4::new(1, 2, 3, -4);
        let r: i32 = transmute(vaddvq_s32(transmute(a)));
        let e = 2_i32;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddvq_u32() {
        let a = u32x4::new(1, 2, 3, 4);
        let r: u32 = transmute(vaddvq_u32(transmute(a)));
        let e = 10_u32;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddvq_s8() {
        let a = i8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, -16);
        let r: i8 = transmute(vaddvq_s8(transmute(a)));
        let e = 104_i8;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddvq_u8() {
        let a = u8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let r: u8 = transmute(vaddvq_u8(transmute(a)));
        let e = 136_u8;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddvq_s64() {
        let a = i64x2::new(1, -2);
        let r: i64 = transmute(vaddvq_s64(transmute(a)));
        let e = -1_i64;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddvq_u64() {
        let a = u64x2::new(1, 2);
        let r: u64 = transmute(vaddvq_u64(transmute(a)));
        let e = 3_u64;
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddlv_s8() {
        let a = i8x8::new(1, 2, 3, 4, 5, 6, 7, -8);
        let r: i16 = vaddlv_s8(transmute(a));
        let e = 20_i16;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddlv_u8() {
        let a = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: u16 = vaddlv_u8(transmute(a));
        let e = 36_u16;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddlvq_s8() {
        let a = i8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, -16);
        let r: i16 = vaddlvq_s8(transmute(a));
        let e = 104_i16;
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddlvq_u8() {
        let a = u8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let r: u16 = vaddlvq_u8(transmute(a));
        let e = 136_u16;
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_f64() {
        let a: [f64; 2] = [0., 1.];
        let e = f64x1::new(1.);
        let r: f64x1 = transmute(vld1_f64(a[1..].as_ptr()));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_f64() {
        let a: [f64; 3] = [0., 1., 2.];
        let e = f64x2::new(1., 2.);
        let r: f64x2 = transmute(vld1q_f64(a[1..].as_ptr()));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_f64() {
        let a: [f64; 2] = [1., 42.];
        let e = f64x1::new(42.);
        let r: f64x1 = transmute(vld1_dup_f64(a[1..].as_ptr()));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_f64() {
        let elem: f64 = 42.;
        let e = f64x2::new(42., 42.);
        let r: f64x2 = transmute(vld1q_dup_f64(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_f64() {
        let a = f64x1::new(0.);
        let elem: f64 = 42.;
        let e = f64x1::new(42.);
        let r: f64x1 = transmute(vld1_lane_f64::<0>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_f64() {
        let a = f64x2::new(0., 1.);
        let elem: f64 = 42.;
        let e = f64x2::new(0., 42.);
        let r: f64x2 = transmute(vld1q_lane_f64::<1>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vst1_f64() {
        let mut vals = [0_f64; 2];
        let a = f64x1::new(1.);

        vst1_f64(vals[1..].as_mut_ptr(), transmute(a));

        assert_eq!(vals[0], 0.);
        assert_eq!(vals[1], 1.);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vst1q_f64() {
        let mut vals = [0_f64; 3];
        let a = f64x2::new(1., 2.);

        vst1q_f64(vals[1..].as_mut_ptr(), transmute(a));

        assert_eq!(vals[0], 0.);
        assert_eq!(vals[1], 1.);
        assert_eq!(vals[2], 2.);
    }

    #[simd_test(enable = "neon,sm4")]
    unsafe fn test_vsm3tt1aq_u32() {
        let a: u32x4 = u32x4::new(1, 2, 3, 4);
        let b: u32x4 = u32x4::new(1, 2, 3, 4);
        let c: u32x4 = u32x4::new(1, 2, 3, 4);
        let e: u32x4 = u32x4::new(2, 1536, 4, 16395);
        let r: u32x4 = transmute(vsm3tt1aq_u32::<0>(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon,sm4")]
    unsafe fn test_vsm3tt1bq_u32() {
        let a: u32x4 = u32x4::new(1, 2, 3, 4);
        let b: u32x4 = u32x4::new(1, 2, 3, 4);
        let c: u32x4 = u32x4::new(1, 2, 3, 4);
        let e: u32x4 = u32x4::new(2, 1536, 4, 16392);
        let r: u32x4 = transmute(vsm3tt1bq_u32::<0>(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon,sm4")]
    unsafe fn test_vsm3tt2aq_u32() {
        let a: u32x4 = u32x4::new(1, 2, 3, 4);
        let b: u32x4 = u32x4::new(1, 2, 3, 4);
        let c: u32x4 = u32x4::new(1, 2, 3, 4);
        let e: u32x4 = u32x4::new(2, 1572864, 4, 1447435);
        let r: u32x4 = transmute(vsm3tt2aq_u32::<0>(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon,sm4")]
    unsafe fn test_vsm3tt2bq_u32() {
        let a: u32x4 = u32x4::new(1, 2, 3, 4);
        let b: u32x4 = u32x4::new(1, 2, 3, 4);
        let c: u32x4 = u32x4::new(1, 2, 3, 4);
        let e: u32x4 = u32x4::new(2, 1572864, 4, 1052680);
        let r: u32x4 = transmute(vsm3tt2bq_u32::<0>(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon,sha3")]
    unsafe fn test_vxarq_u64() {
        let a: u64x2 = u64x2::new(1, 2);
        let b: u64x2 = u64x2::new(3, 4);
        let e: u64x2 = u64x2::new(2, 6);
        let r: u64x2 = transmute(vxarq_u64::<0>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }
}

#[cfg(test)]
#[cfg(target_endian = "little")]
#[path = "../../arm_shared/neon/table_lookup_tests.rs"]
mod table_lookup_tests;

#[cfg(test)]
#[path = "../../arm_shared/neon/shift_and_insert_tests.rs"]
mod shift_and_insert_tests;

#[cfg(test)]
#[path = "../../arm_shared/neon/load_tests.rs"]
mod load_tests;

#[cfg(test)]
#[path = "../../arm_shared/neon/store_tests.rs"]
mod store_tests;
