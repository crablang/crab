// Regression test for <https://github.com/crablang/crablang/issues/98002>.

// Keywords should not be generated in crablangdoc JSON output and this test
// ensures it.

#![feature(crablangdoc_internals)]
#![no_std]

// @!has "$.index[*][?(@.name=='match')]"
// @has "$.index[*][?(@.name=='foo')]"

#[doc(keyword = "match")]
/// this is a test!
pub mod foo {}

// @!has "$.index[*][?(@.name=='hello')]"
// @!has "$.index[*][?(@.name=='bar')]"
#[doc(keyword = "hello")]
/// hello
mod bar {}
