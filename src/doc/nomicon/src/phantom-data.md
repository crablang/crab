# PhantomData

When working with unsafe code, we can often end up in a situation where
types or lifetimes are logically associated with a struct, but not actually
part of a field. This most commonly occurs with lifetimes. For instance, the
`Iter` for `&'a [T]` is (approximately) defined as follows:

```rust,compile_fail
struct Iter<'a, T: 'a> {
    ptr: *const T,
    end: *const T,
}
```

However because `'a` is unused within the struct's body, it's *unbounded*.
[Because of the troubles this has historically caused][unused-param],
unbounded lifetimes and types are *forbidden* in struct definitions.
Therefore we must somehow refer to these types in the body.
Correctly doing this is necessary to have correct variance and drop checking.

[unused-param]: https://rust-lang.github.io/rfcs/0738-variance.html#the-corner-case-unused-parameters-and-parameters-that-are-only-used-unsafely

We do this using `PhantomData`, which is a special marker type. `PhantomData`
consumes no space, but simulates a field of the given type for the purpose of
static analysis. This was deemed to be less error-prone than explicitly telling
the type-system the kind of variance that you want, while also providing other
useful things such as auto traits and the information needed by drop check.

Iter logically contains a bunch of `&'a T`s, so this is exactly what we tell
the `PhantomData` to simulate:

```rust
use std::marker;

struct Iter<'a, T: 'a> {
    ptr: *const T,
    end: *const T,
    _marker: marker::PhantomData<&'a T>,
}
```

and that's it. The lifetime will be bounded, and your iterator will be covariant
over `'a` and `T`. Everything Just Works.

## Generic parameters and drop-checking

In the past, there used to be another thing to take into consideration.

This very documentation used to say:

> Another important example is Vec, which is (approximately) defined as follows:
>
> ```rust
> struct Vec<T> {
>     data: *const T, // *const for variance!
>     len: usize,
>     cap: usize,
> }
> ```
>
> Unlike the previous example, it *appears* that everything is exactly as we
> want. Every generic argument to Vec shows up in at least one field.
> Good to go!
>
> Nope.
>
> The drop checker will generously determine that `Vec<T>` does not own any values
> of type T. This will in turn make it conclude that it doesn't need to worry
> about Vec dropping any T's in its destructor for determining drop check
> soundness. This will in turn allow people to create unsoundness using
> Vec's destructor.
>
> In order to tell the drop checker that we *do* own values of type T, and
> therefore may drop some T's when *we* drop, we must add an extra `PhantomData`
> saying exactly that:
>
> ```rust
> use std::marker;
>
> struct Vec<T> {
>     data: *const T, // *const for variance!
>     len: usize,
>     cap: usize,
>     _owns_T: marker::PhantomData<T>,
> }
> ```

But ever since [RFC 1238](https://rust-lang.github.io/rfcs/1238-nonparametric-dropck.html),
**this is no longer true nor necessary**.

If you were to write:

```rust
struct Vec<T> {
    data: *const T, // `*const` for variance!
    len: usize,
    cap: usize,
}

# #[cfg(any())]
impl<T> Drop for Vec<T> { /* … */ }
```

then the existence of that `impl<T> Drop for Vec<T>` makes it so Rust will consider
that that `Vec<T>` _owns_ values of type `T` (more precisely: may use values of type `T`
in its `Drop` implementation), and Rust will thus not allow them to _dangle_ should a
`Vec<T>` be dropped.

**Adding an extra `_owns_T: PhantomData<T>` field is thus _superfluous_ and accomplishes nothing**.

___

But this situation can sometimes lead to overly restrictive code. That's why the
standard library uses an unstable and `unsafe` attribute to opt back into the old
"unchecked" drop-checking behavior, that this very documentation warned about: the
`#[may_dangle]` attribute.

### An exception: the special case of the standard library and its unstable `#[may_dangle]`

This section can be skipped if you are only writing your own library code; but if you are
curious about what the standard library does with the actual `Vec` definition, you'll notice
that it still needs to use a `_owns_T: PhantomData<T>` field for soundness.

<details><summary>Click here to see why</summary>

Consider the following example:

```rust
fn main() {
    let mut v: Vec<&str> = Vec::new();
    let s: String = "Short-lived".into();
    v.push(&s);
    drop(s);
} // <- `v` is dropped here
```

with a classical `impl<T> Drop for Vec<T> {` definition, the above [is denied].

[is denied]: https://rust.godbolt.org/z/ans15Kqz3

Indeed, in this case we have a `Vec</* T = */ &'s str>` vector of `'s`-lived references
to `str`ings, but in the case of `let s: String`, it is dropped before the `Vec` is, and
thus `'s` **is expired** by the time the `Vec` is dropped, and the
`impl<'s> Drop for Vec<&'s str> {` is used.

This means that if such `Drop` were to be used, it would be dealing with an _expired_, or
_dangling_ lifetime `'s`. But this is contrary to Rust principles, where by default all
Rust references involved in a function signature are non-dangling and valid to dereference.

Hence why Rust has to conservatively deny this snippet.

And yet, in the case of the real `Vec`, the `Drop` impl does not care about `&'s str`,
_since it has no drop glue of its own_: it only wants to deallocate the backing buffer.

In other words, it would be nice if the above snippet was somehow accepted, by special
casing `Vec`, or by relying on some special property of `Vec`: `Vec` could try to
_promise not to use the `&'s str`s it holds when being dropped_.

This is the kind of `unsafe` promise that can be expressed with `#[may_dangle]`:

```rust ,ignore
unsafe impl<#[may_dangle] 's> Drop for Vec<&'s str> { /* … */ }
```

or, more generally:

```rust ,ignore
unsafe impl<#[may_dangle] T> Drop for Vec<T> { /* … */ }
```

is the `unsafe` way to opt out of this conservative assumption that Rust's drop
checker makes about type parameters of a dropped instance not being allowed to dangle.

And when this is done, such as in the standard library, we need to be careful in the
case where `T` has drop glue of its own. In this instance, imagine replacing the
`&'s str`s with a `struct PrintOnDrop<'s> /* = */ (&'s str);` which would have a
`Drop` impl wherein the inner `&'s str` would be dereferenced and printed to the screen.

Indeed, `Drop for Vec<T> {`, before deallocating the backing buffer, does have to transitively
drop each `T` item when it has drop glue; in the case of `PrintOnDrop<'s>`, it means that
`Drop for Vec<PrintOnDrop<'s>>` has to transitively drop the `PrintOnDrop<'s>`s elements before
deallocating the backing buffer.

So when we said that `'s` `#[may_dangle]`, it was an excessively loose statement. We'd rather want
to say: "`'s` may dangle provided it not be involved in some transitive drop glue". Or, more generally,
"`T` may dangle provided it not be involved in some transitive drop glue". This "exception to the
exception" is a pervasive situation whenever **we own a `T`**. That's why Rust's `#[may_dangle]` is
smart enough to know of this opt-out, and will thus be disabled _when the generic parameter is held
in an owned fashion_ by the fields of the struct.

Hence why the standard library ends up with:

```rust
# #[cfg(any())]
// we pinky-swear not to use `T` when dropping a `Vec`…
unsafe impl<#[may_dangle] T> Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe {
            if mem::needs_drop::<T>() {
                /* … except here, that is, … */
                ptr::drop_in_place::<[T]>(/* … */);
            }
            // …
            dealloc(/* … */)
            // …
        }
    }
}

struct Vec<T> {
    // … except for the fact that a `Vec` owns `T` items and
    // may thus be dropping `T` items on drop!
    _owns_T: core::marker::PhantomData<T>,

    ptr: *const T, // `*const` for variance (but this does not express ownership of a `T` *per se*)
    len: usize,
    cap: usize,
}
```

</details>

___

Raw pointers that own an allocation is such a pervasive pattern that the
standard library made a utility for itself called `Unique<T>` which:

* wraps a `*const T` for variance
* includes a `PhantomData<T>`
* auto-derives `Send`/`Sync` as if T was contained
* marks the pointer as `NonZero` for the null-pointer optimization

## Table of `PhantomData` patterns

Here’s a table of all the wonderful ways `PhantomData` could be used:

| Phantom type                | `'a`      | `T`                         | `Send`    | `Sync`    |
|-----------------------------|-----------|-----------------------------|-----------|-----------|
| `PhantomData<T>`            | -         | covariant (with drop check) | `T: Send` | `T: Sync` |
| `PhantomData<&'a T>`        | covariant | covariant                   | `T: Sync` | `T: Sync` |
| `PhantomData<&'a mut T>`    | covariant | invariant                   | `T: Send` | `T: Sync` |
| `PhantomData<*const T>`     | -         | covariant                   | -         | -         |
| `PhantomData<*mut T>`       | -         | invariant                   | -         | -         |
| `PhantomData<fn(T)>`        | -         | contravariant               | `Send`    | `Sync`    |
| `PhantomData<fn() -> T>`    | -         | covariant                   | `Send`    | `Sync`    |
| `PhantomData<fn(T) -> T>`   | -         | invariant                   | `Send`    | `Sync`    |
| `PhantomData<Cell<&'a ()>>` | invariant | -                           | `Send`    | -         |
