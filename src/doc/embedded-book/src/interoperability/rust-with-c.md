# A little Rust with your C

Using Rust code inside a C or C++ project mostly consists of two parts.

- Creating a C-friendly API in Rust
- Embedding your Rust project into an external build system

Apart from `cargo` and `meson`, most build systems don't have native Rust support.
So you're most likely best off just using `cargo` for compiling your crate and
any dependencies.

## Setting up a project

Create a new `cargo` project as usual.

There are flags to tell `cargo` to emit a systems library, instead of
its regular rust target.
This also allows you to set a different output name for your library,
if you want it to differ from the rest of your crate.

```toml
[lib]
name = "your_crate"
crate-type = ["cdylib"]      # Creates dynamic lib
# crate-type = ["staticlib"] # Creates static lib
```

## Building a `C` API

Because C++ has no stable ABI for the Rust compiler to target, we use `C` for
any interoperability between different languages. This is no exception when using Rust
inside of C and C++ code.

### `#[no_mangle]`

The Rust compiler mangles symbol names differently than native code linkers expect.
As such, any function that Rust exports to be used outside of Rust needs to be told
not to be mangled by the compiler.

### `extern "C"`

By default, any function you write in Rust will use the
Rust ABI (which is also not stabilized).
Instead, when building outwards facing FFI APIs we need to
tell the compiler to use the system ABI.

Depending on your platform, you might want to target a specific ABI version, which are
documented [here](https://doc.rust-lang.org/reference/items/external-blocks.html).

---

Putting these parts together, you get a function that looks roughly like this.

```rust,ignore
#[no_mangle]
pub extern "C" fn rust_function() {

}
```

Just as when using `C` code in your Rust project you now need to transform data
from and to a form that the rest of the application will understand.

## Linking and greater project context.

So then, that's one half of the problem solved.
How do you use this now?

**This very much depends on your project and/or build system**

`cargo` will create a `my_lib.so`/`my_lib.dll` or `my_lib.a` file,
depending on your platform and settings. This library can simply be linked
by your build system.

However, calling a Rust function from C requires a header file to declare
the function signatures.

Every function in your Rust-ffi API needs to have a corresponding header function.

```rust,ignore
#[no_mangle]
pub extern "C" fn rust_function() {}
```

would then become

```C
void rust_function();
```

etc.

There is a tool to automate this process,
called [cbindgen] which analyses your Rust code
and then generates headers for your C and C++ projects from it.

[cbindgen]: https://github.com/eqrion/cbindgen

At this point, using the Rust functions from C
is as simple as including the header and calling them!

```C
#include "my-rust-project.h"
rust_function();
```
