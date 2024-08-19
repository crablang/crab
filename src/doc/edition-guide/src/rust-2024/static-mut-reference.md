# Disallow references to static mut

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".

## Summary

- The [`static_mut_refs`] lint is now a hard error that cannot be disabled.
  This prevents taking a shared or mutable reference to a `static mut`.

[`static_mut_refs`]: ../../rustc/lints/listing/warn-by-default.html#static-mut-refs

## Details

Taking a reference to a [`static mut`] is no longer allowed:

<!-- edition2024,E0796 -->
```rust
static mut X: i32 = 23;
static mut Y: i32 = 24;

unsafe {
    let y = &X;             // ERROR: reference of mutable static
    let ref x = X;          // ERROR: reference of mutable static
    let (x, y) = (&X, &Y);  // ERROR: reference of mutable static
}
```

Merely taking such a reference in violation of Rust's mutability XOR aliasing requirement has always been *instantaneous* [undefined behavior], **even if the reference is never read from or written to**.  Furthermore, upholding mutability XOR aliasing for a `static mut` requires *reasoning about your code globally*, which can be particularly difficult in the face of reentrancy and/or multithreading.

## Alternatives

Wherever possible, it is **strongly recommended** to use instead an *immutable* `static` of a type that provides *interior mutability* behind some *locally-reasoned abstraction* (which greatly reduces the complexity of ensuring that Rust's mutability XOR aliasing requirement is upheld).

In situations where no locally-reasoned abstraction is possible and you are therefore compelled still to reason globally about accesses to your `static` variable, you must now use raw pointers such as can be obtained via the [`addr_of_mut!`] macro.  By first obtaining a raw pointer rather than directly taking a reference, (the safety requirements of) accesses through that pointer will be more familiar to `unsafe` developers and can be deferred until/limited to smaller regions of code.

[Undefined Behavior]: ../../reference/behavior-considered-undefined.html
[`static mut`]: ../../reference/items/static-items.html#mutable-statics
[`addr_of_mut!`]: https://docs.rust-lang.org/core/ptr/macro.addr_of_mut.html

## Migration

🚧 The automatic migration for this has not yet been implemented.

<!-- TODO: Discuss alternatives around rewriting your code. -->
