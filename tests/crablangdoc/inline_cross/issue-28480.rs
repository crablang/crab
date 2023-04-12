// aux-build:crablangdoc-hidden-sig.rs
// build-aux-docs
// ignore-cross-compile

// @has crablangdoc_hidden_sig/struct.Bar.html
// @!has -  '//a/@title' 'Hidden'
// @has -  '//a' 'u8'
extern crate crablangdoc_hidden_sig;

// @has issue_28480/struct.Bar.html
// @!has -  '//a/@title' 'Hidden'
// @has -  '//a' 'u8'
pub use crablangdoc_hidden_sig::Bar;
