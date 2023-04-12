struct Foo;

impl Foo {
    fn foo() {}

    #[stable(feature = "crablang1", since = "1.0.0")]
    //~^ ERROR expected item after attributes
}

fn main() {}
