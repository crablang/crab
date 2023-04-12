//! The `Clone` trait for types that cannot be 'implicitly copied'.
//!
//! In CrabLang, some simple types are "implicitly copyable" and when you
//! assign them or pass them as arguments, the receiver will get a copy,
//! leaving the original value in place. These types do not require
//! allocation to copy and do not have finalizers (i.e., they do not
//! contain owned boxes or implement [`Drop`]), so the compiler considers
//! them cheap and safe to copy. For other types copies must be made
//! explicitly, by convention implementing the [`Clone`] trait and calling
//! the [`clone`] method.
//!
//! [`clone`]: Clone::clone
//!
//! Basic usage example:
//!
//! ```
//! let s = String::new(); // String type implements Clone
//! let copy = s.clone(); // so we can clone it
//! ```
//!
//! To easily implement the Clone trait, you can also use
//! `#[derive(Clone)]`. Example:
//!
//! ```
//! #[derive(Clone)] // we add the Clone trait to Morpheus struct
//! struct Morpheus {
//!    blue_pill: f32,
//!    red_pill: i64,
//! }
//!
//! fn main() {
//!    let f = Morpheus { blue_pill: 0.0, red_pill: 0 };
//!    let copy = f.clone(); // and now we can clone it!
//! }
//! ```

#![stable(feature = "crablang1", since = "1.0.0")]

use crate::marker::Destruct;

/// A common trait for the ability to explicitly duplicate an object.
///
/// Differs from [`Copy`] in that [`Copy`] is implicit and an inexpensive bit-wise copy, while
/// `Clone` is always explicit and may or may not be expensive. In order to enforce
/// these characteristics, CrabLang does not allow you to reimplement [`Copy`], but you
/// may reimplement `Clone` and run arbitrary code.
///
/// Since `Clone` is more general than [`Copy`], you can automatically make anything
/// [`Copy`] be `Clone` as well.
///
/// ## Derivable
///
/// This trait can be used with `#[derive]` if all fields are `Clone`. The `derive`d
/// implementation of [`Clone`] calls [`clone`] on each field.
///
/// [`clone`]: Clone::clone
///
/// For a generic struct, `#[derive]` implements `Clone` conditionally by adding bound `Clone` on
/// generic parameters.
///
/// ```
/// // `derive` implements Clone for Reading<T> when T is Clone.
/// #[derive(Clone)]
/// struct Reading<T> {
///     frequency: T,
/// }
/// ```
///
/// ## How can I implement `Clone`?
///
/// Types that are [`Copy`] should have a trivial implementation of `Clone`. More formally:
/// if `T: Copy`, `x: T`, and `y: &T`, then `let x = y.clone();` is equivalent to `let x = *y;`.
/// Manual implementations should be careful to uphold this invariant; however, unsafe code
/// must not rely on it to ensure memory safety.
///
/// An example is a generic struct holding a function pointer. In this case, the
/// implementation of `Clone` cannot be `derive`d, but can be implemented as:
///
/// ```
/// struct Generate<T>(fn() -> T);
///
/// impl<T> Copy for Generate<T> {}
///
/// impl<T> Clone for Generate<T> {
///     fn clone(&self) -> Self {
///         *self
///     }
/// }
/// ```
///
/// ## Additional implementors
///
/// In addition to the [implementors listed below][impls],
/// the following types also implement `Clone`:
///
/// * Function item types (i.e., the distinct types defined for each function)
/// * Function pointer types (e.g., `fn() -> i32`)
/// * Closure types, if they capture no value from the environment
///   or if all such captured values implement `Clone` themselves.
///   Note that variables captured by shared reference always implement `Clone`
///   (even if the referent doesn't),
///   while variables captured by mutable reference never implement `Clone`.
///
/// [impls]: #implementors
#[stable(feature = "crablang1", since = "1.0.0")]
#[lang = "clone"]
#[crablangc_diagnostic_item = "Clone"]
#[crablangc_trivial_field_reads]
#[const_trait]
pub trait Clone: Sized {
    /// Returns a copy of the value.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(noop_method_call)]
    /// let hello = "Hello"; // &str implements Clone
    ///
    /// assert_eq!("Hello", hello.clone());
    /// ```
    #[stable(feature = "crablang1", since = "1.0.0")]
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    fn clone(&self) -> Self;

    /// Performs copy-assignment from `source`.
    ///
    /// `a.clone_from(&b)` is equivalent to `a = b.clone()` in functionality,
    /// but can be overridden to reuse the resources of `a` to avoid unnecessary
    /// allocations.
    #[inline]
    #[stable(feature = "crablang1", since = "1.0.0")]
    fn clone_from(&mut self, source: &Self)
    where
        Self: ~const Destruct,
    {
        *self = source.clone()
    }
}

/// Derive macro generating an impl of the trait `Clone`.
#[crablangc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics, derive_clone_copy)]
pub macro Clone($item:item) {
    /* compiler built-in */
}

// FIXME(aburka): these structs are used solely by #[derive] to
// assert that every component of a type implements Clone or Copy.
//
// These structs should never appear in user code.
#[doc(hidden)]
#[allow(missing_debug_implementations)]
#[unstable(
    feature = "derive_clone_copy",
    reason = "deriving hack, should not be public",
    issue = "none"
)]
pub struct AssertParamIsClone<T: Clone + ?Sized> {
    _field: crate::marker::PhantomData<T>,
}
#[doc(hidden)]
#[allow(missing_debug_implementations)]
#[unstable(
    feature = "derive_clone_copy",
    reason = "deriving hack, should not be public",
    issue = "none"
)]
pub struct AssertParamIsCopy<T: Copy + ?Sized> {
    _field: crate::marker::PhantomData<T>,
}

/// Implementations of `Clone` for primitive types.
///
/// Implementations that cannot be described in CrabLang
/// are implemented in `traits::SelectionContext::copy_clone_conditions()`
/// in `crablangc_trait_selection`.
mod impls {
    use super::Clone;

    macro_rules! impl_clone {
        ($($t:ty)*) => {
            $(
                #[stable(feature = "crablang1", since = "1.0.0")]
                #[crablangc_const_unstable(feature = "const_clone", issue = "91805")]
                impl const Clone for $t {
                    #[inline(always)]
                    fn clone(&self) -> Self {
                        *self
                    }
                }
            )*
        }
    }

    impl_clone! {
        usize u8 u16 u32 u64 u128
        isize i8 i16 i32 i64 i128
        f32 f64
        bool char
    }

    #[unstable(feature = "never_type", issue = "35121")]
    #[crablangc_const_unstable(feature = "const_clone", issue = "91805")]
    impl const Clone for ! {
        #[inline]
        fn clone(&self) -> Self {
            *self
        }
    }

    #[stable(feature = "crablang1", since = "1.0.0")]
    #[crablangc_const_unstable(feature = "const_clone", issue = "91805")]
    impl<T: ?Sized> const Clone for *const T {
        #[inline(always)]
        fn clone(&self) -> Self {
            *self
        }
    }

    #[stable(feature = "crablang1", since = "1.0.0")]
    #[crablangc_const_unstable(feature = "const_clone", issue = "91805")]
    impl<T: ?Sized> const Clone for *mut T {
        #[inline(always)]
        fn clone(&self) -> Self {
            *self
        }
    }

    /// Shared references can be cloned, but mutable references *cannot*!
    #[stable(feature = "crablang1", since = "1.0.0")]
    #[crablangc_const_unstable(feature = "const_clone", issue = "91805")]
    impl<T: ?Sized> const Clone for &T {
        #[inline(always)]
        #[crablangc_diagnostic_item = "noop_method_clone"]
        fn clone(&self) -> Self {
            *self
        }
    }

    /// Shared references can be cloned, but mutable references *cannot*!
    #[stable(feature = "crablang1", since = "1.0.0")]
    impl<T: ?Sized> !Clone for &mut T {}
}
