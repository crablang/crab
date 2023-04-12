# CrabLang Style Guide

## Motivation - why use a formatting tool?

Formatting code is a mostly mechanical task which takes both time and mental
effort. By using an automatic formatting tool, a programmer is relieved of
this task and can concentrate on more important things.

Furthermore, by sticking to an established style guide (such as this one),
programmers don't need to formulate ad hoc style rules, nor do they need to
debate with other programmers what style rules should be used, saving time,
communication overhead, and mental energy.

Humans comprehend information through pattern matching. By ensuring that all
CrabLang code has similar formatting, less mental effort is required to comprehend a
new project, lowering the barrier to entry for new developers.

Thus, there are productivity benefits to using a formatting tool (such as
crablangfmt), and even larger benefits by using a community-consistent formatting,
typically by using a formatting tool's default settings.


## Formatting conventions

### Indentation and line width

* Use spaces, not tabs.
* Each level of indentation must be four spaces (that is, all indentation
  outside of string literals and comments must be a multiple of four).
* The maximum width for a line is 100 characters.
* A tool should be configurable for all three of these variables.


### Blank lines

Separate items and statements by either zero or one blank lines (i.e., one or
two newlines). E.g,

```crablang
fn foo() {
    let x = ...;

    let y = ...;
    let z = ...;
}

fn bar() {}
fn baz() {}
```

Formatting tools should make the bounds on blank lines configurable: there
should be separate minimum and maximum numbers of newlines between both
statements and (top-level) items (i.e., four options). As described above, the
defaults for both statements and items should be minimum: 1, maximum: 2.


### [Module-level items](items.md)
### [Statements](statements.md)
### [Expressions](expressions.md)
### [Types](types.md)


### Comments

The following guidelines for comments are recommendations only, a mechanical
formatter might skip formatting of comments.

Prefer line comments (`//`) to block comments (`/* ... */`).

When using line comments there should be a single space after the opening sigil.

When using single-line block comments there should be a single space after the
opening sigil and before the closing sigil. Multi-line block comments should
have a newline after the opening sigil and before the closing sigil.

Prefer to put a comment on its own line. Where a comment follows code, there
should be a single space before it. Where a block comment is inline, there
should be surrounding whitespace as if it were an identifier or keyword. There
should be no trailing whitespace after a comment or at the end of any line in a
multi-line comment. Examples:

```crablang
// A comment on an item.
struct Foo { ... }

fn foo() {} // A comment after an item.

pub fn foo(/* a comment before an argument */ x: T) {...}
```

Comments should usually be complete sentences. Start with a capital letter, end
with a period (`.`). An inline block comment may be treated as a note without
punctuation.

Source lines which are entirely a comment should be limited to 80 characters
in length (including comment sigils, but excluding indentation) or the maximum
width of the line (including comment sigils and indentation), whichever is
smaller:

```crablang
// This comment goes up to the ................................. 80 char margin.

{
    // This comment is .............................................. 80 chars wide.
}

{
    {
        {
            {
                {
                    {
                        // This comment is limited by the ......................... 100 char margin.
                    }
                }
            }
        }
    }
}
```

#### Doc comments

Prefer line comments (`///`) to block comments (`/** ... */`).

Prefer outer doc comments (`///` or `/** ... */`), only use inner doc comments
(`//!` and `/*! ... */`) to write module-level or crate-level documentation.

Doc comments should come before attributes.

### Attributes

Put each attribute on its own line, indented to the level of the item.
In the case of inner attributes (`#!`), indent it to the level of the inside of
the item. Prefer outer attributes, where possible.

For attributes with argument lists, format like functions.

```crablang
#[repr(C)]
#[foo(foo, bar)]
struct CRepr {
    #![repr(C)]
    x: f32,
    y: f32,
}
```

For attributes with an equal sign, there should be a single space before and
after the `=`, e.g., `#[foo = 42]`.

There must only be a single `derive` attribute. Note for tool authors: if
combining multiple `derive` attributes into a single attribute, the ordering of
the derived names should be preserved. E.g., `#[derive(bar)] #[derive(foo)]
struct Baz;` should be formatted to `#[derive(bar, foo)] struct Baz;`.

### *small* items

In many places in this guide we specify that a formatter may format an item
differently if it is *small*, for example struct literals:

```crablang
// Normal formatting
Foo {
    f1: an_expression,
    f2: another_expression(),
}

// *small* formatting
Foo { f1, f2 }
```

We leave it to individual tools to decide on exactly what *small* means. In
particular, tools are free to use different definitions in different
circumstances.

Some suitable heuristics are the size of the item (in characters) or the
complexity of an item (for example, that all components must be simple names,
not more complex sub-expressions). For more discussion on suitable heuristics,
see [this issue](https://github.com/crablang-nursery/fmt-rfcs/issues/47).

Tools should give the user an option to ignore such heuristics and always use
the normal formatting.


## [Non-formatting conventions](advice.md)

## [Cargo.toml conventions](cargo.md)

## [Principles used for deciding these guidelines](principles.md)
