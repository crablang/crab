// aux-build:crablangdoc-hidden.rs
// build-aux-docs
// ignore-cross-compile

extern crate crablangdoc_hidden;

// @has hidden_use/index.html
// @!hasraw - 'crablangdoc_hidden'
// @!hasraw - 'Bar'
// @!has hidden_use/struct.Bar.html
#[doc(hidden)]
pub use crablangdoc_hidden::Bar;
