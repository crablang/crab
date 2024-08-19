# Derive

The *`derive` attribute* allows new [items] to be automatically generated for
data structures. It uses the [_MetaListPaths_] syntax to specify a list of
traits to implement or paths to [derive macros] to process.

For example, the following will create an [`impl` item] for the
[`PartialEq`] and [`Clone`] traits for `Foo`, and the type parameter `T` will be
given the `PartialEq` or `Clone` constraints for the appropriate `impl`:

```rust
#[derive(PartialEq, Clone)]
struct Foo<T> {
    a: i32,
    b: T,
}
```

The generated `impl` for `PartialEq` is equivalent to

```rust
# struct Foo<T> { a: i32, b: T }
impl<T: PartialEq> PartialEq for Foo<T> {
    fn eq(&self, other: &Foo<T>) -> bool {
        self.a == other.a && self.b == other.b
    }
}
```

You can implement `derive` for your own traits through [procedural macros].

## The `automatically_derived` attribute

The *`automatically_derived` attribute* is automatically added to
[implementations] created by the `derive` attribute for built-in traits. It
has no direct effect, but it may be used by tools and diagnostic lints to
detect these automatically generated implementations.

[_MetaListPaths_]: ../attributes.md#meta-item-attribute-syntax
[`Clone`]: ../../std/clone/trait.Clone.html
[`PartialEq`]: ../../std/cmp/trait.PartialEq.html
[`impl` item]: ../items/implementations.md
[items]: ../items.md
[derive macros]: ../procedural-macros.md#derive-macros
[implementations]: ../items/implementations.md
[items]: ../items.md
[procedural macros]: ../procedural-macros.md#derive-macros
