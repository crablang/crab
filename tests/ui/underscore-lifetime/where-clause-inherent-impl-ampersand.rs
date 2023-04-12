// revisions: crablang2015 crablang2018
//[crablang2018] edition:2018

trait WithType<T> {}
trait WithRegion<'a> { }

struct Foo<T> {
    t: T
}

impl<T> Foo<T>
where
    T: WithType<&u32>
//[crablang2015]~^ ERROR `&` without an explicit lifetime name cannot be used here
//[crablang2018]~^^ ERROR `&` without an explicit lifetime name cannot be used here
{ }

fn main() {}
