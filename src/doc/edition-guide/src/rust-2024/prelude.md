# Additions to the prelude

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".
More information may be found in the tracking issue at <https://github.com/rust-lang/rust/issues/121042>.

## Summary

- The [`Future`] and [`IntoFuture`] traits are now part of the prelude.
- This might make calls to trait methods ambiguous which could make some code fail to compile.
- `RustcEncodable` and `RustcDecodable` have been removed from the prelude.

[`Future`]: ../../std/future/trait.Future.html
[`IntoFuture`]: ../../std/future/trait.IntoFuture.html

## Details

The [prelude of the standard library](../../std/prelude/index.html) is the module containing everything that is automatically imported in every module.
It contains commonly used items such as `Option`, `Vec`, `drop`, and `Clone`.

The Rust compiler prioritizes any manually imported items over those from the prelude,
to make sure additions to the prelude will not break any existing code.
For example, if you have a crate or module called `example` containing a `pub struct Option;`,
then `use example::*;` will make `Option` unambiguously refer to the one from `example`;
not the one from the standard library.

However, adding a _trait_ to the prelude can break existing code in a subtle way.
For example, a call to `x.poll()` which comes from a `MyPoller` trait might fail to compile if `std`'s `Future` is also imported, because the call to `poll` is now ambiguous and could come from either trait.

As a solution, Rust 2024 will use a new prelude.
It's identical to the current one, except for the following changes:

- Added:
    - [`std::future::Future`][`Future`]
    - [`std::future::IntoFuture`][`IntoFuture`]
- Removed:
    - `RustcEncodable`
    - `RustcDecodable`

### `RustcEncodable` and `RustcDecodable` removal

`RustcEncodable` and `RustcDecodable` are two undocumented derive macros that have been removed from the prelude.
These were deprecated before Rust 1.0, but remained within the standard library prelude.
The 2024 Edition has removed these from the prelude since they are not expected to be used.

If in the unlikely case there is a project still using these, it is recommended to switch to a serialization library, such as those found on [crates.io].

[crates.io]: https://crates.io/categories/encoding

## Migration

🚧 The automatic migration for this has not yet been implemented.

### Migration needed

#### Conflicting trait methods

When two traits that are in scope have the same method name, it is ambiguous which trait method should be used. For example:

```rust,edition2021
trait MyPoller {
    // This name is the same as the `poll` method on the `Future` trait from `std`.
    fn poll(&self) {
        println!("polling");
    }
}

impl<T> MyPoller for T {}

fn main() {
    // Pin<&mut async {}> implements both `std::future::Future` and `MyPoller`.
    // If both traits are in scope (as would be the case in Rust 2024),
    // then it becomes ambiguous which `poll` method to call
    core::pin::pin!(async {}).poll();
}
```

We can fix this by using fully qualified syntax:

```rust,ignore
fn main() {
    // Now it is clear which trait method we're referring to
    <_ as MyPoller>::poll(&core::pin::pin!(async {}));
}
```

#### `RustcEncodable` and `RustcDecodable`

It is strongly recommended that you migrate to a different serialization library if you are still using these.
However, these derive macros are still available in the standard library, they are just required to be imported from the older prelude now:

```rust,edition2021
#[allow(soft_unstable)]
use core::prelude::v1::{RustcDecodable, RustcEncodable};
```
