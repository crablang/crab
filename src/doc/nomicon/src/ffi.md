# Foreign Function Interface

## Introduction

This guide will use the [snappy](https://github.com/google/snappy)
compression/decompression library as an introduction to writing bindings for
foreign code. Rust is currently unable to call directly into a C++ library, but
snappy includes a C interface (documented in
[`snappy-c.h`](https://github.com/google/snappy/blob/master/snappy-c.h)).

## A note about libc

Many of these examples use [the `libc` crate][libc], which provides various
type definitions for C types, among other things. If you’re trying these
examples yourself, you’ll need to add `libc` to your `Cargo.toml`:

```toml
[dependencies]
libc = "0.2.0"
```

[libc]: https://crates.io/crates/libc

## Calling foreign functions

The following is a minimal example of calling a foreign function which will
compile if snappy is installed:

<!-- ignore: requires libc crate -->
```rust,ignore
use libc::size_t;

#[link(name = "snappy")]
extern {
    fn snappy_max_compressed_length(source_length: size_t) -> size_t;
}

fn main() {
    let x = unsafe { snappy_max_compressed_length(100) };
    println!("max compressed length of a 100 byte buffer: {}", x);
}
```

The `extern` block is a list of function signatures in a foreign library, in
this case with the platform's C ABI. The `#[link(...)]` attribute is used to
instruct the linker to link against the snappy library so the symbols are
resolved.

Foreign functions are assumed to be unsafe so calls to them need to be wrapped
with `unsafe {}` as a promise to the compiler that everything contained within
truly is safe. C libraries often expose interfaces that aren't thread-safe, and
almost any function that takes a pointer argument isn't valid for all possible
inputs since the pointer could be dangling, and raw pointers fall outside of
Rust's safe memory model.

When declaring the argument types to a foreign function, the Rust compiler
cannot check if the declaration is correct, so specifying it correctly is part
of keeping the binding correct at runtime.

The `extern` block can be extended to cover the entire snappy API:

<!-- ignore: requires libc crate -->
```rust,ignore
use libc::{c_int, size_t};

#[link(name = "snappy")]
extern {
    fn snappy_compress(input: *const u8,
                       input_length: size_t,
                       compressed: *mut u8,
                       compressed_length: *mut size_t) -> c_int;
    fn snappy_uncompress(compressed: *const u8,
                         compressed_length: size_t,
                         uncompressed: *mut u8,
                         uncompressed_length: *mut size_t) -> c_int;
    fn snappy_max_compressed_length(source_length: size_t) -> size_t;
    fn snappy_uncompressed_length(compressed: *const u8,
                                  compressed_length: size_t,
                                  result: *mut size_t) -> c_int;
    fn snappy_validate_compressed_buffer(compressed: *const u8,
                                         compressed_length: size_t) -> c_int;
}
# fn main() {}
```

## Creating a safe interface

The raw C API needs to be wrapped to provide memory safety and make use of higher-level concepts
like vectors. A library can choose to expose only the safe, high-level interface and hide the unsafe
internal details.

Wrapping the functions which expect buffers involves using the `slice::raw` module to manipulate Rust
vectors as pointers to memory. Rust's vectors are guaranteed to be a contiguous block of memory. The
length is the number of elements currently contained, and the capacity is the total size in elements of
the allocated memory. The length is less than or equal to the capacity.

<!-- ignore: requires libc crate -->
```rust,ignore
# use libc::{c_int, size_t};
# unsafe fn snappy_validate_compressed_buffer(_: *const u8, _: size_t) -> c_int { 0 }
# fn main() {}
pub fn validate_compressed_buffer(src: &[u8]) -> bool {
    unsafe {
        snappy_validate_compressed_buffer(src.as_ptr(), src.len() as size_t) == 0
    }
}
```

The `validate_compressed_buffer` wrapper above makes use of an `unsafe` block, but it makes the
guarantee that calling it is safe for all inputs by leaving off `unsafe` from the function
signature.

The `snappy_compress` and `snappy_uncompress` functions are more complex, since a buffer has to be
allocated to hold the output too.

The `snappy_max_compressed_length` function can be used to allocate a vector with the maximum
required capacity to hold the compressed output. The vector can then be passed to the
`snappy_compress` function as an output parameter. An output parameter is also passed to retrieve
the true length after compression for setting the length.

<!-- ignore: requires libc crate -->
```rust,ignore
# use libc::{size_t, c_int};
# unsafe fn snappy_compress(a: *const u8, b: size_t, c: *mut u8,
#                           d: *mut size_t) -> c_int { 0 }
# unsafe fn snappy_max_compressed_length(a: size_t) -> size_t { a }
# fn main() {}
pub fn compress(src: &[u8]) -> Vec<u8> {
    unsafe {
        let srclen = src.len() as size_t;
        let psrc = src.as_ptr();

        let mut dstlen = snappy_max_compressed_length(srclen);
        let mut dst = Vec::with_capacity(dstlen as usize);
        let pdst = dst.as_mut_ptr();

        snappy_compress(psrc, srclen, pdst, &mut dstlen);
        dst.set_len(dstlen as usize);
        dst
    }
}
```

Decompression is similar, because snappy stores the uncompressed size as part of the compression
format and `snappy_uncompressed_length` will retrieve the exact buffer size required.

<!-- ignore: requires libc crate -->
```rust,ignore
# use libc::{size_t, c_int};
# unsafe fn snappy_uncompress(compressed: *const u8,
#                             compressed_length: size_t,
#                             uncompressed: *mut u8,
#                             uncompressed_length: *mut size_t) -> c_int { 0 }
# unsafe fn snappy_uncompressed_length(compressed: *const u8,
#                                      compressed_length: size_t,
#                                      result: *mut size_t) -> c_int { 0 }
# fn main() {}
pub fn uncompress(src: &[u8]) -> Option<Vec<u8>> {
    unsafe {
        let srclen = src.len() as size_t;
        let psrc = src.as_ptr();

        let mut dstlen: size_t = 0;
        snappy_uncompressed_length(psrc, srclen, &mut dstlen);

        let mut dst = Vec::with_capacity(dstlen as usize);
        let pdst = dst.as_mut_ptr();

        if snappy_uncompress(psrc, srclen, pdst, &mut dstlen) == 0 {
            dst.set_len(dstlen as usize);
            Some(dst)
        } else {
            None // SNAPPY_INVALID_INPUT
        }
    }
}
```

Then, we can add some tests to show how to use them.

<!-- ignore: requires libc crate -->
```rust,ignore
# use libc::{c_int, size_t};
# unsafe fn snappy_compress(input: *const u8,
#                           input_length: size_t,
#                           compressed: *mut u8,
#                           compressed_length: *mut size_t)
#                           -> c_int { 0 }
# unsafe fn snappy_uncompress(compressed: *const u8,
#                             compressed_length: size_t,
#                             uncompressed: *mut u8,
#                             uncompressed_length: *mut size_t)
#                             -> c_int { 0 }
# unsafe fn snappy_max_compressed_length(source_length: size_t) -> size_t { 0 }
# unsafe fn snappy_uncompressed_length(compressed: *const u8,
#                                      compressed_length: size_t,
#                                      result: *mut size_t)
#                                      -> c_int { 0 }
# unsafe fn snappy_validate_compressed_buffer(compressed: *const u8,
#                                             compressed_length: size_t)
#                                             -> c_int { 0 }
# fn main() { }
#
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let d = vec![0xde, 0xad, 0xd0, 0x0d];
        let c: &[u8] = &compress(&d);
        assert!(validate_compressed_buffer(c));
        assert!(uncompress(c) == Some(d));
    }

    #[test]
    fn invalid() {
        let d = vec![0, 0, 0, 0];
        assert!(!validate_compressed_buffer(&d));
        assert!(uncompress(&d).is_none());
    }

    #[test]
    fn empty() {
        let d = vec![];
        assert!(!validate_compressed_buffer(&d));
        assert!(uncompress(&d).is_none());
        let c = compress(&d);
        assert!(validate_compressed_buffer(&c));
        assert!(uncompress(&c) == Some(d));
    }
}
```

## Destructors

Foreign libraries often hand off ownership of resources to the calling code.
When this occurs, we must use Rust's destructors to provide safety and guarantee
the release of these resources (especially in the case of panic).

For more about destructors, see the [Drop trait](../std/ops/trait.Drop.html).

## Calling Rust code from C

You may wish to compile Rust code in a way so that it can be called from C.
This is fairly easy, but requires a few things.

### Rust side

First, we assume you have a lib crate named as `rust_from_c`.
`lib.rs` should have Rust code as following:

```rust
#[no_mangle]
pub extern "C" fn hello_from_rust() {
    println!("Hello from Rust!");
}
# fn main() {}
```

The `extern "C"` makes this function adhere to the C calling convention, as discussed below in "[Foreign Calling Conventions]".
The `no_mangle` attribute turns off Rust's name mangling, so that it has a well defined symbol to link to.

Then, to compile Rust code as a shared library that can be called from C, add the following to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]
```

(NOTE: We could also use the `staticlib` crate type but it needs to tweak some linking flags.)

Run `cargo build` and you're ready to go on the Rust side.

[Foreign Calling Conventions]: ffi.md#foreign-calling-conventions

### C side

We'll create a C file to call the `hello_from_rust` function and compile it by `gcc`.

C file should look like:

```c
extern void hello_from_rust();

int main(void) {
    hello_from_rust();
    return 0;
}
```

We name the file as `call_rust.c` and place it on the crate root.
Run the following to compile:

```sh
gcc call_rust.c -o call_rust -lrust_from_c -L./target/debug
```

`-l` and `-L` tell gcc to find our Rust library.

Finally, we can call Rust code from C with `LD_LIBRARY_PATH` specified:

```sh
$ LD_LIBRARY_PATH=./target/debug ./call_rust
Hello from Rust!
```

That's it!
For more realistic example, check the [`cbindgen`].

[`cbindgen`]: https://github.com/eqrion/cbindgen

## Callbacks from C code to Rust functions

Some external libraries require the usage of callbacks to report back their
current state or intermediate data to the caller.
It is possible to pass functions defined in Rust to an external library.
The requirement for this is that the callback function is marked as `extern`
with the correct calling convention to make it callable from C code.

The callback function can then be sent through a registration call
to the C library and afterwards be invoked from there.

A basic example is:

Rust code:

```rust,no_run
extern fn callback(a: i32) {
    println!("I'm called from C with value {0}", a);
}

#[link(name = "extlib")]
extern {
   fn register_callback(cb: extern fn(i32)) -> i32;
   fn trigger_callback();
}

fn main() {
    unsafe {
        register_callback(callback);
        trigger_callback(); // Triggers the callback.
    }
}
```

C code:

```c
typedef void (*rust_callback)(int32_t);
rust_callback cb;

int32_t register_callback(rust_callback callback) {
    cb = callback;
    return 1;
}

void trigger_callback() {
  cb(7); // Will call callback(7) in Rust.
}
```

In this example Rust's `main()` will call `trigger_callback()` in C,
which would, in turn, call back to `callback()` in Rust.

## Targeting callbacks to Rust objects

The former example showed how a global function can be called from C code.
However it is often desired that the callback is targeted to a special
Rust object. This could be the object that represents the wrapper for the
respective C object.

This can be achieved by passing a raw pointer to the object down to the
C library. The C library can then include the pointer to the Rust object in
the notification. This will allow the callback to unsafely access the
referenced Rust object.

Rust code:

```rust,no_run
struct RustObject {
    a: i32,
    // Other members...
}

extern "C" fn callback(target: *mut RustObject, a: i32) {
    println!("I'm called from C with value {0}", a);
    unsafe {
        // Update the value in RustObject with the value received from the callback:
        (*target).a = a;
    }
}

#[link(name = "extlib")]
extern {
   fn register_callback(target: *mut RustObject,
                        cb: extern fn(*mut RustObject, i32)) -> i32;
   fn trigger_callback();
}

fn main() {
    // Create the object that will be referenced in the callback:
    let mut rust_object = Box::new(RustObject { a: 5 });

    unsafe {
        register_callback(&mut *rust_object, callback);
        trigger_callback();
    }
}
```

C code:

```c
typedef void (*rust_callback)(void*, int32_t);
void* cb_target;
rust_callback cb;

int32_t register_callback(void* callback_target, rust_callback callback) {
    cb_target = callback_target;
    cb = callback;
    return 1;
}

void trigger_callback() {
  cb(cb_target, 7); // Will call callback(&rustObject, 7) in Rust.
}
```

## Asynchronous callbacks

In the previously given examples the callbacks are invoked as a direct reaction
to a function call to the external C library.
The control over the current thread is switched from Rust to C to Rust for the
execution of the callback, but in the end the callback is executed on the
same thread that called the function which triggered the callback.

Things get more complicated when the external library spawns its own threads
and invokes callbacks from there.
In these cases access to Rust data structures inside the callbacks is
especially unsafe and proper synchronization mechanisms must be used.
Besides classical synchronization mechanisms like mutexes, one possibility in
Rust is to use channels (in `std::sync::mpsc`) to forward data from the C
thread that invoked the callback into a Rust thread.

If an asynchronous callback targets a special object in the Rust address space
it is also absolutely necessary that no more callbacks are performed by the
C library after the respective Rust object gets destroyed.
This can be achieved by unregistering the callback in the object's
destructor and designing the library in a way that guarantees that no
callback will be performed after deregistration.

## Linking

The `link` attribute on `extern` blocks provides the basic building block for
instructing rustc how it will link to native libraries. There are two accepted
forms of the link attribute today:

* `#[link(name = "foo")]`
* `#[link(name = "foo", kind = "bar")]`

In both of these cases, `foo` is the name of the native library that we're
linking to, and in the second case `bar` is the type of native library that the
compiler is linking to. There are currently three known types of native
libraries:

* Dynamic - `#[link(name = "readline")]`
* Static - `#[link(name = "my_build_dependency", kind = "static")]`
* Frameworks - `#[link(name = "CoreFoundation", kind = "framework")]`

Note that frameworks are only available on macOS targets.

The different `kind` values are meant to differentiate how the native library
participates in linkage. From a linkage perspective, the Rust compiler creates
two flavors of artifacts: partial (rlib/staticlib) and final (dylib/binary).
Native dynamic library and framework dependencies are propagated to the final
artifact boundary, while static library dependencies are not propagated at
all, because the static libraries are integrated directly into the subsequent
artifact.

A few examples of how this model can be used are:

* A native build dependency. Sometimes some C/C++ glue is needed when writing
  some Rust code, but distribution of the C/C++ code in a library format is
  a burden. In this case, the code will be archived into `libfoo.a` and then the
  Rust crate would declare a dependency via `#[link(name = "foo", kind =
  "static")]`.

  Regardless of the flavor of output for the crate, the native static library
  will be included in the output, meaning that distribution of the native static
  library is not necessary.

* A normal dynamic dependency. Common system libraries (like `readline`) are
  available on a large number of systems, and often a static copy of these
  libraries cannot be found. When this dependency is included in a Rust crate,
  partial targets (like rlibs) will not link to the library, but when the rlib
  is included in a final target (like a binary), the native library will be
  linked in.

On macOS, frameworks behave with the same semantics as a dynamic library.

## Unsafe blocks

Some operations, like dereferencing raw pointers or calling functions that have been marked
unsafe are only allowed inside unsafe blocks. Unsafe blocks isolate unsafety and are a promise to
the compiler that the unsafety does not leak out of the block.

Unsafe functions, on the other hand, advertise it to the world. An unsafe function is written like
this:

```rust
unsafe fn kaboom(ptr: *const i32) -> i32 { *ptr }
```

This function can only be called from an `unsafe` block or another `unsafe` function.

## Accessing foreign globals

Foreign APIs often export a global variable which could do something like track
global state. In order to access these variables, you declare them in `extern`
blocks with the `static` keyword:

<!-- ignore: requires libc crate -->
```rust,ignore
#[link(name = "readline")]
extern {
    static rl_readline_version: libc::c_int;
}

fn main() {
    println!("You have readline version {} installed.",
             unsafe { rl_readline_version as i32 });
}
```

Alternatively, you may need to alter global state provided by a foreign
interface. To do this, statics can be declared with `mut` so we can mutate
them.

<!-- ignore: requires libc crate -->
```rust,ignore
use std::ffi::CString;
use std::ptr;

#[link(name = "readline")]
extern {
    static mut rl_prompt: *const libc::c_char;
}

fn main() {
    let prompt = CString::new("[my-awesome-shell] $").unwrap();
    unsafe {
        rl_prompt = prompt.as_ptr();

        println!("{:?}", rl_prompt);

        rl_prompt = ptr::null();
    }
}
```

Note that all interaction with a `static mut` is unsafe, both reading and
writing. Dealing with global mutable state requires a great deal of care.

## Foreign calling conventions

Most foreign code exposes a C ABI, and Rust uses the platform's C calling convention by default when
calling foreign functions. Some foreign functions, most notably the Windows API, use other calling
conventions. Rust provides a way to tell the compiler which convention to use:

<!-- ignore: requires libc crate -->
```rust,ignore
#[cfg(all(target_os = "win32", target_arch = "x86"))]
#[link(name = "kernel32")]
#[allow(non_snake_case)]
extern "stdcall" {
    fn SetEnvironmentVariableA(n: *const u8, v: *const u8) -> libc::c_int;
}
# fn main() { }
```

This applies to the entire `extern` block. The list of supported ABI constraints
are:

* `stdcall`
* `aapcs`
* `cdecl`
* `fastcall`
* `thiscall`
* `vectorcall`
This is currently hidden behind the `abi_vectorcall` gate and is subject to change.
* `Rust`
* `rust-intrinsic`
* `system`
* `C`
* `win64`
* `sysv64`

Most of the abis in this list are self-explanatory, but the `system` abi may
seem a little odd. This constraint selects whatever the appropriate ABI is for
interoperating with the target's libraries. For example, on win32 with a x86
architecture, this means that the abi used would be `stdcall`. On x86_64,
however, windows uses the `C` calling convention, so `C` would be used. This
means that in our previous example, we could have used `extern "system" { ... }`
to define a block for all windows systems, not only x86 ones.

## Interoperability with foreign code

Rust guarantees that the layout of a `struct` is compatible with the platform's
representation in C only if the `#[repr(C)]` attribute is applied to it.
`#[repr(C, packed)]` can be used to lay out struct members without padding.
`#[repr(C)]` can also be applied to an enum.

Rust's owned boxes (`Box<T>`) use non-nullable pointers as handles which point
to the contained object. However, they should not be manually created because
they are managed by internal allocators. References can safely be assumed to be
non-nullable pointers directly to the type.  However, breaking the borrow
checking or mutability rules is not guaranteed to be safe, so prefer using raw
pointers (`*`) if that's needed because the compiler can't make as many
assumptions about them.

Vectors and strings share the same basic memory layout, and utilities are
available in the `vec` and `str` modules for working with C APIs. However,
strings are not terminated with `\0`. If you need a NUL-terminated string for
interoperability with C, you should use the `CString` type in the `std::ffi`
module.

The [`libc` crate on crates.io][libc] includes type aliases and function
definitions for the C standard library in the `libc` module, and Rust links
against `libc` and `libm` by default.

## Variadic functions

In C, functions can be 'variadic', meaning they accept a variable number of arguments. This can
be achieved in Rust by specifying `...` within the argument list of a foreign function declaration:

```no_run
extern {
    fn foo(x: i32, ...);
}

fn main() {
    unsafe {
        foo(10, 20, 30, 40, 50);
    }
}
```

Normal Rust functions can *not* be variadic:

```rust,compile_fail
// This will not compile

fn foo(x: i32, ...) {}
```

## The "nullable pointer optimization"

Certain Rust types are defined to never be `null`. This includes references (`&T`,
`&mut T`), boxes (`Box<T>`), and function pointers (`extern "abi" fn()`). When
interfacing with C, pointers that might be `null` are often used, which would seem to
require some messy `transmute`s and/or unsafe code to handle conversions to/from Rust types.
However, trying to construct/work with these invalid values **is undefined behavior**,
so you should use the following workaround instead.

As a special case, an `enum` is eligible for the "nullable pointer optimization" if it contains
exactly two variants, one of which contains no data and the other contains a field of one of the
non-nullable types listed above.  This means no extra space is required for a discriminant; rather,
the empty variant is represented by putting a `null` value into the non-nullable field. This is
called an "optimization", but unlike other optimizations it is guaranteed to apply to eligible
types.

The most common type that takes advantage of the nullable pointer optimization is `Option<T>`,
where `None` corresponds to `null`. So `Option<extern "C" fn(c_int) -> c_int>` is a correct way
to represent a nullable function pointer using the C ABI (corresponding to the C type
`int (*)(int)`).

Here is a contrived example. Let's say some C library has a facility for registering a
callback, which gets called in certain situations. The callback is passed a function pointer
and an integer and it is supposed to run the function with the integer as a parameter. So
we have function pointers flying across the FFI boundary in both directions.

<!-- ignore: requires libc crate -->
```rust,ignore
use libc::c_int;

# #[cfg(hidden)]
extern "C" {
    /// Registers the callback.
    fn register(cb: Option<extern "C" fn(Option<extern "C" fn(c_int) -> c_int>, c_int) -> c_int>);
}
# unsafe fn register(_: Option<extern "C" fn(Option<extern "C" fn(c_int) -> c_int>,
#                                            c_int) -> c_int>)
# {}

/// This fairly useless function receives a function pointer and an integer
/// from C, and returns the result of calling the function with the integer.
/// In case no function is provided, it squares the integer by default.
extern "C" fn apply(process: Option<extern "C" fn(c_int) -> c_int>, int: c_int) -> c_int {
    match process {
        Some(f) => f(int),
        None    => int * int
    }
}

fn main() {
    unsafe {
        register(Some(apply));
    }
}
```

And the code on the C side looks like this:

```c
void register(int (*f)(int (*)(int), int)) {
    ...
}
```

No `transmute` required!

## FFI and unwinding

It’s important to be mindful of unwinding when working with FFI. Most
ABI strings come in two variants, one with an `-unwind` suffix and one without.
The `Rust` ABI always permits unwinding, so there is no `Rust-unwind` ABI.

If you expect Rust `panic`s or foreign (e.g. C++) exceptions to cross an FFI
boundary, that boundary must use the appropriate `-unwind` ABI string.
Conversely, if you do not expect unwinding to cross an ABI boundary, use one of
the non-`unwind` ABI strings.

> Note: Compiling with `panic=abort` will still cause `panic!` to immediately
abort the process, regardless of which ABI is specified by the function that
`panic`s.

If an unwinding operation does encounter an ABI boundary that is
not permitted to unwind, the behavior depends on the source of the unwinding
(Rust `panic` or a foreign exception):

* `panic` will cause the process to safely abort.
* A foreign exception entering Rust will cause undefined behavior.

Note that the interaction of `catch_unwind` with foreign exceptions **is
undefined**, as is the interaction of `panic` with foreign exception-catching
mechanisms (notably C++'s `try`/`catch`).

### Rust `panic` with `"C-unwind"`

<!-- ignore: using unstable feature -->
```rust,ignore
#[no_mangle]
extern "C-unwind" fn example() {
    panic!("Uh oh");
}
```

This function (when compiled with `panic=unwind`) is permitted to unwind C++
stack frames.

```text
[Rust function with `catch_unwind`, which stops the unwinding]
      |
     ...
      |
[C++ frames]
      |                           ^
      | (calls)                   | (unwinding
      v                           |  goes this
[Rust function `example`]         |  way)
      |                           |
      +--- rust function panics --+
```

If the C++ frames have objects, their destructors will be called.

### C++ `throw` with `"C-unwind"`

<!-- ignore: using unstable feature -->
```rust,ignore
#[link(...)]
extern "C-unwind" {
    // A C++ function that may throw an exception
    fn may_throw();
}

#[no_mangle]
extern "C-unwind" fn rust_passthrough() {
    let b = Box::new(5);
    unsafe { may_throw(); }
    println!("{:?}", &b);
}
```

A C++ function with a `try` block may invoke `rust_passthrough` and `catch` an
exception thrown by `may_throw`.

```text
[C++ function with `try` block that invokes `rust_passthrough`]
      |
     ...
      |
[Rust function `rust_passthrough`]
      |                            ^
      | (calls)                    | (unwinding
      v                            |  goes this
[C++ function `may_throw`]         |  way)
      |                            |
      +--- C++ function throws ----+
```

If `may_throw` does throw an exception, `b` will be dropped. Otherwise, `5`
will be printed.

### `panic` can be stopped at an ABI boundary

```rust
#[no_mangle]
extern "C" fn assert_nonzero(input: u32) {
    assert!(input != 0)
}
```

If `assert_nonzero` is called with the argument `0`, the runtime is guaranteed
to (safely) abort the process, whether or not compiled with `panic=abort`.

### Catching `panic` preemptively

If you are writing Rust code that may panic, and you don't wish to abort the
process if it panics, you must use [`catch_unwind`]:

```rust
use std::panic::catch_unwind;

#[no_mangle]
pub extern "C" fn oh_no() -> i32 {
    let result = catch_unwind(|| {
        panic!("Oops!");
    });
    match result {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

fn main() {}
```

Please note that [`catch_unwind`] will only catch unwinding panics, not
those that abort the process. See the documentation of [`catch_unwind`]
for more information.

[`catch_unwind`]: ../std/panic/fn.catch_unwind.html

## Representing opaque structs

Sometimes, a C library wants to provide a pointer to something, but not let you know the internal details of the thing it wants.
A stable and simple way is to use a `void *` argument:

```c
void foo(void *arg);
void bar(void *arg);
```

We can represent this in Rust with the `c_void` type:

<!-- ignore: requires libc crate -->
```rust,ignore
extern "C" {
    pub fn foo(arg: *mut libc::c_void);
    pub fn bar(arg: *mut libc::c_void);
}
# fn main() {}
```

This is a perfectly valid way of handling the situation. However, we can do a bit
better. To solve this, some C libraries will instead create a `struct`, where
the details and memory layout of the struct are private. This gives some amount
of type safety. These structures are called ‘opaque’. Here’s an example, in C:

```c
struct Foo; /* Foo is a structure, but its contents are not part of the public interface */
struct Bar;
void foo(struct Foo *arg);
void bar(struct Bar *arg);
```

To do this in Rust, let’s create our own opaque types:

```rust
#[repr(C)]
pub struct Foo {
    _data: [u8; 0],
    _marker:
        core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
#[repr(C)]
pub struct Bar {
    _data: [u8; 0],
    _marker:
        core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

extern "C" {
    pub fn foo(arg: *mut Foo);
    pub fn bar(arg: *mut Bar);
}
# fn main() {}
```

By including at least one private field and no constructor,
we create an opaque type that we can't instantiate outside of this module.
(A struct with no field could be instantiated by anyone.)
We also want to use this type in FFI, so we have to add `#[repr(C)]`.
The marker ensures the compiler does not mark the struct as `Send`, `Sync` and `Unpin` are
not applied to the struct. (`*mut u8` is not `Send` or `Sync`, `PhantomPinned` is not `Unpin`)

But because our `Foo` and `Bar` types are
different, we’ll get type safety between the two of them, so we cannot
accidentally pass a pointer to `Foo` to `bar()`.

Notice that it is a really bad idea to use an empty enum as FFI type.
The compiler relies on empty enums being uninhabited, so handling values of type
`&Empty` is a huge footgun and can lead to buggy program behavior (by triggering
undefined behavior).

> **NOTE:** The simplest way would use "extern types".
But it's currently (as of June 2021) unstable and has some unresolved questions, see the [RFC page][extern-type-rfc] and the [tracking issue][extern-type-issue] for more details.

[extern-type-issue]: https://github.com/rust-lang/rust/issues/43467
[extern-type-rfc]: https://rust-lang.github.io/rfcs/1861-extern-types.html
