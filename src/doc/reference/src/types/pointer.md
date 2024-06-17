# Pointer types

All pointers are explicit first-class values.
They can be moved or copied, stored into data structs, and returned from functions.

## References (`&` and `&mut`)

> **<sup>Syntax</sup>**\
> _ReferenceType_ :\
> &nbsp;&nbsp; `&` [_Lifetime_]<sup>?</sup> `mut`<sup>?</sup> [_TypeNoBounds_]

### Shared references (`&`)

Shared references point to memory which is owned by some other value.
When a shared reference to a value is created, it prevents direct mutation of the value.
[Interior mutability] provides an exception for this in certain circumstances.
As the name suggests, any number of shared references to a value may exist.
A shared reference type is written `&type`, or `&'a type` when you need to specify an explicit lifetime.
Copying a reference is a "shallow" operation:
it involves only copying the pointer itself, that is, pointers are `Copy`.
Releasing a reference has no effect on the value it points to, but referencing of a [temporary value] will keep it alive during the scope of the reference itself.

### Mutable references (`&mut`)

Mutable references point to memory which is owned by some other value.
A mutable reference type is written `&mut type` or `&'a mut type`.
A mutable reference (that hasn't been borrowed) is the only way to access the value it points to, so is not `Copy`.

## Raw pointers (`*const` and `*mut`)

> **<sup>Syntax</sup>**\
> _RawPointerType_ :\
> &nbsp;&nbsp; `*` ( `mut` | `const` ) [_TypeNoBounds_]

Raw pointers are pointers without safety or liveness guarantees.
Raw pointers are written as `*const T` or `*mut T`.
For example `*const i32` means a raw pointer to a 32-bit integer.
Copying or dropping a raw pointer has no effect on the lifecycle of any other value.
Dereferencing a raw pointer is an [`unsafe` operation].
This can also be used to convert a raw pointer to a reference by reborrowing it (`&*` or `&mut *`).
Raw pointers are generally discouraged;
they exist to support interoperability with foreign code, and writing performance-critical or low-level functions.

When comparing raw pointers they are compared by their address, rather than by what they point to.
When comparing raw pointers to [dynamically sized types] they also have their additional data compared.

Raw pointers can be created directly using [`core::ptr::addr_of!`] for `*const` pointers and [`core::ptr::addr_of_mut!`] for `*mut` pointers.

## Smart Pointers

The standard library contains additional 'smart pointer' types beyond references and raw pointers.

## Bit validity

Despite pointers and references being similar to `usize`s in the machine code emitted on most platforms,
the semantics of transmuting a reference or pointer type to a non-pointer type is currently undecided.
Thus, it may not be valid to transmute a pointer or reference type, `P`, to a `[u8; size_of::<P>()]`.

For thin raw pointers (i.e., for `P = *const T` or `P = *mut T` for `T: Sized`),
the inverse direction (transmuting from an integer or array of integers to `P`) is always valid.
However, the pointer produced via such a transmutation may not be dereferenced (not even if `T` has size zero).

[`core::ptr::addr_of!`]: ../../core/ptr/macro.addr_of.html
[`core::ptr::addr_of_mut!`]: ../../core/ptr/macro.addr_of_mut.html
[Interior mutability]: ../interior-mutability.md
[_Lifetime_]: ../trait-bounds.md
[_TypeNoBounds_]: ../types.md#type-expressions
[`unsafe` operation]: ../unsafety.md
[dynamically sized types]: ../dynamically-sized-types.md
[temporary value]: ../expressions.md#temporaries
