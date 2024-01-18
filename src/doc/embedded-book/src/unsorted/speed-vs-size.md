# Optimizations: the speed size tradeoff

Everyone wants their program to be super fast and super small but it's usually
not possible to have both characteristics. This section discusses the
different optimization levels that `rustc` provides and how they affect the
execution time and binary size of a program.

## No optimizations

This is the default. When you call `cargo build` you use the development (AKA
`dev`) profile. This profile is optimized for debugging so it enables debug
information and does *not* enable any optimizations, i.e. it uses `-C opt-level
= 0`.

At least for bare metal development, debuginfo is zero cost in the sense that it
won't occupy space in Flash / ROM so we actually recommend that you enable
debuginfo in the release profile -- it is disabled by default. That will let you
use breakpoints when debugging release builds.

``` toml
[profile.release]
# symbols are nice and they don't increase the size on Flash
debug = true
```

No optimizations is great for debugging because stepping through the code feels
like you are executing the program statement by statement, plus you can `print`
stack variables and function arguments in GDB. When the code is optimized, trying
to print variables results in `$0 = <value optimized out>` being printed.

The biggest downside of the `dev` profile is that the resulting binary will be
huge and slow. The size is usually more of a problem because unoptimized
binaries can occupy dozens of KiB of Flash, which your target device may not
have -- the result: your unoptimized binary doesn't fit in your device!

Can we have smaller, debugger friendly binaries? Yes, there's a trick.

### Optimizing dependencies

There's a Cargo feature named [`profile-overrides`] that lets you
override the optimization level of dependencies. You can use that feature to
optimize all dependencies for size while keeping the top crate unoptimized and
debugger friendly.

[`profile-overrides`]: https://doc.rust-lang.org/cargo/reference/profiles.html#overrides

Here's an example:

``` toml
# Cargo.toml
[package]
name = "app"
# ..

[profile.dev.package."*"] # +
opt-level = "z" # +
```

Without the override:

``` text
$ cargo size --bin app -- -A
app  :
section               size        addr
.vector_table         1024   0x8000000
.text                 9060   0x8000400
.rodata               1708   0x8002780
.data                    0  0x20000000
.bss                     4  0x20000000
```

With the override:

``` text
$ cargo size --bin app -- -A
app  :
section               size        addr
.vector_table         1024   0x8000000
.text                 3490   0x8000400
.rodata               1100   0x80011c0
.data                    0  0x20000000
.bss                     4  0x20000000
```

That's a 6 KiB reduction in Flash usage without any loss in the debuggability of
the top crate. If you step into a dependency then you'll start seeing those
`<value optimized out>` messages again but it's usually the case that you want
to debug the top crate and not the dependencies. And if you *do* need to debug a
dependency then you can use the `profile-overrides` feature to exclude a
particular dependency from being optimized. See example below:

``` toml
# ..

# don't optimize the `cortex-m-rt` crate
[profile.dev.package.cortex-m-rt] # +
opt-level = 0 # +

# but do optimize all the other dependencies
[profile.dev.package."*"]
codegen-units = 1 # better optimizations
opt-level = "z"
```

Now the top crate and `cortex-m-rt` are debugger friendly!

## Optimize for speed

As of 2018-09-18 `rustc` supports three "optimize for speed" levels: `opt-level
= 1`, `2` and `3`. When you run `cargo build --release` you are using the release
profile which defaults to `opt-level = 3`.

Both `opt-level = 2` and `3` optimize for speed at the expense of binary size,
but level `3` does more vectorization and inlining than level `2`. In
particular, you'll see that at `opt-level` equal to or greater than `2` LLVM will
unroll loops. Loop unrolling has a rather high cost in terms of Flash / ROM
(e.g. from 26 bytes to 194 for a zero this array loop) but can also halve the
execution time given the right conditions (e.g. number of iterations is big
enough).

Currently there's no way to disable loop unrolling in `opt-level = 2` and `3` so
if you can't afford its cost you should optimize your program for size.

## Optimize for size

As of 2018-09-18 `rustc` supports two "optimize for size" levels: `opt-level =
"s"` and `"z"`. These names were inherited from clang / LLVM and are not too
descriptive but `"z"` is meant to give the idea that it produces smaller
binaries than `"s"`.

If you want your release binaries to be optimized for size then change the
`profile.release.opt-level` setting in `Cargo.toml` as shown below.

``` toml
[profile.release]
# or "z"
opt-level = "s"
```

These two optimization levels greatly reduce LLVM's inline threshold, a metric
used to decide whether to inline a function or not. One of Rust principles are
zero cost abstractions; these abstractions tend to use a lot of newtypes and
small functions to hold invariants (e.g. functions that borrow an inner value
like `deref`, `as_ref`) so a low inline threshold can make LLVM miss
optimization opportunities (e.g. eliminate dead branches, inline calls to
closures).

When optimizing for size you may want to try increasing the inline threshold to
see if that has any effect on the binary size. The recommended way to change the
inline threshold is to append the `-C inline-threshold` flag to the other
rustflags in `.cargo/config.toml`.

``` toml
# .cargo/config.toml
# this assumes that you are using the cortex-m-quickstart template
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
rustflags = [
  # ..
  "-C", "inline-threshold=123", # +
]
```

What value to use? [As of 1.29.0 these are the inline thresholds that the
different optimization levels use][inline-threshold]:

[inline-threshold]: https://github.com/rust-lang/rust/blob/1.29.0/src/librustc_codegen_llvm/back/write.rs#L2105-L2122

- `opt-level = 3` uses 275
- `opt-level = 2` uses 225
- `opt-level = "s"` uses 75
- `opt-level = "z"` uses 25

You should try `225` and `275` when optimizing for size.
