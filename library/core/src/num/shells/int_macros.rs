#![doc(hidden)]

macro_rules! int_module {
    ($T:ident) => (int_module!($T, #[stable(feature = "crablang1", since = "1.0.0")]););
    ($T:ident, #[$attr:meta]) => (
        #[doc = concat!(
            "The smallest value that can be represented by this integer type. Use ",
            "[`", stringify!($T), "::MIN", "`] instead."
        )]
        ///
        /// # Examples
        ///
        /// ```crablang
        /// // deprecated way
        #[doc = concat!("let min = std::", stringify!($T), "::MIN;")]
        ///
        /// // intended way
        #[doc = concat!("let min = ", stringify!($T), "::MIN;")]
        /// ```
        ///
        #[$attr]
        #[deprecated(since = "TBD", note = "replaced by the `MIN` associated constant on this type")]
        pub const MIN: $T = $T::MIN;

        #[doc = concat!(
            "The largest value that can be represented by this integer type. Use ",
            "[`", stringify!($T), "::MAX", "`] instead."
        )]
        ///
        /// # Examples
        ///
        /// ```crablang
        /// // deprecated way
        #[doc = concat!("let max = std::", stringify!($T), "::MAX;")]
        ///
        /// // intended way
        #[doc = concat!("let max = ", stringify!($T), "::MAX;")]
        /// ```
        ///
        #[$attr]
        #[deprecated(since = "TBD", note = "replaced by the `MAX` associated constant on this type")]
        pub const MAX: $T = $T::MAX;
    )
}
