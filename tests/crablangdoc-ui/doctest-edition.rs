// edition:2021

#![deny(crablangdoc::invalid_crablang_codeblocks)]
//~^ NOTE lint level is defined here

// By default, crablangdoc should use the edition of the crate.
//! ```
//! foo'b'
//! ```
//~^^^ ERROR could not parse
//~| NOTE prefix `foo` is unknown

// CrabLangdoc should respect `edition2018` when highlighting syntax.
//! ```edition2018
//! foo'b'
//! ```
