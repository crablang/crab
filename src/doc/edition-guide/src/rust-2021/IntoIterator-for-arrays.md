# IntoIterator for arrays

## Summary

- Arrays implement `IntoIterator` in *all* editions.
- Calls to `IntoIterator::into_iter` are *hidden* in Rust 2015 and Rust 2018 when using method call syntax
  (i.e., `array.into_iter()`). So, `array.into_iter()` still resolves to `(&array).into_iter()` as it
  has before.
- `array.into_iter()` changes meaning to be the call to `IntoIterator::into_iter` in Rust 2021.

## Details

Until Rust 1.53, only *references* to arrays implement `IntoIterator`.
This means you can iterate over `&[1, 2, 3]` and `&mut [1, 2, 3]`,
but not over `[1, 2, 3]` directly.

```rust,ignore
for &e in &[1, 2, 3] {} // Ok :)

for e in [1, 2, 3] {} // Error :(
```

This has been [a long-standing issue][25], but the solution is not as simple as it seems.
Just [adding the trait implementation][20] would break existing code.
`array.into_iter()` already compiles today because that implicitly calls
`(&array).into_iter()` due to [how method call syntax works][22].
Adding the trait implementation would change the meaning.

Usually this type of breakage (adding a trait implementation) is categorized as 'minor' and acceptable.
But in this case there is too much code that would be broken by it.

It has been suggested many times to "only implement `IntoIterator` for arrays in Rust 2021".
However, this is simply not possible.
You can't have a trait implementation exist in one edition and not in another,
since editions can be mixed.

Instead, the trait implementation was added in *all* editions (starting in Rust 1.53.0)
but with a small hack to avoid breakage until Rust 2021.
In Rust 2015 and 2018 code, the compiler will still resolve `array.into_iter()`
to `(&array).into_iter()` like before, as if the trait implementation does not exist.
This *only* applies to the `.into_iter()` method call syntax.
It does not affect any other syntax such as `for e in [1, 2, 3]`, `iter.zip([1, 2, 3])` or
`IntoIterator::into_iter([1, 2, 3])`.
Those will start to work in *all* editions.

While it's a shame that this required a small hack to avoid breakage,
this solution keeps the difference between the editions to an absolute minimum.

[25]: https://github.com/rust-lang/rust/issues/25725
[20]: https://github.com/rust-lang/rust/pull/65819
[22]: https://doc.rust-lang.org/book/ch05-03-method-syntax.html#wheres-the---operator

## Migration

A lint, `array_into_iter`, gets triggered whenever there is some call to `into_iter()` that will change
meaning in Rust 2021. The `array_into_iter` lint has already been a warning by default on all editions 
since the 1.41 release (with several enhancements made in 1.55). If your code is already warning free, 
then it should already be ready to go for Rust 2021!

You can automatically migrate your code to be Rust 2021 Edition compatible or ensure it is already compatible by
running:

```sh
cargo fix --edition
```

Because the difference between editions is small, the migration to Rust 2021 is fairly straight-forward.

For method calls of `into_iter` on arrays, the elements being implemented will change from references to owned values.

For example:

```rust
fn main() {
  let array = [1u8, 2, 3];
  for x in array.into_iter() {
    // x is a `&u8` in Rust 2015 and Rust 2018
    // x is a `u8` in Rust 2021
  }
}
```

The most straightforward way to migrate in Rust 2021, is by keeping the exact behavior from previous editions
by calling `iter()` which also iterates over owned arrays by reference:

```rust
fn main() {
  let array = [1u8, 2, 3];
  for x in array.iter() { // <- This line changed
    // x is a `&u8` in all editions
  }
}
```

### Optional migration

If you are using fully qualified method syntax (i.e., `IntoIterator::into_iter(array)`) in a previous edition,
this can be upgraded to method call syntax (i.e., `array.into_iter()`).
