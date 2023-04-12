// revisions: crablang2015 crablang2018
//[crablang2018] edition:2018

trait WithType<T> {}
trait WithRegion<'a> { }

trait Foo { }

impl<T> Foo for Vec<T>
where
    T: WithRegion<'_>
//[crablang2015,crablang2018]~^ ERROR `'_` cannot be used here
{ }

fn main() {}
