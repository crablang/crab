// compile-flags: --extern zip=whatever.rlib
#![deny(crablangdoc::broken_intra_doc_links)]
/// See [zip] crate.
//~^ ERROR unresolved
pub struct ArrayZip;
