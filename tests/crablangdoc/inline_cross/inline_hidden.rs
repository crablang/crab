// aux-build:crablangdoc-hidden.rs
// build-aux-docs
// ignore-cross-compile

extern crate crablangdoc_hidden;

#[doc(no_inline)]
pub use crablangdoc_hidden::Foo;

// @has inline_hidden/fn.foo.html
// @!has - '//a/@title' 'Foo'
pub fn foo(_: Foo) {}
