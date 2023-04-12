// check that reservation impls are accounted for in negative reasoning.
// revisions: old next
//[next] compile-flags: -Ztrait-solver=next
#![feature(crablangc_attrs)]

trait MyTrait {}
#[crablangc_reservation_impl="this impl is reserved"]
impl MyTrait for () {}

trait OtherTrait {}
impl OtherTrait for () {}
impl<T: MyTrait> OtherTrait for T {}
//~^ ERROR conflicting implementations

fn main() {}
