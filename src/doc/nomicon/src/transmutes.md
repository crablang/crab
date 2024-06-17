# Transmutes

Get out of our way type system! We're going to reinterpret these bits or die
trying! Even though this book is all about doing things that are unsafe, I
really can't emphasize enough that you should deeply think about finding Another Way
than the operations covered in this section. This is really, truly, the most
horribly unsafe thing you can do in Rust. The guardrails here are dental floss.

[`mem::transmute<T, U>`][transmute] takes a value of type `T` and reinterprets
it to have type `U`. The only restriction is that the `T` and `U` are verified
to have the same size. The ways to cause Undefined Behavior with this are mind
boggling.

* First and foremost, creating an instance of *any* type with an invalid state
  is going to cause arbitrary chaos that can't really be predicted. Do not
  transmute `3` to `bool`. Even if you never *do* anything with the `bool`. Just
  don't.

* Transmute has an overloaded return type. If you do not specify the return type
  it may produce a surprising type to satisfy inference.

* Transmuting an `&` to `&mut` is Undefined Behavior. While certain usages may
  *appear* safe, note that the Rust optimizer is free to assume that a shared
  reference won't change through its lifetime and thus such transmutation will
  run afoul of those assumptions. So:
  * Transmuting an `&` to `&mut` is *always* Undefined Behavior.
  * No you can't do it.
  * No you're not special.

* Transmuting to a reference without an explicitly provided lifetime
  produces an [unbounded lifetime].

* When transmuting between different compound types, you have to make sure they
  are laid out the same way! If layouts differ, the wrong fields are going to
  get filled with the wrong data, which will make you unhappy and can also be
  Undefined Behavior (see above).

  So how do you know if the layouts are the same? For `repr(C)` types and
  `repr(transparent)` types, layout is precisely defined. But for your
  run-of-the-mill `repr(Rust)`, it is not. Even different instances of the same
  generic type can have wildly different layout. `Vec<i32>` and `Vec<u32>`
  *might* have their fields in the same order, or they might not. The details of
  what exactly is and is not guaranteed for data layout are still being worked
  out over [at the UCG WG][ucg-layout].

[`mem::transmute_copy<T, U>`][transmute_copy] somehow manages to be *even more*
wildly unsafe than this. It copies `size_of<U>` bytes out of an `&T` and
interprets them as a `U`.  The size check that `mem::transmute` has is gone (as
it may be valid to copy out a prefix), though it is Undefined Behavior for `U`
to be larger than `T`.

Also of course you can get all of the functionality of these functions using raw
pointer casts or `union`s, but without any of the lints or other basic sanity
checks. Raw pointer casts and `union`s do not magically avoid the above rules.

[unbounded lifetime]: ./unbounded-lifetimes.md
[transmute]: ../std/mem/fn.transmute.html
[transmute_copy]: ../std/mem/fn.transmute_copy.html
[ucg-layout]: https://rust-lang.github.io/unsafe-code-guidelines/layout.html
