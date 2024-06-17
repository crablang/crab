# Add `IntoIterator` for `Box<[T]>`

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".
More information may be found in the tracking issue at <https://github.com/rust-lang/rust/issues/123759>.

## Summary

- Boxed slices implement [`IntoIterator`] in *all* editions.
- Calls to [`IntoIterator::into_iter`] are *hidden* in editions prior to 2024 when using method call syntax (i.e., `boxed_slice.into_iter()`). So, `boxed_slice.into_iter()` still resolves to `(&(*boxed_slice)).into_iter()` as it has before.
- `boxed_slice.into_iter()` changes meaning to call [`IntoIterator::into_iter`] in Rust 2024.

[`IntoIterator`]: ../../std/iter/trait.IntoIterator.html
[`IntoIterator::into_iter`]: ../../std/iter/trait.IntoIterator.html#tymethod.into_iter

## Details

Until Rust 1.80, `IntoIterator` was not implemented for boxed slices. In prior versions, if you called `.into_iter()` on a boxed slice, the method call would automatically dereference from `Box<[T]>` to `&[T]`, and return an iterator that yielded references of `&T`. For example, the following worked in prior versions:

```rust
// Example of behavior in previous editions.
let my_boxed_slice: Box<[u32]> = vec![1, 2, 3].into_boxed_slice();
// Note: .into_iter() was required in versions older than 1.80
for x in my_boxed_slice.into_iter() {
    // x is of type &u32 in editions prior to 2024
}
```

In Rust 1.80, implementations of `IntoIterator` were added for boxed slices. This allows iterating over elements of the slice by-value instead of by-reference:

```rust
// NEW as of 1.80, all editions
let my_boxed_slice: Box<[u32]> = vec![1, 2, 3].into_boxed_slice();
for x in my_boxed_slice { // notice no need for calling .into_iter()
    // x is of type u32
}
```

This example is allowed on all editions because previously this was an error since `for` loops do not automatically dereference like the `.into_iter()` method call does.

However, this would normally be a breaking change because existing code that manually called `.into_iter()` on a boxed slice would change from having an iterator over references to an iterator over values. To resolve this problem, method calls of `.into_iter()` on boxed slices have edition-dependent behavior. In editions before 2024, it continues to return an iterator over references, and starting in Edition 2024 it returns an iterator over values.

<!-- TODO: edition2024 -->
```rust
// Example of changed behavior in Edition 2024
let my_boxed_slice: Box<[u32]> = vec![1, 2, 3].into_boxed_slice();
// Example of old code that still manually calls .into_iter()
for x in my_boxed_slice.into_iter() {
    // x is now type u32 in Edition 2024
}
```

## Migration

The [`boxed_slice_into_iter`] lint will automatically modify any calls to `.into_iter()` on boxed slices to call `.iter()` instead to retain the old behavior of yielding references. This lint is part of the `rust-2024-compatibility` lint group, which will automatically be applied when running `cargo fix --edition`. To migrate your code to be Rust 2024 Edition compatible, run:

```sh
cargo fix --edition
```

For example, this will change:

```rust
fn main() {
    let my_boxed_slice: Box<[u32]> = vec![1, 2, 3].into_boxed_slice();
    for x in my_boxed_slice.into_iter() {
        // x is of type &u32
    }
}
```

to be:

```rust
fn main() {
    let my_boxed_slice: Box<[u32]> = vec![1, 2, 3].into_boxed_slice();
    for x in my_boxed_slice.iter() {
        // x is of type &u32
    }
}
```

The [`boxed_slice_into_iter`] lint is defaulted to warn on all editions, so unless you have manually silenced the lint, you should already see it before you migrate.

[`boxed_slice_into_iter`]: ../../rustc/lints/listing/warn-by-default.html#boxed-slice-into-iter
