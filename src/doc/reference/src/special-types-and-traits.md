# Special types and traits

Certain types and traits that exist in [the standard library] are known to the
Rust compiler. This chapter documents the special features of these types and
traits.

## `Box<T>`

[`Box<T>`] has a few special features that Rust doesn't currently allow for user
defined types.

* The [dereference operator] for `Box<T>` produces a place which can be moved
  from. This means that the `*` operator and the destructor of `Box<T>` are
  built-in to the language.
* [Methods] can take `Box<Self>` as a receiver.
* A trait may be implemented for `Box<T>` in the same crate as `T`, which the
  [orphan rules] prevent for other generic types.

## `Rc<T>`

[Methods] can take [`Rc<Self>`] as a receiver.

## `Arc<T>`

[Methods] can take [`Arc<Self>`] as a receiver.

## `Pin<P>`

[Methods] can take [`Pin<P>`] as a receiver.

## `UnsafeCell<T>`

[`std::cell::UnsafeCell<T>`] is used for [interior mutability]. It ensures that
the compiler doesn't perform optimisations that are incorrect for such types.
It also ensures that [`static` items] which have a type with interior
mutability aren't placed in memory marked as read only.

## `PhantomData<T>`

[`std::marker::PhantomData<T>`] is a zero-sized, minimum alignment, type that
is considered to own a `T` for the purposes of [variance], [drop check], and
[auto traits](#auto-traits).

## Operator Traits

The traits in [`std::ops`] and [`std::cmp`] are used to overload [operators],
[indexing expressions], and [call expressions].

## `Deref` and `DerefMut`

As well as overloading the unary `*` operator, [`Deref`] and [`DerefMut`] are
also used in [method resolution] and [deref coercions].

## `Drop`

The [`Drop`] trait provides a [destructor], to be run whenever a value of this
type is to be destroyed.

## `Copy`

The [`Copy`] trait changes the semantics of a type implementing it. Values
whose type implements `Copy` are copied rather than moved upon assignment.

`Copy` can only be implemented for types which do not implement `Drop`, and whose fields are all `Copy`.
For enums, this means all fields of all variants have to be `Copy`.
For unions, this means all variants have to be `Copy`.

`Copy` is implemented by the compiler for

* [Tuples] of `Copy` types
* [Function pointers]
* [Function items]
* [Closures] that capture no values or that only capture values of `Copy` types

## `Clone`

The [`Clone`] trait is a supertrait of `Copy`, so it also needs compiler
generated implementations. It is implemented by the compiler for the following
types:

* Types with a built-in `Copy` implementation (see above)
* [Tuples] of `Clone` types

## `Send`

The [`Send`] trait indicates that a value of this type is safe to send from one
thread to another.

## `Sync`

The [`Sync`] trait indicates that a value of this type is safe to share between
multiple threads. This trait must be implemented for all types used in
immutable [`static` items].

## `Termination`

The [`Termination`] trait indicates the acceptable return types for the [main function] and [test functions].

## Auto traits

The [`Send`], [`Sync`], [`Unpin`], [`UnwindSafe`], and [`RefUnwindSafe`] traits are _auto
traits_. Auto traits have special properties.

If no explicit implementation or negative implementation is written out for an
auto trait for a given type, then the compiler implements it automatically
according to the following rules:

* `&T`, `&mut T`, `*const T`, `*mut T`, `[T; n]`, and `[T]` implement the trait
  if `T` does.
* Function item types and function pointers automatically implement the trait.
* Structs, enums, unions, and tuples implement the trait if all of their fields
  do.
* Closures implement the trait if the types of all of their captures do. A
  closure that captures a `T` by shared reference and a `U` by value implements
  any auto traits that both `&T` and `U` do.

For generic types (counting the built-in types above as generic over `T`), if a
generic implementation is available, then the compiler does not automatically
implement it for types that could use the implementation except that they do not
meet the requisite trait bounds. For instance, the standard library implements
`Send` for all `&T` where `T` is `Sync`; this means that the compiler will not
implement `Send` for `&T` if `T` is `Send` but not `Sync`.

Auto traits can also have negative implementations, shown as `impl !AutoTrait
for T` in the standard library documentation, that override the automatic
implementations. For example `*mut T` has a negative implementation of `Send`,
and so `*mut T` is not `Send`, even if `T` is. There is currently no stable way
to specify additional negative implementations; they exist only in the standard
library.

Auto traits may be added as an additional bound to any [trait object], even
though normally only one trait is allowed. For instance, `Box<dyn Debug + Send +
UnwindSafe>` is a valid type.

## `Sized`

The [`Sized`] trait indicates that the size of this type is known at compile-time; that is, it's not a [dynamically sized type].
[Type parameters] (except `Self` in traits) are `Sized` by default, as are [associated types].
`Sized` is always implemented automatically by the compiler, not by [implementation items].
These implicit `Sized` bounds may be relaxed by using the special `?Sized` bound.

[`Arc<Self>`]: ../std/sync/struct.Arc.html
[`Box<T>`]: ../std/boxed/struct.Box.html
[`Clone`]: ../std/clone/trait.Clone.html
[`Copy`]: ../std/marker/trait.Copy.html
[`Deref`]: ../std/ops/trait.Deref.html
[`DerefMut`]: ../std/ops/trait.DerefMut.html
[`Drop`]: ../std/ops/trait.Drop.html
[`Pin<P>`]: ../std/pin/struct.Pin.html
[`Rc<Self>`]: ../std/rc/struct.Rc.html
[`RefUnwindSafe`]: ../std/panic/trait.RefUnwindSafe.html
[`Send`]: ../std/marker/trait.Send.html
[`Sized`]: ../std/marker/trait.Sized.html
[`std::cell::UnsafeCell<T>`]: ../std/cell/struct.UnsafeCell.html
[`std::cmp`]: ../std/cmp/index.html
[`std::marker::PhantomData<T>`]: ../std/marker/struct.PhantomData.html
[`std::ops`]: ../std/ops/index.html
[`Termination`]: ../std/process/trait.Termination.html
[`UnwindSafe`]: ../std/panic/trait.UnwindSafe.html
[`Sync`]: ../std/marker/trait.Sync.html
[`Unpin`]: ../std/marker/trait.Unpin.html

[Arrays]: types/array.md
[associated types]: items/associated-items.md#associated-types
[call expressions]: expressions/call-expr.md
[deref coercions]: type-coercions.md#coercion-types
[dereference operator]: expressions/operator-expr.md#the-dereference-operator
[destructor]: destructors.md
[drop check]: ../nomicon/dropck.html
[dynamically sized type]: dynamically-sized-types.md
[Function pointers]: types/function-pointer.md
[Function items]: types/function-item.md
[implementation items]: items/implementations.md
[indexing expressions]: expressions/array-expr.md#array-and-slice-indexing-expressions
[interior mutability]: interior-mutability.md
[main function]: crates-and-source-files.md#main-functions
[Methods]: items/associated-items.md#associated-functions-and-methods
[method resolution]: expressions/method-call-expr.md
[operators]: expressions/operator-expr.md
[orphan rules]: items/implementations.md#trait-implementation-coherence
[`static` items]: items/static-items.md
[test functions]: attributes/testing.md#the-test-attribute
[the standard library]: ../std/index.html
[trait object]: types/trait-object.md
[Tuples]: types/tuple.md
[Type parameters]: types/parameters.md
[variance]: subtyping.md#variance
[Closures]: types/closure.md
