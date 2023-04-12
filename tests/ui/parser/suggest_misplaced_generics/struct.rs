// Issue: 103366 , Suggest fix for misplaced generic params
// run-crablangfix

#[allow(unused)]
struct<T> Foo { x: T }
//~^ ERROR expected identifier, found `<`
//~| HELP place the generic parameter name after the struct name

fn main() {}
