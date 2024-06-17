# Subtyping and Variance

Subtyping is implicit and can occur at any stage in type checking or
inference. Subtyping is restricted to two cases:
variance with respect to lifetimes and between types with higher ranked
lifetimes. If we were to erase lifetimes from types, then the only subtyping
would be due to type equality.

Consider the following example: string literals always have `'static`
lifetime. Nevertheless, we can assign `s` to `t`:

```rust
fn bar<'a>() {
    let s: &'static str = "hi";
    let t: &'a str = s;
}
```

Since `'static` outlives the lifetime parameter `'a`, `&'static str` is a
subtype of `&'a str`.

[Higher-ranked]&#32;[function pointers] and [trait objects] have another
subtype relation. They are subtypes of types that are given by substitutions of
the higher-ranked lifetimes. Some examples:

```rust
// Here 'a is substituted for 'static
let subtype: &(for<'a> fn(&'a i32) -> &'a i32) = &((|x| x) as fn(&_) -> &_);
let supertype: &(fn(&'static i32) -> &'static i32) = subtype;

// This works similarly for trait objects
let subtype: &(dyn for<'a> Fn(&'a i32) -> &'a i32) = &|x| x;
let supertype: &(dyn Fn(&'static i32) -> &'static i32) = subtype;

// We can also substitute one higher-ranked lifetime for another
let subtype: &(for<'a, 'b> fn(&'a i32, &'b i32))= &((|x, y| {}) as fn(&_, &_));
let supertype: &for<'c> fn(&'c i32, &'c i32) = subtype;
```

## Variance

Variance is a property that generic types have with respect to their arguments.
A generic type's *variance* in a parameter is how the subtyping of the
parameter affects the subtyping of the type.

* `F<T>` is *covariant* over `T` if `T` being a subtype of `U` implies that
  `F<T>` is a subtype of `F<U>` (subtyping "passes through")
* `F<T>` is *contravariant* over `T` if `T` being a subtype of `U` implies that
  `F<U>` is a subtype of `F<T>`
* `F<T>` is *invariant* over `T` otherwise (no subtyping relation can be
  derived)

Variance of types is automatically determined as follows

| Type                          | Variance in `'a`  | Variance in `T`   |
|-------------------------------|-------------------|-------------------|
| `&'a T`                       | covariant         | covariant         |
| `&'a mut T`                   | covariant         | invariant         |
| `*const T`                    |                   | covariant         |
| `*mut T`                      |                   | invariant         |
| `[T]` and `[T; n]`            |                   | covariant         |
| `fn() -> T`                   |                   | covariant         |
| `fn(T) -> ()`                 |                   | contravariant     |
| `std::cell::UnsafeCell<T>`    |                   | invariant         |
| `std::marker::PhantomData<T>` |                   | covariant         |
| `dyn Trait<T> + 'a`           | covariant         | invariant         |

The variance of other `struct`, `enum`, and `union` types is decided by
looking at the variance of the types of their fields. If the parameter is used
in positions with different variances then the parameter is invariant. For
example the following struct is covariant in `'a` and `T` and invariant in `'b`, `'c`,
and `U`.

```rust
use std::cell::UnsafeCell;
struct Variance<'a, 'b, 'c, T, U: 'a> {
    x: &'a U,               // This makes `Variance` covariant in 'a, and would
                            // make it covariant in U, but U is used later
    y: *const T,            // Covariant in T
    z: UnsafeCell<&'b f64>, // Invariant in 'b
    w: *mut U,              // Invariant in U, makes the whole struct invariant

    f: fn(&'c ()) -> &'c () // Both co- and contravariant, makes 'c invariant
                            // in the struct.
}
```

When used outside of an `struct`, `enum`, or `union`, the variance for parameters is checked at each location separately.

```rust
# use std::cell::UnsafeCell;
fn generic_tuple<'short, 'long: 'short>(
    // 'long is used inside of a tuple in both a co- and invariant position.
    x: (&'long u32, UnsafeCell<&'long u32>),
) {
    // As the variance at these positions is computed separately,
    // we can freely shrink 'long in the covariant position.
    let _: (&'short u32, UnsafeCell<&'long u32>) = x;
}

fn takes_fn_ptr<'short, 'middle: 'short>(
    // 'middle is used in both a co- and contravariant position.
    f: fn(&'middle ()) -> &'middle (),
) {
    // As the variance at these positions is computed separately,
    // we can freely shrink 'middle in the covariant position
    // and extend it in the contravariant position.
    let _: fn(&'static ()) -> &'short () = f;
}
```

[function pointers]: types/function-pointer.md
[Higher-ranked]: ../nomicon/hrtb.html
[trait objects]: types/trait-object.md
