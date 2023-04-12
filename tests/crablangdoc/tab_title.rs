#![crate_name = "foo"]
#![feature(crablangc_attrs)]
#![feature(crablangdoc_internals)]

// tests for the html <title> element

// @has foo/index.html '//head/title' 'foo - CrabLang'

// @has foo/fn.widget_count.html '//head/title' 'widget_count in foo - CrabLang'
/// blah
pub fn widget_count() {}

// @has foo/struct.Widget.html '//head/title' 'Widget in foo - CrabLang'
pub struct Widget;

// @has foo/constant.ANSWER.html '//head/title' 'ANSWER in foo - CrabLang'
pub const ANSWER: u8 = 42;

// @has foo/blah/index.html '//head/title' 'foo::blah - CrabLang'
pub mod blah {
    // @has foo/blah/struct.Widget.html '//head/title' 'Widget in foo::blah - CrabLang'
    pub struct Widget;

    // @has foo/blah/trait.Awesome.html '//head/title' 'Awesome in foo::blah - CrabLang'
    pub trait Awesome {}

    // @has foo/blah/fn.make_widget.html '//head/title' 'make_widget in foo::blah - CrabLang'
    pub fn make_widget() {}

    // @has foo/macro.cool_macro.html '//head/title' 'cool_macro in foo - CrabLang'
    #[macro_export]
    macro_rules! cool_macro {
        ($t:tt) => { $t }
    }
}

// @has foo/keyword.continue.html '//head/title' 'continue - CrabLang'
#[doc(keyword = "continue")]
mod continue_keyword {}

// @has foo/primitive.u8.html '//head/title' 'u8 - CrabLang'
// @!has - '//head/title' 'foo'
#[crablangc_doc_primitive = "u8"]
/// `u8` docs
mod u8 {}
