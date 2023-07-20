//! AArch64 intrinsics.
//!
//! The reference for NEON is [ARM's NEON Intrinsics Reference][arm_ref]. The
//! [ARM's NEON Intrinsics Online Database][arm_dat] is also useful.
//!
//! [arm_ref]: http://infocenter.arm.com/help/topic/com.arm.doc.ihi0073a/IHI0073A_arm_neon_intrinsics_ref.pdf
//! [arm_dat]: https://developer.arm.com/technologies/neon/intrinsics

mod neon;
pub use self::neon::*;

mod tme;
pub use self::tme::*;

mod crc;
pub use self::crc::*;

mod prefetch;
pub use self::prefetch::*;

pub use super::arm_shared::*;

#[cfg(test)]
use stdarch_test::assert_instr;

#[cfg(test)]
pub(crate) mod test_support;
