# Use declarations

> **<sup>Syntax:</sup>**\
> _UseDeclaration_ :\
> &nbsp;&nbsp; `use` _UseTree_ `;`
>
> _UseTree_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; ([_SimplePath_]<sup>?</sup> `::`)<sup>?</sup> `*`\
> &nbsp;&nbsp; | ([_SimplePath_]<sup>?</sup> `::`)<sup>?</sup> `{` (_UseTree_ ( `,`  _UseTree_ )<sup>\*</sup> `,`<sup>?</sup>)<sup>?</sup> `}`\
> &nbsp;&nbsp; | [_SimplePath_]&nbsp;( `as` ( [IDENTIFIER] | `_` ) )<sup>?</sup>

A _use declaration_ creates one or more local name bindings synonymous with
some other [path]. Usually a `use` declaration is used to shorten the path
required to refer to a module item. These declarations may appear in [modules]
and [blocks], usually at the top.

[path]: ../paths.md
[modules]: modules.md
[blocks]: ../expressions/block-expr.md

Use declarations support a number of convenient shortcuts:

* Simultaneously binding a list of paths with a common prefix, using the
  glob-like brace syntax `use a::b::{c, d, e::f, g::h::i};`
* Simultaneously binding a list of paths with a common prefix and their common
  parent module, using the `self` keyword, such as `use a::b::{self, c, d::e};`
* Rebinding the target name as a new local name, using the syntax `use p::q::r
  as x;`. This can also be used with the last two features:
  `use a::b::{self as ab, c as abc}`.
* Binding all paths matching a given prefix, using the asterisk wildcard syntax
  `use a::b::*;`.
* Nesting groups of the previous features multiple times, such as
  `use a::b::{self as ab, c, d::{*, e::f}};`

An example of `use` declarations:

```rust
use std::collections::hash_map::{self, HashMap};

fn foo<T>(_: T){}
fn bar(map1: HashMap<String, usize>, map2: hash_map::HashMap<String, usize>){}

fn main() {
    // use declarations can also exist inside of functions
    use std::option::Option::{Some, None};

    // Equivalent to 'foo(vec![std::option::Option::Some(1.0f64),
    // std::option::Option::None]);'
    foo(vec![Some(1.0f64), None]);

    // Both `hash_map` and `HashMap` are in scope.
    let map1 = HashMap::new();
    let map2 = hash_map::HashMap::new();
    bar(map1, map2);
}
```

## `use` Visibility

Like items, `use` declarations are private to the containing module, by
default. Also like items, a `use` declaration can be public, if qualified by
the `pub` keyword. Such a `use` declaration serves to _re-export_ a name. A
public `use` declaration can therefore _redirect_ some public name to a
different target definition: even a definition with a private canonical path,
inside a different module. If a sequence of such redirections form a cycle or
cannot be resolved unambiguously, they represent a compile-time error.

An example of re-exporting:

```rust
mod quux {
    pub use self::foo::{bar, baz};
    pub mod foo {
        pub fn bar() {}
        pub fn baz() {}
    }
}

fn main() {
    quux::bar();
    quux::baz();
}
```

In this example, the module `quux` re-exports two public names defined in
`foo`.

## `use` Paths

> **Note**: This section is incomplete.

Some examples of what will and will not work for `use` items:
<!-- Note: This example works as-is in either 2015 or 2018. -->

```rust
# #![allow(unused_imports)]
use std::path::{self, Path, PathBuf};  // good: std is a crate name
use crate::foo::baz::foobaz;    // good: foo is at the root of the crate

mod foo {

    pub mod example {
        pub mod iter {}
    }

    use crate::foo::example::iter; // good: foo is at crate root
//  use example::iter;      // bad in 2015 edition: relative paths are not allowed without `self`; good in 2018 edition
    use self::baz::foobaz;  // good: self refers to module 'foo'
    use crate::foo::bar::foobar;   // good: foo is at crate root

    pub mod bar {
        pub fn foobar() { }
    }

    pub mod baz {
        use super::bar::foobar; // good: super refers to module 'foo'
        pub fn foobaz() { }
    }
}

fn main() {}
```

> **Edition Differences**: In the 2015 edition, `use` paths also allow
> accessing items in the crate root. Using the example above, the following
> `use` paths work in 2015 but not 2018:
>
> ```rust,edition2015
> # mod foo {
> #     pub mod example { pub mod iter {} }
> #     pub mod baz { pub fn foobaz() {} }
> # }
> use foo::example::iter;
> use ::foo::baz::foobaz;
> # fn main() {}
> ```
>
> The 2015 edition does not allow use declarations to reference the [extern prelude].
> Thus [`extern crate`] declarations are still required in 2015 to
> reference an external crate in a use declaration. Beginning with the 2018
> edition, use declarations can specify an external crate dependency the same
> way `extern crate` can.
>
> In the 2018 edition, if an in-scope item has the same name as an external
> crate, then `use` of that crate name requires a leading `::` to
> unambiguously select the crate name. This is to retain compatibility with
> potential future changes. <!-- uniform_paths future-proofing -->
>
> ```rust
> // use std::fs; // Error, this is ambiguous.
> use ::std::fs;  // Imports from the `std` crate, not the module below.
> use self::std::fs as self_fs;  // Imports the module below.
>
> mod std {
>     pub mod fs {}
> }
> # fn main() {}
> ```

## Underscore Imports

Items can be imported without binding to a name by using an underscore with
the form `use path as _`. This is particularly useful to import a trait so
that its methods may be used without importing the trait's symbol, for example
if the trait's symbol may conflict with another symbol. Another example is to
link an external crate without importing its name.

Asterisk glob imports will import items imported with `_` in their unnameable
form.

```rust
mod foo {
    pub trait Zoo {
        fn zoo(&self) {}
    }

    impl<T> Zoo for T {}
}

use self::foo::Zoo as _;
struct Zoo;  // Underscore import avoids name conflict with this item.

fn main() {
    let z = Zoo;
    z.zoo();
}
```

The unique, unnameable symbols are created after macro expansion so that
macros may safely emit multiple references to `_` imports. For example, the
following should not produce an error:

```rust
macro_rules! m {
    ($item: item) => { $item $item }
}

m!(use std as _;);
// This expands to:
// use std as _;
// use std as _;
```

[IDENTIFIER]: ../identifiers.md
[_SimplePath_]: ../paths.md#simple-paths
[`extern crate`]: extern-crates.md
[extern prelude]: ../names/preludes.md#extern-prelude
[path qualifiers]: ../paths.md#path-qualifiers
