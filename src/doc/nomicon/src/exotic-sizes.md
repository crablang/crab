# Exotically Sized Types

Most of the time, we expect types to have a statically known and positive size.
This isn't always the case in Rust.

## Dynamically Sized Types (DSTs)

Rust supports Dynamically Sized Types (DSTs): types without a statically
known size or alignment. On the surface, this is a bit nonsensical: Rust *must*
know the size and alignment of something in order to correctly work with it! In
this regard, DSTs are not normal types. Because they lack a statically known
size, these types can only exist behind a pointer. Any pointer to a
DST consequently becomes a *wide* pointer consisting of the pointer and the
information that "completes" them (more on this below).

There are two major DSTs exposed by the language:

* trait objects: `dyn MyTrait`
* slices: [`[T]`][slice], [`str`], and others

A trait object represents some type that implements the traits it specifies.
The exact original type is *erased* in favor of runtime reflection
with a vtable containing all the information necessary to use the type.
The information that completes a trait object pointer is the vtable pointer.
The runtime size of the pointee can be dynamically requested from the vtable.

A slice is simply a view into some contiguous storage -- typically an array or
`Vec`. The information that completes a slice pointer is just the number of elements
it points to. The runtime size of the pointee is just the statically known size
of an element multiplied by the number of elements.

Structs can actually store a single DST directly as their last field, but this
makes them a DST as well:

```rust
// Can't be stored on the stack directly
struct MySuperSlice {
    info: u32,
    data: [u8],
}
```

Although such a type is largely useless without a way to construct it. Currently the
only properly supported way to create a custom DST is by making your type generic
and performing an *unsizing coercion*:

```rust
struct MySuperSliceable<T: ?Sized> {
    info: u32,
    data: T,
}

fn main() {
    let sized: MySuperSliceable<[u8; 8]> = MySuperSliceable {
        info: 17,
        data: [0; 8],
    };

    let dynamic: &MySuperSliceable<[u8]> = &sized;

    // prints: "17 [0, 0, 0, 0, 0, 0, 0, 0]"
    println!("{} {:?}", dynamic.info, &dynamic.data);
}
```

(Yes, custom DSTs are a largely half-baked feature for now.)

## Zero Sized Types (ZSTs)

Rust also allows types to be specified that occupy no space:

```rust
struct Nothing; // No fields = no size

// All fields have no size = no size
struct LotsOfNothing {
    foo: Nothing,
    qux: (),      // empty tuple has no size
    baz: [u8; 0], // empty array has no size
}
```

On their own, Zero Sized Types (ZSTs) are, for obvious reasons, pretty useless.
However as with many curious layout choices in Rust, their potential is realized
in a generic context: Rust largely understands that any operation that produces
or stores a ZST can be reduced to a no-op. First off, storing it doesn't even
make sense -- it doesn't occupy any space. Also there's only one value of that
type, so anything that loads it can just produce it from the aether -- which is
also a no-op since it doesn't occupy any space.

One of the most extreme examples of this is Sets and Maps. Given a
`Map<Key, Value>`, it is common to implement a `Set<Key>` as just a thin wrapper
around `Map<Key, UselessJunk>`. In many languages, this would necessitate
allocating space for UselessJunk and doing work to store and load UselessJunk
only to discard it. Proving this unnecessary would be a difficult analysis for
the compiler.

However in Rust, we can just say that  `Set<Key> = Map<Key, ()>`. Now Rust
statically knows that every load and store is useless, and no allocation has any
size. The result is that the monomorphized code is basically a custom
implementation of a HashSet with none of the overhead that HashMap would have to
support values.

Safe code need not worry about ZSTs, but *unsafe* code must be careful about the
consequence of types with no size. In particular, pointer offsets are no-ops,
and allocators typically [require a non-zero size][alloc].

Note that references to ZSTs (including empty slices), just like all other
references, must be non-null and suitably aligned. Dereferencing a null or
unaligned pointer to a ZST is [undefined behavior][ub], just like for any other
type.

[alloc]: ../std/alloc/trait.GlobalAlloc.html#tymethod.alloc
[ub]: what-unsafe-does.html

## Empty Types

Rust also enables types to be declared that *cannot even be instantiated*. These
types can only be talked about at the type level, and never at the value level.
Empty types can be declared by specifying an enum with no variants:

```rust
enum Void {} // No variants = EMPTY
```

Empty types are even more marginal than ZSTs. The primary motivating example for
an empty type is type-level unreachability. For instance, suppose an API needs to
return a Result in general, but a specific case actually is infallible. It's
actually possible to communicate this at the type level by returning a
`Result<T, Void>`. Consumers of the API can confidently unwrap such a Result
knowing that it's *statically impossible* for this value to be an `Err`, as
this would require providing a value of type `Void`.

In principle, Rust can do some interesting analyses and optimizations based
on this fact. For instance, `Result<T, Void>` is represented as just `T`,
because the `Err` case doesn't actually exist (strictly speaking, this is only
an optimization that is not guaranteed, so for example transmuting one into the
other is still Undefined Behavior).

The following *could* also compile:

```rust,compile_fail
enum Void {}

let res: Result<u32, Void> = Ok(0);

// Err doesn't exist anymore, so Ok is actually irrefutable.
let Ok(num) = res;
```

But this trick doesn't work yet.

One final subtle detail about empty types is that raw pointers to them are
actually valid to construct, but dereferencing them is Undefined Behavior
because that wouldn't make sense.

We recommend against modelling C's `void*` type with `*const Void`.
A lot of people started doing that but quickly ran into trouble because
Rust doesn't really have any safety guards against trying to instantiate
empty types with unsafe code, and if you do it, it's Undefined Behavior.
This was especially problematic because developers had a habit of converting
raw pointers to references and `&Void` is *also* Undefined Behavior to
construct.

`*const ()` (or equivalent) works reasonably well for `void*`, and can be made
into a reference without any safety problems. It still doesn't prevent you from
trying to read or write values, but at least it compiles to a no-op instead
of Undefined Behavior.

## Extern Types

There is [an accepted RFC][extern-types] to add proper types with an unknown size,
called *extern types*, which would let Rust developers model things like C's `void*`
and other "declared but never defined" types more accurately. However as of
Rust 2018, [the feature is stuck in limbo over how `size_of_val::<MyExternType>()`
should behave][extern-types-issue].

[extern-types]: https://github.com/rust-lang/rfcs/blob/master/text/1861-extern-types.md
[extern-types-issue]: https://github.com/rust-lang/rust/issues/43467
[`str`]: ../std/primitive.str.html
[slice]: ../std/primitive.slice.html
