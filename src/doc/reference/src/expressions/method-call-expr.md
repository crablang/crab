# Method-call expressions

> **<sup>Syntax</sup>**\
> _MethodCallExpression_ :\
> &nbsp;&nbsp; [_Expression_] `.` [_PathExprSegment_] `(`[_CallParams_]<sup>?</sup> `)`

A _method call_ consists of an expression (the *receiver*) followed by a single dot, an expression path segment, and a parenthesized expression-list.
Method calls are resolved to associated [methods] on specific traits, either statically dispatching to a method if the exact `self`-type of the left-hand-side is known, or dynamically dispatching if the left-hand-side expression is an indirect [trait object](../types/trait-object.md).

```rust
let pi: Result<f32, _> = "3.14".parse();
let log_pi = pi.unwrap_or(1.0).log(2.72);
# assert!(1.14 < log_pi && log_pi < 1.15)
```

When looking up a method call, the receiver may be automatically dereferenced or borrowed in order to call a method.
This requires a more complex lookup process than for other functions, since there may be a number of possible methods to call.
The following procedure is used:

The first step is to build a list of candidate receiver types.
Obtain these by repeatedly [dereferencing][dereference] the receiver expression's type, adding each type encountered to the list, then finally attempting an [unsized coercion] at the end, and adding the result type if that is successful.
Then, for each candidate `T`, add `&T` and `&mut T` to the list immediately after `T`.

For instance, if the receiver has type `Box<[i32;2]>`, then the candidate types will be `Box<[i32;2]>`, `&Box<[i32;2]>`, `&mut Box<[i32;2]>`, `[i32; 2]` (by dereferencing), `&[i32; 2]`, `&mut [i32; 2]`, `[i32]` (by unsized coercion), `&[i32]`, and finally `&mut [i32]`.

Then, for each candidate type `T`, search for a [visible] method with a receiver of that type in the following places:

1. `T`'s inherent methods (methods implemented directly on `T`).
1. Any of the methods provided by a [visible] trait implemented by `T`.
   If `T` is a type parameter, methods provided by trait bounds on `T` are looked up first.
   Then all remaining methods in scope are looked up.

> Note: the lookup is done for each type in order, which can occasionally lead to surprising results.
> The below code will print "In trait impl!", because `&self` methods are looked up first, the trait method is found before the struct's `&mut self` method is found.
>
> ```rust
> struct Foo {}
>
> trait Bar {
>   fn bar(&self);
> }
>
> impl Foo {
>   fn bar(&mut self) {
>     println!("In struct impl!")
>   }
> }
>
> impl Bar for Foo {
>   fn bar(&self) {
>     println!("In trait impl!")
>   }
> }
>
> fn main() {
>   let mut f = Foo{};
>   f.bar();
> }
> ```

If this results in multiple possible candidates, then it is an error, and the receiver must be [converted][disambiguate call] to an appropriate receiver type to make the method call.

This process does not take into account the mutability or lifetime of the receiver, or whether a method is `unsafe`.
Once a method is looked up, if it can't be called for one (or more) of those reasons, the result is a compiler error.

If a step is reached where there is more than one possible method, such as where generic methods or traits are considered the same, then it is a compiler error.
These cases require a [disambiguating function call syntax] for method and function invocation.

> **Edition Differences**: Before the 2021 edition, during the search for visible methods, if the candidate receiver type is an [array type], methods provided by the standard library [`IntoIterator`] trait are ignored.
>
> The edition used for this purpose is determined by the token representing the method name.
>
> This special case may be removed in the future.


<div class="warning">

***Warning:*** For [trait objects], if there is an inherent method of the same name as a trait method, it will give a compiler error when trying to call the method in a method call expression.
Instead, you can call the method using [disambiguating function call syntax], in which case it calls the trait method, not the inherent method.
There is no way to call the inherent method.
Just don't define inherent methods on trait objects with the same name as a trait method and you'll be fine.

</div>

[_CallParams_]: call-expr.md
[_Expression_]: ../expressions.md
[_PathExprSegment_]: ../paths.md#paths-in-expressions
[visible]: ../visibility-and-privacy.md
[array type]: ../types/array.md
[trait objects]: ../types/trait-object.md
[disambiguate call]: call-expr.md#disambiguating-function-calls
[disambiguating function call syntax]: call-expr.md#disambiguating-function-calls
[dereference]: operator-expr.md#the-dereference-operator
[methods]: ../items/associated-items.md#methods
[unsized coercion]: ../type-coercions.md#unsized-coercions
[`IntoIterator`]: ../../std/iter/trait.IntoIterator.html
