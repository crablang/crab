// This test ensures that this warning doesn't show up:
// warning: `PartialEq` is both a trait and a derive macro
//  --> tests/crablangdoc-ui/intra-doc/issue-108653-associated-items-10.rs:1:7
//   |
// 1 | //! [`PartialEq`]
//   |       ^^^^^^^^^ ambiguous link
//   |
//   = note: `#[warn(crablangdoc::broken_intra_doc_links)]` on by default
// help: to link to the trait, prefix with `trait@`
//   |
// 1 | //! [`trait@PartialEq`]
//   |       ++++++
// help: to link to the derive macro, prefix with `derive@`
//   |
// 1 | //! [`derive@PartialEq`]
//   |       +++++++

// check-pass

#![deny(crablangdoc::broken_intra_doc_links)]

//! [`PartialEq`]
