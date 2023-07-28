# A `no_std` Rust Environment

The term Embedded Programming is used for a wide range of different classes of programming.
Ranging from programming 8-Bit MCUs (like the [ST72325xx](https://www.st.com/resource/en/datasheet/st72325j6.pdf))
with just a few KB of RAM and ROM, up to systems like the Raspberry Pi
([Model B 3+](https://en.wikipedia.org/wiki/Raspberry_Pi#Specifications)) which has a 32/64-bit
4-core Cortex-A53 @ 1.4 GHz and 1GB of RAM. Different restrictions/limitations will apply when writing code
depending on what kind of target and use case you have.

There are two general Embedded Programming classifications:

## Hosted Environments
These kinds of environments are close to a normal PC environment.
What this means is that you are provided with a System Interface [E.G. POSIX](https://en.wikipedia.org/wiki/POSIX)
that provides you with primitives to interact with various systems, such as file systems, networking, memory management, threads, etc.
Standard libraries in turn usually depend on these primitives to implement their functionality.
You may also have some sort of sysroot and restrictions on RAM/ROM-usage, and perhaps some
special HW or I/Os. Overall it feels like coding on a special-purpose PC environment.

## Bare Metal Environments
In a bare metal environment no code has been loaded before your program.
Without the software provided by an OS we can not load the standard library.
Instead the program, along with the crates it uses, can only use the hardware (bare metal) to run.
To prevent rust from loading the standard library use `no_std`.
The platform-agnostic parts of the standard library are available through [libcore](https://doc.rust-lang.org/core/).
libcore also excludes things which are not always desirable in an embedded environment.
One of these things is a memory allocator for dynamic memory allocation.
If you require this or any other functionalities there are often crates which provide these.

### The libstd Runtime
As mentioned before using [libstd](https://doc.rust-lang.org/std/) requires some sort of system integration, but this is not only because
[libstd](https://doc.rust-lang.org/std/) is just providing a common way of accessing OS abstractions, it also provides a runtime.
This runtime, among other things, takes care of setting up stack overflow protection, processing command line arguments,
and spawning the main thread before a program's main function is invoked. This runtime also won't be available in a `no_std` environment.

## Summary
`#![no_std]` is a crate-level attribute that indicates that the crate will link to the core-crate instead of the std-crate.
The [libcore](https://doc.rust-lang.org/core/) crate in turn is a platform-agnostic subset of the std crate
which makes no assumptions about the system the program will run on.
As such, it provides APIs for language primitives like floats, strings and slices, as well as APIs that expose processor features
like atomic operations and SIMD instructions. However it lacks APIs for anything that involves platform integration.
Because of these properties no\_std and [libcore](https://doc.rust-lang.org/core/) code can be used for any kind of
bootstrapping (stage 0) code like bootloaders, firmware or kernels.

### Overview

| feature                                                   | no\_std | std |
|-----------------------------------------------------------|--------|-----|
| heap (dynamic memory)                                     |   *    |  ✓  |
| collections (Vec, BTreeMap, etc)                          |  **    |  ✓  |
| stack overflow protection                                 |   ✘    |  ✓  |
| runs init code before main                                |   ✘    |  ✓  |
| libstd available                                          |   ✘    |  ✓  |
| libcore available                                         |   ✓    |  ✓  |
| writing firmware, kernel, or bootloader code              |   ✓    |  ✘  |

\* Only if you use the `alloc` crate and use a suitable allocator like [alloc-cortex-m].

\** Only if you use the `collections` crate and configure a global default allocator.

\** HashMap and HashSet are not available due to a lack of a secure random number generator.

[alloc-cortex-m]: https://github.com/rust-embedded/alloc-cortex-m

## See Also
* [RFC-1184](https://github.com/rust-lang/rfcs/blob/master/text/1184-stabilize-no_std.md)
