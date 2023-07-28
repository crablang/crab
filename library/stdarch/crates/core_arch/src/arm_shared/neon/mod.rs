//! ARMv7 NEON intrinsics

#[rustfmt::skip]
mod generated;
#[rustfmt::skip]
pub use self::generated::*;

use crate::{
    core_arch::simd::*, core_arch::simd_llvm::*, hint::unreachable_unchecked, mem::transmute,
};
#[cfg(test)]
use stdarch_test::assert_instr;

pub(crate) type p8 = u8;
pub(crate) type p16 = u16;
pub(crate) type p64 = u64;
pub(crate) type p128 = u128;

types! {
    /// ARM-specific 64-bit wide vector of eight packed `i8`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct int8x8_t(pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8);
    /// ARM-specific 64-bit wide vector of eight packed `u8`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct uint8x8_t(pub(crate) u8, pub(crate) u8, pub(crate) u8, pub(crate) u8, pub(crate) u8, pub(crate) u8, pub(crate) u8, pub(crate) u8);
    /// ARM-specific 64-bit wide polynomial vector of eight packed `p8`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct poly8x8_t(pub(crate) p8, pub(crate) p8, pub(crate) p8, pub(crate) p8, pub(crate) p8, pub(crate) p8, pub(crate) p8, pub(crate) p8);
    /// ARM-specific 64-bit wide vector of four packed `i16`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct int16x4_t(pub(crate) i16, pub(crate) i16, pub(crate) i16, pub(crate) i16);
    /// ARM-specific 64-bit wide vector of four packed `u16`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct uint16x4_t(pub(crate) u16, pub(crate) u16, pub(crate) u16, pub(crate) u16);
    // FIXME: ARM-specific 64-bit wide vector of four packed `f16`.
    // pub struct float16x4_t(f16, f16, f16, f16);
    /// ARM-specific 64-bit wide vector of four packed `p16`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct poly16x4_t(pub(crate) p16, pub(crate) p16, pub(crate) p16, pub(crate) p16);
    /// ARM-specific 64-bit wide vector of two packed `i32`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct int32x2_t(pub(crate) i32, pub(crate) i32);
    /// ARM-specific 64-bit wide vector of two packed `u32`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct uint32x2_t(pub(crate) u32, pub(crate) u32);
    /// ARM-specific 64-bit wide vector of two packed `f32`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct float32x2_t(pub(crate) f32, pub(crate) f32);
    /// ARM-specific 64-bit wide vector of one packed `i64`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct int64x1_t(pub(crate) i64);
    /// ARM-specific 64-bit wide vector of one packed `u64`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct uint64x1_t(pub(crate) u64);
    /// ARM-specific 64-bit wide vector of one packed `p64`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct poly64x1_t(pub(crate) p64);

    /// ARM-specific 128-bit wide vector of sixteen packed `i8`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct int8x16_t(
        pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8 , pub(crate) i8, pub(crate) i8,
        pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8, pub(crate) i8 , pub(crate) i8, pub(crate) i8,
    );
    /// ARM-specific 128-bit wide vector of sixteen packed `u8`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct uint8x16_t(
        pub(crate) u8, pub(crate) u8 , pub(crate) u8, pub(crate) u8, pub(crate) u8, pub(crate) u8 , pub(crate) u8, pub(crate) u8,
        pub(crate) u8, pub(crate) u8 , pub(crate) u8,  pub(crate) u8,  pub(crate) u8,  pub(crate) u8 , pub(crate) u8,  pub(crate) u8,
    );
    /// ARM-specific 128-bit wide vector of sixteen packed `p8`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct poly8x16_t(
         pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,
         pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,  pub(crate) p8,
    );
    /// ARM-specific 128-bit wide vector of eight packed `i16`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct int16x8_t(pub(crate) i16, pub(crate) i16, pub(crate) i16, pub(crate) i16, pub(crate) i16, pub(crate) i16, pub(crate) i16, pub(crate) i16);
    /// ARM-specific 128-bit wide vector of eight packed `u16`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct uint16x8_t(pub(crate) u16, pub(crate) u16, pub(crate) u16, pub(crate) u16, pub(crate) u16, pub(crate) u16, pub(crate) u16, pub(crate) u16);
    // FIXME: ARM-specific 128-bit wide vector of eight packed `f16`.
    // pub struct float16x8_t(f16, f16, f16, f16, f16, f16, f16);
    /// ARM-specific 128-bit wide vector of eight packed `p16`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct poly16x8_t(pub(crate) p16, pub(crate) p16, pub(crate) p16, pub(crate) p16, pub(crate) p16, pub(crate) p16, pub(crate) p16, pub(crate) p16);
    /// ARM-specific 128-bit wide vector of four packed `i32`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct int32x4_t(pub(crate) i32, pub(crate) i32, pub(crate) i32, pub(crate) i32);
    /// ARM-specific 128-bit wide vector of four packed `u32`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct uint32x4_t(pub(crate) u32, pub(crate) u32, pub(crate) u32, pub(crate) u32);
    /// ARM-specific 128-bit wide vector of four packed `f32`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct float32x4_t(pub(crate) f32, pub(crate) f32, pub(crate) f32, pub(crate) f32);
    /// ARM-specific 128-bit wide vector of two packed `i64`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct int64x2_t(pub(crate) i64, pub(crate) i64);
    /// ARM-specific 128-bit wide vector of two packed `u64`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct uint64x2_t(pub(crate) u64, pub(crate) u64);
    /// ARM-specific 128-bit wide vector of two packed `p64`.
    #[cfg_attr(not(target_arch = "arm"), stable(feature = "neon_intrinsics", since = "1.59.0"))]
    pub struct poly64x2_t(pub(crate) p64, pub(crate) p64);
}

/// ARM-specific type containing two `int8x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int8x8x2_t(pub int8x8_t, pub int8x8_t);
/// ARM-specific type containing three `int8x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int8x8x3_t(pub int8x8_t, pub int8x8_t, pub int8x8_t);
/// ARM-specific type containing four `int8x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int8x8x4_t(pub int8x8_t, pub int8x8_t, pub int8x8_t, pub int8x8_t);

/// ARM-specific type containing two `int8x16_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int8x16x2_t(pub int8x16_t, pub int8x16_t);
/// ARM-specific type containing three `int8x16_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int8x16x3_t(pub int8x16_t, pub int8x16_t, pub int8x16_t);
/// ARM-specific type containing four `int8x16_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int8x16x4_t(pub int8x16_t, pub int8x16_t, pub int8x16_t, pub int8x16_t);

/// ARM-specific type containing two `uint8x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint8x8x2_t(pub uint8x8_t, pub uint8x8_t);
/// ARM-specific type containing three `uint8x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint8x8x3_t(pub uint8x8_t, pub uint8x8_t, pub uint8x8_t);
/// ARM-specific type containing four `uint8x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint8x8x4_t(pub uint8x8_t, pub uint8x8_t, pub uint8x8_t, pub uint8x8_t);

/// ARM-specific type containing two `uint8x16_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint8x16x2_t(pub uint8x16_t, pub uint8x16_t);
/// ARM-specific type containing three `uint8x16_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint8x16x3_t(pub uint8x16_t, pub uint8x16_t, pub uint8x16_t);
/// ARM-specific type containing four `uint8x16_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint8x16x4_t(
    pub uint8x16_t,
    pub uint8x16_t,
    pub uint8x16_t,
    pub uint8x16_t,
);

/// ARM-specific type containing two `poly8x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly8x8x2_t(pub poly8x8_t, pub poly8x8_t);
/// ARM-specific type containing three `poly8x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly8x8x3_t(pub poly8x8_t, pub poly8x8_t, pub poly8x8_t);
/// ARM-specific type containing four `poly8x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly8x8x4_t(pub poly8x8_t, pub poly8x8_t, pub poly8x8_t, pub poly8x8_t);

/// ARM-specific type containing two `poly8x16_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly8x16x2_t(pub poly8x16_t, pub poly8x16_t);
/// ARM-specific type containing three `poly8x16_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly8x16x3_t(pub poly8x16_t, pub poly8x16_t, pub poly8x16_t);
/// ARM-specific type containing four `poly8x16_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly8x16x4_t(
    pub poly8x16_t,
    pub poly8x16_t,
    pub poly8x16_t,
    pub poly8x16_t,
);

/// ARM-specific type containing two `int16x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int16x4x2_t(pub int16x4_t, pub int16x4_t);
/// ARM-specific type containing three `int16x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int16x4x3_t(pub int16x4_t, pub int16x4_t, pub int16x4_t);
/// ARM-specific type containing four `int16x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int16x4x4_t(pub int16x4_t, pub int16x4_t, pub int16x4_t, pub int16x4_t);

/// ARM-specific type containing two `int16x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int16x8x2_t(pub int16x8_t, pub int16x8_t);
/// ARM-specific type containing three `int16x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int16x8x3_t(pub int16x8_t, pub int16x8_t, pub int16x8_t);
/// ARM-specific type containing four `int16x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int16x8x4_t(pub int16x8_t, pub int16x8_t, pub int16x8_t, pub int16x8_t);

/// ARM-specific type containing two `uint16x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint16x4x2_t(pub uint16x4_t, pub uint16x4_t);
/// ARM-specific type containing three `uint16x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint16x4x3_t(pub uint16x4_t, pub uint16x4_t, pub uint16x4_t);
/// ARM-specific type containing four `uint16x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint16x4x4_t(
    pub uint16x4_t,
    pub uint16x4_t,
    pub uint16x4_t,
    pub uint16x4_t,
);

/// ARM-specific type containing two `uint16x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint16x8x2_t(pub uint16x8_t, pub uint16x8_t);
/// ARM-specific type containing three `uint16x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint16x8x3_t(pub uint16x8_t, pub uint16x8_t, pub uint16x8_t);
/// ARM-specific type containing four `uint16x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint16x8x4_t(
    pub uint16x8_t,
    pub uint16x8_t,
    pub uint16x8_t,
    pub uint16x8_t,
);

/// ARM-specific type containing two `poly16x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly16x4x2_t(pub poly16x4_t, pub poly16x4_t);
/// ARM-specific type containing three `poly16x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly16x4x3_t(pub poly16x4_t, pub poly16x4_t, pub poly16x4_t);
/// ARM-specific type containing four `poly16x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly16x4x4_t(
    pub poly16x4_t,
    pub poly16x4_t,
    pub poly16x4_t,
    pub poly16x4_t,
);

/// ARM-specific type containing two `poly16x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly16x8x2_t(pub poly16x8_t, pub poly16x8_t);
/// ARM-specific type containing three `poly16x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly16x8x3_t(pub poly16x8_t, pub poly16x8_t, pub poly16x8_t);
/// ARM-specific type containing four `poly16x8_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly16x8x4_t(
    pub poly16x8_t,
    pub poly16x8_t,
    pub poly16x8_t,
    pub poly16x8_t,
);

/// ARM-specific type containing two `int32x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int32x2x2_t(pub int32x2_t, pub int32x2_t);
/// ARM-specific type containing three `int32x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int32x2x3_t(pub int32x2_t, pub int32x2_t, pub int32x2_t);
/// ARM-specific type containing four `int32x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int32x2x4_t(pub int32x2_t, pub int32x2_t, pub int32x2_t, pub int32x2_t);

/// ARM-specific type containing two `int32x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int32x4x2_t(pub int32x4_t, pub int32x4_t);
/// ARM-specific type containing three `int32x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int32x4x3_t(pub int32x4_t, pub int32x4_t, pub int32x4_t);
/// ARM-specific type containing four `int32x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int32x4x4_t(pub int32x4_t, pub int32x4_t, pub int32x4_t, pub int32x4_t);

/// ARM-specific type containing two `uint32x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint32x2x2_t(pub uint32x2_t, pub uint32x2_t);
/// ARM-specific type containing three `uint32x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint32x2x3_t(pub uint32x2_t, pub uint32x2_t, pub uint32x2_t);
/// ARM-specific type containing four `uint32x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint32x2x4_t(
    pub uint32x2_t,
    pub uint32x2_t,
    pub uint32x2_t,
    pub uint32x2_t,
);

/// ARM-specific type containing two `uint32x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint32x4x2_t(pub uint32x4_t, pub uint32x4_t);
/// ARM-specific type containing three `uint32x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint32x4x3_t(pub uint32x4_t, pub uint32x4_t, pub uint32x4_t);
/// ARM-specific type containing four `uint32x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint32x4x4_t(
    pub uint32x4_t,
    pub uint32x4_t,
    pub uint32x4_t,
    pub uint32x4_t,
);

/// ARM-specific type containing two `float32x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct float32x2x2_t(pub float32x2_t, pub float32x2_t);
/// ARM-specific type containing three `float32x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct float32x2x3_t(pub float32x2_t, pub float32x2_t, pub float32x2_t);
/// ARM-specific type containing four `float32x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct float32x2x4_t(
    pub float32x2_t,
    pub float32x2_t,
    pub float32x2_t,
    pub float32x2_t,
);

/// ARM-specific type containing two `float32x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct float32x4x2_t(pub float32x4_t, pub float32x4_t);
/// ARM-specific type containing three `float32x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct float32x4x3_t(pub float32x4_t, pub float32x4_t, pub float32x4_t);
/// ARM-specific type containing four `float32x4_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct float32x4x4_t(
    pub float32x4_t,
    pub float32x4_t,
    pub float32x4_t,
    pub float32x4_t,
);

/// ARM-specific type containing four `int64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int64x1x2_t(pub int64x1_t, pub int64x1_t);
/// ARM-specific type containing four `int64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int64x1x3_t(pub int64x1_t, pub int64x1_t, pub int64x1_t);
/// ARM-specific type containing four `int64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int64x1x4_t(pub int64x1_t, pub int64x1_t, pub int64x1_t, pub int64x1_t);

/// ARM-specific type containing four `int64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int64x2x2_t(pub int64x2_t, pub int64x2_t);
/// ARM-specific type containing four `int64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int64x2x3_t(pub int64x2_t, pub int64x2_t, pub int64x2_t);
/// ARM-specific type containing four `int64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct int64x2x4_t(pub int64x2_t, pub int64x2_t, pub int64x2_t, pub int64x2_t);

/// ARM-specific type containing four `uint64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint64x1x2_t(pub uint64x1_t, pub uint64x1_t);
/// ARM-specific type containing four `uint64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint64x1x3_t(pub uint64x1_t, pub uint64x1_t, pub uint64x1_t);
/// ARM-specific type containing four `uint64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint64x1x4_t(
    pub uint64x1_t,
    pub uint64x1_t,
    pub uint64x1_t,
    pub uint64x1_t,
);

/// ARM-specific type containing four `uint64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint64x2x2_t(pub uint64x2_t, pub uint64x2_t);
/// ARM-specific type containing four `uint64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint64x2x3_t(pub uint64x2_t, pub uint64x2_t, pub uint64x2_t);
/// ARM-specific type containing four `uint64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct uint64x2x4_t(
    pub uint64x2_t,
    pub uint64x2_t,
    pub uint64x2_t,
    pub uint64x2_t,
);

/// ARM-specific type containing four `poly64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly64x1x2_t(pub poly64x1_t, pub poly64x1_t);
/// ARM-specific type containing four `poly64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly64x1x3_t(pub poly64x1_t, pub poly64x1_t, pub poly64x1_t);
/// ARM-specific type containing four `poly64x1_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly64x1x4_t(
    pub poly64x1_t,
    pub poly64x1_t,
    pub poly64x1_t,
    pub poly64x1_t,
);

/// ARM-specific type containing four `poly64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly64x2x2_t(pub poly64x2_t, pub poly64x2_t);
/// ARM-specific type containing four `poly64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly64x2x3_t(pub poly64x2_t, pub poly64x2_t, pub poly64x2_t);
/// ARM-specific type containing four `poly64x2_t` vectors.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub struct poly64x2x4_t(
    pub poly64x2_t,
    pub poly64x2_t,
    pub poly64x2_t,
    pub poly64x2_t,
);

#[allow(improper_ctypes)]
extern "unadjusted" {
    // absolute value (64-bit)
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vabs.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.abs.v8i8")]
    fn vabs_s8_(a: int8x8_t) -> int8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vabs.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.abs.v4i16")]
    fn vabs_s16_(a: int16x4_t) -> int16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vabs.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.abs.v2i32")]
    fn vabs_s32_(a: int32x2_t) -> int32x2_t;
    // absolute value (128-bit)
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vabs.v16i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.abs.v16i8")]
    fn vabsq_s8_(a: int8x16_t) -> int8x16_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vabs.v8i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.abs.v8i16")]
    fn vabsq_s16_(a: int16x8_t) -> int16x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vabs.v4i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.abs.v4i32")]
    fn vabsq_s32_(a: int32x4_t) -> int32x4_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmins.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sminp.v8i8")]
    fn vpmins_v8i8(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmins.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sminp.v4i16")]
    fn vpmins_v4i16(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmins.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.sminp.v2i32")]
    fn vpmins_v2i32(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpminu.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uminp.v8i8")]
    fn vpminu_v8i8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpminu.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uminp.v4i16")]
    fn vpminu_v4i16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpminu.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.uminp.v2i32")]
    fn vpminu_v2i32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmins.v2f32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.fminp.v2f32")]
    fn vpminf_v2f32(a: float32x2_t, b: float32x2_t) -> float32x2_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxs.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.smaxp.v8i8")]
    fn vpmaxs_v8i8(a: int8x8_t, b: int8x8_t) -> int8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxs.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.smaxp.v4i16")]
    fn vpmaxs_v4i16(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxs.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.smaxp.v2i32")]
    fn vpmaxs_v2i32(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxu.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.umaxp.v8i8")]
    fn vpmaxu_v8i8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxu.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.umaxp.v4i16")]
    fn vpmaxu_v4i16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxu.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.umaxp.v2i32")]
    fn vpmaxu_v2i32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpmaxs.v2f32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.fmaxp.v2f32")]
    fn vpmaxf_v2f32(a: float32x2_t, b: float32x2_t) -> float32x2_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vraddhn.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.raddhn.v8i8")]
    fn vraddhn_s16_(a: int16x8_t, b: int16x8_t) -> int8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vraddhn.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.raddhn.v4i16")]
    fn vraddhn_s32_(a: int32x4_t, b: int32x4_t) -> int16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vraddhn.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.raddhn.v2i32")]
    fn vraddhn_s64_(a: int64x2_t, b: int64x2_t) -> int32x2_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpadd.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.addp.v4i16")]
    fn vpadd_s16_(a: int16x4_t, b: int16x4_t) -> int16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpadd.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.addp.v2i32")]
    fn vpadd_s32_(a: int32x2_t, b: int32x2_t) -> int32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpadd.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.aarch64.neon.addp.v8i8")]
    fn vpadd_s8_(a: int8x8_t, b: int8x8_t) -> int8x8_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddls.v4i16.v8i8")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.saddlp.v4i16.v8i8"
    )]
    pub(crate) fn vpaddl_s8_(a: int8x8_t) -> int16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddls.v2i32.v4i16")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.saddlp.v2i32.v4i16"
    )]
    pub(crate) fn vpaddl_s16_(a: int16x4_t) -> int32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddls.v1i64.v2i32")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.saddlp.v1i64.v2i32"
    )]
    pub(crate) fn vpaddl_s32_(a: int32x2_t) -> int64x1_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddls.v8i16.v16i8")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.saddlp.v8i16.v16i8"
    )]
    pub(crate) fn vpaddlq_s8_(a: int8x16_t) -> int16x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddls.v4i32.v8i16")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.saddlp.v4i32.v8i16"
    )]
    pub(crate) fn vpaddlq_s16_(a: int16x8_t) -> int32x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddls.v2i64.v4i32")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.saddlp.v2i64.v4i32"
    )]
    pub(crate) fn vpaddlq_s32_(a: int32x4_t) -> int64x2_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddlu.v4i16.v8i8")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.uaddlp.v4i16.v8i8"
    )]
    pub(crate) fn vpaddl_u8_(a: uint8x8_t) -> uint16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddlu.v2i32.v4i16")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.uaddlp.v2i32.v4i16"
    )]
    pub(crate) fn vpaddl_u16_(a: uint16x4_t) -> uint32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddlu.v1i64.v2i32")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.uaddlp.v1i64.v2i32"
    )]
    pub(crate) fn vpaddl_u32_(a: uint32x2_t) -> uint64x1_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddlu.v8i16.v16i8")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.uaddlp.v8i16.v16i8"
    )]
    pub(crate) fn vpaddlq_u8_(a: uint8x16_t) -> uint16x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddlu.v4i32.v8i16")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.uaddlp.v4i32.v8i16"
    )]
    pub(crate) fn vpaddlq_u16_(a: uint16x8_t) -> uint32x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.vpaddlu.v2i64.v4i32")]
    #[cfg_attr(
        target_arch = "aarch64",
        link_name = "llvm.aarch64.neon.uaddlp.v2i64.v4i32"
    )]
    pub(crate) fn vpaddlq_u32_(a: uint32x4_t) -> uint64x2_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.ctpop.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.ctpop.v8i8")]
    fn vcnt_s8_(a: int8x8_t) -> int8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.ctpop.v16i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.ctpop.v16i8")]
    fn vcntq_s8_(a: int8x16_t) -> int8x16_t;

    #[cfg_attr(target_arch = "arm", link_name = "llvm.ctlz.v8i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.ctlz.v8i8")]
    fn vclz_s8_(a: int8x8_t) -> int8x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.ctlz.v16i8")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.ctlz.v16i8")]
    fn vclzq_s8_(a: int8x16_t) -> int8x16_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.ctlz.v4i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.ctlz.v4i16")]
    fn vclz_s16_(a: int16x4_t) -> int16x4_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.ctlz.v8i16")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.ctlz.v8i16")]
    fn vclzq_s16_(a: int16x8_t) -> int16x8_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.ctlz.v2i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.ctlz.v2i32")]
    fn vclz_s32_(a: int32x2_t) -> int32x2_t;
    #[cfg_attr(target_arch = "arm", link_name = "llvm.ctlz.v4i32")]
    #[cfg_attr(target_arch = "aarch64", link_name = "llvm.ctlz.v4i32")]
    fn vclzq_s32_(a: int32x4_t) -> int32x4_t;
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8", LANE = 7))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 7))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_s8<const LANE: i32>(ptr: *const i8, src: int8x8_t) -> int8x8_t {
    static_assert_uimm_bits!(LANE, 3);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8", LANE = 15))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 15))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_s8<const LANE: i32>(ptr: *const i8, src: int8x16_t) -> int8x16_t {
    static_assert_uimm_bits!(LANE, 4);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16", LANE = 3))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 3))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_s16<const LANE: i32>(ptr: *const i16, src: int16x4_t) -> int16x4_t {
    static_assert_uimm_bits!(LANE, 2);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16", LANE = 7))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 7))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_s16<const LANE: i32>(ptr: *const i16, src: int16x8_t) -> int16x8_t {
    static_assert_uimm_bits!(LANE, 3);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32", LANE = 1))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_s32<const LANE: i32>(ptr: *const i32, src: int32x2_t) -> int32x2_t {
    static_assert_uimm_bits!(LANE, 1);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32", LANE = 3))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 3))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_s32<const LANE: i32>(ptr: *const i32, src: int32x4_t) -> int32x4_t {
    static_assert_uimm_bits!(LANE, 2);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr", LANE = 0))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ldr, LANE = 0))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_s64<const LANE: i32>(ptr: *const i64, src: int64x1_t) -> int64x1_t {
    static_assert!(LANE == 0);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr", LANE = 1))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_s64<const LANE: i32>(ptr: *const i64, src: int64x2_t) -> int64x2_t {
    static_assert_uimm_bits!(LANE, 1);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8", LANE = 7))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 7))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_u8<const LANE: i32>(ptr: *const u8, src: uint8x8_t) -> uint8x8_t {
    static_assert_uimm_bits!(LANE, 3);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8", LANE = 15))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 15))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_u8<const LANE: i32>(ptr: *const u8, src: uint8x16_t) -> uint8x16_t {
    static_assert_uimm_bits!(LANE, 4);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16", LANE = 3))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 3))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_u16<const LANE: i32>(ptr: *const u16, src: uint16x4_t) -> uint16x4_t {
    static_assert_uimm_bits!(LANE, 2);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16", LANE = 7))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 7))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_u16<const LANE: i32>(ptr: *const u16, src: uint16x8_t) -> uint16x8_t {
    static_assert_uimm_bits!(LANE, 3);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32", LANE = 1))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_u32<const LANE: i32>(ptr: *const u32, src: uint32x2_t) -> uint32x2_t {
    static_assert_uimm_bits!(LANE, 1);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32", LANE = 3))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 3))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_u32<const LANE: i32>(ptr: *const u32, src: uint32x4_t) -> uint32x4_t {
    static_assert_uimm_bits!(LANE, 2);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr", LANE = 0))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ldr, LANE = 0))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_u64<const LANE: i32>(ptr: *const u64, src: uint64x1_t) -> uint64x1_t {
    static_assert!(LANE == 0);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr", LANE = 1))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_u64<const LANE: i32>(ptr: *const u64, src: uint64x2_t) -> uint64x2_t {
    static_assert_uimm_bits!(LANE, 1);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8", LANE = 7))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 7))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_p8<const LANE: i32>(ptr: *const p8, src: poly8x8_t) -> poly8x8_t {
    static_assert_uimm_bits!(LANE, 3);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8", LANE = 15))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 15))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_p8<const LANE: i32>(ptr: *const p8, src: poly8x16_t) -> poly8x16_t {
    static_assert_uimm_bits!(LANE, 4);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16", LANE = 3))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 3))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_p16<const LANE: i32>(ptr: *const p16, src: poly16x4_t) -> poly16x4_t {
    static_assert_uimm_bits!(LANE, 2);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16", LANE = 7))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 7))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_p16<const LANE: i32>(ptr: *const p16, src: poly16x8_t) -> poly16x8_t {
    static_assert_uimm_bits!(LANE, 3);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vld1_lane_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr", LANE = 0))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ldr, LANE = 0))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_p64<const LANE: i32>(ptr: *const p64, src: poly64x1_t) -> poly64x1_t {
    static_assert!(LANE == 0);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vld1q_lane_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr", LANE = 1))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_p64<const LANE: i32>(ptr: *const p64, src: poly64x2_t) -> poly64x2_t {
    static_assert_uimm_bits!(LANE, 1);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32", LANE = 1))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_lane_f32<const LANE: i32>(ptr: *const f32, src: float32x2_t) -> float32x2_t {
    static_assert_uimm_bits!(LANE, 1);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure to one lane of one register.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32", LANE = 3))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1, LANE = 3))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_lane_f32<const LANE: i32>(ptr: *const f32, src: float32x4_t) -> float32x4_t {
    static_assert_uimm_bits!(LANE, 2);
    simd_insert(src, LANE as u32, *ptr)
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_s8(ptr: *const i8) -> int8x8_t {
    let x = vld1_lane_s8::<0>(ptr, transmute(i8x8::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0, 0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_s8(ptr: *const i8) -> int8x16_t {
    let x = vld1q_lane_s8::<0>(ptr, transmute(i8x16::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_s16(ptr: *const i16) -> int16x4_t {
    let x = vld1_lane_s16::<0>(ptr, transmute(i16x4::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_s16(ptr: *const i16) -> int16x8_t {
    let x = vld1q_lane_s16::<0>(ptr, transmute(i16x8::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0, 0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_s32(ptr: *const i32) -> int32x2_t {
    let x = vld1_lane_s32::<0>(ptr, transmute(i32x2::splat(0)));
    simd_shuffle!(x, x, [0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_s32(ptr: *const i32) -> int32x4_t {
    let x = vld1q_lane_s32::<0>(ptr, transmute(i32x4::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ldr))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_s64(ptr: *const i64) -> int64x1_t {
    #[cfg(target_arch = "aarch64")]
    {
        crate::core_arch::aarch64::vld1_s64(ptr)
    }
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::vld1_s64(ptr)
    }
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_s64(ptr: *const i64) -> int64x2_t {
    let x = vld1q_lane_s64::<0>(ptr, transmute(i64x2::splat(0)));
    simd_shuffle!(x, x, [0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_u8(ptr: *const u8) -> uint8x8_t {
    let x = vld1_lane_u8::<0>(ptr, transmute(u8x8::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0, 0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_u8(ptr: *const u8) -> uint8x16_t {
    let x = vld1q_lane_u8::<0>(ptr, transmute(u8x16::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_u16(ptr: *const u16) -> uint16x4_t {
    let x = vld1_lane_u16::<0>(ptr, transmute(u16x4::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_u16(ptr: *const u16) -> uint16x8_t {
    let x = vld1q_lane_u16::<0>(ptr, transmute(u16x8::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0, 0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_u32(ptr: *const u32) -> uint32x2_t {
    let x = vld1_lane_u32::<0>(ptr, transmute(u32x2::splat(0)));
    simd_shuffle!(x, x, [0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_u32(ptr: *const u32) -> uint32x4_t {
    let x = vld1q_lane_u32::<0>(ptr, transmute(u32x4::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ldr))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_u64(ptr: *const u64) -> uint64x1_t {
    #[cfg(target_arch = "aarch64")]
    {
        crate::core_arch::aarch64::vld1_u64(ptr)
    }
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::vld1_u64(ptr)
    }
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_u64(ptr: *const u64) -> uint64x2_t {
    let x = vld1q_lane_u64::<0>(ptr, transmute(u64x2::splat(0)));
    simd_shuffle!(x, x, [0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_p8(ptr: *const p8) -> poly8x8_t {
    let x = vld1_lane_p8::<0>(ptr, transmute(u8x8::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0, 0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_p8(ptr: *const p8) -> poly8x16_t {
    let x = vld1q_lane_p8::<0>(ptr, transmute(u8x16::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_p16(ptr: *const p16) -> poly16x4_t {
    let x = vld1_lane_p16::<0>(ptr, transmute(u16x4::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_p16(ptr: *const p16) -> poly16x8_t {
    let x = vld1q_lane_p16::<0>(ptr, transmute(u16x8::splat(0)));
    simd_shuffle!(x, x, [0, 0, 0, 0, 0, 0, 0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_f32(ptr: *const f32) -> float32x2_t {
    let x = vld1_lane_f32::<0>(ptr, transmute(f32x2::splat(0.)));
    simd_shuffle!(x, x, [0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vld1_dup_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ldr))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1_dup_p64(ptr: *const p64) -> poly64x1_t {
    #[cfg(target_arch = "aarch64")]
    {
        crate::core_arch::aarch64::vld1_p64(ptr)
    }
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::vld1_p64(ptr)
    }
}

/// Load one single-element structure and Replicate to all lanes (of one register).
///
/// [Arm's documentation](https://developer.arm.com/architectures/instruction-sets/intrinsics/vld1q_dup_p64)
#[inline]
#[target_feature(enable = "neon,aes")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vldr"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_p64(ptr: *const p64) -> poly64x2_t {
    let x = vld1q_lane_p64::<0>(ptr, transmute(u64x2::splat(0)));
    simd_shuffle!(x, x, [0, 0])
}

/// Load one single-element structure and Replicate to all lanes (of one register).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vld1.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ld1r))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vld1q_dup_f32(ptr: *const f32) -> float32x4_t {
    let x = vld1q_lane_f32::<0>(ptr, transmute(f32x4::splat(0.)));
    simd_shuffle!(x, x, [0, 0, 0, 0])
}

// signed absolute difference and accumulate (64-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.s8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("saba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaba_s8(a: int8x8_t, b: int8x8_t, c: int8x8_t) -> int8x8_t {
    simd_add(a, vabd_s8(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.s16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("saba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaba_s16(a: int16x4_t, b: int16x4_t, c: int16x4_t) -> int16x4_t {
    simd_add(a, vabd_s16(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.s32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("saba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaba_s32(a: int32x2_t, b: int32x2_t, c: int32x2_t) -> int32x2_t {
    simd_add(a, vabd_s32(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.u8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("uaba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaba_u8(a: uint8x8_t, b: uint8x8_t, c: uint8x8_t) -> uint8x8_t {
    simd_add(a, vabd_u8(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.u16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("uaba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaba_u16(a: uint16x4_t, b: uint16x4_t, c: uint16x4_t) -> uint16x4_t {
    simd_add(a, vabd_u16(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.u32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("uaba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaba_u32(a: uint32x2_t, b: uint32x2_t, c: uint32x2_t) -> uint32x2_t {
    simd_add(a, vabd_u32(b, c))
}
// signed absolute difference and accumulate (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.s8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("saba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabaq_s8(a: int8x16_t, b: int8x16_t, c: int8x16_t) -> int8x16_t {
    simd_add(a, vabdq_s8(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.s16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("saba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabaq_s16(a: int16x8_t, b: int16x8_t, c: int16x8_t) -> int16x8_t {
    simd_add(a, vabdq_s16(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.s32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("saba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabaq_s32(a: int32x4_t, b: int32x4_t, c: int32x4_t) -> int32x4_t {
    simd_add(a, vabdq_s32(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.u8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("uaba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabaq_u8(a: uint8x16_t, b: uint8x16_t, c: uint8x16_t) -> uint8x16_t {
    simd_add(a, vabdq_u8(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.u16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("uaba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabaq_u16(a: uint16x8_t, b: uint16x8_t, c: uint16x8_t) -> uint16x8_t {
    simd_add(a, vabdq_u16(b, c))
}
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vaba.u32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("uaba"))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabaq_u32(a: uint32x4_t, b: uint32x4_t, c: uint32x4_t) -> uint32x4_t {
    simd_add(a, vabdq_u32(b, c))
}

/// Absolute value (wrapping).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vabs))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(abs))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabs_s8(a: int8x8_t) -> int8x8_t {
    vabs_s8_(a)
}
/// Absolute value (wrapping).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vabs))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(abs))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabs_s16(a: int16x4_t) -> int16x4_t {
    vabs_s16_(a)
}
/// Absolute value (wrapping).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vabs))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(abs))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabs_s32(a: int32x2_t) -> int32x2_t {
    vabs_s32_(a)
}
/// Absolute value (wrapping).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vabs))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(abs))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabsq_s8(a: int8x16_t) -> int8x16_t {
    vabsq_s8_(a)
}
/// Absolute value (wrapping).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vabs))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(abs))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabsq_s16(a: int16x8_t) -> int16x8_t {
    vabsq_s16_(a)
}
/// Absolute value (wrapping).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vabs))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(abs))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vabsq_s32(a: int32x4_t) -> int32x4_t {
    vabsq_s32_(a)
}

/// Add pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadd_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    vpadd_s16_(a, b)
}
/// Add pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadd_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    vpadd_s32_(a, b)
}
/// Add pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadd_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    vpadd_s8_(a, b)
}
/// Add pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadd_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    transmute(vpadd_s16_(transmute(a), transmute(b)))
}
/// Add pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadd_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    transmute(vpadd_s32_(transmute(a), transmute(b)))
}
/// Add pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadd_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    transmute(vpadd_s8_(transmute(a), transmute(b)))
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vadd_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vadd_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vadd_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddq_s64(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vadd_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vadd_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vadd_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(add))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(fadd))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vadd_f32(a: float32x2_t, b: float32x2_t) -> float32x2_t {
    simd_add(a, b)
}

/// Vector add.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vadd))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(fadd))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddq_f32(a: float32x4_t, b: float32x4_t) -> float32x4_t {
    simd_add(a, b)
}

/// Signed Add Long (vector).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_s8(a: int8x8_t, b: int8x8_t) -> int16x8_t {
    let a: int16x8_t = simd_cast(a);
    let b: int16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Long (vector).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_s16(a: int16x4_t, b: int16x4_t) -> int32x4_t {
    let a: int32x4_t = simd_cast(a);
    let b: int32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Long (vector).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_s32(a: int32x2_t, b: int32x2_t) -> int64x2_t {
    let a: int64x2_t = simd_cast(a);
    let b: int64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Long (vector).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_u8(a: uint8x8_t, b: uint8x8_t) -> uint16x8_t {
    let a: uint16x8_t = simd_cast(a);
    let b: uint16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Long (vector).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_u16(a: uint16x4_t, b: uint16x4_t) -> uint32x4_t {
    let a: uint32x4_t = simd_cast(a);
    let b: uint32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Long (vector).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_u32(a: uint32x2_t, b: uint32x2_t) -> uint64x2_t {
    let a: uint64x2_t = simd_cast(a);
    let b: uint64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Long (vector, high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddl2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_high_s8(a: int8x16_t, b: int8x16_t) -> int16x8_t {
    let a: int8x8_t = simd_shuffle!(a, a, [8, 9, 10, 11, 12, 13, 14, 15]);
    let b: int8x8_t = simd_shuffle!(b, b, [8, 9, 10, 11, 12, 13, 14, 15]);
    let a: int16x8_t = simd_cast(a);
    let b: int16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Long (vector, high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddl2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_high_s16(a: int16x8_t, b: int16x8_t) -> int32x4_t {
    let a: int16x4_t = simd_shuffle!(a, a, [4, 5, 6, 7]);
    let b: int16x4_t = simd_shuffle!(b, b, [4, 5, 6, 7]);
    let a: int32x4_t = simd_cast(a);
    let b: int32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Long (vector, high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddl2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_high_s32(a: int32x4_t, b: int32x4_t) -> int64x2_t {
    let a: int32x2_t = simd_shuffle!(a, a, [2, 3]);
    let b: int32x2_t = simd_shuffle!(b, b, [2, 3]);
    let a: int64x2_t = simd_cast(a);
    let b: int64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Long (vector, high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddl2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_high_u8(a: uint8x16_t, b: uint8x16_t) -> uint16x8_t {
    let a: uint8x8_t = simd_shuffle!(a, a, [8, 9, 10, 11, 12, 13, 14, 15]);
    let b: uint8x8_t = simd_shuffle!(b, b, [8, 9, 10, 11, 12, 13, 14, 15]);
    let a: uint16x8_t = simd_cast(a);
    let b: uint16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Long (vector, high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddl2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_high_u16(a: uint16x8_t, b: uint16x8_t) -> uint32x4_t {
    let a: uint16x4_t = simd_shuffle!(a, a, [4, 5, 6, 7]);
    let b: uint16x4_t = simd_shuffle!(b, b, [4, 5, 6, 7]);
    let a: uint32x4_t = simd_cast(a);
    let b: uint32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Long (vector, high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddl2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddl_high_u32(a: uint32x4_t, b: uint32x4_t) -> uint64x2_t {
    let a: uint32x2_t = simd_shuffle!(a, a, [2, 3]);
    let b: uint32x2_t = simd_shuffle!(b, b, [2, 3]);
    let a: uint64x2_t = simd_cast(a);
    let b: uint64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Wide.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddw))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_s8(a: int16x8_t, b: int8x8_t) -> int16x8_t {
    let b: int16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Wide.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddw))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_s16(a: int32x4_t, b: int16x4_t) -> int32x4_t {
    let b: int32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Wide.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddw))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_s32(a: int64x2_t, b: int32x2_t) -> int64x2_t {
    let b: int64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Wide.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddw))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_u8(a: uint16x8_t, b: uint8x8_t) -> uint16x8_t {
    let b: uint16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Wide.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddw))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_u16(a: uint32x4_t, b: uint16x4_t) -> uint32x4_t {
    let b: uint32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Wide.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddw))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_u32(a: uint64x2_t, b: uint32x2_t) -> uint64x2_t {
    let b: uint64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Wide (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddw2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_high_s8(a: int16x8_t, b: int8x16_t) -> int16x8_t {
    let b: int8x8_t = simd_shuffle!(b, b, [8, 9, 10, 11, 12, 13, 14, 15]);
    let b: int16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Wide (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddw2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_high_s16(a: int32x4_t, b: int16x8_t) -> int32x4_t {
    let b: int16x4_t = simd_shuffle!(b, b, [4, 5, 6, 7]);
    let b: int32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Signed Add Wide (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddw2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_high_s32(a: int64x2_t, b: int32x4_t) -> int64x2_t {
    let b: int32x2_t = simd_shuffle!(b, b, [2, 3]);
    let b: int64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Wide (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddw2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_high_u8(a: uint16x8_t, b: uint8x16_t) -> uint16x8_t {
    let b: uint8x8_t = simd_shuffle!(b, b, [8, 9, 10, 11, 12, 13, 14, 15]);
    let b: uint16x8_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Wide (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddw2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_high_u16(a: uint32x4_t, b: uint16x8_t) -> uint32x4_t {
    let b: uint16x4_t = simd_shuffle!(b, b, [4, 5, 6, 7]);
    let b: uint32x4_t = simd_cast(b);
    simd_add(a, b)
}

/// Unsigned Add Wide (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddw))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddw2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddw_high_u32(a: uint64x2_t, b: uint32x4_t) -> uint64x2_t {
    let b: uint32x2_t = simd_shuffle!(b, b, [2, 3]);
    let b: uint64x2_t = simd_cast(b);
    simd_add(a, b)
}

/// Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_s16(a: int16x8_t, b: int16x8_t) -> int8x8_t {
    simd_cast(simd_shr(simd_add(a, b), int16x8_t(8, 8, 8, 8, 8, 8, 8, 8)))
}

/// Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_s32(a: int32x4_t, b: int32x4_t) -> int16x4_t {
    simd_cast(simd_shr(simd_add(a, b), int32x4_t(16, 16, 16, 16)))
}

/// Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_s64(a: int64x2_t, b: int64x2_t) -> int32x2_t {
    simd_cast(simd_shr(simd_add(a, b), int64x2_t(32, 32)))
}

/// Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_u16(a: uint16x8_t, b: uint16x8_t) -> uint8x8_t {
    simd_cast(simd_shr(simd_add(a, b), uint16x8_t(8, 8, 8, 8, 8, 8, 8, 8)))
}

/// Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_u32(a: uint32x4_t, b: uint32x4_t) -> uint16x4_t {
    simd_cast(simd_shr(simd_add(a, b), uint32x4_t(16, 16, 16, 16)))
}

/// Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_u64(a: uint64x2_t, b: uint64x2_t) -> uint32x2_t {
    simd_cast(simd_shr(simd_add(a, b), uint64x2_t(32, 32)))
}

/// Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_high_s16(r: int8x8_t, a: int16x8_t, b: int16x8_t) -> int8x16_t {
    let x = simd_cast(simd_shr(simd_add(a, b), int16x8_t(8, 8, 8, 8, 8, 8, 8, 8)));
    simd_shuffle!(r, x, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
}

/// Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_high_s32(r: int16x4_t, a: int32x4_t, b: int32x4_t) -> int16x8_t {
    let x = simd_cast(simd_shr(simd_add(a, b), int32x4_t(16, 16, 16, 16)));
    simd_shuffle!(r, x, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_high_s64(r: int32x2_t, a: int64x2_t, b: int64x2_t) -> int32x4_t {
    let x = simd_cast(simd_shr(simd_add(a, b), int64x2_t(32, 32)));
    simd_shuffle!(r, x, [0, 1, 2, 3])
}

/// Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_high_u16(r: uint8x8_t, a: uint16x8_t, b: uint16x8_t) -> uint8x16_t {
    let x = simd_cast(simd_shr(simd_add(a, b), uint16x8_t(8, 8, 8, 8, 8, 8, 8, 8)));
    simd_shuffle!(r, x, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
}

/// Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_high_u32(r: uint16x4_t, a: uint32x4_t, b: uint32x4_t) -> uint16x8_t {
    let x = simd_cast(simd_shr(simd_add(a, b), uint32x4_t(16, 16, 16, 16)));
    simd_shuffle!(r, x, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vaddhn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(addhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vaddhn_high_u64(r: uint32x2_t, a: uint64x2_t, b: uint64x2_t) -> uint32x4_t {
    let x = simd_cast(simd_shr(simd_add(a, b), uint64x2_t(32, 32)));
    simd_shuffle!(r, x, [0, 1, 2, 3])
}

/// Rounding Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_s16(a: int16x8_t, b: int16x8_t) -> int8x8_t {
    vraddhn_s16_(a, b)
}

/// Rounding Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_s32(a: int32x4_t, b: int32x4_t) -> int16x4_t {
    vraddhn_s32_(a, b)
}

/// Rounding Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i64))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_s64(a: int64x2_t, b: int64x2_t) -> int32x2_t {
    vraddhn_s64_(a, b)
}

/// Rounding Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_u16(a: uint16x8_t, b: uint16x8_t) -> uint8x8_t {
    transmute(vraddhn_s16_(transmute(a), transmute(b)))
}

/// Rounding Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_u32(a: uint32x4_t, b: uint32x4_t) -> uint16x4_t {
    transmute(vraddhn_s32_(transmute(a), transmute(b)))
}

/// Rounding Add returning High Narrow.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i64))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_u64(a: uint64x2_t, b: uint64x2_t) -> uint32x2_t {
    transmute(vraddhn_s64_(transmute(a), transmute(b)))
}

/// Rounding Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_high_s16(r: int8x8_t, a: int16x8_t, b: int16x8_t) -> int8x16_t {
    let x = vraddhn_s16_(a, b);
    simd_shuffle!(r, x, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
}

/// Rounding Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_high_s32(r: int16x4_t, a: int32x4_t, b: int32x4_t) -> int16x8_t {
    let x = vraddhn_s32_(a, b);
    simd_shuffle!(r, x, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Rounding Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i64))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_high_s64(r: int32x2_t, a: int64x2_t, b: int64x2_t) -> int32x4_t {
    let x = vraddhn_s64_(a, b);
    simd_shuffle!(r, x, [0, 1, 2, 3])
}

/// Rounding Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_high_u16(r: uint8x8_t, a: uint16x8_t, b: uint16x8_t) -> uint8x16_t {
    let x: uint8x8_t = transmute(vraddhn_s16_(transmute(a), transmute(b)));
    simd_shuffle!(r, x, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
}

/// Rounding Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_high_u32(r: uint16x4_t, a: uint32x4_t, b: uint32x4_t) -> uint16x8_t {
    let x: uint16x4_t = transmute(vraddhn_s32_(transmute(a), transmute(b)));
    simd_shuffle!(r, x, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Rounding Add returning High Narrow (high half).
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vraddhn.i64))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(raddhn2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vraddhn_high_u64(r: uint32x2_t, a: uint64x2_t, b: uint64x2_t) -> uint32x4_t {
    let x: uint32x2_t = transmute(vraddhn_s64_(transmute(a), transmute(b)));
    simd_shuffle!(r, x, [0, 1, 2, 3])
}

/// Signed Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.s8))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddl_s8(a: int8x8_t) -> int16x4_t {
    vpaddl_s8_(a)
}

/// Signed Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.s16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddl_s16(a: int16x4_t) -> int32x2_t {
    vpaddl_s16_(a)
}

/// Signed Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.s32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddl_s32(a: int32x2_t) -> int64x1_t {
    vpaddl_s32_(a)
}

/// Signed Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.s8))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddlq_s8(a: int8x16_t) -> int16x8_t {
    vpaddlq_s8_(a)
}

/// Signed Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.s16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddlq_s16(a: int16x8_t) -> int32x4_t {
    vpaddlq_s16_(a)
}

/// Signed Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.s32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(saddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddlq_s32(a: int32x4_t) -> int64x2_t {
    vpaddlq_s32_(a)
}

/// Unsigned Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.u8))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddl_u8(a: uint8x8_t) -> uint16x4_t {
    vpaddl_u8_(a)
}

/// Unsigned Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.u16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddl_u16(a: uint16x4_t) -> uint32x2_t {
    vpaddl_u16_(a)
}

/// Unsigned Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.u32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddl_u32(a: uint32x2_t) -> uint64x1_t {
    vpaddl_u32_(a)
}

/// Unsigned Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.u8))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddlq_u8(a: uint8x16_t) -> uint16x8_t {
    vpaddlq_u8_(a)
}

/// Unsigned Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.u16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddlq_u16(a: uint16x8_t) -> uint32x4_t {
    vpaddlq_u16_(a)
}

/// Unsigned Add Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpaddl.u32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uaddlp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpaddlq_u32(a: uint32x4_t) -> uint64x2_t {
    vpaddlq_u32_(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(xtn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovn_s16(a: int16x8_t) -> int8x8_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(xtn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovn_s32(a: int32x4_t) -> int16x4_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(xtn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovn_s64(a: int64x2_t) -> int32x2_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(xtn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovn_u16(a: uint16x8_t) -> uint8x8_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(xtn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovn_u32(a: uint32x4_t) -> uint16x4_t {
    simd_cast(a)
}

/// Vector narrow integer.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(xtn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovn_u64(a: uint64x2_t) -> uint32x2_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sxtl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovl_s8(a: int8x8_t) -> int16x8_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sxtl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovl_s16(a: int16x4_t) -> int32x4_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sxtl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovl_s32(a: int32x2_t) -> int64x2_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uxtl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovl_u8(a: uint8x8_t) -> uint16x8_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uxtl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovl_u16(a: uint16x4_t) -> uint32x4_t {
    simd_cast(a)
}

/// Vector long move.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmovl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uxtl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovl_u32(a: uint32x2_t) -> uint64x2_t {
    simd_cast(a)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvn_s8(a: int8x8_t) -> int8x8_t {
    let b = int8x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvnq_s8(a: int8x16_t) -> int8x16_t {
    let b = int8x16_t(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvn_s16(a: int16x4_t) -> int16x4_t {
    let b = int16x4_t(-1, -1, -1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvnq_s16(a: int16x8_t) -> int16x8_t {
    let b = int16x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvn_s32(a: int32x2_t) -> int32x2_t {
    let b = int32x2_t(-1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvnq_s32(a: int32x4_t) -> int32x4_t {
    let b = int32x4_t(-1, -1, -1, -1);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvn_u8(a: uint8x8_t) -> uint8x8_t {
    let b = uint8x8_t(255, 255, 255, 255, 255, 255, 255, 255);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvnq_u8(a: uint8x16_t) -> uint8x16_t {
    let b = uint8x16_t(
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    );
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvn_u16(a: uint16x4_t) -> uint16x4_t {
    let b = uint16x4_t(65_535, 65_535, 65_535, 65_535);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvnq_u16(a: uint16x8_t) -> uint16x8_t {
    let b = uint16x8_t(
        65_535, 65_535, 65_535, 65_535, 65_535, 65_535, 65_535, 65_535,
    );
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvn_u32(a: uint32x2_t) -> uint32x2_t {
    let b = uint32x2_t(4_294_967_295, 4_294_967_295);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvnq_u32(a: uint32x4_t) -> uint32x4_t {
    let b = uint32x4_t(4_294_967_295, 4_294_967_295, 4_294_967_295, 4_294_967_295);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvn_p8(a: poly8x8_t) -> poly8x8_t {
    let b = poly8x8_t(255, 255, 255, 255, 255, 255, 255, 255);
    simd_xor(a, b)
}

/// Vector bitwise not.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vmvn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mvn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmvnq_p8(a: poly8x16_t) -> poly8x16_t {
    let b = poly8x16_t(
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    );
    simd_xor(a, b)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbic_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    let c = int8x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_and(simd_xor(b, c), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbicq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    let c = int8x16_t(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    simd_and(simd_xor(b, c), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbic_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    let c = int16x4_t(-1, -1, -1, -1);
    simd_and(simd_xor(b, c), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbicq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    let c = int16x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_and(simd_xor(b, c), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbic_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    let c = int32x2_t(-1, -1);
    simd_and(simd_xor(b, c), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbicq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    let c = int32x4_t(-1, -1, -1, -1);
    simd_and(simd_xor(b, c), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbic_s64(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    let c = int64x1_t(-1);
    simd_and(simd_xor(b, c), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbicq_s64(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    let c = int64x2_t(-1, -1);
    simd_and(simd_xor(b, c), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbic_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    let c = int8x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_and(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbicq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    let c = int8x16_t(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    simd_and(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbic_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    let c = int16x4_t(-1, -1, -1, -1);
    simd_and(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbicq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    let c = int16x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_and(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbic_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    let c = int32x2_t(-1, -1);
    simd_and(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbicq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    let c = int32x4_t(-1, -1, -1, -1);
    simd_and(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbic_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    let c = int64x1_t(-1);
    simd_and(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise bit clear
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbic))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bic))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbicq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    let c = int64x2_t(-1, -1);
    simd_and(simd_xor(b, transmute(c)), a)
}

/// Bitwise Select instructions. This instruction sets each bit in the destination SIMD&FP register
/// to the corresponding bit from the first source SIMD&FP register when the original
/// destination bit was 1, otherwise from the second source SIMD&FP register.

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_s8(a: uint8x8_t, b: int8x8_t, c: int8x8_t) -> int8x8_t {
    let not = int8x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_s16(a: uint16x4_t, b: int16x4_t, c: int16x4_t) -> int16x4_t {
    let not = int16x4_t(-1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_s32(a: uint32x2_t, b: int32x2_t, c: int32x2_t) -> int32x2_t {
    let not = int32x2_t(-1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_s64(a: uint64x1_t, b: int64x1_t, c: int64x1_t) -> int64x1_t {
    let not = int64x1_t(-1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_u8(a: uint8x8_t, b: uint8x8_t, c: uint8x8_t) -> uint8x8_t {
    let not = int8x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_u16(a: uint16x4_t, b: uint16x4_t, c: uint16x4_t) -> uint16x4_t {
    let not = int16x4_t(-1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_u32(a: uint32x2_t, b: uint32x2_t, c: uint32x2_t) -> uint32x2_t {
    let not = int32x2_t(-1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_u64(a: uint64x1_t, b: uint64x1_t, c: uint64x1_t) -> uint64x1_t {
    let not = int64x1_t(-1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_f32(a: uint32x2_t, b: float32x2_t, c: float32x2_t) -> float32x2_t {
    let not = int32x2_t(-1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_p8(a: uint8x8_t, b: poly8x8_t, c: poly8x8_t) -> poly8x8_t {
    let not = int8x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbsl_p16(a: uint16x4_t, b: poly16x4_t, c: poly16x4_t) -> poly16x4_t {
    let not = int16x4_t(-1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_s8(a: uint8x16_t, b: int8x16_t, c: int8x16_t) -> int8x16_t {
    let not = int8x16_t(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_s16(a: uint16x8_t, b: int16x8_t, c: int16x8_t) -> int16x8_t {
    let not = int16x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_s32(a: uint32x4_t, b: int32x4_t, c: int32x4_t) -> int32x4_t {
    let not = int32x4_t(-1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_s64(a: uint64x2_t, b: int64x2_t, c: int64x2_t) -> int64x2_t {
    let not = int64x2_t(-1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_u8(a: uint8x16_t, b: uint8x16_t, c: uint8x16_t) -> uint8x16_t {
    let not = int8x16_t(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_u16(a: uint16x8_t, b: uint16x8_t, c: uint16x8_t) -> uint16x8_t {
    let not = int16x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_u32(a: uint32x4_t, b: uint32x4_t, c: uint32x4_t) -> uint32x4_t {
    let not = int32x4_t(-1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_u64(a: uint64x2_t, b: uint64x2_t, c: uint64x2_t) -> uint64x2_t {
    let not = int64x2_t(-1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_p8(a: uint8x16_t, b: poly8x16_t, c: poly8x16_t) -> poly8x16_t {
    let not = int8x16_t(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_p16(a: uint16x8_t, b: poly16x8_t, c: poly16x8_t) -> poly16x8_t {
    let not = int16x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Bitwise Select. (128-bit)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vbsl))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(bsl))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vbslq_f32(a: uint32x4_t, b: float32x4_t, c: float32x4_t) -> float32x4_t {
    let not = int32x4_t(-1, -1, -1, -1);
    transmute(simd_or(
        simd_and(a, transmute(b)),
        simd_and(simd_xor(a, transmute(not)), transmute(c)),
    ))
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vorn_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    let c = int8x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_or(simd_xor(b, c), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vornq_s8(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    let c = int8x16_t(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    simd_or(simd_xor(b, c), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vorn_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    let c = int16x4_t(-1, -1, -1, -1);
    simd_or(simd_xor(b, c), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vornq_s16(a: int16x8_t, b: int16x8_t) -> int16x8_t {
    let c = int16x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_or(simd_xor(b, c), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vorn_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    let c = int32x2_t(-1, -1);
    simd_or(simd_xor(b, c), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vornq_s32(a: int32x4_t, b: int32x4_t) -> int32x4_t {
    let c = int32x4_t(-1, -1, -1, -1);
    simd_or(simd_xor(b, c), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vorn_s64(a: int64x1_t, b: int64x1_t) -> int64x1_t {
    let c = int64x1_t(-1);
    simd_or(simd_xor(b, c), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vornq_s64(a: int64x2_t, b: int64x2_t) -> int64x2_t {
    let c = int64x2_t(-1, -1);
    simd_or(simd_xor(b, c), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vorn_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    let c = int8x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_or(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vornq_u8(a: uint8x16_t, b: uint8x16_t) -> uint8x16_t {
    let c = int8x16_t(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );
    simd_or(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vorn_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    let c = int16x4_t(-1, -1, -1, -1);
    simd_or(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vornq_u16(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    let c = int16x8_t(-1, -1, -1, -1, -1, -1, -1, -1);
    simd_or(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vorn_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    let c = int32x2_t(-1, -1);
    simd_or(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vornq_u32(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    let c = int32x4_t(-1, -1, -1, -1);
    simd_or(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vorn_u64(a: uint64x1_t, b: uint64x1_t) -> uint64x1_t {
    let c = int64x1_t(-1);
    simd_or(simd_xor(b, transmute(c)), a)
}

/// Vector bitwise inclusive OR NOT
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vorn))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(orn))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vornq_u64(a: uint64x2_t, b: uint64x2_t) -> uint64x2_t {
    let c = int64x2_t(-1, -1);
    simd_or(simd_xor(b, transmute(c)), a)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmin))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sminp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmin_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    vpmins_v8i8(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmin))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sminp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmin_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    vpmins_v4i16(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmin))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sminp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmin_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    vpmins_v2i32(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmin))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uminp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmin_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    vpminu_v8i8(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmin))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uminp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmin_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    vpminu_v4i16(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmin))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uminp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmin_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    vpminu_v2i32(a, b)
}

/// Folding minimum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmin))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(fminp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmin_f32(a: float32x2_t, b: float32x2_t) -> float32x2_t {
    vpminf_v2f32(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmax))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(smaxp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmax_s8(a: int8x8_t, b: int8x8_t) -> int8x8_t {
    vpmaxs_v8i8(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmax))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(smaxp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmax_s16(a: int16x4_t, b: int16x4_t) -> int16x4_t {
    vpmaxs_v4i16(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmax))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(smaxp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmax_s32(a: int32x2_t, b: int32x2_t) -> int32x2_t {
    vpmaxs_v2i32(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmax))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(umaxp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmax_u8(a: uint8x8_t, b: uint8x8_t) -> uint8x8_t {
    vpmaxu_v8i8(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmax))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(umaxp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmax_u16(a: uint16x4_t, b: uint16x4_t) -> uint16x4_t {
    vpmaxu_v4i16(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmax))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(umaxp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmax_u32(a: uint32x2_t, b: uint32x2_t) -> uint32x2_t {
    vpmaxu_v2i32(a, b)
}

/// Folding maximum of adjacent pairs
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpmax))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(fmaxp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpmax_f32(a: float32x2_t, b: float32x2_t) -> float32x2_t {
    vpmaxf_v2f32(a, b)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_u64<const IMM5: i32>(v: uint64x2_t) -> u64 {
    static_assert_uimm_bits!(IMM5, 1);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 0))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_u64<const IMM5: i32>(v: uint64x1_t) -> u64 {
    static_assert!(IMM5 == 0);
    simd_extract(v, 0)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_u16<const IMM5: i32>(v: uint16x4_t) -> u16 {
    static_assert_uimm_bits!(IMM5, 2);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_s16<const IMM5: i32>(v: int16x4_t) -> i16 {
    static_assert_uimm_bits!(IMM5, 2);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_p16<const IMM5: i32>(v: poly16x4_t) -> p16 {
    static_assert_uimm_bits!(IMM5, 2);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_u32<const IMM5: i32>(v: uint32x2_t) -> u32 {
    static_assert_uimm_bits!(IMM5, 1);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_s32<const IMM5: i32>(v: int32x2_t) -> i32 {
    static_assert_uimm_bits!(IMM5, 1);
    simd_extract(v, IMM5 as u32)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_f32<const IMM5: i32>(v: float32x2_t) -> f32 {
    static_assert_uimm_bits!(IMM5, 1);
    simd_extract(v, IMM5 as u32)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 1))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_f32<const IMM5: i32>(v: float32x4_t) -> f32 {
    static_assert_uimm_bits!(IMM5, 2);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 0))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_p64<const IMM5: i32>(v: poly64x1_t) -> p64 {
    static_assert!(IMM5 == 0);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 0))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_p64<const IMM5: i32>(v: poly64x2_t) -> p64 {
    static_assert_uimm_bits!(IMM5, 1);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 0))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_s64<const IMM5: i32>(v: int64x1_t) -> i64 {
    static_assert!(IMM5 == 0);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 0))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_s64<const IMM5: i32>(v: int64x2_t) -> i64 {
    static_assert_uimm_bits!(IMM5, 1);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_u16<const IMM5: i32>(v: uint16x8_t) -> u16 {
    static_assert_uimm_bits!(IMM5, 3);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_u32<const IMM5: i32>(v: uint32x4_t) -> u32 {
    static_assert_uimm_bits!(IMM5, 2);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_s16<const IMM5: i32>(v: int16x8_t) -> i16 {
    static_assert_uimm_bits!(IMM5, 3);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_p16<const IMM5: i32>(v: poly16x8_t) -> p16 {
    static_assert_uimm_bits!(IMM5, 3);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_s32<const IMM5: i32>(v: int32x4_t) -> i32 {
    static_assert_uimm_bits!(IMM5, 2);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_u8<const IMM5: i32>(v: uint8x8_t) -> u8 {
    static_assert_uimm_bits!(IMM5, 3);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_s8<const IMM5: i32>(v: int8x8_t) -> i8 {
    static_assert_uimm_bits!(IMM5, 3);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_lane_p8<const IMM5: i32>(v: poly8x8_t) -> p8 {
    static_assert_uimm_bits!(IMM5, 3);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_u8<const IMM5: i32>(v: uint8x16_t) -> u8 {
    static_assert_uimm_bits!(IMM5, 4);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_s8<const IMM5: i32>(v: int8x16_t) -> i8 {
    static_assert_uimm_bits!(IMM5, 4);
    simd_extract(v, IMM5 as u32)
}

/// Move vector element to general-purpose register
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[rustc_legacy_const_generics(1)]
#[cfg_attr(test, assert_instr(nop, IMM5 = 2))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vgetq_lane_p8<const IMM5: i32>(v: poly8x16_t) -> p8 {
    static_assert_uimm_bits!(IMM5, 4);
    simd_extract(v, IMM5 as u32)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_s8(a: int8x16_t) -> int8x8_t {
    simd_shuffle!(a, a, [8, 9, 10, 11, 12, 13, 14, 15])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_s16(a: int16x8_t) -> int16x4_t {
    simd_shuffle!(a, a, [4, 5, 6, 7])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_s32(a: int32x4_t) -> int32x2_t {
    simd_shuffle!(a, a, [2, 3])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_s64(a: int64x2_t) -> int64x1_t {
    int64x1_t(simd_extract(a, 1))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_u8(a: uint8x16_t) -> uint8x8_t {
    simd_shuffle!(a, a, [8, 9, 10, 11, 12, 13, 14, 15])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_u16(a: uint16x8_t) -> uint16x4_t {
    simd_shuffle!(a, a, [4, 5, 6, 7])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_u32(a: uint32x4_t) -> uint32x2_t {
    simd_shuffle!(a, a, [2, 3])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_u64(a: uint64x2_t) -> uint64x1_t {
    uint64x1_t(simd_extract(a, 1))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_p8(a: poly8x16_t) -> poly8x8_t {
    simd_shuffle!(a, a, [8, 9, 10, 11, 12, 13, 14, 15])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_p16(a: poly16x8_t) -> poly16x4_t {
    simd_shuffle!(a, a, [4, 5, 6, 7])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ext))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_high_f32(a: float32x4_t) -> float32x2_t {
    simd_shuffle!(a, a, [2, 3])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "vget_low_s8", since = "1.60.0")
)]
pub unsafe fn vget_low_s8(a: int8x16_t) -> int8x8_t {
    simd_shuffle!(a, a, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_s16(a: int16x8_t) -> int16x4_t {
    simd_shuffle!(a, a, [0, 1, 2, 3])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_s32(a: int32x4_t) -> int32x2_t {
    simd_shuffle!(a, a, [0, 1])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_s64(a: int64x2_t) -> int64x1_t {
    int64x1_t(simd_extract(a, 0))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_u8(a: uint8x16_t) -> uint8x8_t {
    simd_shuffle!(a, a, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_u16(a: uint16x8_t) -> uint16x4_t {
    simd_shuffle!(a, a, [0, 1, 2, 3])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_u32(a: uint32x4_t) -> uint32x2_t {
    simd_shuffle!(a, a, [0, 1])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_u64(a: uint64x2_t) -> uint64x1_t {
    uint64x1_t(simd_extract(a, 0))
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_p8(a: poly8x16_t) -> poly8x8_t {
    simd_shuffle!(a, a, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_p16(a: poly16x8_t) -> poly16x4_t {
    simd_shuffle!(a, a, [0, 1, 2, 3])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vget_low_f32(a: float32x4_t) -> float32x2_t {
    simd_shuffle!(a, a, [0, 1])
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_s8(value: i8) -> int8x16_t {
    int8x16_t(
        value, value, value, value, value, value, value, value, value, value, value, value, value,
        value, value, value,
    )
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_s16(value: i16) -> int16x8_t {
    int16x8_t(value, value, value, value, value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_s32(value: i32) -> int32x4_t {
    int32x4_t(value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_s64(value: i64) -> int64x2_t {
    int64x2_t(value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_u8(value: u8) -> uint8x16_t {
    uint8x16_t(
        value, value, value, value, value, value, value, value, value, value, value, value, value,
        value, value, value,
    )
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_u16(value: u16) -> uint16x8_t {
    uint16x8_t(value, value, value, value, value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_u32(value: u32) -> uint32x4_t {
    uint32x4_t(value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_u64(value: u64) -> uint64x2_t {
    uint64x2_t(value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_p8(value: p8) -> poly8x16_t {
    poly8x16_t(
        value, value, value, value, value, value, value, value, value, value, value, value, value,
        value, value, value,
    )
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_p16(value: p16) -> poly16x8_t {
    poly16x8_t(value, value, value, value, value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdupq_n_f32(value: f32) -> float32x4_t {
    float32x4_t(value, value, value, value)
}

/// Duplicate vector element to vector or scalar
///
/// Private vfp4 version used by FMA intriniscs because LLVM does
/// not inline the non-vfp4 version in vfp4 functions.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "vfp4"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
unsafe fn vdupq_n_f32_vfp4(value: f32) -> float32x4_t {
    float32x4_t(value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_s8(value: i8) -> int8x8_t {
    int8x8_t(value, value, value, value, value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_s16(value: i16) -> int16x4_t {
    int16x4_t(value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_s32(value: i32) -> int32x2_t {
    int32x2_t(value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(fmov))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_s64(value: i64) -> int64x1_t {
    int64x1_t(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_u8(value: u8) -> uint8x8_t {
    uint8x8_t(value, value, value, value, value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_u16(value: u16) -> uint16x4_t {
    uint16x4_t(value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_u32(value: u32) -> uint32x2_t {
    uint32x2_t(value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(fmov))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_u64(value: u64) -> uint64x1_t {
    uint64x1_t(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_p8(value: p8) -> poly8x8_t {
    poly8x8_t(value, value, value, value, value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_p16(value: p16) -> poly16x4_t {
    poly16x4_t(value, value, value, value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vdup_n_f32(value: f32) -> float32x2_t {
    float32x2_t(value, value)
}

/// Duplicate vector element to vector or scalar
///
/// Private vfp4 version used by FMA intriniscs because LLVM does
/// not inline the non-vfp4 version in vfp4 functions.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "vfp4"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
unsafe fn vdup_n_f32_vfp4(value: f32) -> float32x2_t {
    float32x2_t(value, value)
}

/// Load SIMD&FP register (immediate offset)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(nop))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vldrq_p128(a: *const p128) -> p128 {
    *a
}

/// Store SIMD&FP register (immediate offset)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(nop))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(nop))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vstrq_p128(a: *mut p128, b: p128) {
    *a = b;
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_s8(value: i8) -> int8x8_t {
    vdup_n_s8(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_s16(value: i16) -> int16x4_t {
    vdup_n_s16(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_s32(value: i32) -> int32x2_t {
    vdup_n_s32(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(fmov))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_s64(value: i64) -> int64x1_t {
    vdup_n_s64(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_u8(value: u8) -> uint8x8_t {
    vdup_n_u8(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_u16(value: u16) -> uint16x4_t {
    vdup_n_u16(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_u32(value: u32) -> uint32x2_t {
    vdup_n_u32(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(fmov))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_u64(value: u64) -> uint64x1_t {
    vdup_n_u64(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_p8(value: p8) -> poly8x8_t {
    vdup_n_p8(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_p16(value: p16) -> poly16x4_t {
    vdup_n_p16(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmov_n_f32(value: f32) -> float32x2_t {
    vdup_n_f32(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_s8(value: i8) -> int8x16_t {
    vdupq_n_s8(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_s16(value: i16) -> int16x8_t {
    vdupq_n_s16(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_s32(value: i32) -> int32x4_t {
    vdupq_n_s32(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_s64(value: i64) -> int64x2_t {
    vdupq_n_s64(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_u8(value: u8) -> uint8x16_t {
    vdupq_n_u8(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_u16(value: u16) -> uint16x8_t {
    vdupq_n_u16(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_u32(value: u32) -> uint32x4_t {
    vdupq_n_u32(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vmov"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_u64(value: u64) -> uint64x2_t {
    vdupq_n_u64(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_p8(value: p8) -> poly8x16_t {
    vdupq_n_p8(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_p16(value: p16) -> poly16x8_t {
    vdupq_n_p16(value)
}

/// Duplicate vector element to vector or scalar
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vdup.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(dup))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vmovq_n_f32(value: f32) -> float32x4_t {
    vdupq_n_f32(value)
}

/// Extract vector from pair of vectors
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("nop", N = 0))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("nop", N = 0))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vext_s64<const N: i32>(a: int64x1_t, _b: int64x1_t) -> int64x1_t {
    static_assert!(N == 0);
    a
}

/// Extract vector from pair of vectors
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("nop", N = 0))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr("nop", N = 0))]
#[rustc_legacy_const_generics(2)]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vext_u64<const N: i32>(a: uint64x1_t, _b: uint64x1_t) -> uint64x1_t {
    static_assert!(N == 0);
    a
}

/// Population count per byte.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vcnt))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(cnt))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcnt_s8(a: int8x8_t) -> int8x8_t {
    vcnt_s8_(a)
}
/// Population count per byte.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vcnt))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(cnt))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcntq_s8(a: int8x16_t) -> int8x16_t {
    vcntq_s8_(a)
}
/// Population count per byte.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vcnt))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(cnt))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcnt_u8(a: uint8x8_t) -> uint8x8_t {
    transmute(vcnt_s8_(transmute(a)))
}
/// Population count per byte.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vcnt))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(cnt))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcntq_u8(a: uint8x16_t) -> uint8x16_t {
    transmute(vcntq_s8_(transmute(a)))
}
/// Population count per byte.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vcnt))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(cnt))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcnt_p8(a: poly8x8_t) -> poly8x8_t {
    transmute(vcnt_s8_(transmute(a)))
}
/// Population count per byte.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vcnt))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(cnt))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcntq_p8(a: poly8x16_t) -> poly8x16_t {
    transmute(vcntq_s8_(transmute(a)))
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev16.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev16))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev16_s8(a: int8x8_t) -> int8x8_t {
    simd_shuffle!(a, a, [1, 0, 3, 2, 5, 4, 7, 6])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev16.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev16))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev16q_s8(a: int8x16_t) -> int8x16_t {
    simd_shuffle!(a, a, [1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev16.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev16))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev16_u8(a: uint8x8_t) -> uint8x8_t {
    simd_shuffle!(a, a, [1, 0, 3, 2, 5, 4, 7, 6])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev16.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev16))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev16q_u8(a: uint8x16_t) -> uint8x16_t {
    simd_shuffle!(a, a, [1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev16.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev16))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev16_p8(a: poly8x8_t) -> poly8x8_t {
    simd_shuffle!(a, a, [1, 0, 3, 2, 5, 4, 7, 6])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev16.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev16))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev16q_p8(a: poly8x16_t) -> poly8x16_t {
    simd_shuffle!(a, a, [1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32_s8(a: int8x8_t) -> int8x8_t {
    simd_shuffle!(a, a, [3, 2, 1, 0, 7, 6, 5, 4])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32q_s8(a: int8x16_t) -> int8x16_t {
    simd_shuffle!(a, a, [3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32_u8(a: uint8x8_t) -> uint8x8_t {
    simd_shuffle!(a, a, [3, 2, 1, 0, 7, 6, 5, 4])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32q_u8(a: uint8x16_t) -> uint8x16_t {
    simd_shuffle!(a, a, [3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32_s16(a: int16x4_t) -> int16x4_t {
    simd_shuffle!(a, a, [1, 0, 3, 2])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32q_s16(a: int16x8_t) -> int16x8_t {
    simd_shuffle!(a, a, [1, 0, 3, 2, 5, 4, 7, 6])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32_p16(a: poly16x4_t) -> poly16x4_t {
    simd_shuffle!(a, a, [1, 0, 3, 2])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32q_p16(a: poly16x8_t) -> poly16x8_t {
    simd_shuffle!(a, a, [1, 0, 3, 2, 5, 4, 7, 6])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32_u16(a: uint16x4_t) -> uint16x4_t {
    simd_shuffle!(a, a, [1, 0, 3, 2])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32q_u16(a: uint16x8_t) -> uint16x8_t {
    simd_shuffle!(a, a, [1, 0, 3, 2, 5, 4, 7, 6])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32_p8(a: poly8x8_t) -> poly8x8_t {
    simd_shuffle!(a, a, [3, 2, 1, 0, 7, 6, 5, 4])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev32.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev32))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev32q_p8(a: poly8x16_t) -> poly8x16_t {
    simd_shuffle!(a, a, [3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64_s8(a: int8x8_t) -> int8x8_t {
    simd_shuffle!(a, a, [7, 6, 5, 4, 3, 2, 1, 0])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64q_s8(a: int8x16_t) -> int8x16_t {
    simd_shuffle!(a, a, [7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64_s16(a: int16x4_t) -> int16x4_t {
    simd_shuffle!(a, a, [3, 2, 1, 0])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64q_s16(a: int16x8_t) -> int16x8_t {
    simd_shuffle!(a, a, [3, 2, 1, 0, 7, 6, 5, 4])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64_s32(a: int32x2_t) -> int32x2_t {
    simd_shuffle!(a, a, [1, 0])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64q_s32(a: int32x4_t) -> int32x4_t {
    simd_shuffle!(a, a, [1, 0, 3, 2])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64_u8(a: uint8x8_t) -> uint8x8_t {
    simd_shuffle!(a, a, [7, 6, 5, 4, 3, 2, 1, 0])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64q_u8(a: uint8x16_t) -> uint8x16_t {
    simd_shuffle!(a, a, [7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64_u16(a: uint16x4_t) -> uint16x4_t {
    simd_shuffle!(a, a, [3, 2, 1, 0])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64q_u16(a: uint16x8_t) -> uint16x8_t {
    simd_shuffle!(a, a, [3, 2, 1, 0, 7, 6, 5, 4])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64_u32(a: uint32x2_t) -> uint32x2_t {
    simd_shuffle!(a, a, [1, 0])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64q_u32(a: uint32x4_t) -> uint32x4_t {
    simd_shuffle!(a, a, [1, 0, 3, 2])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64_f32(a: float32x2_t) -> float32x2_t {
    simd_shuffle!(a, a, [1, 0])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.32"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64q_f32(a: float32x4_t) -> float32x4_t {
    simd_shuffle!(a, a, [1, 0, 3, 2])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64_p8(a: poly8x8_t) -> poly8x8_t {
    simd_shuffle!(a, a, [7, 6, 5, 4, 3, 2, 1, 0])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.8"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64q_p8(a: poly8x16_t) -> poly8x16_t {
    simd_shuffle!(a, a, [7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64_p16(a: poly16x4_t) -> poly16x4_t {
    simd_shuffle!(a, a, [3, 2, 1, 0])
}

/// Reversing vector elements (swap endianness)
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr("vrev64.16"))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(rev64))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vrev64q_p16(a: poly16x8_t) -> poly16x8_t {
    simd_shuffle!(a, a, [3, 2, 1, 0, 7, 6, 5, 4])
}

/// Signed Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.s8))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadal_s8(a: int16x4_t, b: int8x8_t) -> int16x4_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadal_s8_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddl_s8_(b), a)
    }
}

/// Signed Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.s16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadal_s16(a: int32x2_t, b: int16x4_t) -> int32x2_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadal_s16_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddl_s16_(b), a)
    }
}

/// Signed Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.s32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadal_s32(a: int64x1_t, b: int32x2_t) -> int64x1_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadal_s32_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddl_s32_(b), a)
    }
}

/// Signed Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.s8))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadalq_s8(a: int16x8_t, b: int8x16_t) -> int16x8_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadalq_s8_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddlq_s8_(b), a)
    }
}

/// Signed Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.s16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadalq_s16(a: int32x4_t, b: int16x8_t) -> int32x4_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadalq_s16_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddlq_s16_(b), a)
    }
}

/// Signed Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.s32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(sadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadalq_s32(a: int64x2_t, b: int32x4_t) -> int64x2_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadalq_s32_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddlq_s32_(b), a)
    }
}

/// Unsigned Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.u8))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadal_u8(a: uint16x4_t, b: uint8x8_t) -> uint16x4_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadal_u8_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddl_u8_(b), a)
    }
}

/// Unsigned Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.u16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadal_u16(a: uint32x2_t, b: uint16x4_t) -> uint32x2_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadal_u16_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddl_u16_(b), a)
    }
}

/// Unsigned Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.u32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadal_u32(a: uint64x1_t, b: uint32x2_t) -> uint64x1_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadal_u32_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddl_u32_(b), a)
    }
}

/// Unsigned Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.u8))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadalq_u8(a: uint16x8_t, b: uint8x16_t) -> uint16x8_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadalq_u8_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddlq_u8_(b), a)
    }
}

/// Unsigned Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.u16))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadalq_u16(a: uint32x4_t, b: uint16x8_t) -> uint32x4_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadalq_u16_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddlq_u16_(b), a)
    }
}

/// Unsigned Add and Accumulate Long Pairwise.
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(vpadal.u32))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(uadalp))]
#[cfg_attr(
    not(target_arch = "arm"),
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vpadalq_u32(a: uint64x2_t, b: uint32x4_t) -> uint64x2_t {
    #[cfg(target_arch = "arm")]
    {
        crate::core_arch::arm::neon::vpadalq_u32_(a, b)
    }
    #[cfg(target_arch = "aarch64")]
    {
        simd_add(vpaddlq_u32_(b), a)
    }
}

/// 8-bit integer matrix multiply-accumulate
#[inline]
#[cfg_attr(not(bootstrap), target_feature(enable = "i8mm"))]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v8"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(nop))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(smmla))]
pub unsafe fn vmmlaq_s32(a: int32x4_t, b: int8x16_t, c: int8x16_t) -> int32x4_t {
    #[allow(improper_ctypes)]
    extern "unadjusted" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.smmla.v4i32.v16i8")]
        #[cfg_attr(
            target_arch = "aarch64",
            link_name = "llvm.aarch64.neon.smmla.v4i32.v16i8"
        )]
        fn vmmlaq_s32_(a: int32x4_t, b: int8x16_t, c: int8x16_t) -> int32x4_t;
    }
    vmmlaq_s32_(a, b, c)
}

/// 8-bit integer matrix multiply-accumulate
#[inline]
#[cfg_attr(not(bootstrap), target_feature(enable = "i8mm"))]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v8"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(nop))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(ummla))]
pub unsafe fn vmmlaq_u32(a: uint32x4_t, b: uint8x16_t, c: uint8x16_t) -> uint32x4_t {
    #[allow(improper_ctypes)]
    extern "unadjusted" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.ummla.v4i32.v16i8")]
        #[cfg_attr(
            target_arch = "aarch64",
            link_name = "llvm.aarch64.neon.ummla.v4i32.v16i8"
        )]
        fn vmmlaq_u32_(a: uint32x4_t, b: uint8x16_t, c: uint8x16_t) -> uint32x4_t;
    }
    vmmlaq_u32_(a, b, c)
}

/// Unsigned and signed 8-bit integer matrix multiply-accumulate
#[inline]
#[cfg_attr(not(bootstrap), target_feature(enable = "i8mm"))]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v8"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(nop))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(usmmla))]
pub unsafe fn vusmmlaq_s32(a: int32x4_t, b: uint8x16_t, c: int8x16_t) -> int32x4_t {
    #[allow(improper_ctypes)]
    extern "unadjusted" {
        #[cfg_attr(target_arch = "arm", link_name = "llvm.arm.neon.usmmla.v4i32.v16i8")]
        #[cfg_attr(
            target_arch = "aarch64",
            link_name = "llvm.aarch64.neon.usmmla.v4i32.v16i8"
        )]
        fn vusmmlaq_s32_(a: int32x4_t, b: uint8x16_t, c: int8x16_t) -> int32x4_t;
    }
    vusmmlaq_s32_(a, b, c)
}

/* FIXME: 16-bit float
/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
pub unsafe fn vcombine_f16 ( low: float16x4_t,  high: float16x4_t) -> float16x8_t {
    simd_shuffle!(low, high, [0, 1, 2, 3, 4, 5, 6, 7])
}
*/

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcombine_f32(low: float32x2_t, high: float32x2_t) -> float32x4_t {
    simd_shuffle!(low, high, [0, 1, 2, 3])
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcombine_p8(low: poly8x8_t, high: poly8x8_t) -> poly8x16_t {
    simd_shuffle!(
        low,
        high,
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    )
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[stable(feature = "neon_intrinsics", since = "1.59.0")]
pub unsafe fn vcombine_p16(low: poly16x4_t, high: poly16x4_t) -> poly16x8_t {
    simd_shuffle!(low, high, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    target_arch = "aarch64",
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcombine_s8(low: int8x8_t, high: int8x8_t) -> int8x16_t {
    simd_shuffle!(
        low,
        high,
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    )
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    target_arch = "aarch64",
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcombine_s16(low: int16x4_t, high: int16x4_t) -> int16x8_t {
    simd_shuffle!(low, high, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    target_arch = "aarch64",
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcombine_s32(low: int32x2_t, high: int32x2_t) -> int32x4_t {
    simd_shuffle!(low, high, [0, 1, 2, 3])
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    target_arch = "aarch64",
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcombine_s64(low: int64x1_t, high: int64x1_t) -> int64x2_t {
    simd_shuffle!(low, high, [0, 1])
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    target_arch = "aarch64",
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcombine_u8(low: uint8x8_t, high: uint8x8_t) -> uint8x16_t {
    simd_shuffle!(
        low,
        high,
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    )
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    target_arch = "aarch64",
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcombine_u16(low: uint16x4_t, high: uint16x4_t) -> uint16x8_t {
    simd_shuffle!(low, high, [0, 1, 2, 3, 4, 5, 6, 7])
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(all(test, target_arch = "arm"), assert_instr(nop))]
#[cfg_attr(all(test, target_arch = "aarch64"), assert_instr(mov))]
#[cfg_attr(
    target_arch = "aarch64",
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcombine_u32(low: uint32x2_t, high: uint32x2_t) -> uint32x4_t {
    simd_shuffle!(low, high, [0, 1, 2, 3])
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    target_arch = "aarch64",
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcombine_u64(low: uint64x1_t, high: uint64x1_t) -> uint64x2_t {
    simd_shuffle!(low, high, [0, 1])
}

/// Vector combine
#[inline]
#[target_feature(enable = "neon")]
#[cfg_attr(target_arch = "arm", target_feature(enable = "v7"))]
#[cfg_attr(test, assert_instr(nop))]
#[cfg_attr(
    target_arch = "aarch64",
    stable(feature = "neon_intrinsics", since = "1.59.0")
)]
pub unsafe fn vcombine_p64(low: poly64x1_t, high: poly64x1_t) -> poly64x2_t {
    simd_shuffle!(low, high, [0, 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_arch = "aarch64")]
    use crate::core_arch::aarch64::*;
    #[cfg(target_arch = "arm")]
    use crate::core_arch::arm::*;
    use crate::core_arch::arm_shared::test_support::*;
    use crate::core_arch::simd::*;
    use std::{i16, i32, i8, mem::transmute, u16, u32, u8, vec::Vec};
    use stdarch_test::simd_test;

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_s8() {
        let a = i8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let elem: i8 = 42;
        let e = i8x8::new(0, 1, 2, 3, 4, 5, 6, 42);
        let r: i8x8 = transmute(vld1_lane_s8::<7>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_s8() {
        let a = i8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let elem: i8 = 42;
        let e = i8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 42);
        let r: i8x16 = transmute(vld1q_lane_s8::<15>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_s16() {
        let a = i16x4::new(0, 1, 2, 3);
        let elem: i16 = 42;
        let e = i16x4::new(0, 1, 2, 42);
        let r: i16x4 = transmute(vld1_lane_s16::<3>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_s16() {
        let a = i16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let elem: i16 = 42;
        let e = i16x8::new(0, 1, 2, 3, 4, 5, 6, 42);
        let r: i16x8 = transmute(vld1q_lane_s16::<7>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_s32() {
        let a = i32x2::new(0, 1);
        let elem: i32 = 42;
        let e = i32x2::new(0, 42);
        let r: i32x2 = transmute(vld1_lane_s32::<1>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_s32() {
        let a = i32x4::new(0, 1, 2, 3);
        let elem: i32 = 42;
        let e = i32x4::new(0, 1, 2, 42);
        let r: i32x4 = transmute(vld1q_lane_s32::<3>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_s64() {
        let a = i64x1::new(0);
        let elem: i64 = 42;
        let e = i64x1::new(42);
        let r: i64x1 = transmute(vld1_lane_s64::<0>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_s64() {
        let a = i64x2::new(0, 1);
        let elem: i64 = 42;
        let e = i64x2::new(0, 42);
        let r: i64x2 = transmute(vld1q_lane_s64::<1>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_u8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let elem: u8 = 42;
        let e = u8x8::new(0, 1, 2, 3, 4, 5, 6, 42);
        let r: u8x8 = transmute(vld1_lane_u8::<7>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let elem: u8 = 42;
        let e = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 42);
        let r: u8x16 = transmute(vld1q_lane_u8::<15>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_u16() {
        let a = u16x4::new(0, 1, 2, 3);
        let elem: u16 = 42;
        let e = u16x4::new(0, 1, 2, 42);
        let r: u16x4 = transmute(vld1_lane_u16::<3>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let elem: u16 = 42;
        let e = u16x8::new(0, 1, 2, 3, 4, 5, 6, 42);
        let r: u16x8 = transmute(vld1q_lane_u16::<7>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_u32() {
        let a = u32x2::new(0, 1);
        let elem: u32 = 42;
        let e = u32x2::new(0, 42);
        let r: u32x2 = transmute(vld1_lane_u32::<1>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_u32() {
        let a = u32x4::new(0, 1, 2, 3);
        let elem: u32 = 42;
        let e = u32x4::new(0, 1, 2, 42);
        let r: u32x4 = transmute(vld1q_lane_u32::<3>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_u64() {
        let a = u64x1::new(0);
        let elem: u64 = 42;
        let e = u64x1::new(42);
        let r: u64x1 = transmute(vld1_lane_u64::<0>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_u64() {
        let a = u64x2::new(0, 1);
        let elem: u64 = 42;
        let e = u64x2::new(0, 42);
        let r: u64x2 = transmute(vld1q_lane_u64::<1>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_p8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let elem: p8 = 42;
        let e = u8x8::new(0, 1, 2, 3, 4, 5, 6, 42);
        let r: u8x8 = transmute(vld1_lane_p8::<7>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_p8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let elem: p8 = 42;
        let e = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 42);
        let r: u8x16 = transmute(vld1q_lane_p8::<15>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_p16() {
        let a = u16x4::new(0, 1, 2, 3);
        let elem: p16 = 42;
        let e = u16x4::new(0, 1, 2, 42);
        let r: u16x4 = transmute(vld1_lane_p16::<3>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_p16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let elem: p16 = 42;
        let e = u16x8::new(0, 1, 2, 3, 4, 5, 6, 42);
        let r: u16x8 = transmute(vld1q_lane_p16::<7>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon,aes")]
    unsafe fn test_vld1_lane_p64() {
        let a = u64x1::new(0);
        let elem: u64 = 42;
        let e = u64x1::new(42);
        let r: u64x1 = transmute(vld1_lane_p64::<0>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon,aes")]
    unsafe fn test_vld1q_lane_p64() {
        let a = u64x2::new(0, 1);
        let elem: u64 = 42;
        let e = u64x2::new(0, 42);
        let r: u64x2 = transmute(vld1q_lane_p64::<1>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_lane_f32() {
        let a = f32x2::new(0., 1.);
        let elem: f32 = 42.;
        let e = f32x2::new(0., 42.);
        let r: f32x2 = transmute(vld1_lane_f32::<1>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_lane_f32() {
        let a = f32x4::new(0., 1., 2., 3.);
        let elem: f32 = 42.;
        let e = f32x4::new(0., 1., 2., 42.);
        let r: f32x4 = transmute(vld1q_lane_f32::<3>(&elem, transmute(a)));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_s8() {
        let elem: i8 = 42;
        let e = i8x8::new(42, 42, 42, 42, 42, 42, 42, 42);
        let r: i8x8 = transmute(vld1_dup_s8(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_s8() {
        let elem: i8 = 42;
        let e = i8x16::new(
            42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42,
        );
        let r: i8x16 = transmute(vld1q_dup_s8(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_s16() {
        let elem: i16 = 42;
        let e = i16x4::new(42, 42, 42, 42);
        let r: i16x4 = transmute(vld1_dup_s16(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_s16() {
        let elem: i16 = 42;
        let e = i16x8::new(42, 42, 42, 42, 42, 42, 42, 42);
        let r: i16x8 = transmute(vld1q_dup_s16(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_s32() {
        let elem: i32 = 42;
        let e = i32x2::new(42, 42);
        let r: i32x2 = transmute(vld1_dup_s32(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_s32() {
        let elem: i32 = 42;
        let e = i32x4::new(42, 42, 42, 42);
        let r: i32x4 = transmute(vld1q_dup_s32(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_s64() {
        let elem: i64 = 42;
        let e = i64x1::new(42);
        let r: i64x1 = transmute(vld1_dup_s64(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_s64() {
        let elem: i64 = 42;
        let e = i64x2::new(42, 42);
        let r: i64x2 = transmute(vld1q_dup_s64(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_u8() {
        let elem: u8 = 42;
        let e = u8x8::new(42, 42, 42, 42, 42, 42, 42, 42);
        let r: u8x8 = transmute(vld1_dup_u8(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_u8() {
        let elem: u8 = 42;
        let e = u8x16::new(
            42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42,
        );
        let r: u8x16 = transmute(vld1q_dup_u8(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_u16() {
        let elem: u16 = 42;
        let e = u16x4::new(42, 42, 42, 42);
        let r: u16x4 = transmute(vld1_dup_u16(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_u16() {
        let elem: u16 = 42;
        let e = u16x8::new(42, 42, 42, 42, 42, 42, 42, 42);
        let r: u16x8 = transmute(vld1q_dup_u16(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_u32() {
        let elem: u32 = 42;
        let e = u32x2::new(42, 42);
        let r: u32x2 = transmute(vld1_dup_u32(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_u32() {
        let elem: u32 = 42;
        let e = u32x4::new(42, 42, 42, 42);
        let r: u32x4 = transmute(vld1q_dup_u32(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_u64() {
        let elem: u64 = 42;
        let e = u64x1::new(42);
        let r: u64x1 = transmute(vld1_dup_u64(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_u64() {
        let elem: u64 = 42;
        let e = u64x2::new(42, 42);
        let r: u64x2 = transmute(vld1q_dup_u64(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_p8() {
        let elem: p8 = 42;
        let e = u8x8::new(42, 42, 42, 42, 42, 42, 42, 42);
        let r: u8x8 = transmute(vld1_dup_p8(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_p8() {
        let elem: p8 = 42;
        let e = u8x16::new(
            42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42,
        );
        let r: u8x16 = transmute(vld1q_dup_p8(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_p16() {
        let elem: p16 = 42;
        let e = u16x4::new(42, 42, 42, 42);
        let r: u16x4 = transmute(vld1_dup_p16(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_p16() {
        let elem: p16 = 42;
        let e = u16x8::new(42, 42, 42, 42, 42, 42, 42, 42);
        let r: u16x8 = transmute(vld1q_dup_p16(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon,aes")]
    unsafe fn test_vld1_dup_p64() {
        let elem: u64 = 42;
        let e = u64x1::new(42);
        let r: u64x1 = transmute(vld1_dup_p64(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon,aes")]
    unsafe fn test_vld1q_dup_p64() {
        let elem: u64 = 42;
        let e = u64x2::new(42, 42);
        let r: u64x2 = transmute(vld1q_dup_p64(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1_dup_f32() {
        let elem: f32 = 42.;
        let e = f32x2::new(42., 42.);
        let r: f32x2 = transmute(vld1_dup_f32(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vld1q_dup_f32() {
        let elem: f32 = 42.;
        let e = f32x4::new(42., 42., 42., 42.);
        let r: f32x4 = transmute(vld1q_dup_f32(&elem));
        assert_eq!(r, e)
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_u8() {
        let v = i8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r = vget_lane_u8::<1>(transmute(v));
        assert_eq!(r, 2);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_u32() {
        let v = i32x4::new(1, 2, 3, 4);
        let r = vgetq_lane_u32::<1>(transmute(v));
        assert_eq!(r, 2);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_s32() {
        let v = i32x4::new(1, 2, 3, 4);
        let r = vgetq_lane_s32::<1>(transmute(v));
        assert_eq!(r, 2);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_u64() {
        let v: u64 = 1;
        let r = vget_lane_u64::<0>(transmute(v));
        assert_eq!(r, 1);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_u16() {
        let v = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r = vgetq_lane_u16::<1>(transmute(v));
        assert_eq!(r, 2);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_s8() {
        let v = i8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = vget_lane_s8::<2>(transmute(v));
        assert_eq!(r, 2);
        let r = vget_lane_s8::<4>(transmute(v));
        assert_eq!(r, 4);
        let r = vget_lane_s8::<5>(transmute(v));
        assert_eq!(r, 5);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_p8() {
        let v = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = vget_lane_p8::<2>(transmute(v));
        assert_eq!(r, 2);
        let r = vget_lane_p8::<3>(transmute(v));
        assert_eq!(r, 3);
        let r = vget_lane_p8::<5>(transmute(v));
        assert_eq!(r, 5);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_p16() {
        let v = u16x4::new(0, 1, 2, 3);
        let r = vget_lane_p16::<2>(transmute(v));
        assert_eq!(r, 2);
        let r = vget_lane_p16::<3>(transmute(v));
        assert_eq!(r, 3);
        let r = vget_lane_p16::<0>(transmute(v));
        assert_eq!(r, 0);
        let r = vget_lane_p16::<1>(transmute(v));
        assert_eq!(r, 1);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_s16() {
        let v = i16x4::new(0, 1, 2, 3);
        let r = vget_lane_s16::<2>(transmute(v));
        assert_eq!(r, 2);
        let r = vget_lane_s16::<3>(transmute(v));
        assert_eq!(r, 3);
        let r = vget_lane_s16::<0>(transmute(v));
        assert_eq!(r, 0);
        let r = vget_lane_s16::<1>(transmute(v));
        assert_eq!(r, 1);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_u16() {
        let v = u16x4::new(0, 1, 2, 3);
        let r = vget_lane_u16::<2>(transmute(v));
        assert_eq!(r, 2);
        let r = vget_lane_u16::<3>(transmute(v));
        assert_eq!(r, 3);
        let r = vget_lane_u16::<0>(transmute(v));
        assert_eq!(r, 0);
        let r = vget_lane_u16::<1>(transmute(v));
        assert_eq!(r, 1);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_f32() {
        let v = f32x2::new(0.0, 1.0);
        let r = vget_lane_f32::<1>(transmute(v));
        assert_eq!(r, 1.0);
        let r = vget_lane_f32::<0>(transmute(v));
        assert_eq!(r, 0.0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_s32() {
        let v = i32x2::new(0, 1);
        let r = vget_lane_s32::<1>(transmute(v));
        assert_eq!(r, 1);
        let r = vget_lane_s32::<0>(transmute(v));
        assert_eq!(r, 0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_u32() {
        let v = u32x2::new(0, 1);
        let r = vget_lane_u32::<1>(transmute(v));
        assert_eq!(r, 1);
        let r = vget_lane_u32::<0>(transmute(v));
        assert_eq!(r, 0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_s64() {
        let v = i64x1::new(1);
        let r = vget_lane_s64::<0>(transmute(v));
        assert_eq!(r, 1);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_lane_p64() {
        let v = u64x1::new(1);
        let r = vget_lane_p64::<0>(transmute(v));
        assert_eq!(r, 1);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_s8() {
        let v = i8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = vgetq_lane_s8::<7>(transmute(v));
        assert_eq!(r, 7);
        let r = vgetq_lane_s8::<13>(transmute(v));
        assert_eq!(r, 13);
        let r = vgetq_lane_s8::<3>(transmute(v));
        assert_eq!(r, 3);
        let r = vgetq_lane_s8::<0>(transmute(v));
        assert_eq!(r, 0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_p8() {
        let v = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = vgetq_lane_p8::<7>(transmute(v));
        assert_eq!(r, 7);
        let r = vgetq_lane_p8::<13>(transmute(v));
        assert_eq!(r, 13);
        let r = vgetq_lane_p8::<3>(transmute(v));
        assert_eq!(r, 3);
        let r = vgetq_lane_p8::<0>(transmute(v));
        assert_eq!(r, 0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_u8() {
        let v = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = vgetq_lane_u8::<7>(transmute(v));
        assert_eq!(r, 7);
        let r = vgetq_lane_u8::<13>(transmute(v));
        assert_eq!(r, 13);
        let r = vgetq_lane_u8::<3>(transmute(v));
        assert_eq!(r, 3);
        let r = vgetq_lane_u8::<0>(transmute(v));
        assert_eq!(r, 0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_s16() {
        let v = i16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = vgetq_lane_s16::<3>(transmute(v));
        assert_eq!(r, 3);
        let r = vgetq_lane_s16::<6>(transmute(v));
        assert_eq!(r, 6);
        let r = vgetq_lane_s16::<0>(transmute(v));
        assert_eq!(r, 0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_p16() {
        let v = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = vgetq_lane_p16::<3>(transmute(v));
        assert_eq!(r, 3);
        let r = vgetq_lane_p16::<7>(transmute(v));
        assert_eq!(r, 7);
        let r = vgetq_lane_p16::<1>(transmute(v));
        assert_eq!(r, 1);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_f32() {
        let v = f32x4::new(0.0, 1.0, 2.0, 3.0);
        let r = vgetq_lane_f32::<3>(transmute(v));
        assert_eq!(r, 3.0);
        let r = vgetq_lane_f32::<0>(transmute(v));
        assert_eq!(r, 0.0);
        let r = vgetq_lane_f32::<2>(transmute(v));
        assert_eq!(r, 2.0);
        let r = vgetq_lane_f32::<1>(transmute(v));
        assert_eq!(r, 1.0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_s64() {
        let v = i64x2::new(0, 1);
        let r = vgetq_lane_s64::<1>(transmute(v));
        assert_eq!(r, 1);
        let r = vgetq_lane_s64::<0>(transmute(v));
        assert_eq!(r, 0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_p64() {
        let v = u64x2::new(0, 1);
        let r = vgetq_lane_p64::<1>(transmute(v));
        assert_eq!(r, 1);
        let r = vgetq_lane_p64::<0>(transmute(v));
        assert_eq!(r, 0);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vext_s64() {
        let a: i64x1 = i64x1::new(0);
        let b: i64x1 = i64x1::new(1);
        let e: i64x1 = i64x1::new(0);
        let r: i64x1 = transmute(vext_s64::<0>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vext_u64() {
        let a: u64x1 = u64x1::new(0);
        let b: u64x1 = u64x1::new(1);
        let e: u64x1 = u64x1::new(0);
        let r: u64x1 = transmute(vext_u64::<0>(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_s8() {
        let a = i8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let e = i8x8::new(9, 10, 11, 12, 13, 14, 15, 16);
        let r: i8x8 = transmute(vget_high_s8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_s16() {
        let a = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let e = i16x4::new(5, 6, 7, 8);
        let r: i16x4 = transmute(vget_high_s16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_s32() {
        let a = i32x4::new(1, 2, 3, 4);
        let e = i32x2::new(3, 4);
        let r: i32x2 = transmute(vget_high_s32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_s64() {
        let a = i64x2::new(1, 2);
        let e = i64x1::new(2);
        let r: i64x1 = transmute(vget_high_s64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_u8() {
        let a = u8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let e = u8x8::new(9, 10, 11, 12, 13, 14, 15, 16);
        let r: u8x8 = transmute(vget_high_u8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_u16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let e = u16x4::new(5, 6, 7, 8);
        let r: u16x4 = transmute(vget_high_u16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_u32() {
        let a = u32x4::new(1, 2, 3, 4);
        let e = u32x2::new(3, 4);
        let r: u32x2 = transmute(vget_high_u32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_u64() {
        let a = u64x2::new(1, 2);
        let e = u64x1::new(2);
        let r: u64x1 = transmute(vget_high_u64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_p8() {
        let a = u8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let e = u8x8::new(9, 10, 11, 12, 13, 14, 15, 16);
        let r: u8x8 = transmute(vget_high_p8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_p16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let e = u16x4::new(5, 6, 7, 8);
        let r: u16x4 = transmute(vget_high_p16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_high_f32() {
        let a = f32x4::new(1.0, 2.0, 3.0, 4.0);
        let e = f32x2::new(3.0, 4.0);
        let r: f32x2 = transmute(vget_high_f32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_s8() {
        let a = i8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let e = i8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: i8x8 = transmute(vget_low_s8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_s16() {
        let a = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let e = i16x4::new(1, 2, 3, 4);
        let r: i16x4 = transmute(vget_low_s16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_s32() {
        let a = i32x4::new(1, 2, 3, 4);
        let e = i32x2::new(1, 2);
        let r: i32x2 = transmute(vget_low_s32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_s64() {
        let a = i64x2::new(1, 2);
        let e = i64x1::new(1);
        let r: i64x1 = transmute(vget_low_s64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_u8() {
        let a = u8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let e = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: u8x8 = transmute(vget_low_u8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_u16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let e = u16x4::new(1, 2, 3, 4);
        let r: u16x4 = transmute(vget_low_u16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_u32() {
        let a = u32x4::new(1, 2, 3, 4);
        let e = u32x2::new(1, 2);
        let r: u32x2 = transmute(vget_low_u32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_u64() {
        let a = u64x2::new(1, 2);
        let e = u64x1::new(1);
        let r: u64x1 = transmute(vget_low_u64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_p8() {
        let a = u8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let e = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: u8x8 = transmute(vget_low_p8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_p16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let e = u16x4::new(1, 2, 3, 4);
        let r: u16x4 = transmute(vget_low_p16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vget_low_f32() {
        let a = f32x4::new(1.0, 2.0, 3.0, 4.0);
        let e = f32x2::new(1.0, 2.0);
        let r: f32x2 = transmute(vget_low_f32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_s8() {
        let v: i8 = 42;
        let e = i8x16::new(
            42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42,
        );
        let r: i8x16 = transmute(vdupq_n_s8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_s16() {
        let v: i16 = 64;
        let e = i16x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: i16x8 = transmute(vdupq_n_s16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_s32() {
        let v: i32 = 64;
        let e = i32x4::new(64, 64, 64, 64);
        let r: i32x4 = transmute(vdupq_n_s32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_s64() {
        let v: i64 = 64;
        let e = i64x2::new(64, 64);
        let r: i64x2 = transmute(vdupq_n_s64(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_u8() {
        let v: u8 = 64;
        let e = u8x16::new(
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        );
        let r: u8x16 = transmute(vdupq_n_u8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_u16() {
        let v: u16 = 64;
        let e = u16x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: u16x8 = transmute(vdupq_n_u16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_u32() {
        let v: u32 = 64;
        let e = u32x4::new(64, 64, 64, 64);
        let r: u32x4 = transmute(vdupq_n_u32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_u64() {
        let v: u64 = 64;
        let e = u64x2::new(64, 64);
        let r: u64x2 = transmute(vdupq_n_u64(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_p8() {
        let v: p8 = 64;
        let e = u8x16::new(
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        );
        let r: u8x16 = transmute(vdupq_n_p8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_p16() {
        let v: p16 = 64;
        let e = u16x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: u16x8 = transmute(vdupq_n_p16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdupq_n_f32() {
        let v: f32 = 64.0;
        let e = f32x4::new(64.0, 64.0, 64.0, 64.0);
        let r: f32x4 = transmute(vdupq_n_f32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_s8() {
        let v: i8 = 64;
        let e = i8x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: i8x8 = transmute(vdup_n_s8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_s16() {
        let v: i16 = 64;
        let e = i16x4::new(64, 64, 64, 64);
        let r: i16x4 = transmute(vdup_n_s16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_s32() {
        let v: i32 = 64;
        let e = i32x2::new(64, 64);
        let r: i32x2 = transmute(vdup_n_s32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_s64() {
        let v: i64 = 64;
        let e = i64x1::new(64);
        let r: i64x1 = transmute(vdup_n_s64(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_u8() {
        let v: u8 = 64;
        let e = u8x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: u8x8 = transmute(vdup_n_u8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_u16() {
        let v: u16 = 64;
        let e = u16x4::new(64, 64, 64, 64);
        let r: u16x4 = transmute(vdup_n_u16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_u32() {
        let v: u32 = 64;
        let e = u32x2::new(64, 64);
        let r: u32x2 = transmute(vdup_n_u32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_u64() {
        let v: u64 = 64;
        let e = u64x1::new(64);
        let r: u64x1 = transmute(vdup_n_u64(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_p8() {
        let v: p8 = 64;
        let e = u8x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: u8x8 = transmute(vdup_n_p8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_p16() {
        let v: p16 = 64;
        let e = u16x4::new(64, 64, 64, 64);
        let r: u16x4 = transmute(vdup_n_p16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vdup_n_f32() {
        let v: f32 = 64.0;
        let e = f32x2::new(64.0, 64.0);
        let r: f32x2 = transmute(vdup_n_f32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vldrq_p128() {
        let v: [p128; 2] = [1, 2];
        let e: p128 = 2;
        let r: p128 = vldrq_p128(v[1..].as_ptr());
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vstrq_p128() {
        let v: [p128; 2] = [1, 2];
        let e: p128 = 2;
        let mut r: p128 = 1;
        vstrq_p128(&mut r, v[1]);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_s8() {
        let v: i8 = 64;
        let e = i8x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: i8x8 = transmute(vmov_n_s8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_s16() {
        let v: i16 = 64;
        let e = i16x4::new(64, 64, 64, 64);
        let r: i16x4 = transmute(vmov_n_s16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_s32() {
        let v: i32 = 64;
        let e = i32x2::new(64, 64);
        let r: i32x2 = transmute(vmov_n_s32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_s64() {
        let v: i64 = 64;
        let e = i64x1::new(64);
        let r: i64x1 = transmute(vmov_n_s64(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_u8() {
        let v: u8 = 64;
        let e = u8x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: u8x8 = transmute(vmov_n_u8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_u16() {
        let v: u16 = 64;
        let e = u16x4::new(64, 64, 64, 64);
        let r: u16x4 = transmute(vmov_n_u16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_u32() {
        let v: u32 = 64;
        let e = u32x2::new(64, 64);
        let r: u32x2 = transmute(vmov_n_u32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_u64() {
        let v: u64 = 64;
        let e = u64x1::new(64);
        let r: u64x1 = transmute(vmov_n_u64(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_p8() {
        let v: p8 = 64;
        let e = u8x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: u8x8 = transmute(vmov_n_p8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_p16() {
        let v: p16 = 64;
        let e = u16x4::new(64, 64, 64, 64);
        let r: u16x4 = transmute(vmov_n_p16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmov_n_f32() {
        let v: f32 = 64.0;
        let e = f32x2::new(64.0, 64.0);
        let r: f32x2 = transmute(vmov_n_f32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_s8() {
        let v: i8 = 64;
        let e = i8x16::new(
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        );
        let r: i8x16 = transmute(vmovq_n_s8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_s16() {
        let v: i16 = 64;
        let e = i16x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: i16x8 = transmute(vmovq_n_s16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_s32() {
        let v: i32 = 64;
        let e = i32x4::new(64, 64, 64, 64);
        let r: i32x4 = transmute(vmovq_n_s32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_s64() {
        let v: i64 = 64;
        let e = i64x2::new(64, 64);
        let r: i64x2 = transmute(vmovq_n_s64(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_u8() {
        let v: u8 = 64;
        let e = u8x16::new(
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        );
        let r: u8x16 = transmute(vmovq_n_u8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_u16() {
        let v: u16 = 64;
        let e = u16x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: u16x8 = transmute(vmovq_n_u16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_u32() {
        let v: u32 = 64;
        let e = u32x4::new(64, 64, 64, 64);
        let r: u32x4 = transmute(vmovq_n_u32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_u64() {
        let v: u64 = 64;
        let e = u64x2::new(64, 64);
        let r: u64x2 = transmute(vmovq_n_u64(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_p8() {
        let v: p8 = 64;
        let e = u8x16::new(
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        );
        let r: u8x16 = transmute(vmovq_n_p8(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_p16() {
        let v: p16 = 64;
        let e = u16x8::new(64, 64, 64, 64, 64, 64, 64, 64);
        let r: u16x8 = transmute(vmovq_n_p16(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovq_n_f32() {
        let v: f32 = 64.0;
        let e = f32x4::new(64.0, 64.0, 64.0, 64.0);
        let r: f32x4 = transmute(vmovq_n_f32(v));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vgetq_lane_u64() {
        let v = i64x2::new(1, 2);
        let r = vgetq_lane_u64::<1>(transmute(v));
        assert_eq!(r, 2);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_s8() {
        test_ari_s8(
            |i, j| vadd_s8(i, j),
            |a: i8, b: i8| -> i8 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddq_s8() {
        testq_ari_s8(
            |i, j| vaddq_s8(i, j),
            |a: i8, b: i8| -> i8 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_s16() {
        test_ari_s16(
            |i, j| vadd_s16(i, j),
            |a: i16, b: i16| -> i16 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddq_s16() {
        testq_ari_s16(
            |i, j| vaddq_s16(i, j),
            |a: i16, b: i16| -> i16 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_s32() {
        test_ari_s32(
            |i, j| vadd_s32(i, j),
            |a: i32, b: i32| -> i32 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddq_s32() {
        testq_ari_s32(
            |i, j| vaddq_s32(i, j),
            |a: i32, b: i32| -> i32 { a.overflowing_add(b).0 },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_u8() {
        test_ari_u8(
            |i, j| vadd_u8(i, j),
            |a: u8, b: u8| -> u8 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddq_u8() {
        testq_ari_u8(
            |i, j| vaddq_u8(i, j),
            |a: u8, b: u8| -> u8 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_u16() {
        test_ari_u16(
            |i, j| vadd_u16(i, j),
            |a: u16, b: u16| -> u16 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddq_u16() {
        testq_ari_u16(
            |i, j| vaddq_u16(i, j),
            |a: u16, b: u16| -> u16 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_u32() {
        test_ari_u32(
            |i, j| vadd_u32(i, j),
            |a: u32, b: u32| -> u32 { a.overflowing_add(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddq_u32() {
        testq_ari_u32(
            |i, j| vaddq_u32(i, j),
            |a: u32, b: u32| -> u32 { a.overflowing_add(b).0 },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vadd_f32() {
        test_ari_f32(|i, j| vadd_f32(i, j), |a: f32, b: f32| -> f32 { a + b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaddq_f32() {
        testq_ari_f32(|i, j| vaddq_f32(i, j), |a: f32, b: f32| -> f32 { a + b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_s8() {
        let v = i8::MAX;
        let a = i8x8::new(v, v, v, v, v, v, v, v);
        let v = 2 * (v as i16);
        let e = i16x8::new(v, v, v, v, v, v, v, v);
        let r: i16x8 = transmute(vaddl_s8(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_s16() {
        let v = i16::MAX;
        let a = i16x4::new(v, v, v, v);
        let v = 2 * (v as i32);
        let e = i32x4::new(v, v, v, v);
        let r: i32x4 = transmute(vaddl_s16(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_s32() {
        let v = i32::MAX;
        let a = i32x2::new(v, v);
        let v = 2 * (v as i64);
        let e = i64x2::new(v, v);
        let r: i64x2 = transmute(vaddl_s32(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_u8() {
        let v = u8::MAX;
        let a = u8x8::new(v, v, v, v, v, v, v, v);
        let v = 2 * (v as u16);
        let e = u16x8::new(v, v, v, v, v, v, v, v);
        let r: u16x8 = transmute(vaddl_u8(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_u16() {
        let v = u16::MAX;
        let a = u16x4::new(v, v, v, v);
        let v = 2 * (v as u32);
        let e = u32x4::new(v, v, v, v);
        let r: u32x4 = transmute(vaddl_u16(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_u32() {
        let v = u32::MAX;
        let a = u32x2::new(v, v);
        let v = 2 * (v as u64);
        let e = u64x2::new(v, v);
        let r: u64x2 = transmute(vaddl_u32(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_high_s8() {
        let a = i8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let x = i8::MAX;
        let b = i8x16::new(x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x);
        let x = x as i16;
        let e = i16x8::new(x + 8, x + 9, x + 10, x + 11, x + 12, x + 13, x + 14, x + 15);
        let r: i16x8 = transmute(vaddl_high_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_high_s16() {
        let a = i16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let x = i16::MAX;
        let b = i16x8::new(x, x, x, x, x, x, x, x);
        let x = x as i32;
        let e = i32x4::new(x + 4, x + 5, x + 6, x + 7);
        let r: i32x4 = transmute(vaddl_high_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_high_s32() {
        let a = i32x4::new(0, 1, 2, 3);
        let x = i32::MAX;
        let b = i32x4::new(x, x, x, x);
        let x = x as i64;
        let e = i64x2::new(x + 2, x + 3);
        let r: i64x2 = transmute(vaddl_high_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_high_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let x = u8::MAX;
        let b = u8x16::new(x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x);
        let x = x as u16;
        let e = u16x8::new(x + 8, x + 9, x + 10, x + 11, x + 12, x + 13, x + 14, x + 15);
        let r: u16x8 = transmute(vaddl_high_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_high_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let x = u16::MAX;
        let b = u16x8::new(x, x, x, x, x, x, x, x);
        let x = x as u32;
        let e = u32x4::new(x + 4, x + 5, x + 6, x + 7);
        let r: u32x4 = transmute(vaddl_high_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddl_high_u32() {
        let a = u32x4::new(0, 1, 2, 3);
        let x = u32::MAX;
        let b = u32x4::new(x, x, x, x);
        let x = x as u64;
        let e = u64x2::new(x + 2, x + 3);
        let r: u64x2 = transmute(vaddl_high_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_s8() {
        let x = i16::MAX;
        let a = i16x8::new(x, 1, 2, 3, 4, 5, 6, 7);
        let y = i8::MAX;
        let b = i8x8::new(y, y, y, y, y, y, y, y);
        let y = y as i16;
        let e = i16x8::new(
            x.wrapping_add(y),
            1 + y,
            2 + y,
            3 + y,
            4 + y,
            5 + y,
            6 + y,
            7 + y,
        );
        let r: i16x8 = transmute(vaddw_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_s16() {
        let x = i32::MAX;
        let a = i32x4::new(x, 1, 2, 3);
        let y = i16::MAX;
        let b = i16x4::new(y, y, y, y);
        let y = y as i32;
        let e = i32x4::new(x.wrapping_add(y), 1 + y, 2 + y, 3 + y);
        let r: i32x4 = transmute(vaddw_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_s32() {
        let x = i64::MAX;
        let a = i64x2::new(x, 1);
        let y = i32::MAX;
        let b = i32x2::new(y, y);
        let y = y as i64;
        let e = i64x2::new(x.wrapping_add(y), 1 + y);
        let r: i64x2 = transmute(vaddw_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_u8() {
        let x = u16::MAX;
        let a = u16x8::new(x, 1, 2, 3, 4, 5, 6, 7);
        let y = u8::MAX;
        let b = u8x8::new(y, y, y, y, y, y, y, y);
        let y = y as u16;
        let e = u16x8::new(
            x.wrapping_add(y),
            1 + y,
            2 + y,
            3 + y,
            4 + y,
            5 + y,
            6 + y,
            7 + y,
        );
        let r: u16x8 = transmute(vaddw_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_u16() {
        let x = u32::MAX;
        let a = u32x4::new(x, 1, 2, 3);
        let y = u16::MAX;
        let b = u16x4::new(y, y, y, y);
        let y = y as u32;
        let e = u32x4::new(x.wrapping_add(y), 1 + y, 2 + y, 3 + y);
        let r: u32x4 = transmute(vaddw_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_u32() {
        let x = u64::MAX;
        let a = u64x2::new(x, 1);
        let y = u32::MAX;
        let b = u32x2::new(y, y);
        let y = y as u64;
        let e = u64x2::new(x.wrapping_add(y), 1 + y);
        let r: u64x2 = transmute(vaddw_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_high_s8() {
        let x = i16::MAX;
        let a = i16x8::new(x, 1, 2, 3, 4, 5, 6, 7);
        let y = i8::MAX;
        let b = i8x16::new(0, 0, 0, 0, 0, 0, 0, 0, y, y, y, y, y, y, y, y);
        let y = y as i16;
        let e = i16x8::new(
            x.wrapping_add(y),
            1 + y,
            2 + y,
            3 + y,
            4 + y,
            5 + y,
            6 + y,
            7 + y,
        );
        let r: i16x8 = transmute(vaddw_high_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_high_s16() {
        let x = i32::MAX;
        let a = i32x4::new(x, 1, 2, 3);
        let y = i16::MAX;
        let b = i16x8::new(0, 0, 0, 0, y, y, y, y);
        let y = y as i32;
        let e = i32x4::new(x.wrapping_add(y), 1 + y, 2 + y, 3 + y);
        let r: i32x4 = transmute(vaddw_high_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_high_s32() {
        let x = i64::MAX;
        let a = i64x2::new(x, 1);
        let y = i32::MAX;
        let b = i32x4::new(0, 0, y, y);
        let y = y as i64;
        let e = i64x2::new(x.wrapping_add(y), 1 + y);
        let r: i64x2 = transmute(vaddw_high_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_high_u8() {
        let x = u16::MAX;
        let a = u16x8::new(x, 1, 2, 3, 4, 5, 6, 7);
        let y = u8::MAX;
        let b = u8x16::new(0, 0, 0, 0, 0, 0, 0, 0, y, y, y, y, y, y, y, y);
        let y = y as u16;
        let e = u16x8::new(
            x.wrapping_add(y),
            1 + y,
            2 + y,
            3 + y,
            4 + y,
            5 + y,
            6 + y,
            7 + y,
        );
        let r: u16x8 = transmute(vaddw_high_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_high_u16() {
        let x = u32::MAX;
        let a = u32x4::new(x, 1, 2, 3);
        let y = u16::MAX;
        let b = u16x8::new(0, 0, 0, 0, y, y, y, y);
        let y = y as u32;
        let e = u32x4::new(x.wrapping_add(y), 1 + y, 2 + y, 3 + y);
        let r: u32x4 = transmute(vaddw_high_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddw_high_u32() {
        let x = u64::MAX;
        let a = u64x2::new(x, 1);
        let y = u32::MAX;
        let b = u32x4::new(0, 0, y, y);
        let y = y as u64;
        let e = u64x2::new(x.wrapping_add(y), 1 + y);
        let r: u64x2 = transmute(vaddw_high_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_s16() {
        let a = i16x8::new(
            (0 << 8) + 1,
            (1 << 8) + 1,
            (2 << 8) + 1,
            (3 << 8) + 1,
            (4 << 8) + 1,
            (5 << 8) + 1,
            (6 << 8) + 1,
            (7 << 8) + 1,
        );
        let e = i8x8::new(0, 2, 4, 6, 8, 10, 12, 14);
        let r: i8x8 = transmute(vaddhn_s16(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_s32() {
        let a = i32x4::new((0 << 16) + 1, (1 << 16) + 1, (2 << 16) + 1, (3 << 16) + 1);
        let e = i16x4::new(0, 2, 4, 6);
        let r: i16x4 = transmute(vaddhn_s32(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_s64() {
        let a = i64x2::new((0 << 32) + 1, (1 << 32) + 1);
        let e = i32x2::new(0, 2);
        let r: i32x2 = transmute(vaddhn_s64(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_u16() {
        let a = u16x8::new(
            (0 << 8) + 1,
            (1 << 8) + 1,
            (2 << 8) + 1,
            (3 << 8) + 1,
            (4 << 8) + 1,
            (5 << 8) + 1,
            (6 << 8) + 1,
            (7 << 8) + 1,
        );
        let e = u8x8::new(0, 2, 4, 6, 8, 10, 12, 14);
        let r: u8x8 = transmute(vaddhn_u16(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_u32() {
        let a = u32x4::new((0 << 16) + 1, (1 << 16) + 1, (2 << 16) + 1, (3 << 16) + 1);
        let e = u16x4::new(0, 2, 4, 6);
        let r: u16x4 = transmute(vaddhn_u32(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_u64() {
        let a = u64x2::new((0 << 32) + 1, (1 << 32) + 1);
        let e = u32x2::new(0, 2);
        let r: u32x2 = transmute(vaddhn_u64(transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_high_s16() {
        let r = i8x8::splat(42);
        let a = i16x8::new(
            (0 << 8) + 1,
            (1 << 8) + 1,
            (2 << 8) + 1,
            (3 << 8) + 1,
            (4 << 8) + 1,
            (5 << 8) + 1,
            (6 << 8) + 1,
            (7 << 8) + 1,
        );
        let e = i8x16::new(42, 42, 42, 42, 42, 42, 42, 42, 0, 2, 4, 6, 8, 10, 12, 14);
        let r: i8x16 = transmute(vaddhn_high_s16(transmute(r), transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_high_s32() {
        let r = i16x4::splat(42);
        let a = i32x4::new((0 << 16) + 1, (1 << 16) + 1, (2 << 16) + 1, (3 << 16) + 1);
        let e = i16x8::new(42, 42, 42, 42, 0, 2, 4, 6);
        let r: i16x8 = transmute(vaddhn_high_s32(transmute(r), transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_high_s64() {
        let r = i32x2::splat(42);
        let a = i64x2::new((0 << 32) + 1, (1 << 32) + 1);
        let e = i32x4::new(42, 42, 0, 2);
        let r: i32x4 = transmute(vaddhn_high_s64(transmute(r), transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_high_u16() {
        let r = u8x8::splat(42);
        let a = u16x8::new(
            (0 << 8) + 1,
            (1 << 8) + 1,
            (2 << 8) + 1,
            (3 << 8) + 1,
            (4 << 8) + 1,
            (5 << 8) + 1,
            (6 << 8) + 1,
            (7 << 8) + 1,
        );
        let e = u8x16::new(42, 42, 42, 42, 42, 42, 42, 42, 0, 2, 4, 6, 8, 10, 12, 14);
        let r: u8x16 = transmute(vaddhn_high_u16(transmute(r), transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_high_u32() {
        let r = u16x4::splat(42);
        let a = u32x4::new((0 << 16) + 1, (1 << 16) + 1, (2 << 16) + 1, (3 << 16) + 1);
        let e = u16x8::new(42, 42, 42, 42, 0, 2, 4, 6);
        let r: u16x8 = transmute(vaddhn_high_u32(transmute(r), transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaddhn_high_u64() {
        let r = u32x2::splat(42);
        let a = u64x2::new((0 << 32) + 1, (1 << 32) + 1);
        let e = u32x4::new(42, 42, 0, 2);
        let r: u32x4 = transmute(vaddhn_high_u64(transmute(r), transmute(a), transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_s16() {
        let round_constant: i16 = (1 << 8) - 1;
        let a = i16x8::new(
            0 << 8,
            1 << 8,
            2 << 8,
            3 << 8,
            4 << 8,
            5 << 8,
            6 << 8,
            7 << 8,
        );
        let b = i16x8::new(
            0 << 8,
            (1 << 8) + round_constant,
            2 << 8,
            (3 << 8) + round_constant,
            4 << 8,
            (5 << 8) + round_constant,
            6 << 8,
            (7 << 8) + round_constant,
        );
        let e = i8x8::new(0, 3, 4, 7, 8, 11, 12, 15);
        let r: i8x8 = transmute(vraddhn_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_s32() {
        let round_constant: i32 = (1 << 16) - 1;
        let a = i32x4::new(0 << 16, 1 << 16, 2 << 16, 3 << 16);
        let b = i32x4::new(
            0 << 16,
            (1 << 16) + round_constant,
            2 << 16,
            (3 << 16) + round_constant,
        );
        let e = i16x4::new(0, 3, 4, 7);
        let r: i16x4 = transmute(vraddhn_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_s64() {
        let round_constant: i64 = (1 << 32) - 1;
        let a = i64x2::new(0 << 32, 1 << 32);
        let b = i64x2::new(0 << 32, (1 << 32) + round_constant);
        let e = i32x2::new(0, 3);
        let r: i32x2 = transmute(vraddhn_s64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_u16() {
        let round_constant: u16 = (1 << 8) - 1;
        let a = u16x8::new(
            0 << 8,
            1 << 8,
            2 << 8,
            3 << 8,
            4 << 8,
            5 << 8,
            6 << 8,
            7 << 8,
        );
        let b = u16x8::new(
            0 << 8,
            (1 << 8) + round_constant,
            2 << 8,
            (3 << 8) + round_constant,
            4 << 8,
            (5 << 8) + round_constant,
            6 << 8,
            (7 << 8) + round_constant,
        );
        let e = u8x8::new(0, 3, 4, 7, 8, 11, 12, 15);
        let r: u8x8 = transmute(vraddhn_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_u32() {
        let round_constant: u32 = (1 << 16) - 1;
        let a = u32x4::new(0 << 16, 1 << 16, 2 << 16, 3 << 16);
        let b = u32x4::new(
            0 << 16,
            (1 << 16) + round_constant,
            2 << 16,
            (3 << 16) + round_constant,
        );
        let e = u16x4::new(0, 3, 4, 7);
        let r: u16x4 = transmute(vraddhn_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_u64() {
        let round_constant: u64 = (1 << 32) - 1;
        let a = u64x2::new(0 << 32, 1 << 32);
        let b = u64x2::new(0 << 32, (1 << 32) + round_constant);
        let e = u32x2::new(0, 3);
        let r: u32x2 = transmute(vraddhn_u64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_high_s16() {
        let r = i8x8::splat(42);
        let round_constant: i16 = (1 << 8) - 1;
        let a = i16x8::new(
            0 << 8,
            1 << 8,
            2 << 8,
            3 << 8,
            4 << 8,
            5 << 8,
            6 << 8,
            7 << 8,
        );
        let b = i16x8::new(
            0 << 8,
            (1 << 8) + round_constant,
            2 << 8,
            (3 << 8) + round_constant,
            4 << 8,
            (5 << 8) + round_constant,
            6 << 8,
            (7 << 8) + round_constant,
        );
        let e = i8x16::new(42, 42, 42, 42, 42, 42, 42, 42, 0, 3, 4, 7, 8, 11, 12, 15);
        let r: i8x16 = transmute(vraddhn_high_s16(transmute(r), transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_high_s32() {
        let r = i16x4::splat(42);
        let round_constant: i32 = (1 << 16) - 1;
        let a = i32x4::new(0 << 16, 1 << 16, 2 << 16, 3 << 16);
        let b = i32x4::new(
            0 << 16,
            (1 << 16) + round_constant,
            2 << 16,
            (3 << 16) + round_constant,
        );
        let e = i16x8::new(42, 42, 42, 42, 0, 3, 4, 7);
        let r: i16x8 = transmute(vraddhn_high_s32(transmute(r), transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_high_s64() {
        let r = i32x2::splat(42);
        let round_constant: i64 = (1 << 32) - 1;
        let a = i64x2::new(0 << 32, 1 << 32);
        let b = i64x2::new(0 << 32, (1 << 32) + round_constant);
        let e = i32x4::new(42, 42, 0, 3);
        let r: i32x4 = transmute(vraddhn_high_s64(transmute(r), transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_high_u16() {
        let r = u8x8::splat(42);
        let round_constant: u16 = (1 << 8) - 1;
        let a = u16x8::new(
            0 << 8,
            1 << 8,
            2 << 8,
            3 << 8,
            4 << 8,
            5 << 8,
            6 << 8,
            7 << 8,
        );
        let b = u16x8::new(
            0 << 8,
            (1 << 8) + round_constant,
            2 << 8,
            (3 << 8) + round_constant,
            4 << 8,
            (5 << 8) + round_constant,
            6 << 8,
            (7 << 8) + round_constant,
        );
        let e = u8x16::new(42, 42, 42, 42, 42, 42, 42, 42, 0, 3, 4, 7, 8, 11, 12, 15);
        let r: u8x16 = transmute(vraddhn_high_u16(transmute(r), transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_high_u32() {
        let r = u16x4::splat(42);
        let round_constant: u32 = (1 << 16) - 1;
        let a = u32x4::new(0 << 16, 1 << 16, 2 << 16, 3 << 16);
        let b = u32x4::new(
            0 << 16,
            (1 << 16) + round_constant,
            2 << 16,
            (3 << 16) + round_constant,
        );
        let e = u16x8::new(42, 42, 42, 42, 0, 3, 4, 7);
        let r: u16x8 = transmute(vraddhn_high_s32(transmute(r), transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vraddhn_high_u64() {
        let r = u32x2::splat(42);
        let round_constant: u64 = (1 << 32) - 1;
        let a = u64x2::new(0 << 32, 1 << 32);
        let b = u64x2::new(0 << 32, (1 << 32) + round_constant);
        let e = u32x4::new(42, 42, 0, 3);
        let r: u32x4 = transmute(vraddhn_high_s64(transmute(r), transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddl_s8() {
        let a = i8x8::new(-4, -3, -2, -1, 0, 1, 2, 3);
        let r: i16x4 = transmute(vpaddl_s8(transmute(a)));
        let e = i16x4::new(-7, -3, 1, 5);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddl_s16() {
        let a = i16x4::new(-2, -1, 0, 1);
        let r: i32x2 = transmute(vpaddl_s16(transmute(a)));
        let e = i32x2::new(-3, 1);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddl_s32() {
        let a = i32x2::new(-1, 0);
        let r: i64x1 = transmute(vpaddl_s32(transmute(a)));
        let e = i64x1::new(-1);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddlq_s8() {
        let a = i8x16::new(-8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7);
        let r: i16x8 = transmute(vpaddlq_s8(transmute(a)));
        let e = i16x8::new(-15, -11, -7, -3, 1, 5, 9, 13);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddlq_s16() {
        let a = i16x8::new(-4, -3, -2, -1, 0, 1, 2, 3);
        let r: i32x4 = transmute(vpaddlq_s16(transmute(a)));
        let e = i32x4::new(-7, -3, 1, 5);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddlq_s32() {
        let a = i32x4::new(-2, -1, 0, 1);
        let r: i64x2 = transmute(vpaddlq_s32(transmute(a)));
        let e = i64x2::new(-3, 1);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddl_u8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, u8::MAX);
        let r: u16x4 = transmute(vpaddl_u8(transmute(a)));
        let e = u16x4::new(1, 5, 9, u8::MAX as u16 + 6);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddl_u16() {
        let a = u16x4::new(0, 1, 2, u16::MAX);
        let r: u32x2 = transmute(vpaddl_u16(transmute(a)));
        let e = u32x2::new(1, u16::MAX as u32 + 2);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddl_u32() {
        let a = u32x2::new(1, u32::MAX);
        let r: u64x1 = transmute(vpaddl_u32(transmute(a)));
        let e = u64x1::new(u32::MAX as u64 + 1);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddlq_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, u8::MAX);
        let r: u16x8 = transmute(vpaddlq_u8(transmute(a)));
        let e = u16x8::new(1, 5, 9, 13, 17, 21, 25, u8::MAX as u16 + 14);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddlq_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, u16::MAX);
        let r: u32x4 = transmute(vpaddlq_u16(transmute(a)));
        let e = u32x4::new(1, 5, 9, u16::MAX as u32 + 6);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpaddlq_u32() {
        let a = u32x4::new(0, 1, 2, u32::MAX);
        let r: u64x2 = transmute(vpaddlq_u32(transmute(a)));
        let e = u64x2::new(1, u32::MAX as u64 + 2);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadal_s8() {
        let a = i16x4::new(42, 42, 42, 42);
        let b = i8x8::new(-4, -3, -2, -1, 0, 1, 2, 3);
        let r: i16x4 = transmute(vpadal_s8(transmute(a), transmute(b)));
        let e = i16x4::new(35, 39, 43, 47);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadal_s16() {
        let a = i32x2::new(42, 42);
        let b = i16x4::new(-2, -1, 0, 1);
        let r: i32x2 = transmute(vpadal_s16(transmute(a), transmute(b)));
        let e = i32x2::new(39, 43);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadal_s32() {
        let a = i64x1::new(42);
        let b = i32x2::new(-1, 0);
        let r: i64x1 = transmute(vpadal_s32(transmute(a), transmute(b)));
        let e = i64x1::new(41);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadalq_s8() {
        let a = i16x8::new(42, 42, 42, 42, 42, 42, 42, 42);
        let b = i8x16::new(-8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7);
        let r: i16x8 = transmute(vpadalq_s8(transmute(a), transmute(b)));
        let e = i16x8::new(27, 31, 35, 39, 43, 47, 51, 55);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadalq_s16() {
        let a = i32x4::new(42, 42, 42, 42);
        let b = i16x8::new(-4, -3, -2, -1, 0, 1, 2, 3);
        let r: i32x4 = transmute(vpadalq_s16(transmute(a), transmute(b)));
        let e = i32x4::new(35, 39, 43, 47);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadalq_s32() {
        let a = i64x2::new(42, 42);
        let b = i32x4::new(-2, -1, 0, 1);
        let r: i64x2 = transmute(vpadalq_s32(transmute(a), transmute(b)));
        let e = i64x2::new(39, 43);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadal_u8() {
        let a = u16x4::new(42, 42, 42, 42);
        let b = u8x8::new(0, 1, 2, 3, 4, 5, 6, u8::MAX);
        let r: u16x4 = transmute(vpadal_u8(transmute(a), transmute(b)));
        let e = u16x4::new(43, 47, 51, u8::MAX as u16 + 48);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadal_u16() {
        let a = u32x2::new(42, 42);
        let b = u16x4::new(0, 1, 2, u16::MAX);
        let r: u32x2 = transmute(vpadal_u16(transmute(a), transmute(b)));
        let e = u32x2::new(43, u16::MAX as u32 + 44);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadal_u32() {
        let a = u64x1::new(42);
        let b = u32x2::new(1, u32::MAX);
        let r: u64x1 = transmute(vpadal_u32(transmute(a), transmute(b)));
        let e = u64x1::new(u32::MAX as u64 + 43);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadalq_u8() {
        let a = u16x8::new(42, 42, 42, 42, 42, 42, 42, 42);
        let b = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, u8::MAX);
        let r: u16x8 = transmute(vpadalq_u8(transmute(a), transmute(b)));
        let e = u16x8::new(43, 47, 51, 55, 59, 63, 67, u8::MAX as u16 + 56);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadalq_u16() {
        let a = u32x4::new(42, 42, 42, 42);
        let b = u16x8::new(0, 1, 2, 3, 4, 5, 6, u16::MAX);
        let r: u32x4 = transmute(vpadalq_u16(transmute(a), transmute(b)));
        let e = u32x4::new(43, 47, 51, u16::MAX as u32 + 48);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadalq_u32() {
        let a = u64x2::new(42, 42);
        let b = u32x4::new(0, 1, 2, u32::MAX);
        let r: u64x2 = transmute(vpadalq_u32(transmute(a), transmute(b)));
        let e = u64x2::new(43, u32::MAX as u64 + 44);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvn_s8() {
        let a = i8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let e = i8x8::new(-1, -2, -3, -4, -5, -6, -7, -8);
        let r: i8x8 = transmute(vmvn_s8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvnq_s8() {
        let a = i8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let e = i8x16::new(
            -1, -2, -3, -4, -5, -6, -7, -8, -9, -10, -11, -12, -13, -14, -15, -16,
        );
        let r: i8x16 = transmute(vmvnq_s8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvn_s16() {
        let a = i16x4::new(0, 1, 2, 3);
        let e = i16x4::new(-1, -2, -3, -4);
        let r: i16x4 = transmute(vmvn_s16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvnq_s16() {
        let a = i16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let e = i16x8::new(-1, -2, -3, -4, -5, -6, -7, -8);
        let r: i16x8 = transmute(vmvnq_s16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvn_s32() {
        let a = i32x2::new(0, 1);
        let e = i32x2::new(-1, -2);
        let r: i32x2 = transmute(vmvn_s32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvnq_s32() {
        let a = i32x4::new(0, 1, 2, 3);
        let e = i32x4::new(-1, -2, -3, -4);
        let r: i32x4 = transmute(vmvnq_s32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvn_u8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let e = u8x8::new(255, 254, 253, 252, 251, 250, 249, 248);
        let r: u8x8 = transmute(vmvn_u8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvnq_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let e = u8x16::new(
            255, 254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 244, 243, 242, 241, 240,
        );
        let r: u8x16 = transmute(vmvnq_u8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvn_u16() {
        let a = u16x4::new(0, 1, 2, 3);
        let e = u16x4::new(65_535, 65_534, 65_533, 65_532);
        let r: u16x4 = transmute(vmvn_u16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvnq_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let e = u16x8::new(
            65_535, 65_534, 65_533, 65_532, 65_531, 65_530, 65_529, 65_528,
        );
        let r: u16x8 = transmute(vmvnq_u16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvn_u32() {
        let a = u32x2::new(0, 1);
        let e = u32x2::new(4_294_967_295, 4_294_967_294);
        let r: u32x2 = transmute(vmvn_u32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvnq_u32() {
        let a = u32x4::new(0, 1, 2, 3);
        let e = u32x4::new(4_294_967_295, 4_294_967_294, 4_294_967_293, 4_294_967_292);
        let r: u32x4 = transmute(vmvnq_u32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvn_p8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let e = u8x8::new(255, 254, 253, 252, 251, 250, 249, 248);
        let r: u8x8 = transmute(vmvn_p8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmvnq_p8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let e = u8x16::new(
            255, 254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 244, 243, 242, 241, 240,
        );
        let r: u8x16 = transmute(vmvnq_p8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbic_s8() {
        let a = i8x8::new(0, -1, -2, -3, -4, -5, -6, -7);
        let b = i8x8::new(1, 1, 1, 1, 1, 1, 1, 1);
        let e = i8x8::new(0, -2, -2, -4, -4, -6, -6, -8);
        let r: i8x8 = transmute(vbic_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbicq_s8() {
        let a = i8x16::new(
            0, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10, -11, -12, -13, -14, -15,
        );
        let b = i8x16::new(1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1);
        let e = i8x16::new(
            0, -2, -2, -4, -4, -6, -6, -8, -8, -10, -10, -12, -12, -14, -14, -16,
        );
        let r: i8x16 = transmute(vbicq_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbic_s16() {
        let a = i16x4::new(0, -1, -2, -3);
        let b = i16x4::new(1, 1, 1, 1);
        let e = i16x4::new(0, -2, -2, -4);
        let r: i16x4 = transmute(vbic_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbicq_s16() {
        let a = i16x8::new(0, -1, -2, -3, -4, -5, -6, -7);
        let b = i16x8::new(1, 1, 1, 1, 1, 1, 1, 1);
        let e = i16x8::new(0, -2, -2, -4, -4, -6, -6, -8);
        let r: i16x8 = transmute(vbicq_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbic_s32() {
        let a = i32x2::new(0, -1);
        let b = i32x2::new(1, 1);
        let e = i32x2::new(0, -2);
        let r: i32x2 = transmute(vbic_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbicq_s32() {
        let a = i32x4::new(0, -1, -2, -3);
        let b = i32x4::new(1, 1, 1, 1);
        let e = i32x4::new(0, -2, -2, -4);
        let r: i32x4 = transmute(vbicq_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbic_s64() {
        let a = i64x1::new(-1);
        let b = i64x1::new(1);
        let e = i64x1::new(-2);
        let r: i64x1 = transmute(vbic_s64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbicq_s64() {
        let a = i64x2::new(0, -1);
        let b = i64x2::new(1, 1);
        let e = i64x2::new(0, -2);
        let r: i64x2 = transmute(vbicq_s64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbic_u8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let b = u8x8::new(1, 1, 1, 1, 1, 1, 1, 1);
        let e = u8x8::new(0, 0, 2, 2, 4, 4, 6, 6);
        let r: u8x8 = transmute(vbic_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbicq_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let b = u8x16::new(1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1);
        let e = u8x16::new(0, 0, 2, 2, 4, 4, 6, 6, 8, 8, 10, 10, 12, 12, 14, 14);
        let r: u8x16 = transmute(vbicq_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbic_u16() {
        let a = u16x4::new(0, 1, 2, 3);
        let b = u16x4::new(1, 1, 1, 1);
        let e = u16x4::new(0, 0, 2, 2);
        let r: u16x4 = transmute(vbic_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbicq_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let b = u16x8::new(1, 1, 1, 1, 1, 1, 1, 1);
        let e = u16x8::new(0, 0, 2, 2, 4, 4, 6, 6);
        let r: u16x8 = transmute(vbicq_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbic_u32() {
        let a = u32x2::new(0, 1);
        let b = u32x2::new(1, 1);
        let e = u32x2::new(0, 0);
        let r: u32x2 = transmute(vbic_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbicq_u32() {
        let a = u32x4::new(0, 1, 2, 3);
        let b = u32x4::new(1, 1, 1, 1);
        let e = u32x4::new(0, 0, 2, 2);
        let r: u32x4 = transmute(vbicq_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbic_u64() {
        let a = u64x1::new(1);
        let b = u64x1::new(1);
        let e = u64x1::new(0);
        let r: u64x1 = transmute(vbic_u64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbicq_u64() {
        let a = u64x2::new(0, 1);
        let b = u64x2::new(1, 1);
        let e = u64x2::new(0, 0);
        let r: u64x2 = transmute(vbicq_u64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_s8() {
        let a = u8x8::new(u8::MAX, 1, u8::MAX, 2, u8::MAX, 0, u8::MAX, 0);
        let b = i8x8::new(
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
        );
        let c = i8x8::new(
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
        );
        let e = i8x8::new(
            i8::MAX,
            i8::MIN | 1,
            i8::MAX,
            i8::MIN | 2,
            i8::MAX,
            i8::MIN,
            i8::MAX,
            i8::MIN,
        );
        let r: i8x8 = transmute(vbsl_s8(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_s16() {
        let a = u16x4::new(u16::MAX, 0, 1, 2);
        let b = i16x4::new(i16::MAX, i16::MAX, i16::MAX, i16::MAX);
        let c = i16x4::new(i16::MIN, i16::MIN, i16::MIN, i16::MIN);
        let e = i16x4::new(i16::MAX, i16::MIN, i16::MIN | 1, i16::MIN | 2);
        let r: i16x4 = transmute(vbsl_s16(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_s32() {
        let a = u32x2::new(u32::MAX, 1);
        let b = i32x2::new(i32::MAX, i32::MAX);
        let c = i32x2::new(i32::MIN, i32::MIN);
        let e = i32x2::new(i32::MAX, i32::MIN | 1);
        let r: i32x2 = transmute(vbsl_s32(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_s64() {
        let a = u64x1::new(1);
        let b = i64x1::new(i64::MAX);
        let c = i64x1::new(i64::MIN);
        let e = i64x1::new(i64::MIN | 1);
        let r: i64x1 = transmute(vbsl_s64(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_u8() {
        let a = u8x8::new(u8::MAX, 1, u8::MAX, 2, u8::MAX, 0, u8::MAX, 0);
        let b = u8x8::new(
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
        );
        let c = u8x8::new(
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
        );
        let e = u8x8::new(u8::MAX, 1, u8::MAX, 2, u8::MAX, u8::MIN, u8::MAX, u8::MIN);
        let r: u8x8 = transmute(vbsl_u8(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_u16() {
        let a = u16x4::new(u16::MAX, 0, 1, 2);
        let b = u16x4::new(u16::MAX, u16::MAX, u16::MAX, u16::MAX);
        let c = u16x4::new(u16::MIN, u16::MIN, u16::MIN, u16::MIN);
        let e = u16x4::new(u16::MAX, 0, 1, 2);
        let r: u16x4 = transmute(vbsl_u16(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_u32() {
        let a = u32x2::new(u32::MAX, 2);
        let b = u32x2::new(u32::MAX, u32::MAX);
        let c = u32x2::new(u32::MIN, u32::MIN);
        let e = u32x2::new(u32::MAX, 2);
        let r: u32x2 = transmute(vbsl_u32(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_u64() {
        let a = u64x1::new(2);
        let b = u64x1::new(u64::MAX);
        let c = u64x1::new(u64::MIN);
        let e = u64x1::new(2);
        let r: u64x1 = transmute(vbsl_u64(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_f32() {
        let a = u32x2::new(1, 0x80000000);
        let b = f32x2::new(8388609f32, -1.23f32);
        let c = f32x2::new(2097152f32, 2.34f32);
        let e = f32x2::new(2097152.25f32, -2.34f32);
        let r: f32x2 = transmute(vbsl_f32(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_p8() {
        let a = u8x8::new(u8::MAX, 1, u8::MAX, 2, u8::MAX, 0, u8::MAX, 0);
        let b = u8x8::new(
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
        );
        let c = u8x8::new(
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
        );
        let e = u8x8::new(u8::MAX, 1, u8::MAX, 2, u8::MAX, u8::MIN, u8::MAX, u8::MIN);
        let r: u8x8 = transmute(vbsl_p8(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbsl_p16() {
        let a = u16x4::new(u16::MAX, 0, 1, 2);
        let b = u16x4::new(u16::MAX, u16::MAX, u16::MAX, u16::MAX);
        let c = u16x4::new(u16::MIN, u16::MIN, u16::MIN, u16::MIN);
        let e = u16x4::new(u16::MAX, 0, 1, 2);
        let r: u16x4 = transmute(vbsl_p16(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_s8() {
        let a = u8x16::new(
            u8::MAX,
            1,
            u8::MAX,
            2,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
        );
        let b = i8x16::new(
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
            i8::MAX,
        );
        let c = i8x16::new(
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
            i8::MIN,
        );
        let e = i8x16::new(
            i8::MAX,
            i8::MIN | 1,
            i8::MAX,
            i8::MIN | 2,
            i8::MAX,
            i8::MIN,
            i8::MAX,
            i8::MIN,
            i8::MAX,
            i8::MIN,
            i8::MAX,
            i8::MIN,
            i8::MAX,
            i8::MIN,
            i8::MAX,
            i8::MIN,
        );
        let r: i8x16 = transmute(vbslq_s8(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_s16() {
        let a = u16x8::new(u16::MAX, 1, u16::MAX, 2, u16::MAX, 0, u16::MAX, 0);
        let b = i16x8::new(
            i16::MAX,
            i16::MAX,
            i16::MAX,
            i16::MAX,
            i16::MAX,
            i16::MAX,
            i16::MAX,
            i16::MAX,
        );
        let c = i16x8::new(
            i16::MIN,
            i16::MIN,
            i16::MIN,
            i16::MIN,
            i16::MIN,
            i16::MIN,
            i16::MIN,
            i16::MIN,
        );
        let e = i16x8::new(
            i16::MAX,
            i16::MIN | 1,
            i16::MAX,
            i16::MIN | 2,
            i16::MAX,
            i16::MIN,
            i16::MAX,
            i16::MIN,
        );
        let r: i16x8 = transmute(vbslq_s16(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_s32() {
        let a = u32x4::new(u32::MAX, 1, u32::MAX, 2);
        let b = i32x4::new(i32::MAX, i32::MAX, i32::MAX, i32::MAX);
        let c = i32x4::new(i32::MIN, i32::MIN, i32::MIN, i32::MIN);
        let e = i32x4::new(i32::MAX, i32::MIN | 1, i32::MAX, i32::MIN | 2);
        let r: i32x4 = transmute(vbslq_s32(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_s64() {
        let a = u64x2::new(u64::MAX, 1);
        let b = i64x2::new(i64::MAX, i64::MAX);
        let c = i64x2::new(i64::MIN, i64::MIN);
        let e = i64x2::new(i64::MAX, i64::MIN | 1);
        let r: i64x2 = transmute(vbslq_s64(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_u8() {
        let a = u8x16::new(
            u8::MAX,
            1,
            u8::MAX,
            2,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
        );
        let b = u8x16::new(
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
        );
        let c = u8x16::new(
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
        );
        let e = u8x16::new(
            u8::MAX,
            1,
            u8::MAX,
            2,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
        );
        let r: u8x16 = transmute(vbslq_u8(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_u16() {
        let a = u16x8::new(u16::MAX, 1, u16::MAX, 2, u16::MAX, 0, u16::MAX, 0);
        let b = u16x8::new(
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
        );
        let c = u16x8::new(
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
        );
        let e = u16x8::new(
            u16::MAX,
            1,
            u16::MAX,
            2,
            u16::MAX,
            u16::MIN,
            u16::MAX,
            u16::MIN,
        );
        let r: u16x8 = transmute(vbslq_u16(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_u32() {
        let a = u32x4::new(u32::MAX, 1, u32::MAX, 2);
        let b = u32x4::new(u32::MAX, u32::MAX, u32::MAX, u32::MAX);
        let c = u32x4::new(u32::MIN, u32::MIN, u32::MIN, u32::MIN);
        let e = u32x4::new(u32::MAX, 1, u32::MAX, 2);
        let r: u32x4 = transmute(vbslq_u32(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_u64() {
        let a = u64x2::new(u64::MAX, 1);
        let b = u64x2::new(u64::MAX, u64::MAX);
        let c = u64x2::new(u64::MIN, u64::MIN);
        let e = u64x2::new(u64::MAX, 1);
        let r: u64x2 = transmute(vbslq_u64(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_f32() {
        let a = u32x4::new(u32::MAX, 0, 1, 0x80000000);
        let b = f32x4::new(-1.23f32, -1.23f32, 8388609f32, -1.23f32);
        let c = f32x4::new(2.34f32, 2.34f32, 2097152f32, 2.34f32);
        let e = f32x4::new(-1.23f32, 2.34f32, 2097152.25f32, -2.34f32);
        let r: f32x4 = transmute(vbslq_f32(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_p8() {
        let a = u8x16::new(
            u8::MAX,
            1,
            u8::MAX,
            2,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
            u8::MAX,
            0,
        );
        let b = u8x16::new(
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
        );
        let c = u8x16::new(
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
            u8::MIN,
        );
        let e = u8x16::new(
            u8::MAX,
            1,
            u8::MAX,
            2,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
            u8::MAX,
            u8::MIN,
        );
        let r: u8x16 = transmute(vbslq_p8(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vbslq_p16() {
        let a = u16x8::new(u16::MAX, 1, u16::MAX, 2, u16::MAX, 0, u16::MAX, 0);
        let b = u16x8::new(
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
        );
        let c = u16x8::new(
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
            u16::MIN,
        );
        let e = u16x8::new(
            u16::MAX,
            1,
            u16::MAX,
            2,
            u16::MAX,
            u16::MIN,
            u16::MAX,
            u16::MIN,
        );
        let r: u16x8 = transmute(vbslq_p16(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorn_s8() {
        let a = i8x8::new(0, -1, -2, -3, -4, -5, -6, -7);
        let b = i8x8::new(-2, -2, -2, -2, -2, -2, -2, -2);
        let e = i8x8::new(1, -1, -1, -3, -3, -5, -5, -7);
        let r: i8x8 = transmute(vorn_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vornq_s8() {
        let a = i8x16::new(
            0, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10, -11, -12, -13, -14, -15,
        );
        let b = i8x16::new(
            -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2,
        );
        let e = i8x16::new(
            1, -1, -1, -3, -3, -5, -5, -7, -7, -9, -9, -11, -11, -13, -13, -15,
        );
        let r: i8x16 = transmute(vornq_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorn_s16() {
        let a = i16x4::new(0, -1, -2, -3);
        let b = i16x4::new(-2, -2, -2, -2);
        let e = i16x4::new(1, -1, -1, -3);
        let r: i16x4 = transmute(vorn_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vornq_s16() {
        let a = i16x8::new(0, -1, -2, -3, -4, -5, -6, -7);
        let b = i16x8::new(-2, -2, -2, -2, -2, -2, -2, -2);
        let e = i16x8::new(1, -1, -1, -3, -3, -5, -5, -7);
        let r: i16x8 = transmute(vornq_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorn_s32() {
        let a = i32x2::new(0, -1);
        let b = i32x2::new(-2, -2);
        let e = i32x2::new(1, -1);
        let r: i32x2 = transmute(vorn_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vornq_s32() {
        let a = i32x4::new(0, -1, -2, -3);
        let b = i32x4::new(-2, -2, -2, -2);
        let e = i32x4::new(1, -1, -1, -3);
        let r: i32x4 = transmute(vornq_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorn_s64() {
        let a = i64x1::new(0);
        let b = i64x1::new(-2);
        let e = i64x1::new(1);
        let r: i64x1 = transmute(vorn_s64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vornq_s64() {
        let a = i64x2::new(0, -1);
        let b = i64x2::new(-2, -2);
        let e = i64x2::new(1, -1);
        let r: i64x2 = transmute(vornq_s64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorn_u8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let t = u8::MAX - 1;
        let b = u8x8::new(t, t, t, t, t, t, t, t);
        let e = u8x8::new(1, 1, 3, 3, 5, 5, 7, 7);
        let r: u8x8 = transmute(vorn_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vornq_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let t = u8::MAX - 1;
        let b = u8x16::new(t, t, t, t, t, t, t, t, t, t, t, t, t, t, t, t);
        let e = u8x16::new(1, 1, 3, 3, 5, 5, 7, 7, 9, 9, 11, 11, 13, 13, 15, 15);
        let r: u8x16 = transmute(vornq_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorn_u16() {
        let a = u16x4::new(0, 1, 2, 3);
        let t = u16::MAX - 1;
        let b = u16x4::new(t, t, t, t);
        let e = u16x4::new(1, 1, 3, 3);
        let r: u16x4 = transmute(vorn_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vornq_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let t = u16::MAX - 1;
        let b = u16x8::new(t, t, t, t, t, t, t, t);
        let e = u16x8::new(1, 1, 3, 3, 5, 5, 7, 7);
        let r: u16x8 = transmute(vornq_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorn_u32() {
        let a = u32x2::new(0, 1);
        let t = u32::MAX - 1;
        let b = u32x2::new(t, t);
        let e = u32x2::new(1, 1);
        let r: u32x2 = transmute(vorn_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vornq_u32() {
        let a = u32x4::new(0, 1, 2, 3);
        let t = u32::MAX - 1;
        let b = u32x4::new(t, t, t, t);
        let e = u32x4::new(1, 1, 3, 3);
        let r: u32x4 = transmute(vornq_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorn_u64() {
        let a = u64x1::new(0);
        let t = u64::MAX - 1;
        let b = u64x1::new(t);
        let e = u64x1::new(1);
        let r: u64x1 = transmute(vorn_u64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vornq_u64() {
        let a = u64x2::new(0, 1);
        let t = u64::MAX - 1;
        let b = u64x2::new(t, t);
        let e = u64x2::new(1, 1);
        let r: u64x2 = transmute(vornq_u64(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovn_s16() {
        let a = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let e = i8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: i8x8 = transmute(vmovn_s16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovn_s32() {
        let a = i32x4::new(1, 2, 3, 4);
        let e = i16x4::new(1, 2, 3, 4);
        let r: i16x4 = transmute(vmovn_s32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovn_s64() {
        let a = i64x2::new(1, 2);
        let e = i32x2::new(1, 2);
        let r: i32x2 = transmute(vmovn_s64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovn_u16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let e = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: u8x8 = transmute(vmovn_u16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovn_u32() {
        let a = u32x4::new(1, 2, 3, 4);
        let e = u16x4::new(1, 2, 3, 4);
        let r: u16x4 = transmute(vmovn_u32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovn_u64() {
        let a = u64x2::new(1, 2);
        let e = u32x2::new(1, 2);
        let r: u32x2 = transmute(vmovn_u64(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovl_s8() {
        let e = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let a = i8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: i16x8 = transmute(vmovl_s8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovl_s16() {
        let e = i32x4::new(1, 2, 3, 4);
        let a = i16x4::new(1, 2, 3, 4);
        let r: i32x4 = transmute(vmovl_s16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovl_s32() {
        let e = i64x2::new(1, 2);
        let a = i32x2::new(1, 2);
        let r: i64x2 = transmute(vmovl_s32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovl_u8() {
        let e = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let a = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let r: u16x8 = transmute(vmovl_u8(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovl_u16() {
        let e = u32x4::new(1, 2, 3, 4);
        let a = u16x4::new(1, 2, 3, 4);
        let r: u32x4 = transmute(vmovl_u16(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmovl_u32() {
        let e = u64x2::new(1, 2);
        let a = u32x2::new(1, 2);
        let r: u64x2 = transmute(vmovl_u32(transmute(a)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_s8() {
        let a = i8x8::new(1, -2, 3, -4, 5, 6, 7, 8);
        let b = i8x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = i8x8::new(-2, -4, 5, 7, 0, 2, 4, 6);
        let r: i8x8 = transmute(vpmin_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_s16() {
        let a = i16x4::new(1, 2, 3, -4);
        let b = i16x4::new(0, 3, 2, 5);
        let e = i16x4::new(1, -4, 0, 2);
        let r: i16x4 = transmute(vpmin_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_s32() {
        let a = i32x2::new(1, -2);
        let b = i32x2::new(0, 3);
        let e = i32x2::new(-2, 0);
        let r: i32x2 = transmute(vpmin_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_u8() {
        let a = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = u8x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = u8x8::new(1, 3, 5, 7, 0, 2, 4, 6);
        let r: u8x8 = transmute(vpmin_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_u16() {
        let a = u16x4::new(1, 2, 3, 4);
        let b = u16x4::new(0, 3, 2, 5);
        let e = u16x4::new(1, 3, 0, 2);
        let r: u16x4 = transmute(vpmin_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_u32() {
        let a = u32x2::new(1, 2);
        let b = u32x2::new(0, 3);
        let e = u32x2::new(1, 0);
        let r: u32x2 = transmute(vpmin_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmin_f32() {
        let a = f32x2::new(1., -2.);
        let b = f32x2::new(0., 3.);
        let e = f32x2::new(-2., 0.);
        let r: f32x2 = transmute(vpmin_f32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_s8() {
        let a = i8x8::new(1, -2, 3, -4, 5, 6, 7, 8);
        let b = i8x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = i8x8::new(1, 3, 6, 8, 3, 5, 7, 9);
        let r: i8x8 = transmute(vpmax_s8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_s16() {
        let a = i16x4::new(1, 2, 3, -4);
        let b = i16x4::new(0, 3, 2, 5);
        let e = i16x4::new(2, 3, 3, 5);
        let r: i16x4 = transmute(vpmax_s16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_s32() {
        let a = i32x2::new(1, -2);
        let b = i32x2::new(0, 3);
        let e = i32x2::new(1, 3);
        let r: i32x2 = transmute(vpmax_s32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_u8() {
        let a = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = u8x8::new(0, 3, 2, 5, 4, 7, 6, 9);
        let e = u8x8::new(2, 4, 6, 8, 3, 5, 7, 9);
        let r: u8x8 = transmute(vpmax_u8(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_u16() {
        let a = u16x4::new(1, 2, 3, 4);
        let b = u16x4::new(0, 3, 2, 5);
        let e = u16x4::new(2, 4, 3, 5);
        let r: u16x4 = transmute(vpmax_u16(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_u32() {
        let a = u32x2::new(1, 2);
        let b = u32x2::new(0, 3);
        let e = u32x2::new(2, 3);
        let r: u32x2 = transmute(vpmax_u32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpmax_f32() {
        let a = f32x2::new(1., -2.);
        let b = f32x2::new(0., 3.);
        let e = f32x2::new(1., 3.);
        let r: f32x2 = transmute(vpmax_f32(transmute(a), transmute(b)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vand_s8() {
        test_bit_s8(|i, j| vand_s8(i, j), |a: i8, b: i8| -> i8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vandq_s8() {
        testq_bit_s8(|i, j| vandq_s8(i, j), |a: i8, b: i8| -> i8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vand_s16() {
        test_bit_s16(|i, j| vand_s16(i, j), |a: i16, b: i16| -> i16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vandq_s16() {
        testq_bit_s16(|i, j| vandq_s16(i, j), |a: i16, b: i16| -> i16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vand_s32() {
        test_bit_s32(|i, j| vand_s32(i, j), |a: i32, b: i32| -> i32 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vandq_s32() {
        testq_bit_s32(|i, j| vandq_s32(i, j), |a: i32, b: i32| -> i32 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vand_s64() {
        test_bit_s64(|i, j| vand_s64(i, j), |a: i64, b: i64| -> i64 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vandq_s64() {
        testq_bit_s64(|i, j| vandq_s64(i, j), |a: i64, b: i64| -> i64 { a & b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vand_u8() {
        test_bit_u8(|i, j| vand_u8(i, j), |a: u8, b: u8| -> u8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vandq_u8() {
        testq_bit_u8(|i, j| vandq_u8(i, j), |a: u8, b: u8| -> u8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vand_u16() {
        test_bit_u16(|i, j| vand_u16(i, j), |a: u16, b: u16| -> u16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vandq_u16() {
        testq_bit_u16(|i, j| vandq_u16(i, j), |a: u16, b: u16| -> u16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vand_u32() {
        test_bit_u32(|i, j| vand_u32(i, j), |a: u32, b: u32| -> u32 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vandq_u32() {
        testq_bit_u32(|i, j| vandq_u32(i, j), |a: u32, b: u32| -> u32 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vand_u64() {
        test_bit_u64(|i, j| vand_u64(i, j), |a: u64, b: u64| -> u64 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vandq_u64() {
        testq_bit_u64(|i, j| vandq_u64(i, j), |a: u64, b: u64| -> u64 { a & b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorr_s8() {
        test_bit_s8(|i, j| vorr_s8(i, j), |a: i8, b: i8| -> i8 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorrq_s8() {
        testq_bit_s8(|i, j| vorrq_s8(i, j), |a: i8, b: i8| -> i8 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorr_s16() {
        test_bit_s16(|i, j| vorr_s16(i, j), |a: i16, b: i16| -> i16 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorrq_s16() {
        testq_bit_s16(|i, j| vorrq_s16(i, j), |a: i16, b: i16| -> i16 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorr_s32() {
        test_bit_s32(|i, j| vorr_s32(i, j), |a: i32, b: i32| -> i32 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorrq_s32() {
        testq_bit_s32(|i, j| vorrq_s32(i, j), |a: i32, b: i32| -> i32 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorr_s64() {
        test_bit_s64(|i, j| vorr_s64(i, j), |a: i64, b: i64| -> i64 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorrq_s64() {
        testq_bit_s64(|i, j| vorrq_s64(i, j), |a: i64, b: i64| -> i64 { a | b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vorr_u8() {
        test_bit_u8(|i, j| vorr_u8(i, j), |a: u8, b: u8| -> u8 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorrq_u8() {
        testq_bit_u8(|i, j| vorrq_u8(i, j), |a: u8, b: u8| -> u8 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorr_u16() {
        test_bit_u16(|i, j| vorr_u16(i, j), |a: u16, b: u16| -> u16 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorrq_u16() {
        testq_bit_u16(|i, j| vorrq_u16(i, j), |a: u16, b: u16| -> u16 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorr_u32() {
        test_bit_u32(|i, j| vorr_u32(i, j), |a: u32, b: u32| -> u32 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorrq_u32() {
        testq_bit_u32(|i, j| vorrq_u32(i, j), |a: u32, b: u32| -> u32 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorr_u64() {
        test_bit_u64(|i, j| vorr_u64(i, j), |a: u64, b: u64| -> u64 { a | b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vorrq_u64() {
        testq_bit_u64(|i, j| vorrq_u64(i, j), |a: u64, b: u64| -> u64 { a | b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_veor_s8() {
        test_bit_s8(|i, j| veor_s8(i, j), |a: i8, b: i8| -> i8 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veorq_s8() {
        testq_bit_s8(|i, j| veorq_s8(i, j), |a: i8, b: i8| -> i8 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veor_s16() {
        test_bit_s16(|i, j| veor_s16(i, j), |a: i16, b: i16| -> i16 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veorq_s16() {
        testq_bit_s16(|i, j| veorq_s16(i, j), |a: i16, b: i16| -> i16 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veor_s32() {
        test_bit_s32(|i, j| veor_s32(i, j), |a: i32, b: i32| -> i32 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veorq_s32() {
        testq_bit_s32(|i, j| veorq_s32(i, j), |a: i32, b: i32| -> i32 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veor_s64() {
        test_bit_s64(|i, j| veor_s64(i, j), |a: i64, b: i64| -> i64 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veorq_s64() {
        testq_bit_s64(|i, j| veorq_s64(i, j), |a: i64, b: i64| -> i64 { a ^ b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_veor_u8() {
        test_bit_u8(|i, j| veor_u8(i, j), |a: u8, b: u8| -> u8 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veorq_u8() {
        testq_bit_u8(|i, j| veorq_u8(i, j), |a: u8, b: u8| -> u8 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veor_u16() {
        test_bit_u16(|i, j| veor_u16(i, j), |a: u16, b: u16| -> u16 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veorq_u16() {
        testq_bit_u16(|i, j| veorq_u16(i, j), |a: u16, b: u16| -> u16 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veor_u32() {
        test_bit_u32(|i, j| veor_u32(i, j), |a: u32, b: u32| -> u32 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veorq_u32() {
        testq_bit_u32(|i, j| veorq_u32(i, j), |a: u32, b: u32| -> u32 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veor_u64() {
        test_bit_u64(|i, j| veor_u64(i, j), |a: u64, b: u64| -> u64 { a ^ b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_veorq_u64() {
        testq_bit_u64(|i, j| veorq_u64(i, j), |a: u64, b: u64| -> u64 { a ^ b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_s8() {
        test_cmp_s8(
            |i, j| vceq_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a == b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_s8() {
        testq_cmp_s8(
            |i, j| vceqq_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a == b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_s16() {
        test_cmp_s16(
            |i, j| vceq_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a == b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_s16() {
        testq_cmp_s16(
            |i, j| vceqq_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a == b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_s32() {
        test_cmp_s32(
            |i, j| vceq_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a == b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_s32() {
        testq_cmp_s32(
            |i, j| vceqq_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a == b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_u8() {
        test_cmp_u8(
            |i, j| vceq_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a == b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_u8() {
        testq_cmp_u8(
            |i, j| vceqq_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a == b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_u16() {
        test_cmp_u16(
            |i, j| vceq_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a == b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_u16() {
        testq_cmp_u16(
            |i, j| vceqq_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a == b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_u32() {
        test_cmp_u32(
            |i, j| vceq_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a == b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_u32() {
        testq_cmp_u32(
            |i, j| vceqq_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a == b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vceq_f32() {
        test_cmp_f32(
            |i, j| vcge_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a == b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vceqq_f32() {
        testq_cmp_f32(
            |i, j| vcgeq_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a == b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_s8() {
        test_cmp_s8(
            |i, j| vcgt_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a > b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_s8() {
        testq_cmp_s8(
            |i, j| vcgtq_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a > b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_s16() {
        test_cmp_s16(
            |i, j| vcgt_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a > b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_s16() {
        testq_cmp_s16(
            |i, j| vcgtq_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a > b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_s32() {
        test_cmp_s32(
            |i, j| vcgt_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a > b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_s32() {
        testq_cmp_s32(
            |i, j| vcgtq_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a > b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_u8() {
        test_cmp_u8(
            |i, j| vcgt_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a > b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_u8() {
        testq_cmp_u8(
            |i, j| vcgtq_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a > b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_u16() {
        test_cmp_u16(
            |i, j| vcgt_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a > b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_u16() {
        testq_cmp_u16(
            |i, j| vcgtq_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a > b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_u32() {
        test_cmp_u32(
            |i, j| vcgt_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a > b {
                    0xFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_u32() {
        testq_cmp_u32(
            |i, j| vcgtq_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a > b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcgt_f32() {
        test_cmp_f32(
            |i, j| vcgt_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a > b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgtq_f32() {
        testq_cmp_f32(
            |i, j| vcgtq_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a > b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vclt_s8() {
        test_cmp_s8(
            |i, j| vclt_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a < b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_s8() {
        testq_cmp_s8(
            |i, j| vcltq_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a < b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vclt_s16() {
        test_cmp_s16(
            |i, j| vclt_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a < b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_s16() {
        testq_cmp_s16(
            |i, j| vcltq_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a < b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vclt_s32() {
        test_cmp_s32(
            |i, j| vclt_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a < b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_s32() {
        testq_cmp_s32(
            |i, j| vcltq_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a < b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vclt_u8() {
        test_cmp_u8(
            |i, j| vclt_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a < b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_u8() {
        testq_cmp_u8(
            |i, j| vcltq_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a < b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vclt_u16() {
        test_cmp_u16(
            |i, j| vclt_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a < b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_u16() {
        testq_cmp_u16(
            |i, j| vcltq_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a < b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vclt_u32() {
        test_cmp_u32(
            |i, j| vclt_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a < b {
                    0xFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_u32() {
        testq_cmp_u32(
            |i, j| vcltq_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a < b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vclt_f32() {
        test_cmp_f32(
            |i, j| vclt_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a < b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcltq_f32() {
        testq_cmp_f32(
            |i, j| vcltq_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a < b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcle_s8() {
        test_cmp_s8(
            |i, j| vcle_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a <= b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_s8() {
        testq_cmp_s8(
            |i, j| vcleq_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a <= b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcle_s16() {
        test_cmp_s16(
            |i, j| vcle_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a <= b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_s16() {
        testq_cmp_s16(
            |i, j| vcleq_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a <= b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcle_s32() {
        test_cmp_s32(
            |i, j| vcle_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a <= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_s32() {
        testq_cmp_s32(
            |i, j| vcleq_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a <= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcle_u8() {
        test_cmp_u8(
            |i, j| vcle_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a <= b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_u8() {
        testq_cmp_u8(
            |i, j| vcleq_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a <= b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcle_u16() {
        test_cmp_u16(
            |i, j| vcle_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a <= b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_u16() {
        testq_cmp_u16(
            |i, j| vcleq_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a <= b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcle_u32() {
        test_cmp_u32(
            |i, j| vcle_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a <= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_u32() {
        testq_cmp_u32(
            |i, j| vcleq_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a <= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcle_f32() {
        test_cmp_f32(
            |i, j| vcle_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a <= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcleq_f32() {
        testq_cmp_f32(
            |i, j| vcleq_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a <= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcge_s8() {
        test_cmp_s8(
            |i, j| vcge_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a >= b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_s8() {
        testq_cmp_s8(
            |i, j| vcgeq_s8(i, j),
            |a: i8, b: i8| -> u8 {
                if a >= b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcge_s16() {
        test_cmp_s16(
            |i, j| vcge_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a >= b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_s16() {
        testq_cmp_s16(
            |i, j| vcgeq_s16(i, j),
            |a: i16, b: i16| -> u16 {
                if a >= b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcge_s32() {
        test_cmp_s32(
            |i, j| vcge_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a >= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_s32() {
        testq_cmp_s32(
            |i, j| vcgeq_s32(i, j),
            |a: i32, b: i32| -> u32 {
                if a >= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcge_u8() {
        test_cmp_u8(
            |i, j| vcge_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a >= b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_u8() {
        testq_cmp_u8(
            |i, j| vcgeq_u8(i, j),
            |a: u8, b: u8| -> u8 {
                if a >= b {
                    0xFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcge_u16() {
        test_cmp_u16(
            |i, j| vcge_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a >= b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_u16() {
        testq_cmp_u16(
            |i, j| vcgeq_u16(i, j),
            |a: u16, b: u16| -> u16 {
                if a >= b {
                    0xFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcge_u32() {
        test_cmp_u32(
            |i, j| vcge_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a >= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_u32() {
        testq_cmp_u32(
            |i, j| vcgeq_u32(i, j),
            |a: u32, b: u32| -> u32 {
                if a >= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vcge_f32() {
        test_cmp_f32(
            |i, j| vcge_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a >= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcgeq_f32() {
        testq_cmp_f32(
            |i, j| vcgeq_f32(i, j),
            |a: f32, b: f32| -> u32 {
                if a >= b {
                    0xFFFFFFFF
                } else {
                    0
                }
            },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vqsub_s8() {
        test_ari_s8(
            |i, j| vqsub_s8(i, j),
            |a: i8, b: i8| -> i8 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsubq_s8() {
        testq_ari_s8(
            |i, j| vqsubq_s8(i, j),
            |a: i8, b: i8| -> i8 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsub_s16() {
        test_ari_s16(
            |i, j| vqsub_s16(i, j),
            |a: i16, b: i16| -> i16 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsubq_s16() {
        testq_ari_s16(
            |i, j| vqsubq_s16(i, j),
            |a: i16, b: i16| -> i16 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsub_s32() {
        test_ari_s32(
            |i, j| vqsub_s32(i, j),
            |a: i32, b: i32| -> i32 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsubq_s32() {
        testq_ari_s32(
            |i, j| vqsubq_s32(i, j),
            |a: i32, b: i32| -> i32 { a.saturating_sub(b) },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vqsub_u8() {
        test_ari_u8(
            |i, j| vqsub_u8(i, j),
            |a: u8, b: u8| -> u8 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsubq_u8() {
        testq_ari_u8(
            |i, j| vqsubq_u8(i, j),
            |a: u8, b: u8| -> u8 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsub_u16() {
        test_ari_u16(
            |i, j| vqsub_u16(i, j),
            |a: u16, b: u16| -> u16 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsubq_u16() {
        testq_ari_u16(
            |i, j| vqsubq_u16(i, j),
            |a: u16, b: u16| -> u16 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsub_u32() {
        test_ari_u32(
            |i, j| vqsub_u32(i, j),
            |a: u32, b: u32| -> u32 { a.saturating_sub(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqsubq_u32() {
        testq_ari_u32(
            |i, j| vqsubq_u32(i, j),
            |a: u32, b: u32| -> u32 { a.saturating_sub(b) },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vhadd_s8() {
        test_ari_s8(|i, j| vhadd_s8(i, j), |a: i8, b: i8| -> i8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhaddq_s8() {
        testq_ari_s8(|i, j| vhaddq_s8(i, j), |a: i8, b: i8| -> i8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhadd_s16() {
        test_ari_s16(|i, j| vhadd_s16(i, j), |a: i16, b: i16| -> i16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhaddq_s16() {
        testq_ari_s16(|i, j| vhaddq_s16(i, j), |a: i16, b: i16| -> i16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhadd_s32() {
        test_ari_s32(|i, j| vhadd_s32(i, j), |a: i32, b: i32| -> i32 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhaddq_s32() {
        testq_ari_s32(|i, j| vhaddq_s32(i, j), |a: i32, b: i32| -> i32 { a & b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vhadd_u8() {
        test_ari_u8(|i, j| vhadd_u8(i, j), |a: u8, b: u8| -> u8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhaddq_u8() {
        testq_ari_u8(|i, j| vhaddq_u8(i, j), |a: u8, b: u8| -> u8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhadd_u16() {
        test_ari_u16(|i, j| vhadd_u16(i, j), |a: u16, b: u16| -> u16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhaddq_u16() {
        testq_ari_u16(|i, j| vhaddq_u16(i, j), |a: u16, b: u16| -> u16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhadd_u32() {
        test_ari_u32(|i, j| vhadd_u32(i, j), |a: u32, b: u32| -> u32 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhaddq_u32() {
        testq_ari_u32(|i, j| vhaddq_u32(i, j), |a: u32, b: u32| -> u32 { a & b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vrhadd_s8() {
        test_ari_s8(|i, j| vrhadd_s8(i, j), |a: i8, b: i8| -> i8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhaddq_s8() {
        testq_ari_s8(|i, j| vrhaddq_s8(i, j), |a: i8, b: i8| -> i8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhadd_s16() {
        test_ari_s16(|i, j| vrhadd_s16(i, j), |a: i16, b: i16| -> i16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhaddq_s16() {
        testq_ari_s16(|i, j| vrhaddq_s16(i, j), |a: i16, b: i16| -> i16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhadd_s32() {
        test_ari_s32(|i, j| vrhadd_s32(i, j), |a: i32, b: i32| -> i32 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhaddq_s32() {
        testq_ari_s32(|i, j| vrhaddq_s32(i, j), |a: i32, b: i32| -> i32 { a & b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vrhadd_u8() {
        test_ari_u8(|i, j| vrhadd_u8(i, j), |a: u8, b: u8| -> u8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhaddq_u8() {
        testq_ari_u8(|i, j| vrhaddq_u8(i, j), |a: u8, b: u8| -> u8 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhadd_u16() {
        test_ari_u16(|i, j| vrhadd_u16(i, j), |a: u16, b: u16| -> u16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhaddq_u16() {
        testq_ari_u16(|i, j| vrhaddq_u16(i, j), |a: u16, b: u16| -> u16 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhadd_u32() {
        test_ari_u32(|i, j| vrhadd_u32(i, j), |a: u32, b: u32| -> u32 { a & b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrhaddq_u32() {
        testq_ari_u32(|i, j| vrhaddq_u32(i, j), |a: u32, b: u32| -> u32 { a & b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vqadd_s8() {
        test_ari_s8(
            |i, j| vqadd_s8(i, j),
            |a: i8, b: i8| -> i8 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqaddq_s8() {
        testq_ari_s8(
            |i, j| vqaddq_s8(i, j),
            |a: i8, b: i8| -> i8 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqadd_s16() {
        test_ari_s16(
            |i, j| vqadd_s16(i, j),
            |a: i16, b: i16| -> i16 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqaddq_s16() {
        testq_ari_s16(
            |i, j| vqaddq_s16(i, j),
            |a: i16, b: i16| -> i16 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqadd_s32() {
        test_ari_s32(
            |i, j| vqadd_s32(i, j),
            |a: i32, b: i32| -> i32 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqaddq_s32() {
        testq_ari_s32(
            |i, j| vqaddq_s32(i, j),
            |a: i32, b: i32| -> i32 { a.saturating_add(b) },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vqadd_u8() {
        test_ari_u8(
            |i, j| vqadd_u8(i, j),
            |a: u8, b: u8| -> u8 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqaddq_u8() {
        testq_ari_u8(
            |i, j| vqaddq_u8(i, j),
            |a: u8, b: u8| -> u8 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqadd_u16() {
        test_ari_u16(
            |i, j| vqadd_u16(i, j),
            |a: u16, b: u16| -> u16 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqaddq_u16() {
        testq_ari_u16(
            |i, j| vqaddq_u16(i, j),
            |a: u16, b: u16| -> u16 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqadd_u32() {
        test_ari_u32(
            |i, j| vqadd_u32(i, j),
            |a: u32, b: u32| -> u32 { a.saturating_add(b) },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vqaddq_u32() {
        testq_ari_u32(
            |i, j| vqaddq_u32(i, j),
            |a: u32, b: u32| -> u32 { a.saturating_add(b) },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmul_s8() {
        test_ari_s8(
            |i, j| vmul_s8(i, j),
            |a: i8, b: i8| -> i8 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmulq_s8() {
        testq_ari_s8(
            |i, j| vmulq_s8(i, j),
            |a: i8, b: i8| -> i8 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmul_s16() {
        test_ari_s16(
            |i, j| vmul_s16(i, j),
            |a: i16, b: i16| -> i16 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmulq_s16() {
        testq_ari_s16(
            |i, j| vmulq_s16(i, j),
            |a: i16, b: i16| -> i16 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmul_s32() {
        test_ari_s32(
            |i, j| vmul_s32(i, j),
            |a: i32, b: i32| -> i32 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmulq_s32() {
        testq_ari_s32(
            |i, j| vmulq_s32(i, j),
            |a: i32, b: i32| -> i32 { a.overflowing_mul(b).0 },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmul_u8() {
        test_ari_u8(
            |i, j| vmul_u8(i, j),
            |a: u8, b: u8| -> u8 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmulq_u8() {
        testq_ari_u8(
            |i, j| vmulq_u8(i, j),
            |a: u8, b: u8| -> u8 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmul_u16() {
        test_ari_u16(
            |i, j| vmul_u16(i, j),
            |a: u16, b: u16| -> u16 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmulq_u16() {
        testq_ari_u16(
            |i, j| vmulq_u16(i, j),
            |a: u16, b: u16| -> u16 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmul_u32() {
        test_ari_u32(
            |i, j| vmul_u32(i, j),
            |a: u32, b: u32| -> u32 { a.overflowing_mul(b).0 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmulq_u32() {
        testq_ari_u32(
            |i, j| vmulq_u32(i, j),
            |a: u32, b: u32| -> u32 { a.overflowing_mul(b).0 },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vmul_f32() {
        test_ari_f32(|i, j| vmul_f32(i, j), |a: f32, b: f32| -> f32 { a * b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vmulq_f32() {
        testq_ari_f32(|i, j| vmulq_f32(i, j), |a: f32, b: f32| -> f32 { a * b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vsub_s8() {
        test_ari_s8(|i, j| vsub_s8(i, j), |a: i8, b: i8| -> i8 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsubq_s8() {
        testq_ari_s8(|i, j| vsubq_s8(i, j), |a: i8, b: i8| -> i8 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsub_s16() {
        test_ari_s16(|i, j| vsub_s16(i, j), |a: i16, b: i16| -> i16 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsubq_s16() {
        testq_ari_s16(|i, j| vsubq_s16(i, j), |a: i16, b: i16| -> i16 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsub_s32() {
        test_ari_s32(|i, j| vsub_s32(i, j), |a: i32, b: i32| -> i32 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsubq_s32() {
        testq_ari_s32(|i, j| vsubq_s32(i, j), |a: i32, b: i32| -> i32 { a - b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vsub_u8() {
        test_ari_u8(|i, j| vsub_u8(i, j), |a: u8, b: u8| -> u8 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsubq_u8() {
        testq_ari_u8(|i, j| vsubq_u8(i, j), |a: u8, b: u8| -> u8 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsub_u16() {
        test_ari_u16(|i, j| vsub_u16(i, j), |a: u16, b: u16| -> u16 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsubq_u16() {
        testq_ari_u16(|i, j| vsubq_u16(i, j), |a: u16, b: u16| -> u16 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsub_u32() {
        test_ari_u32(|i, j| vsub_u32(i, j), |a: u32, b: u32| -> u32 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsubq_u32() {
        testq_ari_u32(|i, j| vsubq_u32(i, j), |a: u32, b: u32| -> u32 { a - b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vsub_f32() {
        test_ari_f32(|i, j| vsub_f32(i, j), |a: f32, b: f32| -> f32 { a - b });
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vsubq_f32() {
        testq_ari_f32(|i, j| vsubq_f32(i, j), |a: f32, b: f32| -> f32 { a - b });
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vhsub_s8() {
        test_ari_s8(
            |i, j| vhsub_s8(i, j),
            |a: i8, b: i8| -> i8 { (((a as i16) - (b as i16)) / 2) as i8 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsubq_s8() {
        testq_ari_s8(
            |i, j| vhsubq_s8(i, j),
            |a: i8, b: i8| -> i8 { (((a as i16) - (b as i16)) / 2) as i8 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsub_s16() {
        test_ari_s16(
            |i, j| vhsub_s16(i, j),
            |a: i16, b: i16| -> i16 { (((a as i32) - (b as i32)) / 2) as i16 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsubq_s16() {
        testq_ari_s16(
            |i, j| vhsubq_s16(i, j),
            |a: i16, b: i16| -> i16 { (((a as i32) - (b as i32)) / 2) as i16 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsub_s32() {
        test_ari_s32(
            |i, j| vhsub_s32(i, j),
            |a: i32, b: i32| -> i32 { (((a as i64) - (b as i64)) / 2) as i32 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsubq_s32() {
        testq_ari_s32(
            |i, j| vhsubq_s32(i, j),
            |a: i32, b: i32| -> i32 { (((a as i64) - (b as i64)) / 2) as i32 },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vhsub_u8() {
        test_ari_u8(
            |i, j| vhsub_u8(i, j),
            |a: u8, b: u8| -> u8 { (((a as u16) - (b as u16)) / 2) as u8 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsubq_u8() {
        testq_ari_u8(
            |i, j| vhsubq_u8(i, j),
            |a: u8, b: u8| -> u8 { (((a as u16) - (b as u16)) / 2) as u8 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsub_u16() {
        test_ari_u16(
            |i, j| vhsub_u16(i, j),
            |a: u16, b: u16| -> u16 { (((a as u16) - (b as u16)) / 2) as u16 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsubq_u16() {
        testq_ari_u16(
            |i, j| vhsubq_u16(i, j),
            |a: u16, b: u16| -> u16 { (((a as u16) - (b as u16)) / 2) as u16 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsub_u32() {
        test_ari_u32(
            |i, j| vhsub_u32(i, j),
            |a: u32, b: u32| -> u32 { (((a as u64) - (b as u64)) / 2) as u32 },
        );
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vhsubq_u32() {
        testq_ari_u32(
            |i, j| vhsubq_u32(i, j),
            |a: u32, b: u32| -> u32 { (((a as u64) - (b as u64)) / 2) as u32 },
        );
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vabs_s8() {
        let a = i8x8::new(-1, 0, 1, -2, 0, 2, -128, 127);
        let r: i8x8 = transmute(vabs_s8(transmute(a)));
        let e = i8x8::new(1, 0, 1, 2, 0, 2, -128, 127);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabsq_s8() {
        let a = i8x16::new(-1, 0, 1, -2, 0, 2, -128, 127, -1, 0, 1, -2, 0, 2, -128, 127);
        let r: i8x16 = transmute(vabsq_s8(transmute(a)));
        let e = i8x16::new(1, 0, 1, 2, 0, 2, -128, 127, 1, 0, 1, 2, 0, 2, -128, 127);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabs_s16() {
        let a = i16x4::new(-1, 0, i16::MIN, i16::MAX);
        let r: i16x4 = transmute(vabs_s16(transmute(a)));
        let e = i16x4::new(1, 0, i16::MIN, i16::MAX);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabsq_s16() {
        let a = i16x8::new(-1, 0, i16::MIN, i16::MAX, -1, 0, i16::MIN, i16::MAX);
        let r: i16x8 = transmute(vabsq_s16(transmute(a)));
        let e = i16x8::new(1, 0, i16::MIN, i16::MAX, 1, 0, i16::MIN, i16::MAX);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabs_s32() {
        let a = i32x2::new(i32::MIN, i32::MIN + 1);
        let r: i32x2 = transmute(vabs_s32(transmute(a)));
        let e = i32x2::new(i32::MIN, i32::MAX);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabsq_s32() {
        let a = i32x4::new(i32::MIN, i32::MIN + 1, 0, -1);
        let r: i32x4 = transmute(vabsq_s32(transmute(a)));
        let e = i32x4::new(i32::MIN, i32::MAX, 0, 1);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vaba_s8() {
        let a = i8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = i8x8::new(1, 1, 1, 1, 1, 1, 1, 1);
        let c = i8x8::new(10, 9, 8, 7, 6, 5, 4, 3);
        let r: i8x8 = transmute(vaba_s8(transmute(a), transmute(b), transmute(c)));
        let e = i8x8::new(10, 10, 10, 10, 10, 10, 10, 10);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaba_s16() {
        let a = i16x4::new(1, 2, 3, 4);
        let b = i16x4::new(1, 1, 1, 1);
        let c = i16x4::new(10, 9, 8, 7);
        let r: i16x4 = transmute(vaba_s16(transmute(a), transmute(b), transmute(c)));
        let e = i16x4::new(10, 10, 10, 10);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaba_s32() {
        let a = i32x2::new(1, 2);
        let b = i32x2::new(1, 1);
        let c = i32x2::new(10, 9);
        let r: i32x2 = transmute(vaba_s32(transmute(a), transmute(b), transmute(c)));
        let e = i32x2::new(10, 10);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaba_u8() {
        let a = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = u8x8::new(1, 1, 1, 1, 1, 1, 1, 1);
        let c = u8x8::new(10, 9, 8, 7, 6, 5, 4, 3);
        let r: u8x8 = transmute(vaba_u8(transmute(a), transmute(b), transmute(c)));
        let e = u8x8::new(10, 10, 10, 10, 10, 10, 10, 10);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaba_u16() {
        let a = u16x4::new(1, 2, 3, 4);
        let b = u16x4::new(1, 1, 1, 1);
        let c = u16x4::new(10, 9, 8, 7);
        let r: u16x4 = transmute(vaba_u16(transmute(a), transmute(b), transmute(c)));
        let e = u16x4::new(10, 10, 10, 10);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vaba_u32() {
        let a = u32x2::new(1, 2);
        let b = u32x2::new(1, 1);
        let c = u32x2::new(10, 9);
        let r: u32x2 = transmute(vaba_u32(transmute(a), transmute(b), transmute(c)));
        let e = u32x2::new(10, 10);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabaq_s8() {
        let a = i8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 8, 7, 6, 5, 4, 3, 2);
        let b = i8x16::new(1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1);
        let c = i8x16::new(10, 9, 8, 7, 6, 5, 4, 3, 12, 13, 14, 15, 16, 17, 18, 19);
        let r: i8x16 = transmute(vabaq_s8(transmute(a), transmute(b), transmute(c)));
        let e = i8x16::new(
            10, 10, 10, 10, 10, 10, 10, 10, 20, 20, 20, 20, 20, 20, 20, 20,
        );
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabaq_s16() {
        let a = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = i16x8::new(1, 1, 1, 1, 1, 1, 1, 1);
        let c = i16x8::new(10, 9, 8, 7, 6, 5, 4, 3);
        let r: i16x8 = transmute(vabaq_s16(transmute(a), transmute(b), transmute(c)));
        let e = i16x8::new(10, 10, 10, 10, 10, 10, 10, 10);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabaq_s32() {
        let a = i32x4::new(1, 2, 3, 4);
        let b = i32x4::new(1, 1, 1, 1);
        let c = i32x4::new(10, 9, 8, 7);
        let r: i32x4 = transmute(vabaq_s32(transmute(a), transmute(b), transmute(c)));
        let e = i32x4::new(10, 10, 10, 10);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabaq_u8() {
        let a = u8x16::new(1, 2, 3, 4, 5, 6, 7, 8, 9, 8, 7, 6, 5, 4, 3, 2);
        let b = u8x16::new(1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1);
        let c = u8x16::new(10, 9, 8, 7, 6, 5, 4, 3, 12, 13, 14, 15, 16, 17, 18, 19);
        let r: u8x16 = transmute(vabaq_u8(transmute(a), transmute(b), transmute(c)));
        let e = u8x16::new(
            10, 10, 10, 10, 10, 10, 10, 10, 20, 20, 20, 20, 20, 20, 20, 20,
        );
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabaq_u16() {
        let a = u16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = u16x8::new(1, 1, 1, 1, 1, 1, 1, 1);
        let c = u16x8::new(10, 9, 8, 7, 6, 5, 4, 3);
        let r: u16x8 = transmute(vabaq_u16(transmute(a), transmute(b), transmute(c)));
        let e = u16x8::new(10, 10, 10, 10, 10, 10, 10, 10);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vabaq_u32() {
        let a = u32x4::new(1, 2, 3, 4);
        let b = u32x4::new(1, 1, 1, 1);
        let c = u32x4::new(10, 9, 8, 7);
        let r: u32x4 = transmute(vabaq_u32(transmute(a), transmute(b), transmute(c)));
        let e = u32x4::new(10, 10, 10, 10);
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon")]
    unsafe fn test_vpadd_s16() {
        let a = i16x4::new(1, 2, 3, 4);
        let b = i16x4::new(0, -1, -2, -3);
        let r: i16x4 = transmute(vpadd_s16(transmute(a), transmute(b)));
        let e = i16x4::new(3, 7, -1, -5);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpadd_s32() {
        let a = i32x2::new(1, 2);
        let b = i32x2::new(0, -1);
        let r: i32x2 = transmute(vpadd_s32(transmute(a), transmute(b)));
        let e = i32x2::new(3, -1);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpadd_s8() {
        let a = i8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = i8x8::new(0, -1, -2, -3, -4, -5, -6, -7);
        let r: i8x8 = transmute(vpadd_s8(transmute(a), transmute(b)));
        let e = i8x8::new(3, 7, 11, 15, -1, -5, -9, -13);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpadd_u16() {
        let a = u16x4::new(1, 2, 3, 4);
        let b = u16x4::new(30, 31, 32, 33);
        let r: u16x4 = transmute(vpadd_u16(transmute(a), transmute(b)));
        let e = u16x4::new(3, 7, 61, 65);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpadd_u32() {
        let a = u32x2::new(1, 2);
        let b = u32x2::new(30, 31);
        let r: u32x2 = transmute(vpadd_u32(transmute(a), transmute(b)));
        let e = u32x2::new(3, 61);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vpadd_u8() {
        let a = u8x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let b = u8x8::new(30, 31, 32, 33, 34, 35, 36, 37);
        let r: u8x8 = transmute(vpadd_u8(transmute(a), transmute(b)));
        let e = u8x8::new(3, 7, 11, 15, 61, 65, 69, 73);
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcnt_s8() {
        let a: i8x8 = transmute(u8x8::new(
            0b11001000, 0b11111111, 0b00000000, 0b11011111, 0b10000001, 0b10101001, 0b00001000,
            0b00111111,
        ));
        let e = i8x8::new(3, 8, 0, 7, 2, 4, 1, 6);
        let r: i8x8 = transmute(vcnt_s8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcntq_s8() {
        let a: i8x16 = transmute(u8x16::new(
            0b11001000, 0b11111111, 0b00000000, 0b11011111, 0b10000001, 0b10101001, 0b00001000,
            0b00111111, 0b11101110, 0b00000000, 0b11111111, 0b00100001, 0b11111111, 0b10010111,
            0b11100000, 0b00010000,
        ));
        let e = i8x16::new(3, 8, 0, 7, 2, 4, 1, 6, 6, 0, 8, 2, 8, 5, 3, 1);
        let r: i8x16 = transmute(vcntq_s8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcnt_u8() {
        let a = u8x8::new(
            0b11001000, 0b11111111, 0b00000000, 0b11011111, 0b10000001, 0b10101001, 0b00001000,
            0b00111111,
        );
        let e = u8x8::new(3, 8, 0, 7, 2, 4, 1, 6);
        let r: u8x8 = transmute(vcnt_u8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcntq_u8() {
        let a = u8x16::new(
            0b11001000, 0b11111111, 0b00000000, 0b11011111, 0b10000001, 0b10101001, 0b00001000,
            0b00111111, 0b11101110, 0b00000000, 0b11111111, 0b00100001, 0b11111111, 0b10010111,
            0b11100000, 0b00010000,
        );
        let e = u8x16::new(3, 8, 0, 7, 2, 4, 1, 6, 6, 0, 8, 2, 8, 5, 3, 1);
        let r: u8x16 = transmute(vcntq_u8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcnt_p8() {
        let a = u8x8::new(
            0b11001000, 0b11111111, 0b00000000, 0b11011111, 0b10000001, 0b10101001, 0b00001000,
            0b00111111,
        );
        let e = u8x8::new(3, 8, 0, 7, 2, 4, 1, 6);
        let r: u8x8 = transmute(vcnt_p8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vcntq_p8() {
        let a = u8x16::new(
            0b11001000, 0b11111111, 0b00000000, 0b11011111, 0b10000001, 0b10101001, 0b00001000,
            0b00111111, 0b11101110, 0b00000000, 0b11111111, 0b00100001, 0b11111111, 0b10010111,
            0b11100000, 0b00010000,
        );
        let e = u8x16::new(3, 8, 0, 7, 2, 4, 1, 6, 6, 0, 8, 2, 8, 5, 3, 1);
        let r: u8x16 = transmute(vcntq_p8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev16_s8() {
        let a = i8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = i8x8::new(1, 0, 3, 2, 5, 4, 7, 6);
        let e: i8x8 = transmute(vrev16_s8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev16q_s8() {
        let a = i8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = i8x16::new(1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14);
        let e: i8x16 = transmute(vrev16q_s8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev16_u8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = u8x8::new(1, 0, 3, 2, 5, 4, 7, 6);
        let e: u8x8 = transmute(vrev16_u8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev16q_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = u8x16::new(1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14);
        let e: u8x16 = transmute(vrev16q_u8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev16_p8() {
        let a = i8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = i8x8::new(1, 0, 3, 2, 5, 4, 7, 6);
        let e: i8x8 = transmute(vrev16_p8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev16q_p8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = u8x16::new(1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14);
        let e: u8x16 = transmute(vrev16q_p8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32_s8() {
        let a = i8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = i8x8::new(3, 2, 1, 0, 7, 6, 5, 4);
        let e: i8x8 = transmute(vrev32_s8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32q_s8() {
        let a = i8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = i8x16::new(3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12);
        let e: i8x16 = transmute(vrev32q_s8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32_u8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = u8x8::new(3, 2, 1, 0, 7, 6, 5, 4);
        let e: u8x8 = transmute(vrev32_u8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32q_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = u8x16::new(3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12);
        let e: u8x16 = transmute(vrev32q_u8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32_s16() {
        let a = i16x4::new(0, 1, 2, 3);
        let r = i16x4::new(1, 0, 3, 2);
        let e: i16x4 = transmute(vrev32_s16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32q_s16() {
        let a = i16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = i16x8::new(1, 0, 3, 2, 5, 4, 7, 6);
        let e: i16x8 = transmute(vrev32q_s16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32_p16() {
        let a = i16x4::new(0, 1, 2, 3);
        let r = i16x4::new(1, 0, 3, 2);
        let e: i16x4 = transmute(vrev32_p16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32q_p16() {
        let a = i16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = i16x8::new(1, 0, 3, 2, 5, 4, 7, 6);
        let e: i16x8 = transmute(vrev32q_p16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32_u16() {
        let a = u16x4::new(0, 1, 2, 3);
        let r = u16x4::new(1, 0, 3, 2);
        let e: u16x4 = transmute(vrev32_u16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32q_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = u16x8::new(1, 0, 3, 2, 5, 4, 7, 6);
        let e: u16x8 = transmute(vrev32q_u16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32_p8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = u8x8::new(3, 2, 1, 0, 7, 6, 5, 4);
        let e: u8x8 = transmute(vrev32_p8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev32q_p8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = u8x16::new(3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12);
        let e: u8x16 = transmute(vrev32q_p8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64_s8() {
        let a = i8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = i8x8::new(7, 6, 5, 4, 3, 2, 1, 0);
        let e: i8x8 = transmute(vrev64_s8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64q_s8() {
        let a = i8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = i8x16::new(7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8);
        let e: i8x16 = transmute(vrev64q_s8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64_s16() {
        let a = i16x4::new(0, 1, 2, 3);
        let r = i16x4::new(3, 2, 1, 0);
        let e: i16x4 = transmute(vrev64_s16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64q_s16() {
        let a = i16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = i16x8::new(3, 2, 1, 0, 7, 6, 5, 4);
        let e: i16x8 = transmute(vrev64q_s16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64_s32() {
        let a = i32x2::new(0, 1);
        let r = i32x2::new(1, 0);
        let e: i32x2 = transmute(vrev64_s32(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64q_s32() {
        let a = i32x4::new(0, 1, 2, 3);
        let r = i32x4::new(1, 0, 3, 2);
        let e: i32x4 = transmute(vrev64q_s32(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64_u8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = u8x8::new(7, 6, 5, 4, 3, 2, 1, 0);
        let e: u8x8 = transmute(vrev64_u8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64q_u8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = u8x16::new(7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8);
        let e: u8x16 = transmute(vrev64q_u8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64_u16() {
        let a = u16x4::new(0, 1, 2, 3);
        let r = u16x4::new(3, 2, 1, 0);
        let e: u16x4 = transmute(vrev64_u16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64q_u16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = u16x8::new(3, 2, 1, 0, 7, 6, 5, 4);
        let e: u16x8 = transmute(vrev64q_u16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64_u32() {
        let a = u32x2::new(0, 1);
        let r = u32x2::new(1, 0);
        let e: u32x2 = transmute(vrev64_u32(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64q_u32() {
        let a = u32x4::new(0, 1, 2, 3);
        let r = u32x4::new(1, 0, 3, 2);
        let e: u32x4 = transmute(vrev64q_u32(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64_f32() {
        let a = f32x2::new(1.0, 2.0);
        let r = f32x2::new(2.0, 1.0);
        let e: f32x2 = transmute(vrev64_f32(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64q_f32() {
        let a = f32x4::new(1.0, 2.0, -2.0, -1.0);
        let r = f32x4::new(2.0, 1.0, -1.0, -2.0);
        let e: f32x4 = transmute(vrev64q_f32(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64_p8() {
        let a = u8x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = u8x8::new(7, 6, 5, 4, 3, 2, 1, 0);
        let e: u8x8 = transmute(vrev64_p8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64q_p8() {
        let a = u8x16::new(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let r = u8x16::new(7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8);
        let e: u8x16 = transmute(vrev64q_p8(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64_p16() {
        let a = u16x4::new(0, 1, 2, 3);
        let r = u16x4::new(3, 2, 1, 0);
        let e: u16x4 = transmute(vrev64_p16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon")]
    unsafe fn test_vrev64q_p16() {
        let a = u16x8::new(0, 1, 2, 3, 4, 5, 6, 7);
        let r = u16x8::new(3, 2, 1, 0, 7, 6, 5, 4);
        let e: u16x8 = transmute(vrev64q_p16(transmute(a)));
        assert_eq!(r, e);
    }
    #[simd_test(enable = "neon,i8mm")]
    unsafe fn test_vmmlaq_s32() {
        let a = i32x4::new(1, 3, 4, -0x10000);
        let b = i8x16::new(1, 21, 31, 14, 5, 6, -128, 8, 9, 13, 15, 12, 13, -1, 20, 16);
        let c = i8x16::new(12, 22, 3, 4, -1, 56, 7, 8, 91, 10, -128, 15, 13, 14, 17, 16);
        let e = i32x4::new(123, -5353, 690, -65576);
        let r: i32x4 = transmute(vmmlaq_s32(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon,i8mm")]
    unsafe fn test_vmmlaq_u32() {
        let a = u32x4::new(1, 3, 4, 0xffff0000);
        let b = u8x16::new(1, 21, 31, 14, 5, 6, 128, 8, 9, 13, 15, 12, 13, 255, 20, 16);
        let c = u8x16::new(12, 22, 3, 4, 255, 56, 7, 8, 91, 10, 128, 15, 13, 14, 17, 16);
        let e = u32x4::new(3195, 6935, 18354, 4294909144);
        let r: u32x4 = transmute(vmmlaq_u32(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }

    #[simd_test(enable = "neon,i8mm")]
    unsafe fn test_vusmmlaq_s32() {
        let a = i32x4::new(1, 3, 4, -0x10000);
        let b = u8x16::new(1, 21, 31, 14, 5, 6, 128, 8, 9, 13, 15, 12, 13, 255, 20, 16);
        let c = i8x16::new(12, 22, 3, 4, -1, 56, 7, 8, 91, 10, -128, 15, 13, 14, 17, 16);
        let e = i32x4::new(1915, -1001, 15026, -61992);
        let r: i32x4 = transmute(vusmmlaq_s32(transmute(a), transmute(b), transmute(c)));
        assert_eq!(r, e);
    }

    macro_rules! test_vcombine {
        ($test_id:ident => $fn_id:ident ([$($a:expr),*], [$($b:expr),*])) => {
            #[allow(unused_assignments)]
            #[simd_test(enable = "neon")]
            unsafe fn $test_id() {
                let a = [$($a),*];
                let b = [$($b),*];
                let e = [$($a),* $(, $b)*];
                let c = $fn_id(transmute(a), transmute(b));
                let mut d = e;
                d = transmute(c);
                assert_eq!(d, e);
            }
        }
    }

    test_vcombine!(test_vcombine_s8 => vcombine_s8([3_i8, -4, 5, -6, 7, 8, 9, 10], [13_i8, -14, 15, -16, 17, 18, 19, 110]));
    test_vcombine!(test_vcombine_u8 => vcombine_u8([3_u8, 4, 5, 6, 7, 8, 9, 10], [13_u8, 14, 15, 16, 17, 18, 19, 110]));
    test_vcombine!(test_vcombine_p8 => vcombine_p8([3_u8, 4, 5, 6, 7, 8, 9, 10], [13_u8, 14, 15, 16, 17, 18, 19, 110]));

    test_vcombine!(test_vcombine_s16 => vcombine_s16([3_i16, -4, 5, -6], [13_i16, -14, 15, -16]));
    test_vcombine!(test_vcombine_u16 => vcombine_u16([3_u16, 4, 5, 6], [13_u16, 14, 15, 16]));
    test_vcombine!(test_vcombine_p16 => vcombine_p16([3_u16, 4, 5, 6], [13_u16, 14, 15, 16]));
    // FIXME: 16-bit floats
    // test_vcombine!(test_vcombine_f16 => vcombine_f16([3_f16, 4., 5., 6.],
    // [13_f16, 14., 15., 16.]));

    test_vcombine!(test_vcombine_s32 => vcombine_s32([3_i32, -4], [13_i32, -14]));
    test_vcombine!(test_vcombine_u32 => vcombine_u32([3_u32, 4], [13_u32, 14]));
    // note: poly32x4 does not exist, and neither does vcombine_p32
    test_vcombine!(test_vcombine_f32 => vcombine_f32([3_f32, -4.], [13_f32, -14.]));

    test_vcombine!(test_vcombine_s64 => vcombine_s64([-3_i64], [13_i64]));
    test_vcombine!(test_vcombine_u64 => vcombine_u64([3_u64], [13_u64]));
    test_vcombine!(test_vcombine_p64 => vcombine_p64([3_u64], [13_u64]));
    #[cfg(target_arch = "aarch64")]
    test_vcombine!(test_vcombine_f64 => vcombine_f64([-3_f64], [13_f64]));
}

#[cfg(all(test, target_arch = "arm", target_endian = "little"))]
mod table_lookup_tests;

#[cfg(all(test, target_arch = "arm"))]
mod shift_and_insert_tests;

#[cfg(all(test, target_arch = "arm"))]
mod load_tests;

#[cfg(all(test, target_arch = "arm"))]
mod store_tests;
