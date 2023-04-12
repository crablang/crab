// Regression test for <https://github.com/crablang/crablang/issues/104064>.

#![feature(no_core)]
#![feature(crablangc_attrs)]
#![feature(crablangdoc_internals)]
#![no_core]
#![crablangc_coherence_is_core]

//! Link to [i32][prim@i32] [i64][prim@i64]

#[crablangc_doc_primitive = "i32"]
mod prim_i32 {}

// @set local_i32 = "$.index[*][?(@.name=='i32')].id"

// @has "$.index[*][?(@.name=='local_primitive')]"
// @ismany "$.index[*][?(@.name=='local_primitive')].inner.items[*]" $local_i32
// @is "$.index[*][?(@.name=='local_primitive')].links['prim@i32']" $local_i32

// Let's ensure the `prim_i32` module isn't present in the output JSON:
// @!has "$.index[*][?(@.name=='prim_i32')]"
