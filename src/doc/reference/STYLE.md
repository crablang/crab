# Rust reference style guide

Some conventions and content guidelines are specified in the [introduction].
This document serves as a guide for editors and reviewers.

There is a [`style-check`](style-check/) tool which is run in CI to check some of these. To use it locally, run `cargo run --manifest-path=style-check/Cargo.toml src`.

## Markdown formatting

* Use ATX-style heading with sentence case.
* Use one line per sentence to make diffs nicer.
  Do not wrap long lines.
* Use reference links, with shortcuts if appropriate.
  Place the sorted link reference definitions at the bottom of the file, or at the bottom of a section if there is an unusually large number of links that are specific to the section.

    ```
    Example of shortcut link: [enumerations]
    Example of reference link with label: [block expression][block]

    [block]: expressions/block-expr.md
    [enumerations]: types/enum.md
    ```

* Links should be relative with the `.md` extension.
  Links to other rust-lang books that are published with the reference or the standard library API should also be relative so that the linkchecker can validate them.
* See the [Conventions] section for formatting callouts such as notes, edition differences, and warnings.
* Formatting to avoid:
    * Avoid trailing spaces.
    * Avoid double blank lines.

### Code examples

Code examples should use code blocks with triple backticks.
The language should always be specified (such as `rust`).

```rust
println!("Hello!");
```

See https://highlightjs.org/ for a list of supported languages.

Rust examples are tested via rustdoc, and should include the appropriate annotations when tests are expected to fail:

* `edition2015` or `edition2018` — If it is edition-specific (see `book.toml` for the default).
* `no_run` — The example should compile successfully, but should not be executed.
* `should_panic` — The example should compile and run, but produce a panic.
* `compile_fail` — The example is expected to fail to compile.
* `ignore` — The example shouldn't be built or tested.
  This should be avoided if possible.
  Usually this is only necessary when the testing framework does not support it (such as external crates or modules, or a proc-macro), or it contains pseudo-code which is not valid Rust.
  An HTML comment such as `<!-- ignore: requires extern crate -->` should be placed before the example to explain why it is ignored.

See the [rustdoc documentation] for more detail.

## Language and grammar

* Use American English spelling.
* Use Oxford commas.
* Idioms and styling to avoid:
    * Avoid slashes for alternatives ("program/binary"), use conjunctions or rewrite it ("program or binary").
    * Avoid qualifying something as "in Rust", the entire reference is about Rust.

## Content

* Whenever there is a difference between editions, the differences should be called out with an "Edition Differences" block.
  The main text should stick to what is common between the editions.
  However, for large differences (such as "async"), the main text may contain edition-specific content as long as it is made clear which editions it applies to.

[conventions]: src/introduction.md#conventions
[introduction]: src/introduction.md
[rustdoc documentation]: https://doc.rust-lang.org/rustdoc/documentation-tests.html
