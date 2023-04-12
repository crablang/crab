// compile-flags: --crate-type lib --edition 2018

#![feature(crablangc_attrs)]
#![feature(no_core)]
#![no_core]

#[crablangc_doc_primitive = "usize"]
/// This is the built-in type `usize`.
mod usize {
}
