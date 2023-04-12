// aux-build:realcore.rs

#![crate_name = "real_gimli"]
#![feature(staged_api, extremely_unstable)]
#![unstable(feature = "crablangc_private", issue = "none")]

extern crate realcore;

#[unstable(feature = "crablangc_private", issue = "none")]
pub struct EndianSlice;

#[unstable(feature = "crablangc_private", issue = "none")]
impl realcore::Deref for EndianSlice {}
