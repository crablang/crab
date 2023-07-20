# Advanced migration strategies

## How migrations work

[`cargo fix --edition`][`cargo fix`] works by running the equivalent of [`cargo check`] on your project with special [lints] enabled which will detect code that may not compile in the next edition.
These lints include instructions on how to modify the code to make it compatible on both the current and the next edition.
`cargo fix` applies these changes to the source code, and then runs `cargo check` again to verify that the fixes work.
If the fixes fail, then it will back out the changes and display a warning.

Changing the code to be simultaneously compatible with both the current and next edition makes it easier to incrementally migrate the code.
If the automated migration does not completely succeed, or requires manual help, you can iterate while staying on the original edition before changing `Cargo.toml` to use the next edition.

The lints that `cargo fix --edition` apply are part of a [lint group].
For example, when migrating from 2018 to 2021, Cargo uses the `rust-2021-compatibility` group of lints to fix the code.
Check the [Partial migration](#partial-migration-with-broken-code) section below for tips on using individual lints to help with migration.

`cargo fix` may run `cargo check` multiple times.
For example, after applying one set of fixes, this may trigger new warnings which require further fixes.
Cargo repeats this until no new warnings are generated.

## Migrating multiple configurations

`cargo fix` can only work with a single configuration at a time.
If you use [Cargo features] or [conditional compilation], then you may need to run `cargo fix` multiple times with different flags.

For example, if you have code that uses `#[cfg]` attributes to include different code for different platforms, you may need to run `cargo fix` with the `--target` option to fix for different targets.
This may require moving your code between machines if you don't have cross-compiling available.

Similarly, if you have conditions on Cargo features, like `#[cfg(feature = "my-optional-thing")]`, it is recommended to use the `--all-features` flag to allow `cargo fix` to migrate all the code behind those feature gates.
If you want to migrate feature code individually, you can use the `--features` flag to migrate one at a time.

## Migrating a large project or workspace

You can migrate a large project incrementally to make the process easier if you run into problems.

In a [Cargo workspace], each package defines its own edition, so the process naturally involves migrating one package at a time.

Within a [Cargo package], you can either migrate the entire package at once, or migrate individual [Cargo targets] one at a time.
For example, if you have multiple binaries, tests, and examples, you can use specific target selection flags with `cargo fix --edition` to migrate just that one target.
By default, `cargo fix` uses `--all-targets`.

For even more advanced cases, you can specify the edition for each individual target in `Cargo.toml` like this:

```toml
[[bin]]
name = "my-binary"
edition = "2018"
```

This usually should not be required, but is an option if you have a lot of targets and are having difficulty migrating them all together.

## Partial migration with broken code

Sometimes the fixes suggested by the compiler may fail to work.
When this happens, Cargo will report a warning indicating what happened and what the error was.
However, by default it will automatically back out the changes it made.
It can be helpful to keep the code in the broken state and manually resolve the issue.
Some of the fixes may have been correct, and the broken fix maybe be *mostly* correct, but just need minor tweaking.

In this situation, use the `--broken-code` option with `cargo fix` to tell Cargo not to back out the changes.
Then, you can go manually inspect the error and investigate what is needed to fix it.

Another option to incrementally migrate a project is to apply individual fixes separately, one at a time.
You can do this by adding the individual lints as warnings, and then either running `cargo fix` (without the `--edition` flag) or using your editor or IDE to apply its suggestions if it supports "Quick Fixes".

For example, the 2018 edition uses the [`keyword-idents`] lint to fix any conflicting keywords.
You can add `#![warn(keyword_idents)]` to the top of each crate (like at the top of `src/lib.rs` or `src/main.rs`).
Then, running `cargo fix` will apply just the suggestions for that lint.

You can see the list of lints enabled for each edition in the [lint group] page, or run the `rustc -Whelp` command.

## Migrating macros

Some macros may require manual work to fix them for the next edition.
For example, `cargo fix --edition` may not be able to automatically fix a macro that generates syntax that does not work in the next edition.

This may be a problem for both [proc macros] and `macro_rules`-style macros.
`macro_rules` macros can sometimes be automatically updated if the macro is used within the same crate, but there are several situations where it cannot.
Proc macros in general cannot be automatically fixed at all.

For example, if we migrate a crate containing this (contrived) macro `foo` from 2015 to 2018, `foo` would not be automatically fixed.

```rust
#[macro_export]
macro_rules! foo {
    () => {
        let dyn = 1;
        println!("it is {}", dyn);
    };
}
```

When this macro is defined in a 2015 crate, it can be used from a crate of any other edition due to macro hygiene (discussed below).
In 2015, `dyn` is a normal identifier and can be used without restriction.

However, in 2018, `dyn` is no longer a valid identifier.
When using `cargo fix --edition` to migrate to 2018, Cargo won't display any warnings or errors at all.
However, `foo` won't work when called from any crate.

If you have macros, you are encouraged to make sure you have tests that fully cover the macro's syntax.
You may also want to test the macros by importing and using them in crates from multiple editions, just to ensure it works correctly everywhere.
If you run into issues, you'll need to read through the chapters of this guide to understand how the code can be changed to work across all editions.

### Macro hygiene

Macros use a system called "edition hygiene" where the tokens within a macro are marked with which edition they come from.
This allows external macros to be called from crates of varying editions without needing to worry about which edition it is called from.

Let's take a closer look at the example above that defines a `macro_rules` macro using `dyn` as an identifier.
If that macro was defined in a crate using the 2015 edition, then that macro works fine, even if it were called from a 2018 crate where `dyn` is a keyword and that would normally be a syntax error.
The `let dyn = 1;` tokens are marked as being from 2015, and the compiler will remember that wherever that code gets expanded.
The parser looks at the edition of the tokens to know how to interpret it.

The problem arises when changing the edition to 2018 in the crate where it is defined.
Now, those tokens are tagged with the 2018 edition, and those will fail to parse.
However, since we never called the macro from our crate, `cargo fix --edition` never had a chance to inspect the macro and fix it.

<!-- TODO: hopefully someday, the reference will have chapters on how expansion works, and this can link there for actual details. -->

## Documentation tests

At this time, `cargo fix` is not able to update [documentation tests].
After updating the edition in `Cargo.toml`, you should run `cargo test` to ensure everything still passes.
If your documentation tests use syntax that is not supported in the new edition, you will need to update them manually.

In rare cases, you can manually set the edition for each test.
For example, you can use the [`edition2018` annotation][rustdoc-annotation] on the triple backticks to tell `rustdoc` which edition to use.

## Generated code

Another area where the automated fixes cannot apply is if you have a build script which generates Rust code at compile time (see [Code generation] for an example).
In this situation, if you end up with code that doesn't work in the next edition, you will need to manually change the build script to generate code that is compatible.

## Migrating non-Cargo projects

If your project is not using Cargo as a build system, it may still be possible to make use of the automated lints to assist migrating to the next edition.
You can enable the migration lints as described above by enabling the appropriate [lint group].
For example, you can use the `#![warn(rust_2021_compatibility)]` attribute or the `-Wrust-2021-compatibility` or `--force-warns=rust-2021-compatibility` [CLI flag].

The next step is to apply those lints to your code.
There are several options here:

* Manually read the warnings and apply the suggestions recommended by the compiler.
* Use an editor or IDE that supports automatically applying suggestions.
  For example, [Visual Studio Code] with the [Rust Analyzer extension] has the ability to use the "Quick Fix" links to automatically apply suggestions.
  Many other editors and IDEs have similar functionality.
* Write a migration tool using the [`rustfix`] library.
  This is the library that Cargo uses internally to take the [JSON messages] from the compiler and modify the source code.
  Check the [`examples` directory][rustfix-examples] for examples of how to use the library.

## Writing idiomatic code in a new edition

Editions are not only about new features and removing old ones.
In any programming language, idioms change over time, and Rust is no exception.
While old code will continue to compile, it might be written with different idioms today.

For example, in Rust 2015, external crates must be listed with `extern crate` like this:

```rust,ignore
// src/lib.rs
extern crate rand;
```

In Rust 2018, it is [no longer necessary](../rust-2018/path-changes.md#no-more-extern-crate) to include these items.

`cargo fix` has the `--edition-idioms` option to automatically transition some of these idioms to the new syntax.

> **Warning**: The current *"idiom lints"* are known to have some problems.
> They may make incorrect suggestions which may fail to compile.
> The current lints are:
> * Edition 2018:
>     * [`unused-extern-crates`]
>     * [`explicit-outlives-requirements`]
> * Edition 2021 does not have any idiom lints.
>
> The following instructions are recommended only for the intrepid who are willing to work through a few compiler/Cargo bugs!
> If you run into problems, you can try the `--broken-code` option [described above](#partial-migration-with-broken-code) to make as much progress as possible, and then resolve the remaining issues manually.

With that out of the way, we can instruct Cargo to fix our code snippet with:

```console
cargo fix --edition-idioms
```

Afterwards, the line with `extern crate rand;` in `src/lib.rs` will be removed.

We're now more idiomatic, and we didn't have to fix our code manually!

[`cargo check`]: ../../cargo/commands/cargo-check.html
[`cargo fix`]: ../../cargo/commands/cargo-fix.html
[`explicit-outlives-requirements`]:  ../../rustc/lints/listing/allowed-by-default.html#explicit-outlives-requirements
[`keyword-idents`]: ../../rustc/lints/listing/allowed-by-default.html#keyword-idents
[`rustfix`]: https://github.com/rust-lang/rustfix
[`unused-extern-crates`]: ../../rustc/lints/listing/allowed-by-default.html#unused-extern-crates
[Cargo features]: ../../cargo/reference/features.html
[Cargo package]: ../../cargo/reference/manifest.html#the-package-section
[Cargo targets]: ../../cargo/reference/cargo-targets.html
[Cargo workspace]: ../../cargo/reference/workspaces.html
[CLI flag]: ../../rustc/lints/levels.html#via-compiler-flag
[Code generation]: ../../cargo/reference/build-script-examples.html#code-generation
[conditional compilation]: ../../reference/conditional-compilation.html
[documentation tests]: ../../rustdoc/documentation-tests.html
[JSON messages]: ../../rustc/json.html
[lint group]: ../../rustc/lints/groups.html
[lints]: ../../rustc/lints/index.html
[proc macros]: ../../reference/procedural-macros.html
[Rust Analyzer extension]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer
[rustdoc-annotation]: ../../rustdoc/documentation-tests.html#attributes
[rustfix-examples]: https://github.com/rust-lang/rustfix/tree/master/examples
[Visual Studio Code]: https://code.visualstudio.com/
