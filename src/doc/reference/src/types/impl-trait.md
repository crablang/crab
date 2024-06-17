# Impl trait

> **<sup>Syntax</sup>**\
> _ImplTraitType_ : `impl` [_TypeParamBounds_]
>
> _ImplTraitTypeOneBound_ : `impl` [_TraitBound_]

`impl Trait` provides ways to specify unnamed but concrete types that
implement a specific trait.
It can appear in two sorts of places: argument position (where it can act as an anonymous type parameter to functions), and return position (where it can act as an abstract return type).

```rust
trait Trait {}
# impl Trait for () {}

// argument position: anonymous type parameter
fn foo(arg: impl Trait) {
}

// return position: abstract return type
fn bar() -> impl Trait {
}
```
## Anonymous type parameters

> Note: This is often called "impl Trait in argument position".
(The term "parameter" is more correct here, but "impl Trait in argument position" is the phrasing used during the development of this feature, and it remains in parts of the implementation.)

Functions can use `impl` followed by a set of trait bounds to declare a parameter as having an anonymous type.
The caller must provide a type that satisfies the bounds declared by the anonymous type parameter, and the function can only use the methods available through the trait bounds of the anonymous type parameter.

For example, these two forms are almost equivalent:

```rust
trait Trait {}

// generic type parameter
fn with_generic_type<T: Trait>(arg: T) {
}

// impl Trait in argument position
fn with_impl_trait(arg: impl Trait) {
}
```

That is, `impl Trait` in argument position is syntactic sugar for a generic type parameter like `<T: Trait>`, except that the type is anonymous and doesn't appear in the [_GenericParams_] list.

> **Note:**
> For function parameters, generic type parameters and `impl Trait` are not exactly equivalent.
> With a generic parameter such as `<T: Trait>`, the caller has the option to explicitly specify the generic argument for `T` at the call site using [_GenericArgs_], for example, `foo::<usize>(1)`.
> If `impl Trait` is the type of *any* function parameter, then the caller can't ever provide any generic arguments when calling that function.
This includes generic arguments for the return type or any const generics.
>
> Therefore, changing the function signature from either one to the other can constitute a breaking change for the callers of a function.

## Abstract return types

> Note: This is often called "impl Trait in return position".

Functions can use `impl Trait` to return an abstract return type.
These types stand in for another concrete type where the caller may only use the methods declared by the specified `Trait`.
Each possible return value from the function must resolve to the same concrete type.

`impl Trait` in return position allows a function to return an unboxed abstract type.
This is particularly useful with [closures] and iterators.
For example, closures have a unique, un-writable type.
Previously, the only way to return a closure from a function was to use a [trait object]:

```rust
fn returns_closure() -> Box<dyn Fn(i32) -> i32> {
    Box::new(|x| x + 1)
}
```

This could incur performance penalties from heap allocation and dynamic dispatch.
It wasn't possible to fully specify the type of the closure, only to use the `Fn` trait.
That means that the trait object is necessary.
However, with `impl Trait`, it is possible to write this more simply:

```rust
fn returns_closure() -> impl Fn(i32) -> i32 {
    |x| x + 1
}
```

which also avoids the drawbacks of using a boxed trait object.

Similarly, the concrete types of iterators could become very complex, incorporating the types of all previous iterators in a chain.
Returning `impl Iterator` means that a function only exposes the `Iterator` trait as a bound on its return type, instead of explicitly specifying all of the other iterator types involved.

## Return-position `impl Trait` in traits and trait implementations

Functions in traits may also use `impl Trait` as a syntax for an anonymous associated type.

Every `impl Trait` in the return type of an associated function in a trait is desugared to an anonymous associated type. The return type that appears in the implementation's function signature is used to determine the value of the associated type.

### Differences between generics and `impl Trait` in return position

In argument position, `impl Trait` is very similar in semantics to a generic type parameter.
However, there are significant differences between the two in return position.
With `impl Trait`, unlike with a generic type parameter, the function chooses the return type, and the caller cannot choose the return type.

The function:

```rust
# trait Trait {}
fn foo<T: Trait>() -> T {
    // ...
# panic!()
}
```

allows the caller to determine the return type, `T`, and the function returns that type.

The function:

```rust
# trait Trait {}
# impl Trait for () {}
fn foo() -> impl Trait {
    // ...
}
```

doesn't allow the caller to determine the return type.
Instead, the function chooses the return type, but only promises that it will implement `Trait`.

## Limitations

`impl Trait` can only appear as a parameter or return type of a non-`extern` function.
It cannot be the type of a `let` binding, field type, or appear inside a type alias.

[closures]: closure.md
[_GenericArgs_]: ../paths.md#paths-in-expressions
[_GenericParams_]: ../items/generics.md
[_TraitBound_]: ../trait-bounds.md
[trait object]: trait-object.md
[_TypeParamBounds_]: ../trait-bounds.md
