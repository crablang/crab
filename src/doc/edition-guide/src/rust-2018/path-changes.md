# Path and module system changes

![Minimum Rust version: 1.31](https://img.shields.io/badge/Minimum%20Rust%20Version-1.31-brightgreen.svg)

## Summary

- Paths in `use` declarations now work the same as other paths.
- Paths starting with `::` must now be followed with an external crate.
- Paths in `pub(in path)` visibility modifiers must now start with `crate`, `self`, or `super`.

## Motivation

The module system is often one of the hardest things for people new to Rust. Everyone
has their own things that take time to master, of course, but there's a root
cause for why it's so confusing to many: while there are simple and
consistent rules defining the module system, their consequences can feel
inconsistent, counterintuitive and mysterious.

As such, the 2018 edition of Rust introduces a few new module system
features, but they end up *simplifying* the module system, to make it more
clear as to what is going on.

Here's a brief summary:

* `extern crate` is no longer needed in 99% of circumstances.
* The `crate` keyword refers to the current crate.
* Paths may start with a crate name, even within submodules.
* Paths starting with `::` must reference an external crate.
* A `foo.rs` and `foo/` subdirectory may coexist; `mod.rs` is no longer needed
  when placing submodules in a subdirectory.
* Paths in `use` declarations work the same as other paths.

These may seem like arbitrary new rules when put this way, but the mental
model is now significantly simplified overall. Read on for more details!

## More details

Let's talk about each new feature in turn.

### No more `extern crate`

This one is quite straightforward: you no longer need to write `extern crate` to
import a crate into your project. Before:

```rust,ignore
// Rust 2015

extern crate futures;

mod submodule {
    use futures::Future;
}
```

After:

```rust,ignore
// Rust 2018

mod submodule {
    use futures::Future;
}
```

Now, to add a new crate to your project, you can add it to your `Cargo.toml`,
and then there is no step two. If you're not using Cargo, you already had to pass
`--extern` flags to give `rustc` the location of external crates, so you'd just
keep doing what you were doing there as well.

> One small note here: `cargo fix` will not currently automate this change. We may
> have it do this for you in the future.

#### An exception

There's one exception to this rule, and that's the "sysroot" crates. These are the
crates distributed with Rust itself.

Usually these are only needed in very specialized situations. Starting in
1.41, `rustc` accepts the `--extern=CRATE_NAME` flag which automatically adds
the given crate name in a way similar to `extern crate`. Build tools may use
this to inject sysroot crates into the crate's prelude. Cargo does not have a
general way to express this, though it uses it for `proc_macro` crates.

Some examples of needing to explicitly import sysroot crates are:

* [`std`]: Usually this is not necessary, because `std` is automatically
  imported unless the crate is marked with [`#![no_std]`][no_std].
* [`core`]: Usually this is not necessary, because `core` is automatically
  imported, unless the crate is marked with [`#![no_core]`][no_core]. For
  example, some of the internal crates used by the standard library itself
  need this.
* [`proc_macro`]: This is automatically imported by Cargo if it is a
  proc-macro crate starting in 1.42. `extern crate proc_macro;` would be
  needed if you want to support older releases, or if using another build tool
  that does not pass the appropriate `--extern` flags to `rustc`.
* [`alloc`]: Items in the `alloc` crate are usually accessed via re-exports in
  the `std` crate. If you are working with a `no_std` crate that supports
  allocation, then you may need to explicitly import `alloc`.
* [`test`]: This is only available on the [nightly channel], and is usually
  only used for the unstable benchmark support.

[`alloc`]: ../../alloc/index.html
[`core`]: ../../core/index.html
[`proc_macro`]: ../../proc_macro/index.html
[`std`]: ../../std/index.html
[`test`]: ../../test/index.html
[nightly channel]: ../../book/appendix-07-nightly-rust.html
[no_core]: https://github.com/rust-lang/rust/issues/29639
[no_std]: ../../reference/names/preludes.html#the-no_std-attribute

#### Macros

One other use for `extern crate` was to import macros; that's no longer needed.
Macros may be imported with `use` like any other item. For example, the
following use of `extern crate`:

```rust,ignore
#[macro_use]
extern crate bar;

fn main() {
    baz!();
}
```

Can be changed to something like the following:

```rust,ignore
use bar::baz;

fn main() {
    baz!();
}
```

#### Renaming crates

If you've been using `as` to rename your crate like this:

```rust,ignore
extern crate futures as f;

use f::Future;
```

then removing the `extern crate` line on its own won't work. You'll need to do this:

```rust,ignore
use futures as f;

use self::f::Future;
```

This change will need to happen in any module that uses `f`.

### The `crate` keyword refers to the current crate

In `use` declarations and in other code, you can refer to the root of the
current crate with the `crate::` prefix. For instance, `crate::foo::bar` will
always refer to the name `bar` inside the module `foo`, from anywhere else in
the same crate.

The prefix `::` previously referred to either the crate root or an external
crate; it now unambiguously refers to an external crate. For instance,
`::foo::bar` always refers to the name `bar` inside the external crate `foo`.

### Extern crate paths

Previously, using an external crate in a module without a `use` import
required a leading `::` on the path.

```rust,ignore
// Rust 2015

extern crate chrono;

fn foo() {
    // this works in the crate root
    let x = chrono::Utc::now();
}

mod submodule {
    fn function() {
        // but in a submodule it requires a leading :: if not imported with `use`
        let x = ::chrono::Utc::now();
    }
}
```

Now, extern crate names are in scope in the entire crate, including
submodules.

```rust,ignore
// Rust 2018

fn foo() {
    // this works in the crate root
    let x = chrono::Utc::now();
}

mod submodule {
    fn function() {
        // crates may be referenced directly, even in submodules
        let x = chrono::Utc::now();
    }
}
```

If you have a local module or item with the same name as an external crate, a
path begining with that name will be taken to refer to the local module or
item. To explicitly refer to the external crate, use the `::name` form.


### No more `mod.rs`

In Rust 2015, if you have a submodule:

```rust,ignore
// This `mod` declaration looks for the `foo` module in
// `foo.rs` or `foo/mod.rs`.
mod foo;
```

It can live in `foo.rs` or `foo/mod.rs`. If it has submodules of its own, it
*must* be `foo/mod.rs`. So a `bar` submodule of `foo` would live at
`foo/bar.rs`.

In Rust 2018 the restriction that a module with submodules must be named
`mod.rs` is lifted. `foo.rs` can just be `foo.rs`,
and the submodule is still `foo/bar.rs`. This eliminates the special
name, and if you have a bunch of files open in your editor, you can clearly
see their names, instead of having a bunch of tabs named `mod.rs`.

<table>
  <thead>
    <tr>
      <th>Rust 2015</th>
      <th>Rust 2018</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>
<pre>
.
├── lib.rs
└── foo/
    ├── mod.rs
    └── bar.rs
</pre>
    </td>
    <td>
<pre>
.
├── lib.rs
├── foo.rs
└── foo/
    └── bar.rs
</pre>
      </td>
    </tr>
  </tbody>
</table>

### `use` paths

![Minimum Rust version: 1.32](https://img.shields.io/badge/Minimum%20Rust%20Version-1.32-brightgreen.svg)

Rust 2018 simplifies and unifies path handling compared to Rust 2015. In Rust
2015, paths work differently in `use` declarations than they do elsewhere. In
particular, paths in `use` declarations would always start from the crate
root, while paths in other code implicitly started from the current scope.
Those differences didn't have any effect in the top-level module, which meant
that everything would seem straightforward until working on a project large
enough to have submodules.

In Rust 2018, paths in `use` declarations and in other code work the same way,
both in the top-level module and in any submodule. You can use a relative path
from the current scope, a path starting from an external crate name, or a path
starting with `::`, `crate`, `super`, or `self`.

Code that looked like this:

```rust,ignore
// Rust 2015

extern crate futures;

use futures::Future;

mod foo {
    pub struct Bar;
}

use foo::Bar;

fn my_poll() -> futures::Poll { ... }

enum SomeEnum {
    V1(usize),
    V2(String),
}

fn func() {
    let five = std::sync::Arc::new(5);
    use SomeEnum::*;
    match ... {
        V1(i) => { ... }
        V2(s) => { ... }
    }
}
```

will look exactly the same in Rust 2018, except that you can delete the `extern
crate` line:

```rust,ignore
// Rust 2018

use futures::Future;

mod foo {
    pub struct Bar;
}

use foo::Bar;

fn my_poll() -> futures::Poll { ... }

enum SomeEnum {
    V1(usize),
    V2(String),
}

fn func() {
    let five = std::sync::Arc::new(5);
    use SomeEnum::*;
    match ... {
        V1(i) => { ... }
        V2(s) => { ... }
    }
}
```

The same code will also work completely unmodified in a submodule:

```rust,ignore
// Rust 2018

mod submodule {
    use futures::Future;

    mod foo {
        pub struct Bar;
    }

    use foo::Bar;

    fn my_poll() -> futures::Poll { ... }

    enum SomeEnum {
        V1(usize),
        V2(String),
    }

    fn func() {
        let five = std::sync::Arc::new(5);
        use SomeEnum::*;
        match ... {
            V1(i) => { ... }
            V2(s) => { ... }
        }
    }
}
```

This makes it easy to move code around in a project, and avoids introducing
additional complexity to multi-module projects.
