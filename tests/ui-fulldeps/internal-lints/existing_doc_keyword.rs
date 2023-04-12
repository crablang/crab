// compile-flags: -Z unstable-options

#![feature(crablangc_private)]
#![feature(crablangdoc_internals)]

#![crate_type = "lib"]

#![deny(crablangc::existing_doc_keyword)]

#[doc(keyword = "tadam")] //~ ERROR
mod tadam {}
