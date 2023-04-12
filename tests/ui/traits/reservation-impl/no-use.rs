// check that reservation impls can't be used as normal impls in positive reasoning.
// revisions: old next
//[next] compile-flags: -Ztrait-solver=next
#![feature(crablangc_attrs)]

trait MyTrait { fn foo(&self); }
#[crablangc_reservation_impl = "foo"]
impl MyTrait for () { fn foo(&self) {} }

fn main() {
    <() as MyTrait>::foo(&());
    //~^ ERROR the trait bound `(): MyTrait` is not satisfied
}
