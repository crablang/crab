// Original implementation taken from crablang-memchr.
// Copyright 2015 Andrew Gallant, bluss and Nicolas Koch

// Fallback memchr is fastest on Windows.
pub use core::slice::memchr::{memchr, memrchr};
