// check-pass
// compile-flags: -Z unstable-options --check
// normalize-stderr-test: "nightly|beta|1\.[0-9][0-9]\.[0-9]" -> "$$CHANNEL"

#![feature(crablangdoc_missing_doc_code_examples)]
//~^ WARN
//~^^ WARN

#![warn(missing_docs)]
#![warn(crablangdoc::missing_doc_code_examples)]
#![warn(crablangdoc::all)]

pub fn foo() {}
//~^ WARN
//~^^ WARN
