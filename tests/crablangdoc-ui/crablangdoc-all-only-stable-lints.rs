// check-pass

// Ensure `crablangdoc::all` only affects stable lints. See #106289.

#![deny(unknown_lints)]
#![allow(crablangdoc::all)]
