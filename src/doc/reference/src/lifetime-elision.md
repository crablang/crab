# Lifetime elision

Rust has rules that allow lifetimes to be elided in various places where the
compiler can infer a sensible default choice.

## Lifetime elision in functions

In order to make common patterns more ergonomic, lifetime arguments can be
*elided* in [function item], [function pointer], and [closure trait] signatures.
The following rules are used to infer lifetime parameters for elided lifetimes.
It is an error to elide lifetime parameters that cannot be inferred. The
placeholder lifetime, `'_`, can also be used to have a lifetime inferred in the
same way. For lifetimes in paths, using `'_` is preferred. Trait object
lifetimes follow different rules discussed
[below](#default-trait-object-lifetimes).

* Each elided lifetime in the parameters becomes a distinct lifetime parameter.
* If there is exactly one lifetime used in the parameters (elided or not), that
  lifetime is assigned to *all* elided output lifetimes.

In method signatures there is another rule

* If the receiver has type `&Self`  or `&mut Self`, then the lifetime of that
  reference to `Self` is assigned to all elided output lifetime parameters.

Examples:

```rust
# trait T {}
# trait ToCStr {}
# struct Thing<'a> {f: &'a i32}
# struct Command;
#
# trait Example {
fn print1(s: &str);                                   // elided
fn print2(s: &'_ str);                                // also elided
fn print3<'a>(s: &'a str);                            // expanded

fn debug1(lvl: usize, s: &str);                       // elided
fn debug2<'a>(lvl: usize, s: &'a str);                // expanded

fn substr1(s: &str, until: usize) -> &str;            // elided
fn substr2<'a>(s: &'a str, until: usize) -> &'a str;  // expanded

fn get_mut1(&mut self) -> &mut dyn T;                 // elided
fn get_mut2<'a>(&'a mut self) -> &'a mut dyn T;       // expanded

fn args1<T: ToCStr>(&mut self, args: &[T]) -> &mut Command;                  // elided
fn args2<'a, 'b, T: ToCStr>(&'a mut self, args: &'b [T]) -> &'a mut Command; // expanded

fn new1(buf: &mut [u8]) -> Thing<'_>;                 // elided - preferred
fn new2(buf: &mut [u8]) -> Thing;                     // elided
fn new3<'a>(buf: &'a mut [u8]) -> Thing<'a>;          // expanded
# }

type FunPtr1 = fn(&str) -> &str;                      // elided
type FunPtr2 = for<'a> fn(&'a str) -> &'a str;        // expanded

type FunTrait1 = dyn Fn(&str) -> &str;                // elided
type FunTrait2 = dyn for<'a> Fn(&'a str) -> &'a str;  // expanded
```

```rust,compile_fail
// The following examples show situations where it is not allowed to elide the
// lifetime parameter.

# trait Example {
// Cannot infer, because there are no parameters to infer from.
fn get_str() -> &str;                                 // ILLEGAL

// Cannot infer, ambiguous if it is borrowed from the first or second parameter.
fn frob(s: &str, t: &str) -> &str;                    // ILLEGAL
# }
```

## Default trait object lifetimes

The assumed lifetime of references held by a [trait object] is called its
_default object lifetime bound_. These were defined in [RFC 599] and amended in
[RFC 1156].

These default object lifetime bounds are used instead of the lifetime parameter
elision rules defined above when the lifetime bound is omitted entirely. If
`'_` is used as the lifetime bound then the bound follows the usual elision
rules.

If the trait object is used as a type argument of a generic type then the
containing type is first used to try to infer a bound.

* If there is a unique bound from the containing type then that is the default
* If there is more than one bound from the containing type then an explicit
  bound must be specified

If neither of those rules apply, then the bounds on the trait are used:

* If the trait is defined with a single lifetime _bound_ then that bound is
  used.
* If `'static` is used for any lifetime bound then `'static` is used.
* If the trait has no lifetime bounds, then the lifetime is inferred in
  expressions and is `'static` outside of expressions.

```rust
// For the following trait...
trait Foo { }

// These two are the same because Box<T> has no lifetime bound on T
type T1 = Box<dyn Foo>;
type T2 = Box<dyn Foo + 'static>;

// ...and so are these:
impl dyn Foo {}
impl dyn Foo + 'static {}

// ...so are these, because &'a T requires T: 'a
type T3<'a> = &'a dyn Foo;
type T4<'a> = &'a (dyn Foo + 'a);

// std::cell::Ref<'a, T> also requires T: 'a, so these are the same
type T5<'a> = std::cell::Ref<'a, dyn Foo>;
type T6<'a> = std::cell::Ref<'a, dyn Foo + 'a>;
```

```rust,compile_fail
// This is an example of an error.
# trait Foo { }
struct TwoBounds<'a, 'b, T: ?Sized + 'a + 'b> {
    f1: &'a i32,
    f2: &'b i32,
    f3: T,
}
type T7<'a, 'b> = TwoBounds<'a, 'b, dyn Foo>;
//                                  ^^^^^^^
// Error: the lifetime bound for this object type cannot be deduced from context
```

Note that the innermost object sets the bound, so `&'a Box<dyn Foo>` is still
`&'a Box<dyn Foo + 'static>`.

```rust
// For the following trait...
trait Bar<'a>: 'a { }

// ...these two are the same:
type T1<'a> = Box<dyn Bar<'a>>;
type T2<'a> = Box<dyn Bar<'a> + 'a>;

// ...and so are these:
impl<'a> dyn Bar<'a> {}
impl<'a> dyn Bar<'a> + 'a {}
```

## `'static` lifetime elision

Both [constant] and [static] declarations of reference types have *implicit*
`'static` lifetimes unless an explicit lifetime is specified. As such, the
constant declarations involving `'static` above may be written without the
lifetimes.

```rust
// STRING: &'static str
const STRING: &str = "bitstring";

struct BitsNStrings<'a> {
    mybits: [u32; 2],
    mystring: &'a str,
}

// BITS_N_STRINGS: BitsNStrings<'static>
const BITS_N_STRINGS: BitsNStrings<'_> = BitsNStrings {
    mybits: [1, 2],
    mystring: STRING,
};
```

Note that if the `static` or `const` items include function or closure
references, which themselves include references, the compiler will first try
the standard elision rules. If it is unable to resolve the lifetimes by its
usual rules, then it will error. By way of example:

```rust
# struct Foo;
# struct Bar;
# struct Baz;
# fn somefunc(a: &Foo, b: &Bar, c: &Baz) -> usize {42}
// Resolved as `fn<'a>(&'a str) -> &'a str`.
const RESOLVED_SINGLE: fn(&str) -> &str = |x| x;

// Resolved as `Fn<'a, 'b, 'c>(&'a Foo, &'b Bar, &'c Baz) -> usize`.
const RESOLVED_MULTIPLE: &dyn Fn(&Foo, &Bar, &Baz) -> usize = &somefunc;
```

```rust,compile_fail
# struct Foo;
# struct Bar;
# struct Baz;
# fn somefunc<'a,'b>(a: &'a Foo, b: &'b Bar) -> &'a Baz {unimplemented!()}
// There is insufficient information to bound the return reference lifetime
// relative to the argument lifetimes, so this is an error.
const RESOLVED_STATIC: &dyn Fn(&Foo, &Bar) -> &Baz = &somefunc;
//                                            ^
// this function's return type contains a borrowed value, but the signature
// does not say whether it is borrowed from argument 1 or argument 2
```

[closure trait]: types/closure.md
[constant]: items/constant-items.md
[function item]: types/function-item.md
[function pointer]: types/function-pointer.md
[RFC 599]: https://github.com/rust-lang/rfcs/blob/master/text/0599-default-object-bound.md
[RFC 1156]: https://github.com/rust-lang/rfcs/blob/master/text/1156-adjust-default-object-bounds.md
[static]: items/static-items.md
[trait object]: types/trait-object.md
