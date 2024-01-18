# repr(Rust)

First and foremost, all types have an alignment specified in bytes. The
alignment of a type specifies what addresses are valid to store the value at. A
value with alignment `n` must only be stored at an address that is a multiple of
`n`. So alignment 2 means you must be stored at an even address, and 1 means
that you can be stored anywhere. Alignment is at least 1, and always a power
of 2.

Primitives are usually aligned to their size, although this is
platform-specific behavior. For example, on x86 `u64` and `f64` are often
aligned to 4 bytes (32 bits).

A type's size must always be a multiple of its alignment (Zero being a valid size
for any alignment). This ensures that an array of that type may always be indexed
by offsetting by a multiple of its size. Note that the size and alignment of a
type may not be known statically in the case of [dynamically sized types][dst].

Rust gives you the following ways to lay out composite data:

* structs (named product types)
* tuples (anonymous product types)
* arrays (homogeneous product types)
* enums (named sum types -- tagged unions)
* unions (untagged unions)

An enum is said to be *field-less* if none of its variants have associated data.

By default, composite structures have an alignment equal to the maximum
of their fields' alignments. Rust will consequently insert padding where
necessary to ensure that all fields are properly aligned and that the overall
type's size is a multiple of its alignment. For instance:

```rust
struct A {
    a: u8,
    b: u32,
    c: u16,
}
```

will be 32-bit aligned on a target that aligns these primitives to their
respective sizes. The whole struct will therefore have a size that is a multiple
of 32-bits. It may become:

```rust
struct A {
    a: u8,
    _pad1: [u8; 3], // to align `b`
    b: u32,
    c: u16,
    _pad2: [u8; 2], // to make overall size multiple of 4
}
```

or maybe:

```rust
struct A {
    b: u32,
    c: u16,
    a: u8,
    _pad: u8,
}
```

There is *no indirection* for these types; all data is stored within the struct,
as you would expect in C. However with the exception of arrays (which are
densely packed and in-order), the layout of data is not specified by default.
Given the two following struct definitions:

```rust
struct A {
    a: i32,
    b: u64,
}

struct B {
    a: i32,
    b: u64,
}
```

Rust *does* guarantee that two instances of A have their data laid out in
exactly the same way. However Rust *does not* currently guarantee that an
instance of A has the same field ordering or padding as an instance of B.

With A and B as written, this point would seem to be pedantic, but several other
features of Rust make it desirable for the language to play with data layout in
complex ways.

For instance, consider this struct:

```rust
struct Foo<T, U> {
    count: u16,
    data1: T,
    data2: U,
}
```

Now consider the monomorphizations of `Foo<u32, u16>` and `Foo<u16, u32>`. If
Rust lays out the fields in the order specified, we expect it to pad the
values in the struct to satisfy their alignment requirements. So if Rust
didn't reorder fields, we would expect it to produce the following:

<!-- ignore: explanation code -->
```rust,ignore
struct Foo<u16, u32> {
    count: u16,
    data1: u16,
    data2: u32,
}

struct Foo<u32, u16> {
    count: u16,
    _pad1: u16,
    data1: u32,
    data2: u16,
    _pad2: u16,
}
```

The latter case quite simply wastes space. An optimal use of space
requires different monomorphizations to have *different field orderings*.

Enums make this consideration even more complicated. Naively, an enum such as:

```rust
enum Foo {
    A(u32),
    B(u64),
    C(u8),
}
```

might be laid out as:

```rust
struct FooRepr {
    data: u64, // this is either a u64, u32, or u8 based on `tag`
    tag: u8,   // 0 = A, 1 = B, 2 = C
}
```

And indeed this is approximately how it would be laid out (modulo the
size and position of `tag`).

However there are several cases where such a representation is inefficient. The
classic case of this is Rust's "null pointer optimization": an enum consisting
of a single outer unit variant (e.g. `None`) and a (potentially nested) non-
nullable pointer variant (e.g. `Some(&T)`) makes the tag unnecessary. A null
pointer can safely be interpreted as the unit (`None`) variant. The net
result is that, for example, `size_of::<Option<&T>>() == size_of::<&T>()`.

There are many types in Rust that are, or contain, non-nullable pointers such as
`Box<T>`, `Vec<T>`, `String`, `&T`, and `&mut T`. Similarly, one can imagine
nested enums pooling their tags into a single discriminant, as they are by
definition known to have a limited range of valid values. In principle enums could
use fairly elaborate algorithms to store bits throughout nested types with
forbidden values. As such it is *especially* desirable that
we leave enum layout unspecified today.

[dst]: exotic-sizes.html#dynamically-sized-types-dsts
