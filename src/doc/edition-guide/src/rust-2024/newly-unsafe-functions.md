# Unsafe functions

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".
More information may be found in the tracking issue at <https://github.com/rust-lang/rust/issues/124866>.

## Summary

- The following functions are now marked [`unsafe`]:
    - [`std::env::set_var`]
    - [`std::env::remove_var`]
    - [`std::os::unix::process::CommandExt::before_exec`]

[`unsafe`]: ../../reference/unsafe-keyword.html#unsafe-functions-unsafe-fn
[`std::env::set_var`]: ../../std/env/fn.set_var.html
[`std::env::remove_var`]: ../../std/env/fn.remove_var.html
[`std::os::unix::process::CommandExt::before_exec`]: ../../std/os/unix/process/trait.CommandExt.html#method.before_exec

## Details

Over time it has become evident that certain functions in the standard library should have been marked as `unsafe`. However, adding `unsafe` to a function can be a breaking change since it requires existing code to be placed in an `unsafe` block. To avoid the breaking change, these functions are marked as `unsafe` starting in the 2024 Edition, while not requiring `unsafe` in previous editions.

### `std::env::{set_var, remove_var}`

It can be unsound to call [`std::env::set_var`] or [`std::env::remove_var`] in a multi-threaded program due to safety limitations of the way the process environment is handled on some platforms. The standard library originally defined these as safe functions, but it was later determined that was not correct.

It is important to ensure that these functions are not called when any other thread might be running. See the [Safety] section of the function documentation for more details.

[Safety]: ../../std/env/fn.set_var.html#safety

### `std::os::unix::process::CommandExt::before_exec`

The [`std::os::unix::process::CommandExt::before_exec`] function is a unix-specific function which provides a way to run a closure before calling `exec`. This function was deprecated in the 1.37 release, and replaced with [`pre_exec`] which does the same thing, but is marked as `unsafe`.

Even though `before_exec` is deprecated, it is now correctly marked as `unsafe` starting in the 2024 Edition. This should help ensure that any legacy code which has not already migrated to `pre_exec` to require an `unsafe` block.

There are very strict safety requirements for the `before_exec` closure to satisfy. See the [Safety section][pre-exec-safety] for more details.

[`pre_exec`]: ../../std/os/unix/process/trait.CommandExt.html#tymethod.pre_exec
[pre-exec-safety]: ../../std/os/unix/process/trait.CommandExt.html#notes-and-safety

## Migration

To make your code compile in both the 2021 and 2024 editions, you will need to make sure that these functions are called only from within `unsafe` blocks.

**⚠ Caution**: It is important that you manually inspect the calls to these functions and possibly rewrite your code to satisfy the preconditions of those functions. In particular, `set_var` and `remove_var` should not be called if there might be multiple threads running. You may need to elect to use a different mechanism other than environment variables to manage your use case.

The [`deprecated_safe_2024`] lint will automatically modify any use of these functions to be wrapped in an `unsafe` block so that it can compile on both editions. This lint is part of the `rust-2024-compatibility` lint group, which will automatically be applied when running `cargo fix --edition`. To migrate your code to be Rust 2024 Edition compatible, run:

```sh
cargo fix --edition
```

For example, this will change:

```rust
fn main() {
    std::env::set_var("FOO", "123");
}
```

to be:

```rust
fn main() {
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("FOO", "123") };
}
```

Just beware that this automatic migration will not be able to verify that these functions are being used correctly. It is still your responsibility to manually review their usage.

Alternatively, you can manually enable the lint to find places these functions are called:

```rust
// Add this to the root of your crate to do a manual migration.
#![warn(deprecated_safe_2024)]
```

[`deprecated_safe_2024`]: ../../rustc/lints/listing/allowed-by-default.html#deprecated-safe-2024
