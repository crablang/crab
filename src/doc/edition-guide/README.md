# The Rust Edition Guide

This book explains the concept of "editions", major new eras in [Rust]'s
development. You can [read the book
online](https://doc.rust-lang.org/nightly/edition-guide/).

[Rust]: https://www.rust-lang.org/

## License

The Rust Edition Guide is dual licensed under `MIT`/`Apache2`, just like Rust
itself. See the `LICENSE-*` files in this repository for more details.

## Building locally

You can also build the book and read it locally if you'd like.

### Requirements

Building the book requires [mdBook] 0.4. To get it:

[mdBook]: https://github.com/rust-lang/mdBook

```bash
$ cargo install mdbook
```

### Building

The most straight-forward way to build and view the book locally is to use the following command:
```bash
$ mdbook serve --open
```

This builds the HTML version of the book, starts a webserver at
http://localhost:3000, and opens your default web browser. It will also
automatically rebuild the book whenever the source changes, and the page
should automatically reload.

To run the tests:

```bash
$ mdbook test
```
