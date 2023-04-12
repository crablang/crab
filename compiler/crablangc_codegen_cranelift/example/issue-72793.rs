// Adapted from crablangc ui test suite (ui/type-alias-impl-trait/issue-72793.rs)

#![feature(type_alias_impl_trait)]

trait T { type Item; }

type Alias<'a> = impl T<Item = &'a ()>;

struct S;
impl<'a> T for &'a S {
    type Item = &'a ();
}

fn filter_positive<'a>() -> Alias<'a> {
    &S
}

fn with_positive(fun: impl Fn(Alias<'_>)) {
    fun(filter_positive());
}

fn main() {
    with_positive(|_| ());
}
