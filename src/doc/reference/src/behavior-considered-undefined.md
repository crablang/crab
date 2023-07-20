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

***Warning:*** The following list is not exhaustive. There is no formal model of
Rust's semantics for what is and is not allowed in unsafe code, so there may be
more behavior considered unsafe. The following list is just what we know for
sure is undefined behavior. Please read the [Rustonomicon] before writing unsafe
code.

</div>

* Data races.
* Evaluating a [dereference expression] (`*expr`) on a raw pointer that is
  [dangling] or unaligned, even in [place expression context]
  (e.g. `addr_of!(*expr)`).
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
* Mutating immutable data. All data inside a [`const`] item is immutable. Moreover, all
  data reached through a shared reference or data owned by an immutable binding
  is immutable, unless that data is contained within an [`UnsafeCell<U>`].
* Invoking undefined behavior via compiler intrinsics.
* Executing code compiled with platform features that the current platform
  does not support (see [`target_feature`]), *except* if the platform explicitly documents this to be safe.
* Calling a function with the wrong call ABI or unwinding from a function with the wrong unwind ABI.
* Producing an invalid value, even in private fields and locals. "Producing" a
  value happens any time a value is assigned to or read from a place, passed to
  a function/primitive operation or returned from a function/primitive
  operation.
  The following values are invalid (at their respective type):
  * A value other than `false` (`0`) or `true` (`1`) in a [`bool`].
  * A discriminant in an `enum` not included in the type definition.
  * A null `fn` pointer.
  * A value in a `char` which is a surrogate or above `char::MAX`.
  * A `!` (all values are invalid for this type).
  * An integer (`i*`/`u*`), floating point value (`f*`), or raw pointer obtained
    from [uninitialized memory][undef], or uninitialized memory in a `str`.
  * A reference or `Box<T>` that is [dangling], unaligned, or points to an invalid value.
  * Invalid metadata in a wide reference, `Box<T>`, or raw pointer:
    * `dyn Trait` metadata is invalid if it is not a pointer to a vtable for
      `Trait` that matches the actual dynamic trait the pointer or reference points to.
    * Slice metadata is invalid if the length is not a valid `usize`
      (i.e., it must not be read from uninitialized memory).
  * Invalid values for a type with a custom definition of invalid values.
    In the standard library, this affects [`NonNull<T>`] and [`NonZero*`].

    > **Note**: `rustc` achieves this with the unstable
    > `rustc_layout_scalar_valid_range_*` attributes.
* Incorrect use of inline assembly. For more details, refer to the [rules] to
  follow when writing code that uses inline assembly.

**Note:** Uninitialized memory is also implicitly invalid for any type that has
a restricted set of valid values. In other words, the only cases in which
reading uninitialized memory is permitted are inside `union`s and in "padding"
(the gaps between the fields/elements of a type).

> **Note**: Undefined behavior affects the entire program. For example, calling
> a function in C that exhibits undefined behavior of C means your entire
> program contains undefined behaviour that can also affect the Rust code. And
> vice versa, undefined behavior in Rust can cause adverse affects on code
> executed by any FFI calls to other languages.

### Dangling pointers
[dangling]: #dangling-pointers

A reference/pointer is "dangling" if it is null or not all of the bytes it
points to are part of the same live allocation (so in particular they all have to be
part of *some* allocation). The span of bytes it points to is determined by the
pointer value and the size of the pointee type (using `size_of_val`).

If the size is 0, then the pointer must either point inside of a live allocation
(including pointing just after the last byte of the allocation), or it must be
directly constructed from a non-zero integer literal.

Note that dynamically sized types (such as slices and strings) point to their
entire range, so it is important that the length metadata is never too large. In
particular, the dynamic size of a Rust value (as determined by `size_of_val`)
must never exceed `isize::MAX`.

[`bool`]: types/boolean.md
[`const`]: items/constant-items.md
[noalias]: http://llvm.org/docs/LangRef.html#noalias
[pointer aliasing rules]: http://llvm.org/docs/LangRef.html#pointer-aliasing-rules
[undef]: http://llvm.org/docs/LangRef.html#undefined-values
[`target_feature`]: attributes/codegen.md#the-target_feature-attribute
[`UnsafeCell<U>`]: ../std/cell/struct.UnsafeCell.html
[Rustonomicon]: ../nomicon/index.html
[`NonNull<T>`]: ../core/ptr/struct.NonNull.html
[`NonZero*`]: ../core/num/index.html
[dereference expression]: expressions/operator-expr.md#the-dereference-operator
[place expression context]: expressions.md#place-expressions-and-value-expressions
[rules]: inline-assembly.md#rules-for-inline-assembly
