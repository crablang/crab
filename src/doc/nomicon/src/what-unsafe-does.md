# What Unsafe Rust Can Do

The only things that are different in Unsafe Rust are that you can:

* Dereference raw pointers
* Call `unsafe` functions (including C functions, compiler intrinsics, and the raw allocator)
* Implement `unsafe` traits
* Mutate statics
* Access fields of `union`s

That's it. The reason these operations are relegated to Unsafe is that misusing
any of these things will cause the ever dreaded Undefined Behavior. Invoking
Undefined Behavior gives the compiler full rights to do arbitrarily bad things
to your program. You definitely *should not* invoke Undefined Behavior.

Unlike C, Undefined Behavior is pretty limited in scope in Rust. All the core
language cares about is preventing the following things:

* Dereferencing (using the `*` operator on) dangling or unaligned pointers (see below)
* Breaking the [pointer aliasing rules][]
* Calling a function with the wrong call ABI or unwinding from a function with the wrong unwind ABI.
* Causing a [data race][race]
* Executing code compiled with [target features][] that the current thread of execution does
  not support
* Producing invalid values (either alone or as a field of a compound type such
  as `enum`/`struct`/array/tuple):
  * a `bool` that isn't 0 or 1
  * an `enum` with an invalid discriminant
  * a null `fn` pointer
  * a `char` outside the ranges [0x0, 0xD7FF] and [0xE000, 0x10FFFF]
  * a `!` (all values are invalid for this type)
  * an integer (`i*`/`u*`), floating point value (`f*`), or raw pointer read from
    [uninitialized memory][], or uninitialized memory in a `str`.
  * a reference/`Box` that is dangling, unaligned, or points to an invalid value.
  * a wide reference, `Box`, or raw pointer that has invalid metadata:
    * `dyn Trait` metadata is invalid if it is not a pointer to a vtable for
      `Trait` that matches the actual dynamic trait the pointer or reference points to
    * slice metadata is invalid if the length is not a valid `usize`
      (i.e., it must not be read from uninitialized memory)
  * a type with custom invalid values that is one of those values, such as a
    [`NonNull`] that is null. (Requesting custom invalid values is an unstable
    feature, but some stable libstd types, like `NonNull`, make use of it.)

"Producing" a value happens any time a value is assigned, passed to a
function/primitive operation or returned from a function/primitive operation.

A reference/pointer is "dangling" if it is null or not all of the bytes it
points to are part of the same allocation (so in particular they all have to be
part of *some* allocation). The span of bytes it points to is determined by the
pointer value and the size of the pointee type. As a consequence, if the span is
empty, "dangling" is the same as "null". Note that slices and strings point
to their entire range, so it's important that the length metadata is never too
large (in particular, allocations and therefore slices and strings cannot be
bigger than `isize::MAX` bytes). If for some reason this is too cumbersome,
consider using raw pointers.

That's it. That's all the causes of Undefined Behavior baked into Rust. Of
course, unsafe functions and traits are free to declare arbitrary other
constraints that a program must maintain to avoid Undefined Behavior. For
instance, the allocator APIs declare that deallocating unallocated memory is
Undefined Behavior.

However, violations of these constraints generally will just transitively lead to one of
the above problems. Some additional constraints may also derive from compiler
intrinsics that make special assumptions about how code can be optimized. For instance,
Vec and Box make use of intrinsics that require their pointers to be non-null at all times.

Rust is otherwise quite permissive with respect to other dubious operations.
Rust considers it "safe" to:

* Deadlock
* Have a [race condition][race]
* Leak memory
* Overflow integers (with the built-in operators such as `+` etc.)
* Abort the program
* Delete the production database

However any program that actually manages to do such a thing is *probably*
incorrect. Rust provides lots of tools to make these things rare, but
these problems are considered impractical to categorically prevent.

[pointer aliasing rules]: references.html
[uninitialized memory]: uninitialized.html
[race]: races.html
[target features]: ../reference/attributes/codegen.html#the-target_feature-attribute
[`NonNull`]: ../std/ptr/struct.NonNull.html
