# Authoring Guide

This document serves as a guide for editors and reviewers. Some conventions and content guidelines are specified in the [introduction].

[introduction]: ../src/introduction.md

## Markdown formatting

* Use [ATX-style headings][atx] (not Setext) with [sentence case].
* Do not use tabs, only spaces.
* Files must end with a newline.
* Lines must not end with spaces. Double spaces have semantic meaning, but can be invisible. Use a trailing backslash if you need a hard line break.
* If possible, avoid double blank lines.
* Do not use indented code blocks; use 3+ backticks code blocks instead.
* Code blocks should have an explicit language tag.
* Do not wrap long lines. This helps with reviewing diffs of the source.
* Use [smart punctuation] instead of Unicode characters. For example, use `---` for em-dash instead of the Unicode character. Characters like em-dash can be difficult to see in a fixed-width editor, and some editors may not have easy methods to enter such characters.
* Links should be relative with the `.md` extension. Links to other rust-lang books that are published with the reference or the standard library API should also be relative so that the linkchecker can validate them.
* The use of reference links is preferred, with shortcuts if appropriate. Place the sorted link reference definitions at the bottom of the file, or at the bottom of a section if there are an unusually large number of links that are specific to the section.

    ```markdown
    Example of shortcut link: [enumerations]
    Example of reference link with label: [block expression][block]

    [block]: expressions/block-expr.md
    [enumerations]: types/enum.md
    ```
* See the [Conventions] section for formatting callouts such as notes, edition differences, and warnings.

There are automated checks for some of these rules. Run `cargo run --manifest-path style-check/Cargo.toml -- src` to run them locally.

[atx]: https://spec.commonmark.org/0.31.2/#atx-headings
[conventions]: ../src/introduction.md#conventions
[sentence case]: https://apastyle.apa.org/style-grammar-guidelines/capitalization/sentence-case
[smart punctuation]: https://rust-lang.github.io/mdBook/format/markdown.html#smart-punctuation

### Code examples

Code examples should use code blocks with triple backticks. The language should always be specified (such as `rust`).

```rust
println!("Hello!");
```

See <https://rust-lang.github.io/mdBook/format/theme/syntax-highlighting.html#supported-languages> for a list of supported languages.

Rust examples are tested via rustdoc, and should include the appropriate annotations:

* `edition2015` or `edition2018` --- If it is edition-specific (see `book.toml` for the default).
* `no_run` --- The example should compile successfully, but should not be executed.
* `should_panic` --- The example should compile and run, but produce a panic.
* `compile_fail` --- The example is expected to fail to compile.
* `ignore` --- The example shouldn't be built or tested. This should be avoided if possible. Usually this is only necessary when the testing framework does not support it (such as external crates or modules, or a proc-macro), or it contains pseudo-code which is not valid Rust. An HTML comment such as `<!-- ignore: requires extern crate -->` should be placed before the example to explain why it is ignored.
* `Exxxx` --- If the example is expected to fail to compile with a specific error code, include that code so that rustdoc will check that the expected code is used.

See the [rustdoc documentation] for more detail.

[rustdoc documentation]: https://doc.rust-lang.org/rustdoc/documentation-tests.html

## Special markdown constructs

The following are extensions provided by [`mdbook-spec`](https://github.com/rust-lang/spec/tree/main/mdbook-spec).

### Rules

Most clauses should be preceded with a rule. Rules can be specified in the markdown source with the following on a line by itself:

```markdown
r[foo.bar]
```

The rule name should be lowercase, with periods separating from most general to most specific (like `r[array.repeat.zero]`).

Rules can be linked to by their ID using markdown such as `[foo.bar]`. There are automatic link references so that any rule can be referred to from any page in the book.

In the HTML, the rules are clickable just like headers.

### Standard library links

You should link to the standard library without specifying a URL in a fashion similar to [rustdoc intra-doc links][intra]. Some examples:

We can link to the page on `Option`:

```markdown
[`std::option::Option`]
```

In these links, generics are ignored and can be included:

```markdown
[`std::option::Option<T>`]
```

If we don't want the full path in the text, we can write:

```markdown
[`Option`](std::option::Option)
```

Macros can end in `!`. This can be helpful for disambiguation.  For example, this refers to the macro rather than the module:

```markdown
[`alloc::vec!`]
```

Explicit namespace disambiguation is also supported:

```markdown
[`std::vec`](mod@std::vec)
```

[intra]: https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html

### Admonitions

Admonitions use a style similar to GitHub-flavored markdown, where the style name is placed at the beginning of a blockquote, such as:

```markdown
> [!WARNING]
> This is a warning.
```

All this does is apply a CSS class to the blockquote. You should define the color or style of the rule in the `css/custom.css` file if it isn't already defined.

## Style

Idioms and styling to avoid:

* Use American English spelling.
* Use Oxford commas.
* Avoid slashes for alternatives ("program/binary"); use conjunctions or rewrite it ("program or binary").
* Avoid qualifying something as "in Rust"; the entire reference is about Rust.

## Content guidelines

The following are guidelines for the content of the reference.

### Targets

The reference does not document which targets exist, or the properties of specific targets. The reference may refer to *platforms* or *target properties* where required by the language. Some examples:

* Conditional-compilation keys like `target_os` are specified to exist, but not what their values must be.
* The `windows_subsystem` attribute specifies that it only works on Windows platforms.
* Inline assembly and the `target_feature` attribute specify the architectures that are supported.

### Editions

The main text and flow should document only the current edition. Whenever there is a difference between editions, the differences should be called out with an "Edition Differences" block.
