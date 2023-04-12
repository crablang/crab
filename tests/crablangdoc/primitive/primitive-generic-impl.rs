#![feature(crablangc_attrs)]
#![crate_name = "foo"]

// @has foo/primitive.i32.html '//*[@id="impl-ToString-for-i32"]//h3[@class="code-header"]' 'impl<T> ToString for T'

#[crablangc_doc_primitive = "i32"]
/// Some useless docs, wouhou!
mod i32 {}
