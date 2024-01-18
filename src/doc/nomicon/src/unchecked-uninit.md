# Unchecked Uninitialized Memory

One interesting exception to this rule is working with arrays. Safe Rust doesn't
permit you to partially initialize an array. When you initialize an array, you
can either set every value to the same thing with `let x = [val; N]`, or you can
specify each member individually with `let x = [val1, val2, val3]`.
Unfortunately this is pretty rigid, especially if you need to initialize your
array in a more incremental or dynamic way.

Unsafe Rust gives us a powerful tool to handle this problem:
[`MaybeUninit`]. This type can be used to handle memory that has not been fully
initialized yet.

With `MaybeUninit`, we can initialize an array element by element as follows:

```rust
use std::mem::{self, MaybeUninit};

// Size of the array is hard-coded but easy to change (meaning, changing just
// the constant is sufficient). This means we can't use [a, b, c] syntax to
// initialize the array, though, as we would have to keep that in sync
// with `SIZE`!
const SIZE: usize = 10;

let x = {
    // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
    // safe because the type we are claiming to have initialized here is a
    // bunch of `MaybeUninit`s, which do not require initialization.
    let mut x: [MaybeUninit<Box<u32>>; SIZE] = unsafe {
        MaybeUninit::uninit().assume_init()
    };

    // Dropping a `MaybeUninit` does nothing. Thus using raw pointer
    // assignment instead of `ptr::write` does not cause the old
    // uninitialized value to be dropped.
    // Exception safety is not a concern because Box can't panic
    for i in 0..SIZE {
        x[i] = MaybeUninit::new(Box::new(i as u32));
    }

    // Everything is initialized. Transmute the array to the
    // initialized type.
    unsafe { mem::transmute::<_, [Box<u32>; SIZE]>(x) }
};

dbg!(x);
```

This code proceeds in three steps:

1. Create an array of `MaybeUninit<T>`. With current stable Rust, we have to use
   unsafe code for this: we take some uninitialized piece of memory
   (`MaybeUninit::uninit()`) and claim we have fully initialized it
   ([`assume_init()`][assume_init]). This seems ridiculous, because we didn't!
   The reason this is correct is that the array consists itself entirely of
   `MaybeUninit`, which do not actually require initialization. For most other
   types, doing `MaybeUninit::uninit().assume_init()` produces an invalid
   instance of said type, so you got yourself some Undefined Behavior.

2. Initialize the array. The subtle aspect of this is that usually, when we use
   `=` to assign to a value that the Rust type checker considers to already be
   initialized (like `x[i]`), the old value stored on the left-hand side gets
   dropped. This would be a disaster. However, in this case, the type of the
   left-hand side is `MaybeUninit<Box<u32>>`, and dropping that does not do
   anything! See below for some more discussion of this `drop` issue.

3. Finally, we have to change the type of our array to remove the
   `MaybeUninit`. With current stable Rust, this requires a `transmute`.
   This transmute is legal because in memory, `MaybeUninit<T>` looks the same as `T`.

    However, note that in general, `Container<MaybeUninit<T>>>` does *not* look
   the same as `Container<T>`! Imagine if `Container` was `Option`, and `T` was
   `bool`, then `Option<bool>` exploits that `bool` only has two valid values,
   but `Option<MaybeUninit<bool>>` cannot do that because the `bool` does not
   have to be initialized.

    So, it depends on `Container` whether transmuting away the `MaybeUninit` is
   allowed. For arrays, it is (and eventually the standard library will
   acknowledge that by providing appropriate methods).

It's worth spending a bit more time on the loop in the middle, and in particular
the assignment operator and its interaction with `drop`. If we wrote something like:

<!-- ignore: simplified code -->
```rust,ignore
*x[i].as_mut_ptr() = Box::new(i as u32); // WRONG!
```

we would actually overwrite a `Box<u32>`, leading to `drop` of uninitialized
data, which would cause much sadness and pain.

The correct alternative, if for some reason we cannot use `MaybeUninit::new`, is
to use the [`ptr`] module. In particular, it provides three functions that allow
us to assign bytes to a location in memory without dropping the old value:
[`write`], [`copy`], and [`copy_nonoverlapping`].

* `ptr::write(ptr, val)` takes a `val` and moves it into the address pointed
  to by `ptr`.
* `ptr::copy(src, dest, count)` copies the bits that `count` T items would occupy
  from src to dest. (this is equivalent to C's memmove -- note that the argument
  order is reversed!)
* `ptr::copy_nonoverlapping(src, dest, count)` does what `copy` does, but a
  little faster on the assumption that the two ranges of memory don't overlap.
  (this is equivalent to C's memcpy -- note that the argument order is reversed!)

It should go without saying that these functions, if misused, will cause serious
havoc or just straight up Undefined Behavior. The only requirement of these
functions *themselves* is that the locations you want to read and write
are allocated and properly aligned. However, the ways writing arbitrary bits to
arbitrary locations of memory can break things are basically uncountable!

It's worth noting that you don't need to worry about `ptr::write`-style
shenanigans with types which don't implement `Drop` or contain `Drop` types,
because Rust knows not to try to drop them. This is what we relied on in the
above example.

However when working with uninitialized memory you need to be ever-vigilant for
Rust trying to drop values you make like this before they're fully initialized.
Every control path through that variable's scope must initialize the value
before it ends, if it has a destructor.
*[This includes code panicking](unwinding.html)*. `MaybeUninit` helps a bit
here, because it does not implicitly drop its content - but all this really
means in case of a panic is that instead of a double-free of the not yet
initialized parts, you end up with a memory leak of the already initialized
parts.

Note that, to use the `ptr` methods, you need to first obtain a *raw pointer* to
the data you want to initialize. It is illegal to construct a *reference* to
uninitialized data, which implies that you have to be careful when obtaining
said raw pointer:

* For an array of `T`, you can use `base_ptr.add(idx)` where `base_ptr: *mut T`
to compute the address of array index `idx`. This relies on
how arrays are laid out in memory.
* For a struct, however, in general we do not know how it is laid out, and we
also cannot use `&mut base_ptr.field` as that would be creating a
reference. So, you must carefully use the [`addr_of_mut`] macro. This creates
a raw pointer to the field without creating an intermediate reference:

```rust
use std::{ptr, mem::MaybeUninit};

struct Demo {
    field: bool,
}

let mut uninit = MaybeUninit::<Demo>::uninit();
// `&uninit.as_mut().field` would create a reference to an uninitialized `bool`,
// and thus be Undefined Behavior!
let f1_ptr = unsafe { ptr::addr_of_mut!((*uninit.as_mut_ptr()).field) };
unsafe { f1_ptr.write(true); }

let init = unsafe { uninit.assume_init() };
```

One last remark: when reading old Rust code, you might stumble upon the
deprecated `mem::uninitialized` function.  That function used to be the only way
to deal with uninitialized memory on the stack, but it turned out to be
impossible to properly integrate with the rest of the language.  Always use
`MaybeUninit` instead in new code, and port old code over when you get the
opportunity.

And that's about it for working with uninitialized memory! Basically nothing
anywhere expects to be handed uninitialized memory, so if you're going to pass
it around at all, be sure to be *really* careful.

[`MaybeUninit`]: ../core/mem/union.MaybeUninit.html
[assume_init]: ../core/mem/union.MaybeUninit.html#method.assume_init
[`ptr`]: ../core/ptr/index.html
[`addr_of_mut`]: ../core/ptr/macro.addr_of_mut.html
[`write`]: ../core/ptr/fn.write.html
[`copy`]: ../std/ptr/fn.copy.html
[`copy_nonoverlapping`]: ../std/ptr/fn.copy_nonoverlapping.html
