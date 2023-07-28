# Typestate Programming

The concept of [typestates] describes the encoding of information about the current state of an object into the type of that object. Although this can sound a little arcane, if you have used the [Builder Pattern] in Rust, you have already started using Typestate Programming!

[typestates]: https://en.wikipedia.org/wiki/Typestate_analysis
[Builder Pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html

```rust
pub mod foo_module {
    #[derive(Debug)]
    pub struct Foo {
        inner: u32,
    }

    pub struct FooBuilder {
        a: u32,
        b: u32,
    }

    impl FooBuilder {
        pub fn new(starter: u32) -> Self {
            Self {
                a: starter,
                b: starter,
            }
        }

        pub fn double_a(self) -> Self {
            Self {
                a: self.a * 2,
                b: self.b,
            }
        }

        pub fn into_foo(self) -> Foo {
            Foo {
                inner: self.a + self.b,
            }
        }
    }
}

fn main() {
    let x = foo_module::FooBuilder::new(10)
        .double_a()
        .into_foo();

    println!("{:#?}", x);
}
```

In this example, there is no direct way to create a `Foo` object. We must create a `FooBuilder`, and properly initialize it before we can obtain the `Foo` object we want.

This minimal example encodes two states:

* `FooBuilder`, which represents an "unconfigured", or "configuration in process" state
* `Foo`, which represents a "configured", or "ready to use" state.

## Strong Types

Because Rust has a [Strong Type System], there is no easy way to magically create an instance of `Foo`, or to turn a `FooBuilder` into a `Foo` without calling the `into_foo()` method. Additionally, calling the `into_foo()` method consumes the original `FooBuilder` structure, meaning it can not be reused without the creation of a new instance.

[Strong Type System]: https://en.wikipedia.org/wiki/Strong_and_weak_typing

This allows us to represent the states of our system as types, and to include the necessary actions for state transitions into the methods that exchange one type for another. By creating a `FooBuilder`, and exchanging it for a `Foo` object, we have walked through the steps of a basic state machine.
