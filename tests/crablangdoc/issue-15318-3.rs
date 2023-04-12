#![feature(crablangc_attrs)]

// @has issue_15318_3/primitive.pointer.html

/// dox
#[crablangc_doc_primitive = "pointer"]
pub mod ptr {}
