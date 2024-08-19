## Behavior considered undefined

Rust code is incorrect if it exhibits any of the behaviors in the following
list. This includes code within `unsafe` blocks and `unsafe` functions.
`unsafe` only means that avoiding undefined behavior is on the programmer; it
does not change anything about the fact that Rust programs must never cause
undefined behavior.

It is the programmer's responsibility when writing `unsafe` code to ensure that
any safe code interacting with the `unsafe` code cannot trigger these
behaviors. `unsafe` code that satisfies this property for any safe client is
called *sound*; if `unsafe` code can be misused by safe code to exhibit
undefined behavior, it is *unsound*.

<div class="warning">

***Warning:*** The following list is not exhaustive; it may grow or shrink.
There is no formal model of Rust's semantics for what is and is not allowed in
unsafe code, so there may be more behavior considered unsafe. We also reserve
the right to make some of the behavior in that list defined in the future. In
other words, this list does not say that anything will *definitely* always be
undefined in all future Rust version (but we might make such commitments for
some list items in the future).

Please read the [Rustonomicon] before writing unsafe code.

</div>

* Data races.
* Accessing (loading from or storing to) a place that is [dangling] or [based on
  a misaligned pointer].
* Performing a place projection that violates the requirements of [in-bounds
  pointer arithmetic][offset]. A place projection is a [field
  expression][project-field], a [tuple index expression][project-tuple], or an
  [array/slice index expression][project-slice].
* Breaking the [pointer aliasing rules]. `Box<T>`, `&mut T` and `&T` follow
  LLVM’s scoped [noalias] model, except if the `&T` contains an
  [`UnsafeCell<U>`]. References and boxes must not be [dangling] while they are
  live. The exact liveness duration is not specified, but some bounds exist:
  * For references, the liveness duration is upper-bounded by the syntactic
    lifetime assigned by the borrow checker; it cannot be live any *longer* than
    that lifetime.
  * Each time a reference or box is passed to or returned from a function, it is
    considered live.
  * When a reference (but not a `Box`!) is passed to a function, it is live at
    least as long as that function call, again except if the `&T` contains an
    [`UnsafeCell<U>`].

  All this also applies when values of these
  types are passed in a (nested) field of a compound type, but not behind
  pointer indirections.
* Mutating immutable bytes. All bytes inside a [`const`] item are immutable.
  The bytes owned by an immutable binding or immutable `static` are immutable, unless those bytes are part of an [`UnsafeCell<U>`].

  Moreover, the bytes [pointed to] by a shared reference, including transitively through other references (both shared and mutable) and `Box`es, are immutable; transitivity includes those references stored in fields of compound types.

  A mutation is any write of more than 0 bytes which overlaps with any of the relevant bytes (even if that write does not change the memory contents).
* Invoking undefined behavior via compiler intrinsics.
* Executing code compiled with platform features that the current platform
  does not support (see [`target_feature`]), *except* if the platform explicitly documents this to be safe.
* Calling a function with the wrong call ABI or unwinding from a function with the wrong unwind ABI.
* Producing an [invalid value][invalid-values]. "Producing" a
  value happens any time a value is assigned to or read from a place, passed to
  a function/primitive operation or returned from a function/primitive
  operation.
* Incorrect use of inline assembly. For more details, refer to the [rules] to
  follow when writing code that uses inline assembly.
* **In [const context](const_eval.md#const-context)**: transmuting or otherwise
  reinterpreting a pointer (reference, raw pointer, or function pointer) into
  some allocated object as a non-pointer type (such as integers).
  'Reinterpreting' refers to loading the pointer value at integer type without a
  cast, e.g. by doing raw pointer casts or using a union.

> **Note**: Undefined behavior affects the entire program. For example, calling
> a function in C that exhibits undefined behavior of C means your entire
> program contains undefined behaviour that can also affect the Rust code. And
> vice versa, undefined behavior in Rust can cause adverse affects on code
> executed by any FFI calls to other languages.

### Pointed-to bytes

The span of bytes a pointer or reference "points to" is determined by the pointer value and the size of the pointee type (using `size_of_val`).

### Places based on misaligned pointers
[based on a misaligned pointer]: #places-based-on-misaligned-pointers

A place is said to be "based on a misaligned pointer" if the last `*` projection
during place computation was performed on a pointer that was not aligned for its
type. (If there is no `*` projection in the place expression, then this is
accessing the field of a local and rustc will guarantee proper alignment. If
there are multiple `*` projection, then each of them incurs a load of the
pointer-to-be-dereferenced itself from memory, and each of these loads is
subject to the alignment constraint. Note that some `*` projections can be
omitted in surface Rust syntax due to automatic dereferencing; we are
considering the fully expanded place expression here.)

For instance, if `ptr` has type `*const S` where `S` has an alignment of 8, then
`ptr` must be 8-aligned or else `(*ptr).f` is "based on an misaligned pointer".
This is true even if the type of the field `f` is `u8` (i.e., a type with
alignment 1). In other words, the alignment requirement derives from the type of
the pointer that was dereferenced, *not* the type of the field that is being
accessed.

Note that a place based on a misaligned pointer only leads to undefined behavior
when it is loaded from or stored to. `addr_of!`/`addr_of_mut!` on such a place
is allowed. `&`/`&mut` on a place requires the alignment of the field type (or
else the program would be "producing an invalid value"), which generally is a
less restrictive requirement than being based on an aligned pointer. Taking a
reference will lead to a compiler error in cases where the field type might be
more aligned than the type that contains it, i.e., `repr(packed)`. This means
that being based on an aligned pointer is always sufficient to ensure that the
new reference is aligned, but it is not always necessary.

### Dangling pointers
[dangling]: #dangling-pointers

A reference/pointer is "dangling" if not all of the bytes it
[points to] are part of the same live allocation (so in particular they all have to be
part of *some* allocation).

If the size is 0, then the pointer is trivially never "dangling"
(even if it is a null pointer).

Note that dynamically sized types (such as slices and strings) point to their
entire range, so it is important that the length metadata is never too large. In
particular, the dynamic size of a Rust value (as determined by `size_of_val`)
must never exceed `isize::MAX`, since it is impossible for a single allocation
to be larger than `isize::MAX`.

### Invalid values
[invalid-values]: #invalid-values

The Rust compiler assumes that all values produced during program execution are
"valid", and producing an invalid value is hence immediate UB.

Whether a value is valid depends on the type:
* A [`bool`] value must be `false` (`0`) or `true` (`1`).
* A `fn` pointer value must be non-null.
* A `char` value must not be a surrogate (i.e., must not be in the range `0xD800..=0xDFFF`) and must be equal to or less than `char::MAX`.
* A `!` value must never exist.
* An integer (`i*`/`u*`), floating point value (`f*`), or raw pointer must be
  initialized, i.e., must not be obtained from [uninitialized memory][undef].
* A `str` value is treated like `[u8]`, i.e. it must be initialized.
* An `enum` must have a valid discriminant, and all fields of the variant indicated by that discriminant must be valid at their respective type.
* A `struct`, tuple, and array requires all fields/elements to be valid at their respective type.
* For a `union`, the exact validity requirements are not decided yet.
  Obviously, all values that can be created entirely in safe code are valid.
  If the union has a zero-sized field, then every possible value is valid.
  Further details are [still being debated](https://github.com/rust-lang/unsafe-code-guidelines/issues/438).
* A reference or [`Box<T>`] must be aligned, it cannot be [dangling], and it must point to a valid value
  (in case of dynamically sized types, using the actual dynamic type of the
  pointee as determined by the metadata).
  Note that the last point (about pointing to a valid value) remains a subject of some debate.
* The metadata of a wide reference, [`Box<T>`], or raw pointer must match
  the type of the unsized tail:
  * `dyn Trait` metadata must be a pointer to a compiler-generated vtable for `Trait`.
    (For raw pointers, this requirement remains a subject of some debate.)
  * Slice (`[T]`) metadata must be a valid `usize`.
    Furthermore, for wide references and [`Box<T>`], slice metadata is invalid
    if it makes the total size of the pointed-to value bigger than `isize::MAX`.
* If a type has a custom range of a valid values, then a valid value must be in that range.
  In the standard library, this affects [`NonNull<T>`] and [`NonZero<T>`].

  > **Note**: `rustc` achieves this with the unstable
  > `rustc_layout_scalar_valid_range_*` attributes.

**Note:** Uninitialized memory is also implicitly invalid for any type that has
a restricted set of valid values. In other words, the only cases in which
reading uninitialized memory is permitted are inside `union`s and in "padding"
(the gaps between the fields of a type).


[`bool`]: types/boolean.md
[`const`]: items/constant-items.md
[noalias]: http://llvm.org/docs/LangRef.html#noalias
[pointer aliasing rules]: http://llvm.org/docs/LangRef.html#pointer-aliasing-rules
[undef]: http://llvm.org/docs/LangRef.html#undefined-values
[`target_feature`]: attributes/codegen.md#the-target_feature-attribute
[`UnsafeCell<U>`]: ../std/cell/struct.UnsafeCell.html
[Rustonomicon]: ../nomicon/index.html
[`NonNull<T>`]: ../core/ptr/struct.NonNull.html
[`NonZero<T>`]: ../core/num/struct.NonZero.html
[`Box<T>`]: ../alloc/boxed/struct.Box.html
[place expression context]: expressions.md#place-expressions-and-value-expressions
[rules]: inline-assembly.md#rules-for-inline-assembly
[points to]: #pointed-to-bytes
[pointed to]: #pointed-to-bytes
[offset]: ../std/primitive.pointer.html#method.offset
[project-field]: expressions/field-expr.md
[project-tuple]: expressions/tuple-expr.md#tuple-indexing-expressions
[project-slice]: expressions/array-expr.md#array-and-slice-indexing-expressions
