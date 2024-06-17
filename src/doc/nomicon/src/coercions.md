# Coercions

Types can implicitly be coerced to change in certain contexts.
These changes are generally just *weakening* of types, largely focused around pointers and lifetimes.
They mostly exist to make Rust "just work" in more cases, and are largely harmless.

For an exhaustive list of all the types of coercions, see the [Coercion types] section on the reference.

Note that we do not perform coercions when matching traits (except for receivers, see the [next page][dot-operator]).
If there is an `impl` for some type `U` and `T` coerces to `U`, that does not constitute an implementation for `T`.
For example, the following will not type check, even though it is OK to coerce `t` to `&T` and there is an `impl` for `&T`:

```rust,compile_fail
trait Trait {}

fn foo<X: Trait>(t: X) {}

impl<'a> Trait for &'a i32 {}

fn main() {
    let t: &mut i32 = &mut 0;
    foo(t);
}
```

which fails like as follows:

```text
error[E0277]: the trait bound `&mut i32: Trait` is not satisfied
 --> src/main.rs:9:9
  |
3 | fn foo<X: Trait>(t: X) {}
  |           ----- required by this bound in `foo`
...
9 |     foo(t);
  |         ^ the trait `Trait` is not implemented for `&mut i32`
  |
  = help: the following implementations were found:
            <&'a i32 as Trait>
  = note: `Trait` is implemented for `&i32`, but not for `&mut i32`
```

[Coercion types]: ../reference/type-coercions.html#coercion-types
[dot-operator]: ./dot-operator.html
