macro_rules! uint_impl {
    (
        Self = $SelfT:ty,
        ActualT = $ActualT:ident,
        SignedT = $SignedT:ident,
        NonZeroT = $NonZeroT:ident,

        // There are all for use *only* in doc comments.
        // As such, they're all passed as literals -- passing them as a string
        // literal is fine if they need to be multiple code tokens.
        // In non-comments, use the associated constants rather than these.
        BITS = $BITS:literal,
        MAX = $MaxV:literal,
        rot = $rot:literal,
        rot_op = $rot_op:literal,
        rot_result = $rot_result:literal,
        swap_op = $swap_op:literal,
        swapped = $swapped:literal,
        reversed = $reversed:literal,
        le_bytes = $le_bytes:literal,
        be_bytes = $be_bytes:literal,
        to_xe_bytes_doc = $to_xe_bytes_doc:expr,
        from_xe_bytes_doc = $from_xe_bytes_doc:expr,
        bound_condition = $bound_condition:literal,
    ) => {
        /// The smallest value that can be represented by this integer type.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MIN, 0);")]
        /// ```
        #[stable(feature = "assoc_int_consts", since = "1.43.0")]
        pub const MIN: Self = 0;

        /// The largest value that can be represented by this integer type
        #[doc = concat!("(2<sup>", $BITS, "</sup> &minus; 1", $bound_condition, ").")]
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX, ", stringify!($MaxV), ");")]
        /// ```
        #[stable(feature = "assoc_int_consts", since = "1.43.0")]
        pub const MAX: Self = !0;

        /// The size of this integer type in bits.
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::BITS, ", stringify!($BITS), ");")]
        /// ```
        #[stable(feature = "int_bits_const", since = "1.53.0")]
        pub const BITS: u32 = Self::MAX.count_ones();

        /// Converts a string slice in a given base to an integer.
        ///
        /// The string is expected to be an optional `+` sign
        /// followed by digits.
        /// Leading and trailing whitespace represent an error.
        /// Digits are a subset of these characters, depending on `radix`:
        ///
        /// * `0-9`
        /// * `a-z`
        /// * `A-Z`
        ///
        /// # Panics
        ///
        /// This function panics if `radix` is not in the range from 2 to 36.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::from_str_radix(\"A\", 16), Ok(10));")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        pub fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseIntError> {
            from_str_radix(src, radix)
        }

        /// Returns the number of ones in the binary representation of `self`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = 0b01001100", stringify!($SelfT), ";")]
        ///
        /// assert_eq!(n.count_ones(), 3);
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[doc(alias = "popcount")]
        #[doc(alias = "popcnt")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn count_ones(self) -> u32 {
            intrinsics::ctpop(self as $ActualT) as u32
        }

        /// Returns the number of zeros in the binary representation of `self`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.count_zeros(), 0);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn count_zeros(self) -> u32 {
            (!self).count_ones()
        }

        /// Returns the number of leading zeros in the binary representation of `self`.
        ///
        /// Depending on what you're doing with the value, you might also be interested in the
        /// [`ilog2`] function which returns a consistent number, even if the type widens.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = ", stringify!($SelfT), "::MAX >> 2;")]
        ///
        /// assert_eq!(n.leading_zeros(), 2);
        /// ```
        #[doc = concat!("[`ilog2`]: ", stringify!($SelfT), "::ilog2")]
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn leading_zeros(self) -> u32 {
            intrinsics::ctlz(self as $ActualT) as u32
        }

        /// Returns the number of trailing zeros in the binary representation
        /// of `self`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = 0b0101000", stringify!($SelfT), ";")]
        ///
        /// assert_eq!(n.trailing_zeros(), 3);
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn trailing_zeros(self) -> u32 {
            intrinsics::cttz(self) as u32
        }

        /// Returns the number of leading ones in the binary representation of `self`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = !(", stringify!($SelfT), "::MAX >> 2);")]
        ///
        /// assert_eq!(n.leading_ones(), 2);
        /// ```
        #[stable(feature = "leading_trailing_ones", since = "1.46.0")]
        #[crablangc_const_stable(feature = "leading_trailing_ones", since = "1.46.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn leading_ones(self) -> u32 {
            (!self).leading_zeros()
        }

        /// Returns the number of trailing ones in the binary representation
        /// of `self`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = 0b1010111", stringify!($SelfT), ";")]
        ///
        /// assert_eq!(n.trailing_ones(), 3);
        /// ```
        #[stable(feature = "leading_trailing_ones", since = "1.46.0")]
        #[crablangc_const_stable(feature = "leading_trailing_ones", since = "1.46.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn trailing_ones(self) -> u32 {
            (!self).trailing_zeros()
        }

        /// Shifts the bits to the left by a specified amount, `n`,
        /// wrapping the truncated bits to the end of the resulting integer.
        ///
        /// Please note this isn't the same operation as the `<<` shifting operator!
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = ", $rot_op, stringify!($SelfT), ";")]
        #[doc = concat!("let m = ", $rot_result, ";")]
        ///
        #[doc = concat!("assert_eq!(n.rotate_left(", $rot, "), m);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn rotate_left(self, n: u32) -> Self {
            intrinsics::rotate_left(self, n as $SelfT)
        }

        /// Shifts the bits to the right by a specified amount, `n`,
        /// wrapping the truncated bits to the beginning of the resulting
        /// integer.
        ///
        /// Please note this isn't the same operation as the `>>` shifting operator!
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = ", $rot_result, stringify!($SelfT), ";")]
        #[doc = concat!("let m = ", $rot_op, ";")]
        ///
        #[doc = concat!("assert_eq!(n.rotate_right(", $rot, "), m);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn rotate_right(self, n: u32) -> Self {
            intrinsics::rotate_right(self, n as $SelfT)
        }

        /// Reverses the byte order of the integer.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = ", $swap_op, stringify!($SelfT), ";")]
        /// let m = n.swap_bytes();
        ///
        #[doc = concat!("assert_eq!(m, ", $swapped, ");")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn swap_bytes(self) -> Self {
            intrinsics::bswap(self as $ActualT) as Self
        }

        /// Reverses the order of bits in the integer. The least significant bit becomes the most significant bit,
        ///                 second least-significant bit becomes second most-significant bit, etc.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = ", $swap_op, stringify!($SelfT), ";")]
        /// let m = n.reverse_bits();
        ///
        #[doc = concat!("assert_eq!(m, ", $reversed, ");")]
        #[doc = concat!("assert_eq!(0, 0", stringify!($SelfT), ".reverse_bits());")]
        /// ```
        #[stable(feature = "reverse_bits", since = "1.37.0")]
        #[crablangc_const_stable(feature = "reverse_bits", since = "1.37.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn reverse_bits(self) -> Self {
            intrinsics::bitreverse(self as $ActualT) as Self
        }

        /// Converts an integer from big endian to the target's endianness.
        ///
        /// On big endian this is a no-op. On little endian the bytes are
        /// swapped.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = 0x1A", stringify!($SelfT), ";")]
        ///
        /// if cfg!(target_endian = "big") {
        #[doc = concat!("    assert_eq!(", stringify!($SelfT), "::from_be(n), n)")]
        /// } else {
        #[doc = concat!("    assert_eq!(", stringify!($SelfT), "::from_be(n), n.swap_bytes())")]
        /// }
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use]
        #[inline(always)]
        pub const fn from_be(x: Self) -> Self {
            #[cfg(target_endian = "big")]
            {
                x
            }
            #[cfg(not(target_endian = "big"))]
            {
                x.swap_bytes()
            }
        }

        /// Converts an integer from little endian to the target's endianness.
        ///
        /// On little endian this is a no-op. On big endian the bytes are
        /// swapped.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = 0x1A", stringify!($SelfT), ";")]
        ///
        /// if cfg!(target_endian = "little") {
        #[doc = concat!("    assert_eq!(", stringify!($SelfT), "::from_le(n), n)")]
        /// } else {
        #[doc = concat!("    assert_eq!(", stringify!($SelfT), "::from_le(n), n.swap_bytes())")]
        /// }
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use]
        #[inline(always)]
        pub const fn from_le(x: Self) -> Self {
            #[cfg(target_endian = "little")]
            {
                x
            }
            #[cfg(not(target_endian = "little"))]
            {
                x.swap_bytes()
            }
        }

        /// Converts `self` to big endian from the target's endianness.
        ///
        /// On big endian this is a no-op. On little endian the bytes are
        /// swapped.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = 0x1A", stringify!($SelfT), ";")]
        ///
        /// if cfg!(target_endian = "big") {
        ///     assert_eq!(n.to_be(), n)
        /// } else {
        ///     assert_eq!(n.to_be(), n.swap_bytes())
        /// }
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn to_be(self) -> Self { // or not to be?
            #[cfg(target_endian = "big")]
            {
                self
            }
            #[cfg(not(target_endian = "big"))]
            {
                self.swap_bytes()
            }
        }

        /// Converts `self` to little endian from the target's endianness.
        ///
        /// On little endian this is a no-op. On big endian the bytes are
        /// swapped.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("let n = 0x1A", stringify!($SelfT), ";")]
        ///
        /// if cfg!(target_endian = "little") {
        ///     assert_eq!(n.to_le(), n)
        /// } else {
        ///     assert_eq!(n.to_le(), n.swap_bytes())
        /// }
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn to_le(self) -> Self {
            #[cfg(target_endian = "little")]
            {
                self
            }
            #[cfg(not(target_endian = "little"))]
            {
                self.swap_bytes()
            }
        }

        /// Checked integer addition. Computes `self + rhs`, returning `None`
        /// if overflow occurred.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!(
            "assert_eq!((", stringify!($SelfT), "::MAX - 2).checked_add(1), ",
            "Some(", stringify!($SelfT), "::MAX - 1));"
        )]
        #[doc = concat!("assert_eq!((", stringify!($SelfT), "::MAX - 2).checked_add(3), None);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_checked_int_methods", since = "1.47.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_add(self, rhs: Self) -> Option<Self> {
            let (a, b) = self.overflowing_add(rhs);
            if unlikely!(b) {None} else {Some(a)}
        }

        /// Unchecked integer addition. Computes `self + rhs`, assuming overflow
        /// cannot occur.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior when
        #[doc = concat!("`self + rhs > ", stringify!($SelfT), "::MAX` or `self + rhs < ", stringify!($SelfT), "::MIN`,")]
        /// i.e. when [`checked_add`] would return `None`.
        ///
        #[doc = concat!("[`checked_add`]: ", stringify!($SelfT), "::checked_add")]
        #[unstable(
            feature = "unchecked_math",
            reason = "niche optimization path",
            issue = "85122",
        )]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[crablangc_const_unstable(feature = "const_inherent_unchecked_arith", issue = "85122")]
        #[inline(always)]
        #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
        pub const unsafe fn unchecked_add(self, rhs: Self) -> Self {
            // SAFETY: the caller must uphold the safety contract for
            // `unchecked_add`.
            unsafe { intrinsics::unchecked_add(self, rhs) }
        }

        /// Checked addition with a signed integer. Computes `self + rhs`,
        /// returning `None` if overflow occurred.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".checked_add_signed(2), Some(3));")]
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".checked_add_signed(-2), None);")]
        #[doc = concat!("assert_eq!((", stringify!($SelfT), "::MAX - 2).checked_add_signed(3), None);")]
        /// ```
        #[stable(feature = "mixed_integer_ops", since = "1.66.0")]
        #[crablangc_const_stable(feature = "mixed_integer_ops", since = "1.66.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_add_signed(self, rhs: $SignedT) -> Option<Self> {
            let (a, b) = self.overflowing_add_signed(rhs);
            if unlikely!(b) {None} else {Some(a)}
        }

        /// Checked integer subtraction. Computes `self - rhs`, returning
        /// `None` if overflow occurred.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".checked_sub(1), Some(0));")]
        #[doc = concat!("assert_eq!(0", stringify!($SelfT), ".checked_sub(1), None);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_checked_int_methods", since = "1.47.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
            let (a, b) = self.overflowing_sub(rhs);
            if unlikely!(b) {None} else {Some(a)}
        }

        /// Unchecked integer subtraction. Computes `self - rhs`, assuming overflow
        /// cannot occur.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior when
        #[doc = concat!("`self - rhs > ", stringify!($SelfT), "::MAX` or `self - rhs < ", stringify!($SelfT), "::MIN`,")]
        /// i.e. when [`checked_sub`] would return `None`.
        ///
        #[doc = concat!("[`checked_sub`]: ", stringify!($SelfT), "::checked_sub")]
        #[unstable(
            feature = "unchecked_math",
            reason = "niche optimization path",
            issue = "85122",
        )]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[crablangc_const_unstable(feature = "const_inherent_unchecked_arith", issue = "85122")]
        #[inline(always)]
        #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
        pub const unsafe fn unchecked_sub(self, rhs: Self) -> Self {
            // SAFETY: the caller must uphold the safety contract for
            // `unchecked_sub`.
            unsafe { intrinsics::unchecked_sub(self, rhs) }
        }

        /// Checked integer multiplication. Computes `self * rhs`, returning
        /// `None` if overflow occurred.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".checked_mul(1), Some(5));")]
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.checked_mul(2), None);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_checked_int_methods", since = "1.47.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
            let (a, b) = self.overflowing_mul(rhs);
            if unlikely!(b) {None} else {Some(a)}
        }

        /// Unchecked integer multiplication. Computes `self * rhs`, assuming overflow
        /// cannot occur.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior when
        #[doc = concat!("`self * rhs > ", stringify!($SelfT), "::MAX` or `self * rhs < ", stringify!($SelfT), "::MIN`,")]
        /// i.e. when [`checked_mul`] would return `None`.
        ///
        #[doc = concat!("[`checked_mul`]: ", stringify!($SelfT), "::checked_mul")]
        #[unstable(
            feature = "unchecked_math",
            reason = "niche optimization path",
            issue = "85122",
        )]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[crablangc_const_unstable(feature = "const_inherent_unchecked_arith", issue = "85122")]
        #[inline(always)]
        #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
        pub const unsafe fn unchecked_mul(self, rhs: Self) -> Self {
            // SAFETY: the caller must uphold the safety contract for
            // `unchecked_mul`.
            unsafe { intrinsics::unchecked_mul(self, rhs) }
        }

        /// Checked integer division. Computes `self / rhs`, returning `None`
        /// if `rhs == 0`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(128", stringify!($SelfT), ".checked_div(2), Some(64));")]
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".checked_div(0), None);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_checked_int_div", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_div(self, rhs: Self) -> Option<Self> {
            if unlikely!(rhs == 0) {
                None
            } else {
                // SAFETY: div by zero has been checked above and unsigned types have no other
                // failure modes for division
                Some(unsafe { intrinsics::unchecked_div(self, rhs) })
            }
        }

        /// Checked Euclidean division. Computes `self.div_euclid(rhs)`, returning `None`
        /// if `rhs == 0`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(128", stringify!($SelfT), ".checked_div_euclid(2), Some(64));")]
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".checked_div_euclid(0), None);")]
        /// ```
        #[stable(feature = "euclidean_division", since = "1.38.0")]
        #[crablangc_const_stable(feature = "const_euclidean_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_div_euclid(self, rhs: Self) -> Option<Self> {
            if unlikely!(rhs == 0) {
                None
            } else {
                Some(self.div_euclid(rhs))
            }
        }


        /// Checked integer remainder. Computes `self % rhs`, returning `None`
        /// if `rhs == 0`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".checked_rem(2), Some(1));")]
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".checked_rem(0), None);")]
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_checked_int_div", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_rem(self, rhs: Self) -> Option<Self> {
            if unlikely!(rhs == 0) {
                None
            } else {
                // SAFETY: div by zero has been checked above and unsigned types have no other
                // failure modes for division
                Some(unsafe { intrinsics::unchecked_rem(self, rhs) })
            }
        }

        /// Checked Euclidean modulo. Computes `self.rem_euclid(rhs)`, returning `None`
        /// if `rhs == 0`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".checked_rem_euclid(2), Some(1));")]
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".checked_rem_euclid(0), None);")]
        /// ```
        #[stable(feature = "euclidean_division", since = "1.38.0")]
        #[crablangc_const_stable(feature = "const_euclidean_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_rem_euclid(self, rhs: Self) -> Option<Self> {
            if unlikely!(rhs == 0) {
                None
            } else {
                Some(self.rem_euclid(rhs))
            }
        }

        /// Returns the logarithm of the number with respect to an arbitrary base,
        /// rounded down.
        ///
        /// This method might not be optimized owing to implementation details;
        /// `ilog2` can produce results more efficiently for base 2, and `ilog10`
        /// can produce results more efficiently for base 10.
        ///
        /// # Panics
        ///
        /// This function will panic if `self` is zero, or if `base` is less than 2.
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".ilog(5), 1);")]
        /// ```
        #[stable(feature = "int_log", since = "1.67.0")]
        #[crablangc_const_stable(feature = "int_log", since = "1.67.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        #[track_caller]
        pub const fn ilog(self, base: Self) -> u32 {
            assert!(base >= 2, "base of integer logarithm must be at least 2");
            if let Some(log) = self.checked_ilog(base) {
                log
            } else {
                int_log10::panic_for_nonpositive_argument()
            }
        }

        /// Returns the base 2 logarithm of the number, rounded down.
        ///
        /// # Panics
        ///
        /// This function will panic if `self` is zero.
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("assert_eq!(2", stringify!($SelfT), ".ilog2(), 1);")]
        /// ```
        #[stable(feature = "int_log", since = "1.67.0")]
        #[crablangc_const_stable(feature = "int_log", since = "1.67.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        #[track_caller]
        pub const fn ilog2(self) -> u32 {
            if let Some(log) = self.checked_ilog2() {
                log
            } else {
                int_log10::panic_for_nonpositive_argument()
            }
        }

        /// Returns the base 10 logarithm of the number, rounded down.
        ///
        /// # Panics
        ///
        /// This function will panic if `self` is zero.
        ///
        /// # Example
        ///
        /// ```
        #[doc = concat!("assert_eq!(10", stringify!($SelfT), ".ilog10(), 1);")]
        /// ```
        #[stable(feature = "int_log", since = "1.67.0")]
        #[crablangc_const_stable(feature = "int_log", since = "1.67.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        #[track_caller]
        pub const fn ilog10(self) -> u32 {
            if let Some(log) = self.checked_ilog10() {
                log
            } else {
                int_log10::panic_for_nonpositive_argument()
            }
        }

        /// Returns the logarithm of the number with respect to an arbitrary base,
        /// rounded down.
        ///
        /// Returns `None` if the number is zero, or if the base is not at least 2.
        ///
        /// This method might not be optimized owing to implementation details;
        /// `checked_ilog2` can produce results more efficiently for base 2, and
        /// `checked_ilog10` can produce results more efficiently for base 10.
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".checked_ilog(5), Some(1));")]
        /// ```
        #[stable(feature = "int_log", since = "1.67.0")]
        #[crablangc_const_stable(feature = "int_log", since = "1.67.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_ilog(self, base: Self) -> Option<u32> {
            if self <= 0 || base <= 1 {
                None
            } else {
                let mut n = 0;
                let mut r = self;

                // Optimization for 128 bit wide integers.
                if Self::BITS == 128 {
                    let b = Self::ilog2(self) / (Self::ilog2(base) + 1);
                    n += b;
                    r /= base.pow(b as u32);
                }

                while r >= base {
                    r /= base;
                    n += 1;
                }
                Some(n)
            }
        }

        /// Returns the base 2 logarithm of the number, rounded down.
        ///
        /// Returns `None` if the number is zero.
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("assert_eq!(2", stringify!($SelfT), ".checked_ilog2(), Some(1));")]
        /// ```
        #[stable(feature = "int_log", since = "1.67.0")]
        #[crablangc_const_stable(feature = "int_log", since = "1.67.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_ilog2(self) -> Option<u32> {
            if let Some(x) = <$NonZeroT>::new(self) {
                Some(x.ilog2())
            } else {
                None
            }
        }

        /// Returns the base 10 logarithm of the number, rounded down.
        ///
        /// Returns `None` if the number is zero.
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("assert_eq!(10", stringify!($SelfT), ".checked_ilog10(), Some(1));")]
        /// ```
        #[stable(feature = "int_log", since = "1.67.0")]
        #[crablangc_const_stable(feature = "int_log", since = "1.67.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_ilog10(self) -> Option<u32> {
            if let Some(x) = <$NonZeroT>::new(self) {
                Some(x.ilog10())
            } else {
                None
            }
        }

        /// Checked negation. Computes `-self`, returning `None` unless `self ==
        /// 0`.
        ///
        /// Note that negating any positive integer will overflow.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(0", stringify!($SelfT), ".checked_neg(), Some(0));")]
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".checked_neg(), None);")]
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_checked_int_methods", since = "1.47.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_neg(self) -> Option<Self> {
            let (a, b) = self.overflowing_neg();
            if unlikely!(b) {None} else {Some(a)}
        }

        /// Checked shift left. Computes `self << rhs`, returning `None`
        /// if `rhs` is larger than or equal to the number of bits in `self`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(0x1", stringify!($SelfT), ".checked_shl(4), Some(0x10));")]
        #[doc = concat!("assert_eq!(0x10", stringify!($SelfT), ".checked_shl(129), None);")]
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_checked_int_methods", since = "1.47.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_shl(self, rhs: u32) -> Option<Self> {
            let (a, b) = self.overflowing_shl(rhs);
            if unlikely!(b) {None} else {Some(a)}
        }

        /// Unchecked shift left. Computes `self << rhs`, assuming that
        /// `rhs` is less than the number of bits in `self`.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior if `rhs` is larger than
        /// or equal to the number of bits in `self`,
        /// i.e. when [`checked_shl`] would return `None`.
        ///
        #[doc = concat!("[`checked_shl`]: ", stringify!($SelfT), "::checked_shl")]
        #[unstable(
            feature = "unchecked_math",
            reason = "niche optimization path",
            issue = "85122",
        )]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[crablangc_const_unstable(feature = "const_inherent_unchecked_arith", issue = "85122")]
        #[inline(always)]
        #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
        pub const unsafe fn unchecked_shl(self, rhs: u32) -> Self {
            // SAFETY: the caller must uphold the safety contract for
            // `unchecked_shl`.
            // Any legal shift amount is losslessly representable in the self type.
            unsafe { intrinsics::unchecked_shl(self, rhs.try_into().ok().unwrap_unchecked()) }
        }

        /// Checked shift right. Computes `self >> rhs`, returning `None`
        /// if `rhs` is larger than or equal to the number of bits in `self`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(0x10", stringify!($SelfT), ".checked_shr(4), Some(0x1));")]
        #[doc = concat!("assert_eq!(0x10", stringify!($SelfT), ".checked_shr(129), None);")]
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_checked_int_methods", since = "1.47.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_shr(self, rhs: u32) -> Option<Self> {
            let (a, b) = self.overflowing_shr(rhs);
            if unlikely!(b) {None} else {Some(a)}
        }

        /// Unchecked shift right. Computes `self >> rhs`, assuming that
        /// `rhs` is less than the number of bits in `self`.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior if `rhs` is larger than
        /// or equal to the number of bits in `self`,
        /// i.e. when [`checked_shr`] would return `None`.
        ///
        #[doc = concat!("[`checked_shr`]: ", stringify!($SelfT), "::checked_shr")]
        #[unstable(
            feature = "unchecked_math",
            reason = "niche optimization path",
            issue = "85122",
        )]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[crablangc_const_unstable(feature = "const_inherent_unchecked_arith", issue = "85122")]
        #[inline(always)]
        #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
        pub const unsafe fn unchecked_shr(self, rhs: u32) -> Self {
            // SAFETY: the caller must uphold the safety contract for
            // `unchecked_shr`.
            // Any legal shift amount is losslessly representable in the self type.
            unsafe { intrinsics::unchecked_shr(self, rhs.try_into().ok().unwrap_unchecked()) }
        }

        /// Checked exponentiation. Computes `self.pow(exp)`, returning `None` if
        /// overflow occurred.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(2", stringify!($SelfT), ".checked_pow(5), Some(32));")]
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.checked_pow(2), None);")]
        /// ```
        #[stable(feature = "no_panic_pow", since = "1.34.0")]
        #[crablangc_const_stable(feature = "const_int_pow", since = "1.50.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_pow(self, mut exp: u32) -> Option<Self> {
            if exp == 0 {
                return Some(1);
            }
            let mut base = self;
            let mut acc: Self = 1;

            while exp > 1 {
                if (exp & 1) == 1 {
                    acc = try_opt!(acc.checked_mul(base));
                }
                exp /= 2;
                base = try_opt!(base.checked_mul(base));
            }

            // since exp!=0, finally the exp must be 1.
            // Deal with the final bit of the exponent separately, since
            // squaring the base afterwards is not necessary and may cause a
            // needless overflow.

            acc.checked_mul(base)
        }

        /// Saturating integer addition. Computes `self + rhs`, saturating at
        /// the numeric bounds instead of overflowing.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".saturating_add(1), 101);")]
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.saturating_add(127), ", stringify!($SelfT), "::MAX);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[crablangc_const_stable(feature = "const_saturating_int_methods", since = "1.47.0")]
        #[inline(always)]
        pub const fn saturating_add(self, rhs: Self) -> Self {
            intrinsics::saturating_add(self, rhs)
        }

        /// Saturating addition with a signed integer. Computes `self + rhs`,
        /// saturating at the numeric bounds instead of overflowing.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".saturating_add_signed(2), 3);")]
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".saturating_add_signed(-2), 0);")]
        #[doc = concat!("assert_eq!((", stringify!($SelfT), "::MAX - 2).saturating_add_signed(4), ", stringify!($SelfT), "::MAX);")]
        /// ```
        #[stable(feature = "mixed_integer_ops", since = "1.66.0")]
        #[crablangc_const_stable(feature = "mixed_integer_ops", since = "1.66.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn saturating_add_signed(self, rhs: $SignedT) -> Self {
            let (res, overflow) = self.overflowing_add(rhs as Self);
            if overflow == (rhs < 0) {
                res
            } else if overflow {
                Self::MAX
            } else {
                0
            }
        }

        /// Saturating integer subtraction. Computes `self - rhs`, saturating
        /// at the numeric bounds instead of overflowing.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".saturating_sub(27), 73);")]
        #[doc = concat!("assert_eq!(13", stringify!($SelfT), ".saturating_sub(127), 0);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[crablangc_const_stable(feature = "const_saturating_int_methods", since = "1.47.0")]
        #[inline(always)]
        pub const fn saturating_sub(self, rhs: Self) -> Self {
            intrinsics::saturating_sub(self, rhs)
        }

        /// Saturating integer multiplication. Computes `self * rhs`,
        /// saturating at the numeric bounds instead of overflowing.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(2", stringify!($SelfT), ".saturating_mul(10), 20);")]
        #[doc = concat!("assert_eq!((", stringify!($SelfT), "::MAX).saturating_mul(10), ", stringify!($SelfT),"::MAX);")]
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_saturating_int_methods", since = "1.47.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn saturating_mul(self, rhs: Self) -> Self {
            match self.checked_mul(rhs) {
                Some(x) => x,
                None => Self::MAX,
            }
        }

        /// Saturating integer division. Computes `self / rhs`, saturating at the
        /// numeric bounds instead of overflowing.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".saturating_div(2), 2);")]
        ///
        /// ```
        ///
        /// ```should_panic
        #[doc = concat!("let _ = 1", stringify!($SelfT), ".saturating_div(0);")]
        ///
        /// ```
        #[stable(feature = "saturating_div", since = "1.58.0")]
        #[crablangc_const_stable(feature = "saturating_div", since = "1.58.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn saturating_div(self, rhs: Self) -> Self {
            // on unsigned types, there is no overflow in integer division
            self.wrapping_div(rhs)
        }

        /// Saturating integer exponentiation. Computes `self.pow(exp)`,
        /// saturating at the numeric bounds instead of overflowing.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(4", stringify!($SelfT), ".saturating_pow(3), 64);")]
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.saturating_pow(2), ", stringify!($SelfT), "::MAX);")]
        /// ```
        #[stable(feature = "no_panic_pow", since = "1.34.0")]
        #[crablangc_const_stable(feature = "const_int_pow", since = "1.50.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn saturating_pow(self, exp: u32) -> Self {
            match self.checked_pow(exp) {
                Some(x) => x,
                None => Self::MAX,
            }
        }

        /// Wrapping (modular) addition. Computes `self + rhs`,
        /// wrapping around at the boundary of the type.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(200", stringify!($SelfT), ".wrapping_add(55), 255);")]
        #[doc = concat!("assert_eq!(200", stringify!($SelfT), ".wrapping_add(", stringify!($SelfT), "::MAX), 199);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn wrapping_add(self, rhs: Self) -> Self {
            intrinsics::wrapping_add(self, rhs)
        }

        /// Wrapping (modular) addition with a signed integer. Computes
        /// `self + rhs`, wrapping around at the boundary of the type.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".wrapping_add_signed(2), 3);")]
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".wrapping_add_signed(-2), ", stringify!($SelfT), "::MAX);")]
        #[doc = concat!("assert_eq!((", stringify!($SelfT), "::MAX - 2).wrapping_add_signed(4), 1);")]
        /// ```
        #[stable(feature = "mixed_integer_ops", since = "1.66.0")]
        #[crablangc_const_stable(feature = "mixed_integer_ops", since = "1.66.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn wrapping_add_signed(self, rhs: $SignedT) -> Self {
            self.wrapping_add(rhs as Self)
        }

        /// Wrapping (modular) subtraction. Computes `self - rhs`,
        /// wrapping around at the boundary of the type.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".wrapping_sub(100), 0);")]
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".wrapping_sub(", stringify!($SelfT), "::MAX), 101);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn wrapping_sub(self, rhs: Self) -> Self {
            intrinsics::wrapping_sub(self, rhs)
        }

        /// Wrapping (modular) multiplication. Computes `self *
        /// rhs`, wrapping around at the boundary of the type.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// Please note that this example is shared between integer types.
        /// Which explains why `u8` is used here.
        ///
        /// ```
        /// assert_eq!(10u8.wrapping_mul(12), 120);
        /// assert_eq!(25u8.wrapping_mul(12), 44);
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn wrapping_mul(self, rhs: Self) -> Self {
            intrinsics::wrapping_mul(self, rhs)
        }

        /// Wrapping (modular) division. Computes `self / rhs`.
        /// Wrapped division on unsigned types is just normal division.
        /// There's no way wrapping could ever happen.
        /// This function exists, so that all operations
        /// are accounted for in the wrapping operations.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".wrapping_div(10), 10);")]
        /// ```
        #[stable(feature = "num_wrapping", since = "1.2.0")]
        #[crablangc_const_stable(feature = "const_wrapping_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn wrapping_div(self, rhs: Self) -> Self {
            self / rhs
        }

        /// Wrapping Euclidean division. Computes `self.div_euclid(rhs)`.
        /// Wrapped division on unsigned types is just normal division.
        /// There's no way wrapping could ever happen.
        /// This function exists, so that all operations
        /// are accounted for in the wrapping operations.
        /// Since, for the positive integers, all common
        /// definitions of division are equal, this
        /// is exactly equal to `self.wrapping_div(rhs)`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".wrapping_div_euclid(10), 10);")]
        /// ```
        #[stable(feature = "euclidean_division", since = "1.38.0")]
        #[crablangc_const_stable(feature = "const_euclidean_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn wrapping_div_euclid(self, rhs: Self) -> Self {
            self / rhs
        }

        /// Wrapping (modular) remainder. Computes `self % rhs`.
        /// Wrapped remainder calculation on unsigned types is
        /// just the regular remainder calculation.
        /// There's no way wrapping could ever happen.
        /// This function exists, so that all operations
        /// are accounted for in the wrapping operations.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".wrapping_rem(10), 0);")]
        /// ```
        #[stable(feature = "num_wrapping", since = "1.2.0")]
        #[crablangc_const_stable(feature = "const_wrapping_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn wrapping_rem(self, rhs: Self) -> Self {
            self % rhs
        }

        /// Wrapping Euclidean modulo. Computes `self.rem_euclid(rhs)`.
        /// Wrapped modulo calculation on unsigned types is
        /// just the regular remainder calculation.
        /// There's no way wrapping could ever happen.
        /// This function exists, so that all operations
        /// are accounted for in the wrapping operations.
        /// Since, for the positive integers, all common
        /// definitions of division are equal, this
        /// is exactly equal to `self.wrapping_rem(rhs)`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".wrapping_rem_euclid(10), 0);")]
        /// ```
        #[stable(feature = "euclidean_division", since = "1.38.0")]
        #[crablangc_const_stable(feature = "const_euclidean_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn wrapping_rem_euclid(self, rhs: Self) -> Self {
            self % rhs
        }

        /// Wrapping (modular) negation. Computes `-self`,
        /// wrapping around at the boundary of the type.
        ///
        /// Since unsigned types do not have negative equivalents
        /// all applications of this function will wrap (except for `-0`).
        /// For values smaller than the corresponding signed type's maximum
        /// the result is the same as casting the corresponding signed value.
        /// Any larger values are equivalent to `MAX + 1 - (val - MAX - 1)` where
        /// `MAX` is the corresponding signed type's maximum.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(0_", stringify!($SelfT), ".wrapping_neg(), 0);")]
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.wrapping_neg(), 1);")]
        #[doc = concat!("assert_eq!(13_", stringify!($SelfT), ".wrapping_neg(), (!13) + 1);")]
        #[doc = concat!("assert_eq!(42_", stringify!($SelfT), ".wrapping_neg(), !(42 - 1));")]
        /// ```
        #[stable(feature = "num_wrapping", since = "1.2.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn wrapping_neg(self) -> Self {
            (0 as $SelfT).wrapping_sub(self)
        }

        /// Panic-free bitwise shift-left; yields `self << mask(rhs)`,
        /// where `mask` removes any high-order bits of `rhs` that
        /// would cause the shift to exceed the bitwidth of the type.
        ///
        /// Note that this is *not* the same as a rotate-left; the
        /// RHS of a wrapping shift-left is restricted to the range
        /// of the type, rather than the bits shifted out of the LHS
        /// being returned to the other end. The primitive integer
        /// types all implement a [`rotate_left`](Self::rotate_left) function,
        /// which may be what you want instead.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".wrapping_shl(7), 128);")]
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".wrapping_shl(128), 1);")]
        /// ```
        #[stable(feature = "num_wrapping", since = "1.2.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        #[crablangc_allow_const_fn_unstable(const_inherent_unchecked_arith)]
        pub const fn wrapping_shl(self, rhs: u32) -> Self {
            // SAFETY: the masking by the bitsize of the type ensures that we do not shift
            // out of bounds
            unsafe {
                self.unchecked_shl(rhs & (Self::BITS - 1))
            }
        }

        /// Panic-free bitwise shift-right; yields `self >> mask(rhs)`,
        /// where `mask` removes any high-order bits of `rhs` that
        /// would cause the shift to exceed the bitwidth of the type.
        ///
        /// Note that this is *not* the same as a rotate-right; the
        /// RHS of a wrapping shift-right is restricted to the range
        /// of the type, rather than the bits shifted out of the LHS
        /// being returned to the other end. The primitive integer
        /// types all implement a [`rotate_right`](Self::rotate_right) function,
        /// which may be what you want instead.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(128", stringify!($SelfT), ".wrapping_shr(7), 1);")]
        #[doc = concat!("assert_eq!(128", stringify!($SelfT), ".wrapping_shr(128), 128);")]
        /// ```
        #[stable(feature = "num_wrapping", since = "1.2.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        #[crablangc_allow_const_fn_unstable(const_inherent_unchecked_arith)]
        pub const fn wrapping_shr(self, rhs: u32) -> Self {
            // SAFETY: the masking by the bitsize of the type ensures that we do not shift
            // out of bounds
            unsafe {
                self.unchecked_shr(rhs & (Self::BITS - 1))
            }
        }

        /// Wrapping (modular) exponentiation. Computes `self.pow(exp)`,
        /// wrapping around at the boundary of the type.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(3", stringify!($SelfT), ".wrapping_pow(5), 243);")]
        /// assert_eq!(3u8.wrapping_pow(6), 217);
        /// ```
        #[stable(feature = "no_panic_pow", since = "1.34.0")]
        #[crablangc_const_stable(feature = "const_int_pow", since = "1.50.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn wrapping_pow(self, mut exp: u32) -> Self {
            if exp == 0 {
                return 1;
            }
            let mut base = self;
            let mut acc: Self = 1;

            while exp > 1 {
                if (exp & 1) == 1 {
                    acc = acc.wrapping_mul(base);
                }
                exp /= 2;
                base = base.wrapping_mul(base);
            }

            // since exp!=0, finally the exp must be 1.
            // Deal with the final bit of the exponent separately, since
            // squaring the base afterwards is not necessary and may cause a
            // needless overflow.
            acc.wrapping_mul(base)
        }

        /// Calculates `self` + `rhs`
        ///
        /// Returns a tuple of the addition along with a boolean indicating
        /// whether an arithmetic overflow would occur. If an overflow would
        /// have occurred then the wrapped value is returned.
        ///
        /// # Examples
        ///
        /// Basic usage
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".overflowing_add(2), (7, false));")]
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.overflowing_add(1), (0, true));")]
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
            let (a, b) = intrinsics::add_with_overflow(self as $ActualT, rhs as $ActualT);
            (a as Self, b)
        }

        /// Calculates `self` + `rhs` + `carry` and returns a tuple containing
        /// the sum and the output carry.
        ///
        /// Performs "ternary addition" of two integer operands and a carry-in
        /// bit, and returns an output integer and a carry-out bit. This allows
        /// chaining together multiple additions to create a wider addition, and
        /// can be useful for bignum addition.
        ///
        #[doc = concat!("This can be thought of as a ", stringify!($BITS), "-bit \"full adder\", in the electronics sense.")]
        ///
        /// If the input carry is false, this method is equivalent to
        /// [`overflowing_add`](Self::overflowing_add), and the output carry is
        /// equal to the overflow flag. Note that although carry and overflow
        /// flags are similar for unsigned integers, they are different for
        /// signed integers.
        ///
        /// # Examples
        ///
        /// ```
        /// #![feature(bigint_helper_methods)]
        ///
        #[doc = concat!("//    3  MAX    (a = 3 × 2^", stringify!($BITS), " + 2^", stringify!($BITS), " - 1)")]
        #[doc = concat!("// +  5    7    (b = 5 × 2^", stringify!($BITS), " + 7)")]
        /// // ---------
        #[doc = concat!("//    9    6    (sum = 9 × 2^", stringify!($BITS), " + 6)")]
        ///
        #[doc = concat!("let (a1, a0): (", stringify!($SelfT), ", ", stringify!($SelfT), ") = (3, ", stringify!($SelfT), "::MAX);")]
        #[doc = concat!("let (b1, b0): (", stringify!($SelfT), ", ", stringify!($SelfT), ") = (5, 7);")]
        /// let carry0 = false;
        ///
        /// let (sum0, carry1) = a0.carrying_add(b0, carry0);
        /// assert_eq!(carry1, true);
        /// let (sum1, carry2) = a1.carrying_add(b1, carry1);
        /// assert_eq!(carry2, false);
        ///
        /// assert_eq!((sum1, sum0), (9, 6));
        /// ```
        #[unstable(feature = "bigint_helper_methods", issue = "85532")]
        #[crablangc_const_unstable(feature = "const_bigint_helper_methods", issue = "85532")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
            // note: longer-term this should be done via an intrinsic, but this has been shown
            //   to generate optimal code for now, and LLVM doesn't have an equivalent intrinsic
            let (a, b) = self.overflowing_add(rhs);
            let (c, d) = a.overflowing_add(carry as $SelfT);
            (c, b || d)
        }

        /// Calculates `self` + `rhs` with a signed `rhs`
        ///
        /// Returns a tuple of the addition along with a boolean indicating
        /// whether an arithmetic overflow would occur. If an overflow would
        /// have occurred then the wrapped value is returned.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".overflowing_add_signed(2), (3, false));")]
        #[doc = concat!("assert_eq!(1", stringify!($SelfT), ".overflowing_add_signed(-2), (", stringify!($SelfT), "::MAX, true));")]
        #[doc = concat!("assert_eq!((", stringify!($SelfT), "::MAX - 2).overflowing_add_signed(4), (1, true));")]
        /// ```
        #[stable(feature = "mixed_integer_ops", since = "1.66.0")]
        #[crablangc_const_stable(feature = "mixed_integer_ops", since = "1.66.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn overflowing_add_signed(self, rhs: $SignedT) -> (Self, bool) {
            let (res, overflowed) = self.overflowing_add(rhs as Self);
            (res, overflowed ^ (rhs < 0))
        }

        /// Calculates `self` - `rhs`
        ///
        /// Returns a tuple of the subtraction along with a boolean indicating
        /// whether an arithmetic overflow would occur. If an overflow would
        /// have occurred then the wrapped value is returned.
        ///
        /// # Examples
        ///
        /// Basic usage
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".overflowing_sub(2), (3, false));")]
        #[doc = concat!("assert_eq!(0", stringify!($SelfT), ".overflowing_sub(1), (", stringify!($SelfT), "::MAX, true));")]
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
            let (a, b) = intrinsics::sub_with_overflow(self as $ActualT, rhs as $ActualT);
            (a as Self, b)
        }

        /// Calculates `self` &minus; `rhs` &minus; `borrow` and returns a tuple
        /// containing the difference and the output borrow.
        ///
        /// Performs "ternary subtraction" by subtracting both an integer
        /// operand and a borrow-in bit from `self`, and returns an output
        /// integer and a borrow-out bit. This allows chaining together multiple
        /// subtractions to create a wider subtraction, and can be useful for
        /// bignum subtraction.
        ///
        /// # Examples
        ///
        /// ```
        /// #![feature(bigint_helper_methods)]
        ///
        #[doc = concat!("//    9    6    (a = 9 × 2^", stringify!($BITS), " + 6)")]
        #[doc = concat!("// -  5    7    (b = 5 × 2^", stringify!($BITS), " + 7)")]
        /// // ---------
        #[doc = concat!("//    3  MAX    (diff = 3 × 2^", stringify!($BITS), " + 2^", stringify!($BITS), " - 1)")]
        ///
        #[doc = concat!("let (a1, a0): (", stringify!($SelfT), ", ", stringify!($SelfT), ") = (9, 6);")]
        #[doc = concat!("let (b1, b0): (", stringify!($SelfT), ", ", stringify!($SelfT), ") = (5, 7);")]
        /// let borrow0 = false;
        ///
        /// let (diff0, borrow1) = a0.borrowing_sub(b0, borrow0);
        /// assert_eq!(borrow1, true);
        /// let (diff1, borrow2) = a1.borrowing_sub(b1, borrow1);
        /// assert_eq!(borrow2, false);
        ///
        #[doc = concat!("assert_eq!((diff1, diff0), (3, ", stringify!($SelfT), "::MAX));")]
        /// ```
        #[unstable(feature = "bigint_helper_methods", issue = "85532")]
        #[crablangc_const_unstable(feature = "const_bigint_helper_methods", issue = "85532")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
            // note: longer-term this should be done via an intrinsic, but this has been shown
            //   to generate optimal code for now, and LLVM doesn't have an equivalent intrinsic
            let (a, b) = self.overflowing_sub(rhs);
            let (c, d) = a.overflowing_sub(borrow as $SelfT);
            (c, b || d)
        }

        /// Computes the absolute difference between `self` and `other`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".abs_diff(80), 20", stringify!($SelfT), ");")]
        #[doc = concat!("assert_eq!(100", stringify!($SelfT), ".abs_diff(110), 10", stringify!($SelfT), ");")]
        /// ```
        #[stable(feature = "int_abs_diff", since = "1.60.0")]
        #[crablangc_const_stable(feature = "int_abs_diff", since = "1.60.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn abs_diff(self, other: Self) -> Self {
            if mem::size_of::<Self>() == 1 {
                // Trick LLVM into generating the psadbw instruction when SSE2
                // is available and this function is autovectorized for u8's.
                (self as i32).wrapping_sub(other as i32).abs() as Self
            } else {
                if self < other {
                    other - self
                } else {
                    self - other
                }
            }
        }

        /// Calculates the multiplication of `self` and `rhs`.
        ///
        /// Returns a tuple of the multiplication along with a boolean
        /// indicating whether an arithmetic overflow would occur. If an
        /// overflow would have occurred then the wrapped value is returned.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// Please note that this example is shared between integer types.
        /// Which explains why `u32` is used here.
        ///
        /// ```
        /// assert_eq!(5u32.overflowing_mul(2), (10, false));
        /// assert_eq!(1_000_000_000u32.overflowing_mul(10), (1410065408, true));
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                          without modifying the original"]
        #[inline(always)]
        pub const fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
            let (a, b) = intrinsics::mul_with_overflow(self as $ActualT, rhs as $ActualT);
            (a as Self, b)
        }

        /// Calculates the divisor when `self` is divided by `rhs`.
        ///
        /// Returns a tuple of the divisor along with a boolean indicating
        /// whether an arithmetic overflow would occur. Note that for unsigned
        /// integers overflow never occurs, so the second value is always
        /// `false`.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is 0.
        ///
        /// # Examples
        ///
        /// Basic usage
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".overflowing_div(2), (2, false));")]
        /// ```
        #[inline(always)]
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_overflowing_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        pub const fn overflowing_div(self, rhs: Self) -> (Self, bool) {
            (self / rhs, false)
        }

        /// Calculates the quotient of Euclidean division `self.div_euclid(rhs)`.
        ///
        /// Returns a tuple of the divisor along with a boolean indicating
        /// whether an arithmetic overflow would occur. Note that for unsigned
        /// integers overflow never occurs, so the second value is always
        /// `false`.
        /// Since, for the positive integers, all common
        /// definitions of division are equal, this
        /// is exactly equal to `self.overflowing_div(rhs)`.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is 0.
        ///
        /// # Examples
        ///
        /// Basic usage
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".overflowing_div_euclid(2), (2, false));")]
        /// ```
        #[inline(always)]
        #[stable(feature = "euclidean_division", since = "1.38.0")]
        #[crablangc_const_stable(feature = "const_euclidean_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        pub const fn overflowing_div_euclid(self, rhs: Self) -> (Self, bool) {
            (self / rhs, false)
        }

        /// Calculates the remainder when `self` is divided by `rhs`.
        ///
        /// Returns a tuple of the remainder after dividing along with a boolean
        /// indicating whether an arithmetic overflow would occur. Note that for
        /// unsigned integers overflow never occurs, so the second value is
        /// always `false`.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is 0.
        ///
        /// # Examples
        ///
        /// Basic usage
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".overflowing_rem(2), (1, false));")]
        /// ```
        #[inline(always)]
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_overflowing_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        pub const fn overflowing_rem(self, rhs: Self) -> (Self, bool) {
            (self % rhs, false)
        }

        /// Calculates the remainder `self.rem_euclid(rhs)` as if by Euclidean division.
        ///
        /// Returns a tuple of the modulo after dividing along with a boolean
        /// indicating whether an arithmetic overflow would occur. Note that for
        /// unsigned integers overflow never occurs, so the second value is
        /// always `false`.
        /// Since, for the positive integers, all common
        /// definitions of division are equal, this operation
        /// is exactly equal to `self.overflowing_rem(rhs)`.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is 0.
        ///
        /// # Examples
        ///
        /// Basic usage
        ///
        /// ```
        #[doc = concat!("assert_eq!(5", stringify!($SelfT), ".overflowing_rem_euclid(2), (1, false));")]
        /// ```
        #[inline(always)]
        #[stable(feature = "euclidean_division", since = "1.38.0")]
        #[crablangc_const_stable(feature = "const_euclidean_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        pub const fn overflowing_rem_euclid(self, rhs: Self) -> (Self, bool) {
            (self % rhs, false)
        }

        /// Negates self in an overflowing fashion.
        ///
        /// Returns `!self + 1` using wrapping operations to return the value
        /// that represents the negation of this unsigned value. Note that for
        /// positive unsigned values overflow always occurs, but negating 0 does
        /// not overflow.
        ///
        /// # Examples
        ///
        /// Basic usage
        ///
        /// ```
        #[doc = concat!("assert_eq!(0", stringify!($SelfT), ".overflowing_neg(), (0, false));")]
        #[doc = concat!("assert_eq!(2", stringify!($SelfT), ".overflowing_neg(), (-2i32 as ", stringify!($SelfT), ", true));")]
        /// ```
        #[inline(always)]
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        pub const fn overflowing_neg(self) -> (Self, bool) {
            ((!self).wrapping_add(1), self != 0)
        }

        /// Shifts self left by `rhs` bits.
        ///
        /// Returns a tuple of the shifted version of self along with a boolean
        /// indicating whether the shift value was larger than or equal to the
        /// number of bits. If the shift value is too large, then value is
        /// masked (N-1) where N is the number of bits, and this value is then
        /// used to perform the shift.
        ///
        /// # Examples
        ///
        /// Basic usage
        ///
        /// ```
        #[doc = concat!("assert_eq!(0x1", stringify!($SelfT), ".overflowing_shl(4), (0x10, false));")]
        #[doc = concat!("assert_eq!(0x1", stringify!($SelfT), ".overflowing_shl(132), (0x10, true));")]
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn overflowing_shl(self, rhs: u32) -> (Self, bool) {
            (self.wrapping_shl(rhs), rhs >= Self::BITS)
        }

        /// Shifts self right by `rhs` bits.
        ///
        /// Returns a tuple of the shifted version of self along with a boolean
        /// indicating whether the shift value was larger than or equal to the
        /// number of bits. If the shift value is too large, then value is
        /// masked (N-1) where N is the number of bits, and this value is then
        /// used to perform the shift.
        ///
        /// # Examples
        ///
        /// Basic usage
        ///
        /// ```
        #[doc = concat!("assert_eq!(0x10", stringify!($SelfT), ".overflowing_shr(4), (0x1, false));")]
        #[doc = concat!("assert_eq!(0x10", stringify!($SelfT), ".overflowing_shr(132), (0x1, true));")]
        /// ```
        #[stable(feature = "wrapping", since = "1.7.0")]
        #[crablangc_const_stable(feature = "const_wrapping_math", since = "1.32.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn overflowing_shr(self, rhs: u32) -> (Self, bool) {
            (self.wrapping_shr(rhs), rhs >= Self::BITS)
        }

        /// Raises self to the power of `exp`, using exponentiation by squaring.
        ///
        /// Returns a tuple of the exponentiation along with a bool indicating
        /// whether an overflow happened.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(3", stringify!($SelfT), ".overflowing_pow(5), (243, false));")]
        /// assert_eq!(3u8.overflowing_pow(6), (217, true));
        /// ```
        #[stable(feature = "no_panic_pow", since = "1.34.0")]
        #[crablangc_const_stable(feature = "const_int_pow", since = "1.50.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn overflowing_pow(self, mut exp: u32) -> (Self, bool) {
            if exp == 0{
                return (1,false);
            }
            let mut base = self;
            let mut acc: Self = 1;
            let mut overflown = false;
            // Scratch space for storing results of overflowing_mul.
            let mut r;

            while exp > 1 {
                if (exp & 1) == 1 {
                    r = acc.overflowing_mul(base);
                    acc = r.0;
                    overflown |= r.1;
                }
                exp /= 2;
                r = base.overflowing_mul(base);
                base = r.0;
                overflown |= r.1;
            }

            // since exp!=0, finally the exp must be 1.
            // Deal with the final bit of the exponent separately, since
            // squaring the base afterwards is not necessary and may cause a
            // needless overflow.
            r = acc.overflowing_mul(base);
            r.1 |= overflown;

            r
        }

        /// Raises self to the power of `exp`, using exponentiation by squaring.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(2", stringify!($SelfT), ".pow(5), 32);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_int_pow", since = "1.50.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        #[crablangc_inherit_overflow_checks]
        pub const fn pow(self, mut exp: u32) -> Self {
            if exp == 0 {
                return 1;
            }
            let mut base = self;
            let mut acc = 1;

            while exp > 1 {
                if (exp & 1) == 1 {
                    acc = acc * base;
                }
                exp /= 2;
                base = base * base;
            }

            // since exp!=0, finally the exp must be 1.
            // Deal with the final bit of the exponent separately, since
            // squaring the base afterwards is not necessary and may cause a
            // needless overflow.
            acc * base
        }

        /// Performs Euclidean division.
        ///
        /// Since, for the positive integers, all common
        /// definitions of division are equal, this
        /// is exactly equal to `self / rhs`.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is 0.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(7", stringify!($SelfT), ".div_euclid(4), 1); // or any other integer type")]
        /// ```
        #[stable(feature = "euclidean_division", since = "1.38.0")]
        #[crablangc_const_stable(feature = "const_euclidean_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        #[crablangc_inherit_overflow_checks]
        pub const fn div_euclid(self, rhs: Self) -> Self {
            self / rhs
        }


        /// Calculates the least remainder of `self (mod rhs)`.
        ///
        /// Since, for the positive integers, all common
        /// definitions of division are equal, this
        /// is exactly equal to `self % rhs`.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is 0.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(7", stringify!($SelfT), ".rem_euclid(4), 3); // or any other integer type")]
        /// ```
        #[stable(feature = "euclidean_division", since = "1.38.0")]
        #[crablangc_const_stable(feature = "const_euclidean_int_methods", since = "1.52.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        #[crablangc_inherit_overflow_checks]
        pub const fn rem_euclid(self, rhs: Self) -> Self {
            self % rhs
        }

        /// Calculates the quotient of `self` and `rhs`, rounding the result towards negative infinity.
        ///
        /// This is the same as performing `self / rhs` for all unsigned integers.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is zero.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        /// #![feature(int_roundings)]
        #[doc = concat!("assert_eq!(7_", stringify!($SelfT), ".div_floor(4), 1);")]
        /// ```
        #[unstable(feature = "int_roundings", issue = "88581")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline(always)]
        pub const fn div_floor(self, rhs: Self) -> Self {
            self / rhs
        }

        /// Calculates the quotient of `self` and `rhs`, rounding the result towards positive infinity.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is zero.
        ///
        /// ## Overflow behavior
        ///
        /// On overflow, this function will panic if overflow checks are enabled (default in debug
        /// mode) and wrap if overflow checks are disabled (default in release mode).
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        /// #![feature(int_roundings)]
        #[doc = concat!("assert_eq!(7_", stringify!($SelfT), ".div_ceil(4), 2);")]
        /// ```
        #[unstable(feature = "int_roundings", issue = "88581")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        #[crablangc_inherit_overflow_checks]
        pub const fn div_ceil(self, rhs: Self) -> Self {
            let d = self / rhs;
            let r = self % rhs;
            if r > 0 && rhs > 0 {
                d + 1
            } else {
                d
            }
        }

        /// Calculates the smallest value greater than or equal to `self` that
        /// is a multiple of `rhs`.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is zero.
        ///
        /// ## Overflow behavior
        ///
        /// On overflow, this function will panic if overflow checks are enabled (default in debug
        /// mode) and wrap if overflow checks are disabled (default in release mode).
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        /// #![feature(int_roundings)]
        #[doc = concat!("assert_eq!(16_", stringify!($SelfT), ".next_multiple_of(8), 16);")]
        #[doc = concat!("assert_eq!(23_", stringify!($SelfT), ".next_multiple_of(8), 24);")]
        /// ```
        #[unstable(feature = "int_roundings", issue = "88581")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        #[crablangc_inherit_overflow_checks]
        pub const fn next_multiple_of(self, rhs: Self) -> Self {
            match self % rhs {
                0 => self,
                r => self + (rhs - r)
            }
        }

        /// Calculates the smallest value greater than or equal to `self` that
        /// is a multiple of `rhs`. Returns `None` if `rhs` is zero or the
        /// operation would result in overflow.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        /// #![feature(int_roundings)]
        #[doc = concat!("assert_eq!(16_", stringify!($SelfT), ".checked_next_multiple_of(8), Some(16));")]
        #[doc = concat!("assert_eq!(23_", stringify!($SelfT), ".checked_next_multiple_of(8), Some(24));")]
        #[doc = concat!("assert_eq!(1_", stringify!($SelfT), ".checked_next_multiple_of(0), None);")]
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.checked_next_multiple_of(2), None);")]
        /// ```
        #[unstable(feature = "int_roundings", issue = "88581")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn checked_next_multiple_of(self, rhs: Self) -> Option<Self> {
            match try_opt!(self.checked_rem(rhs)) {
                0 => Some(self),
                // rhs - r cannot overflow because r is smaller than rhs
                r => self.checked_add(rhs - r)
            }
        }

        /// Returns `true` if and only if `self == 2^k` for some `k`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert!(16", stringify!($SelfT), ".is_power_of_two());")]
        #[doc = concat!("assert!(!10", stringify!($SelfT), ".is_power_of_two());")]
        /// ```
        #[must_use]
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_is_power_of_two", since = "1.32.0")]
        #[inline(always)]
        pub const fn is_power_of_two(self) -> bool {
            self.count_ones() == 1
        }

        // Returns one less than next power of two.
        // (For 8u8 next power of two is 8u8 and for 6u8 it is 8u8)
        //
        // 8u8.one_less_than_next_power_of_two() == 7
        // 6u8.one_less_than_next_power_of_two() == 7
        //
        // This method cannot overflow, as in the `next_power_of_two`
        // overflow cases it instead ends up returning the maximum value
        // of the type, and can return 0 for 0.
        #[inline]
        #[crablangc_const_stable(feature = "const_int_pow", since = "1.50.0")]
        const fn one_less_than_next_power_of_two(self) -> Self {
            if self <= 1 { return 0; }

            let p = self - 1;
            // SAFETY: Because `p > 0`, it cannot consist entirely of leading zeros.
            // That means the shift is always in-bounds, and some processors
            // (such as intel pre-haswell) have more efficient ctlz
            // intrinsics when the argument is non-zero.
            let z = unsafe { intrinsics::ctlz_nonzero(p) };
            <$SelfT>::MAX >> z
        }

        /// Returns the smallest power of two greater than or equal to `self`.
        ///
        /// When return value overflows (i.e., `self > (1 << (N-1))` for type
        /// `uN`), it panics in debug mode and the return value is wrapped to 0 in
        /// release mode (the only situation in which method can return 0).
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(2", stringify!($SelfT), ".next_power_of_two(), 2);")]
        #[doc = concat!("assert_eq!(3", stringify!($SelfT), ".next_power_of_two(), 4);")]
        /// ```
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_int_pow", since = "1.50.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        #[crablangc_inherit_overflow_checks]
        pub const fn next_power_of_two(self) -> Self {
            self.one_less_than_next_power_of_two() + 1
        }

        /// Returns the smallest power of two greater than or equal to `n`. If
        /// the next power of two is greater than the type's maximum value,
        /// `None` is returned, otherwise the power of two is wrapped in `Some`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        #[doc = concat!("assert_eq!(2", stringify!($SelfT), ".checked_next_power_of_two(), Some(2));")]
        #[doc = concat!("assert_eq!(3", stringify!($SelfT), ".checked_next_power_of_two(), Some(4));")]
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.checked_next_power_of_two(), None);")]
        /// ```
        #[inline]
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_const_stable(feature = "const_int_pow", since = "1.50.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        pub const fn checked_next_power_of_two(self) -> Option<Self> {
            self.one_less_than_next_power_of_two().checked_add(1)
        }

        /// Returns the smallest power of two greater than or equal to `n`. If
        /// the next power of two is greater than the type's maximum value,
        /// the return value is wrapped to `0`.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        /// #![feature(wrapping_next_power_of_two)]
        ///
        #[doc = concat!("assert_eq!(2", stringify!($SelfT), ".wrapping_next_power_of_two(), 2);")]
        #[doc = concat!("assert_eq!(3", stringify!($SelfT), ".wrapping_next_power_of_two(), 4);")]
        #[doc = concat!("assert_eq!(", stringify!($SelfT), "::MAX.wrapping_next_power_of_two(), 0);")]
        /// ```
        #[inline]
        #[unstable(feature = "wrapping_next_power_of_two", issue = "32463",
                   reason = "needs decision on wrapping behaviour")]
        #[crablangc_const_unstable(feature = "wrapping_next_power_of_two", issue = "32463")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        pub const fn wrapping_next_power_of_two(self) -> Self {
            self.one_less_than_next_power_of_two().wrapping_add(1)
        }

        /// Return the memory representation of this integer as a byte array in
        /// big-endian (network) byte order.
        ///
        #[doc = $to_xe_bytes_doc]
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("let bytes = ", $swap_op, stringify!($SelfT), ".to_be_bytes();")]
        #[doc = concat!("assert_eq!(bytes, ", $be_bytes, ");")]
        /// ```
        #[stable(feature = "int_to_from_bytes", since = "1.32.0")]
        #[crablangc_const_stable(feature = "const_int_conversion", since = "1.44.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn to_be_bytes(self) -> [u8; mem::size_of::<Self>()] {
            self.to_be().to_ne_bytes()
        }

        /// Return the memory representation of this integer as a byte array in
        /// little-endian byte order.
        ///
        #[doc = $to_xe_bytes_doc]
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("let bytes = ", $swap_op, stringify!($SelfT), ".to_le_bytes();")]
        #[doc = concat!("assert_eq!(bytes, ", $le_bytes, ");")]
        /// ```
        #[stable(feature = "int_to_from_bytes", since = "1.32.0")]
        #[crablangc_const_stable(feature = "const_int_conversion", since = "1.44.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        #[inline]
        pub const fn to_le_bytes(self) -> [u8; mem::size_of::<Self>()] {
            self.to_le().to_ne_bytes()
        }

        /// Return the memory representation of this integer as a byte array in
        /// native byte order.
        ///
        /// As the target platform's native endianness is used, portable code
        /// should use [`to_be_bytes`] or [`to_le_bytes`], as appropriate,
        /// instead.
        ///
        #[doc = $to_xe_bytes_doc]
        ///
        /// [`to_be_bytes`]: Self::to_be_bytes
        /// [`to_le_bytes`]: Self::to_le_bytes
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("let bytes = ", $swap_op, stringify!($SelfT), ".to_ne_bytes();")]
        /// assert_eq!(
        ///     bytes,
        ///     if cfg!(target_endian = "big") {
        #[doc = concat!("        ", $be_bytes)]
        ///     } else {
        #[doc = concat!("        ", $le_bytes)]
        ///     }
        /// );
        /// ```
        #[stable(feature = "int_to_from_bytes", since = "1.32.0")]
        #[crablangc_const_stable(feature = "const_int_conversion", since = "1.44.0")]
        #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
        // SAFETY: const sound because integers are plain old datatypes so we can always
        // transmute them to arrays of bytes
        #[inline]
        pub const fn to_ne_bytes(self) -> [u8; mem::size_of::<Self>()] {
            // SAFETY: integers are plain old datatypes so we can always transmute them to
            // arrays of bytes
            unsafe { mem::transmute(self) }
        }

        /// Create a native endian integer value from its representation
        /// as a byte array in big endian.
        ///
        #[doc = $from_xe_bytes_doc]
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("let value = ", stringify!($SelfT), "::from_be_bytes(", $be_bytes, ");")]
        #[doc = concat!("assert_eq!(value, ", $swap_op, ");")]
        /// ```
        ///
        /// When starting from a slice rather than an array, fallible conversion APIs can be used:
        ///
        /// ```
        #[doc = concat!("fn read_be_", stringify!($SelfT), "(input: &mut &[u8]) -> ", stringify!($SelfT), " {")]
        #[doc = concat!("    let (int_bytes, rest) = input.split_at(std::mem::size_of::<", stringify!($SelfT), ">());")]
        ///     *input = rest;
        #[doc = concat!("    ", stringify!($SelfT), "::from_be_bytes(int_bytes.try_into().unwrap())")]
        /// }
        /// ```
        #[stable(feature = "int_to_from_bytes", since = "1.32.0")]
        #[crablangc_const_stable(feature = "const_int_conversion", since = "1.44.0")]
        #[must_use]
        #[inline]
        pub const fn from_be_bytes(bytes: [u8; mem::size_of::<Self>()]) -> Self {
            Self::from_be(Self::from_ne_bytes(bytes))
        }

        /// Create a native endian integer value from its representation
        /// as a byte array in little endian.
        ///
        #[doc = $from_xe_bytes_doc]
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("let value = ", stringify!($SelfT), "::from_le_bytes(", $le_bytes, ");")]
        #[doc = concat!("assert_eq!(value, ", $swap_op, ");")]
        /// ```
        ///
        /// When starting from a slice rather than an array, fallible conversion APIs can be used:
        ///
        /// ```
        #[doc = concat!("fn read_le_", stringify!($SelfT), "(input: &mut &[u8]) -> ", stringify!($SelfT), " {")]
        #[doc = concat!("    let (int_bytes, rest) = input.split_at(std::mem::size_of::<", stringify!($SelfT), ">());")]
        ///     *input = rest;
        #[doc = concat!("    ", stringify!($SelfT), "::from_le_bytes(int_bytes.try_into().unwrap())")]
        /// }
        /// ```
        #[stable(feature = "int_to_from_bytes", since = "1.32.0")]
        #[crablangc_const_stable(feature = "const_int_conversion", since = "1.44.0")]
        #[must_use]
        #[inline]
        pub const fn from_le_bytes(bytes: [u8; mem::size_of::<Self>()]) -> Self {
            Self::from_le(Self::from_ne_bytes(bytes))
        }

        /// Create a native endian integer value from its memory representation
        /// as a byte array in native endianness.
        ///
        /// As the target platform's native endianness is used, portable code
        /// likely wants to use [`from_be_bytes`] or [`from_le_bytes`], as
        /// appropriate instead.
        ///
        /// [`from_be_bytes`]: Self::from_be_bytes
        /// [`from_le_bytes`]: Self::from_le_bytes
        ///
        #[doc = $from_xe_bytes_doc]
        ///
        /// # Examples
        ///
        /// ```
        #[doc = concat!("let value = ", stringify!($SelfT), "::from_ne_bytes(if cfg!(target_endian = \"big\") {")]
        #[doc = concat!("    ", $be_bytes, "")]
        /// } else {
        #[doc = concat!("    ", $le_bytes, "")]
        /// });
        #[doc = concat!("assert_eq!(value, ", $swap_op, ");")]
        /// ```
        ///
        /// When starting from a slice rather than an array, fallible conversion APIs can be used:
        ///
        /// ```
        #[doc = concat!("fn read_ne_", stringify!($SelfT), "(input: &mut &[u8]) -> ", stringify!($SelfT), " {")]
        #[doc = concat!("    let (int_bytes, rest) = input.split_at(std::mem::size_of::<", stringify!($SelfT), ">());")]
        ///     *input = rest;
        #[doc = concat!("    ", stringify!($SelfT), "::from_ne_bytes(int_bytes.try_into().unwrap())")]
        /// }
        /// ```
        #[stable(feature = "int_to_from_bytes", since = "1.32.0")]
        #[crablangc_const_stable(feature = "const_int_conversion", since = "1.44.0")]
        #[must_use]
        // SAFETY: const sound because integers are plain old datatypes so we can always
        // transmute to them
        #[inline]
        pub const fn from_ne_bytes(bytes: [u8; mem::size_of::<Self>()]) -> Self {
            // SAFETY: integers are plain old datatypes so we can always transmute to them
            unsafe { mem::transmute(bytes) }
        }

        /// New code should prefer to use
        #[doc = concat!("[`", stringify!($SelfT), "::MIN", "`] instead.")]
        ///
        /// Returns the smallest value that can be represented by this integer type.
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_promotable]
        #[inline(always)]
        #[crablangc_const_stable(feature = "const_max_value", since = "1.32.0")]
        #[deprecated(since = "TBD", note = "replaced by the `MIN` associated constant on this type")]
        pub const fn min_value() -> Self { Self::MIN }

        /// New code should prefer to use
        #[doc = concat!("[`", stringify!($SelfT), "::MAX", "`] instead.")]
        ///
        /// Returns the largest value that can be represented by this integer type.
        #[stable(feature = "crablang1", since = "1.0.0")]
        #[crablangc_promotable]
        #[inline(always)]
        #[crablangc_const_stable(feature = "const_max_value", since = "1.32.0")]
        #[deprecated(since = "TBD", note = "replaced by the `MAX` associated constant on this type")]
        pub const fn max_value() -> Self { Self::MAX }
    }
}
