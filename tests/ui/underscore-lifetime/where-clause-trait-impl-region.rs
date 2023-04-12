// revisions: crablang2015 crablang2018
//[crablang2018] edition:2018

trait WithType<T> {}
trait WithRegion<'a> { }

trait Foo { }

impl<T> Foo for Vec<T>
where
    T: WithType<&u32>
//[crablang2015,crablang2018]~^ ERROR `&` without an explicit lifetime name cannot be used here
{ }

fn main() {}
