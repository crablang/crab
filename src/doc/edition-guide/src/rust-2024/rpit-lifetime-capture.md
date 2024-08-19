# RPIT lifetime capture rules

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".

This chapter describes changes related to the **Lifetime Capture Rules 2024** introduced in [RFC 3498], including how to use opaque type *precise capturing* (introduced in [RFC 3617]) to migrate your code.

[RFC 3498]: https://github.com/rust-lang/rfcs/pull/3498
[RFC 3617]: https://github.com/rust-lang/rfcs/pull/3617

## Summary

- In Rust 2024, *all* in-scope generic parameters, including lifetime parameters, are implicitly captured when the `use<..>` bound is not present.
- Uses of the `Captures` trick (`Captures<..>` bounds) and of the outlives trick (e.g. `'_` bounds) can be replaced by `use<..>` bounds (in all editions) or removed entirely (in Rust 2024).

## Details

### Capturing

*Capturing* a generic parameter in an RPIT (return-position impl Trait) opaque type allows for that parameter to be used in the corresponding hidden type.  In Rust 1.82, we added `use<..>` bounds that allow specifying explicitly which generic parameters to capture.  Those will be helpful for migrating your code to Rust 2024, and will be helpful in this chapter for explaining how the edition-specific implicit capturing rules work.  These `use<..>` bounds look like this:

```rust
# #![feature(precise_capturing)]
fn capture<'a, T>(x: &'a (), y: T) -> impl Sized + use<'a, T> {
    //                                ~~~~~~~~~~~~~~~~~~~~~~~
    //                             This is the RPIT opaque type.
    //
    //                                It captures `'a` and `T`.
    (x, y)
  //~~~~~~
  // The hidden type is: `(&'a (), T)`.
  //
  // This type can use `'a` and `T` because they were captured.
}
```

The generic parameters that are captured affect how the opaque type can be used.  E.g., this is an error because the lifetime is captured despite the fact that the hidden type does not use the lifetime:

```rust,compile_fail
# #![feature(precise_capturing)]
fn capture<'a>(_: &'a ()) -> impl Sized + use<'a> {}

fn test<'a>(x: &'a ()) -> impl Sized + 'static {
    capture(x)
    //~^ ERROR lifetime may not live long enough
}
```

Conversely, this is OK:

```rust
# #![feature(precise_capturing)]
fn capture<'a>(_: &'a ()) -> impl Sized + use<> {}

fn test<'a>(x: &'a ()) -> impl Sized + 'static {
    capture(x) //~ OK
}
```

### Edition-specific rules when no `use<..>` bound is present

If the `use<..>` bound is not present, then the compiler uses edition-specific rules to decide which in-scope generic parameters to capture implicitly.

In all editions, all in-scope type and const generic parameters are captured implicitly when the `use<..>` bound is not present.  E.g.:

```rust
# #![feature(precise_capturing)]
fn f_implicit<T, const C: usize>() -> impl Sized {}
//                                    ~~~~~~~~~~
//                         No `use<..>` bound is present here.
//
// In all editions, the above is equivalent to:
fn f_explicit<T, const C: usize>() -> impl Sized + use<T, C> {}
```

In Rust 2021 and earlier editions, when the `use<..>` bound is not present, generic lifetime parameters are only captured when they appear syntactically within a bound in RPIT opaque types in the signature of bare functions and associated functions and methods within inherent impls.  However, starting in Rust 2024, these in-scope generic lifetime parameters are unconditionally captured.  E.g.:

```rust
# #![feature(precise_capturing)]
fn f_implicit(_: &()) -> impl Sized {}
// In Rust 2021 and earlier, the above is equivalent to:
fn f_2021(_: &()) -> impl Sized + use<> {}
// In Rust 2024 and later, it's equivalent to:
fn f_2024(_: &()) -> impl Sized + use<'_> {}
```

This makes the behavior consistent with RPIT opaque types in the signature of associated functions and methods within trait impls, uses of RPIT within trait definitions (RPITIT), and opaque `Future` types created by `async fn`, all of which implicitly capture all in-scope generic lifetime parameters in all editions when the `use<..>` bound is not present.

### Outer generic parameters

Generic parameters from an outer impl are considered to be in scope when deciding what is implicitly captured.  E.g.:

```rust
# #![feature(precise_capturing)]
struct S<T, const C: usize>((T, [(); C]));
impl<T, const C: usize> S<T, C> {
//   ~~~~~~~~~~~~~~~~~
// These generic parameters are in scope.
    fn f_implicit<U>() -> impl Sized {}
    //            ~       ~~~~~~~~~~
    //            ^ This generic is in scope too.
    //                    ^
    //                    |
    //     No `use<..>` bound is present here.
    //
    // In all editions, it's equivalent to:
    fn f_explicit<U>() -> impl Sized + use<T, U, C> {}
}
```

### Lifetimes from higher-ranked binders

Similarly, generic lifetime parameters introduced into scope by a higher-ranked `for<..>` binder are considered to be in scope.  E.g.:

```rust
# #![feature(precise_capturing)]
trait Tr<'a> { type Ty; }
impl Tr<'_> for () { type Ty = (); }

fn f_implicit() -> impl for<'a> Tr<'a, Ty = impl Copy> {}
// In Rust 2021 and earlier, the above is equivalent to:
fn f_2021() -> impl for<'a> Tr<'a, Ty = impl Copy + use<>> {}
// In Rust 2024 and later, it's equivalent to:
//fn f_2024() -> impl for<'a> Tr<'a, Ty = impl Copy + use<'a>> {}
//                                        ~~~~~~~~~~~~~~~~~~~~
// However, note that the capturing of higher-ranked lifetimes in
// nested opaque types is not yet supported.
```

### Argument position impl Trait (APIT)

Anonymous (i.e. unnamed) generic parameters created by the use of APIT (argument position impl Trait) are considered to be in scope.  E.g.:

```rust
# #![feature(precise_capturing)]
fn f_implicit(_: impl Sized) -> impl Sized {}
//               ~~~~~~~~~~
//           This is called APIT.
//
// The above is *roughly* equivalent to:
fn f_explicit<_0: Sized>(_: _0) -> impl Sized + use<_0> {}
```

Note that the former is not *exactly* equivalent to the latter because, by naming the generic parameter, turbofish syntax can now be used to provide an argument for it.  There is no way to explicitly include an anonymous generic parameter in a `use<..>` bound other than by converting it to a named generic parameter.

## Migration

### Migrating while avoiding overcapturing

The `impl_trait_overcaptures` lint flags RPIT opaque types that will capture additional lifetimes in Rust 2024.  This lint is part of the `rust-2024-compatibility` lint group which is automatically applied when running `cargo fix --edition`.  In most cases, the lint can automatically insert `use<..>` bounds where needed such that no additional lifetimes are captured in Rust 2024.

To migrate your code to be compatible with Rust 2024, run:

```sh
cargo fix --edition
```

For example, this will change:

```rust
fn f<'a>(x: &'a ()) -> impl Sized { *x }
```

...into:

```rust
# #![feature(precise_capturing)]
fn f<'a>(x: &'a ()) -> impl Sized + use<> { *x }
```

Without this `use<>` bound, in Rust 2024, the opaque type would capture the `'a` lifetime parameter.  By adding this bound, the migration lint preserves the existing semantics.

### Migrating cases involving APIT

In some cases, the lint cannot make the change automatically because a generic parameter needs to be given a name so that it can appear within a `use<..>` bound.  In these cases, the lint will alert you that a change may need to be made manually.  E.g., given:

```rust,edition2021
fn f<'a>(x: &'a (), y: impl Sized) -> impl Sized { (*x, y) }
//   ^^                ~~~~~~~~~~
//               This is a use of APIT.
//
//~^ WARN `impl Sized` will capture more lifetimes than possibly intended in edition 2024
//~| NOTE specifically, this lifetime is in scope but not mentioned in the type's bounds
#
# fn test<'a>(x: &'a (), y: ()) -> impl Sized + 'static {
#     f(x, y)
# }
```

The code cannot be converted automatically because of the use of APIT and the fact that the generic type parameter must be named in the `use<..>` bound.  To convert this code to Rust 2024 without capturing the lifetime, you must name that type parameter.  E.g.:

```rust
# #![feature(precise_capturing)]
# #![deny(impl_trait_overcaptures)]
fn f<'a, T: Sized>(x: &'a (), y: T) -> impl Sized + use<T> { (*x, y) }
//       ~~~~~~~~
// The type parameter has been named here.
#
# fn test<'a>(x: &'a (), y: ()) -> impl Sized + use<> {
#     f(x, y)
# }
```

Note that this changes the API of the function slightly as a type argument can now be explicitly provided for this parameter using turbofish syntax.  If this is undesired, you might consider instead whether you can simply continue to omit the `use<..>` bound and allow the lifetime to be captured.  This might be particularly desirable if you might in the future want to use that lifetime in the hidden type and would like to save space for that.

### Migrating away from the `Captures` trick

Prior to the introduction of precise capturing `use<..>` bounds in Rust 1.82, correctly capturing a lifetime in an RPIT opaque type often required using the `Captures` trick.  E.g.:

```rust
#[doc(hidden)]
pub trait Captures<T: ?Sized> {}
impl<T: ?Sized, U: ?Sized> Captures<T> for U {}

fn f<'a, T>(x: &'a (), y: T) -> impl Sized + Captures<(&'a (), T)> {
//                                           ~~~~~~~~~~~~~~~~~~~~~
//                            This is called the `Captures` trick.
    (x, y)
}
#
# fn test<'t, 'x>(t: &'t (), x: &'x ()) {
#     f(t, x);
# }
```

With the `use<..>` bound syntax, the `Captures` trick is no longer needed and can be replaced with the following in all editions:

```rust
# #![feature(precise_capturing)]
fn f<'a, T>(x: &'a (), y: T) -> impl Sized + use<'a, T> {
    (x, y)
}
#
# fn test<'t, 'x>(t: &'t (), x: &'x ()) {
#     f(t, x);
# }
```

In Rust 2024, the `use<..>` bound can often be omitted entirely, and the above can be written simply as:

<!-- TODO: edition2024 -->
```rust
# #![feature(lifetime_capture_rules_2024)]
fn f<'a, T>(x: &'a (), y: T) -> impl Sized {
    (x, y)
}
#
# fn test<'t, 'x>(t: &'t (), x: &'x ()) {
#     f(t, x);
# }
```

There is no automatic migration for this, and the `Captures` trick still works in Rust 2024, but you might want to consider migrating code manually away from using this old trick.

### Migrating away from the outlives trick

Prior to the introduction of precise capturing `use<..>` bounds in Rust 1.82, it was common to use the "outlives trick" when a lifetime needed to be used in the hidden type of some opaque.  E.g.:

```rust
fn f<'a, T: 'a>(x: &'a (), y: T) -> impl Sized + 'a {
    //    ~~~~                                 ~~~~
    //    ^                     This is the outlives trick.
    //    |
    // This bound is needed only for the trick.
    (x, y)
//  ~~~~~~
// The hidden type is `(&'a (), T)`.
}
```

This trick was less baroque than the `Captures` trick, but also less correct.  As we can see in the example above, even though any lifetime components within `T` are independent from the lifetime `'a`, we're required to add a `T: 'a` bound in order to make the trick work.  This created undue and surprising restrictions on callers.

Using precise capturing, you can write the above instead, in all editions, as:

```rust
# #![feature(precise_capturing)]
fn f<T>(x: &(), y: T) -> impl Sized + use<'_, T> {
    (x, y)
}
#
# fn test<'t, 'x>(t: &'t (), x: &'x ()) {
#    f(t, x);
# }
```

In Rust 2024, the `use<..>` bound can often be omitted entirely, and the above can be written simply as:

<!-- TODO: edition2024 -->
```rust
# #![feature(precise_capturing)]
# #![feature(lifetime_capture_rules_2024)]
fn f<T>(x: &(), y: T) -> impl Sized {
    (x, y)
}
#
# fn test<'t, 'x>(t: &'t (), x: &'x ()) {
#    f(t, x);
# }
```

There is no automatic migration for this, and the outlives trick still works in Rust 2024, but you might want to consider migrating code manually away from using this old trick.
