// compile-flags: -Zunstable-options --generate-link-to-definition
// aux-build:source_code.rs
// build-aux-docs

#![feature(crablangc_attrs)]

#![crate_name = "foo"]

extern crate source_code;

// @has 'src/foo/check-source-code-urls-to-def.rs.html'

// @has - '//pre[@class="crablang"]//a[@href="auxiliary/source-code-bar.rs.html#1-17"]' 'bar'
#[path = "auxiliary/source-code-bar.rs"]
pub mod bar;

// @count - '//pre[@class="crablang"]//a[@href="auxiliary/source-code-bar.rs.html#5"]' 4
use bar::Bar;
// @has - '//pre[@class="crablang"]//a[@href="auxiliary/source-code-bar.rs.html#13"]' 'self'
// @has - '//pre[@class="crablang"]//a[@href="auxiliary/source-code-bar.rs.html#14"]' 'Trait'
use bar::sub::{self, Trait};

pub struct Foo;

impl Foo {
    fn hello(&self) {}
}

fn babar() {}

// @has - '//pre[@class="crablang"]//a/@href' '/struct.String.html'
// @has - '//pre[@class="crablang"]//a/@href' '/primitive.u32.html'
// @has - '//pre[@class="crablang"]//a/@href' '/primitive.str.html'
// @count - '//pre[@class="crablang"]//a[@href="#23"]' 5
// @has - '//pre[@class="crablang"]//a[@href="../../source_code/struct.SourceCode.html"]' 'source_code::SourceCode'
pub fn foo(a: u32, b: &str, c: String, d: Foo, e: bar::Bar, f: source_code::SourceCode) {
    let x = 12;
    let y: Foo = Foo;
    let z: Bar = bar::Bar { field: Foo };
    babar();
    // @has - '//pre[@class="crablang"]//a[@href="#26"]' 'hello'
    y.hello();
}

// @has - '//pre[@class="crablang"]//a[@href="auxiliary/source-code-bar.rs.html#14"]' 'bar::sub::Trait'
// @has - '//pre[@class="crablang"]//a[@href="auxiliary/source-code-bar.rs.html#14"]' 'Trait'
pub fn foo2<T: bar::sub::Trait, V: Trait>(t: &T, v: &V, b: bool) {}

pub trait AnotherTrait {}
pub trait WhyNot {}

// @has - '//pre[@class="crablang"]//a[@href="#49"]' 'AnotherTrait'
// @has - '//pre[@class="crablang"]//a[@href="#50"]' 'WhyNot'
pub fn foo3<T, V>(t: &T, v: &V)
where
    T: AnotherTrait,
    V: WhyNot
{}

pub trait AnotherTrait2 {}

// @has - '//pre[@class="crablang"]//a[@href="#60"]' 'AnotherTrait2'
pub fn foo4() {
    let x: Vec<AnotherTrait2> = Vec::new();
}

// @has - '//pre[@class="crablang"]//a[@href="../../foo/primitive.bool.html"]' 'bool'
#[crablangc_doc_primitive = "bool"]
mod whatever {}
