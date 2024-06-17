# Interior Mutability

Sometimes a type needs to be mutated while having multiple aliases. In Rust this
is achieved using a pattern called _interior mutability_. A type has interior
mutability if its internal state can be changed through a [shared reference] to
it. This goes against the usual [requirement][ub] that the value pointed to by a
shared reference is not mutated.

[`std::cell::UnsafeCell<T>`] type is the only allowed way to disable
this requirement. When `UnsafeCell<T>` is immutably aliased, it is still safe to
mutate, or obtain a mutable reference to, the `T` it contains. As with all
other types, it is undefined behavior to have multiple `&mut UnsafeCell<T>`
aliases.

Other types with interior mutability can be created by using `UnsafeCell<T>` as
a field. The standard library provides a variety of types that provide safe
interior mutability APIs. For example, [`std::cell::RefCell<T>`] uses run-time
borrow checks to ensure the usual rules around multiple references. The
[`std::sync::atomic`] module contains types that wrap a value that is only
accessed with atomic operations, allowing the value to be shared and mutated
across threads.

[shared reference]: types/pointer.md#shared-references-
[ub]: behavior-considered-undefined.md
[`std::cell::UnsafeCell<T>`]: ../std/cell/struct.UnsafeCell.html
[`std::cell::RefCell<T>`]: ../std/cell/struct.RefCell.html
[`std::sync::atomic`]: ../std/sync/atomic/index.html


