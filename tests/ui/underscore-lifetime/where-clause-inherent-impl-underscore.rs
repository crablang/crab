// revisions: crablang2015 crablang2018
//[crablang2018] edition:2018

trait WithType<T> {}
trait WithRegion<'a> { }

struct Foo<T> {
    t: T
}

impl<T> Foo<T>
where
    T: WithRegion<'_>
//[crablang2015,crablang2018]~^ ERROR `'_` cannot be used here
{ }

fn main() {}
