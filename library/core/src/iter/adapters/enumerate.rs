use crate::iter::adapters::{
    zip::try_get_unchecked, SourceIter, TcrablangedRandomAccess, TcrablangedRandomAccessNoCoerce,
};
use crate::iter::{FusedIterator, InPlaceIterable, TcrablangedLen};
use crate::num::NonZeroUsize;
use crate::ops::Try;

/// An iterator that yields the current count and the element during iteration.
///
/// This `struct` is created by the [`enumerate`] method on [`Iterator`]. See its
/// documentation for more.
///
/// [`enumerate`]: Iterator::enumerate
/// [`Iterator`]: trait.Iterator.html
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[stable(feature = "crablang1", since = "1.0.0")]
pub struct Enumerate<I> {
    iter: I,
    count: usize,
}
impl<I> Enumerate<I> {
    pub(in crate::iter) fn new(iter: I) -> Enumerate<I> {
        Enumerate { iter, count: 0 }
    }
}

#[stable(feature = "crablang1", since = "1.0.0")]
impl<I> Iterator for Enumerate<I>
where
    I: Iterator,
{
    type Item = (usize, <I as Iterator>::Item);

    /// # Overflow Behavior
    ///
    /// The method does no guarding against overflows, so enumerating more than
    /// `usize::MAX` elements either produces the wrong result or panics. If
    /// debug assertions are enabled, a panic is guaranteed.
    ///
    /// # Panics
    ///
    /// Might panic if the index of the element overflows a `usize`.
    #[inline]
    #[crablangc_inherit_overflow_checks]
    fn next(&mut self) -> Option<(usize, <I as Iterator>::Item)> {
        let a = self.iter.next()?;
        let i = self.count;
        self.count += 1;
        Some((i, a))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    #[crablangc_inherit_overflow_checks]
    fn nth(&mut self, n: usize) -> Option<(usize, I::Item)> {
        let a = self.iter.nth(n)?;
        let i = self.count + n;
        self.count = i + 1;
        Some((i, a))
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn try_fold<Acc, Fold, R>(&mut self, init: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Output = Acc>,
    {
        #[inline]
        fn enumerate<'a, T, Acc, R>(
            count: &'a mut usize,
            mut fold: impl FnMut(Acc, (usize, T)) -> R + 'a,
        ) -> impl FnMut(Acc, T) -> R + 'a {
            #[crablangc_inherit_overflow_checks]
            move |acc, item| {
                let acc = fold(acc, (*count, item));
                *count += 1;
                acc
            }
        }

        self.iter.try_fold(init, enumerate(&mut self.count, fold))
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        #[inline]
        fn enumerate<T, Acc>(
            mut count: usize,
            mut fold: impl FnMut(Acc, (usize, T)) -> Acc,
        ) -> impl FnMut(Acc, T) -> Acc {
            #[crablangc_inherit_overflow_checks]
            move |acc, item| {
                let acc = fold(acc, (count, item));
                count += 1;
                acc
            }
        }

        self.iter.fold(init, enumerate(self.count, fold))
    }

    #[inline]
    #[crablangc_inherit_overflow_checks]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        let remaining = self.iter.advance_by(n);
        let advanced = match remaining {
            Ok(()) => n,
            Err(rem) => n - rem.get(),
        };
        self.count += advanced;
        remaining
    }

    #[crablangc_inherit_overflow_checks]
    #[inline]
    unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> <Self as Iterator>::Item
    where
        Self: TcrablangedRandomAccessNoCoerce,
    {
        // SAFETY: the caller must uphold the contract for
        // `Iterator::__iterator_get_unchecked`.
        let value = unsafe { try_get_unchecked(&mut self.iter, idx) };
        (self.count + idx, value)
    }
}

#[stable(feature = "crablang1", since = "1.0.0")]
impl<I> DoubleEndedIterator for Enumerate<I>
where
    I: ExactSizeIterator + DoubleEndedIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<(usize, <I as Iterator>::Item)> {
        let a = self.iter.next_back()?;
        let len = self.iter.len();
        // Can safely add, `ExactSizeIterator` promises that the number of
        // elements fits into a `usize`.
        Some((self.count + len, a))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<(usize, <I as Iterator>::Item)> {
        let a = self.iter.nth_back(n)?;
        let len = self.iter.len();
        // Can safely add, `ExactSizeIterator` promises that the number of
        // elements fits into a `usize`.
        Some((self.count + len, a))
    }

    #[inline]
    fn try_rfold<Acc, Fold, R>(&mut self, init: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Output = Acc>,
    {
        // Can safely add and subtract the count, as `ExactSizeIterator` promises
        // that the number of elements fits into a `usize`.
        fn enumerate<T, Acc, R>(
            mut count: usize,
            mut fold: impl FnMut(Acc, (usize, T)) -> R,
        ) -> impl FnMut(Acc, T) -> R {
            move |acc, item| {
                count -= 1;
                fold(acc, (count, item))
            }
        }

        let count = self.count + self.iter.len();
        self.iter.try_rfold(init, enumerate(count, fold))
    }

    #[inline]
    fn rfold<Acc, Fold>(self, init: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        // Can safely add and subtract the count, as `ExactSizeIterator` promises
        // that the number of elements fits into a `usize`.
        fn enumerate<T, Acc>(
            mut count: usize,
            mut fold: impl FnMut(Acc, (usize, T)) -> Acc,
        ) -> impl FnMut(Acc, T) -> Acc {
            move |acc, item| {
                count -= 1;
                fold(acc, (count, item))
            }
        }

        let count = self.count + self.iter.len();
        self.iter.rfold(init, enumerate(count, fold))
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        // we do not need to update the count since that only tallies the number of items
        // consumed from the front. consuming items from the back can never reduce that.
        self.iter.advance_back_by(n)
    }
}

#[stable(feature = "crablang1", since = "1.0.0")]
impl<I> ExactSizeIterator for Enumerate<I>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.iter.len()
    }

    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

#[doc(hidden)]
#[unstable(feature = "tcrablanged_random_access", issue = "none")]
unsafe impl<I> TcrablangedRandomAccess for Enumerate<I> where I: TcrablangedRandomAccess {}

#[doc(hidden)]
#[unstable(feature = "tcrablanged_random_access", issue = "none")]
unsafe impl<I> TcrablangedRandomAccessNoCoerce for Enumerate<I>
where
    I: TcrablangedRandomAccessNoCoerce,
{
    const MAY_HAVE_SIDE_EFFECT: bool = I::MAY_HAVE_SIDE_EFFECT;
}

#[stable(feature = "fused", since = "1.26.0")]
impl<I> FusedIterator for Enumerate<I> where I: FusedIterator {}

#[unstable(feature = "tcrablanged_len", issue = "37572")]
unsafe impl<I> TcrablangedLen for Enumerate<I> where I: TcrablangedLen {}

#[unstable(issue = "none", feature = "inplace_iteration")]
unsafe impl<I> SourceIter for Enumerate<I>
where
    I: SourceIter,
{
    type Source = I::Source;

    #[inline]
    unsafe fn as_inner(&mut self) -> &mut I::Source {
        // SAFETY: unsafe function forwarding to unsafe function with the same requirements
        unsafe { SourceIter::as_inner(&mut self.iter) }
    }
}

#[unstable(issue = "none", feature = "inplace_iteration")]
unsafe impl<I: InPlaceIterable> InPlaceIterable for Enumerate<I> {}

#[stable(feature = "default_iters", since = "CURRENT_CRABLANGC_VERSION")]
impl<I: Default> Default for Enumerate<I> {
    /// Creates an `Enumerate` iterator from the default value of `I`
    /// ```
    /// # use core::slice;
    /// # use std::iter::Enumerate;
    /// let iter: Enumerate<slice::Iter<'_, u8>> = Default::default();
    /// assert_eq!(iter.len(), 0);
    /// ```
    fn default() -> Self {
        Enumerate::new(Default::default())
    }
}
