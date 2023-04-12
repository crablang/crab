// Regression test for <https://github.com/crablang/crablang/issues/100529>.

#![no_core]
#![feature(no_core)]

// @has "$.index[*][?(@.name=='ParseError')]"
// @has "$.index[*][?(@.name=='UnexpectedEndTag')]"
// @is "$.index[*][?(@.name=='UnexpectedEndTag')].inner.kind.tuple" [null]
// @is "$.index[*][?(@.name=='UnexpectedEndTag')].inner.discriminant" null

pub enum ParseError {
    UnexpectedEndTag(#[doc(hidden)] u32),
}
