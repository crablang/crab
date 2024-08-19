# Unsafe `extern` blocks

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".
More information may be found in the tracking issue at <https://github.com/rust-lang/rust/issues/123743>.

## Summary

- [`extern` blocks] must now be marked with the `unsafe` keyword.

[`extern` blocks]: ../../reference/items/external-blocks.html

## Details

Rust 1.82 added the ability in all editions to mark [`extern` blocks] with the `unsafe` keyword.[^RFC3484] Adding the `unsafe` keyword helps to emphasize that it is the responsibility of the author of the `extern` block to ensure that the signatures are correct. If the signatures are not correct, then it may result in undefined behavior.

The syntax for an unsafe `extern` block looks like this:

```rust
unsafe extern "C" {
    // sqrt (from libm) may be called with any `f64`
    pub safe fn sqrt(x: f64) -> f64;

    // strlen (from libc) requires a valid pointer,
    // so we mark it as being an unsafe fn
    pub unsafe fn strlen(p: *const std::ffi::c_char) -> usize;

    // this function doesn't say safe or unsafe, so it defaults to unsafe
    pub fn free(p: *mut core::ffi::c_void);

    pub safe static IMPORTANT_BYTES: [u8; 256];
}
```

In addition to being able to mark an `extern` block as `unsafe`, you can also specify if individual items in the `extern` block are `safe` or `unsafe`. Items marked as `safe` can be used without an `unsafe` block.

Starting with the 2024 Edition, it is now required to include the `unsafe` keyword on an `extern` block. This is intended to make it very clear that there are safety requirements that must be upheld by the extern definitions.

[^RFC3484]: See [RFC 3484](https://github.com/rust-lang/rfcs/blob/master/text/3484-unsafe-extern-blocks.md) for the original proposal.

## Migration

The [`missing_unsafe_on_extern`] lint can update `extern` blocks to add the `unsafe` keyword. The lint is part of the `rust-2024-compatibility` lint group which is included in the automatic edition migration. In order to migrate your code to be Rust 2024 Edition compatible, run:

```sh
cargo fix --edition
```

Just beware that this automatic migration will not be able to verify that the signatures in the `extern` block are correct. It is still your responsibility to manually review their definition.

Alternatively, you can manually enable the lint to find places where there are `unsafe` blocks that need to be updated.

```rust
// Add this to the root of your crate to do a manual migration.
#![warn(missing_unsafe_on_extern)]
```

[`missing_unsafe_on_extern`]: ../../rustc/lints/listing/allowed-by-default.html#missing-unsafe-on-extern
