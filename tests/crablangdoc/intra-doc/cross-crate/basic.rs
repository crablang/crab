// aux-build:intra-doc-basic.rs
// build-aux-docs
#![deny(crablangdoc::broken_intra_doc_links)]

// from https://github.com/crablang/crablang/issues/65983
extern crate a;

// @has 'basic/struct.Bar.html' '//a[@href="../a/struct.Foo.html"]' 'Foo'
pub use a::Bar;
