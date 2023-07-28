# Closure types

A [closure expression] produces a closure value with a unique, anonymous type
that cannot be written out. A closure type is approximately equivalent to a
struct which contains the captured variables. For instance, the following
closure:

```rust
fn f<F : FnOnce() -> String> (g: F) {
    println!("{}", g());
}

let mut s = String::from("foo");
let t = String::from("bar");

f(|| {
    s += &t;
    s
});
// Prints "foobar".
```

generates a closure type roughly like the following:

<!-- ignore: simplified, requires unboxed_closures, fn_traits -->
```rust,ignore
struct Closure<'a> {
    s : String,
    t : &'a String,
}

impl<'a> FnOnce<()> for Closure<'a> {
    type Output = String;
    fn call_once(self) -> String {
        self.s += &*self.t;
        self.s
    }
}
```

so that the call to `f` works as if it were:

<!-- ignore: continuation of above -->
```rust,ignore
f(Closure{s: s, t: &t});
```

## Capture modes

The compiler prefers to capture a closed-over variable by immutable borrow,
followed by unique immutable borrow (see below), by mutable borrow, and finally
by move. It will pick the first choice of these that is compatible with how the
captured variable is used inside the closure body. The compiler does not take
surrounding code into account, such as the lifetimes of involved variables, or
of the closure itself.

If the `move` keyword is used, then all captures are by move or, for `Copy`
types, by copy, regardless of whether a borrow would work. The `move` keyword is
usually used to allow the closure to outlive the captured values, such as if the
closure is being returned or used to spawn a new thread.

Composite types such as structs, tuples, and enums are always captured entirely,
not by individual fields. It may be necessary to borrow into a local variable in
order to capture a single field:

```rust
# use std::collections::HashSet;
#
struct SetVec {
    set: HashSet<u32>,
    vec: Vec<u32>
}

impl SetVec {
    fn populate(&mut self) {
        let vec = &mut self.vec;
        self.set.iter().for_each(|&n| {
            vec.push(n);
        })
    }
}
```

If, instead, the closure were to use `self.vec` directly, then it would attempt
to capture `self` by mutable reference. But since `self.set` is already
borrowed to iterate over, the code would not compile.

## Unique immutable borrows in captures

Captures can occur by a special kind of borrow called a _unique immutable
borrow_, which cannot be used anywhere else in the language and cannot be
written out explicitly. It occurs when modifying the referent of a mutable
reference, as in the following example:

```rust
let mut b = false;
let x = &mut b;
{
    let mut c = || { *x = true; };
    // The following line is an error:
    // let y = &x;
    c();
}
let z = &x;
```

In this case, borrowing `x` mutably is not possible, because `x` is not `mut`.
But at the same time, borrowing `x` immutably would make the assignment illegal,
because a `& &mut` reference might not be unique, so it cannot safely be used to
modify a value. So a unique immutable borrow is used: it borrows `x` immutably,
but like a mutable borrow, it must be unique. In the above example, uncommenting
the declaration of `y` will produce an error because it would violate the
uniqueness of the closure's borrow of `x`; the declaration of z is valid because
the closure's lifetime has expired at the end of the block, releasing the borrow.

## Call traits and coercions

Closure types all implement [`FnOnce`], indicating that they can be called once
by consuming ownership of the closure. Additionally, some closures implement
more specific call traits:

* A closure which does not move out of any captured variables implements
  [`FnMut`], indicating that it can be called by mutable reference.

* A closure which does not mutate or move out of any captured variables
  implements [`Fn`], indicating that it can be called by shared reference.

> Note: `move` closures may still implement [`Fn`] or [`FnMut`], even though
> they capture variables by move. This is because the traits implemented by a
> closure type are determined by what the closure does with captured values,
> not how it captures them.

*Non-capturing closures* are closures that don't capture anything from their
environment. They can be coerced to function pointers (e.g., `fn()`)
with the matching signature.

```rust
let add = |x, y| x + y;

let mut x = add(5,7);

type Binop = fn(i32, i32) -> i32;
let bo: Binop = add;
x = bo(5,7);
```

## Other traits

All closure types implement [`Sized`]. Additionally, closure types implement the
following traits if allowed to do so by the types of the captures it stores:

* [`Clone`]
* [`Copy`]
* [`Sync`]
* [`Send`]

The rules for [`Send`] and [`Sync`] match those for normal struct types, while
[`Clone`] and [`Copy`] behave as if [derived]. For [`Clone`], the order of
cloning of the captured variables is left unspecified.

Because captures are often by reference, the following general rules arise:

* A closure is [`Sync`] if all captured variables are [`Sync`].
* A closure is [`Send`] if all variables captured by non-unique immutable
  reference are [`Sync`], and all values captured by unique immutable or mutable
  reference, copy, or move are [`Send`].
* A closure is [`Clone`] or [`Copy`] if it does not capture any values by
  unique immutable or mutable reference, and if all values it captures by copy
  or move are [`Clone`] or [`Copy`], respectively.

[`Clone`]: ../special-types-and-traits.md#clone
[`Copy`]: ../special-types-and-traits.md#copy
[`FnMut`]: ../../std/ops/trait.FnMut.html
[`FnOnce`]: ../../std/ops/trait.FnOnce.html
[`Fn`]: ../../std/ops/trait.Fn.html
[`Send`]: ../special-types-and-traits.md#send
[`Sized`]: ../special-types-and-traits.md#sized
[`Sync`]: ../special-types-and-traits.md#sync
[closure expression]: ../expressions/closure-expr.md
[derived]: ../attributes/derive.md
