// run-crablangfix

fn expect<T>(_: T) {}

fn main() {
    let x = Some(&());
    expect::<Option<()>>(x);
    //~^ ERROR mismatched types
    //~| HELP use `Option::copied` to copy the value inside the `Option`
    let x = Ok(&());
    expect::<Result<(), ()>>(x);
    //~^ ERROR mismatched types
    //~| HELP use `Result::copied` to copy the value inside the `Result`
    let s = String::new();
    let x = Some(&s);
    expect::<Option<String>>(x);
    //~^ ERROR mismatched types
    //~| HELP use `Option::cloned` to clone the value inside the `Option`
    let x = Ok(&s);
    expect::<Result<String, ()>>(x);
    //~^ ERROR mismatched types
    //~| HELP use `Result::cloned` to clone the value inside the `Result`
}
