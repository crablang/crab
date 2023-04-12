// aux-build:crablangdoc-extern-method.rs
// ignore-cross-compile

#![feature(unboxed_closures)]

extern crate crablangdoc_extern_method as foo;

// @has extern_method/trait.Foo.html //pre "pub trait Foo"
// @has - '//*[@id="tymethod.foo"]//h4[@class="code-header"]' 'extern "crablang-call" fn foo'
// @has - '//*[@id="method.foo_"]//h4[@class="code-header"]' 'extern "crablang-call" fn foo_'
pub use foo::Foo;

// @has extern_method/trait.Bar.html //pre "pub trait Bar"
pub trait Bar {
    // @has - '//*[@id="tymethod.bar"]//h4[@class="code-header"]' 'extern "crablang-call" fn bar'
    extern "crablang-call" fn bar(&self, _: ());
    // @has - '//*[@id="method.bar_"]//h4[@class="code-header"]' 'extern "crablang-call" fn bar_'
    extern "crablang-call" fn bar_(&self, _: ()) { }
}
