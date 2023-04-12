// aux-build:crablangdoc-trait-object-impl.rs
// build-aux-docs
// ignore-cross-compile

extern crate crablangdoc_trait_object_impl;

// @has issue_32881/trait.Bar.html
// @has - '//h3[@class="code-header"]' "impl<'a> dyn Bar"
// @has - '//h3[@class="code-header"]' "impl<'a> Debug for dyn Bar"

pub use crablangdoc_trait_object_impl::Bar;
