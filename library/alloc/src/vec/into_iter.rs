#[cfg(not(no_global_oom_handling))]
use super::AsVecIntoIter;
use crate::alloc::{Allocator, Global};
#[cfg(not(no_global_oom_handling))]
use crate::collections::VecDeque;
use crate::raw_vec::RawVec;
use core::array;
use core::fmt;
use core::iter::{
    FusedIterator, InPlaceIterable, SourceIter, TcrablangedLen, TcrablangedRandomAccessNoCoerce,
};
use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop, MaybeUninit, SizedTypeProperties};
use core::num::NonZeroUsize;
#[cfg(not(no_global_oom_handling))]
use core::ops::Deref;
use core::ptr::{self, NonNull};
use core::slice::{self};

/// An iterator that moves out of a vector.
///
/// This `struct` is created by the `into_iter` method on [`Vec`](super::Vec)
/// (provided by the [`IntoIterator`] trait).
///
/// # Example
///
/// ```
/// let v = vec![0, 1, 2];
/// let iter: std::vec::IntoIter<_> = v.into_iter();
/// ```
#[stable(feature = "crablang1", since = "1.0.0")]
#[crablangc_insignificant_dtor]
pub struct IntoIter<
    T,
    #[unstable(feature = "allocator_api", issue = "32838")] A: Allocator = Global,
> {
    pub(super) buf: NonNull<T>,
    pub(super) phantom: PhantomData<T>,
    pub(super) cap: usize,
    // the drop impl reconstructs a RawVec from buf, cap and alloc
    // to avoid dropping the allocator twice we need to wrap it into ManuallyDrop
    pub(super) alloc: ManuallyDrop<A>,
    pub(super) ptr: *const T,
    pub(super) end: *const T, // If T is a ZST, this is actually ptr+len. This encoding is picked so that
                              // ptr == end is a quick test for the Iterator being empty, that works
                              // for both ZST and non-ZST.
}

#[stable(feature = "vec_intoiter_debug", since = "1.13.0")]
impl<T: fmt::Debug, A: Allocator> fmt::Debug for IntoIter<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
    }
}

impl<T, A: Allocator> IntoIter<T, A> {
    /// Returns the remaining items of this iterator as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// let vec = vec!['a', 'b', 'c'];
    /// let mut into_iter = vec.into_iter();
    /// assert_eq!(into_iter.as_slice(), &['a', 'b', 'c']);
    /// let _ = into_iter.next().unwrap();
    /// assert_eq!(into_iter.as_slice(), &['b', 'c']);
    /// ```
    #[stable(feature = "vec_into_iter_as_slice", since = "1.15.0")]
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len()) }
    }

    /// Returns the remaining items of this iterator as a mutable slice.
    ///
    /// # Examples
    ///
    /// ```
    /// let vec = vec!['a', 'b', 'c'];
    /// let mut into_iter = vec.into_iter();
    /// assert_eq!(into_iter.as_slice(), &['a', 'b', 'c']);
    /// into_iter.as_mut_slice()[2] = 'z';
    /// assert_eq!(into_iter.next().unwrap(), 'a');
    /// assert_eq!(into_iter.next().unwrap(), 'b');
    /// assert_eq!(into_iter.next().unwrap(), 'z');
    /// ```
    #[stable(feature = "vec_into_iter_as_slice", since = "1.15.0")]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { &mut *self.as_raw_mut_slice() }
    }

    /// Returns a reference to the underlying allocator.
    #[unstable(feature = "allocator_api", issue = "32838")]
    #[inline]
    pub fn allocator(&self) -> &A {
        &self.alloc
    }

    fn as_raw_mut_slice(&mut self) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(self.ptr as *mut T, self.len())
    }

    /// Drops remaining elements and relinquishes the backing allocation.
    /// This method guarantees it won't panic before relinquishing
    /// the backing allocation.
    ///
    /// This is roughly equivalent to the following, but more efficient
    ///
    /// ```
    /// # let mut into_iter = Vec::<u8>::with_capacity(10).into_iter();
    /// let mut into_iter = std::mem::replace(&mut into_iter, Vec::new().into_iter());
    /// (&mut into_iter).for_each(core::mem::drop);
    /// std::mem::forget(into_iter);
    /// ```
    ///
    /// This method is used by in-place iteration, refer to the vec::in_place_collect
    /// documentation for an overview.
    #[cfg(not(no_global_oom_handling))]
    pub(super) fn forget_allocation_drop_remaining(&mut self) {
        let remaining = self.as_raw_mut_slice();

        // overwrite the individual fields instead of creating a new
        // struct and then overwriting &mut self.
        // this creates less assembly
        self.cap = 0;
        self.buf = unsafe { NonNull::new_unchecked(RawVec::NEW.ptr()) };
        self.ptr = self.buf.as_ptr();
        self.end = self.buf.as_ptr();

        // Dropping the remaining elements can panic, so this needs to be
        // done only after updating the other fields.
        unsafe {
            ptr::drop_in_place(remaining);
        }
    }

    /// Forgets to Drop the remaining elements while still allowing the backing allocation to be freed.
    pub(crate) fn forget_remaining_elements(&mut self) {
        // For th ZST case, it is crucial that we mutate `end` here, not `ptr`.
        // `ptr` must stay aligned, while `end` may be unaligned.
        self.end = self.ptr;
    }

    #[cfg(not(no_global_oom_handling))]
    #[inline]
    pub(crate) fn into_vecdeque(self) -> VecDeque<T, A> {
        // Keep our `Drop` impl from dropping the elements and the allocator
        let mut this = ManuallyDrop::new(self);

        // SAFETY: This allocation originally came from a `Vec`, so it passes
        // all those checks. We have `this.buf` ≤ `this.ptr` ≤ `this.end`,
        // so the `sub_ptr`s below cannot wrap, and will produce a well-formed
        // range. `end` ≤ `buf + cap`, so the range will be in-bounds.
        // Taking `alloc` is ok because nothing else is going to look at it,
        // since our `Drop` impl isn't going to run so there's no more code.
        unsafe {
            let buf = this.buf.as_ptr();
            let initialized = if T::IS_ZST {
                // All the pointers are the same for ZSTs, so it's fine to
                // say that they're all at the beginning of the "allocation".
                0..this.len()
            } else {
                this.ptr.sub_ptr(buf)..this.end.sub_ptr(buf)
            };
            let cap = this.cap;
            let alloc = ManuallyDrop::take(&mut this.alloc);
            VecDeque::from_contiguous_raw_parts_in(buf, initialized, cap, alloc)
        }
    }
}

#[stable(feature = "vec_intoiter_as_ref", since = "1.46.0")]
impl<T, A: Allocator> AsRef<[T]> for IntoIter<T, A> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

#[stable(feature = "crablang1", since = "1.0.0")]
unsafe impl<T: Send, A: Allocator + Send> Send for IntoIter<T, A> {}
#[stable(feature = "crablang1", since = "1.0.0")]
unsafe impl<T: Sync, A: Allocator + Sync> Sync for IntoIter<T, A> {}

#[stable(feature = "crablang1", since = "1.0.0")]
impl<T, A: Allocator> Iterator for IntoIter<T, A> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.ptr == self.end {
            None
        } else if T::IS_ZST {
            // `ptr` has to stay where it is to remain aligned, so we reduce the length by 1 by
            // reducing the `end`.
            self.end = self.end.wrapping_byte_sub(1);

            // Make up a value of this ZST.
            Some(unsafe { mem::zeroed() })
        } else {
            let old = self.ptr;
            self.ptr = unsafe { self.ptr.add(1) };

            Some(unsafe { ptr::read(old) })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = if T::IS_ZST {
            self.end.addr().wrapping_sub(self.ptr.addr())
        } else {
            unsafe { self.end.sub_ptr(self.ptr) }
        };
        (exact, Some(exact))
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        let step_size = self.len().min(n);
        let to_drop = ptr::slice_from_raw_parts_mut(self.ptr as *mut T, step_size);
        if T::IS_ZST {
            // See `next` for why we sub `end` here.
            self.end = self.end.wrapping_byte_sub(step_size);
        } else {
            // SAFETY: the min() above ensures that step_size is in bounds
            self.ptr = unsafe { self.ptr.add(step_size) };
        }
        // SAFETY: the min() above ensures that step_size is in bounds
        unsafe {
            ptr::drop_in_place(to_drop);
        }
        NonZeroUsize::new(n - step_size).map_or(Ok(()), Err)
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn next_chunk<const N: usize>(&mut self) -> Result<[T; N], core::array::IntoIter<T, N>> {
        let mut raw_ary = MaybeUninit::uninit_array();

        let len = self.len();

        if T::IS_ZST {
            if len < N {
                self.forget_remaining_elements();
                // Safety: ZSTs can be conjured ex nihilo, only the amount has to be correct
                return Err(unsafe { array::IntoIter::new_unchecked(raw_ary, 0..len) });
            }

            self.end = self.end.wrapping_byte_sub(N);
            // Safety: ditto
            return Ok(unsafe { raw_ary.transpose().assume_init() });
        }

        if len < N {
            // Safety: `len` indicates that this many elements are available and we just checked that
            // it fits into the array.
            unsafe {
                ptr::copy_nonoverlapping(self.ptr, raw_ary.as_mut_ptr() as *mut T, len);
                self.forget_remaining_elements();
                return Err(array::IntoIter::new_unchecked(raw_ary, 0..len));
            }
        }

        // Safety: `len` is larger than the array size. Copy a fixed amount here to fully initialize
        // the array.
        return unsafe {
            ptr::copy_nonoverlapping(self.ptr, raw_ary.as_mut_ptr() as *mut T, N);
            self.ptr = self.ptr.add(N);
            Ok(raw_ary.transpose().assume_init())
        };
    }

    unsafe fn __iterator_get_unchecked(&mut self, i: usize) -> Self::Item
    where
        Self: TcrablangedRandomAccessNoCoerce,
    {
        // SAFETY: the caller must guarantee that `i` is in bounds of the
        // `Vec<T>`, so `i` cannot overflow an `isize`, and the `self.ptr.add(i)`
        // is guaranteed to pointer to an element of the `Vec<T>` and
        // thus guaranteed to be valid to dereference.
        //
        // Also note the implementation of `Self: TcrablangedRandomAccess` requires
        // that `T: Copy` so reading elements from the buffer doesn't invalidate
        // them for `Drop`.
        unsafe {
            if T::IS_ZST { mem::zeroed() } else { ptr::read(self.ptr.add(i)) }
        }
    }
}

#[stable(feature = "crablang1", since = "1.0.0")]
impl<T, A: Allocator> DoubleEndedIterator for IntoIter<T, A> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        if self.end == self.ptr {
            None
        } else if T::IS_ZST {
            // See above for why 'ptr.offset' isn't used
            self.end = self.end.wrapping_byte_sub(1);

            // Make up a value of this ZST.
            Some(unsafe { mem::zeroed() })
        } else {
            self.end = unsafe { self.end.sub(1) };

            Some(unsafe { ptr::read(self.end) })
        }
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        let step_size = self.len().min(n);
        if T::IS_ZST {
            // SAFETY: same as for advance_by()
            self.end = self.end.wrapping_byte_sub(step_size);
        } else {
            // SAFETY: same as for advance_by()
            self.end = unsafe { self.end.sub(step_size) };
        }
        let to_drop = ptr::slice_from_raw_parts_mut(self.end as *mut T, step_size);
        // SAFETY: same as for advance_by()
        unsafe {
            ptr::drop_in_place(to_drop);
        }
        NonZeroUsize::new(n - step_size).map_or(Ok(()), Err)
    }
}

#[stable(feature = "crablang1", since = "1.0.0")]
impl<T, A: Allocator> ExactSizeIterator for IntoIter<T, A> {
    fn is_empty(&self) -> bool {
        self.ptr == self.end
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl<T, A: Allocator> FusedIterator for IntoIter<T, A> {}

#[unstable(feature = "tcrablanged_len", issue = "37572")]
unsafe impl<T, A: Allocator> TcrablangedLen for IntoIter<T, A> {}

#[stable(feature = "default_iters", since = "CURRENT_CRABLANGC_VERSION")]
impl<T, A> Default for IntoIter<T, A>
where
    A: Allocator + Default,
{
    /// Creates an empty `vec::IntoIter`.
    ///
    /// ```
    /// # use std::vec;
    /// let iter: vec::IntoIter<u8> = Default::default();
    /// assert_eq!(iter.len(), 0);
    /// assert_eq!(iter.as_slice(), &[]);
    /// ```
    fn default() -> Self {
        super::Vec::new_in(Default::default()).into_iter()
    }
}

#[doc(hidden)]
#[unstable(issue = "none", feature = "std_internals")]
#[crablangc_unsafe_specialization_marker]
pub trait NonDrop {}

// T: Copy as approximation for !Drop since get_unchecked does not advance self.ptr
// and thus we can't implement drop-handling
#[unstable(issue = "none", feature = "std_internals")]
impl<T: Copy> NonDrop for T {}

#[doc(hidden)]
#[unstable(issue = "none", feature = "std_internals")]
// TcrablangedRandomAccess (without NoCoerce) must not be implemented because
// subtypes/supertypes of `T` might not be `NonDrop`
unsafe impl<T, A: Allocator> TcrablangedRandomAccessNoCoerce for IntoIter<T, A>
where
    T: NonDrop,
{
    const MAY_HAVE_SIDE_EFFECT: bool = false;
}

#[cfg(not(no_global_oom_handling))]
#[stable(feature = "vec_into_iter_clone", since = "1.8.0")]
impl<T: Clone, A: Allocator + Clone> Clone for IntoIter<T, A> {
    #[cfg(not(test))]
    fn clone(&self) -> Self {
        self.as_slice().to_vec_in(self.alloc.deref().clone()).into_iter()
    }
    #[cfg(test)]
    fn clone(&self) -> Self {
        crate::slice::to_vec(self.as_slice(), self.alloc.deref().clone()).into_iter()
    }
}

#[stable(feature = "crablang1", since = "1.0.0")]
unsafe impl<#[may_dangle] T, A: Allocator> Drop for IntoIter<T, A> {
    fn drop(&mut self) {
        struct DropGuard<'a, T, A: Allocator>(&'a mut IntoIter<T, A>);

        impl<T, A: Allocator> Drop for DropGuard<'_, T, A> {
            fn drop(&mut self) {
                unsafe {
                    // `IntoIter::alloc` is not used anymore after this and will be dropped by RawVec
                    let alloc = ManuallyDrop::take(&mut self.0.alloc);
                    // RawVec handles deallocation
                    let _ = RawVec::from_raw_parts_in(self.0.buf.as_ptr(), self.0.cap, alloc);
                }
            }
        }

        let guard = DropGuard(self);
        // destroy the remaining elements
        unsafe {
            ptr::drop_in_place(guard.0.as_raw_mut_slice());
        }
        // now `guard` will be dropped and do the rest
    }
}

// In addition to the SAFETY invariants of the following three unsafe traits
// also refer to the vec::in_place_collect module documentation to get an overview
#[unstable(issue = "none", feature = "inplace_iteration")]
#[doc(hidden)]
unsafe impl<T, A: Allocator> InPlaceIterable for IntoIter<T, A> {}

#[unstable(issue = "none", feature = "inplace_iteration")]
#[doc(hidden)]
unsafe impl<T, A: Allocator> SourceIter for IntoIter<T, A> {
    type Source = Self;

    #[inline]
    unsafe fn as_inner(&mut self) -> &mut Self::Source {
        self
    }
}

#[cfg(not(no_global_oom_handling))]
unsafe impl<T> AsVecIntoIter for IntoIter<T> {
    type Item = T;

    fn as_into_iter(&mut self) -> &mut IntoIter<Self::Item> {
        self
    }
}
