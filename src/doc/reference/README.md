# The Rust Language Reference

This document is the primary reference for the Rust programming language.

This document is not normative. It may include details that are specific
to `rustc` itself, and should not be taken as a specification for the
Rust language. We intend to produce such a document someday, but this is
what we have for now.

## Dependencies

- rustc (the Rust compiler).
- [mdbook](https://rust-lang.github.io/mdBook/) (use `cargo install mdbook` to install it).
- rust nightly (you would be required to set your Rust version to the nightly version to make sure all tests pass)

## Build steps

To build the project, follow the steps given below :

Clone the project by downloading the ZIP from the [GitHub page](https://github.com/rust-lang/reference) or
run the following command:

```
git clone https://github.com/rust-lang/reference
```

Change the directory to the downloaded repository:

```sh
cd reference
```

To run the tests, you would need to set the Rust version to the nightly release. You can do this by executing the following command:

```shell
rustup override set nightly
```

This will set the nightly version only for your the current project.

If you wish to set Rust nightly for all your projects, you can run the command: 

```shell
rustup default nightly
```

Now, run the following command to test the code snippets to catch compilation errors:

```shell
mdbook test
```


To generate a local instance of the book, run:

```sh
mdbook build
```

The generated HTML will be in the `book` folder.
