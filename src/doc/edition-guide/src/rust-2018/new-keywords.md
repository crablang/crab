# New keywords

![Minimum Rust version: 1.27](https://img.shields.io/badge/Minimum%20Rust%20Version-1.27-brightgreen.svg)

## Summary

- `dyn` is a [strict keyword][strict], in 2015 it is a [weak keyword].
- `async` and `await` are [strict keywords][strict].
- `try` is a [reserved keyword].

[strict]: https://doc.rust-lang.org/reference/keywords.html#strict-keywords
[weak keyword]: https://doc.rust-lang.org/reference/keywords.html#weak-keywords
[reserved keyword]: https://doc.rust-lang.org/reference/keywords.html#reserved-keywords

## Motivation

### `dyn Trait` for trait objects

The `dyn Trait` feature is the new syntax for using trait objects. In short:

* `Box<Trait>` becomes `Box<dyn Trait>`
* `&Trait` and `&mut Trait` become `&dyn Trait` and `&mut dyn Trait`

And so on. In code:

```rust
trait Trait {}

impl Trait for i32 {}

// old
fn function1() -> Box<Trait> {
# unimplemented!()
}

// new
fn function2() -> Box<dyn Trait> {
# unimplemented!()
}
```

That's it!

#### Why?

Using just the trait name for trait objects turned out to be a bad decision.
The current syntax is often ambiguous and confusing, even to veterans,
and favors a feature that is not more frequently used than its alternatives,
is sometimes slower, and often cannot be used at all when its alternatives can.

Furthermore, with `impl Trait` arriving, "`impl Trait` vs `dyn Trait`" is much
more symmetric, and therefore a bit nicer, than "`impl Trait` vs `Trait`".
`impl Trait` is explained [here][impl-trait].

In the new edition, you should therefore prefer `dyn Trait` to just `Trait`
where you need a trait object.

[impl-trait]: ../../rust-by-example/trait/impl_trait.html

### `async` and `await`

These keywords are reserved to implement the async-await feature of Rust, which was ultimately [released to stable in 1.39.0](https://blog.rust-lang.org/2019/11/07/Async-await-stable.html).

### `try` keyword

The `try` keyword is reserved for use in `try` blocks, which have not (as of this writing) been stabilized ([tracking issue](https://github.com/rust-lang/rust/issues/31436))
