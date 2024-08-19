# The Rust Language Reference

This document is the primary reference for the Rust programming
language.

This document is not normative. It may include details that are specific
to `rustc` itself, and should not be taken as a specification for the
Rust language. We intend to produce such a document someday, but this is
what we have for now.

## Dependencies

- Nightly Rust
- [mdbook](https://rust-lang.github.io/mdBook/)

## Installing dependencies

First, ensure that you have a recent copy of the nightly Rust compiler
installed, as this is needed in order to run the tests:

```sh
rustup toolchain install nightly
```

Now, ensure you have `mdbook` installed, as this is needed in order to
build the Reference:

```sh
cargo install --locked mdbook
```

## Building

To build the Reference, first clone the project:

```sh
git clone https://github.com/rust-lang/reference.git
```

(Alternatively, if you don't want to use `git`, [download][] a ZIP file
of the project, extract it using your preferred tool, and rename the
top-level directory to `reference`.)

[download]: https://github.com/rust-lang/reference/archive/refs/heads/master.zip

Now change your current directory to the working directory:

```sh
cd reference
```

To test all of the code examples in the Reference, run:

```sh
mdbook test
```

For authors, consider using the server functionality which supports automatic reload.

To build the Reference locally (in `build/`) and open it in a web
browser, run:

```sh
mdbook build --open
```

This will open a browser with a websocket live-link to automatically reload whenever the source is updated.

You can also open any current build of the reference by running:

```sh
mdbook serve --open
```
