# Never type fallback change

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".

## Summary

- Never type (`!`) to any type ("never-to-any") coercions fall back to never type (`!`) rather than to unit type (`()`).
- The [`never_type_fallback_flowing_into_unsafe`] lint is now `deny` by default.

[`never_type_fallback_flowing_into_unsafe`]: ../../rustc/lints/listing/warn-by-default.html#never-type-fallback-flowing-into-unsafe

## Details

When the compiler sees a value of type `!` (never) in a [coercion site][], it implicitly inserts a coercion to allow the type checker to infer any type:

```rust,should_panic
# #![feature(never_type)]
// This:
let x: u8 = panic!();

// ...is (essentially) turned by the compiler into:
let x: u8 = absurd(panic!());

// ...where `absurd` is the following function
// (it's sound because `!` always marks unreachable code):
fn absurd<T>(x: !) -> T { x }
```

This can lead to compilation errors if the type cannot be inferred:

```rust,compile_fail,E0282
# #![feature(never_type)]
# fn absurd<T>(x: !) -> T { x }
// This:
{ panic!() };

// ...gets turned into this:
{ absurd(panic!()) }; //~ ERROR can't infer the type of `absurd`
```

To prevent such errors, the compiler remembers where it inserted `absurd` calls, and if it can't infer the type, it uses the fallback type instead:

```rust,should_panic
# #![feature(never_type)]
# fn absurd<T>(x: !) -> T { x }
type Fallback = /* An arbitrarily selected type! */ !;
{ absurd::<Fallback>(panic!()) }
```

This is what is known as "never type fallback".

Historically, the fallback type has been `()` (unit).  This caused `!` to spontaneously coerce to `()` even when the compiler would not infer `()` without the fallback.  That was confusing and has prevented the stabilization of the `!` type.

In the 2024 edition, the fallback type is now `!`.  (We plan to make this change across all editions at a later date.)  This makes things work more intuitively.  Now when you pass `!` and there is no reason to coerce it to something else, it is kept as `!`.

In some cases your code might depend on the fallback type being `()`, so this can cause compilation errors or changes in behavior.

[coercion site]: ../../reference/type-coercions.html#coercion-sites

### `never_type_fallback_flowing_into_unsafe`

The default level of the [`never_type_fallback_flowing_into_unsafe`] lint has been raised from `warn` to `deny` in the 2024 Edition. This lint helps detect a particular interaction with the fallback to `!` and `unsafe` code which may lead to undefined behavior. See the link for a complete description.

## Migration

There is no automatic fix, but there is automatic detection of code that will be broken by the edition change.  While still on a previous edition you will see warnings if your code will be broken.

The fix is to specify the type explicitly so that the fallback type is not used.  Unfortunately, it might not be trivial to see which type needs to be specified.

One of the most common patterns broken by this change is using `f()?;` where `f` is generic over the `Ok`-part of the return type:

```rust
# #![allow(dependency_on_unit_never_type_fallback)]
# fn outer<T>(x: T) -> Result<T, ()> {
fn f<T: Default>() -> Result<T, ()> {
    Ok(T::default())
}

f()?;
# Ok(x)
# }
```

You might think that, in this example, type `T` can't be inferred.  However, due to the current desugaring of the `?` operator, it was inferred as `()`, and it will now be inferred as `!`.

To fix the issue you need to specify the `T` type explicitly:

<!-- TODO: edition2024 -->
```rust
# #![deny(dependency_on_unit_never_type_fallback)]
# fn outer<T>(x: T) -> Result<T, ()> {
# fn f<T: Default>() -> Result<T, ()> {
#     Ok(T::default())
# }
f::<()>()?;
// ...or:
() = f()?;
# Ok(x)
# }
```

Another relatively common case is panicking in a closure:

```rust,should_panic
# #![allow(dependency_on_unit_never_type_fallback)]
trait Unit {}
impl Unit for () {}

fn run<R: Unit>(f: impl FnOnce() -> R) {
    f();
}

run(|| panic!());
```

Previously `!` from the `panic!` coerced to `()` which implements `Unit`.  However now the `!` is kept as `!` so this code fails because `!` doesn't implement `Unit`.  To fix this you can specify the return type of the closure:

<!-- TODO: edition2024 -->
```rust,should_panic
# #![deny(dependency_on_unit_never_type_fallback)]
# trait Unit {}
# impl Unit for () {}
#
# fn run<R: Unit>(f: impl FnOnce() -> R) {
#     f();
# }
run(|| -> () { panic!() });
```

A similar case to that of `f()?` can be seen when using a `!`-typed expression in one branch and a function with an unconstrained return type in the other:

```rust
# #![allow(dependency_on_unit_never_type_fallback)]
if true {
    Default::default()
} else {
    return
};
```

Previously `()` was inferred as the return type of `Default::default()` because `!` from `return` was spuriously coerced to `()`.  Now, `!` will be inferred instead causing this code to not compile because `!` does not implement `Default`.

Again, this can be fixed by specifying the type explicitly:

<!-- TODO: edition2024 -->
```rust
# #![deny(dependency_on_unit_never_type_fallback)]
() = if true {
    Default::default()
} else {
    return
};

// ...or:

if true {
    <() as Default>::default()
} else {
    return
};
```
