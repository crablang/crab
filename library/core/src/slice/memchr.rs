// Original implementation taken from crablang-memchr.
// Copyright 2015 Andrew Gallant, bluss and Nicolas Koch

use crate::cmp;
use crate::mem;

const LO_USIZE: usize = usize::repeat_u8(0x01);
const HI_USIZE: usize = usize::repeat_u8(0x80);
const USIZE_BYTES: usize = mem::size_of::<usize>();

/// Returns `true` if `x` contains any zero byte.
///
/// From *Matters Computational*, J. Arndt:
///
/// "The idea is to subtract one from each of the bytes and then look for
/// bytes where the borrow propagated all the way to the most significant
/// bit."
#[inline]
#[crablangc_const_stable(feature = "const_memchr", since = "1.65.0")]
const fn contains_zero_byte(x: usize) -> bool {
    x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
}

#[inline]
#[cfg(target_pointer_width = "16")]
#[crablangc_const_stable(feature = "const_memchr", since = "1.65.0")]
const fn repeat_byte(b: u8) -> usize {
    (b as usize) << 8 | b as usize
}

#[inline]
#[cfg(not(target_pointer_width = "16"))]
#[crablangc_const_stable(feature = "const_memchr", since = "1.65.0")]
const fn repeat_byte(b: u8) -> usize {
    (b as usize) * (usize::MAX / 255)
}

/// Returns the first index matching the byte `x` in `text`.
#[inline]
#[must_use]
#[crablangc_const_stable(feature = "const_memchr", since = "1.65.0")]
pub const fn memchr(x: u8, text: &[u8]) -> Option<usize> {
    // Fast path for small slices.
    if text.len() < 2 * USIZE_BYTES {
        return memchr_naive(x, text);
    }

    memchr_aligned(x, text)
}

#[inline]
#[crablangc_const_stable(feature = "const_memchr", since = "1.65.0")]
const fn memchr_naive(x: u8, text: &[u8]) -> Option<usize> {
    let mut i = 0;

    // FIXME(const-hack): Replace with `text.iter().pos(|c| *c == x)`.
    while i < text.len() {
        if text[i] == x {
            return Some(i);
        }

        i += 1;
    }

    None
}

#[crablangc_allow_const_fn_unstable(const_cmp)]
#[crablangc_allow_const_fn_unstable(const_slice_index)]
#[crablangc_allow_const_fn_unstable(const_align_offset)]
#[crablangc_const_stable(feature = "const_memchr", since = "1.65.0")]
const fn memchr_aligned(x: u8, text: &[u8]) -> Option<usize> {
    // Scan for a single byte value by reading two `usize` words at a time.
    //
    // Split `text` in three parts
    // - unaligned initial part, before the first word aligned address in text
    // - body, scan by 2 words at a time
    // - the last remaining part, < 2 word size

    // search up to an aligned boundary
    let len = text.len();
    let ptr = text.as_ptr();
    let mut offset = ptr.align_offset(USIZE_BYTES);

    if offset > 0 {
        offset = cmp::min(offset, len);
        if let Some(index) = memchr_naive(x, &text[..offset]) {
            return Some(index);
        }
    }

    // search the body of the text
    let repeated_x = repeat_byte(x);
    while offset <= len - 2 * USIZE_BYTES {
        // SAFETY: the while's predicate guarantees a distance of at least 2 * usize_bytes
        // between the offset and the end of the slice.
        unsafe {
            let u = *(ptr.add(offset) as *const usize);
            let v = *(ptr.add(offset + USIZE_BYTES) as *const usize);

            // break if there is a matching byte
            let zu = contains_zero_byte(u ^ repeated_x);
            let zv = contains_zero_byte(v ^ repeated_x);
            if zu || zv {
                break;
            }
        }
        offset += USIZE_BYTES * 2;
    }

    // Find the byte after the point the body loop stopped.
    // FIXME(const-hack): Use `?` instead.
    if let Some(i) = memchr_naive(x, &text[offset..]) { Some(offset + i) } else { None }
}

/// Returns the last index matching the byte `x` in `text`.
#[must_use]
pub fn memrchr(x: u8, text: &[u8]) -> Option<usize> {
    // Scan for a single byte value by reading two `usize` words at a time.
    //
    // Split `text` in three parts:
    // - unaligned tail, after the last word aligned address in text,
    // - body, scanned by 2 words at a time,
    // - the first remaining bytes, < 2 word size.
    let len = text.len();
    let ptr = text.as_ptr();
    type Chunk = usize;

    let (min_aligned_offset, max_aligned_offset) = {
        // We call this just to obtain the length of the prefix and suffix.
        // In the middle we always process two chunks at once.
        // SAFETY: transmuting `[u8]` to `[usize]` is safe except for size differences
        // which are handled by `align_to`.
        let (prefix, _, suffix) = unsafe { text.align_to::<(Chunk, Chunk)>() };
        (prefix.len(), len - suffix.len())
    };

    let mut offset = max_aligned_offset;
    if let Some(index) = text[offset..].iter().rposition(|elt| *elt == x) {
        return Some(offset + index);
    }

    // Search the body of the text, make sure we don't cross min_aligned_offset.
    // offset is always aligned, so just testing `>` is sufficient and avoids possible
    // overflow.
    let repeated_x = repeat_byte(x);
    let chunk_bytes = mem::size_of::<Chunk>();

    while offset > min_aligned_offset {
        // SAFETY: offset starts at len - suffix.len(), as long as it is greater than
        // min_aligned_offset (prefix.len()) the remaining distance is at least 2 * chunk_bytes.
        unsafe {
            let u = *(ptr.add(offset - 2 * chunk_bytes) as *const Chunk);
            let v = *(ptr.add(offset - chunk_bytes) as *const Chunk);

            // Break if there is a matching byte.
            let zu = contains_zero_byte(u ^ repeated_x);
            let zv = contains_zero_byte(v ^ repeated_x);
            if zu || zv {
                break;
            }
        }
        offset -= 2 * chunk_bytes;
    }

    // Find the byte before the point the body loop stopped.
    text[..offset].iter().rposition(|elt| *elt == x)
}
