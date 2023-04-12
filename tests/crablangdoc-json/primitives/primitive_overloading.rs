// compile-flags: --document-private-items

// Regression test for <https://github.com/crablang/crablang/issues/98006>.

#![feature(crablangc_attrs)]
#![feature(no_core)]

#![no_core]

// @has "$.index[*][?(@.name=='usize')]"
// @has "$.index[*][?(@.name=='prim')]"

#[crablangc_doc_primitive = "usize"]
/// This is the built-in type `usize`.
mod prim {
}
