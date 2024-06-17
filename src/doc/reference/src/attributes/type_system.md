# Type system attributes

The following [attributes] are used for changing how a type can be used.

## The `non_exhaustive` attribute

The *`non_exhaustive` attribute* indicates that a type or variant may have
more fields or variants added in the future. It can be applied to
[`struct`s][struct], [`enum`s][enum], and `enum` variants.

The `non_exhaustive` attribute uses the [_MetaWord_] syntax and thus does not
take any inputs.

Within the defining crate, `non_exhaustive` has no effect.

```rust
#[non_exhaustive]
pub struct Config {
    pub window_width: u16,
    pub window_height: u16,
}

#[non_exhaustive]
pub struct Token;

#[non_exhaustive]
pub struct Id(pub u64);

#[non_exhaustive]
pub enum Error {
    Message(String),
    Other,
}

pub enum Message {
    #[non_exhaustive] Send { from: u32, to: u32, contents: String },
    #[non_exhaustive] Reaction(u32),
    #[non_exhaustive] Quit,
}

// Non-exhaustive structs can be constructed as normal within the defining crate.
let config = Config { window_width: 640, window_height: 480 };
let token = Token;
let id = Id(4);

// Non-exhaustive structs can be matched on exhaustively within the defining crate.
let Config { window_width, window_height } = config;
let Token = token;
let Id(id_number) = id;

let error = Error::Other;
let message = Message::Reaction(3);

// Non-exhaustive enums can be matched on exhaustively within the defining crate.
match error {
    Error::Message(ref s) => { },
    Error::Other => { },
}

match message {
    // Non-exhaustive variants can be matched on exhaustively within the defining crate.
    Message::Send { from, to, contents } => { },
    Message::Reaction(id) => { },
    Message::Quit => { },
}
```

Outside of the defining crate, types annotated with `non_exhaustive` have limitations that
preserve backwards compatibility when new fields or variants are added.

Non-exhaustive types cannot be constructed outside of the defining crate:

- Non-exhaustive variants ([`struct`][struct] or [`enum` variant][enum]) cannot be constructed
  with a [_StructExpression_] \(including with [functional update syntax]).
- The implicitly defined same-named constant of a [unit-like struct][struct],
  or the same-named constructor function of a [tuple struct][struct],
  has a [visibility] no greater than `pub(crate)`.
  That is, if the struct’s visibility is `pub`, then the constant or constructor’s visibility
  is `pub(crate)`, and otherwise the visibility of the two items is the same
  (as is the case without `#[non_exhaustive]`).
- [`enum`][enum] instances can be constructed.

The following examples of construction do not compile when outside the defining crate:

<!-- ignore: requires external crates -->
```rust,ignore
// These are types defined in an upstream crate that have been annotated as
// `#[non_exhaustive]`.
use upstream::{Config, Token, Id, Error, Message};

// Cannot construct an instance of `Config`; if new fields were added in
// a new version of `upstream` then this would fail to compile, so it is
// disallowed.
let config = Config { window_width: 640, window_height: 480 };

// Cannot construct an instance of `Token`; if new fields were added, then
// it would not be a unit-like struct any more, so the same-named constant
// created by it being a unit-like struct is not public outside the crate;
// this code fails to compile.
let token = Token;

// Cannot construct an instance of `Id`; if new fields were added, then
// its constructor function signature would change, so its constructor
// function is not public outside the crate; this code fails to compile.
let id = Id(5);

// Can construct an instance of `Error`; new variants being introduced would
// not result in this failing to compile.
let error = Error::Message("foo".to_string());

// Cannot construct an instance of `Message::Send` or `Message::Reaction`;
// if new fields were added in a new version of `upstream` then this would
// fail to compile, so it is disallowed.
let message = Message::Send { from: 0, to: 1, contents: "foo".to_string(), };
let message = Message::Reaction(0);

// Cannot construct an instance of `Message::Quit`; if this were converted to
// a tuple-variant `upstream` then this would fail to compile.
let message = Message::Quit;
```

There are limitations when matching on non-exhaustive types outside of the defining crate:

- When pattern matching on a non-exhaustive variant ([`struct`][struct] or [`enum` variant][enum]),
  a [_StructPattern_] must be used which must include a `..`. A tuple variant's constructor's
  [visibility] is reduced to be no greater than `pub(crate)`.
- When pattern matching on a non-exhaustive [`enum`][enum], matching on a variant does not
  contribute towards the exhaustiveness of the arms.

The following examples of matching do not compile when outside the defining crate:

<!-- ignore: requires external crates -->
```rust, ignore
// These are types defined in an upstream crate that have been annotated as
// `#[non_exhaustive]`.
use upstream::{Config, Token, Id, Error, Message};

// Cannot match on a non-exhaustive enum without including a wildcard arm.
match error {
  Error::Message(ref s) => { },
  Error::Other => { },
  // would compile with: `_ => {},`
}

// Cannot match on a non-exhaustive struct without a wildcard.
if let Ok(Config { window_width, window_height }) = config {
    // would compile with: `..`
}

// Cannot match a non-exhaustive unit-like or tuple struct except by using
// braced struct syntax with a wildcard.
// This would compile as `let Token { .. } = token;`
let Token = token;
// This would compile as `let Id { 0: id_number, .. } = id;`
let Id(id_number) = id;

match message {
  // Cannot match on a non-exhaustive struct enum variant without including a wildcard.
  Message::Send { from, to, contents } => { },
  // Cannot match on a non-exhaustive tuple or unit enum variant.
  Message::Reaction(type) => { },
  Message::Quit => { },
}
```

It's also not allowed to cast non-exhaustive types from foreign crates.
```rust, ignore
use othercrate::NonExhaustiveEnum;

// Cannot cast a non-exhaustive enum outside of its defining crate.
let _ = NonExhaustiveEnum::default() as u8;
```

Non-exhaustive types are always considered inhabited in downstream crates.

[_MetaWord_]: ../attributes.md#meta-item-attribute-syntax
[_StructExpression_]: ../expressions/struct-expr.md
[_StructPattern_]: ../patterns.md#struct-patterns
[_TupleStructPattern_]: ../patterns.md#tuple-struct-patterns
[`if let`]: ../expressions/if-expr.md#if-let-expressions
[`match`]: ../expressions/match-expr.md
[attributes]: ../attributes.md
[enum]: ../items/enumerations.md
[functional update syntax]: ../expressions/struct-expr.md#functional-update-syntax
[struct]: ../items/structs.md
[visibility]: ../visibility-and-privacy.md
