# Drop Check

We have seen how lifetimes provide us some fairly simple rules for ensuring
that we never read dangling references. However up to this point we have only ever
interacted with the _outlives_ relationship in an inclusive manner. That is,
when we talked about `'a: 'b`, it was ok for `'a` to live _exactly_ as long as
`'b`. At first glance, this seems to be a meaningless distinction. Nothing ever
gets dropped at the same time as another, right? This is why we used the
following desugaring of `let` statements:

<!-- ignore: simplified code -->
```rust,ignore
let x;
let y;
```

desugaring to:

<!-- ignore: desugared code -->
```rust,ignore
{
    let x;
    {
        let y;
    }
}
```

There are some more complex situations which are not possible to desugar using
scopes, but the order is still defined ‒ variables are dropped in the reverse
order of their definition, fields of structs and tuples in order of their
definition. There are some more details about order of drop in [RFC 1857][rfc1857].

Let's do this:

<!-- ignore: simplified code -->
```rust,ignore
let tuple = (vec![], vec![]);
```

The left vector is dropped first. But does it mean the right one strictly
outlives it in the eyes of the borrow checker? The answer to this question is
_no_. The borrow checker could track fields of tuples separately, but it would
still be unable to decide what outlives what in case of vector elements, which
are dropped manually via pure-library code the borrow checker doesn't
understand.

So why do we care? We care because if the type system isn't careful, it could
accidentally make dangling pointers. Consider the following simple program:

```rust
struct Inspector<'a>(&'a u8);

struct World<'a> {
    inspector: Option<Inspector<'a>>,
    days: Box<u8>,
}

fn main() {
    let mut world = World {
        inspector: None,
        days: Box::new(1),
    };
    world.inspector = Some(Inspector(&world.days));
}
```

This program is totally sound and compiles today. The fact that `days` does not
strictly outlive `inspector` doesn't matter. As long as the `inspector` is
alive, so is `days`.

However if we add a destructor, the program will no longer compile!

```rust,compile_fail
struct Inspector<'a>(&'a u8);

impl<'a> Drop for Inspector<'a> {
    fn drop(&mut self) {
        println!("I was only {} days from retirement!", self.0);
    }
}

struct World<'a> {
    inspector: Option<Inspector<'a>>,
    days: Box<u8>,
}

fn main() {
    let mut world = World {
        inspector: None,
        days: Box::new(1),
    };
    world.inspector = Some(Inspector(&world.days));
    // Let's say `days` happens to get dropped first.
    // Then when Inspector is dropped, it will try to read free'd memory!
}
```

```text
error[E0597]: `world.days` does not live long enough
  --> src/main.rs:19:38
   |
19 |     world.inspector = Some(Inspector(&world.days));
   |                                      ^^^^^^^^^^^ borrowed value does not live long enough
...
22 | }
   | -
   | |
   | `world.days` dropped here while still borrowed
   | borrow might be used here, when `world` is dropped and runs the destructor for type `World<'_>`
```

You can try changing the order of fields or use a tuple instead of the struct,
it'll still not compile.

Implementing `Drop` lets the `Inspector` execute some arbitrary code during its
death. This means it can potentially observe that types that are supposed to
live as long as it does actually were destroyed first.

Interestingly, only generic types need to worry about this. If they aren't
generic, then the only lifetimes they can harbor are `'static`, which will truly
live _forever_. This is why this problem is referred to as _sound generic drop_.
Sound generic drop is enforced by the _drop checker_. As of this writing, some
of the finer details of how the drop checker (also called dropck) validates
types is totally up in the air. However The Big Rule is the subtlety that we
have focused on this whole section:

**For a generic type to soundly implement drop, its generics arguments must
strictly outlive it.**

Obeying this rule is (usually) necessary to satisfy the borrow
checker; obeying it is sufficient but not necessary to be
sound. That is, if your type obeys this rule then it's definitely
sound to drop.

The reason that it is not always necessary to satisfy the above rule
is that some Drop implementations will not access borrowed data even
though their type gives them the capability for such access, or because we know
the specific drop order and the borrowed data is still fine even if the borrow
checker doesn't know that.

For example, this variant of the above `Inspector` example will never
access borrowed data:

```rust,compile_fail
struct Inspector<'a>(&'a u8, &'static str);

impl<'a> Drop for Inspector<'a> {
    fn drop(&mut self) {
        println!("Inspector(_, {}) knows when *not* to inspect.", self.1);
    }
}

struct World<'a> {
    inspector: Option<Inspector<'a>>,
    days: Box<u8>,
}

fn main() {
    let mut world = World {
        inspector: None,
        days: Box::new(1),
    };
    world.inspector = Some(Inspector(&world.days, "gadget"));
    // Let's say `days` happens to get dropped first.
    // Even when Inspector is dropped, its destructor will not access the
    // borrowed `days`.
}
```

Likewise, this variant will also never access borrowed data:

```rust,compile_fail
struct Inspector<T>(T, &'static str);

impl<T> Drop for Inspector<T> {
    fn drop(&mut self) {
        println!("Inspector(_, {}) knows when *not* to inspect.", self.1);
    }
}

struct World<T> {
    inspector: Option<Inspector<T>>,
    days: Box<u8>,
}

fn main() {
    let mut world = World {
        inspector: None,
        days: Box::new(1),
    };
    world.inspector = Some(Inspector(&world.days, "gadget"));
    // Let's say `days` happens to get dropped first.
    // Even when Inspector is dropped, its destructor will not access the
    // borrowed `days`.
}
```

However, _both_ of the above variants are rejected by the borrow
checker during the analysis of `fn main`, saying that `days` does not
live long enough.

The reason is that the borrow checking analysis of `main` does not
know about the internals of each `Inspector`'s `Drop` implementation. As
far as the borrow checker knows while it is analyzing `main`, the body
of an inspector's destructor might access that borrowed data.

Therefore, the drop checker forces all borrowed data in a value to
strictly outlive that value.

## An Escape Hatch

The precise rules that govern drop checking may be less restrictive in
the future.

The current analysis is deliberately conservative and trivial; it forces all
borrowed data in a value to outlive that value, which is certainly sound.

Future versions of the language may make the analysis more precise, to
reduce the number of cases where sound code is rejected as unsafe.
This would help address cases such as the two `Inspector`s above that
know not to inspect during destruction.

In the meantime, there is an unstable attribute that one can use to
assert (unsafely) that a generic type's destructor is _guaranteed_ to
not access any expired data, even if its type gives it the capability
to do so.

That attribute is called `may_dangle` and was introduced in [RFC 1327][rfc1327].
To deploy it on the `Inspector` from above, we would write:

```rust
#![feature(dropck_eyepatch)]

struct Inspector<'a>(&'a u8, &'static str);

unsafe impl<#[may_dangle] 'a> Drop for Inspector<'a> {
    fn drop(&mut self) {
        println!("Inspector(_, {}) knows when *not* to inspect.", self.1);
    }
}

struct World<'a> {
    days: Box<u8>,
    inspector: Option<Inspector<'a>>,
}

fn main() {
    let mut world = World {
        inspector: None,
        days: Box::new(1),
    };
    world.inspector = Some(Inspector(&world.days, "gadget"));
}
```

Use of this attribute requires the `Drop` impl to be marked `unsafe` because the
compiler is not checking the implicit assertion that no potentially expired data
(e.g. `self.0` above) is accessed.

The attribute can be applied to any number of lifetime and type parameters. In
the following example, we assert that we access no data behind a reference of
lifetime `'b` and that the only uses of `T` will be moves or drops, but omit
the attribute from `'a` and `U`, because we do access data with that lifetime
and that type:

```rust
#![feature(dropck_eyepatch)]
use std::fmt::Display;

struct Inspector<'a, 'b, T, U: Display>(&'a u8, &'b u8, T, U);

unsafe impl<'a, #[may_dangle] 'b, #[may_dangle] T, U: Display> Drop for Inspector<'a, 'b, T, U> {
    fn drop(&mut self) {
        println!("Inspector({}, _, _, {})", self.0, self.3);
    }
}
```

It is sometimes obvious that no such access can occur, like the case above.
However, when dealing with a generic type parameter, such access can
occur indirectly. Examples of such indirect access are:

- invoking a callback,
- via a trait method call.

(Future changes to the language, such as impl specialization, may add
other avenues for such indirect access.)

Here is an example of invoking a callback:

```rust
struct Inspector<T>(T, &'static str, Box<for <'r> fn(&'r T) -> String>);

impl<T> Drop for Inspector<T> {
    fn drop(&mut self) {
        // The `self.2` call could access a borrow e.g. if `T` is `&'a _`.
        println!("Inspector({}, {}) unwittingly inspects expired data.",
                 (self.2)(&self.0), self.1);
    }
}
```

Here is an example of a trait method call:

```rust
use std::fmt;

struct Inspector<T: fmt::Display>(T, &'static str);

impl<T: fmt::Display> Drop for Inspector<T> {
    fn drop(&mut self) {
        // There is a hidden call to `<T as Display>::fmt` below, which
        // could access a borrow e.g. if `T` is `&'a _`
        println!("Inspector({}, {}) unwittingly inspects expired data.",
                 self.0, self.1);
    }
}
```

And of course, all of these accesses could be further hidden within
some other method invoked by the destructor, rather than being written
directly within it.

In all of the above cases where the `&'a u8` is accessed in the
destructor, adding the `#[may_dangle]`
attribute makes the type vulnerable to misuse that the borrow
checker will not catch, inviting havoc. It is better to avoid adding
the attribute.

## A related side note about drop order

While the drop order of fields inside a struct is defined, relying on it is
fragile and subtle. When the order matters, it is better to use the
[`ManuallyDrop`] wrapper.

## Is that all about drop checker?

It turns out that when writing unsafe code, we generally don't need to
worry at all about doing the right thing for the drop checker. However there
is one special case that you need to worry about, which we will look at in
the next section.

[rfc1327]: https://github.com/rust-lang/rfcs/blob/master/text/1327-dropck-param-eyepatch.md
[rfc1857]: https://github.com/rust-lang/rfcs/blob/master/text/1857-stabilize-drop-order.md
[`manuallydrop`]: ../std/mem/struct.ManuallyDrop.html
