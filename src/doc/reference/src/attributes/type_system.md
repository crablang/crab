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

// Non-exhaustive structs can be matched on exhaustively within the defining crate.
if let Config { window_width, window_height } = config {
    // ...
}

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
- [`enum`][enum] instances can be constructed.

<!-- ignore: requires external crates -->
```rust,ignore
// `Config`, `Error`, and `Message` are types defined in an upstream crate that have been
// annotated as `#[non_exhaustive]`.
use upstream::{Config, Error, Message};

// Cannot construct an instance of `Config`, if new fields were added in
// a new version of `upstream` then this would fail to compile, so it is
// disallowed.
let config = Config { window_width: 640, window_height: 480 };

// Can construct an instance of `Error`, new variants being introduced would
// not result in this failing to compile.
let error = Error::Message("foo".to_string());

// Cannot construct an instance of `Message::Send` or `Message::Reaction`,
// if new fields were added in a new version of `upstream` then this would
// fail to compile, so it is disallowed.
let message = Message::Send { from: 0, to: 1, contents: "foo".to_string(), };
let message = Message::Reaction(0);

// Cannot construct an instance of `Message::Quit`, if this were converted to
// a tuple-variant `upstream` then this would fail to compile.
let message = Message::Quit;
```

There are limitations when matching on non-exhaustive types outside of the defining crate:

- When pattern matching on a non-exhaustive variant ([`struct`][struct] or [`enum` variant][enum]),
  a [_StructPattern_] must be used which must include a `..`. Tuple variant constructor visibility
  is lowered to `min($vis, pub(crate))`.
- When pattern matching on a non-exhaustive [`enum`][enum], matching on a variant does not
  contribute towards the exhaustiveness of the arms.

<!-- ignore: requires external crates -->
```rust, ignore
// `Config`, `Error`, and `Message` are types defined in an upstream crate that have been
// annotated as `#[non_exhaustive]`.
use upstream::{Config, Error, Message};

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
