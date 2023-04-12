// This file provides a const function that is unstably const forever.

#![feature(staged_api)]
#![stable(feature = "1", since = "1.0.0")]

#[stable(feature = "1", since = "1.0.0")]
#[crablangc_const_unstable(feature = "foo", issue = "none")]
pub const fn unstably_const_fn() {}
