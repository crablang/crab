# Transitioning an existing project to a new edition

Rust includes tooling to automatically transition a project from one edition to the next.
It will update your source code so that it is compatible with the next edition.
Briefly, the steps to update to the next edition are:

1. Run `cargo fix --edition`
2. Edit `Cargo.toml` and set the `edition` field to the next edition, for example `edition = "2021"`
3. Run `cargo build` or `cargo test` to verify the fixes worked.

The following sections dig into the details of these steps, and some of the issues you may encounter along the way.

> It's our intention that the migration to new editions is as smooth an
> experience as possible. If it's difficult for you to upgrade to the latest edition,
> we consider that a bug. If you run into problems with this process, please
> [file a bug report](https://github.com/rust-lang/rust/issues/new/choose). Thank you!

## Starting the migration

As an example, let's take a look at transitioning from the 2015 edition to the 2018 edition.
The steps are essentially the same when transitioning to other editions like 2021.

Imagine we have a crate that has this code in `src/lib.rs`:

```rust
trait Foo {
    fn foo(&self, i32);
}
```

This code uses an anonymous parameter, that `i32`. This is [not
supported in Rust 2018](../rust-2018/trait-system/no-anon-params.md), and
so this would fail to compile. Let's get this code up to date!

## Updating your code to be compatible with the new edition

Your code may or may not use features that are incompatible with the new edition.
In order to help transition to the next edition, Cargo includes the [`cargo fix`] subcommand to automatically update your source code.
To start, let's run it:

```console
cargo fix --edition
```

This will check your code, and automatically fix any issues that it can.
Let's look at `src/lib.rs` again:

```rust
trait Foo {
    fn foo(&self, _: i32);
}
```

It's re-written our code to introduce a parameter name for that `i32` value.
In this case, since it had no name, `cargo fix` will replace it with `_`,
which is conventional for unused variables.

`cargo fix` can't always fix your code automatically.
If `cargo fix` can't fix something, it will print the warning that it cannot fix
to the console. If you see one of these warnings, you'll have to update your code manually.
See the [Advanced migration strategies] chapter for more on working with the migration process, and read the chapters in this guide which explain which changes are needed.
If you have problems, please seek help at the [user's forums](https://users.rust-lang.org/).

## Enabling the new edition to use new features

In order to use some new features, you must explicitly opt in to the new
edition. Once you're ready to continue, change your `Cargo.toml` to add the new
`edition` key/value pair. For example:

```toml
[package]
name = "foo"
version = "0.1.0"
edition = "2018"
```

If there's no `edition` key, Cargo will default to Rust 2015. But in this case,
we've chosen `2018`, and so our code will compile with Rust 2018!

The next step is to test your project on the new edition.
Run your project tests to verify that everything still works, such as running [`cargo test`].
If new warnings are issued, you may want to consider running `cargo fix` again (without the `--edition` flag) to apply any suggestions given by the compiler.

Congrats! Your code is now valid in both Rust 2015 and Rust 2018!

[`cargo fix`]: ../../cargo/commands/cargo-fix.html
[`cargo test`]: ../../cargo/commands/cargo-test.html
[Advanced migration strategies]: advanced-migrations.md
[nightly channel]: ../../book/appendix-07-nightly-rust.html
