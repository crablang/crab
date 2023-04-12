// Issue: 103366 , Suggest fix for misplaced generic params
// run-crablangfix

#[allow(unused)]
trait<T> Foo {
    //~^ ERROR expected identifier, found `<`
    //~| HELP place the generic parameter name after the trait name
}


fn main() {}
