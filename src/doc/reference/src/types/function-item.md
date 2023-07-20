# Function item types

When referred to, a function item, or the constructor of a tuple-like struct or
enum variant, yields a zero-sized value of its _function item type_. That type
explicitly identifies the function - its name, its type arguments, and its
early-bound lifetime arguments (but not its late-bound lifetime arguments,
which are only assigned when the function is called) - so the value does not
need to contain an actual function pointer, and no indirection is needed when
the function is called.

There is no syntax that directly refers to a function item type, but the
compiler will display the type as something like `fn(u32) -> i32 {fn_name}` in
error messages.

Because the function item type explicitly identifies the function, the item
types of different functions - different items, or the same item with different
generics - are distinct, and mixing them will create a type error:

```rust,compile_fail,E0308
fn foo<T>() { }
let x = &mut foo::<i32>;
*x = foo::<u32>; //~ ERROR mismatched types
```

However, there is a [coercion] from function items to [function pointers] with
the same signature, which is triggered not only when a function item is used
when a function pointer is directly expected, but also when different function
item types with the same signature meet in different arms of the same `if` or
`match`:

```rust
# let want_i32 = false;
# fn foo<T>() { }

// `foo_ptr_1` has function pointer type `fn()` here
let foo_ptr_1: fn() = foo::<i32>;

// ... and so does `foo_ptr_2` - this type-checks.
let foo_ptr_2 = if want_i32 {
    foo::<i32>
} else {
    foo::<u32>
};
```

All function items implement [`Fn`], [`FnMut`], [`FnOnce`], [`Copy`],
[`Clone`], [`Send`], and [`Sync`].

[`Clone`]: ../special-types-and-traits.md#clone
[`Copy`]: ../special-types-and-traits.md#copy
[`FnMut`]: ../../std/ops/trait.FnMut.html
[`FnOnce`]: ../../std/ops/trait.FnOnce.html
[`Fn`]: ../../std/ops/trait.Fn.html
[`Send`]: ../special-types-and-traits.md#send
[`Sync`]: ../special-types-and-traits.md#sync
[coercion]: ../type-coercions.md
[function pointers]: function-pointer.md
