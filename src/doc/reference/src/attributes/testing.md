# Testing attributes

The following [attributes] are used for specifying functions for performing
tests. Compiling a crate in "test" mode enables building the test functions
along with a test harness for executing the tests. Enabling the test mode also
enables the [`test` conditional compilation option].

## The `test` attribute

The *`test` attribute* marks a function to be executed as a test. These
functions are only compiled when in test mode. Test functions must be free,
monomorphic functions that take no arguments, and the return type must implement the [`Termination`] trait, for example:

* `()`
* `Result<T, E> where T: Termination, E: Debug`
* `!`

<!-- If the previous section needs updating (from "must take no arguments"
  onwards, also update it in the crates-and-source-files.md file -->

> Note: The test mode is enabled by passing the `--test` argument to `rustc`
> or using `cargo test`.

The test harness calls the returned value's [`report`] method, and classifies the test as passed or failed depending on whether the resulting [`ExitCode`] represents successful termination.
In particular:
* Tests that return `()` pass as long as they terminate and do not panic.
* Tests that return a `Result<(), E>` pass as long as they return `Ok(())`.
* Tests that return `ExitCode::SUCCESS` pass, and tests that return `ExitCode::FAILURE` fail.
* Tests that do not terminate neither pass nor fail.

```rust
# use std::io;
# fn setup_the_thing() -> io::Result<i32> { Ok(1) }
# fn do_the_thing(s: &i32) -> io::Result<()> { Ok(()) }
#[test]
fn test_the_thing() -> io::Result<()> {
    let state = setup_the_thing()?; // expected to succeed
    do_the_thing(&state)?;          // expected to succeed
    Ok(())
}
```

## The `ignore` attribute

A function annotated with the `test` attribute can also be annotated with the
`ignore` attribute. The *`ignore` attribute* tells the test harness to not
execute that function as a test. It will still be compiled when in test mode.

The `ignore` attribute may optionally be written with the [_MetaNameValueStr_]
syntax to specify a reason why the test is ignored.

```rust
#[test]
#[ignore = "not yet implemented"]
fn mytest() {
    // …
}
```

> **Note**: The `rustc` test harness supports the `--include-ignored` flag to
> force ignored tests to be run.

## The `should_panic` attribute

A function annotated with the `test` attribute that returns `()` can also be
annotated with the `should_panic` attribute. The *`should_panic` attribute*
makes the test only pass if it actually panics.

The `should_panic` attribute may optionally take an input string that must
appear within the panic message. If the string is not found in the message,
then the test will fail. The string may be passed using the
[_MetaNameValueStr_] syntax or the [_MetaListNameValueStr_] syntax with an
`expected` field.

```rust
#[test]
#[should_panic(expected = "values don't match")]
fn mytest() {
    assert_eq!(1, 2, "values don't match");
}
```

[_MetaListNameValueStr_]: ../attributes.md#meta-item-attribute-syntax
[_MetaNameValueStr_]: ../attributes.md#meta-item-attribute-syntax
[`Termination`]: ../../std/process/trait.Termination.html
[`report`]: ../../std/process/trait.Termination.html#tymethod.report
[`test` conditional compilation option]: ../conditional-compilation.md#test
[attributes]: ../attributes.md
[`ExitCode`]: ../../std/process/struct.ExitCode.html
