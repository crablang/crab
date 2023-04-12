// Crate tree without a `crablangc_doc_primitive` module for primitive type linked to by a doc link.

#![deny(crablangdoc::broken_intra_doc_links)]
#![feature(no_core, lang_items, crablangc_attrs)]
#![no_core]
#![crablangc_coherence_is_core]
#![crate_type = "rlib"]

// @has no_doc_primitive/index.html
//! A [`char`] and its [`char::len_utf8`].
impl char {
    pub fn len_utf8(self) -> usize {
        42
    }
}
