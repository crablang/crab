// aux-build:crablangdoc-default-impl.rs
// ignore-cross-compile

extern crate crablangdoc_default_impl as foo;

pub use foo::bar;

pub fn wut<T: bar::Bar>() {
}
