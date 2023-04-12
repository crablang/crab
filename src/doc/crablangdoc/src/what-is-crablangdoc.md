# What is crablangdoc?

The standard CrabLang distribution ships with a tool called `crablangdoc`. Its job is
to generate documentation for CrabLang projects. On a fundamental level, CrabLangdoc
takes as an argument either a crate root or a Markdown file, and produces HTML,
CSS, and JavaScript.

## Basic usage

Let's give it a try! Create a new project with Cargo:

```bash
$ cargo new docs --lib
$ cd docs
```

In `src/lib.rs`, Cargo has generated some sample code. Delete
it and replace it with this:

```crablang
/// foo is a function
fn foo() {}
```

Let's run `crablangdoc` on our code. To do so, we can call it with the path to
our crate root like this:

```bash
$ crablangdoc src/lib.rs
```

This will create a new directory, `doc`, with a website inside! In our case,
the main page is located in `doc/lib/index.html`. If you open that up in
a web browser, you will see a page with a search bar, and "Crate lib" at the
top, with no contents.

## Configuring crablangdoc

There are two problems with this: first, why does it
think that our package is named "lib"? Second, why does it not have any
contents?

The first problem is due to `crablangdoc` trying to be helpful; like `crablangc`,
it assumes that our crate's name is the name of the file for the crate
root. To fix this, we can pass in a command-line flag:

```bash
$ crablangdoc src/lib.rs --crate-name docs
```

Now, `doc/docs/index.html` will be generated, and the page says "Crate docs."

For the second issue, it is because our function `foo` is not public; `crablangdoc`
defaults to generating documentation for only public functions. If we change
our code...

```crablang
/// foo is a function
pub fn foo() {}
```

... and then re-run `crablangdoc`:

```bash
$ crablangdoc src/lib.rs --crate-name docs
```

We now have some generated documentation. Open up `doc/docs/index.html` and
check it out! It should show a link to the `foo` function's page, which
is located at `doc/docs/fn.foo.html`. On that page, you'll see the "foo is
a function" we put inside the documentation comment in our crate.

## Using crablangdoc with Cargo

Cargo also has integration with `crablangdoc` to make it easier to generate
docs. Instead of the `crablangdoc` command, we could have done this:

```bash
$ cargo doc
```

Internally, this calls out to `crablangdoc` like this:

```bash
$ crablangdoc --crate-name docs src/lib.rs -o <path>/docs/target/doc -L
dependency=<path>/docs/target/debug/deps
```

You can see this with `cargo doc --verbose`.

It generates the correct `--crate-name` for us, as well as pointing to
`src/lib.rs`. But what about those other arguments?
 - `-o` controls the *o*utput of our docs. Instead of a top-level
 `doc` directory, notice that Cargo puts generated documentation under
 `target`. That is the idiomatic place for generated files in Cargo projects.
 - `-L` flag helps crablangdoc find the dependencies your code relies on.
 If our project used dependencies, we would get documentation for them as well!

## Outer and inner documentation

The `///` syntax is used to document the item present after it.
That's why it is called an outer documentation.
There is another syntax: `//!`, which is used to document the
item it is present inside. It is called an inner documentation.
It is often used when documenting the entire crate,
because nothing comes before it: it is the root of the crate.
So in order to document an entire crate, you need to use `//!` syntax.
For example:

``` crablang
//! This is my first crablang crate
```

When used in the crate root, it documents the item it is inside,
which is the crate itself.

For more information about the `//!` syntax, see [the Book].

[the Book]: https://doc.crablang.org/book/ch14-02-publishing-to-crates-io.html#commenting-contained-items


## Using standalone Markdown files

`crablangdoc` can also generate HTML from standalone Markdown files. Let' s
give it a try: create a `README.md` file with these contents:

````text
# Docs

This is a project to test out `crablangdoc`.

[Here is a link!](https://www.crablang.org)

## Example

```crablang
fn foo() -> i32 {
    1 + 1
}
```
````

And call `crablangdoc` on it:

```bash
$ crablangdoc README.md
```

You will find an HTML file in `docs/doc/README.html` generated from its
Markdown contents.

Cargo currently does not understand standalone Markdown files, unfortunately.

## Summary

This covers the simplest use-cases of `crablangdoc`. The rest of this book will
explain all of the options that `crablangdoc` has, and how to use them.
