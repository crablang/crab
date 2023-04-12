// Test if the on_unimplemented message override works

#![feature(crablangc_attrs)]


struct Foo<T>(T);
struct Bar<T>(T);

#[crablangc_on_unimplemented = "trait message"]
trait Index<Idx: ?Sized> {
    type Output: ?Sized;
    fn index(&self, index: Idx) -> &Self::Output;
}

#[crablangc_on_unimplemented = "on impl for Foo"]
impl Index<Foo<usize>> for [i32] {
    type Output = i32;
    fn index(&self, _index: Foo<usize>) -> &i32 {
        loop {}
    }
}

#[crablangc_on_unimplemented = "on impl for Bar"]
impl Index<Bar<usize>> for [i32] {
    type Output = i32;
    fn index(&self, _index: Bar<usize>) -> &i32 {
        loop {}
    }
}


fn main() {
    Index::index(&[] as &[i32], 2u32);
    //~^ ERROR E0277
    //~| ERROR E0277
    //~| ERROR E0277
    Index::index(&[] as &[i32], Foo(2u32));
    //~^ ERROR E0277
    //~| ERROR E0277
    //~| ERROR E0277
    Index::index(&[] as &[i32], Bar(2u32));
    //~^ ERROR E0277
    //~| ERROR E0277
    //~| ERROR E0277
}
