# Diagnostic attributes

The following [attributes] are used for controlling or generating diagnostic
messages during compilation.

## Lint check attributes

A lint check names a potentially undesirable coding pattern, such as
unreachable code or omitted documentation. The lint attributes `allow`,
`expect`, `warn`, `deny`, and `forbid` use the [_MetaListPaths_] syntax
to specify a list of lint names to change the lint level for the entity
to which the attribute applies.

For any lint check `C`:

* `#[allow(C)]` overrides the check for `C` so that violations will go
   unreported.
* `#[expect(C)]` indicates that lint `C` is expected to be emitted. The
  attribute will suppress the emission of `C` or issue a warning, if the
  expectation is unfulfilled.
* `#[warn(C)]` warns about violations of `C` but continues compilation.
* `#[deny(C)]` signals an error after encountering a violation of `C`,
* `#[forbid(C)]` is the same as `deny(C)`, but also forbids changing the lint
   level afterwards,

> Note: The lint checks supported by `rustc` can be found via `rustc -W help`,
> along with their default settings and are documented in the [rustc book].

```rust
pub mod m1 {
    // Missing documentation is ignored here
    #[allow(missing_docs)]
    pub fn undocumented_one() -> i32 { 1 }

    // Missing documentation signals a warning here
    #[warn(missing_docs)]
    pub fn undocumented_too() -> i32 { 2 }

    // Missing documentation signals an error here
    #[deny(missing_docs)]
    pub fn undocumented_end() -> i32 { 3 }
}
```

Lint attributes can override the level specified from a previous attribute, as
long as the level does not attempt to change a forbidden lint. Previous
attributes are those from a higher level in the syntax tree, or from a
previous attribute on the same entity as listed in left-to-right source order.

This example shows how one can use `allow` and `warn` to toggle a particular
check on and off:

```rust
#[warn(missing_docs)]
pub mod m2 {
    #[allow(missing_docs)]
    pub mod nested {
        // Missing documentation is ignored here
        pub fn undocumented_one() -> i32 { 1 }

        // Missing documentation signals a warning here,
        // despite the allow above.
        #[warn(missing_docs)]
        pub fn undocumented_two() -> i32 { 2 }
    }

    // Missing documentation signals a warning here
    pub fn undocumented_too() -> i32 { 3 }
}
```

This example shows how one can use `forbid` to disallow uses of `allow` or
`expect` for that lint check:

```rust,compile_fail
#[forbid(missing_docs)]
pub mod m3 {
    // Attempting to toggle warning signals an error here
    #[allow(missing_docs)]
    /// Returns 2.
    pub fn undocumented_too() -> i32 { 2 }
}
```

> Note: `rustc` allows setting lint levels on the
> [command-line][rustc-lint-cli], and also supports [setting
> caps][rustc-lint-caps] on the lints that are reported.

### Lint Reasons

All lint attributes support an additional `reason` parameter, to give context why
a certain attribute was added. This reason will be displayed as part of the lint
message if the lint is emitted at the defined level.

```rust,edition2015,compile_fail
// `keyword_idents` is allowed by default. Here we deny it to
// avoid migration of identifiers when we update the edition.
#![deny(
    keyword_idents,
    reason = "we want to avoid these idents to be future compatible"
)]

// This name was allowed in Rust's 2015 edition. We still aim to avoid
// this to be future compatible and not confuse end users.
fn dyn() {}
```

Here is another example, where the lint is allowed with a reason:

```rust
use std::path::PathBuf;

pub fn get_path() -> PathBuf {
    // The `reason` parameter on `allow` attributes acts as documentation for the reader.
    #[allow(unused_mut, reason = "this is only modified on some platforms")]
    let mut file_name = PathBuf::from("git");

    #[cfg(target_os = "windows")]
    file_name.set_extension("exe");

    file_name
}
```

### The `#[expect]` attribute

The `#[expect(C)]` attribute creates a lint expectation for lint `C`. The
expectation will be fulfilled, if a `#[warn(C)]` attribute at the same location
would result in a lint emission. If the expectation is unfulfilled, because
lint `C` would not be emitted, the `unfulfilled_lint_expectations` lint will
be emitted at the attribute.

```rust
fn main() {
    // This `#[expect]` attribute creates a lint expectation, that the `unused_variables`
    // lint would be emitted by the following statement. This expectation is
    // unfulfilled, since the `question` variable is used by the `println!` macro.
    // Therefore, the `unfulfilled_lint_expectations` lint will be emitted at the
    // attribute.
    #[expect(unused_variables)]
    let question = "who lives in a pineapple under the sea?";
    println!("{question}");

    // This `#[expect]` attribute creates a lint expectation that will be fulfilled, since
    // the `answer` variable is never used. The `unused_variables` lint, that would usually
    // be emitted, is suppressed. No warning will be issued for the statement or attribute.
    #[expect(unused_variables)]
    let answer = "SpongeBob SquarePants!";
}
```

The lint expectation is only fulfilled by lint emissions which have been suppressed by
the `expect` attribute. If the lint level is modified in the scope with other level
attributes like `allow` or `warn`, the lint emission will be handled accordingly and the
expectation will remain unfulfilled.

```rust
#[expect(unused_variables)]
fn select_song() {
    // This will emit the `unused_variables` lint at the warn level
    // as defined by the `warn` attribute. This will not fulfill the
    // expectation above the function.
    #[warn(unused_variables)]
    let song_name = "Crab Rave";

    // The `allow` attribute suppresses the lint emission. This will not
    // fulfill the expectation as it has been suppressed by the `allow`
    // attribute and not the `expect` attribute above the function.
    #[allow(unused_variables)]
    let song_creator = "Noisestorm";

    // This `expect` attribute will suppress the `unused_variables` lint emission
    // at the variable. The `expect` attribute above the function will still not
    // be fulfilled, since this lint emission has been suppressed by the local
    // expect attribute.
    #[expect(unused_variables)]
    let song_version = "Monstercat Release";
}
```

If the `expect` attribute contains several lints, each one is expected separately. For a
lint group it's enough if one lint inside the group has been emitted:

```rust
// This expectation will be fulfilled by the unused value inside the function
// since the emitted `unused_variables` lint is inside the `unused` lint group.
#[expect(unused)]
pub fn thoughts() {
    let unused = "I'm running out of examples";
}

pub fn another_example() {
    // This attribute creates two lint expectations. The `unused_mut` lint will be
    // suppressed and with that fulfill the first expectation. The `unused_variables`
    // wouldn't be emitted, since the variable is used. That expectation will therefore
    // be unsatisfied, and a warning will be emitted.
    #[expect(unused_mut, unused_variables)]
    let mut link = "https://www.rust-lang.org/";

    println!("Welcome to our community: {link}");
}
```

> Note: The behavior of `#[expect(unfulfilled_lint_expectations)]` is currently
> defined to always generate the `unfulfilled_lint_expectations` lint.

### Lint groups

Lints may be organized into named groups so that the level of related lints
can be adjusted together. Using a named group is equivalent to listing out the
lints within that group.

```rust,compile_fail
// This allows all lints in the "unused" group.
#[allow(unused)]
// This overrides the "unused_must_use" lint from the "unused"
// group to deny.
#[deny(unused_must_use)]
fn example() {
    // This does not generate a warning because the "unused_variables"
    // lint is in the "unused" group.
    let x = 1;
    // This generates an error because the result is unused and
    // "unused_must_use" is marked as "deny".
    std::fs::remove_file("some_file"); // ERROR: unused `Result` that must be used
}
```

There is a special group named "warnings" which includes all lints at the
"warn" level. The "warnings" group ignores attribute order and applies to all
lints that would otherwise warn within the entity.

```rust,compile_fail
# unsafe fn an_unsafe_fn() {}
// The order of these two attributes does not matter.
#[deny(warnings)]
// The unsafe_code lint is normally "allow" by default.
#[warn(unsafe_code)]
fn example_err() {
    // This is an error because the `unsafe_code` warning has
    // been lifted to "deny".
    unsafe { an_unsafe_fn() } // ERROR: usage of `unsafe` block
}
```

### Tool lint attributes

Tool lints allows using scoped lints, to `allow`, `warn`, `deny` or `forbid`
lints of certain tools.

Tool lints only get checked when the associated tool is active. If a lint
attribute, such as `allow`, references a nonexistent tool lint, the compiler
will not warn about the nonexistent lint until you use the tool.

Otherwise, they work just like regular lint attributes:

```rust
// set the entire `pedantic` clippy lint group to warn
#![warn(clippy::pedantic)]
// silence warnings from the `filter_map` clippy lint
#![allow(clippy::filter_map)]

fn main() {
    // ...
}

// silence the `cmp_nan` clippy lint just for this function
#[allow(clippy::cmp_nan)]
fn foo() {
    // ...
}
```

> Note: `rustc` currently recognizes the tool lints for "[clippy]" and "[rustdoc]".

## The `deprecated` attribute

The *`deprecated` attribute* marks an item as deprecated. `rustc` will issue
warnings on usage of `#[deprecated]` items. `rustdoc` will show item
deprecation, including the `since` version and `note`, if available.

The `deprecated` attribute has several forms:

- `deprecated` --- Issues a generic message.
- `deprecated = "message"` --- Includes the given string in the deprecation
  message.
- [_MetaListNameValueStr_] syntax with two optional fields:
  - `since` --- Specifies a version number when the item was deprecated. `rustc`
    does not currently interpret the string, but external tools like [Clippy]
    may check the validity of the value.
  - `note` --- Specifies a string that should be included in the deprecation
    message. This is typically used to provide an explanation about the
    deprecation and preferred alternatives.

The `deprecated` attribute may be applied to any [item], [trait item], [enum
variant], [struct field], [external block item], or [macro definition]. It
cannot be applied to [trait implementation items]. When applied to an item
containing other items, such as a [module] or [implementation], all child
items inherit the deprecation attribute.
<!-- NOTE: It is only rejected for trait impl items
(AnnotationKind::Prohibited). In all other locations, it is silently ignored.
Tuple struct fields are ignored.
-->

Here is an example:

```rust
#[deprecated(since = "5.2.0", note = "foo was rarely used. Users should instead use bar")]
pub fn foo() {}

pub fn bar() {}
```

The [RFC][1270-deprecation.md] contains motivations and more details.

[1270-deprecation.md]: https://github.com/rust-lang/rfcs/blob/master/text/1270-deprecation.md

## The `must_use` attribute

The *`must_use` attribute* is used to issue a diagnostic warning when a value
is not "used". It can be applied to user-defined composite types
([`struct`s][struct], [`enum`s][enum], and [`union`s][union]), [functions],
and [traits].

The `must_use` attribute may include a message by using the
[_MetaNameValueStr_] syntax such as `#[must_use = "example message"]`. The
message will be given alongside the warning.

When used on user-defined composite types, if the [expression] of an
[expression statement] has that type, then the `unused_must_use` lint is
violated.

```rust
#[must_use]
struct MustUse {
    // some fields
}

# impl MustUse {
#   fn new() -> MustUse { MustUse {} }
# }
#
// Violates the `unused_must_use` lint.
MustUse::new();
```

When used on a function, if the [expression] of an [expression statement] is a
[call expression] to that function, then the `unused_must_use` lint is
violated.

```rust
#[must_use]
fn five() -> i32 { 5i32 }

// Violates the unused_must_use lint.
five();
```

When used on a [trait declaration], a [call expression] of an [expression
statement] to a function that returns an [impl trait] or a [dyn trait] of that trait violates
the `unused_must_use` lint.

```rust
#[must_use]
trait Critical {}
impl Critical for i32 {}

fn get_critical() -> impl Critical {
    4i32
}

// Violates the `unused_must_use` lint.
get_critical();
```

When used on a function in a trait declaration, then the behavior also applies
when the call expression is a function from an implementation of the trait.

```rust
trait Trait {
    #[must_use]
    fn use_me(&self) -> i32;
}

impl Trait for i32 {
    fn use_me(&self) -> i32 { 0i32 }
}

// Violates the `unused_must_use` lint.
5i32.use_me();
```

When used on a function in a trait implementation, the attribute does nothing.

> Note: Trivial no-op expressions containing the value will not violate the
> lint. Examples include wrapping the value in a type that does not implement
> [`Drop`] and then not using that type and being the final expression of a
> [block expression] that is not used.
>
> ```rust
> #[must_use]
> fn five() -> i32 { 5i32 }
>
> // None of these violate the unused_must_use lint.
> (five(),);
> Some(five());
> { five() };
> if true { five() } else { 0i32 };
> match true {
>     _ => five()
> };
> ```

> Note: It is idiomatic to use a [let statement] with a pattern of `_`
> when a must-used value is purposely discarded.
>
> ```rust
> #[must_use]
> fn five() -> i32 { 5i32 }
>
> // Does not violate the unused_must_use lint.
> let _ = five();
> ```

## The `diagnostic` tool attribute namespace

The `#[diagnostic]` attribute namespace is a home for attributes to influence compile-time error messages.
The hints provided by these attributes are not guaranteed to be used.
Unknown attributes in this namespace are accepted, though they may emit warnings for unused attributes.
Additionally, invalid inputs to known attributes will typically be a warning (see the attribute definitions for details).
This is meant to allow adding or discarding attributes and changing inputs in the future to allow changes without the need to keep the non-meaningful attributes or options working.

### The `diagnostic::on_unimplemented` attribute

The `#[diagnostic::on_unimplemented]` attribute is a hint to the compiler to supplement the error message that would normally be generated in scenarios where a trait is required but not implemented on a type.
The attribute should be placed on a [trait declaration], though it is not an error to be located in other positions.
The attribute uses the [_MetaListNameValueStr_] syntax to specify its inputs, though any malformed input to the attribute is not considered as an error to provide both forwards and backwards compatibility.
The following keys have the given meaning:

* `message` --- The text for the top level error message.
* `label` --- The text for the label shown inline in the broken code in the error message.
* `note` --- Provides additional notes.

The `note` option can appear several times, which results in several note messages being emitted.
If any of the other options appears several times the first occurrence of the relevant option specifies the actually used value.
Any other occurrence generates an lint warning.
For any other non-existing option a lint-warning is generated.

All three options accept a string as an argument, interpreted using the same formatting as a [`std::fmt`] string.
Format parameters with the given named parameter will be replaced with the following text:

* `{Self}` --- The name of the type implementing the trait.
* `{` *GenericParameterName* `}` --- The name of the generic argument's type for the given generic parameter.

Any other format parameter will generate a warning, but will otherwise be included in the string as-is.

Invalid format strings may generate a warning, but are otherwise allowed, but may not display as intended.
Format specifiers may generate a warning, but are otherwise ignored.

In this example:

```rust,compile_fail,E0277
#[diagnostic::on_unimplemented(
    message = "My Message for `ImportantTrait<{A}>` implemented for `{Self}`",
    label = "My Label",
    note = "Note 1",
    note = "Note 2"
)]
trait ImportantTrait<A> {}

fn use_my_trait(_: impl ImportantTrait<i32>) {}

fn main() {
    use_my_trait(String::new());
}
```

the compiler may generate an error message which looks like this:

```text
error[E0277]: My Message for `ImportantTrait<i32>` implemented for `String`
  --> src/main.rs:14:18
   |
14 |     use_my_trait(String::new());
   |     ------------ ^^^^^^^^^^^^^ My Label
   |     |
   |     required by a bound introduced by this call
   |
   = help: the trait `ImportantTrait<i32>` is not implemented for `String`
   = note: Note 1
   = note: Note 2
```

[`std::fmt`]: ../../std/fmt/index.html
[Clippy]: https://github.com/rust-lang/rust-clippy
[_MetaListNameValueStr_]: ../attributes.md#meta-item-attribute-syntax
[_MetaListPaths_]: ../attributes.md#meta-item-attribute-syntax
[_MetaNameValueStr_]: ../attributes.md#meta-item-attribute-syntax
[`Drop`]: ../special-types-and-traits.md#drop
[attributes]: ../attributes.md
[block expression]: ../expressions/block-expr.md
[call expression]: ../expressions/call-expr.md
[dyn trait]: ../types/trait-object.md
[enum variant]: ../items/enumerations.md
[enum]: ../items/enumerations.md
[expression statement]: ../statements.md#expression-statements
[expression]: ../expressions.md
[external block item]: ../items/external-blocks.md
[functions]: ../items/functions.md
[impl trait]: ../types/impl-trait.md
[implementation]: ../items/implementations.md
[item]: ../items.md
[let statement]: ../statements.md#let-statements
[macro definition]: ../macros-by-example.md
[module]: ../items/modules.md
[rustc book]: ../../rustc/lints/index.html
[rustc-lint-caps]: ../../rustc/lints/levels.html#capping-lints
[rustc-lint-cli]: ../../rustc/lints/levels.html#via-compiler-flag
[rustdoc]: ../../rustdoc/lints.html
[struct field]: ../items/structs.md
[struct]: ../items/structs.md
[trait declaration]: ../items/traits.md
[trait implementation items]: ../items/implementations.md#trait-implementations
[trait item]: ../items/traits.md
[traits]: ../items/traits.md
[union]: ../items/unions.md
