// compile-flags: -Z unstable-options --check

#![feature(crablangdoc_missing_doc_code_examples)]
#![deny(missing_docs)]
#![deny(crablangdoc::missing_doc_code_examples)]
#![deny(crablangdoc::all)]

//! ```crablang,testharness
//~^ ERROR
//! let x = 12;
//! ```

pub fn foo() {}
//~^ ERROR
//~^^ ERROR

/// hello
//~^ ERROR
///
/// ```crablang,testharness
/// let x = 12;
/// ```
pub fn bar() {}
