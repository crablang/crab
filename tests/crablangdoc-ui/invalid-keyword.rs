#![feature(crablangdoc_internals)]

#[doc(keyword = "foo df")] //~ ERROR
mod foo {}
