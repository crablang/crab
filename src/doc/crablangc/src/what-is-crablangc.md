# What is crablangc?

Welcome to "The crablangc book"! `crablangc` is the compiler for the CrabLang programming
language, provided by the project itself. Compilers take your source code and
produce binary code, either as a library or executable.

Most CrabLang programmers don't invoke `crablangc` directly, but instead do it through
[Cargo](../cargo/index.html). It's all in service of `crablangc` though! If you
want to see how Cargo calls `crablangc`, you can

```bash
$ cargo build --verbose
```

And it will print out each `crablangc` invocation. This book can help you
understand what each of these options does. Additionally, while most
CrabLangaceans use Cargo, not all do: sometimes they integrate `crablangc` into other
build systems. This book should provide a guide to all of the options you'd
need to do so.

## Basic usage

Let's say you've got a little hello world program in a file `hello.rs`:

```crablang
fn main() {
    println!("Hello, world!");
}
```

To turn this source code into an executable, you can use `crablangc`:

```bash
$ crablangc hello.rs
$ ./hello # on a *NIX
$ .\hello.exe # on Windows
```

Note that we only ever pass `crablangc` the *crate root*, not every file we wish
to compile. For example, if we had a `main.rs` that looked like this:

```crablang,ignore (needs-multiple-files)
mod foo;

fn main() {
    foo::hello();
}
```

And a `foo.rs` that had this:

```crablang,no_run
pub fn hello() {
    println!("Hello, world!");
}
```

To compile this, we'd run this command:

```bash
$ crablangc main.rs
```

No need to tell `crablangc` about `foo.rs`; the `mod` statements give it
everything that it needs. This is different than how you would use a C
compiler, where you invoke the compiler on each file, and then link
everything together. In other words, the *crate* is a translation unit, not a
particular module.
