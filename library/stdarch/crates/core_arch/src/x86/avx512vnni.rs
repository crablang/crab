use crate::{
    core_arch::{simd::*, simd_llvm::*, x86::*},
    mem::transmute,
};

#[cfg(test)]
use stdarch_test::assert_instr;

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_dpwssd_epi32&expand=2219)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssd))]
pub unsafe fn _mm512_dpwssd_epi32(src: __m512i, a: __m512i, b: __m512i) -> __m512i {
    transmute(vpdpwssd(src.as_i32x16(), a.as_i32x16(), b.as_i32x16()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_mask_dpwssd_epi32&expand=2220)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssd))]
pub unsafe fn _mm512_mask_dpwssd_epi32(
    src: __m512i,
    k: __mmask16,
    a: __m512i,
    b: __m512i,
) -> __m512i {
    let r = _mm512_dpwssd_epi32(src, a, b).as_i32x16();
    transmute(simd_select_bitmask(k, r, src.as_i32x16()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_maskz_dpwssd_epi32&expand=2221)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssd))]
pub unsafe fn _mm512_maskz_dpwssd_epi32(
    k: __mmask16,
    src: __m512i,
    a: __m512i,
    b: __m512i,
) -> __m512i {
    let r = _mm512_dpwssd_epi32(src, a, b).as_i32x16();
    let zero = _mm512_setzero_si512().as_i32x16();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_dpwssd_epi32&expand=2216)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssd))]
pub unsafe fn _mm256_dpwssd_epi32(src: __m256i, a: __m256i, b: __m256i) -> __m256i {
    transmute(vpdpwssd256(src.as_i32x8(), a.as_i32x8(), b.as_i32x8()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_mask_dpwssd_epi32&expand=2217)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssd))]
pub unsafe fn _mm256_mask_dpwssd_epi32(
    src: __m256i,
    k: __mmask8,
    a: __m256i,
    b: __m256i,
) -> __m256i {
    let r = _mm256_dpwssd_epi32(src, a, b).as_i32x8();
    transmute(simd_select_bitmask(k, r, src.as_i32x8()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_maskz_dpwssd_epi32&expand=2218)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssd))]
pub unsafe fn _mm256_maskz_dpwssd_epi32(
    k: __mmask8,
    src: __m256i,
    a: __m256i,
    b: __m256i,
) -> __m256i {
    let r = _mm256_dpwssd_epi32(src, a, b).as_i32x8();
    let zero = _mm256_setzero_si256().as_i32x8();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_dpwssd_epi32&expand=2213)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssd))]
pub unsafe fn _mm_dpwssd_epi32(src: __m128i, a: __m128i, b: __m128i) -> __m128i {
    transmute(vpdpwssd128(src.as_i32x4(), a.as_i32x4(), b.as_i32x4()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_mask_dpwssd_epi32&expand=2214)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssd))]
pub unsafe fn _mm_mask_dpwssd_epi32(src: __m128i, k: __mmask8, a: __m128i, b: __m128i) -> __m128i {
    let r = _mm_dpwssd_epi32(src, a, b).as_i32x4();
    transmute(simd_select_bitmask(k, r, src.as_i32x4()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_maskz_dpwssd_epi32&expand=2215)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssd))]
pub unsafe fn _mm_maskz_dpwssd_epi32(k: __mmask8, src: __m128i, a: __m128i, b: __m128i) -> __m128i {
    let r = _mm_dpwssd_epi32(src, a, b).as_i32x4();
    let zero = _mm_setzero_si128().as_i32x4();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_dpwssds_epi32&expand=2228)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssds))]
pub unsafe fn _mm512_dpwssds_epi32(src: __m512i, a: __m512i, b: __m512i) -> __m512i {
    transmute(vpdpwssds(src.as_i32x16(), a.as_i32x16(), b.as_i32x16()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_mask_dpwssds_epi32&expand=2229)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssds))]
pub unsafe fn _mm512_mask_dpwssds_epi32(
    src: __m512i,
    k: __mmask16,
    a: __m512i,
    b: __m512i,
) -> __m512i {
    let r = _mm512_dpwssds_epi32(src, a, b).as_i32x16();
    transmute(simd_select_bitmask(k, r, src.as_i32x16()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_maskz_dpwssds_epi32&expand=2230)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssds))]
pub unsafe fn _mm512_maskz_dpwssds_epi32(
    k: __mmask16,
    src: __m512i,
    a: __m512i,
    b: __m512i,
) -> __m512i {
    let r = _mm512_dpwssds_epi32(src, a, b).as_i32x16();
    let zero = _mm512_setzero_si512().as_i32x16();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_dpwssds_epi32&expand=2225)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssds))]
pub unsafe fn _mm256_dpwssds_epi32(src: __m256i, a: __m256i, b: __m256i) -> __m256i {
    transmute(vpdpwssds256(src.as_i32x8(), a.as_i32x8(), b.as_i32x8()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_mask_dpwssds_epi32&expand=2226)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssds))]
pub unsafe fn _mm256_mask_dpwssds_epi32(
    src: __m256i,
    k: __mmask8,
    a: __m256i,
    b: __m256i,
) -> __m256i {
    let r = _mm256_dpwssds_epi32(src, a, b).as_i32x8();
    transmute(simd_select_bitmask(k, r, src.as_i32x8()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_maskz_dpwssds_epi32&expand=2227)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssds))]
pub unsafe fn _mm256_maskz_dpwssds_epi32(
    k: __mmask8,
    src: __m256i,
    a: __m256i,
    b: __m256i,
) -> __m256i {
    let r = _mm256_dpwssds_epi32(src, a, b).as_i32x8();
    let zero = _mm256_setzero_si256().as_i32x8();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_dpwssds_epi32&expand=2222)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssds))]
pub unsafe fn _mm_dpwssds_epi32(src: __m128i, a: __m128i, b: __m128i) -> __m128i {
    transmute(vpdpwssds128(src.as_i32x4(), a.as_i32x4(), b.as_i32x4()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_mask_dpwssds_epi32&expand=2223)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssds))]
pub unsafe fn _mm_mask_dpwssds_epi32(src: __m128i, k: __mmask8, a: __m128i, b: __m128i) -> __m128i {
    let r = _mm_dpwssds_epi32(src, a, b).as_i32x4();
    transmute(simd_select_bitmask(k, r, src.as_i32x4()))
}

/// Multiply groups of 2 adjacent pairs of signed 16-bit integers in a with corresponding 16-bit integers in b, producing 2 intermediate signed 32-bit results. Sum these 2 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_maskz_dpwssds_epi32&expand=2224)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpwssds))]
pub unsafe fn _mm_maskz_dpwssds_epi32(
    k: __mmask8,
    src: __m128i,
    a: __m128i,
    b: __m128i,
) -> __m128i {
    let r = _mm_dpwssds_epi32(src, a, b).as_i32x4();
    let zero = _mm_setzero_si128().as_i32x4();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_dpbusd_epi32&expand=2201)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusd))]
pub unsafe fn _mm512_dpbusd_epi32(src: __m512i, a: __m512i, b: __m512i) -> __m512i {
    transmute(vpdpbusd(src.as_i32x16(), a.as_i32x16(), b.as_i32x16()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_mask_dpbusd_epi32&expand=2202)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusd))]
pub unsafe fn _mm512_mask_dpbusd_epi32(
    src: __m512i,
    k: __mmask16,
    a: __m512i,
    b: __m512i,
) -> __m512i {
    let r = _mm512_dpbusd_epi32(src, a, b).as_i32x16();
    transmute(simd_select_bitmask(k, r, src.as_i32x16()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_maskz_dpbusd_epi32&expand=2203)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusd))]
pub unsafe fn _mm512_maskz_dpbusd_epi32(
    k: __mmask16,
    src: __m512i,
    a: __m512i,
    b: __m512i,
) -> __m512i {
    let r = _mm512_dpbusd_epi32(src, a, b).as_i32x16();
    let zero = _mm512_setzero_si512().as_i32x16();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_dpbusd_epi32&expand=2198)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusd))]
pub unsafe fn _mm256_dpbusd_epi32(src: __m256i, a: __m256i, b: __m256i) -> __m256i {
    transmute(vpdpbusd256(src.as_i32x8(), a.as_i32x8(), b.as_i32x8()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_mask_dpbusd_epi32&expand=2199)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusd))]
pub unsafe fn _mm256_mask_dpbusd_epi32(
    src: __m256i,
    k: __mmask8,
    a: __m256i,
    b: __m256i,
) -> __m256i {
    let r = _mm256_dpbusd_epi32(src, a, b).as_i32x8();
    transmute(simd_select_bitmask(k, r, src.as_i32x8()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_maskz_dpbusd_epi32&expand=2200)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusd))]
pub unsafe fn _mm256_maskz_dpbusd_epi32(
    k: __mmask8,
    src: __m256i,
    a: __m256i,
    b: __m256i,
) -> __m256i {
    let r = _mm256_dpbusd_epi32(src, a, b).as_i32x8();
    let zero = _mm256_setzero_si256().as_i32x8();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_dpbusd_epi32&expand=2195)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusd))]
pub unsafe fn _mm_dpbusd_epi32(src: __m128i, a: __m128i, b: __m128i) -> __m128i {
    transmute(vpdpbusd128(src.as_i32x4(), a.as_i32x4(), b.as_i32x4()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_mask_dpbusd_epi32&expand=2196)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusd))]
pub unsafe fn _mm_mask_dpbusd_epi32(src: __m128i, k: __mmask8, a: __m128i, b: __m128i) -> __m128i {
    let r = _mm_dpbusd_epi32(src, a, b).as_i32x4();
    transmute(simd_select_bitmask(k, r, src.as_i32x4()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_maskz_dpbusd_epi32&expand=2197)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusd))]
pub unsafe fn _mm_maskz_dpbusd_epi32(k: __mmask8, src: __m128i, a: __m128i, b: __m128i) -> __m128i {
    let r = _mm_dpbusd_epi32(src, a, b).as_i32x4();
    let zero = _mm_setzero_si128().as_i32x4();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_dpbusds_epi32&expand=2210)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusds))]
pub unsafe fn _mm512_dpbusds_epi32(src: __m512i, a: __m512i, b: __m512i) -> __m512i {
    transmute(vpdpbusds(src.as_i32x16(), a.as_i32x16(), b.as_i32x16()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_mask_dpbusds_epi32&expand=2211)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusds))]
pub unsafe fn _mm512_mask_dpbusds_epi32(
    src: __m512i,
    k: __mmask16,
    a: __m512i,
    b: __m512i,
) -> __m512i {
    let r = _mm512_dpbusds_epi32(src, a, b).as_i32x16();
    transmute(simd_select_bitmask(k, r, src.as_i32x16()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm512_maskz_dpbusds_epi32&expand=2212)
#[inline]
#[target_feature(enable = "avx512vnni")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusds))]
pub unsafe fn _mm512_maskz_dpbusds_epi32(
    k: __mmask16,
    src: __m512i,
    a: __m512i,
    b: __m512i,
) -> __m512i {
    let r = _mm512_dpbusds_epi32(src, a, b).as_i32x16();
    let zero = _mm512_setzero_si512().as_i32x16();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_dpbusds_epi32&expand=2207)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusds))]
pub unsafe fn _mm256_dpbusds_epi32(src: __m256i, a: __m256i, b: __m256i) -> __m256i {
    transmute(vpdpbusds256(src.as_i32x8(), a.as_i32x8(), b.as_i32x8()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_mask_dpbusds_epi32&expand=2208)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusds))]
pub unsafe fn _mm256_mask_dpbusds_epi32(
    src: __m256i,
    k: __mmask8,
    a: __m256i,
    b: __m256i,
) -> __m256i {
    let r = _mm256_dpbusds_epi32(src, a, b).as_i32x8();
    transmute(simd_select_bitmask(k, r, src.as_i32x8()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm256_maskz_dpbusds_epi32&expand=2209)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusds))]
pub unsafe fn _mm256_maskz_dpbusds_epi32(
    k: __mmask8,
    src: __m256i,
    a: __m256i,
    b: __m256i,
) -> __m256i {
    let r = _mm256_dpbusds_epi32(src, a, b).as_i32x8();
    let zero = _mm256_setzero_si256().as_i32x8();
    transmute(simd_select_bitmask(k, r, zero))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst.
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_dpbusds_epi32&expand=2204)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusds))]
pub unsafe fn _mm_dpbusds_epi32(src: __m128i, a: __m128i, b: __m128i) -> __m128i {
    transmute(vpdpbusds128(src.as_i32x4(), a.as_i32x4(), b.as_i32x4()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using writemask k (elements are copied from src when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_mask_dpbusds_epi32&expand=2205)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusds))]
pub unsafe fn _mm_mask_dpbusds_epi32(src: __m128i, k: __mmask8, a: __m128i, b: __m128i) -> __m128i {
    let r = _mm_dpbusds_epi32(src, a, b).as_i32x4();
    transmute(simd_select_bitmask(k, r, src.as_i32x4()))
}

/// Multiply groups of 4 adjacent pairs of unsigned 8-bit integers in a with corresponding signed 8-bit integers in b, producing 4 intermediate signed 16-bit results. Sum these 4 results with the corresponding 32-bit integer in src using signed saturation, and store the packed 32-bit results in dst using zeromask k (elements are zeroed out when the corresponding mask bit is not set).
///
/// [Intel's documentation](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#text=_mm_maskz_dpbusds_epi32&expand=2206)
#[inline]
#[target_feature(enable = "avx512vnni,avx512vl")]
#[unstable(feature = "stdarch_x86_avx512", issue = "111137")]
#[cfg_attr(test, assert_instr(vpdpbusds))]
pub unsafe fn _mm_maskz_dpbusds_epi32(
    k: __mmask8,
    src: __m128i,
    a: __m128i,
    b: __m128i,
) -> __m128i {
    let r = _mm_dpbusds_epi32(src, a, b).as_i32x4();
    let zero = _mm_setzero_si128().as_i32x4();
    transmute(simd_select_bitmask(k, r, zero))
}

#[allow(improper_ctypes)]
extern "C" {
    #[link_name = "llvm.x86.avx512.vpdpwssd.512"]
    fn vpdpwssd(src: i32x16, a: i32x16, b: i32x16) -> i32x16;
    #[link_name = "llvm.x86.avx512.vpdpwssd.256"]
    fn vpdpwssd256(src: i32x8, a: i32x8, b: i32x8) -> i32x8;
    #[link_name = "llvm.x86.avx512.vpdpwssd.128"]
    fn vpdpwssd128(src: i32x4, a: i32x4, b: i32x4) -> i32x4;

    #[link_name = "llvm.x86.avx512.vpdpwssds.512"]
    fn vpdpwssds(src: i32x16, a: i32x16, b: i32x16) -> i32x16;
    #[link_name = "llvm.x86.avx512.vpdpwssds.256"]
    fn vpdpwssds256(src: i32x8, a: i32x8, b: i32x8) -> i32x8;
    #[link_name = "llvm.x86.avx512.vpdpwssds.128"]
    fn vpdpwssds128(src: i32x4, a: i32x4, b: i32x4) -> i32x4;

    #[link_name = "llvm.x86.avx512.vpdpbusd.512"]
    fn vpdpbusd(src: i32x16, a: i32x16, b: i32x16) -> i32x16;
    #[link_name = "llvm.x86.avx512.vpdpbusd.256"]
    fn vpdpbusd256(src: i32x8, a: i32x8, b: i32x8) -> i32x8;
    #[link_name = "llvm.x86.avx512.vpdpbusd.128"]
    fn vpdpbusd128(src: i32x4, a: i32x4, b: i32x4) -> i32x4;

    #[link_name = "llvm.x86.avx512.vpdpbusds.512"]
    fn vpdpbusds(src: i32x16, a: i32x16, b: i32x16) -> i32x16;
    #[link_name = "llvm.x86.avx512.vpdpbusds.256"]
    fn vpdpbusds256(src: i32x8, a: i32x8, b: i32x8) -> i32x8;
    #[link_name = "llvm.x86.avx512.vpdpbusds.128"]
    fn vpdpbusds128(src: i32x4, a: i32x4, b: i32x4) -> i32x4;
}

#[cfg(test)]
mod tests {

    use crate::core_arch::x86::*;
    use stdarch_test::simd_test;

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_dpwssd_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm512_dpwssd_epi32(src, a, b);
        let e = _mm512_set1_epi32(3);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_mask_dpwssd_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm512_mask_dpwssd_epi32(src, 0b00000000_00000000, a, b);
        assert_eq_m512i(r, src);
        let r = _mm512_mask_dpwssd_epi32(src, 0b11111111_11111111, a, b);
        let e = _mm512_set1_epi32(3);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_maskz_dpwssd_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm512_maskz_dpwssd_epi32(0b00000000_00000000, src, a, b);
        assert_eq_m512i(r, _mm512_setzero_si512());
        let r = _mm512_maskz_dpwssd_epi32(0b11111111_11111111, src, a, b);
        let e = _mm512_set1_epi32(3);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_dpwssd_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm256_dpwssd_epi32(src, a, b);
        let e = _mm256_set1_epi32(3);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_mask_dpwssd_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm256_mask_dpwssd_epi32(src, 0b00000000, a, b);
        assert_eq_m256i(r, src);
        let r = _mm256_mask_dpwssd_epi32(src, 0b11111111, a, b);
        let e = _mm256_set1_epi32(3);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_maskz_dpwssd_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm256_maskz_dpwssd_epi32(0b00000000, src, a, b);
        assert_eq_m256i(r, _mm256_setzero_si256());
        let r = _mm256_maskz_dpwssd_epi32(0b11111111, src, a, b);
        let e = _mm256_set1_epi32(3);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_dpwssd_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm_dpwssd_epi32(src, a, b);
        let e = _mm_set1_epi32(3);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_mask_dpwssd_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm_mask_dpwssd_epi32(src, 0b00000000, a, b);
        assert_eq_m128i(r, src);
        let r = _mm_mask_dpwssd_epi32(src, 0b00001111, a, b);
        let e = _mm_set1_epi32(3);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_maskz_dpwssd_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm_maskz_dpwssd_epi32(0b00000000, src, a, b);
        assert_eq_m128i(r, _mm_setzero_si128());
        let r = _mm_maskz_dpwssd_epi32(0b00001111, src, a, b);
        let e = _mm_set1_epi32(3);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_dpwssds_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm512_dpwssds_epi32(src, a, b);
        let e = _mm512_set1_epi32(3);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_mask_dpwssds_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm512_mask_dpwssds_epi32(src, 0b00000000_00000000, a, b);
        assert_eq_m512i(r, src);
        let r = _mm512_mask_dpwssds_epi32(src, 0b11111111_11111111, a, b);
        let e = _mm512_set1_epi32(3);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_maskz_dpwssds_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm512_maskz_dpwssds_epi32(0b00000000_00000000, src, a, b);
        assert_eq_m512i(r, _mm512_setzero_si512());
        let r = _mm512_maskz_dpwssds_epi32(0b11111111_11111111, src, a, b);
        let e = _mm512_set1_epi32(3);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_dpwssds_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm256_dpwssds_epi32(src, a, b);
        let e = _mm256_set1_epi32(3);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_mask_dpwssds_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm256_mask_dpwssds_epi32(src, 0b00000000, a, b);
        assert_eq_m256i(r, src);
        let r = _mm256_mask_dpwssds_epi32(src, 0b11111111, a, b);
        let e = _mm256_set1_epi32(3);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_maskz_dpwssds_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm256_maskz_dpwssds_epi32(0b00000000, src, a, b);
        assert_eq_m256i(r, _mm256_setzero_si256());
        let r = _mm256_maskz_dpwssds_epi32(0b11111111, src, a, b);
        let e = _mm256_set1_epi32(3);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_dpwssds_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm_dpwssds_epi32(src, a, b);
        let e = _mm_set1_epi32(3);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_mask_dpwssds_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm_mask_dpwssds_epi32(src, 0b00000000, a, b);
        assert_eq_m128i(r, src);
        let r = _mm_mask_dpwssds_epi32(src, 0b00001111, a, b);
        let e = _mm_set1_epi32(3);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_maskz_dpwssds_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 16 | 1 << 0);
        let b = _mm_set1_epi32(1 << 16 | 1 << 0);
        let r = _mm_maskz_dpwssds_epi32(0b00000000, src, a, b);
        assert_eq_m128i(r, _mm_setzero_si128());
        let r = _mm_maskz_dpwssds_epi32(0b00001111, src, a, b);
        let e = _mm_set1_epi32(3);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_dpbusd_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm512_dpbusd_epi32(src, a, b);
        let e = _mm512_set1_epi32(5);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_mask_dpbusd_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm512_mask_dpbusd_epi32(src, 0b00000000_00000000, a, b);
        assert_eq_m512i(r, src);
        let r = _mm512_mask_dpbusd_epi32(src, 0b11111111_11111111, a, b);
        let e = _mm512_set1_epi32(5);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_maskz_dpbusd_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm512_maskz_dpbusd_epi32(0b00000000_00000000, src, a, b);
        assert_eq_m512i(r, _mm512_setzero_si512());
        let r = _mm512_maskz_dpbusd_epi32(0b11111111_11111111, src, a, b);
        let e = _mm512_set1_epi32(5);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_dpbusd_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm256_dpbusd_epi32(src, a, b);
        let e = _mm256_set1_epi32(5);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_mask_dpbusd_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm256_mask_dpbusd_epi32(src, 0b00000000, a, b);
        assert_eq_m256i(r, src);
        let r = _mm256_mask_dpbusd_epi32(src, 0b11111111, a, b);
        let e = _mm256_set1_epi32(5);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_maskz_dpbusd_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm256_maskz_dpbusd_epi32(0b00000000, src, a, b);
        assert_eq_m256i(r, _mm256_setzero_si256());
        let r = _mm256_maskz_dpbusd_epi32(0b11111111, src, a, b);
        let e = _mm256_set1_epi32(5);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_dpbusd_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm_dpbusd_epi32(src, a, b);
        let e = _mm_set1_epi32(5);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_mask_dpbusd_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm_mask_dpbusd_epi32(src, 0b00000000, a, b);
        assert_eq_m128i(r, src);
        let r = _mm_mask_dpbusd_epi32(src, 0b00001111, a, b);
        let e = _mm_set1_epi32(5);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_maskz_dpbusd_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm_maskz_dpbusd_epi32(0b00000000, src, a, b);
        assert_eq_m128i(r, _mm_setzero_si128());
        let r = _mm_maskz_dpbusd_epi32(0b00001111, src, a, b);
        let e = _mm_set1_epi32(5);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_dpbusds_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm512_dpbusds_epi32(src, a, b);
        let e = _mm512_set1_epi32(5);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_mask_dpbusds_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm512_mask_dpbusds_epi32(src, 0b00000000_00000000, a, b);
        assert_eq_m512i(r, src);
        let r = _mm512_mask_dpbusds_epi32(src, 0b11111111_11111111, a, b);
        let e = _mm512_set1_epi32(5);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni")]
    unsafe fn test_mm512_maskz_dpbusds_epi32() {
        let src = _mm512_set1_epi32(1);
        let a = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm512_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm512_maskz_dpbusds_epi32(0b00000000_00000000, src, a, b);
        assert_eq_m512i(r, _mm512_setzero_si512());
        let r = _mm512_maskz_dpbusds_epi32(0b11111111_11111111, src, a, b);
        let e = _mm512_set1_epi32(5);
        assert_eq_m512i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_dpbusds_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm256_dpbusds_epi32(src, a, b);
        let e = _mm256_set1_epi32(5);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_mask_dpbusds_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm256_mask_dpbusds_epi32(src, 0b00000000, a, b);
        assert_eq_m256i(r, src);
        let r = _mm256_mask_dpbusds_epi32(src, 0b11111111, a, b);
        let e = _mm256_set1_epi32(5);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm256_maskz_dpbusds_epi32() {
        let src = _mm256_set1_epi32(1);
        let a = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm256_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm256_maskz_dpbusds_epi32(0b00000000, src, a, b);
        assert_eq_m256i(r, _mm256_setzero_si256());
        let r = _mm256_maskz_dpbusds_epi32(0b11111111, src, a, b);
        let e = _mm256_set1_epi32(5);
        assert_eq_m256i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_dpbusds_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm_dpbusds_epi32(src, a, b);
        let e = _mm_set1_epi32(5);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_mask_dpbusds_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm_mask_dpbusds_epi32(src, 0b00000000, a, b);
        assert_eq_m128i(r, src);
        let r = _mm_mask_dpbusds_epi32(src, 0b00001111, a, b);
        let e = _mm_set1_epi32(5);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "avx512vnni,avx512vl")]
    unsafe fn test_mm_maskz_dpbusds_epi32() {
        let src = _mm_set1_epi32(1);
        let a = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let b = _mm_set1_epi32(1 << 24 | 1 << 16 | 1 << 8 | 1 << 0);
        let r = _mm_maskz_dpbusds_epi32(0b00000000, src, a, b);
        assert_eq_m128i(r, _mm_setzero_si128());
        let r = _mm_maskz_dpbusds_epi32(0b00001111, src, a, b);
        let e = _mm_set1_epi32(5);
        assert_eq_m128i(r, e);
    }
}
