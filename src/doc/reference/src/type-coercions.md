# Type coercions

**Type coercions** are implicit operations that change the type of a value.
They happen automatically at specific locations and are highly restricted in
what types actually coerce.

Any conversions allowed by coercion can also be explicitly performed by the
[type cast operator], `as`.

Coercions are originally defined in [RFC 401] and expanded upon in [RFC 1558].

## Coercion sites

A coercion can only occur at certain coercion sites in a program; these are
typically places where the desired type is explicit or can be derived by
propagation from explicit types (without type inference). Possible coercion
sites are:

* `let` statements where an explicit type is given.

   For example, `&mut 42` is coerced to have type `&i8` in the following:

   ```rust
   let _: &i8 = &mut 42;
   ```

* `static` and `const` item declarations (similar to `let` statements).

* Arguments for function calls

  The value being coerced is the actual parameter, and it is coerced to
  the type of the formal parameter.

  For example, `&mut 42` is coerced to have type `&i8` in the following:

  ```rust
  fn bar(_: &i8) { }

  fn main() {
      bar(&mut 42);
  }
  ```

  For method calls, the receiver (`self` parameter) can only take advantage
  of [unsized coercions](#unsized-coercions).

* Instantiations of struct, union, or enum variant fields

  For example, `&mut 42` is coerced to have type `&i8` in the following:

  ```rust
  struct Foo<'a> { x: &'a i8 }

  fn main() {
      Foo { x: &mut 42 };
  }
  ```

* Function results&mdash;either the final line of a block if it is not
  semicolon-terminated or any expression in a `return` statement

  For example, `x` is coerced to have type `&dyn Display` in the following:

  ```rust
  use std::fmt::Display;
  fn foo(x: &u32) -> &dyn Display {
      x
  }
  ```

If the expression in one of these coercion sites is a coercion-propagating
expression, then the relevant sub-expressions in that expression are also
coercion sites. Propagation recurses from these new coercion sites.
Propagating expressions and their relevant sub-expressions are:

* Array literals, where the array has type `[U; n]`. Each sub-expression in
the array literal is a coercion site for coercion to type `U`.

* Array literals with repeating syntax, where the array has type `[U; n]`. The
repeated sub-expression is a coercion site for coercion to type `U`.

* Tuples, where a tuple is a coercion site to type `(U_0, U_1, ..., U_n)`.
Each sub-expression is a coercion site to the respective type, e.g. the
zeroth sub-expression is a coercion site to type `U_0`.

* Parenthesized sub-expressions (`(e)`): if the expression has type `U`, then
the sub-expression is a coercion site to `U`.

* Blocks: if a block has type `U`, then the last expression in the block (if
it is not semicolon-terminated) is a coercion site to `U`. This includes
blocks which are part of control flow statements, such as `if`/`else`, if
the block has a known type.

## Coercion types

Coercion is allowed between the following types:

* `T` to `U` if `T` is a [subtype] of `U` (*reflexive case*)

* `T_1` to `T_3` where `T_1` coerces to `T_2` and `T_2` coerces to `T_3`
(*transitive case*)

    Note that this is not fully supported yet.

* `&mut T` to `&T`

* `*mut T` to `*const T`

* `&T` to `*const T`

* `&mut T` to `*mut T`

* `&T` or `&mut T` to `&U` if `T` implements `Deref<Target = U>`. For example:

  ```rust
  use std::ops::Deref;

  struct CharContainer {
      value: char,
  }

  impl Deref for CharContainer {
      type Target = char;

      fn deref<'a>(&'a self) -> &'a char {
          &self.value
      }
  }

  fn foo(arg: &char) {}

  fn main() {
      let x = &mut CharContainer { value: 'y' };
      foo(x); //&mut CharContainer is coerced to &char.
  }
  ```

* `&mut T` to `&mut U` if `T` implements `DerefMut<Target = U>`.

* TyCtor(`T`) to TyCtor(`U`), where TyCtor(`T`) is one of
    - `&T`
    - `&mut T`
    - `*const T`
    - `*mut T`
    - `Box<T>`

    and where `U` can be obtained from `T` by [unsized coercion](#unsized-coercions).

    <!--In the future, coerce_inner will be recursively extended to tuples and
    structs. In addition, coercions from subtraits to supertraits will be
    added. See [RFC 401] for more details.-->

* Function item types to `fn` pointers

* Non capturing closures to `fn` pointers

* `!` to any `T`

### Unsized Coercions

The following coercions are called `unsized coercions`, since they
relate to converting sized types to unsized types, and are permitted in a few
cases where other coercions are not, as described above. They can still happen
anywhere else a coercion can occur.

Two traits, [`Unsize`] and [`CoerceUnsized`], are used
to assist in this process and expose it for library use. The following
coercions are built-ins and, if `T` can be coerced to `U` with one of them, then
an implementation of `Unsize<U>` for `T` will be provided:

* `[T; n]` to `[T]`.

* `T` to `dyn U`, when `T` implements `U + Sized`, and `U` is [object safe].

* `Foo<..., T, ...>` to `Foo<..., U, ...>`, when:
    * `Foo` is a struct.
    * `T` implements `Unsize<U>`.
    * The last field of `Foo` has a type involving `T`.
    * If that field has type `Bar<T>`, then `Bar<T>` implements `Unsized<Bar<U>>`.
    * T is not part of the type of any other fields.

Additionally, a type `Foo<T>` can implement `CoerceUnsized<Foo<U>>` when `T`
implements `Unsize<U>` or `CoerceUnsized<Foo<U>>`. This allows it to provide a
unsized coercion to `Foo<U>`.

> Note: While the definition of the unsized coercions and their implementation
> has been stabilized, the traits themselves are not yet stable and therefore
> can't be used directly in stable Rust.

## Least upper bound coercions

In some contexts, the compiler must coerce together multiple types to try and
find the most general type. This is called a "Least Upper Bound" coercion.
LUB coercion is used and only used in the following situations:

+ To find the common type for a series of if branches.
+ To find the common type for a series of match arms.
+ To find the common type for array elements.
+ To find the type for the return type of a closure with multiple return statements.
+ To check the type for the return type of a function with multiple return statements.

In each such case, there are a set of types `T0..Tn` to be mutually coerced
to some target type `T_t`, which is unknown to start. Computing the LUB
coercion is done iteratively. The target type `T_t` begins as the type `T0`.
For each new type `Ti`, we consider whether

+ If `Ti` can be coerced to the current target type `T_t`, then no change is made.
+ Otherwise, check whether `T_t` can be coerced to `Ti`; if so, the `T_t` is
changed to `Ti`. (This check is also conditioned on whether all of the source
expressions considered thus far have implicit coercions.)
+ If not, try to compute a mutual supertype of `T_t` and `Ti`, which will become the new target type.

### Examples:

```rust
# let (a, b, c) = (0, 1, 2);
// For if branches
let bar = if true {
    a
} else if false {
    b
} else {
    c
};

// For match arms
let baw = match 42 {
    0 => a,
    1 => b,
    _ => c,
};

// For array elements
let bax = [a, b, c];

// For closure with multiple return statements
let clo = || {
    if true {
        a
    } else if false {
        b
    } else {
        c
    }
};
let baz = clo();

// For type checking of function with multiple return statements
fn foo() -> i32 {
    let (a, b, c) = (0, 1, 2);
    match 42 {
        0 => a,
        1 => b,
        _ => c,
    }
}
```

In these examples, types of the `ba*` are found by LUB coercion. And the
compiler checks whether LUB coercion result of `a`, `b`, `c` is `i32` in the
processing of the function `foo`.

### Caveat

This description is obviously informal. Making it more precise is expected to
proceed as part of a general effort to specify the Rust type checker more
precisely.

[RFC 401]: https://github.com/rust-lang/rfcs/blob/master/text/0401-coercions.md
[RFC 1558]: https://github.com/rust-lang/rfcs/blob/master/text/1558-closure-to-fn-coercion.md
[subtype]: subtyping.md
[object safe]: items/traits.md#object-safety
[type cast operator]: expressions/operator-expr.md#type-cast-expressions
[`Unsize`]: ../std/marker/trait.Unsize.html
[`CoerceUnsized`]: ../std/ops/trait.CoerceUnsized.html
