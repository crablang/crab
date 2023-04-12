Version 1.68.2 (2023-03-28)
===========================

- [Update the GitHub RSA host key bundled within Cargo](https://github.com/crablang/cargo/pull/11883).
  The key was [rotated by GitHub](https://github.blog/2023-03-23-we-updated-our-rsa-ssh-host-key/)
  on 2023-03-24 after the old one leaked.
- [Mark the old GitHub RSA host key as revoked](https://github.com/crablang/cargo/pull/11889).
  This will prevent Cargo from accepting the leaked key even when tcrablanged by
  the system.
- [Add support for `@revoked` and a better error message for `@cert-authority` in Cargo's SSH host key verification](https://github.com/crablang/cargo/pull/11635)

Version 1.68.1 (2023-03-23)
===========================

- [Fix miscompilation in produced Windows MSVC artifacts](https://github.com/crablang/crablang/pull/109094)
  This was introduced by enabling ThinLTO for the distributed crablangc which led
  to miscompilations in the resulting binary. Currently this is believed to be
  limited to the -Zdylib-lto flag used for crablangc compilation, rather than a
  general bug in ThinLTO, so only crablangc artifacts should be affected.
- [Fix --enable-local-crablang builds](https://github.com/crablang/crablang/pull/109111/)
- [Treat `$prefix-clang` as `clang` in linker detection code](https://github.com/crablang/crablang/pull/109156)
- [Fix panic in compiler code](https://github.com/crablang/crablang/pull/108162)

Version 1.68.0 (2023-03-09)
==========================

<a id="1.68.0-Language"></a>

Language
--------

- [Stabilize default_alloc_error_handler](https://github.com/crablang/crablang/pull/102318/)
  This allows usage of `alloc` on stable without requiring the 
  definition of a handler for allocation failure. Defining custom handlers is still unstable.
- [Stabilize `efiapi` calling convention.](https://github.com/crablang/crablang/pull/105795/)
- [Remove implicit promotion for types with drop glue](https://github.com/crablang/crablang/pull/105085/)

<a id="1.68.0-Compiler"></a>

Compiler
--------

- [Change `bindings_with_variant_name` to deny-by-default](https://github.com/crablang/crablang/pull/104154/)
- [Allow .. to be parsed as let initializer](https://github.com/crablang/crablang/pull/105701/)
- [Add `armv7-sony-vita-newlibeabihf` as a tier 3 target](https://github.com/crablang/crablang/pull/105712/)
- [Always check alignment during compile-time const evaluation](https://github.com/crablang/crablang/pull/104616/)
- [Disable "split dwarf inlining" by default.](https://github.com/crablang/crablang/pull/106709/)
- [Add vendor to Fuchsia's target triple](https://github.com/crablang/crablang/pull/106429/)
- [Enable sanitizers for s390x-linux](https://github.com/crablang/crablang/pull/107127/)

<a id="1.68.0-Libraries"></a>

Libraries
---------

- [Loosen the bound on the Debug implementation of Weak.](https://github.com/crablang/crablang/pull/90291/)
- [Make `std::task::Context` !Send and !Sync](https://github.com/crablang/crablang/pull/95985/)
- [PhantomData layout guarantees](https://github.com/crablang/crablang/pull/104081/)
- [Don't derive Debug for `OnceWith` & `RepeatWith`](https://github.com/crablang/crablang/pull/104163/)
- [Implement DerefMut for PathBuf](https://github.com/crablang/crablang/pull/105018/)
- [Add O(1) `Vec -> VecDeque` conversion guarantee](https://github.com/crablang/crablang/pull/105128/)
- [Leak amplification for peek_mut() to ensure BinaryHeap's invariant is always met](https://github.com/crablang/crablang/pull/105851/)

<a id="1.68.0-Stabilized-APIs"></a>

Stabilized APIs
---------------

- [`{core,std}::pin::pin!`](https://doc.crablang.org/stable/std/pin/macro.pin.html)
- [`impl From<bool> for {f32,f64}`](https://doc.crablang.org/stable/std/primitive.f32.html#impl-From%3Cbool%3E-for-f32)
- [`std::path::MAIN_SEPARATOR_STR`](https://doc.crablang.org/stable/std/path/constant.MAIN_SEPARATOR_STR.html)
- [`impl DerefMut for PathBuf`](https://doc.crablang.org/stable/std/path/struct.PathBuf.html#impl-DerefMut-for-PathBuf)

These APIs are now stable in const contexts:

- [`VecDeque::new`](https://doc.crablang.org/stable/std/collections/struct.VecDeque.html#method.new)

<a id="1.68.0-Cargo"></a>

Cargo
-----

- [Stabilize sparse registry support for crates.io](https://github.com/crablang/cargo/pull/11224/)
- [`cargo build --verbose` tells you more about why it recompiles.](https://github.com/crablang/cargo/pull/11407/)
- [Show progress of crates.io index update even `net.git-fetch-with-cli` option enabled](https://github.com/crablang/cargo/pull/11579/)

<a id="1.68.0-Misc"></a>

Misc
----

<a id="1.68.0-Compatibility-Notes"></a>

Compatibility Notes
-------------------

- [Only support Android NDK 25 or newer](https://blog.crablang.org/2023/01/09/android-ndk-update-r25.html)
- [Add `SEMICOLON_IN_EXPRESSIONS_FROM_MACROS` to future-incompat report](https://github.com/crablang/crablang/pull/103418/)
- [Only specify `--target` by default for `-Zgcc-ld=lld` on wasm](https://github.com/crablang/crablang/pull/101792/)
- [Bump `IMPLIED_BOUNDS_ENTAILMENT` to Deny + ReportNow](https://github.com/crablang/crablang/pull/106465/)
- [`std::task::Context` no longer implements Send and Sync](https://github.com/crablang/crablang/pull/95985)

<a id="1.68.0-Internal-Changes"></a>

Internal Changes
----------------

These changes do not affect any public interfaces of CrabLang, but they represent
significant improvements to the performance or internals of crablangc and related
tools.

- [Encode spans relative to the enclosing item](https://github.com/crablang/crablang/pull/84762/)
- [Don't normalize in AstConv](https://github.com/crablang/crablang/pull/101947/)
- [Find the right lower bound region in the scenario of partial order relations](https://github.com/crablang/crablang/pull/104765/)
- [Fix impl block in const expr](https://github.com/crablang/crablang/pull/104889/)
- [Check ADT fields for copy implementations considering regions](https://github.com/crablang/crablang/pull/105102/)
- [crablangdoc: simplify JS search routine by not messing with lev distance](https://github.com/crablang/crablang/pull/105796/)
- [Enable ThinLTO for crablangc on `x86_64-pc-windows-msvc`](https://github.com/crablang/crablang/pull/103591/)
- [Enable ThinLTO for crablangc on `x86_64-apple-darwin`](https://github.com/crablang/crablang/pull/103647/)

Version 1.67.1 (2023-02-09)
===========================

- [Fix interoperability with thin archives.](https://github.com/crablang/crablang/pull/107360)
- [Fix an internal error in the compiler build process.](https://github.com/crablang/crablang/pull/105624)
- [Downgrade `clippy::uninlined_format_args` to pedantic.](https://github.com/crablang/crablang-clippy/pull/10265)

Version 1.67.0 (2023-01-26)
==========================

<a id="1.67.0-Language"></a>

Language
--------

- [Make `Sized` predicates coinductive, allowing cycles.](https://github.com/crablang/crablang/pull/100386/)
- [`#[must_use]` annotations on `async fn` also affect the `Future::Output`.](https://github.com/crablang/crablang/pull/100633/)
- [Elaborate supertrait obligations when deducing closure signatures.](https://github.com/crablang/crablang/pull/101834/)
- [Invalid literals are no longer an error under `cfg(FALSE)`.](https://github.com/crablang/crablang/pull/102944/)
- [Unreserve braced enum variants in value namespace.](https://github.com/crablang/crablang/pull/103578/)

<a id="1.67.0-Compiler"></a>

Compiler
--------

- [Enable varargs support for calling conventions other than `C` or `cdecl`.](https://github.com/crablang/crablang/pull/97971/)
- [Add new MIR constant propagation based on dataflow analysis.](https://github.com/crablang/crablang/pull/101168/)
- [Optimize field ordering by grouping m\*2^n-sized fields with equivalently aligned ones.](https://github.com/crablang/crablang/pull/102750/)
- [Stabilize native library modifier `verbatim`.](https://github.com/crablang/crablang/pull/104360/)

Added, updated, and removed targets:

- [Add a tier 3 target for PowerPC on AIX](https://github.com/crablang/crablang/pull/102293/), `powerpc64-ibm-aix`.
- [Add a tier 3 target for the Sony PlayStation 1](https://github.com/crablang/crablang/pull/102689/), `mipsel-sony-psx`.
- [Add tier 3 `no_std` targets for the QNX Neutrino RTOS](https://github.com/crablang/crablang/pull/102701/),
  `aarch64-unknown-nto-qnx710` and `x86_64-pc-nto-qnx710`.
- [Promote UEFI targets to tier 2](https://github.com/crablang/crablang/pull/103933/), `aarch64-unknown-uefi`, `i686-unknown-uefi`, and `x86_64-unknown-uefi`.
- [Remove tier 3 `linuxkernel` targets](https://github.com/crablang/crablang/pull/104015/) (not used by the actual kernel).

Refer to CrabLang's [platform support page][platform-support-doc]
for more information on CrabLang's tiered platform support.

<a id="1.67.0-Libraries"></a>

Libraries
---------

- [Merge `crossbeam-channel` into `std::sync::mpsc`.](https://github.com/crablang/crablang/pull/93563/)
- [Fix inconsistent rounding of 0.5 when formatted to 0 decimal places.](https://github.com/crablang/crablang/pull/102935/)
- [Derive `Eq` and `Hash` for `ControlFlow`.](https://github.com/crablang/crablang/pull/103084/)
- [Don't build `compiler_builtins` with `-C panic=abort`.](https://github.com/crablang/crablang/pull/103786/)

<a id="1.67.0-Stabilized-APIs"></a>

Stabilized APIs
---------------

- [`{integer}::checked_ilog`](https://doc.crablang.org/stable/std/primitive.i32.html#method.checked_ilog)
- [`{integer}::checked_ilog2`](https://doc.crablang.org/stable/std/primitive.i32.html#method.checked_ilog2)
- [`{integer}::checked_ilog10`](https://doc.crablang.org/stable/std/primitive.i32.html#method.checked_ilog10)
- [`{integer}::ilog`](https://doc.crablang.org/stable/std/primitive.i32.html#method.ilog)
- [`{integer}::ilog2`](https://doc.crablang.org/stable/std/primitive.i32.html#method.ilog2)
- [`{integer}::ilog10`](https://doc.crablang.org/stable/std/primitive.i32.html#method.ilog10)
- [`NonZeroU*::ilog2`](https://doc.crablang.org/stable/std/num/struct.NonZeroU32.html#method.ilog2)
- [`NonZeroU*::ilog10`](https://doc.crablang.org/stable/std/num/struct.NonZeroU32.html#method.ilog10)
- [`NonZero*::BITS`](https://doc.crablang.org/stable/std/num/struct.NonZeroU32.html#associatedconstant.BITS)

These APIs are now stable in const contexts:

- [`char::from_u32`](https://doc.crablang.org/stable/std/primitive.char.html#method.from_u32)
- [`char::from_digit`](https://doc.crablang.org/stable/std/primitive.char.html#method.from_digit)
- [`char::to_digit`](https://doc.crablang.org/stable/std/primitive.char.html#method.to_digit)
- [`core::char::from_u32`](https://doc.crablang.org/stable/core/char/fn.from_u32.html)
- [`core::char::from_digit`](https://doc.crablang.org/stable/core/char/fn.from_digit.html)

<a id="1.67.0-Compatibility-Notes"></a>

Compatibility Notes
-------------------

- [The layout of `repr(CrabLang)` types now groups m\*2^n-sized fields with
  equivalently aligned ones.](https://github.com/crablang/crablang/pull/102750/)
  This is intended to be an optimization, but it is also known to increase type
  sizes in a few cases for the placement of enum tags. As a reminder, the layout
  of `repr(CrabLang)` types is an implementation detail, subject to change.
- [0.5 now rounds to 0 when formatted to 0 decimal places.](https://github.com/crablang/crablang/pull/102935/)
  This makes it consistent with the rest of floating point formatting that
  rounds ties toward even digits.
- [Chains of `&&` and `||` will now drop temporaries from their sub-expressions in
  evaluation order, left-to-right.](https://github.com/crablang/crablang/pull/103293/)
  Previously, it was "twisted" such that the _first_ expression dropped its
  temporaries _last_, after all of the other expressions dropped in order.
- [Underscore suffixes on string literals are now a hard error.](https://github.com/crablang/crablang/pull/103914/)
  This has been a future-compatibility warning since 1.20.0.
- [Stop passing `-export-dynamic` to `wasm-ld`.](https://github.com/crablang/crablang/pull/105405/)
- [`main` is now mangled as `__main_void` on `wasm32-wasi`.](https://github.com/crablang/crablang/pull/105468/)
- [Cargo now emits an error if there are multiple registries in the configuration
  with the same index URL.](https://github.com/crablang/cargo/pull/10592)

<a id="1.67.0-Internal-Changes"></a>

Internal Changes
----------------

These changes do not affect any public interfaces of CrabLang, but they represent
significant improvements to the performance or internals of crablangc and related
tools.

- [Rewrite LLVM's archive writer in CrabLang.](https://github.com/crablang/crablang/pull/97485/)

Version 1.66.1 (2023-01-10)
===========================

- Added validation of SSH host keys for git URLs in Cargo ([CVE-2022-46176](https://www.cve.org/CVERecord?id=CVE-2022-46176))

Version 1.66.0 (2022-12-15)
==========================

Language
--------
- [Permit specifying explicit discriminants on all `repr(Int)` enums](https://github.com/crablang/crablang/pull/95710/)
  ```crablang
  #[repr(u8)]
  enum Foo {
      A(u8) = 0,
      B(i8) = 1,
      C(bool) = 42,
  }
  ```
- [Allow transmutes between the same type differing only in lifetimes](https://github.com/crablang/crablang/pull/101520/)
- [Change constant evaluation errors from a deny-by-default lint to a hard error](https://github.com/crablang/crablang/pull/102091/)
- [Trigger `must_use` on `impl Trait` for supertraits](https://github.com/crablang/crablang/pull/102287/)
  This makes `impl ExactSizeIterator` respect the existing `#[must_use]` annotation on `Iterator`.
- [Allow `..=X` in patterns](https://github.com/crablang/crablang/pull/102275/)
- [Uplift `clippy::for_loops_over_fallibles` lint into crablangc](https://github.com/crablang/crablang/pull/99696/)
- [Stabilize `sym` operands in inline assembly](https://github.com/crablang/crablang/pull/103168/)
- [Update to Unicode 15](https://github.com/crablang/crablang/pull/101912/)
- [Opaque types no longer imply lifetime bounds](https://github.com/crablang/crablang/pull/95474/)
  This is a soundness fix which may break code that was erroneously relying on this behavior.

Compiler
--------
- [Add armv5te-none-eabi and thumbv5te-none-eabi tier 3 targets](https://github.com/crablang/crablang/pull/101329/)
  - Refer to CrabLang's [platform support page][platform-support-doc] for more
    information on CrabLang's tiered platform support.
- [Add support for linking against macOS universal libraries](https://github.com/crablang/crablang/pull/98736)

Libraries
---------
- [Fix `#[derive(Default)]` on a generic `#[default]` enum adding unnecessary `Default` bounds](https://github.com/crablang/crablang/pull/101040/)
- [Update to Unicode 15](https://github.com/crablang/crablang/pull/101821/)

Stabilized APIs
---------------

- [`proc_macro::Span::source_text`](https://doc.crablang.org/stable/proc_macro/struct.Span.html#method.source_text)
- [`uX::{checked_add_signed, overflowing_add_signed, saturating_add_signed, wrapping_add_signed}`](https://doc.crablang.org/stable/std/primitive.u8.html#method.checked_add_signed)
- [`iX::{checked_add_unsigned, overflowing_add_unsigned, saturating_add_unsigned, wrapping_add_unsigned}`](https://doc.crablang.org/stable/std/primitive.i8.html#method.checked_add_unsigned)
- [`iX::{checked_sub_unsigned, overflowing_sub_unsigned, saturating_sub_unsigned, wrapping_sub_unsigned}`](https://doc.crablang.org/stable/std/primitive.i8.html#method.checked_sub_unsigned)
- [`BTreeSet::{first, last, pop_first, pop_last}`](https://doc.crablang.org/stable/std/collections/struct.BTreeSet.html#method.first)
- [`BTreeMap::{first_key_value, last_key_value, first_entry, last_entry, pop_first, pop_last}`](https://doc.crablang.org/stable/std/collections/struct.BTreeMap.html#method.first_key_value)
- [Add `AsFd` implementations for stdio lock types on WASI.](https://github.com/crablang/crablang/pull/101768/)
- [`impl TryFrom<Vec<T>> for Box<[T; N]>`](https://doc.crablang.org/stable/std/boxed/struct.Box.html#impl-TryFrom%3CVec%3CT%2C%20Global%3E%3E-for-Box%3C%5BT%3B%20N%5D%2C%20Global%3E)
- [`core::hint::black_box`](https://doc.crablang.org/stable/std/hint/fn.black_box.html)
- [`Duration::try_from_secs_{f32,f64}`](https://doc.crablang.org/stable/std/time/struct.Duration.html#method.try_from_secs_f32)
- [`Option::unzip`](https://doc.crablang.org/stable/std/option/enum.Option.html#method.unzip)
- [`std::os::fd`](https://doc.crablang.org/stable/std/os/fd/index.html)


CrabLangdoc
-------

- [Add CrabLangdoc warning for invalid HTML tags in the documentation](https://github.com/crablang/crablang/pull/101720/)

Cargo
-----

- [Added `cargo remove` to remove dependencies from Cargo.toml](https://doc.crablang.org/nightly/cargo/commands/cargo-remove.html)
- [`cargo publish` now waits for the new version to be downloadable before exiting](https://github.com/crablang/cargo/pull/11062)

See [detailed release notes](https://github.com/crablang/cargo/blob/master/CHANGELOG.md#cargo-166-2022-12-15) for more.

Compatibility Notes
-------------------

- [Only apply `ProceduralMasquerade` hack to older versions of `rental`](https://github.com/crablang/crablang/pull/94063/)
- [Don't export `__heap_base` and `__data_end` on wasm32-wasi.](https://github.com/crablang/crablang/pull/102385/)
- [Don't export `__wasm_init_memory` on WebAssembly.](https://github.com/crablang/crablang/pull/102426/)
- [Only export `__tls_*` on wasm32-unknown-unknown.](https://github.com/crablang/crablang/pull/102440/)
- [Don't link to `libresolv` in libstd on Darwin](https://github.com/crablang/crablang/pull/102766/)
- [Update libstd's libc to 0.2.135 (to make `libstd` no longer pull in `libiconv.dylib` on Darwin)](https://github.com/crablang/crablang/pull/103277/)
- [Opaque types no longer imply lifetime bounds](https://github.com/crablang/crablang/pull/95474/)
  This is a soundness fix which may break code that was erroneously relying on this behavior.
- [Make `order_dependent_trait_objects` show up in future-breakage reports](https://github.com/crablang/crablang/pull/102635/)
- [Change std::process::Command spawning to default to inheriting the parent's signal mask](https://github.com/crablang/crablang/pull/101077/)

Internal Changes
----------------

These changes do not affect any public interfaces of CrabLang, but they represent
significant improvements to the performance or internals of crablangc and related
tools.

- [Enable BOLT for LLVM compilation](https://github.com/crablang/crablang/pull/94381/)
- [Enable LTO for crablangc_driver.so](https://github.com/crablang/crablang/pull/101403/)

Version 1.65.0 (2022-11-03)
==========================

Language
--------
- [Error on `as` casts of enums with `#[non_exhaustive]` variants](https://github.com/crablang/crablang/pull/92744/)
- [Stabilize `let else`](https://github.com/crablang/crablang/pull/93628/)
- [Stabilize generic associated types (GATs)](https://github.com/crablang/crablang/pull/96709/)
- [Add lints `let_underscore_drop` and `let_underscore_lock` from Clippy](https://github.com/crablang/crablang/pull/97739/)
- [Stabilize `break`ing from arbitrary labeled blocks ("label-break-value")](https://github.com/crablang/crablang/pull/99332/)
- [Uninitialized integers, floats, and raw pointers are now considered immediate UB](https://github.com/crablang/crablang/pull/98919/).
  Usage of `MaybeUninit` is the correct way to work with uninitialized memory.
- [Stabilize raw-dylib for Windows x86_64, aarch64, and thumbv7a](https://github.com/crablang/crablang/pull/99916/)
- [Do not allow `Drop` impl on foreign ADTs](https://github.com/crablang/crablang/pull/99576/)

Compiler
--------
- [Stabilize -Csplit-debuginfo on Linux](https://github.com/crablang/crablang/pull/98051/)
- [Use niche-filling optimization even when multiple variants have data](https://github.com/crablang/crablang/pull/94075/)
- [Associated type projections are now verified to be well-formed prior to resolving the underlying type](https://github.com/crablang/crablang/pull/99217/#issuecomment-1209365630)
- [Stringify non-shorthand visibility correctly](https://github.com/crablang/crablang/pull/100350/)
- [Normalize struct field types when unsizing](https://github.com/crablang/crablang/pull/101831/)
- [Update to LLVM 15](https://github.com/crablang/crablang/pull/99464/)
- [Fix aarch64 call abi to correctly zeroext when needed](https://github.com/crablang/crablang/pull/97800/)
- [debuginfo: Generalize C++-like encoding for enums](https://github.com/crablang/crablang/pull/98393/)
- [Add `special_module_name` lint](https://github.com/crablang/crablang/pull/94467/)
- [Add support for generating unique profraw files by default when using `-C instrument-coverage`](https://github.com/crablang/crablang/pull/100384/)
- [Allow dynamic linking for iOS/tvOS targets](https://github.com/crablang/crablang/pull/100636/)

New targets:

- [Add armv4t-none-eabi as a tier 3 target](https://github.com/crablang/crablang/pull/100244/)
- [Add powerpc64-unknown-openbsd and riscv64-unknown-openbsd as tier 3 targets](https://github.com/crablang/crablang/pull/101025/)
  - Refer to CrabLang's [platform support page][platform-support-doc] for more
    information on CrabLang's tiered platform support.

Libraries
---------

- [Don't generate `PartialEq::ne` in derive(PartialEq)](https://github.com/crablang/crablang/pull/98655/)
- [Windows RNG: Use `BCRYPT_RNG_ALG_HANDLE` by default](https://github.com/crablang/crablang/pull/101325/)
- [Forbid mixing `System` with direct system allocator calls](https://github.com/crablang/crablang/pull/101394/)
- [Document no support for writing to non-blocking stdio/stderr](https://github.com/crablang/crablang/pull/101416/)
- [`std::layout::Layout` size must not overflow `isize::MAX` when rounded up to `align`](https://github.com/crablang/crablang/pull/95295)
  This also changes the safety conditions on `Layout::from_size_align_unchecked`.

Stabilized APIs
---------------

- [`std::backtrace::Backtrace`](https://doc.crablang.org/stable/std/backtrace/struct.Backtrace.html)
- [`Bound::as_ref`](https://doc.crablang.org/stable/std/ops/enum.Bound.html#method.as_ref)
- [`std::io::read_to_string`](https://doc.crablang.org/stable/std/io/fn.read_to_string.html)
- [`<*const T>::cast_mut`](https://doc.crablang.org/stable/std/primitive.pointer.html#method.cast_mut)
- [`<*mut T>::cast_const`](https://doc.crablang.org/stable/std/primitive.pointer.html#method.cast_const)

These APIs are now stable in const contexts:

- [`<*const T>::offset_from`](https://doc.crablang.org/stable/std/primitive.pointer.html#method.offset_from)
- [`<*mut T>::offset_from`](https://doc.crablang.org/stable/std/primitive.pointer.html#method.offset_from)

Cargo
-----

- [Apply GitHub fast path even for partial hashes](https://github.com/crablang/cargo/pull/10807/)
- [Do not add home bin path to PATH if it's already there](https://github.com/crablang/cargo/pull/11023/)
- [Take priority into account within the pending queue](https://github.com/crablang/cargo/pull/11032/).
  This slightly optimizes job scheduling by Cargo, with typically small improvements on larger crate graph builds.

Compatibility Notes
-------------------

- [`std::layout::Layout` size must not overflow `isize::MAX` when rounded up to `align`](https://github.com/crablang/crablang/pull/95295).
  This also changes the safety conditions on `Layout::from_size_align_unchecked`.
- [`PollFn` now only implements `Unpin` if the closure is `Unpin`](https://github.com/crablang/crablang/pull/102737).
  This is a possible breaking change if users were relying on the blanket unpin implementation.
  See discussion on the PR for details of why this change was made.
- [Drop ExactSizeIterator impl from std::char::EscapeAscii](https://github.com/crablang/crablang/pull/99880)
  This is a backwards-incompatible change to the standard library's surface
  area, but is unlikely to affect real world usage.
- [Do not consider a single repeated lifetime eligible for elision in the return type](https://github.com/crablang/crablang/pull/103450)
  This behavior was unintentionally changed in 1.64.0, and this release reverts that change by making this an error again.
- [Reenable disabled early syntax gates as future-incompatibility lints](https://github.com/crablang/crablang/pull/99935/)
- [Update the minimum external LLVM to 13](https://github.com/crablang/crablang/pull/100460/)
- [Don't duplicate file descriptors into stdio fds](https://github.com/crablang/crablang/pull/101426/)
- [Sunset RLS](https://github.com/crablang/crablang/pull/100863/)
- [Deny usage of `#![cfg_attr(..., crate_type = ...)]` to set the crate type](https://github.com/crablang/crablang/pull/99784/)
  This strengthens the forward compatibility lint deprecated_cfg_attr_crate_type_name to deny.
- [`llvm-has-crablang-patches` allows setting the build system to treat the LLVM as having CrabLang-specific patches](https://github.com/crablang/crablang/pull/101072)
  This option may need to be set for distributions that are building CrabLang with a patched LLVM via `llvm-config`, not the built-in LLVM.
- Combining three or more languages (e.g. Objective C, C++ and CrabLang) into one binary may hit linker limitations when using `lld`. For more information, see [issue 102754][102754].

[102754]: https://github.com/crablang/crablang/issues/102754

Internal Changes
----------------

These changes do not affect any public interfaces of CrabLang, but they represent
significant improvements to the performance or internals of crablangc and related
tools.

- [Add `x.sh` and `x.ps1` shell scripts](https://github.com/crablang/crablang/pull/99992/)
- [compiletest: use target cfg instead of hard-coded tables](https://github.com/crablang/crablang/pull/100260/)
- [Use object instead of LLVM for reading bitcode from rlibs](https://github.com/crablang/crablang/pull/98100/)
- [Enable MIR inlining for optimized compilations](https://github.com/crablang/crablang/pull/91743)
  This provides a 3-10% improvement in compiletimes for real world crates. See [perf results](https://perf.crablang.org/compare.html?start=aedf78e56b2279cc869962feac5153b6ba7001ed&end=0075bb4fad68e64b6d1be06bf2db366c30bc75e1&stat=instructions:u).

Version 1.64.0 (2022-09-22)
===========================

Language
--------
- [Unions with mutable references or tuples of allowed types are now allowed](https://github.com/crablang/crablang/pull/97995/)
- It is now considered valid to deallocate memory pointed to by a shared reference `&T` [if every byte in `T` is inside an `UnsafeCell`](https://github.com/crablang/crablang/pull/98017/)
- Unused tuple struct fields are now warned against in an allow-by-default lint, [`unused_tuple_struct_fields`](https://github.com/crablang/crablang/pull/95977/), similar to the existing warning for unused struct fields. This lint will become warn-by-default in the future.

Compiler
--------
- [Add Nintendo Switch as tier 3 target](https://github.com/crablang/crablang/pull/88991/)
  - Refer to CrabLang's [platform support page][platform-support-doc] for more
    information on CrabLang's tiered platform support.
- [Only compile `#[used]` as llvm.compiler.used for ELF targets](https://github.com/crablang/crablang/pull/93718/)
- [Add the `--diagnostic-width` compiler flag to define the terminal width.](https://github.com/crablang/crablang/pull/95635/)
- [Add support for link-flavor `crablang-lld` for iOS, tvOS and watchOS](https://github.com/crablang/crablang/pull/98771/)

Libraries
---------
- [Remove restrictions on compare-exchange memory ordering.](https://github.com/crablang/crablang/pull/98383/)
- You can now `write!` or `writeln!` into an `OsString`: [Implement `fmt::Write` for `OsString`](https://github.com/crablang/crablang/pull/97915/)
- [Make RwLockReadGuard covariant](https://github.com/crablang/crablang/pull/96820/)
- [Implement `FusedIterator` for `std::net::[Into]Incoming`](https://github.com/crablang/crablang/pull/97300/)
- [`impl<T: AsRawFd> AsRawFd for {Arc,Box}<T>`](https://github.com/crablang/crablang/pull/97437/)
- [`ptr::copy` and `ptr::swap` are doing untyped copies](https://github.com/crablang/crablang/pull/97712/)
- [Add cgroupv1 support to `available_parallelism`](https://github.com/crablang/crablang/pull/97925/)
- [Mitigate many incorrect uses of `mem::uninitialized`](https://github.com/crablang/crablang/pull/99182/)

Stabilized APIs
---------------

- [`future::IntoFuture`](https://doc.crablang.org/stable/std/future/trait.IntoFuture.html)
- [`future::poll_fn`](https://doc.crablang.org/stable/std/future/fn.poll_fn.html)
- [`task::ready!`](https://doc.crablang.org/stable/std/task/macro.ready.html)
- [`num::NonZero*::checked_mul`](https://doc.crablang.org/stable/std/num/struct.NonZeroUsize.html#method.checked_mul)
- [`num::NonZero*::checked_pow`](https://doc.crablang.org/stable/std/num/struct.NonZeroUsize.html#method.checked_pow)
- [`num::NonZero*::saturating_mul`](https://doc.crablang.org/stable/std/num/struct.NonZeroUsize.html#method.saturating_mul)
- [`num::NonZero*::saturating_pow`](https://doc.crablang.org/stable/std/num/struct.NonZeroUsize.html#method.saturating_pow)
- [`num::NonZeroI*::abs`](https://doc.crablang.org/stable/std/num/struct.NonZeroIsize.html#method.abs)
- [`num::NonZeroI*::checked_abs`](https://doc.crablang.org/stable/std/num/struct.NonZeroIsize.html#method.checked_abs)
- [`num::NonZeroI*::overflowing_abs`](https://doc.crablang.org/stable/std/num/struct.NonZeroIsize.html#method.overflowing_abs)
- [`num::NonZeroI*::saturating_abs`](https://doc.crablang.org/stable/std/num/struct.NonZeroIsize.html#method.saturating_abs)
- [`num::NonZeroI*::unsigned_abs`](https://doc.crablang.org/stable/std/num/struct.NonZeroIsize.html#method.unsigned_abs)
- [`num::NonZeroI*::wrapping_abs`](https://doc.crablang.org/stable/std/num/struct.NonZeroIsize.html#method.wrapping_abs)
- [`num::NonZeroU*::checked_add`](https://doc.crablang.org/stable/std/num/struct.NonZeroUsize.html#method.checked_add)
- [`num::NonZeroU*::checked_next_power_of_two`](https://doc.crablang.org/stable/std/num/struct.NonZeroUsize.html#method.checked_next_power_of_two)
- [`num::NonZeroU*::saturating_add`](https://doc.crablang.org/stable/std/num/struct.NonZeroUsize.html#method.saturating_add)
- [`os::unix::process::CommandExt::process_group`](https://doc.crablang.org/stable/std/os/unix/process/trait.CommandExt.html#tymethod.process_group)
- [`os::windows::fs::FileTypeExt::is_symlink_dir`](https://doc.crablang.org/stable/std/os/windows/fs/trait.FileTypeExt.html#tymethod.is_symlink_dir)
- [`os::windows::fs::FileTypeExt::is_symlink_file`](https://doc.crablang.org/stable/std/os/windows/fs/trait.FileTypeExt.html#tymethod.is_symlink_file)

These types were previously stable in `std::ffi`, but are now also available in `core` and `alloc`:

- [`core::ffi::CStr`](https://doc.crablang.org/stable/core/ffi/struct.CStr.html)
- [`core::ffi::FromBytesWithNulError`](https://doc.crablang.org/stable/core/ffi/struct.FromBytesWithNulError.html)
- [`alloc::ffi::CString`](https://doc.crablang.org/stable/alloc/ffi/struct.CString.html)
- [`alloc::ffi::FromVecWithNulError`](https://doc.crablang.org/stable/alloc/ffi/struct.FromVecWithNulError.html)
- [`alloc::ffi::IntoStringError`](https://doc.crablang.org/stable/alloc/ffi/struct.IntoStringError.html)
- [`alloc::ffi::NulError`](https://doc.crablang.org/stable/alloc/ffi/struct.NulError.html)

These types were previously stable in `std::os::raw`, but are now also available in `core::ffi` and `std::ffi`:

- [`ffi::c_char`](https://doc.crablang.org/stable/std/ffi/type.c_char.html)
- [`ffi::c_double`](https://doc.crablang.org/stable/std/ffi/type.c_double.html)
- [`ffi::c_float`](https://doc.crablang.org/stable/std/ffi/type.c_float.html)
- [`ffi::c_int`](https://doc.crablang.org/stable/std/ffi/type.c_int.html)
- [`ffi::c_long`](https://doc.crablang.org/stable/std/ffi/type.c_long.html)
- [`ffi::c_longlong`](https://doc.crablang.org/stable/std/ffi/type.c_longlong.html)
- [`ffi::c_schar`](https://doc.crablang.org/stable/std/ffi/type.c_schar.html)
- [`ffi::c_short`](https://doc.crablang.org/stable/std/ffi/type.c_short.html)
- [`ffi::c_uchar`](https://doc.crablang.org/stable/std/ffi/type.c_uchar.html)
- [`ffi::c_uint`](https://doc.crablang.org/stable/std/ffi/type.c_uint.html)
- [`ffi::c_ulong`](https://doc.crablang.org/stable/std/ffi/type.c_ulong.html)
- [`ffi::c_ulonglong`](https://doc.crablang.org/stable/std/ffi/type.c_ulonglong.html)
- [`ffi::c_ushort`](https://doc.crablang.org/stable/std/ffi/type.c_ushort.html)

These APIs are now usable in const contexts:

- [`slice::from_raw_parts`](https://doc.crablang.org/stable/core/slice/fn.from_raw_parts.html)

Cargo
-----
- [Packages can now inherit settings from the workspace so that the settings
  can be centralized in one place.](https://github.com/crablang/cargo/pull/10859) See
  [`workspace.package`](https://doc.crablang.org/nightly/cargo/reference/workspaces.html#the-workspacepackage-table)
  and
  [`workspace.dependencies`](https://doc.crablang.org/nightly/cargo/reference/workspaces.html#the-workspacedependencies-table)
  for more details on how to define these common settings.
- [Cargo commands can now accept multiple `--target` flags to build for
  multiple targets at once](https://github.com/crablang/cargo/pull/10766), and the
  [`build.target`](https://doc.crablang.org/nightly/cargo/reference/config.html#buildtarget)
  config option may now take an array of multiple targets.
- [The `--jobs` argument can now take a negative number to count backwards from
  the max CPUs.](https://github.com/crablang/cargo/pull/10844)
- [`cargo add` will now update `Cargo.lock`.](https://github.com/crablang/cargo/pull/10902)
- [Added](https://github.com/crablang/cargo/pull/10838) the
  [`--crate-type`](https://doc.crablang.org/nightly/cargo/commands/cargo-crablangc.html#option-cargo-crablangc---crate-type)
  flag to `cargo crablangc` to override the crate type.
- [Significantly improved the performance fetching git dependencies from GitHub
  when using a hash in the `rev` field.](https://github.com/crablang/cargo/pull/10079)

Misc
----
- [The `crablang-analyzer` crablangup component is now available on the stable channel.](https://github.com/crablang/crablang/pull/98640/)

Compatibility Notes
-------------------
- The minimum required versions for all `-linux-gnu` targets are now at least kernel 3.2 and glibc 2.17, for targets that previously supported older versions: [Increase the minimum linux-gnu versions](https://github.com/crablang/crablang/pull/95026/)
- [Network primitives are now implemented with the ideal CrabLang layout, not the C system layout](https://github.com/crablang/crablang/pull/78802/). This can cause problems when transmuting the types.
- [Add assertion that `transmute_copy`'s `U` is not larger than `T`](https://github.com/crablang/crablang/pull/98839/)
- [A soundness bug in `BTreeMap` was fixed](https://github.com/crablang/crablang/pull/99413/) that allowed data it was borrowing to be dropped before the container.
- [The Drop behavior of C-like enums cast to ints has changed](https://github.com/crablang/crablang/pull/96862/). These are already discouraged by a compiler warning.
- [Relate late-bound closure lifetimes to parent fn in NLL](https://github.com/crablang/crablang/pull/98835/)
- [Errors at const-eval time are now in future incompatibility reports](https://github.com/crablang/crablang/pull/97743/)
- On the `thumbv6m-none-eabi` target, some incorrect `asm!` statements were erroneously accepted if they used the high registers (r8 to r14) as an input/output operand. [This is no longer accepted](https://github.com/crablang/crablang/pull/99155/).
- [`impl Trait` was accidentally accepted as the associated type value of return-position `impl Trait`](https://github.com/crablang/crablang/pull/97346/), without fulfilling all the trait bounds of that associated type, as long as the hidden type satisfies said bounds. This has been fixed.

Internal Changes
----------------

These changes do not affect any public interfaces of CrabLang, but they represent
significant improvements to the performance or internals of crablangc and related
tools.

- Windows builds now use profile-guided optimization, providing 10-20% improvements to compiler performance: [Utilize PGO for windows x64 crablangc dist builds](https://github.com/crablang/crablang/pull/96978/)
- [Stop keeping metadata in memory before writing it to disk](https://github.com/crablang/crablang/pull/96544/)
- [compiletest: strip debuginfo by default for mode=ui](https://github.com/crablang/crablang/pull/98140/)
- Many improvements to generated code for derives, including performance improvements:
  - [Don't use match-destructuring for derived ops on structs.](https://github.com/crablang/crablang/pull/98446/)
  - [Many small deriving cleanups](https://github.com/crablang/crablang/pull/98741/)
  - [More derive output improvements](https://github.com/crablang/crablang/pull/98758/)
  - [Clarify deriving code](https://github.com/crablang/crablang/pull/98915/)
  - [Final derive output improvements](https://github.com/crablang/crablang/pull/99046/)
  - [Stop injecting `#[allow(unused_qualifications)]` in generated `derive` implementations](https://github.com/crablang/crablang/pull/99485/)
  - [Improve `derive(Debug)`](https://github.com/crablang/crablang/pull/98190/)
- [Bump to clap 3](https://github.com/crablang/crablang/pull/98213/)
- [fully move dropck to mir](https://github.com/crablang/crablang/pull/98641/)
- [Optimize `Vec::insert` for the case where `index == len`.](https://github.com/crablang/crablang/pull/98755/)
- [Convert crablang-analyzer to an in-tree tool](https://github.com/crablang/crablang/pull/99603/)

Version 1.63.0 (2022-08-11)
==========================

Language
--------
- [Remove migrate borrowck mode for pre-NLL errors.][95565]
- [Modify MIR building to drop repeat expressions with length zero.][95953]
- [Remove label/lifetime shadowing warnings.][96296]
- [Allow explicit generic arguments in the presence of `impl Trait` args.][96868]
- [Make `cenum_impl_drop_cast` warnings deny-by-default.][97652]
- [Prevent unwinding when `-C panic=abort` is used regardless of declared ABI.][96959]
- [lub: don't bail out due to empty binders.][97867]

Compiler
--------
- [Stabilize the `bundle` native library modifier,][95818] also removing the
  deprecated `static-nobundle` linking kind.
- [Add Apple WatchOS compile targets\*.][95243]
- [Add a Windows application manifest to crablangc-main.][96737]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
   information on CrabLang's tiered platform support.

Libraries
---------
- [Implement `Copy`, `Clone`, `PartialEq` and `Eq` for `core::fmt::Alignment`.][94530]
- [Extend `ptr::null` and `null_mut` to all thin (including extern) types.][94954]
- [`impl Read and Write for VecDeque<u8>`.][95632]
- [STD support for the Nintendo 3DS.][95897]
- [Use rounding in float to Duration conversion methods.][96051]
- [Make write/print macros eagerly drop temporaries.][96455]
- [Implement internal traits that enable `[OsStr]::join`.][96881]
- [Implement `Hash` for `core::alloc::Layout`.][97034]
- [Add capacity documentation for `OsString`.][97202]
- [Put a bound on collection misbehavior.][97316]
- [Make `std::mem::needs_drop` accept `?Sized`.][97675]
- [`impl Termination for Infallible` and then make the `Result` impls of `Termination` more generic.][97803]
- [Document CrabLang's stance on `/proc/self/mem`.][97837]

Stabilized APIs
---------------

- [`array::from_fn`]
- [`Box::into_pin`]
- [`BinaryHeap::try_reserve`]
- [`BinaryHeap::try_reserve_exact`]
- [`OsString::try_reserve`]
- [`OsString::try_reserve_exact`]
- [`PathBuf::try_reserve`]
- [`PathBuf::try_reserve_exact`]
- [`Path::try_exists`]
- [`Ref::filter_map`]
- [`RefMut::filter_map`]
- [`NonNull::<[T]>::len`][`NonNull::<slice>::len`]
- [`ToOwned::clone_into`]
- [`Ipv6Addr::to_ipv4_mapped`]
- [`unix::io::AsFd`]
- [`unix::io::BorrowedFd<'fd>`]
- [`unix::io::OwnedFd`]
- [`windows::io::AsHandle`]
- [`windows::io::BorrowedHandle<'handle>`]
- [`windows::io::OwnedHandle`]
- [`windows::io::HandleOrInvalid`]
- [`windows::io::HandleOrNull`]
- [`windows::io::InvalidHandleError`]
- [`windows::io::NullHandleError`]
- [`windows::io::AsSocket`]
- [`windows::io::BorrowedSocket<'handle>`]
- [`windows::io::OwnedSocket`]
- [`thread::scope`]
- [`thread::Scope`]
- [`thread::ScopedJoinHandle`]

These APIs are now usable in const contexts:

- [`array::from_ref`]
- [`slice::from_ref`]
- [`intrinsics::copy`]
- [`intrinsics::copy_nonoverlapping`]
- [`<*const T>::copy_to`]
- [`<*const T>::copy_to_nonoverlapping`]
- [`<*mut T>::copy_to`]
- [`<*mut T>::copy_to_nonoverlapping`]
- [`<*mut T>::copy_from`]
- [`<*mut T>::copy_from_nonoverlapping`]
- [`str::from_utf8`]
- [`Utf8Error::error_len`]
- [`Utf8Error::valid_up_to`]
- [`Condvar::new`]
- [`Mutex::new`]
- [`RwLock::new`]

Cargo
-----
- [Stabilize the `--config path` command-line argument.][cargo/10755]
- [Expose crablang-version in the environment as `CARGO_PKG_CRABLANG_VERSION`.][cargo/10713]

Compatibility Notes
-------------------

- [`#[link]` attributes are now checked more strictly,][96885] which may introduce
  errors for invalid attribute arguments that were previously ignored.
- [Rounding is now used when converting a float to a `Duration`.][96051] The converted
  duration can differ slightly from what it was.

Internal Changes
----------------

These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc
and related tools.

- [Prepare CrabLang for LLVM opaque pointers.][94214]

[94214]: https://github.com/crablang/crablang/pull/94214/
[94530]: https://github.com/crablang/crablang/pull/94530/
[94954]: https://github.com/crablang/crablang/pull/94954/
[95243]: https://github.com/crablang/crablang/pull/95243/
[95565]: https://github.com/crablang/crablang/pull/95565/
[95632]: https://github.com/crablang/crablang/pull/95632/
[95818]: https://github.com/crablang/crablang/pull/95818/
[95897]: https://github.com/crablang/crablang/pull/95897/
[95953]: https://github.com/crablang/crablang/pull/95953/
[96051]: https://github.com/crablang/crablang/pull/96051/
[96296]: https://github.com/crablang/crablang/pull/96296/
[96455]: https://github.com/crablang/crablang/pull/96455/
[96737]: https://github.com/crablang/crablang/pull/96737/
[96868]: https://github.com/crablang/crablang/pull/96868/
[96881]: https://github.com/crablang/crablang/pull/96881/
[96885]: https://github.com/crablang/crablang/pull/96885/
[96959]: https://github.com/crablang/crablang/pull/96959/
[97034]: https://github.com/crablang/crablang/pull/97034/
[97202]: https://github.com/crablang/crablang/pull/97202/
[97316]: https://github.com/crablang/crablang/pull/97316/
[97652]: https://github.com/crablang/crablang/pull/97652/
[97675]: https://github.com/crablang/crablang/pull/97675/
[97803]: https://github.com/crablang/crablang/pull/97803/
[97837]: https://github.com/crablang/crablang/pull/97837/
[97867]: https://github.com/crablang/crablang/pull/97867/
[cargo/10713]: https://github.com/crablang/cargo/pull/10713/
[cargo/10755]: https://github.com/crablang/cargo/pull/10755/

[`array::from_fn`]: https://doc.crablang.org/stable/std/array/fn.from_fn.html
[`Box::into_pin`]: https://doc.crablang.org/stable/std/boxed/struct.Box.html#method.into_pin
[`BinaryHeap::try_reserve_exact`]: https://doc.crablang.org/stable/alloc/collections/binary_heap/struct.BinaryHeap.html#method.try_reserve_exact
[`BinaryHeap::try_reserve`]: https://doc.crablang.org/stable/std/collections/struct.BinaryHeap.html#method.try_reserve
[`OsString::try_reserve`]: https://doc.crablang.org/stable/std/ffi/struct.OsString.html#method.try_reserve
[`OsString::try_reserve_exact`]: https://doc.crablang.org/stable/std/ffi/struct.OsString.html#method.try_reserve_exact
[`PathBuf::try_reserve`]: https://doc.crablang.org/stable/std/path/struct.PathBuf.html#method.try_reserve
[`PathBuf::try_reserve_exact`]: https://doc.crablang.org/stable/std/path/struct.PathBuf.html#method.try_reserve_exact
[`Path::try_exists`]: https://doc.crablang.org/stable/std/path/struct.Path.html#method.try_exists
[`Ref::filter_map`]: https://doc.crablang.org/stable/std/cell/struct.Ref.html#method.filter_map
[`RefMut::filter_map`]: https://doc.crablang.org/stable/std/cell/struct.RefMut.html#method.filter_map
[`NonNull::<slice>::len`]: https://doc.crablang.org/stable/std/ptr/struct.NonNull.html#method.len
[`ToOwned::clone_into`]: https://doc.crablang.org/stable/std/borrow/trait.ToOwned.html#method.clone_into
[`Ipv6Addr::to_ipv4_mapped`]: https://doc.crablang.org/stable/std/net/struct.Ipv6Addr.html#method.to_ipv4_mapped
[`unix::io::AsFd`]: https://doc.crablang.org/stable/std/os/unix/io/trait.AsFd.html
[`unix::io::BorrowedFd<'fd>`]: https://doc.crablang.org/stable/std/os/unix/io/struct.BorrowedFd.html
[`unix::io::OwnedFd`]: https://doc.crablang.org/stable/std/os/unix/io/struct.OwnedFd.html
[`windows::io::AsHandle`]: https://doc.crablang.org/stable/std/os/windows/io/trait.AsHandle.html
[`windows::io::BorrowedHandle<'handle>`]: https://doc.crablang.org/stable/std/os/windows/io/struct.BorrowedHandle.html
[`windows::io::OwnedHandle`]: https://doc.crablang.org/stable/std/os/windows/io/struct.OwnedHandle.html
[`windows::io::HandleOrInvalid`]: https://doc.crablang.org/stable/std/os/windows/io/struct.HandleOrInvalid.html
[`windows::io::HandleOrNull`]: https://doc.crablang.org/stable/std/os/windows/io/struct.HandleOrNull.html
[`windows::io::InvalidHandleError`]: https://doc.crablang.org/stable/std/os/windows/io/struct.InvalidHandleError.html
[`windows::io::NullHandleError`]: https://doc.crablang.org/stable/std/os/windows/io/struct.NullHandleError.html
[`windows::io::AsSocket`]: https://doc.crablang.org/stable/std/os/windows/io/trait.AsSocket.html
[`windows::io::BorrowedSocket<'handle>`]: https://doc.crablang.org/stable/std/os/windows/io/struct.BorrowedSocket.html
[`windows::io::OwnedSocket`]: https://doc.crablang.org/stable/std/os/windows/io/struct.OwnedSocket.html
[`thread::scope`]: https://doc.crablang.org/stable/std/thread/fn.scope.html
[`thread::Scope`]: https://doc.crablang.org/stable/std/thread/struct.Scope.html
[`thread::ScopedJoinHandle`]: https://doc.crablang.org/stable/std/thread/struct.ScopedJoinHandle.html

[`array::from_ref`]: https://doc.crablang.org/stable/std/array/fn.from_ref.html
[`slice::from_ref`]: https://doc.crablang.org/stable/std/slice/fn.from_ref.html
[`intrinsics::copy`]: https://doc.crablang.org/stable/std/intrinsics/fn.copy.html
[`intrinsics::copy_nonoverlapping`]: https://doc.crablang.org/stable/std/intrinsics/fn.copy_nonoverlapping.html
[`<*const T>::copy_to`]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.copy_to
[`<*const T>::copy_to_nonoverlapping`]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.copy_to_nonoverlapping
[`<*mut T>::copy_to`]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.copy_to-1
[`<*mut T>::copy_to_nonoverlapping`]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.copy_to_nonoverlapping-1
[`<*mut T>::copy_from`]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.copy_from
[`<*mut T>::copy_from_nonoverlapping`]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.copy_from_nonoverlapping
[`str::from_utf8`]: https://doc.crablang.org/stable/std/str/fn.from_utf8.html
[`Utf8Error::error_len`]: https://doc.crablang.org/stable/std/str/struct.Utf8Error.html#method.error_len
[`Utf8Error::valid_up_to`]: https://doc.crablang.org/stable/std/str/struct.Utf8Error.html#method.valid_up_to
[`Condvar::new`]: https://doc.crablang.org/stable/std/sync/struct.Condvar.html#method.new
[`Mutex::new`]: https://doc.crablang.org/stable/std/sync/struct.Mutex.html#method.new
[`RwLock::new`]: https://doc.crablang.org/stable/std/sync/struct.RwLock.html#method.new

Version 1.62.1 (2022-07-19)
==========================

CrabLang 1.62.1 addresses a few recent regressions in the compiler and standard
library, and also mitigates a CPU vulnerability on Intel SGX.

* [The compiler fixed unsound function coercions involving `impl Trait` return types.][98608]
* [The compiler fixed an incremental compilation bug with `async fn` lifetimes.][98890]
* [Windows added a fallback for overlapped I/O in synchronous reads and writes.][98950]
* [The `x86_64-fortanix-unknown-sgx` target added a mitigation for the
  MMIO stale data vulnerability][98126], advisory [INTEL-SA-00615].

[98608]: https://github.com/crablang/crablang/issues/98608
[98890]: https://github.com/crablang/crablang/issues/98890
[98950]: https://github.com/crablang/crablang/pull/98950
[98126]: https://github.com/crablang/crablang/pull/98126
[INTEL-SA-00615]: https://www.intel.com/content/www/us/en/security-center/advisory/intel-sa-00615.html

Version 1.62.0 (2022-06-30)
==========================

Language
--------

- [Stabilize `#[derive(Default)]` on enums with a `#[default]` variant][94457]
- [Teach flow sensitive checks that visibly uninhabited call expressions never return][93313]
- [Fix constants not getting dropped if part of a diverging expression][94775]
- [Support unit struct/enum variant in destructuring assignment][95380]
- [Remove mutable_borrow_reservation_conflict lint and allow the code pattern][96268]
- [`const` functions may now specify `extern "C"` or `extern "CrabLang"`][95346]

Compiler
--------

- [linker: Stop using whole-archive on dependencies of dylibs][96436]
- [Make `unaligned_references` lint deny-by-default][95372]
  This lint is also a future compatibility lint, and is expected to eventually
  become a hard error.
- [Only add codegen backend to dep info if -Zbinary-dep-depinfo is used][93969]
- [Reject `#[thread_local]` attribute on non-static items][95006]
- [Add tier 3 `aarch64-pc-windows-gnullvm` and `x86_64-pc-windows-gnullvm` targets\*][94872]
- [Implement a lint to warn about unused macro rules][96150]
- [Promote `x86_64-unknown-none` target to Tier 2\*][95705]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
   information on CrabLang's tiered platform support.

Libraries
---------

- [Windows: Use a pipe relay for chaining pipes][95841]
- [Replace Linux Mutex and Condvar with futex based ones.][95035]
- [Replace RwLock by a futex based one on Linux][95801]
- [std: directly use pthread in UNIX parker implementation][96393]

Stabilized APIs
---------------

- [`bool::then_some`]
- [`f32::total_cmp`]
- [`f64::total_cmp`]
- [`Stdin::lines`]
- [`windows::CommandExt::raw_arg`]
- [`impl<T: Default> Default for AssertUnwindSafe<T>`]
- [`From<Rc<str>> for Rc<[u8]>`][rc-u8-from-str]
- [`From<Arc<str>> for Arc<[u8]>`][arc-u8-from-str]
- [`FusedIterator for EncodeWide`]
- [RDM intrinsics on aarch64][stdarch/1285]

Clippy
------

- [Create clippy lint against unexpectedly late drop for temporaries in match scrutinee expressions][94206]

Cargo
-----

- Added the `cargo add` command for adding dependencies to `Cargo.toml` from
  the command-line.
  [docs](https://doc.crablang.org/nightly/cargo/commands/cargo-add.html)
- Package ID specs now support `name@version` syntax in addition to the
  previous `name:version` to align with the behavior in `cargo add` and other
  tools. `cargo install` and `cargo yank` also now support this syntax so the
  version does not need to passed as a separate flag.
- The `git` and `registry` directories in Cargo's home directory (usually
  `~/.cargo`) are now marked as cache directories so that they are not
  included in backups or content indexing (on Windows).
- Added automatic `@` argfile support, which will use "response files" if the
  command-line to `crablangc` exceeds the operating system's limit.

Compatibility Notes
-------------------

- `cargo test` now passes `--target` to `crablangdoc` if the specified target is
  the same as the host target.
  [#10594](https://github.com/crablang/cargo/pull/10594)
- [crablangdoc: doctests are now run on unexported `macro_rules!` macros, matching other private items][96630]
- [crablangdoc: Remove .woff font files][96279]
- [Enforce Copy bounds for repeat elements while considering lifetimes][95819]
- [Windows: Fix potentinal unsoundness by aborting if `File` reads or writes cannot
  complete synchronously][95469].

Internal Changes
----------------

- [Unify ReentrantMutex implementations across all platforms][96042]

These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc
and related tools.

[93313]: https://github.com/crablang/crablang/pull/93313/
[93969]: https://github.com/crablang/crablang/pull/93969/
[94206]: https://github.com/crablang/crablang/pull/94206/
[94457]: https://github.com/crablang/crablang/pull/94457/
[94775]: https://github.com/crablang/crablang/pull/94775/
[94872]: https://github.com/crablang/crablang/pull/94872/
[95006]: https://github.com/crablang/crablang/pull/95006/
[95035]: https://github.com/crablang/crablang/pull/95035/
[95346]: https://github.com/crablang/crablang/pull/95346/
[95372]: https://github.com/crablang/crablang/pull/95372/
[95380]: https://github.com/crablang/crablang/pull/95380/
[95431]: https://github.com/crablang/crablang/pull/95431/
[95469]: https://github.com/crablang/crablang/pull/95469/
[95705]: https://github.com/crablang/crablang/pull/95705/
[95801]: https://github.com/crablang/crablang/pull/95801/
[95819]: https://github.com/crablang/crablang/pull/95819/
[95841]: https://github.com/crablang/crablang/pull/95841/
[96042]: https://github.com/crablang/crablang/pull/96042/
[96150]: https://github.com/crablang/crablang/pull/96150/
[96268]: https://github.com/crablang/crablang/pull/96268/
[96279]: https://github.com/crablang/crablang/pull/96279/
[96393]: https://github.com/crablang/crablang/pull/96393/
[96436]: https://github.com/crablang/crablang/pull/96436/
[96557]: https://github.com/crablang/crablang/pull/96557/
[96630]: https://github.com/crablang/crablang/pull/96630/

[`bool::then_some`]: https://doc.crablang.org/stable/std/primitive.bool.html#method.then_some
[`f32::total_cmp`]: https://doc.crablang.org/stable/std/primitive.f32.html#method.total_cmp
[`f64::total_cmp`]: https://doc.crablang.org/stable/std/primitive.f64.html#method.total_cmp
[`Stdin::lines`]: https://doc.crablang.org/stable/std/io/struct.Stdin.html#method.lines
[`impl<T: Default> Default for AssertUnwindSafe<T>`]: https://doc.crablang.org/stable/std/panic/struct.AssertUnwindSafe.html#impl-Default
[rc-u8-from-str]: https://doc.crablang.org/stable/std/rc/struct.Rc.html#impl-From%3CRc%3Cstr%3E%3E
[arc-u8-from-str]: https://doc.crablang.org/stable/std/sync/struct.Arc.html#impl-From%3CArc%3Cstr%3E%3E
[stdarch/1285]: https://github.com/crablang/stdarch/pull/1285
[`windows::CommandExt::raw_arg`]: https://doc.crablang.org/stable/std/os/windows/process/trait.CommandExt.html#tymethod.raw_arg
[`FusedIterator for EncodeWide`]: https://doc.crablang.org/stable/std/os/windows/ffi/struct.EncodeWide.html#impl-FusedIterator

Version 1.61.0 (2022-05-19)
==========================

Language
--------

- [`const fn` signatures can now include generic trait bounds][93827]
- [`const fn` signatures can now use `impl Trait` in argument and return position][93827]
- [Function pointers can now be created, cast, and passed around in a `const fn`][93827]
- [Recursive calls can now set the value of a function's opaque `impl Trait` return type][94081]

Compiler
--------

- [Linking modifier syntax in `#[link]` attributes and on the command line, as well as the `whole-archive` modifier specifically, are now supported][93901]
- [The `char` type is now described as UTF-32 in debuginfo][89887]
- The [`#[target_feature]`][target_feature] attribute [can now be used with aarch64 features][90621]
- X86 [`#[target_feature = "adx"]` is now stable][93745]

Libraries
---------

- [`ManuallyDrop<T>` is now documented to have the same layout as `T`][88375]
- [`#[ignore = "…"]` messages are printed when running tests][92714]
- [Consistently show absent stdio handles on Windows as NULL handles][93263]
- [Make `std::io::stdio::lock()` return `'static` handles.][93965] Previously, the creation of locked handles to stdin/stdout/stderr would borrow the handles being locked, which prevented writing `let out = std::io::stdout().lock();` because `out` would outlive the return value of `stdout()`. Such code now works, eliminating a common pitfall that affected many CrabLang users.
- [`Vec::from_raw_parts` is now less restrictive about its inputs][95016]
- [`std::thread::available_parallelism` now takes cgroup quotas into account.][92697] Since `available_parallelism` is often used to create a thread pool for parallel computation, which may be CPU-bound for performance, `available_parallelism` will return a value consistent with the ability to use that many threads continuously, if possible. For instance, in a container with 8 virtual CPUs but quotas only allowing for 50% usage, `available_parallelism` will return 4.

Stabilized APIs
---------------

- [`Pin::static_mut`]
- [`Pin::static_ref`]
- [`Vec::retain_mut`]
- [`VecDeque::retain_mut`]
- [`Write` for `Cursor<[u8; N]>`][cursor-write-array]
- [`std::os::unix::net::SocketAddr::from_pathname`]
- [`std::process::ExitCode`] and [`std::process::Termination`]. The stabilization of these two APIs now makes it possible for programs to return errors from `main` with custom exit codes.
- [`std::thread::JoinHandle::is_finished`]

These APIs are now usable in const contexts:

- [`<*const T>::offset` and `<*mut T>::offset`][ptr-offset]
- [`<*const T>::wrapping_offset` and `<*mut T>::wrapping_offset`][ptr-wrapping_offset]
- [`<*const T>::add` and `<*mut T>::add`][ptr-add]
- [`<*const T>::sub` and `<*mut T>::sub`][ptr-sub]
- [`<*const T>::wrapping_add` and `<*mut T>::wrapping_add`][ptr-wrapping_add]
- [`<*const T>::wrapping_sub` and `<*mut T>::wrapping_sub`][ptr-wrapping_sub]
- [`<[T]>::as_mut_ptr`][slice-as_mut_ptr]
- [`<[T]>::as_ptr_range`][slice-as_ptr_range]
- [`<[T]>::as_mut_ptr_range`][slice-as_mut_ptr_range]

Cargo
-----

No feature changes, but see compatibility notes.

Compatibility Notes
-------------------

- Previously native static libraries were linked as `whole-archive` in some cases, but now crablangc tries not to use `whole-archive` unless explicitly requested. This [change][93901] may result in linking errors in some cases. To fix such errors, native libraries linked from the command line, build scripts, or [`#[link]` attributes][link-attr] need to
  - (more common) either be reordered to respect dependencies between them (if `a` depends on `b` then `a` should go first and `b` second)
  - (less common) or be updated to use the [`+whole-archive`] modifier.
- [Catching a second unwind from FFI code while cleaning up from a CrabLang panic now causes the process to abort][92911]
- [Proc macros no longer see `ident` matchers wrapped in groups][92472]
- [The number of `#` in `r#` raw string literals is now required to be less than 256][95251]
- [When checking that a dyn type satisfies a trait bound, supertrait bounds are now enforced][92285]
- [`cargo vendor` now only accepts one value for each `--sync` flag][cargo/10448]
- [`cfg` predicates in `all()` and `any()` are always evaluated to detect errors, instead of short-circuiting.][94295] The compatibility considerations here arise in nightly-only code that used the short-circuiting behavior of `all` to write something like `cfg(all(feature = "nightly", syntax-requiring-nightly))`, which will now fail to compile. Instead, use either `cfg_attr(feature = "nightly", ...)` or nested uses of `cfg`.
- [bootstrap: static-libstdcpp is now enabled by default, and can now be disabled when llvm-tools is enabled][94832]

Internal Changes
----------------

These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc
and related tools.

- [debuginfo: Refactor debuginfo generation for types][94261]
- [Remove the everybody loops pass][93913]

[88375]: https://github.com/crablang/crablang/pull/88375/
[89887]: https://github.com/crablang/crablang/pull/89887/
[90621]: https://github.com/crablang/crablang/pull/90621/
[92285]: https://github.com/crablang/crablang/pull/92285/
[92472]: https://github.com/crablang/crablang/pull/92472/
[92697]: https://github.com/crablang/crablang/pull/92697/
[92714]: https://github.com/crablang/crablang/pull/92714/
[92911]: https://github.com/crablang/crablang/pull/92911/
[93263]: https://github.com/crablang/crablang/pull/93263/
[93745]: https://github.com/crablang/crablang/pull/93745/
[93827]: https://github.com/crablang/crablang/pull/93827/
[93901]: https://github.com/crablang/crablang/pull/93901/
[93913]: https://github.com/crablang/crablang/pull/93913/
[93965]: https://github.com/crablang/crablang/pull/93965/
[94081]: https://github.com/crablang/crablang/pull/94081/
[94261]: https://github.com/crablang/crablang/pull/94261/
[94295]: https://github.com/crablang/crablang/pull/94295/
[94832]: https://github.com/crablang/crablang/pull/94832/
[95016]: https://github.com/crablang/crablang/pull/95016/
[95251]: https://github.com/crablang/crablang/pull/95251/
[`+whole-archive`]: https://doc.crablang.org/stable/crablangc/command-line-arguments.html#linking-modifiers-whole-archive
[`Pin::static_mut`]: https://doc.crablang.org/stable/std/pin/struct.Pin.html#method.static_mut
[`Pin::static_ref`]: https://doc.crablang.org/stable/std/pin/struct.Pin.html#method.static_ref
[`Vec::retain_mut`]: https://doc.crablang.org/stable/std/vec/struct.Vec.html#method.retain_mut
[`VecDeque::retain_mut`]: https://doc.crablang.org/stable/std/collections/struct.VecDeque.html#method.retain_mut
[`std::os::unix::net::SocketAddr::from_pathname`]: https://doc.crablang.org/stable/std/os/unix/net/struct.SocketAddr.html#method.from_pathname
[`std::process::ExitCode`]: https://doc.crablang.org/stable/std/process/struct.ExitCode.html
[`std::process::Termination`]: https://doc.crablang.org/stable/std/process/trait.Termination.html
[`std::thread::JoinHandle::is_finished`]: https://doc.crablang.org/stable/std/thread/struct.JoinHandle.html#method.is_finished
[cargo/10448]: https://github.com/crablang/cargo/pull/10448/
[cursor-write-array]: https://doc.crablang.org/stable/std/io/struct.Cursor.html#impl-Write-4
[link-attr]: https://doc.crablang.org/stable/reference/items/external-blocks.html#the-link-attribute
[ptr-add]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.add
[ptr-offset]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.offset
[ptr-sub]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.sub
[ptr-wrapping_add]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.wrapping_add
[ptr-wrapping_offset]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.wrapping_offset
[ptr-wrapping_sub]: https://doc.crablang.org/stable/std/primitive.pointer.html#method.wrapping_sub
[slice-as_mut_ptr]: https://doc.crablang.org/stable/std/primitive.slice.html#method.as_mut_ptr
[slice-as_mut_ptr_range]: https://doc.crablang.org/stable/std/primitive.slice.html#method.as_mut_ptr_range
[slice-as_ptr_range]: https://doc.crablang.org/stable/std/primitive.slice.html#method.as_ptr_range
[target_feature]: https://doc.crablang.org/reference/attributes/codegen.html#the-target_feature-attribute


Version 1.60.0 (2022-04-07)
==========================

Language
--------
- [Stabilize `#[cfg(panic = "...")]` for either `"unwind"` or `"abort"`.][93658]
- [Stabilize `#[cfg(target_has_atomic = "...")]` for each integer size and `"ptr"`.][93824]

Compiler
--------
- [Enable combining `+crt-static` and `relocation-model=pic` on `x86_64-unknown-linux-gnu`][86374]
- [Fixes wrong `unreachable_pub` lints on nested and glob public reexport][87487]
- [Stabilize `-Z instrument-coverage` as `-C instrument-coverage`][90132]
- [Stabilize `-Z print-link-args` as `--print link-args`][91606]
- [Add new Tier 3 target `mips64-openwrt-linux-musl`\*][92300]
- [Add new Tier 3 target `armv7-unknown-linux-uclibceabi` (softfloat)\*][92383]
- [Fix invalid removal of newlines from doc comments][92357]
- [Add kernel target for CrabLangyHermit][92670]
- [Deny mixing bin crate type with lib crate types][92933]
- [Make crablangc use `CRABLANG_BACKTRACE=full` by default][93566]
- [Upgrade to LLVM 14][93577]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
   information on CrabLang's tiered platform support.

Libraries
---------
- [Guarantee call order for `sort_by_cached_key`][89621]
- [Improve `Duration::try_from_secs_f32`/`f64` accuracy by directly processing exponent and mantissa][90247]
- [Make `Instant::{duration_since, elapsed, sub}` saturating][89926]
- [Remove non-monotonic clocks workarounds in `Instant::now`][89926]
- [Make `BuildHasherDefault`, `iter::Empty` and `future::Pending` covariant][92630]

Stabilized APIs
---------------
- [`Arc::new_cyclic`][arc_new_cyclic]
- [`Rc::new_cyclic`][rc_new_cyclic]
- [`slice::EscapeAscii`][slice_escape_ascii]
- [`<[u8]>::escape_ascii`][slice_u8_escape_ascii]
- [`u8::escape_ascii`][u8_escape_ascii]
- [`Vec::spare_capacity_mut`][vec_spare_capacity_mut]
- [`MaybeUninit::assume_init_drop`][assume_init_drop]
- [`MaybeUninit::assume_init_read`][assume_init_read]
- [`i8::abs_diff`][i8_abs_diff]
- [`i16::abs_diff`][i16_abs_diff]
- [`i32::abs_diff`][i32_abs_diff]
- [`i64::abs_diff`][i64_abs_diff]
- [`i128::abs_diff`][i128_abs_diff]
- [`isize::abs_diff`][isize_abs_diff]
- [`u8::abs_diff`][u8_abs_diff]
- [`u16::abs_diff`][u16_abs_diff]
- [`u32::abs_diff`][u32_abs_diff]
- [`u64::abs_diff`][u64_abs_diff]
- [`u128::abs_diff`][u128_abs_diff]
- [`usize::abs_diff`][usize_abs_diff]
- [`Display for io::ErrorKind`][display_error_kind]
- [`From<u8> for ExitCode`][from_u8_exit_code]
- [`Not for !` (the "never" type)][not_never]
- [_Op_`Assign<$t> for Wrapping<$t>`][wrapping_assign_ops]
- [`arch::is_aarch64_feature_detected!`][is_aarch64_feature_detected]

Cargo
-----
- [Port cargo from `toml-rs` to `toml_edit`][cargo/10086]
- [Stabilize `-Ztimings` as `--timings`][cargo/10245]
- [Stabilize namespaced and weak dependency features.][cargo/10269]
- [Accept more `cargo:crablangc-link-arg-*` types from build script output.][cargo/10274]
- [cargo-new should not add ignore rule on Cargo.lock inside subdirs][cargo/10379]

Misc
----
- [Ship docs on Tier 2 platforms by reusing the closest Tier 1 platform docs][92800]
- [Drop crablangc-docs from complete profile][93742]
- [bootstrap: tidy up flag handling for llvm build][93918]

Compatibility Notes
-------------------
- [Remove compiler-rt linking hack on Android][83822]
- [Mitigations for platforms with non-monotonic clocks have been removed from
  `Instant::now`][89926]. On platforms that don't provide monotonic clocks, an
  instant is not guaranteed to be greater than an earlier instant anymore.
- [`Instant::{duration_since, elapsed, sub}` do not panic anymore on underflow,
  saturating to `0` instead][89926]. In the real world the panic happened mostly
  on platforms with buggy monotonic clock implementations rather than catching
  programming errors like reversing the start and end times. Such programming
  errors will now results in `0` rather than a panic.
- In a future release we're planning to increase the baseline requirements for
  the Linux kernel to version 3.2, and for glibc to version 2.17. We'd love
  your feedback in [PR #95026][95026].

Internal Changes
----------------

These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc
and related tools.

- [Switch all libraries to the 2021 edition][92068]

[83822]: https://github.com/crablang/crablang/pull/83822
[86374]: https://github.com/crablang/crablang/pull/86374
[87487]: https://github.com/crablang/crablang/pull/87487
[89621]: https://github.com/crablang/crablang/pull/89621
[89926]: https://github.com/crablang/crablang/pull/89926
[90132]: https://github.com/crablang/crablang/pull/90132
[90247]: https://github.com/crablang/crablang/pull/90247
[91606]: https://github.com/crablang/crablang/pull/91606
[92068]: https://github.com/crablang/crablang/pull/92068
[92300]: https://github.com/crablang/crablang/pull/92300
[92357]: https://github.com/crablang/crablang/pull/92357
[92383]: https://github.com/crablang/crablang/pull/92383
[92630]: https://github.com/crablang/crablang/pull/92630
[92670]: https://github.com/crablang/crablang/pull/92670
[92800]: https://github.com/crablang/crablang/pull/92800
[92933]: https://github.com/crablang/crablang/pull/92933
[93566]: https://github.com/crablang/crablang/pull/93566
[93577]: https://github.com/crablang/crablang/pull/93577
[93658]: https://github.com/crablang/crablang/pull/93658
[93742]: https://github.com/crablang/crablang/pull/93742
[93824]: https://github.com/crablang/crablang/pull/93824
[93918]: https://github.com/crablang/crablang/pull/93918
[95026]: https://github.com/crablang/crablang/pull/95026

[cargo/10086]: https://github.com/crablang/cargo/pull/10086
[cargo/10245]: https://github.com/crablang/cargo/pull/10245
[cargo/10269]: https://github.com/crablang/cargo/pull/10269
[cargo/10274]: https://github.com/crablang/cargo/pull/10274
[cargo/10379]: https://github.com/crablang/cargo/pull/10379

[arc_new_cyclic]: https://doc.crablang.org/stable/std/sync/struct.Arc.html#method.new_cyclic
[rc_new_cyclic]: https://doc.crablang.org/stable/std/rc/struct.Rc.html#method.new_cyclic
[slice_escape_ascii]: https://doc.crablang.org/stable/std/slice/struct.EscapeAscii.html
[slice_u8_escape_ascii]: https://doc.crablang.org/stable/std/primitive.slice.html#method.escape_ascii
[u8_escape_ascii]: https://doc.crablang.org/stable/std/primitive.u8.html#method.escape_ascii
[vec_spare_capacity_mut]: https://doc.crablang.org/stable/std/vec/struct.Vec.html#method.spare_capacity_mut
[assume_init_drop]: https://doc.crablang.org/stable/std/mem/union.MaybeUninit.html#method.assume_init_drop
[assume_init_read]: https://doc.crablang.org/stable/std/mem/union.MaybeUninit.html#method.assume_init_read
[i8_abs_diff]: https://doc.crablang.org/stable/std/primitive.i8.html#method.abs_diff
[i16_abs_diff]: https://doc.crablang.org/stable/std/primitive.i16.html#method.abs_diff
[i32_abs_diff]: https://doc.crablang.org/stable/std/primitive.i32.html#method.abs_diff
[i64_abs_diff]: https://doc.crablang.org/stable/std/primitive.i64.html#method.abs_diff
[i128_abs_diff]: https://doc.crablang.org/stable/std/primitive.i128.html#method.abs_diff
[isize_abs_diff]: https://doc.crablang.org/stable/std/primitive.isize.html#method.abs_diff
[u8_abs_diff]: https://doc.crablang.org/stable/std/primitive.u8.html#method.abs_diff
[u16_abs_diff]: https://doc.crablang.org/stable/std/primitive.u16.html#method.abs_diff
[u32_abs_diff]: https://doc.crablang.org/stable/std/primitive.u32.html#method.abs_diff
[u64_abs_diff]: https://doc.crablang.org/stable/std/primitive.u64.html#method.abs_diff
[u128_abs_diff]: https://doc.crablang.org/stable/std/primitive.u128.html#method.abs_diff
[usize_abs_diff]: https://doc.crablang.org/stable/std/primitive.usize.html#method.abs_diff
[display_error_kind]: https://doc.crablang.org/stable/std/io/enum.ErrorKind.html#impl-Display
[from_u8_exit_code]: https://doc.crablang.org/stable/std/process/struct.ExitCode.html#impl-From%3Cu8%3E
[not_never]: https://doc.crablang.org/stable/std/primitive.never.html#impl-Not
[wrapping_assign_ops]: https://doc.crablang.org/stable/std/num/struct.Wrapping.html#trait-implementations
[is_aarch64_feature_detected]: https://doc.crablang.org/stable/std/arch/macro.is_aarch64_feature_detected.html

Version 1.59.0 (2022-02-24)
==========================

Language
--------

- [Stabilize default arguments for const parameters and remove the ordering restriction for type and const parameters][90207]
- [Stabilize destructuring assignment][90521]
- [Relax private in public lint on generic bounds and where clauses of trait impls][90586]
- [Stabilize asm! and global_asm! for x86, x86_64, ARM, Aarch64, and RISC-V][91728]

Compiler
--------

- [Stabilize new symbol mangling format, leaving it opt-in (-Csymbol-mangling-version=v0)][90128]
- [Emit LLVM optimization remarks when enabled with `-Cremark`][90833]
- [Fix sparc64 ABI for aggregates with floating point members][91003]
- [Warn when a `#[test]`-like built-in attribute macro is present multiple times.][91172]
- [Add support for riscv64gc-unknown-freebsd][91284]
- [Stabilize `-Z emit-future-incompat` as `--json future-incompat`][91535]
- [Soft disable incremental compilation][94124]

This release disables incremental compilation, unless the user has explicitly
opted in via the newly added CRABLANGC_FORCE_INCREMENTAL=1 environment variable.
This is due to a known and relatively frequently occurring bug in incremental
compilation, which causes builds to issue internal compiler errors. This
particular bug is already fixed on nightly, but that fix has not yet rolled out
to stable and is deemed too risky for a direct stable backport.

As always, we encourage users to test with nightly and report bugs so that we
can track failures and fix issues earlier.

See [94124] for more details.

[94124]: https://github.com/crablang/crablang/issues/94124

Libraries
---------

- [Remove unnecessary bounds for some Hash{Map,Set} methods][91593]

Stabilized APIs
---------------

- [`std::thread::available_parallelism`][available_parallelism]
- [`Result::copied`][result-copied]
- [`Result::cloned`][result-cloned]
- [`arch::asm!`][asm]
- [`arch::global_asm!`][global_asm]
- [`ops::ControlFlow::is_break`][is_break]
- [`ops::ControlFlow::is_continue`][is_continue]
- [`TryFrom<char> for u8`][try_from_char_u8]
- [`char::TryFromCharError`][try_from_char_err]
  implementing `Clone`, `Debug`, `Display`, `PartialEq`, `Copy`, `Eq`, `Error`
- [`iter::zip`][zip]
- [`NonZeroU8::is_power_of_two`][is_power_of_two8]
- [`NonZeroU16::is_power_of_two`][is_power_of_two16]
- [`NonZeroU32::is_power_of_two`][is_power_of_two32]
- [`NonZeroU64::is_power_of_two`][is_power_of_two64]
- [`NonZeroU128::is_power_of_two`][is_power_of_two128]
- [`NonZeroUsize::is_power_of_two`][is_power_of_two_usize]
- [`DoubleEndedIterator for ToLowercase`][lowercase]
- [`DoubleEndedIterator for ToUppercase`][uppercase]
- [`TryFrom<&mut [T]> for [T; N]`][tryfrom_ref_arr]
- [`UnwindSafe for Once`][unwindsafe_once]
- [`RefUnwindSafe for Once`][refunwindsafe_once]
- [armv8 neon intrinsics for aarch64][stdarch/1266]

Const-stable:

- [`mem::MaybeUninit::as_ptr`][muninit_ptr]
- [`mem::MaybeUninit::assume_init`][muninit_init]
- [`mem::MaybeUninit::assume_init_ref`][muninit_init_ref]
- [`ffi::CStr::from_bytes_with_nul_unchecked`][cstr_from_bytes]

Cargo
-----

- [Stabilize the `strip` profile option][cargo/10088]
- [Stabilize future-incompat-report][cargo/10165]
- [Support abbreviating `--release` as `-r`][cargo/10133]
- [Support `term.quiet` configuration][cargo/10152]
- [Remove `--host` from cargo {publish,search,login}][cargo/10145]

Compatibility Notes
-------------------

- [Refactor weak symbols in std::sys::unix][90846]
  This may add new, versioned, symbols when building with a newer glibc, as the
  standard library uses weak linkage rather than dynamically attempting to load
  certain symbols at runtime.
- [Deprecate crate_type and crate_name nested inside `#![cfg_attr]`][83744]
  This adds a future compatibility lint to supporting the use of cfg_attr
  wrapping either crate_type or crate_name specification within CrabLang files;
  it is recommended that users migrate to setting the equivalent command line
  flags.
- [Remove effect of `#[no_link]` attribute on name resolution][92034]
  This may expose new names, leading to conflicts with preexisting names in a
  given namespace and a compilation failure.
- [Cargo will document libraries before binaries.][cargo/10172]
- [Respect doc=false in dependencies, not just the root crate][cargo/10201]
- [Weaken guarantee around advancing underlying iterators in zip][83791]
- [Make split_inclusive() on an empty slice yield an empty output][89825]
- [Update std::env::temp_dir to use GetTempPath2 on Windows when available.][89999]
- [unreachable! was updated to match other formatting macro behavior on CrabLang 2021][92137]

Internal Changes
----------------

These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc
and related tools.

- [Fix many cases of normalization-related ICEs][91255]
- [Replace dominators algorithm with simple Lengauer-Tarjan][85013]
- [Store liveness in interval sets for region inference][90637]

- [Remove `in_band_lifetimes` from the compiler and standard library, in preparation for removing this
  unstable feature.][91867]

[91867]: https://github.com/crablang/crablang/issues/91867
[83744]: https://github.com/crablang/crablang/pull/83744/
[83791]: https://github.com/crablang/crablang/pull/83791/
[85013]: https://github.com/crablang/crablang/pull/85013/
[89825]: https://github.com/crablang/crablang/pull/89825/
[89999]: https://github.com/crablang/crablang/pull/89999/
[90128]: https://github.com/crablang/crablang/pull/90128/
[90207]: https://github.com/crablang/crablang/pull/90207/
[90521]: https://github.com/crablang/crablang/pull/90521/
[90586]: https://github.com/crablang/crablang/pull/90586/
[90637]: https://github.com/crablang/crablang/pull/90637/
[90833]: https://github.com/crablang/crablang/pull/90833/
[90846]: https://github.com/crablang/crablang/pull/90846/
[91003]: https://github.com/crablang/crablang/pull/91003/
[91172]: https://github.com/crablang/crablang/pull/91172/
[91255]: https://github.com/crablang/crablang/pull/91255/
[91284]: https://github.com/crablang/crablang/pull/91284/
[91535]: https://github.com/crablang/crablang/pull/91535/
[91593]: https://github.com/crablang/crablang/pull/91593/
[91728]: https://github.com/crablang/crablang/pull/91728/
[91878]: https://github.com/crablang/crablang/pull/91878/
[91896]: https://github.com/crablang/crablang/pull/91896/
[91926]: https://github.com/crablang/crablang/pull/91926/
[91984]: https://github.com/crablang/crablang/pull/91984/
[92020]: https://github.com/crablang/crablang/pull/92020/
[92034]: https://github.com/crablang/crablang/pull/92034/
[92137]: https://github.com/crablang/crablang/pull/92137/
[92483]: https://github.com/crablang/crablang/pull/92483/
[cargo/10088]: https://github.com/crablang/cargo/pull/10088/
[cargo/10133]: https://github.com/crablang/cargo/pull/10133/
[cargo/10145]: https://github.com/crablang/cargo/pull/10145/
[cargo/10152]: https://github.com/crablang/cargo/pull/10152/
[cargo/10165]: https://github.com/crablang/cargo/pull/10165/
[cargo/10172]: https://github.com/crablang/cargo/pull/10172/
[cargo/10201]: https://github.com/crablang/cargo/pull/10201/
[cargo/10269]: https://github.com/crablang/cargo/pull/10269/

[cstr_from_bytes]: https://doc.crablang.org/stable/std/ffi/struct.CStr.html#method.from_bytes_with_nul_unchecked
[muninit_ptr]: https://doc.crablang.org/stable/std/mem/union.MaybeUninit.html#method.as_ptr
[muninit_init]: https://doc.crablang.org/stable/std/mem/union.MaybeUninit.html#method.assume_init
[muninit_init_ref]: https://doc.crablang.org/stable/std/mem/union.MaybeUninit.html#method.assume_init_ref
[unwindsafe_once]: https://doc.crablang.org/stable/std/sync/struct.Once.html#impl-UnwindSafe
[refunwindsafe_once]: https://doc.crablang.org/stable/std/sync/struct.Once.html#impl-RefUnwindSafe
[tryfrom_ref_arr]: https://doc.crablang.org/stable/std/convert/trait.TryFrom.html#impl-TryFrom%3C%26%27_%20mut%20%5BT%5D%3E
[lowercase]: https://doc.crablang.org/stable/std/char/struct.ToLowercase.html#impl-DoubleEndedIterator
[uppercase]: https://doc.crablang.org/stable/std/char/struct.ToUppercase.html#impl-DoubleEndedIterator
[try_from_char_err]: https://doc.crablang.org/stable/std/char/struct.TryFromCharError.html
[available_parallelism]: https://doc.crablang.org/stable/std/thread/fn.available_parallelism.html
[result-copied]: https://doc.crablang.org/stable/std/result/enum.Result.html#method.copied
[result-cloned]: https://doc.crablang.org/stable/std/result/enum.Result.html#method.cloned
[asm]: https://doc.crablang.org/stable/core/arch/macro.asm.html
[global_asm]: https://doc.crablang.org/stable/core/arch/macro.global_asm.html
[is_break]: https://doc.crablang.org/stable/std/ops/enum.ControlFlow.html#method.is_break
[is_continue]: https://doc.crablang.org/stable/std/ops/enum.ControlFlow.html#method.is_continue
[try_from_char_u8]: https://doc.crablang.org/stable/std/primitive.char.html#impl-TryFrom%3Cchar%3E
[zip]: https://doc.crablang.org/stable/std/iter/fn.zip.html
[is_power_of_two8]: https://doc.crablang.org/stable/core/num/struct.NonZeroU8.html#method.is_power_of_two
[is_power_of_two16]: https://doc.crablang.org/stable/core/num/struct.NonZeroU16.html#method.is_power_of_two
[is_power_of_two32]: https://doc.crablang.org/stable/core/num/struct.NonZeroU32.html#method.is_power_of_two
[is_power_of_two64]: https://doc.crablang.org/stable/core/num/struct.NonZeroU64.html#method.is_power_of_two
[is_power_of_two128]: https://doc.crablang.org/stable/core/num/struct.NonZeroU128.html#method.is_power_of_two
[is_power_of_two_usize]: https://doc.crablang.org/stable/core/num/struct.NonZeroUsize.html#method.is_power_of_two
[stdarch/1266]: https://github.com/crablang/stdarch/pull/1266

Version 1.58.1 (2022-01-19)
===========================

* Fix race condition in `std::fs::remove_dir_all` ([CVE-2022-21658])
* [Handle captured arguments in the `useless_format` Clippy lint][clippy/8295]
* [Move `non_send_fields_in_send_ty` Clippy lint to nursery][clippy/8075]
* [Fix wrong error message displayed when some imports are missing][91254]
* [Fix crablangfmt not formatting generated files from stdin][92912]

[CVE-2022-21658]: https://www.cve.org/CVERecord?id=CVE-2022-21658
[91254]: https://github.com/crablang/crablang/pull/91254
[92912]: https://github.com/crablang/crablang/pull/92912
[clippy/8075]: https://github.com/crablang/crablang-clippy/pull/8075
[clippy/8295]: https://github.com/crablang/crablang-clippy/pull/8295

Version 1.58.0 (2022-01-13)
==========================

Language
--------

- [Format strings can now capture arguments simply by writing `{ident}` in the string.][90473] This works in all macros accepting format strings. Support for this in `panic!` (`panic!("{ident}")`) requires the 2021 edition; panic invocations in previous editions that appear to be trying to use this will result in a warning lint about not having the intended effect.
- [`*const T` pointers can now be dereferenced in const contexts.][89551]
- [The rules for when a generic struct implements `Unsize` have been relaxed.][90417]

Compiler
--------

- [Add LLVM CFI support to the CrabLang compiler][89652]
- [Stabilize -Z strip as -C strip][90058]. Note that while release builds already don't add debug symbols for the code you compile, the compiled standard library that ships with CrabLang includes debug symbols, so you may want to use the `strip` option to remove these symbols to produce smaller release binaries. Note that this release only includes support in crablangc, not directly in cargo.
- [Add support for LLVM coverage mapping format versions 5 and 6][91207]
- [Emit LLVM optimization remarks when enabled with `-Cremark`][90833]
- [Update the minimum external LLVM to 12][90175]
- [Add `x86_64-unknown-none` at Tier 3*][89062]
- [Build musl dist artifacts with debuginfo enabled][90733]. When building release binaries using musl, you may want to use the newly stabilized strip option to remove these debug symbols, reducing the size of your binaries.
- [Don't abort compilation after giving a lint error][87337]
- [Error messages point at the source of trait bound obligations in more places][89580]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
   information on CrabLang's tiered platform support.

Libraries
---------

- [All remaining functions in the standard library have `#[must_use]` annotations where appropriate][89692], producing a warning when ignoring their return value. This helps catch mistakes such as expecting a function to mutate a value in place rather than return a new value.
- [Paths are automatically canonicalized on Windows for operations that support it][89174]
- [Re-enable debug checks for `copy` and `copy_nonoverlapping`][90041]
- [Implement `RefUnwindSafe` for `Rc<T>`][87467]
- [Make RSplit<T, P>: Clone not require T: Clone][90117]
- [Implement `Termination` for `Result<Infallible, E>`][88601]. This allows writing `fn main() -> Result<Infallible, ErrorType>`, for a program whose successful exits never involve returning from `main` (for instance, a program that calls `exit`, or that uses `exec` to run another program).

Stabilized APIs
---------------

- [`Metadata::is_symlink`]
- [`Path::is_symlink`]
- [`{integer}::saturating_div`]
- [`Option::unwrap_unchecked`]
- [`Result::unwrap_unchecked`]
- [`Result::unwrap_err_unchecked`]
- [`File::options`]

These APIs are now usable in const contexts:

- [`Duration::new`]
- [`Duration::checked_add`]
- [`Duration::saturating_add`]
- [`Duration::checked_sub`]
- [`Duration::saturating_sub`]
- [`Duration::checked_mul`]
- [`Duration::saturating_mul`]
- [`Duration::checked_div`]

Cargo
-----

- [Add --message-format for install command][cargo/10107]
- [Warn when alias shadows external subcommand][cargo/10082]

CrabLangdoc
-------

- [Show all Deref implementations recursively in crablangdoc][90183]
- [Use computed visibility in crablangdoc][88447]

Compatibility Notes
-------------------

- [Try all stable method candidates first before trying unstable ones][90329]. This change ensures that adding new nightly-only methods to the CrabLang standard library will not break code invoking methods of the same name from traits outside the standard library.
- Windows: [`std::process::Command` will no longer search the current directory for executables.][87704]
- [All proc-macro backward-compatibility lints are now deny-by-default.][88041]
- [proc_macro: Append .0 to unsuffixed float if it would otherwise become int token][90297]
- [Refactor weak symbols in std::sys::unix][90846]. This optimizes accesses to glibc functions, by avoiding the use of dlopen. This does not increase the [minimum expected version of glibc](https://doc.crablang.org/nightly/crablangc/platform-support.html). However, software distributions that use symbol versions to detect library dependencies, and which take weak symbols into account in that analysis, may detect crablang binaries as requiring newer versions of glibc.
- [crablangdoc now rejects some unexpected semicolons in doctests][91026]

Internal Changes
----------------

These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc
and related tools.

- [Implement coherence checks for negative trait impls][90104]
- [Add crablangc lint, warning when iterating over hashmaps][89558]
- [Optimize live point computation][90491]
- [Enable verification for 1/32nd of queries loaded from disk][90361]
- [Implement version of normalize_erasing_regions that allows for normalization failure][91255]

[87337]: https://github.com/crablang/crablang/pull/87337/
[87467]: https://github.com/crablang/crablang/pull/87467/
[87704]: https://github.com/crablang/crablang/pull/87704/
[88041]: https://github.com/crablang/crablang/pull/88041/
[88447]: https://github.com/crablang/crablang/pull/88447/
[88601]: https://github.com/crablang/crablang/pull/88601/
[89062]: https://github.com/crablang/crablang/pull/89062/
[89174]: https://github.com/crablang/crablang/pull/89174/
[89551]: https://github.com/crablang/crablang/pull/89551/
[89558]: https://github.com/crablang/crablang/pull/89558/
[89580]: https://github.com/crablang/crablang/pull/89580/
[89652]: https://github.com/crablang/crablang/pull/89652/
[90041]: https://github.com/crablang/crablang/pull/90041/
[90058]: https://github.com/crablang/crablang/pull/90058/
[90104]: https://github.com/crablang/crablang/pull/90104/
[90117]: https://github.com/crablang/crablang/pull/90117/
[90175]: https://github.com/crablang/crablang/pull/90175/
[90183]: https://github.com/crablang/crablang/pull/90183/
[90297]: https://github.com/crablang/crablang/pull/90297/
[90329]: https://github.com/crablang/crablang/pull/90329/
[90361]: https://github.com/crablang/crablang/pull/90361/
[90417]: https://github.com/crablang/crablang/pull/90417/
[90473]: https://github.com/crablang/crablang/pull/90473/
[90491]: https://github.com/crablang/crablang/pull/90491/
[90733]: https://github.com/crablang/crablang/pull/90733/
[90833]: https://github.com/crablang/crablang/pull/90833/
[90846]: https://github.com/crablang/crablang/pull/90846/
[91026]: https://github.com/crablang/crablang/pull/91026/
[91207]: https://github.com/crablang/crablang/pull/91207/
[91255]: https://github.com/crablang/crablang/pull/91255/
[cargo/10082]: https://github.com/crablang/cargo/pull/10082/
[cargo/10107]: https://github.com/crablang/cargo/pull/10107/
[`Metadata::is_symlink`]: https://doc.crablang.org/stable/std/fs/struct.Metadata.html#method.is_symlink
[`Path::is_symlink`]: https://doc.crablang.org/stable/std/path/struct.Path.html#method.is_symlink
[`{integer}::saturating_div`]: https://doc.crablang.org/stable/std/primitive.i8.html#method.saturating_div
[`Option::unwrap_unchecked`]: https://doc.crablang.org/stable/std/option/enum.Option.html#method.unwrap_unchecked
[`Result::unwrap_unchecked`]: https://doc.crablang.org/stable/std/result/enum.Result.html#method.unwrap_unchecked
[`Result::unwrap_err_unchecked`]: https://doc.crablang.org/stable/std/result/enum.Result.html#method.unwrap_err_unchecked
[`File::options`]: https://doc.crablang.org/stable/std/fs/struct.File.html#method.options
[`Duration::new`]: https://doc.crablang.org/stable/std/time/struct.Duration.html#method.new

Version 1.57.0 (2021-12-02)
==========================

Language
--------

- [Macro attributes may follow `#[derive]` and will see the original (pre-`cfg`) input.][87220]
- [Accept curly-brace macros in expressions, like `m!{ .. }.method()` and `m!{ .. }?`.][88690]
- [Allow panicking in constant evaluation.][89508]
- [Ignore derived `Clone` and `Debug` implementations during dead code analysis.][85200]

Compiler
--------

- [Create more accurate debuginfo for vtables.][89597]
- [Add `armv6k-nintendo-3ds` at Tier 3\*.][88529]
- [Add `armv7-unknown-linux-uclibceabihf` at Tier 3\*.][88952]
- [Add `m68k-unknown-linux-gnu` at Tier 3\*.][88321]
- [Add SOLID targets at Tier 3\*:][86191] `aarch64-kmc-solid_asp3`, `armv7a-kmc-solid_asp3-eabi`, `armv7a-kmc-solid_asp3-eabihf`

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
   information on CrabLang's tiered platform support.

Libraries
---------

- [Avoid allocations and copying in `Vec::leak`][89337]
- [Add `#[repr(i8)]` to `Ordering`][89507]
- [Optimize `File::read_to_end` and `read_to_string`][89582]
- [Update to Unicode 14.0][89614]
- [Many more functions are marked `#[must_use]`][89692], producing a warning
  when ignoring their return value. This helps catch mistakes such as expecting
  a function to mutate a value in place rather than return a new value.

Stabilised APIs
---------------

- [`[T; N]::as_mut_slice`][`array::as_mut_slice`]
- [`[T; N]::as_slice`][`array::as_slice`]
- [`collections::TryReserveError`]
- [`HashMap::try_reserve`]
- [`HashSet::try_reserve`]
- [`String::try_reserve`]
- [`String::try_reserve_exact`]
- [`Vec::try_reserve`]
- [`Vec::try_reserve_exact`]
- [`VecDeque::try_reserve`]
- [`VecDeque::try_reserve_exact`]
- [`Iterator::map_while`]
- [`iter::MapWhile`]
- [`proc_macro::is_available`]
- [`Command::get_program`]
- [`Command::get_args`]
- [`Command::get_envs`]
- [`Command::get_current_dir`]
- [`CommandArgs`]
- [`CommandEnvs`]

These APIs are now usable in const contexts:

- [`hint::unreachable_unchecked`]

Cargo
-----

- [Stabilize custom profiles][cargo/9943]

Compatibility notes
-------------------

- [Ignore derived `Clone` and `Debug` implementations during dead code analysis.][85200]
  This will break some builds that set `#![deny(dead_code)]`.

Internal changes
----------------
These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc
and related tools.

- [Added an experimental backend for codegen with `libgccjit`.][87260]

[85200]: https://github.com/crablang/crablang/pull/85200/
[86191]: https://github.com/crablang/crablang/pull/86191/
[87220]: https://github.com/crablang/crablang/pull/87220/
[87260]: https://github.com/crablang/crablang/pull/87260/
[88321]: https://github.com/crablang/crablang/pull/88321/
[88529]: https://github.com/crablang/crablang/pull/88529/
[88690]: https://github.com/crablang/crablang/pull/88690/
[88952]: https://github.com/crablang/crablang/pull/88952/
[89337]: https://github.com/crablang/crablang/pull/89337/
[89507]: https://github.com/crablang/crablang/pull/89507/
[89508]: https://github.com/crablang/crablang/pull/89508/
[89582]: https://github.com/crablang/crablang/pull/89582/
[89597]: https://github.com/crablang/crablang/pull/89597/
[89614]: https://github.com/crablang/crablang/pull/89614/
[89692]: https://github.com/crablang/crablang/issues/89692/
[cargo/9943]: https://github.com/crablang/cargo/pull/9943/
[`array::as_mut_slice`]: https://doc.crablang.org/std/primitive.array.html#method.as_mut_slice
[`array::as_slice`]: https://doc.crablang.org/std/primitive.array.html#method.as_slice
[`collections::TryReserveError`]: https://doc.crablang.org/std/collections/struct.TryReserveError.html
[`HashMap::try_reserve`]: https://doc.crablang.org/std/collections/hash_map/struct.HashMap.html#method.try_reserve
[`HashSet::try_reserve`]: https://doc.crablang.org/std/collections/hash_set/struct.HashSet.html#method.try_reserve
[`String::try_reserve`]: https://doc.crablang.org/alloc/string/struct.String.html#method.try_reserve
[`String::try_reserve_exact`]: https://doc.crablang.org/alloc/string/struct.String.html#method.try_reserve_exact
[`Vec::try_reserve`]: https://doc.crablang.org/std/vec/struct.Vec.html#method.try_reserve
[`Vec::try_reserve_exact`]: https://doc.crablang.org/std/vec/struct.Vec.html#method.try_reserve_exact
[`VecDeque::try_reserve`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.try_reserve
[`VecDeque::try_reserve_exact`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.try_reserve_exact
[`Iterator::map_while`]: https://doc.crablang.org/std/iter/trait.Iterator.html#method.map_while
[`iter::MapWhile`]: https://doc.crablang.org/std/iter/struct.MapWhile.html
[`proc_macro::is_available`]: https://doc.crablang.org/proc_macro/fn.is_available.html
[`Command::get_program`]: https://doc.crablang.org/std/process/struct.Command.html#method.get_program
[`Command::get_args`]: https://doc.crablang.org/std/process/struct.Command.html#method.get_args
[`Command::get_envs`]: https://doc.crablang.org/std/process/struct.Command.html#method.get_envs
[`Command::get_current_dir`]: https://doc.crablang.org/std/process/struct.Command.html#method.get_current_dir
[`CommandArgs`]: https://doc.crablang.org/std/process/struct.CommandArgs.html
[`CommandEnvs`]: https://doc.crablang.org/std/process/struct.CommandEnvs.html

Version 1.56.1 (2021-11-01)
===========================

- New lints to detect the presence of bidirectional-override Unicode
  codepoints in the compiled source code ([CVE-2021-42574])

[CVE-2021-42574]: https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2021-42574

Version 1.56.0 (2021-10-21)
========================

Language
--------

- [The 2021 Edition is now stable.][crablang#88100]
  See [the edition guide][crablang-2021-edition-guide] for more details.
- [The pattern in `binding @ pattern` can now also introduce new bindings.][crablang#85305]
- [Union field access is permitted in `const fn`.][crablang#85769]

[crablang-2021-edition-guide]: https://doc.crablang.org/nightly/edition-guide/crablang-2021/index.html

Compiler
--------

- [Upgrade to LLVM 13.][crablang#87570]
- [Support memory, address, and thread sanitizers on aarch64-unknown-freebsd.][crablang#88023]
- [Allow specifying a deployment target version for all iOS targets][crablang#87699]
- [Warnings can be forced on with `--force-warn`.][crablang#87472]
  This feature is primarily intended for usage by `cargo fix`, rather than end users.
- [Promote `aarch64-apple-ios-sim` to Tier 2\*.][crablang#87760]
- [Add `powerpc-unknown-freebsd` at Tier 3\*.][crablang#87370]
- [Add `riscv32imc-esp-espidf` at Tier 3\*.][crablang#87666]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
---------

- [Allow writing of incomplete UTF-8 sequences via stdout/stderr on Windows.][crablang#83342]
  The Windows console still requires valid Unicode, but this change allows
  splitting a UTF-8 character across multiple write calls. This allows, for
  instance, programs that just read and write data buffers (e.g. copying a file
  to stdout) without regard for Unicode or character boundaries.
- [Prefer `AtomicU{64,128}` over Mutex for Instant backsliding protection.][crablang#83093]
  For this use case, atomics scale much better under contention.
- [Implement `Extend<(A, B)>` for `(Extend<A>, Extend<B>)`][crablang#85835]
- [impl Default, Copy, Clone for std::io::Sink and std::io::Empty][crablang#86744]
- [`impl From<[(K, V); N]>` for all collections.][crablang#84111]
- [Remove `P: Unpin` bound on impl Future for Pin.][crablang#81363]
- [Treat invalid environment variable names as non-existent.][crablang#86183]
  Previously, the environment functions would panic if given a variable name
  with an internal null character or equal sign (`=`). Now, these functions will
  just treat such names as non-existent variables, since the OS cannot represent
  the existence of a variable with such a name.

Stabilised APIs
---------------

- [`std::os::unix::fs::chroot`]
- [`UnsafeCell::raw_get`]
- [`BufWriter::into_parts`]
- [`core::panic::{UnwindSafe, RefUnwindSafe, AssertUnwindSafe}`]
  These APIs were previously stable in `std`, but are now also available in `core`.
- [`Vec::shrink_to`]
- [`String::shrink_to`]
- [`OsString::shrink_to`]
- [`PathBuf::shrink_to`]
- [`BinaryHeap::shrink_to`]
- [`VecDeque::shrink_to`]
- [`HashMap::shrink_to`]
- [`HashSet::shrink_to`]

These APIs are now usable in const contexts:

- [`std::mem::transmute`]
- [`[T]::first`][`slice::first`]
- [`[T]::split_first`][`slice::split_first`]
- [`[T]::last`][`slice::last`]
- [`[T]::split_last`][`slice::split_last`]

Cargo
-----

- [Cargo supports specifying a minimum supported CrabLang version in Cargo.toml.][`crablang-version`]
  This has no effect at present on dependency version selection.
  We encourage crates to specify their minimum supported CrabLang version, and we encourage CI systems
  that support CrabLang code to include a crate's specified minimum version in the test matrix for that
  crate by default.

Compatibility notes
-------------------

- [Update to new argument parsing rules on Windows.][crablang#87580]
  This adjusts CrabLang's standard library to match the behavior of the standard
  libraries for C/C++. The rules have changed slightly over time, and this PR
  brings us to the latest set of rules (changed in 2008).
- [Disallow the aapcs calling convention on aarch64][crablang#88399]
  This was already not supported by LLVM; this change surfaces this lack of
  support with a better error message.
- [Make `SEMICOLON_IN_EXPRESSIONS_FROM_MACROS` warn by default][crablang#87385]
- [Warn when an escaped newline skips multiple lines.][crablang#87671]
- [Calls to `libc::getpid` / `std::process::id` from `Command::pre_exec`
   may return different values on glibc <= 2.24.][crablang#81825]
   CrabLang now invokes the `clone3` system call directly, when available, to use new functionality
   available via that system call. Older versions of glibc cache the result of `getpid`, and only
   update that cache when calling glibc's clone/fork functions, so a direct system call bypasses
   that cache update. glibc 2.25 and newer no longer cache `getpid` for exactly this reason.

Internal changes
----------------
These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc
and related tools.

- [LLVM is compiled with PGO in published x86_64-unknown-linux-gnu artifacts.][crablang#88069]
  This improves the performance of most CrabLang builds.
- [Unify representation of macros in internal data structures.][crablang#88019]
  This change fixes a host of bugs with the handling of macros by the compiler,
  as well as crablangdoc.

[`std::os::unix::fs::chroot`]: https://doc.crablang.org/stable/std/os/unix/fs/fn.chroot.html
[`UnsafeCell::raw_get`]: https://doc.crablang.org/stable/std/cell/struct.UnsafeCell.html#method.raw_get
[`BufWriter::into_parts`]: https://doc.crablang.org/stable/std/io/struct.BufWriter.html#method.into_parts
[`core::panic::{UnwindSafe, RefUnwindSafe, AssertUnwindSafe}`]: https://github.com/crablang/crablang/pull/84662
[`Vec::shrink_to`]: https://doc.crablang.org/stable/std/vec/struct.Vec.html#method.shrink_to
[`String::shrink_to`]: https://doc.crablang.org/stable/std/string/struct.String.html#method.shrink_to
[`OsString::shrink_to`]: https://doc.crablang.org/stable/std/ffi/struct.OsString.html#method.shrink_to
[`PathBuf::shrink_to`]: https://doc.crablang.org/stable/std/path/struct.PathBuf.html#method.shrink_to
[`BinaryHeap::shrink_to`]: https://doc.crablang.org/stable/std/collections/struct.BinaryHeap.html#method.shrink_to
[`VecDeque::shrink_to`]: https://doc.crablang.org/stable/std/collections/struct.VecDeque.html#method.shrink_to
[`HashMap::shrink_to`]: https://doc.crablang.org/stable/std/collections/hash_map/struct.HashMap.html#method.shrink_to
[`HashSet::shrink_to`]: https://doc.crablang.org/stable/std/collections/hash_set/struct.HashSet.html#method.shrink_to
[`std::mem::transmute`]: https://doc.crablang.org/stable/std/mem/fn.transmute.html
[`slice::first`]: https://doc.crablang.org/stable/std/primitive.slice.html#method.first
[`slice::split_first`]: https://doc.crablang.org/stable/std/primitive.slice.html#method.split_first
[`slice::last`]: https://doc.crablang.org/stable/std/primitive.slice.html#method.last
[`slice::split_last`]: https://doc.crablang.org/stable/std/primitive.slice.html#method.split_last
[`crablang-version`]: https://doc.crablang.org/nightly/cargo/reference/manifest.html#the-crablang-version-field
[crablang#87671]: https://github.com/crablang/crablang/pull/87671
[crablang#86183]: https://github.com/crablang/crablang/pull/86183
[crablang#87385]: https://github.com/crablang/crablang/pull/87385
[crablang#88100]: https://github.com/crablang/crablang/pull/88100
[crablang#85305]: https://github.com/crablang/crablang/pull/85305
[crablang#88069]: https://github.com/crablang/crablang/pull/88069
[crablang#87472]: https://github.com/crablang/crablang/pull/87472
[crablang#87699]: https://github.com/crablang/crablang/pull/87699
[crablang#87570]: https://github.com/crablang/crablang/pull/87570
[crablang#88023]: https://github.com/crablang/crablang/pull/88023
[crablang#87760]: https://github.com/crablang/crablang/pull/87760
[crablang#87370]: https://github.com/crablang/crablang/pull/87370
[crablang#87580]: https://github.com/crablang/crablang/pull/87580
[crablang#83342]: https://github.com/crablang/crablang/pull/83342
[crablang#83093]: https://github.com/crablang/crablang/pull/83093
[crablang#85835]: https://github.com/crablang/crablang/pull/85835
[crablang#86744]: https://github.com/crablang/crablang/pull/86744
[crablang#81363]: https://github.com/crablang/crablang/pull/81363
[crablang#84111]: https://github.com/crablang/crablang/pull/84111
[crablang#85769]: https://github.com/crablang/crablang/pull/85769#issuecomment-854363720
[crablang#88399]: https://github.com/crablang/crablang/pull/88399
[crablang#81825]: https://github.com/crablang/crablang/pull/81825#issuecomment-808406918
[crablang#88019]: https://github.com/crablang/crablang/pull/88019
[crablang#87666]: https://github.com/crablang/crablang/pull/87666

Version 1.55.0 (2021-09-09)
============================

Language
--------
- [You can now write open "from" range patterns (`X..`), which will start at `X` and
  will end at the maximum value of the integer.][83918]
- [You can now explicitly import the prelude of different editions
  through `std::prelude` (e.g. `use std::prelude::crablang_2021::*;`).][86294]

Compiler
--------
- [Added tier 3\* support for `powerpc64le-unknown-freebsd`.][83572]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
   information on CrabLang's tiered platform support.

Libraries
---------

- [Updated std's float parsing to use the Eisel-Lemire algorithm.][86761]
  These improvements should in general provide faster string parsing of floats,
  no longer reject certain valid floating point values, and reduce
  the produced code size for non-stripped artifacts.
- [`string::Drain` now implements `AsRef<str>` and `AsRef<[u8]>`.][86858]

Stabilised APIs
---------------

- [`Bound::cloned`]
- [`Drain::as_str`]
- [`IntoInnerError::into_error`]
- [`IntoInnerError::into_parts`]
- [`MaybeUninit::assume_init_mut`]
- [`MaybeUninit::assume_init_ref`]
- [`MaybeUninit::write`]
- [`array::map`]
- [`ops::ControlFlow`]
- [`x86::_bittest`]
- [`x86::_bittestandcomplement`]
- [`x86::_bittestandreset`]
- [`x86::_bittestandset`]
- [`x86_64::_bittest64`]
- [`x86_64::_bittestandcomplement64`]
- [`x86_64::_bittestandreset64`]
- [`x86_64::_bittestandset64`]

The following previously stable functions are now `const`.

- [`str::from_utf8_unchecked`]


Cargo
-----
- [Cargo will now deduplicate compiler diagnostics to the terminal when invoking
  crablangc in parallel such as when using `cargo test`.][cargo/9675]
- [The package definition in `cargo metadata` now includes the `"default_run"`
  field from the manifest.][cargo/9550]
- [Added `cargo d` as an alias for `cargo doc`.][cargo/9680]
- [Added `{lib}` as formatting option for `cargo tree` to print the `"lib_name"`
  of packages.][cargo/9663]

CrabLangdoc
-------
- [Added "Go to item on exact match" search option.][85876]
- [The "Implementors" section on traits no longer shows redundant
  method definitions.][85970]
- [Trait implementations are toggled open by default.][86260] This should make the
  implementations more searchable by tools like `CTRL+F` in your browser.
- [Intra-doc links should now correctly resolve associated items (e.g. methods)
  through type aliases.][86334]
- [Traits which are marked with `#[doc(hidden)]` will no longer appear in the
  "Trait Implementations" section.][86513]


Compatibility Notes
-------------------
- [std functions that return an `io::Error` will no longer use the
  `ErrorKind::Other` variant.][85746] This is to better reflect that these
  kinds of errors could be categorised [into newer more specific `ErrorKind`
  variants][79965], and that they do not represent a user error.
- [Using environment variable names with `process::Command` on Windows now
  behaves as expected.][85270] Previously using envionment variables with
  `Command` would cause them to be ASCII-uppercased.
- [CrabLangdoc will now warn on using crablangdoc lints that aren't prefixed
  with `crablangdoc::`][86849]
- `CRABLANGFLAGS` is no longer set for build scripts. Build scripts
  should use `CARGO_ENCODED_CRABLANGFLAGS` instead. See the
  [documentation](https://doc.crablang.org/nightly/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts)
  for more details.

[86849]: https://github.com/crablang/crablang/pull/86849
[86513]: https://github.com/crablang/crablang/pull/86513
[86334]: https://github.com/crablang/crablang/pull/86334
[86260]: https://github.com/crablang/crablang/pull/86260
[85970]: https://github.com/crablang/crablang/pull/85970
[85876]: https://github.com/crablang/crablang/pull/85876
[83572]: https://github.com/crablang/crablang/pull/83572
[86294]: https://github.com/crablang/crablang/pull/86294
[86858]: https://github.com/crablang/crablang/pull/86858
[86761]: https://github.com/crablang/crablang/pull/86761
[85746]: https://github.com/crablang/crablang/pull/85746
[85270]: https://github.com/crablang/crablang/pull/85270
[83918]: https://github.com/crablang/crablang/pull/83918
[79965]: https://github.com/crablang/crablang/pull/79965
[cargo/9663]: https://github.com/crablang/cargo/pull/9663
[cargo/9675]: https://github.com/crablang/cargo/pull/9675
[cargo/9550]: https://github.com/crablang/cargo/pull/9550
[cargo/9680]: https://github.com/crablang/cargo/pull/9680
[`array::map`]: https://doc.crablang.org/stable/std/primitive.array.html#method.map
[`Bound::cloned`]: https://doc.crablang.org/stable/std/ops/enum.Bound.html#method.cloned
[`Drain::as_str`]: https://doc.crablang.org/stable/std/string/struct.Drain.html#method.as_str
[`IntoInnerError::into_error`]: https://doc.crablang.org/stable/std/io/struct.IntoInnerError.html#method.into_error
[`IntoInnerError::into_parts`]: https://doc.crablang.org/stable/std/io/struct.IntoInnerError.html#method.into_parts
[`MaybeUninit::assume_init_mut`]: https://doc.crablang.org/stable/std/mem/union.MaybeUninit.html#method.assume_init_mut
[`MaybeUninit::assume_init_ref`]: https://doc.crablang.org/stable/std/mem/union.MaybeUninit.html#method.assume_init_ref
[`MaybeUninit::write`]: https://doc.crablang.org/stable/std/mem/union.MaybeUninit.html#method.write
[`ops::ControlFlow`]: https://doc.crablang.org/stable/std/ops/enum.ControlFlow.html
[`str::from_utf8_unchecked`]: https://doc.crablang.org/stable/std/str/fn.from_utf8_unchecked.html
[`x86::_bittest`]: https://doc.crablang.org/stable/core/arch/x86/fn._bittest.html
[`x86::_bittestandcomplement`]: https://doc.crablang.org/stable/core/arch/x86/fn._bittestandcomplement.html
[`x86::_bittestandreset`]: https://doc.crablang.org/stable/core/arch/x86/fn._bittestandreset.html
[`x86::_bittestandset`]: https://doc.crablang.org/stable/core/arch/x86/fn._bittestandset.html
[`x86_64::_bittest64`]: https://doc.crablang.org/stable/core/arch/x86_64/fn._bittest64.html
[`x86_64::_bittestandcomplement64`]: https://doc.crablang.org/stable/core/arch/x86_64/fn._bittestandcomplement64.html
[`x86_64::_bittestandreset64`]: https://doc.crablang.org/stable/core/arch/x86_64/fn._bittestandreset64.html
[`x86_64::_bittestandset64`]: https://doc.crablang.org/stable/core/arch/x86_64/fn._bittestandset64.html


Version 1.54.0 (2021-07-29)
============================

Language
-----------------------

- [You can now use macros for values in some built-in attributes.][83366]
  This primarily allows you to call macros within the `#[doc]` attribute. For
  example, to include external documentation in your crate, you can now write
  the following:
  ```crablang
  #![doc = include_str!("README.md")]
  ```

- [You can now cast between unsized slice types (and types which contain
  unsized slices) in `const fn`.][85078]
- [You can now use multiple generic lifetimes with `impl Trait` where the
   lifetimes don't explicitly outlive another.][84701] In code this means
   that you can now have `impl Trait<'a, 'b>` where as before you could
   only have `impl Trait<'a, 'b> where 'b: 'a`.

Compiler
-----------------------

- [CrabLangc will now search for custom JSON targets in
  `/lib/crablanglib/<target-triple>/target.json` where `/` is the "sysroot"
  directory.][83800] You can find your sysroot directory by running
  `crablangc --print sysroot`.
- [Added `wasm` as a `target_family` for WebAssembly platforms.][84072]
- [You can now use `#[target_feature]` on safe functions when targeting
  WebAssembly platforms.][84988]
- [Improved debugger output for enums on Windows MSVC platforms.][85292]
- [Added tier 3\* support for `bpfel-unknown-none`
   and `bpfeb-unknown-none`.][79608]
- [`-Zmutable-noalias=yes`][82834] is enabled by default when using LLVM 12 or above.

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
   information on CrabLang's tiered platform support.

Libraries
-----------------------

- [`panic::panic_any` will now `#[track_caller]`.][85745]
- [Added `OutOfMemory` as a variant of `io::ErrorKind`.][84744]
- [ `proc_macro::Literal` now implements `FromStr`.][84717]
- [The implementations of vendor intrinsics in core::arch have been
   significantly refactored.][83278] The main user-visible changes are
   a 50% reduction in the size of libcore.rlib and stricter validation
   of constant operands passed to intrinsics. The latter is technically
   a breaking change, but allows CrabLang to more closely match the C vendor
   intrinsics API.

Stabilized APIs
---------------

- [`BTreeMap::into_keys`]
- [`BTreeMap::into_values`]
- [`HashMap::into_keys`]
- [`HashMap::into_values`]
- [`arch::wasm32`]
- [`VecDeque::binary_search`]
- [`VecDeque::binary_search_by`]
- [`VecDeque::binary_search_by_key`]
- [`VecDeque::partition_point`]

Cargo
-----

- [Added the `--prune <spec>` option to `cargo-tree` to remove a package from
  the dependency graph.][cargo/9520]
- [Added the `--depth` option to `cargo-tree` to print only to a certain depth
  in the tree ][cargo/9499]
- [Added the `no-proc-macro` value to `cargo-tree --edges` to hide procedural
  macro dependencies.][cargo/9488]
- [A new environment variable named `CARGO_TARGET_TMPDIR` is available.][cargo/9375]
  This variable points to a directory that integration tests and benches
  can use as a "scratchpad" for testing filesystem operations.

Compatibility Notes
-------------------
- [Mixing Option and Result via `?` is no longer permitted in closures for inferred types.][86831]
- [Previously unsound code is no longer permitted where different constructors in branches
  could require different lifetimes.][85574]
- As previously mentioned the [`std::arch` intrinsics now uses stricter const checking][83278]
  than before and may reject some previously accepted code.
- [`i128` multiplication on Cortex M0+ platforms currently unconditionally causes overflow
   when compiled with `codegen-units = 1`.][86063]

[85574]: https://github.com/crablang/crablang/issues/85574
[86831]: https://github.com/crablang/crablang/issues/86831
[86063]: https://github.com/crablang/crablang/issues/86063
[79608]: https://github.com/crablang/crablang/pull/79608
[84988]: https://github.com/crablang/crablang/pull/84988
[84701]: https://github.com/crablang/crablang/pull/84701
[84072]: https://github.com/crablang/crablang/pull/84072
[85745]: https://github.com/crablang/crablang/pull/85745
[84744]: https://github.com/crablang/crablang/pull/84744
[85078]: https://github.com/crablang/crablang/pull/85078
[84717]: https://github.com/crablang/crablang/pull/84717
[83800]: https://github.com/crablang/crablang/pull/83800
[83366]: https://github.com/crablang/crablang/pull/83366
[83278]: https://github.com/crablang/crablang/pull/83278
[85292]: https://github.com/crablang/crablang/pull/85292
[82834]: https://github.com/crablang/crablang/pull/82834
[cargo/9520]: https://github.com/crablang/cargo/pull/9520
[cargo/9499]: https://github.com/crablang/cargo/pull/9499
[cargo/9488]: https://github.com/crablang/cargo/pull/9488
[cargo/9375]: https://github.com/crablang/cargo/pull/9375
[`BTreeMap::into_keys`]: https://doc.crablang.org/std/collections/struct.BTreeMap.html#method.into_keys
[`BTreeMap::into_values`]: https://doc.crablang.org/std/collections/struct.BTreeMap.html#method.into_values
[`HashMap::into_keys`]: https://doc.crablang.org/std/collections/struct.HashMap.html#method.into_keys
[`HashMap::into_values`]: https://doc.crablang.org/std/collections/struct.HashMap.html#method.into_values
[`arch::wasm32`]: https://doc.crablang.org/core/arch/wasm32/index.html
[`VecDeque::binary_search`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.binary_search
[`VecDeque::binary_search_by`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.binary_search_by

[`VecDeque::binary_search_by_key`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.binary_search_by_key

[`VecDeque::partition_point`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.partition_point

Version 1.53.0 (2021-06-17)
============================

Language
-----------------------
- [You can now use unicode for identifiers.][83799] This allows multilingual
  identifiers but still doesn't allow glyphs that are not considered characters
  such as `◆` or `🦀`. More specifically you can now use any identifier that
  matches the UAX #31 "Unicode Identifier and Pattern Syntax" standard. This
  is the same standard as languages like Python, however CrabLang uses NFC
  normalization which may be different from other languages.
- [You can now specify "or patterns" inside pattern matches.][79278]
  Previously you could only use `|` (OR) on complete patterns. E.g.
  ```crablang
  let x = Some(2u8);
  // Before
  matches!(x, Some(1) | Some(2));
  // Now
  matches!(x, Some(1 | 2));
  ```
- [Added the `:pat_param` `macro_rules!` matcher.][83386] This matcher
  has the same semantics as the `:pat` matcher. This is to allow `:pat`
  to change semantics to being a pattern fragment in a future edition.

Compiler
-----------------------
- [Updated the minimum external LLVM version to LLVM 10.][83387]
- [Added Tier 3\* support for the `wasm64-unknown-unknown` target.][80525]
- [Improved debuginfo for closures and async functions on Windows MSVC.][83941]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
-----------------------
- [Abort messages will now forward to `android_set_abort_message` on
  Android platforms when available.][81469]
- [`slice::IterMut<'_, T>` now implements `AsRef<[T]>`][82771]
- [Arrays of any length now implement `IntoIterator`.][84147]
  Currently calling `.into_iter()` as a method on an array will
  return `impl Iterator<Item=&T>`, but this may change in a
  future edition to change `Item` to `T`. Calling `IntoIterator::into_iter`
  directly on arrays will provide `impl Iterator<Item=T>` as expected.
- [`leading_zeros`, and `trailing_zeros` are now available on all
  `NonZero` integer types.][84082]
- [`{f32, f64}::from_str` now parse and print special values
  (`NaN`, `-0`) according to IEEE 754.][78618]
- [You can now index into slices using `(Bound<usize>, Bound<usize>)`.][77704]
- [Add the `BITS` associated constant to all numeric types.][82565]

Stabilised APIs
---------------
- [`AtomicBool::fetch_update`]
- [`AtomicPtr::fetch_update`]
- [`BTreeMap::retain`]
- [`BTreeSet::retain`]
- [`BufReader::seek_relative`]
- [`DebugStruct::non_exhaustive`]
- [`Duration::MAX`]
- [`Duration::ZERO`]
- [`Duration::is_zero`]
- [`Duration::saturating_add`]
- [`Duration::saturating_mul`]
- [`Duration::saturating_sub`]
- [`ErrorKind::Unsupported`]
- [`Option::insert`]
- [`Ordering::is_eq`]
- [`Ordering::is_ge`]
- [`Ordering::is_gt`]
- [`Ordering::is_le`]
- [`Ordering::is_lt`]
- [`Ordering::is_ne`]
- [`OsStr::is_ascii`]
- [`OsStr::make_ascii_lowercase`]
- [`OsStr::make_ascii_uppercase`]
- [`OsStr::to_ascii_lowercase`]
- [`OsStr::to_ascii_uppercase`]
- [`Peekable::peek_mut`]
- [`Rc::decrement_strong_count`]
- [`Rc::increment_strong_count`]
- [`Vec::extend_from_within`]
- [`array::from_mut`]
- [`array::from_ref`]
- [`cmp::max_by_key`]
- [`cmp::max_by`]
- [`cmp::min_by_key`]
- [`cmp::min_by`]
- [`f32::is_subnormal`]
- [`f64::is_subnormal`]

Cargo
-----------------------
- [Cargo now supports git repositories where the default `HEAD` branch is not
  "master".][cargo/9392] This also includes a switch to the version 3 `Cargo.lock` format
  which can handle default branches correctly.
- [macOS targets now default to `unpacked` split-debuginfo.][cargo/9298]
- [The `authors` field is no longer included in `Cargo.toml` for new
  projects.][cargo/9282]

CrabLangdoc
-----------------------
- [Added the `crablangdoc::bare_urls` lint that warns when you have URLs
  without hyperlinks.][81764]

Compatibility Notes
-------------------
- [Implement token-based handling of attributes during expansion][82608]
- [`Ipv4::from_str` will now reject octal format IP addresses in addition
  to rejecting hexadecimal IP addresses.][83652] The octal format can lead
  to confusion and potential security vulnerabilities and [is no
  longer recommended][ietf6943].
- [The added `BITS` constant may conflict with external definitions.][85667]
  In particular, this was known to be a problem in the `lexical-core` crate,
  but they have published fixes for semantic versions 0.4 through 0.7. To
  update this dependency alone, use `cargo update -p lexical-core`.
- Incremental compilation remains off by default, unless one uses the `CRABLANGC_FORCE_INCREMENTAL=1` environment variable added in 1.52.1.

Internal Only
-------------
These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc and
related tools.

- [Rework the `std::sys::windows::alloc` implementation.][83065]
- [crablangdoc: Don't enter an infer_ctxt in get_blanket_impls for impls that aren't blanket impls.][82864]
- [crablangdoc: Only look at blanket impls in `get_blanket_impls`][83681]
- [Rework crablangdoc const type][82873]

[85667]: https://github.com/crablang/crablang/pull/85667
[83386]: https://github.com/crablang/crablang/pull/83386
[82771]: https://github.com/crablang/crablang/pull/82771
[84147]: https://github.com/crablang/crablang/pull/84147
[84082]: https://github.com/crablang/crablang/pull/84082
[83799]: https://github.com/crablang/crablang/pull/83799
[83681]: https://github.com/crablang/crablang/pull/83681
[83652]: https://github.com/crablang/crablang/pull/83652
[83387]: https://github.com/crablang/crablang/pull/83387
[82873]: https://github.com/crablang/crablang/pull/82873
[82864]: https://github.com/crablang/crablang/pull/82864
[82608]: https://github.com/crablang/crablang/pull/82608
[82565]: https://github.com/crablang/crablang/pull/82565
[80525]: https://github.com/crablang/crablang/pull/80525
[79278]: https://github.com/crablang/crablang/pull/79278
[78618]: https://github.com/crablang/crablang/pull/78618
[77704]: https://github.com/crablang/crablang/pull/77704
[83941]: https://github.com/crablang/crablang/pull/83941
[83065]: https://github.com/crablang/crablang/pull/83065
[81764]: https://github.com/crablang/crablang/pull/81764
[81469]: https://github.com/crablang/crablang/pull/81469
[cargo/9298]: https://github.com/crablang/cargo/pull/9298
[cargo/9282]: https://github.com/crablang/cargo/pull/9282
[cargo/9392]: https://github.com/crablang/cargo/pull/9392
[`AtomicBool::fetch_update`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicBool.html#method.fetch_update
[`AtomicPtr::fetch_update`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicPtr.html#method.fetch_update
[`BTreeMap::retain`]: https://doc.crablang.org/std/collections/struct.BTreeMap.html#method.retain
[`BTreeSet::retain`]: https://doc.crablang.org/std/collections/struct.BTreeSet.html#method.retain
[`BufReader::seek_relative`]: https://doc.crablang.org/std/io/struct.BufReader.html#method.seek_relative
[`DebugStruct::non_exhaustive`]: https://doc.crablang.org/std/fmt/struct.DebugStruct.html#method.finish_non_exhaustive
[`Duration::MAX`]: https://doc.crablang.org/std/time/struct.Duration.html#associatedconstant.MAX
[`Duration::ZERO`]: https://doc.crablang.org/std/time/struct.Duration.html#associatedconstant.ZERO
[`Duration::is_zero`]: https://doc.crablang.org/std/time/struct.Duration.html#method.is_zero
[`Duration::saturating_add`]: https://doc.crablang.org/std/time/struct.Duration.html#method.saturating_add
[`Duration::saturating_mul`]: https://doc.crablang.org/std/time/struct.Duration.html#method.saturating_mul
[`Duration::saturating_sub`]: https://doc.crablang.org/std/time/struct.Duration.html#method.saturating_sub
[`ErrorKind::Unsupported`]: https://doc.crablang.org/std/io/enum.ErrorKind.html#variant.Unsupported
[`Option::insert`]: https://doc.crablang.org/std/option/enum.Option.html#method.insert
[`Ordering::is_eq`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.is_eq
[`Ordering::is_ge`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.is_ge
[`Ordering::is_gt`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.is_gt
[`Ordering::is_le`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.is_le
[`Ordering::is_lt`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.is_lt
[`Ordering::is_ne`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.is_ne
[`OsStr::is_ascii`]: https://doc.crablang.org/std/ffi/struct.OsStr.html#method.is_ascii
[`OsStr::make_ascii_lowercase`]: https://doc.crablang.org/std/ffi/struct.OsStr.html#method.make_ascii_lowercase
[`OsStr::make_ascii_uppercase`]: https://doc.crablang.org/std/ffi/struct.OsStr.html#method.make_ascii_uppercase
[`OsStr::to_ascii_lowercase`]: https://doc.crablang.org/std/ffi/struct.OsStr.html#method.to_ascii_lowercase
[`OsStr::to_ascii_uppercase`]: https://doc.crablang.org/std/ffi/struct.OsStr.html#method.to_ascii_uppercase
[`Peekable::peek_mut`]: https://doc.crablang.org/std/iter/struct.Peekable.html#method.peek_mut
[`Rc::decrement_strong_count`]: https://doc.crablang.org/std/rc/struct.Rc.html#method.increment_strong_count
[`Rc::increment_strong_count`]: https://doc.crablang.org/std/rc/struct.Rc.html#method.increment_strong_count
[`Vec::extend_from_within`]: https://doc.crablang.org/beta/std/vec/struct.Vec.html#method.extend_from_within
[`array::from_mut`]: https://doc.crablang.org/beta/std/array/fn.from_mut.html
[`array::from_ref`]: https://doc.crablang.org/beta/std/array/fn.from_ref.html
[`cmp::max_by_key`]: https://doc.crablang.org/beta/std/cmp/fn.max_by_key.html
[`cmp::max_by`]: https://doc.crablang.org/beta/std/cmp/fn.max_by.html
[`cmp::min_by_key`]: https://doc.crablang.org/beta/std/cmp/fn.min_by_key.html
[`cmp::min_by`]: https://doc.crablang.org/beta/std/cmp/fn.min_by.html
[`f32::is_subnormal`]: https://doc.crablang.org/std/primitive.f64.html#method.is_subnormal
[`f64::is_subnormal`]: https://doc.crablang.org/std/primitive.f64.html#method.is_subnormal
[ietf6943]: https://datatracker.ietf.org/doc/html/rfc6943#section-3.1.1


Version 1.52.1 (2021-05-10)
============================

This release disables incremental compilation, unless the user has explicitly
opted in via the newly added CRABLANGC_FORCE_INCREMENTAL=1 environment variable.

This is due to the widespread, and frequently occurring, breakage encountered by
CrabLang users due to newly enabled incremental verification in 1.52.0. Notably,
CrabLang users **should** upgrade to 1.52.0 or 1.52.1: the bugs that are detected by
newly added incremental verification are still present in past stable versions,
and are not yet fixed on any channel. These bugs can lead to miscompilation of
CrabLang binaries.

These problems only affect incremental builds, so release builds with Cargo
should not be affected unless the user has explicitly opted into incremental.
Debug and check builds are affected.

See [84970] for more details.

[84970]: https://github.com/crablang/crablang/issues/84970

Version 1.52.0 (2021-05-06)
============================

Language
--------
- [Added the `unsafe_op_in_unsafe_fn` lint, which checks whether the unsafe code
  in an `unsafe fn` is wrapped in a `unsafe` block.][79208] This lint
  is allowed by default, and may become a warning or hard error in a
  future edition.
- [You can now cast mutable references to arrays to a pointer of the same type as
  the element.][81479]

Compiler
--------
- [Upgraded the default LLVM to LLVM 12.][81451]

Added tier 3\* support for the following targets.

- [`s390x-unknown-linux-musl`][82166]
- [`riscv32gc-unknown-linux-musl` & `riscv64gc-unknown-linux-musl`][82202]
- [`powerpc-unknown-openbsd`][82733]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
---------
- [`OsString` now implements `Extend` and `FromIterator`.][82121]
- [`cmp::Reverse` now has `#[repr(transparent)]` representation.][81879]
- [`Arc<impl Error>` now implements `error::Error`.][80553]
- [All integer division and remainder operations are now `const`.][80962]

Stabilised APIs
-------------
- [`Arguments::as_str`]
- [`char::MAX`]
- [`char::REPLACEMENT_CHARACTER`]
- [`char::UNICODE_VERSION`]
- [`char::decode_utf16`]
- [`char::from_digit`]
- [`char::from_u32_unchecked`]
- [`char::from_u32`]
- [`slice::partition_point`]
- [`str::rsplit_once`]
- [`str::split_once`]

The following previously stable APIs are now `const`.

- [`char::len_utf8`]
- [`char::len_utf16`]
- [`char::to_ascii_uppercase`]
- [`char::to_ascii_lowercase`]
- [`char::eq_ignore_ascii_case`]
- [`u8::to_ascii_uppercase`]
- [`u8::to_ascii_lowercase`]
- [`u8::eq_ignore_ascii_case`]

CrabLangdoc
-------
- [CrabLangdoc lints are now treated as a tool lint, meaning that
  lints are now prefixed with `crablangdoc::` (e.g. `#[warn(crablangdoc::broken_intra_doc_links)]`).][80527]
  Using the old style is still allowed, and will become a warning in
  a future release.
- [CrabLangdoc now supports argument files.][82261]
- [CrabLangdoc now generates smart punctuation for documentation.][79423]
- [You can now use "task lists" in CrabLangdoc Markdown.][81766] E.g.
  ```markdown
  - [x] Complete
  - [ ] Todo
  ```

Misc
----
- [You can now pass multiple filters to tests.][81356] E.g.
  `cargo test -- foo bar` will run all tests that match `foo` and `bar`.
- [CrabLangup now distributes PDB symbols for the `std` library on Windows,
  allowing you to see `std` symbols when debugging.][82218]

Internal Only
-------------
These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc and
related tools.

- [Check the result cache before the DepGraph when ensuring queries][81855]
- [Try fast_reject::simplify_type in coherence before doing full check][81744]
- [Only store a LocalDefId in some HIR nodes][81611]
- [Store HIR attributes in a side table][79519]

Compatibility Notes
-------------------
- [Cargo build scripts are now forbidden from setting `CRABLANGC_BOOTSTRAP`.][cargo/9181]
- [Removed support for the `x86_64-rumprun-netbsd` target.][82594]
- [Deprecated the `x86_64-sun-solaris` target in favor of `x86_64-pc-solaris`.][82216]
- [CrabLangdoc now only accepts `,`, ` `, and `\t` as delimiters for specifying
  languages in code blocks.][78429]
- [CrabLangc now catches more cases of `pub_use_of_private_extern_crate`][80763]
- [Changes in how proc macros handle whitespace may lead to panics when used
  with older `proc-macro-hack` versions. A `cargo update` should be sufficient to fix this in all cases.][84136]
- [Turn `#[derive]` into a regular macro attribute][79078]

[84136]: https://github.com/crablang/crablang/issues/84136
[80763]: https://github.com/crablang/crablang/pull/80763
[82166]: https://github.com/crablang/crablang/pull/82166
[82121]: https://github.com/crablang/crablang/pull/82121
[81879]: https://github.com/crablang/crablang/pull/81879
[82261]: https://github.com/crablang/crablang/pull/82261
[82218]: https://github.com/crablang/crablang/pull/82218
[82216]: https://github.com/crablang/crablang/pull/82216
[82202]: https://github.com/crablang/crablang/pull/82202
[81855]: https://github.com/crablang/crablang/pull/81855
[81766]: https://github.com/crablang/crablang/pull/81766
[81744]: https://github.com/crablang/crablang/pull/81744
[81611]: https://github.com/crablang/crablang/pull/81611
[81479]: https://github.com/crablang/crablang/pull/81479
[81451]: https://github.com/crablang/crablang/pull/81451
[81356]: https://github.com/crablang/crablang/pull/81356
[80962]: https://github.com/crablang/crablang/pull/80962
[80553]: https://github.com/crablang/crablang/pull/80553
[80527]: https://github.com/crablang/crablang/pull/80527
[79519]: https://github.com/crablang/crablang/pull/79519
[79423]: https://github.com/crablang/crablang/pull/79423
[79208]: https://github.com/crablang/crablang/pull/79208
[78429]: https://github.com/crablang/crablang/pull/78429
[82733]: https://github.com/crablang/crablang/pull/82733
[82594]: https://github.com/crablang/crablang/pull/82594
[79078]: https://github.com/crablang/crablang/pull/79078
[cargo/9181]: https://github.com/crablang/cargo/pull/9181
[`char::MAX`]: https://doc.crablang.org/std/primitive.char.html#associatedconstant.MAX
[`char::REPLACEMENT_CHARACTER`]: https://doc.crablang.org/std/primitive.char.html#associatedconstant.REPLACEMENT_CHARACTER
[`char::UNICODE_VERSION`]: https://doc.crablang.org/std/primitive.char.html#associatedconstant.UNICODE_VERSION
[`char::decode_utf16`]: https://doc.crablang.org/std/primitive.char.html#method.decode_utf16
[`char::from_u32`]: https://doc.crablang.org/std/primitive.char.html#method.from_u32
[`char::from_u32_unchecked`]: https://doc.crablang.org/std/primitive.char.html#method.from_u32_unchecked
[`char::from_digit`]: https://doc.crablang.org/std/primitive.char.html#method.from_digit
[`Peekable::next_if`]: https://doc.crablang.org/stable/std/iter/struct.Peekable.html#method.next_if
[`Peekable::next_if_eq`]: https://doc.crablang.org/stable/std/iter/struct.Peekable.html#method.next_if_eq
[`Arguments::as_str`]: https://doc.crablang.org/stable/std/fmt/struct.Arguments.html#method.as_str
[`str::split_once`]: https://doc.crablang.org/stable/std/primitive.str.html#method.split_once
[`str::rsplit_once`]: https://doc.crablang.org/stable/std/primitive.str.html#method.rsplit_once
[`slice::partition_point`]: https://doc.crablang.org/stable/std/primitive.slice.html#method.partition_point
[`char::len_utf8`]: https://doc.crablang.org/stable/std/primitive.char.html#method.len_utf8
[`char::len_utf16`]: https://doc.crablang.org/stable/std/primitive.char.html#method.len_utf16
[`char::to_ascii_uppercase`]: https://doc.crablang.org/stable/std/primitive.char.html#method.to_ascii_uppercase
[`char::to_ascii_lowercase`]: https://doc.crablang.org/stable/std/primitive.char.html#method.to_ascii_lowercase
[`char::eq_ignore_ascii_case`]: https://doc.crablang.org/stable/std/primitive.char.html#method.eq_ignore_ascii_case
[`u8::to_ascii_uppercase`]: https://doc.crablang.org/stable/std/primitive.u8.html#method.to_ascii_uppercase
[`u8::to_ascii_lowercase`]: https://doc.crablang.org/stable/std/primitive.u8.html#method.to_ascii_lowercase
[`u8::eq_ignore_ascii_case`]: https://doc.crablang.org/stable/std/primitive.u8.html#method.eq_ignore_ascii_case

Version 1.51.0 (2021-03-25)
============================

Language
--------
- [You can now parameterize items such as functions, traits, and `struct`s by constant
  values in addition to by types and lifetimes.][79135] Also known as "const generics"
  E.g. you can now write the following. Note: Only values of primitive integers,
  `bool`, or `char` types are currently permitted.
  ```crablang
  struct GenericArray<T, const LENGTH: usize> {
      inner: [T; LENGTH]
  }

  impl<T, const LENGTH: usize> GenericArray<T, LENGTH> {
      const fn last(&self) -> Option<&T> {
          if LENGTH == 0 {
              None
          } else {
              Some(&self.inner[LENGTH - 1])
          }
      }
  }
  ```


Compiler
--------

- [Added the `-Csplit-debuginfo` codegen option for macOS platforms.][79570]
  This option controls whether debug information is split across multiple files
  or packed into a single file. **Note** This option is unstable on other platforms.
- [Added tier 3\* support for `aarch64_be-unknown-linux-gnu`,
  `aarch64-unknown-linux-gnu_ilp32`, and `aarch64_be-unknown-linux-gnu_ilp32` targets.][81455]
- [Added tier 3 support for `i386-unknown-linux-gnu` and `i486-unknown-linux-gnu` targets.][80662]
- [The `target-cpu=native` option will now detect individual features of CPUs.][80749]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
---------

- [`Box::downcast` is now also implemented for any `dyn Any + Send + Sync` object.][80945]
- [`str` now implements `AsMut<str>`.][80279]
- [`u64` and `u128` now implement `From<char>`.][79502]
- [`Error` is now implemented for `&T` where `T` implements `Error`.][75180]
- [`Poll::{map_ok, map_err}` are now implemented for `Poll<Option<Result<T, E>>>`.][80968]
- [`unsigned_abs` is now implemented for all signed integer types.][80959]
- [`io::Empty` now implements `io::Seek`.][78044]
- [`rc::Weak<T>` and `sync::Weak<T>`'s methods such as `as_ptr` are now implemented for
  `T: ?Sized` types.][80764]
- [`Div` and `Rem` by their `NonZero` variant is now implemented for all unsigned integers.][79134]


Stabilized APIs
---------------

- [`Arc::decrement_strong_count`]
- [`Arc::increment_strong_count`]
- [`Once::call_once_force`]
- [`Peekable::next_if_eq`]
- [`Peekable::next_if`]
- [`Seek::stream_position`]
- [`array::IntoIter`]
- [`panic::panic_any`]
- [`ptr::addr_of!`]
- [`ptr::addr_of_mut!`]
- [`slice::fill_with`]
- [`slice::split_inclusive_mut`]
- [`slice::split_inclusive`]
- [`slice::strip_prefix`]
- [`slice::strip_suffix`]
- [`str::split_inclusive`]
- [`sync::OnceState`]
- [`task::Wake`]
- [`VecDeque::range`]
- [`VecDeque::range_mut`]

Cargo
-----
- [Added the `split-debuginfo` profile option to control the -Csplit-debuginfo
  codegen option.][cargo/9112]
- [Added the `resolver` field to `Cargo.toml` to enable the new feature resolver
  and CLI option behavior.][cargo/8997] Version 2 of the feature resolver will try
  to avoid unifying features of dependencies where that unification could be unwanted.
  Such as using the same dependency with a `std` feature in a build scripts and
  proc-macros, while using the `no-std` feature in the final binary. See the
  [Cargo book documentation][feature-resolver@2.0] for more information on the feature.

CrabLangdoc
-------

- [CrabLangdoc will now include documentation for methods available from _nested_ `Deref` traits.][80653]
- [You can now provide a `--default-theme` flag which sets the default theme to use for
  documentation.][79642]

Various improvements to intra-doc links:

- [You can link to non-path primitives such as `slice`.][80181]
- [You can link to associated items.][74489]
- [You can now include generic parameters when linking to items, like `Vec<T>`.][76934]

Misc
----
- [You can now pass `--include-ignored` to tests (e.g. with
  `cargo test -- --include-ignored`) to include testing tests marked `#[ignore]`.][80053]

Compatibility Notes
-------------------

- [WASI platforms no longer use the `wasm-bindgen` ABI, and instead use the wasm32 ABI.][79998]
- [`crablangc` no longer promotes division, modulo and indexing operations to `const` that
  could fail.][80579]
- [The minimum version of glibc for the following platforms has been bumped to version 2.31
  for the distributed artifacts.][81521]
    - `armv5te-unknown-linux-gnueabi`
    - `sparc64-unknown-linux-gnu`
    - `thumbv7neon-unknown-linux-gnueabihf`
    - `armv7-unknown-linux-gnueabi`
    - `x86_64-unknown-linux-gnux32`
- [`atomic::spin_loop_hint` has been deprecated.][80966] It's recommended to use `hint::spin_loop` instead.

Internal Only
-------------

- [Consistently avoid constructing optimized MIR when not doing codegen][80718]

[79135]: https://github.com/crablang/crablang/pull/79135
[74489]: https://github.com/crablang/crablang/pull/74489
[76934]: https://github.com/crablang/crablang/pull/76934
[79570]: https://github.com/crablang/crablang/pull/79570
[80181]: https://github.com/crablang/crablang/pull/80181
[79642]: https://github.com/crablang/crablang/pull/79642
[80945]: https://github.com/crablang/crablang/pull/80945
[80279]: https://github.com/crablang/crablang/pull/80279
[80053]: https://github.com/crablang/crablang/pull/80053
[79502]: https://github.com/crablang/crablang/pull/79502
[75180]: https://github.com/crablang/crablang/pull/75180
[81521]: https://github.com/crablang/crablang/pull/81521
[80968]: https://github.com/crablang/crablang/pull/80968
[80959]: https://github.com/crablang/crablang/pull/80959
[80718]: https://github.com/crablang/crablang/pull/80718
[80653]: https://github.com/crablang/crablang/pull/80653
[80579]: https://github.com/crablang/crablang/pull/80579
[79998]: https://github.com/crablang/crablang/pull/79998
[78044]: https://github.com/crablang/crablang/pull/78044
[81455]: https://github.com/crablang/crablang/pull/81455
[80764]: https://github.com/crablang/crablang/pull/80764
[80749]: https://github.com/crablang/crablang/pull/80749
[80662]: https://github.com/crablang/crablang/pull/80662
[79134]: https://github.com/crablang/crablang/pull/79134
[80966]: https://github.com/crablang/crablang/pull/80966
[cargo/8997]: https://github.com/crablang/cargo/pull/8997
[cargo/9112]: https://github.com/crablang/cargo/pull/9112
[feature-resolver@2.0]: https://doc.crablang.org/nightly/cargo/reference/features.html#feature-resolver-version-2
[`Once::call_once_force`]: https://doc.crablang.org/stable/std/sync/struct.Once.html#method.call_once_force
[`sync::OnceState`]: https://doc.crablang.org/stable/std/sync/struct.OnceState.html
[`panic::panic_any`]: https://doc.crablang.org/stable/std/panic/fn.panic_any.html
[`slice::strip_prefix`]: https://doc.crablang.org/stable/std/primitive.slice.html#method.strip_prefix
[`slice::strip_suffix`]: https://doc.crablang.org/stable/std/primitive.slice.html#method.strip_prefix
[`Arc::increment_strong_count`]: https://doc.crablang.org/nightly/std/sync/struct.Arc.html#method.increment_strong_count
[`Arc::decrement_strong_count`]: https://doc.crablang.org/nightly/std/sync/struct.Arc.html#method.decrement_strong_count
[`slice::fill_with`]: https://doc.crablang.org/nightly/std/primitive.slice.html#method.fill_with
[`ptr::addr_of!`]: https://doc.crablang.org/nightly/std/ptr/macro.addr_of.html
[`ptr::addr_of_mut!`]: https://doc.crablang.org/nightly/std/ptr/macro.addr_of_mut.html
[`array::IntoIter`]: https://doc.crablang.org/nightly/std/array/struct.IntoIter.html
[`slice::split_inclusive`]: https://doc.crablang.org/nightly/std/primitive.slice.html#method.split_inclusive
[`slice::split_inclusive_mut`]: https://doc.crablang.org/nightly/std/primitive.slice.html#method.split_inclusive_mut
[`str::split_inclusive`]: https://doc.crablang.org/nightly/std/primitive.str.html#method.split_inclusive
[`task::Wake`]: https://doc.crablang.org/nightly/std/task/trait.Wake.html
[`Seek::stream_position`]: https://doc.crablang.org/nightly/std/io/trait.Seek.html#method.stream_position
[`Peekable::next_if`]: https://doc.crablang.org/nightly/std/iter/struct.Peekable.html#method.next_if
[`Peekable::next_if_eq`]: https://doc.crablang.org/nightly/std/iter/struct.Peekable.html#method.next_if_eq
[`VecDeque::range`]: https://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.range
[`VecDeque::range_mut`]: https://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.range_mut

Version 1.50.0 (2021-02-11)
============================

Language
-----------------------
- [You can now use `const` values for `x` in `[x; N]` array expressions.][79270]
  This has been technically possible since 1.38.0, as it was unintentionally stabilized.
- [Assignments to `ManuallyDrop<T>` union fields are now considered safe.][78068]

Compiler
-----------------------
- [Added tier 3\* support for the `armv5te-unknown-linux-uclibceabi` target.][78142]
- [Added tier 3 support for the `aarch64-apple-ios-macabi` target.][77484]
- [The `x86_64-unknown-freebsd` is now built with the full toolset.][79484]
- [Dropped support for all cloudabi targets.][78439]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
-----------------------

- [`proc_macro::Punct` now implements `PartialEq<char>`.][78636]
- [`ops::{Index, IndexMut}` are now implemented for fixed sized arrays of any length.][74989]
- [On Unix platforms, the `std::fs::File` type now has a "niche" of `-1`.][74699]
  This value cannot be a valid file descriptor, and now means `Option<File>` takes
  up the same amount of space as `File`.

Stabilized APIs
---------------

- [`bool::then`]
- [`btree_map::Entry::or_insert_with_key`]
- [`f32::clamp`]
- [`f64::clamp`]
- [`hash_map::Entry::or_insert_with_key`]
- [`Ord::clamp`]
- [`RefCell::take`]
- [`slice::fill`]
- [`UnsafeCell::get_mut`]

The following previously stable methods are now `const`.

- [`IpAddr::is_ipv4`]
- [`IpAddr::is_ipv6`]
- [`IpAddr::is_unspecified`]
- [`IpAddr::is_loopback`]
- [`IpAddr::is_multicast`]
- [`Ipv4Addr::octets`]
- [`Ipv4Addr::is_loopback`]
- [`Ipv4Addr::is_private`]
- [`Ipv4Addr::is_link_local`]
- [`Ipv4Addr::is_multicast`]
- [`Ipv4Addr::is_broadcast`]
- [`Ipv4Addr::is_documentation`]
- [`Ipv4Addr::to_ipv6_compatible`]
- [`Ipv4Addr::to_ipv6_mapped`]
- [`Ipv6Addr::segments`]
- [`Ipv6Addr::is_unspecified`]
- [`Ipv6Addr::is_loopback`]
- [`Ipv6Addr::is_multicast`]
- [`Ipv6Addr::to_ipv4`]
- [`Layout::size`]
- [`Layout::align`]
- [`Layout::from_size_align`]
- `pow` for all integer types.
- `checked_pow` for all integer types.
- `saturating_pow` for all integer types.
- `wrapping_pow` for all integer types.
- `next_power_of_two` for all unsigned integer types.
- `checked_next_power_of_two` for all unsigned integer types.

Cargo
-----------------------

- [Added the `[build.crablangc-workspace-wrapper]` option.][cargo/8976]
  This option sets a wrapper to execute instead of `crablangc`, for workspace members only.
- [`cargo:rerun-if-changed` will now, if provided a directory, scan the entire
  contents of that directory for changes.][cargo/8973]
- [Added the `--workspace` flag to the `cargo update` command.][cargo/8725]

Misc
----

- [The search results tab and the help button are focusable with keyboard in crablangdoc.][79896]
- [Running tests will now print the total time taken to execute.][75752]

Compatibility Notes
-------------------

- [The `compare_and_swap` method on atomics has been deprecated.][79261] It's
  recommended to use the `compare_exchange` and `compare_exchange_weak` methods instead.
- [Changes in how `TokenStream`s are checked have fixed some cases where you could write
  unhygenic `macro_rules!` macros.][79472]
- [`#![test]` as an inner attribute is now considered unstable like other inner macro
  attributes, and reports an error by default through the `soft_unstable` lint.][79003]
- [Overriding a `forbid` lint at the same level that it was set is now a hard error.][78864]
- [You can no longer intercept `panic!` calls by supplying your own macro.][78343] It's
  recommended to use the `#[panic_handler]` attribute to provide your own implementation.
- [Semi-colons after item statements (e.g. `struct Foo {};`) now produce a warning.][78296]

[74989]: https://github.com/crablang/crablang/pull/74989
[79261]: https://github.com/crablang/crablang/pull/79261
[79896]: https://github.com/crablang/crablang/pull/79896
[79484]: https://github.com/crablang/crablang/pull/79484
[79472]: https://github.com/crablang/crablang/pull/79472
[79270]: https://github.com/crablang/crablang/pull/79270
[79003]: https://github.com/crablang/crablang/pull/79003
[78864]: https://github.com/crablang/crablang/pull/78864
[78636]: https://github.com/crablang/crablang/pull/78636
[78439]: https://github.com/crablang/crablang/pull/78439
[78343]: https://github.com/crablang/crablang/pull/78343
[78296]: https://github.com/crablang/crablang/pull/78296
[78068]: https://github.com/crablang/crablang/pull/78068
[75752]: https://github.com/crablang/crablang/pull/75752
[74699]: https://github.com/crablang/crablang/pull/74699
[78142]: https://github.com/crablang/crablang/pull/78142
[77484]: https://github.com/crablang/crablang/pull/77484
[cargo/8976]: https://github.com/crablang/cargo/pull/8976
[cargo/8973]: https://github.com/crablang/cargo/pull/8973
[cargo/8725]: https://github.com/crablang/cargo/pull/8725
[`IpAddr::is_ipv4`]: https://doc.crablang.org/stable/std/net/enum.IpAddr.html#method.is_ipv4
[`IpAddr::is_ipv6`]: https://doc.crablang.org/stable/std/net/enum.IpAddr.html#method.is_ipv6
[`IpAddr::is_unspecified`]: https://doc.crablang.org/stable/std/net/enum.IpAddr.html#method.is_unspecified
[`IpAddr::is_loopback`]: https://doc.crablang.org/stable/std/net/enum.IpAddr.html#method.is_loopback
[`IpAddr::is_multicast`]: https://doc.crablang.org/stable/std/net/enum.IpAddr.html#method.is_multicast
[`Ipv4Addr::octets`]: https://doc.crablang.org/stable/std/net/struct.Ipv4Addr.html#method.octets
[`Ipv4Addr::is_loopback`]: https://doc.crablang.org/stable/std/net/struct.Ipv4Addr.html#method.is_loopback
[`Ipv4Addr::is_private`]: https://doc.crablang.org/stable/std/net/struct.Ipv4Addr.html#method.is_private
[`Ipv4Addr::is_link_local`]: https://doc.crablang.org/stable/std/net/struct.Ipv4Addr.html#method.is_link_local
[`Ipv4Addr::is_multicast`]: https://doc.crablang.org/stable/std/net/struct.Ipv4Addr.html#method.is_multicast
[`Ipv4Addr::is_broadcast`]: https://doc.crablang.org/stable/std/net/struct.Ipv4Addr.html#method.is_broadcast
[`Ipv4Addr::is_documentation`]: https://doc.crablang.org/stable/std/net/struct.Ipv4Addr.html#method.is_documentation
[`Ipv4Addr::to_ipv6_compatible`]: https://doc.crablang.org/stable/std/net/struct.Ipv4Addr.html#method.to_ipv6_compatible
[`Ipv4Addr::to_ipv6_mapped`]: https://doc.crablang.org/stable/std/net/struct.Ipv4Addr.html#method.to_ipv6_mapped
[`Ipv6Addr::segments`]: https://doc.crablang.org/stable/std/net/struct.Ipv6Addr.html#method.segments
[`Ipv6Addr::is_unspecified`]: https://doc.crablang.org/stable/std/net/struct.Ipv6Addr.html#method.is_unspecified
[`Ipv6Addr::is_loopback`]: https://doc.crablang.org/stable/std/net/struct.Ipv6Addr.html#method.is_loopback
[`Ipv6Addr::is_multicast`]: https://doc.crablang.org/stable/std/net/struct.Ipv6Addr.html#method.is_multicast
[`Ipv6Addr::to_ipv4`]: https://doc.crablang.org/stable/std/net/struct.Ipv6Addr.html#method.to_ipv4
[`Layout::align`]: https://doc.crablang.org/stable/std/alloc/struct.Layout.html#method.align
[`Layout::from_size_align`]: https://doc.crablang.org/stable/std/alloc/struct.Layout.html#method.from_size_align
[`Layout::size`]: https://doc.crablang.org/stable/std/alloc/struct.Layout.html#method.size
[`Ord::clamp`]: https://doc.crablang.org/stable/std/cmp/trait.Ord.html#method.clamp
[`RefCell::take`]: https://doc.crablang.org/stable/std/cell/struct.RefCell.html#method.take
[`UnsafeCell::get_mut`]: https://doc.crablang.org/stable/std/cell/struct.UnsafeCell.html#method.get_mut
[`bool::then`]: https://doc.crablang.org/stable/std/primitive.bool.html#method.then
[`btree_map::Entry::or_insert_with_key`]: https://doc.crablang.org/stable/std/collections/btree_map/enum.Entry.html#method.or_insert_with_key
[`f32::clamp`]: https://doc.crablang.org/stable/std/primitive.f32.html#method.clamp
[`f64::clamp`]: https://doc.crablang.org/stable/std/primitive.f64.html#method.clamp
[`hash_map::Entry::or_insert_with_key`]: https://doc.crablang.org/stable/std/collections/hash_map/enum.Entry.html#method.or_insert_with_key
[`slice::fill`]: https://doc.crablang.org/stable/std/primitive.slice.html#method.fill


Version 1.49.0 (2020-12-31)
============================

Language
-----------------------

- [Unions can now implement `Drop`, and you can now have a field in a union
  with `ManuallyDrop<T>`.][77547]
- [You can now cast uninhabited enums to integers.][76199]
- [You can now bind by reference and by move in patterns.][76119] This
  allows you to selectively borrow individual components of a type. E.g.
  ```crablang
  #[derive(Debug)]
  struct Person {
      name: String,
      age: u8,
  }

  let person = Person {
      name: String::from("Alice"),
      age: 20,
  };

  // `name` is moved out of person, but `age` is referenced.
  let Person { name, ref age } = person;
  println!("{} {}", name, age);
  ```

Compiler
-----------------------

- [Added tier 1\* support for `aarch64-unknown-linux-gnu`.][78228]
- [Added tier 2 support for `aarch64-apple-darwin`.][75991]
- [Added tier 2 support for `aarch64-pc-windows-msvc`.][75914]
- [Added tier 3 support for `mipsel-unknown-none`.][78676]
- [Raised the minimum supported LLVM version to LLVM 9.][78848]
- [Output from threads spawned in tests is now captured.][78227]
- [Change os and vendor values to "none" and "unknown" for some targets][78951]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
-----------------------

- [`RangeInclusive` now checks for exhaustion when calling `contains` and indexing.][78109]
- [`ToString::to_string` now no longer shrinks the internal buffer in the default implementation.][77997]

Stabilized APIs
---------------

- [`slice::select_nth_unstable`]
- [`slice::select_nth_unstable_by`]
- [`slice::select_nth_unstable_by_key`]

The following previously stable methods are now `const`.

- [`Poll::is_ready`]
- [`Poll::is_pending`]

Cargo
-----------------------
- [Building a crate with `cargo-package` should now be independently reproducible.][cargo/8864]
- [`cargo-tree` now marks proc-macro crates.][cargo/8765]
- [Added `CARGO_PRIMARY_PACKAGE` build-time environment variable.][cargo/8758] This
  variable will be set if the crate being built is one the user selected to build, either
  with `-p` or through defaults.
- [You can now use glob patterns when specifying packages & targets.][cargo/8752]


Compatibility Notes
-------------------

- [Demoted `i686-unknown-freebsd` from host tier 2 to target tier 2 support.][78746]
- [Macros that end with a semi-colon are now treated as statements even if they expand to nothing.][78376]
- [CrabLangc will now check for the validity of some built-in attributes on enum variants.][77015]
  Previously such invalid or unused attributes could be ignored.
- Leading whitespace is stripped more uniformly in documentation comments, which may change behavior. You
  read [this post about the changes][crablangdoc-ws-post] for more details.
- [Trait bounds are no longer inferred for associated types.][79904]

Internal Only
-------------
These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc and
related tools.

- [crablangc's internal crates are now compiled using the `initial-exec` Thread
  Local Storage model.][78201]
- [Calculate visibilities once in resolve.][78077]
- [Added `system` to the `llvm-libunwind` bootstrap config option.][77703]
- [Added `--color` for configuring terminal color support to bootstrap.][79004]


[75991]: https://github.com/crablang/crablang/pull/75991
[78951]: https://github.com/crablang/crablang/pull/78951
[78848]: https://github.com/crablang/crablang/pull/78848
[78746]: https://github.com/crablang/crablang/pull/78746
[78376]: https://github.com/crablang/crablang/pull/78376
[78228]: https://github.com/crablang/crablang/pull/78228
[78227]: https://github.com/crablang/crablang/pull/78227
[78201]: https://github.com/crablang/crablang/pull/78201
[78109]: https://github.com/crablang/crablang/pull/78109
[78077]: https://github.com/crablang/crablang/pull/78077
[77997]: https://github.com/crablang/crablang/pull/77997
[77703]: https://github.com/crablang/crablang/pull/77703
[77547]: https://github.com/crablang/crablang/pull/77547
[77015]: https://github.com/crablang/crablang/pull/77015
[76199]: https://github.com/crablang/crablang/pull/76199
[76119]: https://github.com/crablang/crablang/pull/76119
[75914]: https://github.com/crablang/crablang/pull/75914
[79004]: https://github.com/crablang/crablang/pull/79004
[78676]: https://github.com/crablang/crablang/pull/78676
[79904]: https://github.com/crablang/crablang/issues/79904
[cargo/8864]: https://github.com/crablang/cargo/pull/8864
[cargo/8765]: https://github.com/crablang/cargo/pull/8765
[cargo/8758]: https://github.com/crablang/cargo/pull/8758
[cargo/8752]: https://github.com/crablang/cargo/pull/8752
[`slice::select_nth_unstable`]: https://doc.crablang.org/nightly/std/primitive.slice.html#method.select_nth_unstable
[`slice::select_nth_unstable_by`]: https://doc.crablang.org/nightly/std/primitive.slice.html#method.select_nth_unstable_by
[`slice::select_nth_unstable_by_key`]: https://doc.crablang.org/nightly/std/primitive.slice.html#method.select_nth_unstable_by_key
[`Poll::is_ready`]: https://doc.crablang.org/stable/std/task/enum.Poll.html#method.is_ready
[`Poll::is_pending`]: https://doc.crablang.org/stable/std/task/enum.Poll.html#method.is_pending
[crablangdoc-ws-post]: https://blog.guillaume-gomez.fr/articles/2020-11-11+New+doc+comment+handling+in+crablangdoc

Version 1.48.0 (2020-11-19)
==========================

Language
--------

- [The `unsafe` keyword is now syntactically permitted on modules.][75857] This
  is still rejected *semantically*, but can now be parsed by procedural macros.

Compiler
--------
- [Stabilised the `-C link-self-contained=<yes|no>` compiler flag.][76158] This tells
  `crablangc` whether to link its own C runtime and libraries or to rely on a external
  linker to find them. (Supported only on `windows-gnu`, `linux-musl`, and `wasi` platforms.)
- [You can now use `-C target-feature=+crt-static` on `linux-gnu` targets.][77386]
  Note: If you're using cargo you must explicitly pass the `--target` flag.
- [Added tier 2\* support for `aarch64-unknown-linux-musl`.][76420]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
---------
- [`io::Write` is now implemented for `&ChildStdin` `&Sink`, `&Stdout`,
  and `&Stderr`.][76275]
- [All arrays of any length now implement `TryFrom<Vec<T>>`.][76310]
- [The `matches!` macro now supports having a trailing comma.][74880]
- [`Vec<A>` now implements `PartialEq<[B]>` where `A: PartialEq<B>`.][74194]
- [The `RefCell::{replace, replace_with, clone}` methods now all use `#[track_caller]`.][77055]

Stabilized APIs
---------------
- [`slice::as_ptr_range`]
- [`slice::as_mut_ptr_range`]
- [`VecDeque::make_contiguous`]
- [`future::pending`]
- [`future::ready`]

The following previously stable methods are now `const fn`'s:

- [`Option::is_some`]
- [`Option::is_none`]
- [`Option::as_ref`]
- [`Result::is_ok`]
- [`Result::is_err`]
- [`Result::as_ref`]
- [`Ordering::reverse`]
- [`Ordering::then`]

Cargo
-----

CrabLangdoc
-------
- [You can now link to items in `crablangdoc` using the intra-doc link
  syntax.][74430] E.g. ``/// Uses [`std::future`]`` will automatically generate
  a link to `std::future`'s documentation. See ["Linking to items by
  name"][intradoc-links] for more information.
- [You can now specify `#[doc(alias = "<alias>")]` on items to add search aliases
  when searching through `crablangdoc`'s UI.][75740]

Compatibility Notes
-------------------
- [Promotion of references to `'static` lifetime inside `const fn` now follows the
  same rules as inside a `fn` body.][75502] In particular, `&foo()` will not be
  promoted to `'static` lifetime any more inside `const fn`s.
- [Associated type bindings on trait objects are now verified to meet the bounds
  declared on the trait when checking that they implement the trait.][27675]
- [When trait bounds on associated types or opaque types are ambiguous, the
  compiler no longer makes an arbitrary choice on which bound to use.][54121]
- [Fixed recursive nonterminals not being expanded in macros during
  pretty-print/reparse check.][77153] This may cause errors if your macro wasn't
  correctly handling recursive nonterminal tokens.
- [`&mut` references to non zero-sized types are no longer promoted.][75585]
- [`crablangc` will now warn if you use attributes like `#[link_name]` or `#[cold]`
  in places where they have no effect.][73461]
- [Updated `_mm256_extract_epi8` and `_mm256_extract_epi16` signatures in
  `arch::{x86, x86_64}` to return `i32` to match the vendor signatures.][73166]
- [`mem::uninitialized` will now panic if any inner types inside a struct or enum
  disallow zero-initialization.][71274]
- [`#[target_feature]` will now error if used in a place where it has no effect.][78143]
- [Foreign exceptions are now caught by `catch_unwind` and will cause an abort.][70212]
  Note: This behaviour is not guaranteed and is still considered undefined behaviour,
  see the [`catch_unwind`] documentation for further information.



Internal Only
-------------
These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc and
related tools.

- [Building `crablangc` from source now uses `ninja` by default over `make`.][74922]
  You can continue building with `make` by setting `ninja=false` in
  your `config.toml`.
- [cg_llvm: `fewer_names` in `uncached_llvm_type`][76030]
- [Made `ensure_sufficient_stack()` non-generic][76680]

[78143]: https://github.com/crablang/crablang/issues/78143
[76680]: https://github.com/crablang/crablang/pull/76680/
[76030]: https://github.com/crablang/crablang/pull/76030/
[70212]: https://github.com/crablang/crablang/pull/70212/
[27675]: https://github.com/crablang/crablang/issues/27675/
[54121]: https://github.com/crablang/crablang/issues/54121/
[71274]: https://github.com/crablang/crablang/pull/71274/
[77386]: https://github.com/crablang/crablang/pull/77386/
[77153]: https://github.com/crablang/crablang/pull/77153/
[77055]: https://github.com/crablang/crablang/pull/77055/
[76275]: https://github.com/crablang/crablang/pull/76275/
[76310]: https://github.com/crablang/crablang/pull/76310/
[76420]: https://github.com/crablang/crablang/pull/76420/
[76158]: https://github.com/crablang/crablang/pull/76158/
[75857]: https://github.com/crablang/crablang/pull/75857/
[75585]: https://github.com/crablang/crablang/pull/75585/
[75740]: https://github.com/crablang/crablang/pull/75740/
[75502]: https://github.com/crablang/crablang/pull/75502/
[74880]: https://github.com/crablang/crablang/pull/74880/
[74922]: https://github.com/crablang/crablang/pull/74922/
[74430]: https://github.com/crablang/crablang/pull/74430/
[74194]: https://github.com/crablang/crablang/pull/74194/
[73461]: https://github.com/crablang/crablang/pull/73461/
[73166]: https://github.com/crablang/crablang/pull/73166/
[intradoc-links]: https://doc.crablang.org/crablangdoc/linking-to-items-by-name.html
[`catch_unwind`]: https://doc.crablang.org/std/panic/fn.catch_unwind.html
[`Option::is_some`]: https://doc.crablang.org/std/option/enum.Option.html#method.is_some
[`Option::is_none`]: https://doc.crablang.org/std/option/enum.Option.html#method.is_none
[`Option::as_ref`]: https://doc.crablang.org/std/option/enum.Option.html#method.as_ref
[`Result::is_ok`]: https://doc.crablang.org/std/result/enum.Result.html#method.is_ok
[`Result::is_err`]: https://doc.crablang.org/std/result/enum.Result.html#method.is_err
[`Result::as_ref`]: https://doc.crablang.org/std/result/enum.Result.html#method.as_ref
[`Ordering::reverse`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.reverse
[`Ordering::then`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.then
[`slice::as_ptr_range`]: https://doc.crablang.org/std/primitive.slice.html#method.as_ptr_range
[`slice::as_mut_ptr_range`]: https://doc.crablang.org/std/primitive.slice.html#method.as_mut_ptr_range
[`VecDeque::make_contiguous`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.make_contiguous
[`future::pending`]: https://doc.crablang.org/std/future/fn.pending.html
[`future::ready`]: https://doc.crablang.org/std/future/fn.ready.html


Version 1.47.0 (2020-10-08)
==========================

Language
--------
- [Closures will now warn when not used.][74869]

Compiler
--------
- [Stabilized the `-C control-flow-guard` codegen option][73893], which enables
  [Control Flow Guard][1.47.0-cfg] for Windows platforms, and is ignored on other
  platforms.
- [Upgraded to LLVM 11.][73526]
- [Added tier 3\* support for the `thumbv4t-none-eabi` target.][74419]
- [Upgrade the FreeBSD toolchain to version 11.4][75204]
- [`CRABLANG_BACKTRACE`'s output is now more compact.][75048]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
---------
- [`CStr` now implements `Index<RangeFrom<usize>>`.][74021]
- [Traits in `std`/`core` are now implemented for arrays of any length, not just
  those of length less than 33.][74060]
- [`ops::RangeFull` and `ops::Range` now implement Default.][73197]
- [`panic::Location` now implements `Copy`, `Clone`, `Eq`, `Hash`, `Ord`,
  `PartialEq`, and `PartialOrd`.][73583]

Stabilized APIs
---------------
- [`Ident::new_raw`]
- [`Range::is_empty`]
- [`RangeInclusive::is_empty`]
- [`Result::as_deref`]
- [`Result::as_deref_mut`]
- [`Vec::leak`]
- [`pointer::offset_from`]
- [`f32::TAU`]
- [`f64::TAU`]

The following previously stable APIs have now been made const.

- [The `new` method for all `NonZero` integers.][73858]
- [The `checked_add`,`checked_sub`,`checked_mul`,`checked_neg`, `checked_shl`,
  `checked_shr`, `saturating_add`, `saturating_sub`, and `saturating_mul`
  methods for all integers.][73858]
- [The `checked_abs`, `saturating_abs`, `saturating_neg`, and `signum`  for all
  signed integers.][73858]
- [The `is_ascii_alphabetic`, `is_ascii_uppercase`, `is_ascii_lowercase`,
  `is_ascii_alphanumeric`, `is_ascii_digit`, `is_ascii_hexdigit`,
  `is_ascii_punctuation`, `is_ascii_graphic`, `is_ascii_whitespace`, and
  `is_ascii_control` methods for `char` and `u8`.][73858]

Cargo
-----
- [`build-dependencies` are now built with opt-level 0 by default.][cargo/8500]
  You can override this by setting the following in your `Cargo.toml`.
  ```toml
  [profile.release.build-override]
  opt-level = 3
  ```
- [`cargo-help` will now display man pages for commands rather just the
  `--help` text.][cargo/8456]
- [`cargo-metadata` now emits a `test` field indicating if a target has
  tests enabled.][cargo/8478]
- [`workspace.default-members` now respects `workspace.exclude`.][cargo/8485]
- [`cargo-publish` will now use an alternative registry by default if it's the
  only registry specified in `package.publish`.][cargo/8571]

Misc
----
- [Added a help button beside CrabLangdoc's searchbar that explains crablangdoc's
  type based search.][75366]
- [Added the Ayu theme to crablangdoc.][71237]

Compatibility Notes
-------------------
- [Bumped the minimum supported Emscripten version to 1.39.20.][75716]
- [Fixed a regression parsing `{} && false` in tail expressions.][74650]
- [Added changes to how proc-macros are expanded in `macro_rules!` that should
  help to preserve more span information.][73084] These changes may cause
  compilation errors if your macro was unhygenic or didn't correctly handle
  `Delimiter::None`.
- [Moved support for the CloudABI target to tier 3.][75568]
- [`linux-gnu` targets now require minimum kernel 2.6.32 and glibc 2.11.][74163]
- [Added the `crablangc-docs` component.][75560] This allows you to install
  and read the documentation for the compiler internal APIs. (Currently only
  available for `x86_64-unknown-linux-gnu`.)

Internal Only
--------

- [Improved default settings for bootstrapping in `x.py`.][73964] You can read details about this change in the ["Changes to `x.py` defaults"](https://blog.crablang.org/inside-crablang/2020/08/30/changes-to-x-py-defaults.html) post on the Inside CrabLang blog.

[1.47.0-cfg]: https://docs.microsoft.com/en-us/windows/win32/secbp/control-flow-guard
[75048]: https://github.com/crablang/crablang/pull/75048/
[74163]: https://github.com/crablang/crablang/pull/74163/
[71237]: https://github.com/crablang/crablang/pull/71237/
[74869]: https://github.com/crablang/crablang/pull/74869/
[73858]: https://github.com/crablang/crablang/pull/73858/
[75716]: https://github.com/crablang/crablang/pull/75716/
[75560]: https://github.com/crablang/crablang/pull/75560/
[75568]: https://github.com/crablang/crablang/pull/75568/
[75366]: https://github.com/crablang/crablang/pull/75366/
[75204]: https://github.com/crablang/crablang/pull/75204/
[74650]: https://github.com/crablang/crablang/pull/74650/
[74419]: https://github.com/crablang/crablang/pull/74419/
[73964]: https://github.com/crablang/crablang/pull/73964/
[74021]: https://github.com/crablang/crablang/pull/74021/
[74060]: https://github.com/crablang/crablang/pull/74060/
[73893]: https://github.com/crablang/crablang/pull/73893/
[73526]: https://github.com/crablang/crablang/pull/73526/
[73583]: https://github.com/crablang/crablang/pull/73583/
[73084]: https://github.com/crablang/crablang/pull/73084/
[73197]: https://github.com/crablang/crablang/pull/73197/
[cargo/8456]: https://github.com/crablang/cargo/pull/8456/
[cargo/8478]: https://github.com/crablang/cargo/pull/8478/
[cargo/8485]: https://github.com/crablang/cargo/pull/8485/
[cargo/8500]: https://github.com/crablang/cargo/pull/8500/
[cargo/8571]: https://github.com/crablang/cargo/pull/8571/
[`Ident::new_raw`]:  https://doc.crablang.org/nightly/proc_macro/struct.Ident.html#method.new_raw
[`Range::is_empty`]: https://doc.crablang.org/nightly/std/ops/struct.Range.html#method.is_empty
[`RangeInclusive::is_empty`]: https://doc.crablang.org/nightly/std/ops/struct.RangeInclusive.html#method.is_empty
[`Result::as_deref_mut`]: https://doc.crablang.org/nightly/std/result/enum.Result.html#method.as_deref_mut
[`Result::as_deref`]: https://doc.crablang.org/nightly/std/result/enum.Result.html#method.as_deref
[`Vec::leak`]: https://doc.crablang.org/nightly/std/vec/struct.Vec.html#method.leak
[`f32::TAU`]: https://doc.crablang.org/nightly/std/f32/consts/constant.TAU.html
[`f64::TAU`]: https://doc.crablang.org/nightly/std/f64/consts/constant.TAU.html
[`pointer::offset_from`]: https://doc.crablang.org/nightly/std/primitive.pointer.html#method.offset_from


Version 1.46.0 (2020-08-27)
==========================

Language
--------
- [`if`, `match`, and `loop` expressions can now be used in const functions.][72437]
- [Additionally you are now also able to coerce and cast to slices (`&[T]`) in
  const functions.][73862]
- [The `#[track_caller]` attribute can now be added to functions to use the
  function's caller's location information for panic messages.][72445]
- [Recursively indexing into tuples no longer needs parentheses.][71322] E.g.
  `x.0.0` over `(x.0).0`.
- [`mem::transmute` can now be used in statics and constants.][72920] **Note**
  You currently can't use `mem::transmute` in constant functions.

Compiler
--------
- [You can now use the `cdylib` target on Apple iOS and tvOS platforms.][73516]
- [Enabled static "Position Independent Executables" by default
  for `x86_64-unknown-linux-musl`.][70740]

Libraries
---------
- [`mem::forget` is now a `const fn`.][73887]
- [`String` now implements `From<char>`.][73466]
- [The `leading_ones`, and `trailing_ones` methods have been stabilised for all
  integer types.][73032]
- [`vec::IntoIter<T>` now implements `AsRef<[T]>`.][72583]
- [All non-zero integer types (`NonZeroU8`) now implement `TryFrom` for their
  zero-able equivalent (e.g. `TryFrom<u8>`).][72717]
- [`&[T]` and `&mut [T]` now implement `PartialEq<Vec<T>>`.][71660]
- [`(String, u16)` now implements `ToSocketAddrs`.][73007]
- [`vec::Drain<'_, T>` now implements `AsRef<[T]>`.][72584]

Stabilized APIs
---------------
- [`Option::zip`]
- [`vec::Drain::as_slice`]

Cargo
-----
Added a number of new environment variables that are now available when
compiling your crate.

- [`CARGO_BIN_NAME` and `CARGO_CRATE_NAME`][cargo/8270] Providing the name of
  the specific binary being compiled and the name of the crate.
- [`CARGO_PKG_LICENSE`][cargo/8325] The license from the manifest of the package.
- [`CARGO_PKG_LICENSE_FILE`][cargo/8387] The path to the license file.

Compatibility Notes
-------------------
- [The target configuration option `abi_blacklist` has been renamed
  to `unsupported_abis`.][74150] The old name will still continue to work.
- [CrabLangc will now warn if you cast a C-like enum that implements `Drop`.][72331]
  This was previously accepted but will become a hard error in a future release.
- [CrabLangc will fail to compile if you have a struct with
  `#[repr(i128)]` or `#[repr(u128)]`.][74109] This representation is currently only
  allowed on `enum`s.
- [Tokens passed to `macro_rules!` are now always captured.][73293] This helps
  ensure that spans have the correct information, and may cause breakage if you
  were relying on receiving spans with dummy information.
- [The InnoSetup installer for Windows is no longer available.][72569] This was
  a legacy installer that was replaced by a MSI installer a few years ago but
  was still being built.
- [`{f32, f64}::asinh` now returns the correct values for negative numbers.][72486]
- [CrabLangc will no longer accept overlapping trait implementations that only
  differ in how the lifetime was bound.][72493]
- [CrabLangc now correctly relates the lifetime of an existential associated
  type.][71896] This fixes some edge cases where `crablangc` would erroneously allow
  you to pass a shorter lifetime than expected.
- [CrabLangc now dynamically links to `libz` (also called `zlib`) on Linux.][74420]
  The library will need to be installed for `crablangc` to work, even though we
  expect it to be already available on most systems.
- [Tests annotated with `#[should_panic]` are broken on ARMv7 while running
  under QEMU.][74820]
- [Pretty printing of some tokens in procedural macros changed.][75453] The
  exact output returned by crablangc's pretty printing is an unstable
  implementation detail: we recommend any macro relying on it to switch to a
  more robust parsing system.

[75453]: https://github.com/crablang/crablang/issues/75453/
[74820]: https://github.com/crablang/crablang/issues/74820/
[74420]: https://github.com/crablang/crablang/issues/74420/
[74109]: https://github.com/crablang/crablang/pull/74109/
[74150]: https://github.com/crablang/crablang/pull/74150/
[73862]: https://github.com/crablang/crablang/pull/73862/
[73887]: https://github.com/crablang/crablang/pull/73887/
[73466]: https://github.com/crablang/crablang/pull/73466/
[73516]: https://github.com/crablang/crablang/pull/73516/
[73293]: https://github.com/crablang/crablang/pull/73293/
[73007]: https://github.com/crablang/crablang/pull/73007/
[73032]: https://github.com/crablang/crablang/pull/73032/
[72920]: https://github.com/crablang/crablang/pull/72920/
[72569]: https://github.com/crablang/crablang/pull/72569/
[72583]: https://github.com/crablang/crablang/pull/72583/
[72584]: https://github.com/crablang/crablang/pull/72584/
[72717]: https://github.com/crablang/crablang/pull/72717/
[72437]: https://github.com/crablang/crablang/pull/72437/
[72445]: https://github.com/crablang/crablang/pull/72445/
[72486]: https://github.com/crablang/crablang/pull/72486/
[72493]: https://github.com/crablang/crablang/pull/72493/
[72331]: https://github.com/crablang/crablang/pull/72331/
[71896]: https://github.com/crablang/crablang/pull/71896/
[71660]: https://github.com/crablang/crablang/pull/71660/
[71322]: https://github.com/crablang/crablang/pull/71322/
[70740]: https://github.com/crablang/crablang/pull/70740/
[cargo/8270]: https://github.com/crablang/cargo/pull/8270/
[cargo/8325]: https://github.com/crablang/cargo/pull/8325/
[cargo/8387]: https://github.com/crablang/cargo/pull/8387/
[`Option::zip`]: https://doc.crablang.org/stable/std/option/enum.Option.html#method.zip
[`vec::Drain::as_slice`]: https://doc.crablang.org/stable/std/vec/struct.Drain.html#method.as_slice


Version 1.45.2 (2020-08-03)
==========================

* [Fix bindings in tuple struct patterns][74954]
* [Fix track_caller integration with trait objects][74784]

[74954]: https://github.com/crablang/crablang/issues/74954
[74784]: https://github.com/crablang/crablang/issues/74784


Version 1.45.1 (2020-07-30)
==========================

* [Fix const propagation with references.][73613]
* [crablangfmt accepts crablangfmt_skip in cfg_attr again.][73078]
* [Avoid spurious implicit region bound.][74509]
* [Install clippy on x.py install][74457]

[73613]: https://github.com/crablang/crablang/pull/73613
[73078]: https://github.com/crablang/crablang/issues/73078
[74509]: https://github.com/crablang/crablang/pull/74509
[74457]: https://github.com/crablang/crablang/pull/74457


Version 1.45.0 (2020-07-16)
==========================

Language
--------
- [Out of range float to int conversions using `as` has been defined as a saturating
  conversion.][71269] This was previously undefined behaviour, but you can use the
   `{f64, f32}::to_int_unchecked` methods to continue using the current behaviour, which
   may be desirable in rare performance sensitive situations.
- [`mem::Discriminant<T>` now uses `T`'s discriminant type instead of always
  using `u64`.][70705]
- [Function like procedural macros can now be used in expression, pattern, and  statement
  positions.][68717] This means you can now use a function-like procedural macro
  anywhere you can use a declarative (`macro_rules!`) macro.

Compiler
--------
- [You can now override individual target features through the `target-feature`
  flag.][72094] E.g. `-C target-feature=+avx2 -C target-feature=+fma` is now
  equivalent to `-C target-feature=+avx2,+fma`.
- [Added the `force-unwind-tables` flag.][69984] This option allows
  crablangc to always generate unwind tables regardless of panic strategy.
- [Added the `embed-bitcode` flag.][71716] This codegen flag allows crablangc
  to include LLVM bitcode into generated `rlib`s (this is on by default).
- [Added the `tiny` value to the `code-model` codegen flag.][72397]
- [Added tier 3 support\* for the `mipsel-sony-psp` target.][72062]
- [Added tier 3 support for the `thumbv7a-uwp-windows-msvc` target.][72133]
- [Upgraded to LLVM 10.][67759]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.


Libraries
---------
- [`net::{SocketAddr, SocketAddrV4, SocketAddrV6}` now implements `PartialOrd`
  and `Ord`.][72239]
- [`proc_macro::TokenStream` now implements `Default`.][72234]
- [You can now use `char` with
  `ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo}` to iterate over
  a range of codepoints.][72413] E.g.
  you can now write the following;
  ```crablang
  for ch in 'a'..='z' {
      print!("{}", ch);
  }
  println!();
  // Prints "abcdefghijklmnopqrstuvwxyz"
  ```
- [`OsString` now implements `FromStr`.][71662]
- [The `saturating_neg` method has been added to all signed integer primitive
  types, and the `saturating_abs` method has been added for all integer
  primitive types.][71886]
- [`Arc<T>`, `Rc<T>` now implement  `From<Cow<'_, T>>`, and `Box` now
  implements `From<Cow>` when `T` is `[T: Copy]`, `str`, `CStr`, `OsStr`,
  or `Path`.][71447]
- [`Box<[T]>` now implements `From<[T; N]>`.][71095]
- [`BitOr` and `BitOrAssign` are implemented for all `NonZero`
  integer types.][69813]
- [The `fetch_min`, and `fetch_max` methods have been added to all atomic
  integer types.][72324]
- [The `fetch_update` method has been added to all atomic integer types.][71843]

Stabilized APIs
---------------
- [`Arc::as_ptr`]
- [`BTreeMap::remove_entry`]
- [`Rc::as_ptr`]
- [`rc::Weak::as_ptr`]
- [`rc::Weak::from_raw`]
- [`rc::Weak::into_raw`]
- [`str::strip_prefix`]
- [`str::strip_suffix`]
- [`sync::Weak::as_ptr`]
- [`sync::Weak::from_raw`]
- [`sync::Weak::into_raw`]
- [`char::UNICODE_VERSION`]
- [`Span::resolved_at`]
- [`Span::located_at`]
- [`Span::mixed_site`]
- [`unix::process::CommandExt::arg0`]

Cargo
-----

- [Cargo uses the `embed-bitcode` flag to optimize disk usage and build
  time.][cargo/8066]

Misc
----
- [CrabLangdoc now supports strikethrough text in Markdown.][71928] E.g.
  `~~outdated information~~` becomes "~~outdated information~~".
- [Added an emoji to CrabLangdoc's deprecated API message.][72014]

Compatibility Notes
-------------------
- [Trying to self initialize a static value (that is creating a value using
  itself) is unsound and now causes a compile error.][71140]
- [`{f32, f64}::powi` now returns a slightly different value on Windows.][73420]
  This is due to changes in LLVM's intrinsics which `{f32, f64}::powi` uses.
- [CrabLangdoc's CLI's extra error exit codes have been removed.][71900] These were
  previously undocumented and not intended for public use. CrabLangdoc still provides
  a non-zero exit code on errors.
- [CrabLangc's `lto` flag is incompatible with the new `embed-bitcode=no`.][71848]
  This may cause issues if LTO is enabled through `CRABLANGFLAGS` or `cargo crablangc`
  flags while cargo is adding `embed-bitcode` itself. The recommended way to
  control LTO is with Cargo profiles, either in `Cargo.toml` or `.cargo/config`,
  or by setting `CARGO_PROFILE_<name>_LTO` in the environment.

Internals Only
--------------
- [Make clippy a git subtree instead of a git submodule][70655]
- [Unify the undo log of all snapshot types][69464]

[71848]: https://github.com/crablang/crablang/issues/71848/
[73420]: https://github.com/crablang/crablang/issues/73420/
[72324]: https://github.com/crablang/crablang/pull/72324/
[71843]: https://github.com/crablang/crablang/pull/71843/
[71886]: https://github.com/crablang/crablang/pull/71886/
[72234]: https://github.com/crablang/crablang/pull/72234/
[72239]: https://github.com/crablang/crablang/pull/72239/
[72397]: https://github.com/crablang/crablang/pull/72397/
[72413]: https://github.com/crablang/crablang/pull/72413/
[72014]: https://github.com/crablang/crablang/pull/72014/
[72062]: https://github.com/crablang/crablang/pull/72062/
[72094]: https://github.com/crablang/crablang/pull/72094/
[72133]: https://github.com/crablang/crablang/pull/72133/
[67759]: https://github.com/crablang/crablang/pull/67759/
[71900]: https://github.com/crablang/crablang/pull/71900/
[71928]: https://github.com/crablang/crablang/pull/71928/
[71662]: https://github.com/crablang/crablang/pull/71662/
[71716]: https://github.com/crablang/crablang/pull/71716/
[71447]: https://github.com/crablang/crablang/pull/71447/
[71269]: https://github.com/crablang/crablang/pull/71269/
[71095]: https://github.com/crablang/crablang/pull/71095/
[71140]: https://github.com/crablang/crablang/pull/71140/
[70655]: https://github.com/crablang/crablang/pull/70655/
[70705]: https://github.com/crablang/crablang/pull/70705/
[69984]: https://github.com/crablang/crablang/pull/69984/
[69813]: https://github.com/crablang/crablang/pull/69813/
[69464]: https://github.com/crablang/crablang/pull/69464/
[68717]: https://github.com/crablang/crablang/pull/68717/
[cargo/8066]: https://github.com/crablang/cargo/pull/8066
[`Arc::as_ptr`]: https://doc.crablang.org/stable/std/sync/struct.Arc.html#method.as_ptr
[`BTreeMap::remove_entry`]: https://doc.crablang.org/stable/std/collections/struct.BTreeMap.html#method.remove_entry
[`Rc::as_ptr`]: https://doc.crablang.org/stable/std/rc/struct.Rc.html#method.as_ptr
[`rc::Weak::as_ptr`]: https://doc.crablang.org/stable/std/rc/struct.Weak.html#method.as_ptr
[`rc::Weak::from_raw`]: https://doc.crablang.org/stable/std/rc/struct.Weak.html#method.from_raw
[`rc::Weak::into_raw`]: https://doc.crablang.org/stable/std/rc/struct.Weak.html#method.into_raw
[`sync::Weak::as_ptr`]: https://doc.crablang.org/stable/std/sync/struct.Weak.html#method.as_ptr
[`sync::Weak::from_raw`]: https://doc.crablang.org/stable/std/sync/struct.Weak.html#method.from_raw
[`sync::Weak::into_raw`]: https://doc.crablang.org/stable/std/sync/struct.Weak.html#method.into_raw
[`str::strip_prefix`]: https://doc.crablang.org/stable/std/primitive.str.html#method.strip_prefix
[`str::strip_suffix`]: https://doc.crablang.org/stable/std/primitive.str.html#method.strip_suffix
[`char::UNICODE_VERSION`]: https://doc.crablang.org/stable/std/char/constant.UNICODE_VERSION.html
[`Span::resolved_at`]: https://doc.crablang.org/stable/proc_macro/struct.Span.html#method.resolved_at
[`Span::located_at`]: https://doc.crablang.org/stable/proc_macro/struct.Span.html#method.located_at
[`Span::mixed_site`]: https://doc.crablang.org/stable/proc_macro/struct.Span.html#method.mixed_site
[`unix::process::CommandExt::arg0`]: https://doc.crablang.org/std/os/unix/process/trait.CommandExt.html#tymethod.arg0


Version 1.44.1 (2020-06-18)
===========================

* [crablangfmt accepts crablangfmt_skip in cfg_attr again.][73078]
* [Don't hash executable filenames on apple platforms, fixing backtraces.][cargo/8329]
* [Fix crashes when finding backtrace on macOS.][71397]
* [Clippy applies lint levels into different files.][clippy/5356]

[71397]: https://github.com/crablang/crablang/issues/71397
[73078]: https://github.com/crablang/crablang/issues/73078
[cargo/8329]: https://github.com/crablang/cargo/pull/8329
[clippy/5356]: https://github.com/crablang/crablang-clippy/issues/5356


Version 1.44.0 (2020-06-04)
==========================

Language
--------
- [You can now use `async/.await` with `#[no_std]` enabled.][69033]
- [Added the `unused_braces` lint.][70081]

**Syntax-only changes**

- [Expansion-driven outline module parsing][69838]
```crablang
#[cfg(FALSE)]
mod foo {
    mod bar {
        mod baz; // `foo/bar/baz.rs` doesn't exist, but no error!
    }
}
```

These are still rejected semantically, so you will likely receive an error but
these changes can be seen and parsed by macros and conditional compilation.

Compiler
--------
- [CrabLangc now respects the `-C codegen-units` flag in incremental mode.][70156]
  Additionally when in incremental mode crablangc defaults to 256 codegen units.
- [Refactored `catch_unwind` to have zero-cost, unless unwinding is enabled and
  a panic is thrown.][67502]
- [Added tier 3\* support for the `aarch64-unknown-none` and
  `aarch64-unknown-none-softfloat` targets.][68334]
- [Added tier 3 support for `arm64-apple-tvos` and
  `x86_64-apple-tvos` targets.][68191]


Libraries
---------
- [Special cased `vec![]` to map directly to `Vec::new()`.][70632] This allows
  `vec![]` to be able to be used in `const` contexts.
- [`convert::Infallible` now implements `Hash`.][70281]
- [`OsString` now implements `DerefMut` and `IndexMut` returning
  a `&mut OsStr`.][70048]
- [Unicode 13 is now supported.][69929]
- [`String` now implements `From<&mut str>`.][69661]
- [`IoSlice` now implements `Copy`.][69403]
- [`Vec<T>` now implements `From<[T; N]>`.][68692] Where `N` is at most 32.
- [`proc_macro::LexError` now implements `fmt::Display` and `Error`.][68899]
- [`from_le_bytes`, `to_le_bytes`, `from_be_bytes`, `to_be_bytes`,
  `from_ne_bytes`, and `to_ne_bytes` methods are now `const` for all
  integer types.][69373]

Stabilized APIs
---------------
- [`PathBuf::with_capacity`]
- [`PathBuf::capacity`]
- [`PathBuf::clear`]
- [`PathBuf::reserve`]
- [`PathBuf::reserve_exact`]
- [`PathBuf::shrink_to_fit`]
- [`f32::to_int_unchecked`]
- [`f64::to_int_unchecked`]
- [`Layout::align_to`]
- [`Layout::pad_to_align`]
- [`Layout::array`]
- [`Layout::extend`]

Cargo
-----
- [Added the `cargo tree` command which will print a tree graph of
  your dependencies.][cargo/8062] E.g.
  ```
    mdbook v0.3.2 (/Users/src/crablang/mdbook)
  ├── ammonia v3.0.0
  │   ├── html5ever v0.24.0
  │   │   ├── log v0.4.8
  │   │   │   └── cfg-if v0.1.9
  │   │   ├── mac v0.1.1
  │   │   └── markup5ever v0.9.0
  │   │       ├── log v0.4.8 (*)
  │   │       ├── phf v0.7.24
  │   │       │   └── phf_shared v0.7.24
  │   │       │       ├── siphasher v0.2.3
  │   │       │       └── unicase v1.4.2
  │   │       │           [build-dependencies]
  │   │       │           └── version_check v0.1.5
  ...
  ```
  You can also display dependencies on multiple versions of the same crate with
  `cargo tree -d` (short for `cargo tree --duplicates`).

Misc
----
- [CrabLangdoc now allows you to specify `--crate-version` to have crablangdoc include
  the version in the sidebar.][69494]

Compatibility Notes
-------------------
- [CrabLangc now correctly generates static libraries on Windows GNU targets with
  the `.a` extension, rather than the previous `.lib`.][70937]
- [Removed the `-C no_integrated_as` flag from crablangc.][70345]
- [The `file_name` property in JSON output of macro errors now points the actual
  source file rather than the previous format of `<NAME macros>`.][70969]
  **Note:** this may not point to a file that actually exists on the user's system.
- [The minimum required external LLVM version has been bumped to LLVM 8.][71147]
- [`mem::{zeroed, uninitialised}` will now panic when used with types that do
  not allow zero initialization such as `NonZeroU8`.][66059] This was
  previously a warning.
- [In 1.45.0 (the next release) converting a `f64` to `u32` using the `as`
  operator has been defined as a saturating operation.][71269] This was previously
  undefined behaviour, but you can use the `{f64, f32}::to_int_unchecked` methods to
  continue using the current behaviour, which may be desirable in rare performance
  sensitive situations.

Internal Only
-------------
These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of crablangc and
related tools.

- [dep_graph Avoid allocating a set on when the number reads are small.][69778]
- [Replace big JS dict with JSON parsing.][71250]

[69373]: https://github.com/crablang/crablang/pull/69373/
[66059]: https://github.com/crablang/crablang/pull/66059/
[68191]: https://github.com/crablang/crablang/pull/68191/
[68899]: https://github.com/crablang/crablang/pull/68899/
[71147]: https://github.com/crablang/crablang/pull/71147/
[71250]: https://github.com/crablang/crablang/pull/71250/
[70937]: https://github.com/crablang/crablang/pull/70937/
[70969]: https://github.com/crablang/crablang/pull/70969/
[70632]: https://github.com/crablang/crablang/pull/70632/
[70281]: https://github.com/crablang/crablang/pull/70281/
[70345]: https://github.com/crablang/crablang/pull/70345/
[70048]: https://github.com/crablang/crablang/pull/70048/
[70081]: https://github.com/crablang/crablang/pull/70081/
[70156]: https://github.com/crablang/crablang/pull/70156/
[71269]: https://github.com/crablang/crablang/pull/71269/
[69838]: https://github.com/crablang/crablang/pull/69838/
[69929]: https://github.com/crablang/crablang/pull/69929/
[69661]: https://github.com/crablang/crablang/pull/69661/
[69778]: https://github.com/crablang/crablang/pull/69778/
[69494]: https://github.com/crablang/crablang/pull/69494/
[69403]: https://github.com/crablang/crablang/pull/69403/
[69033]: https://github.com/crablang/crablang/pull/69033/
[68692]: https://github.com/crablang/crablang/pull/68692/
[68334]: https://github.com/crablang/crablang/pull/68334/
[67502]: https://github.com/crablang/crablang/pull/67502/
[cargo/8062]: https://github.com/crablang/cargo/pull/8062/
[`PathBuf::with_capacity`]: https://doc.crablang.org/std/path/struct.PathBuf.html#method.with_capacity
[`PathBuf::capacity`]: https://doc.crablang.org/std/path/struct.PathBuf.html#method.capacity
[`PathBuf::clear`]: https://doc.crablang.org/std/path/struct.PathBuf.html#method.clear
[`PathBuf::reserve`]: https://doc.crablang.org/std/path/struct.PathBuf.html#method.reserve
[`PathBuf::reserve_exact`]: https://doc.crablang.org/std/path/struct.PathBuf.html#method.reserve_exact
[`PathBuf::shrink_to_fit`]: https://doc.crablang.org/std/path/struct.PathBuf.html#method.shrink_to_fit
[`f32::to_int_unchecked`]: https://doc.crablang.org/std/primitive.f32.html#method.to_int_unchecked
[`f64::to_int_unchecked`]: https://doc.crablang.org/std/primitive.f64.html#method.to_int_unchecked
[`Layout::align_to`]: https://doc.crablang.org/std/alloc/struct.Layout.html#method.align_to
[`Layout::pad_to_align`]: https://doc.crablang.org/std/alloc/struct.Layout.html#method.pad_to_align
[`Layout::array`]: https://doc.crablang.org/std/alloc/struct.Layout.html#method.array
[`Layout::extend`]: https://doc.crablang.org/std/alloc/struct.Layout.html#method.extend


Version 1.43.1 (2020-05-07)
===========================

* [Updated openssl-src to 1.1.1g for CVE-2020-1967.][71430]
* [Fixed the stabilization of AVX-512 features.][71473]
* [Fixed `cargo package --list` not working with unpublished dependencies.][cargo/8151]

[71430]: https://github.com/crablang/crablang/pull/71430
[71473]: https://github.com/crablang/crablang/issues/71473
[cargo/8151]: https://github.com/crablang/cargo/issues/8151


Version 1.43.0 (2020-04-23)
==========================

Language
--------
- [Fixed using binary operations with `&{number}` (e.g. `&1.0`) not having
  the type inferred correctly.][68129]
- [Attributes such as `#[cfg()]` can now be used on `if` expressions.][69201]

**Syntax only changes**
- [Allow `type Foo: Ord` syntactically.][69361]
- [Fuse associated and extern items up to defaultness.][69194]
- [Syntactically allow `self` in all `fn` contexts.][68764]
- [Merge `fn` syntax + cleanup item parsing.][68728]
- [`item` macro fragments can be interpolated into `trait`s, `impl`s, and `extern` blocks.][69366]
  For example, you may now write:
  ```crablang
  macro_rules! mac_trait {
      ($i:item) => {
          trait T { $i }
      }
  }
  mac_trait! {
      fn foo() {}
  }
  ```

These are still rejected *semantically*, so you will likely receive an error but
these changes can be seen and parsed by macros and
conditional compilation.


Compiler
--------
- [You can now pass multiple lint flags to crablangc to override the previous
  flags.][67885] For example; `crablangc -D unused -A unused-variables` denies
  everything in the `unused` lint group except `unused-variables` which
  is explicitly allowed. However, passing `crablangc -A unused-variables -D unused` denies
  everything in the `unused` lint group **including** `unused-variables` since
  the allow flag is specified before the deny flag (and therefore overridden).
- [crablangc will now prefer your system MinGW libraries over its bundled libraries
  if they are available on `windows-gnu`.][67429]
- [crablangc now buffers errors/warnings printed in JSON.][69227]

Libraries
---------
- [`Arc<[T; N]>`, `Box<[T; N]>`, and `Rc<[T; N]>`, now implement
  `TryFrom<Arc<[T]>>`,`TryFrom<Box<[T]>>`, and `TryFrom<Rc<[T]>>`
  respectively.][69538] **Note** These conversions are only available when `N`
  is `0..=32`.
- [You can now use associated constants on floats and integers directly, rather
  than having to import the module.][68952] e.g. You can now write `u32::MAX` or
  `f32::NAN` with no imports.
- [`u8::is_ascii` is now `const`.][68984]
- [`String` now implements `AsMut<str>`.][68742]
- [Added the `primitive` module to `std` and `core`.][67637] This module
  reexports CrabLang's primitive types. This is mainly useful in macros
  where you want avoid these types being shadowed.
- [Relaxed some of the trait bounds on `HashMap` and `HashSet`.][67642]
- [`string::FromUtf8Error` now implements `Clone + Eq`.][68738]

Stabilized APIs
---------------
- [`Once::is_completed`]
- [`f32::LOG10_2`]
- [`f32::LOG2_10`]
- [`f64::LOG10_2`]
- [`f64::LOG2_10`]
- [`iter::once_with`]

Cargo
-----
- [You can now set config `[profile]`s in your `.cargo/config`, or through
  your environment.][cargo/7823]
- [Cargo will now set `CARGO_BIN_EXE_<name>` pointing to a binary's
  executable path when running integration tests or benchmarks.][cargo/7697]
  `<name>` is the name of your binary as-is e.g. If you wanted the executable
  path for a binary named `my-program`you would use `env!("CARGO_BIN_EXE_my-program")`.

Misc
----
- [Certain checks in the `const_err` lint were deemed unrelated to const
  evaluation][69185], and have been moved to the `unconditional_panic` and
  `arithmetic_overflow` lints.

Compatibility Notes
-------------------

- [Having trailing syntax in the `assert!` macro is now a hard error.][69548] This
  has been a warning since 1.36.0.
- [Fixed `Self` not having the correctly inferred type.][69340] This incorrectly
  led to some instances being accepted, and now correctly emits a hard error.

[69340]: https://github.com/crablang/crablang/pull/69340

Internal Only
-------------
These changes provide no direct user facing benefits, but represent significant
improvements to the internals and overall performance of `crablangc` and
related tools.

- [All components are now built with `opt-level=3` instead of `2`.][67878]
- [Improved how crablangc generates drop code.][67332]
- [Improved performance from `#[inline]`-ing certain hot functions.][69256]
- [traits: preallocate 2 Vecs of known initial size][69022]
- [Avoid exponential behaviour when relating types][68772]
- [Skip `Drop` terminators for enum variants without drop glue][68943]
- [Improve performance of coherence checks][68966]
- [Deduplicate types in the generator witness][68672]
- [Invert control in struct_lint_level.][68725]

[67332]: https://github.com/crablang/crablang/pull/67332/
[67429]: https://github.com/crablang/crablang/pull/67429/
[67637]: https://github.com/crablang/crablang/pull/67637/
[67642]: https://github.com/crablang/crablang/pull/67642/
[67878]: https://github.com/crablang/crablang/pull/67878/
[67885]: https://github.com/crablang/crablang/pull/67885/
[68129]: https://github.com/crablang/crablang/pull/68129/
[68672]: https://github.com/crablang/crablang/pull/68672/
[68725]: https://github.com/crablang/crablang/pull/68725/
[68728]: https://github.com/crablang/crablang/pull/68728/
[68738]: https://github.com/crablang/crablang/pull/68738/
[68742]: https://github.com/crablang/crablang/pull/68742/
[68764]: https://github.com/crablang/crablang/pull/68764/
[68772]: https://github.com/crablang/crablang/pull/68772/
[68943]: https://github.com/crablang/crablang/pull/68943/
[68952]: https://github.com/crablang/crablang/pull/68952/
[68966]: https://github.com/crablang/crablang/pull/68966/
[68984]: https://github.com/crablang/crablang/pull/68984/
[69022]: https://github.com/crablang/crablang/pull/69022/
[69185]: https://github.com/crablang/crablang/pull/69185/
[69194]: https://github.com/crablang/crablang/pull/69194/
[69201]: https://github.com/crablang/crablang/pull/69201/
[69227]: https://github.com/crablang/crablang/pull/69227/
[69548]: https://github.com/crablang/crablang/pull/69548/
[69256]: https://github.com/crablang/crablang/pull/69256/
[69361]: https://github.com/crablang/crablang/pull/69361/
[69366]: https://github.com/crablang/crablang/pull/69366/
[69538]: https://github.com/crablang/crablang/pull/69538/
[cargo/7823]: https://github.com/crablang/cargo/pull/7823
[cargo/7697]: https://github.com/crablang/cargo/pull/7697
[`Once::is_completed`]: https://doc.crablang.org/std/sync/struct.Once.html#method.is_completed
[`f32::LOG10_2`]: https://doc.crablang.org/std/f32/consts/constant.LOG10_2.html
[`f32::LOG2_10`]: https://doc.crablang.org/std/f32/consts/constant.LOG2_10.html
[`f64::LOG10_2`]: https://doc.crablang.org/std/f64/consts/constant.LOG10_2.html
[`f64::LOG2_10`]: https://doc.crablang.org/std/f64/consts/constant.LOG2_10.html
[`iter::once_with`]: https://doc.crablang.org/std/iter/fn.once_with.html


Version 1.42.0 (2020-03-12)
==========================

Language
--------
- [You can now use the slice pattern syntax with subslices.][67712] e.g.
  ```crablang
  fn foo(words: &[&str]) {
      match words {
          ["Hello", "World", "!", ..] => println!("Hello World!"),
          ["Foo", "Bar", ..] => println!("Baz"),
          rest => println!("{:?}", rest),
      }
  }
  ```
- [You can now use `#[repr(transparent)]` on univariant `enum`s.][68122] Meaning
  that you can create an enum that has the exact layout and ABI of the type
  it contains.
- [You can now use outer attribute procedural macros on inline modules.][64273]
- [There are some *syntax-only* changes:][67131]
   - `default` is syntactically allowed before items in `trait` definitions.
   - Items in `impl`s (i.e. `const`s, `type`s, and `fn`s) may syntactically
     leave out their bodies in favor of `;`.
   - Bounds on associated types in `impl`s are now syntactically allowed
     (e.g. `type Foo: Ord;`).
   - `...` (the C-variadic type) may occur syntactically directly as the type of
      any function parameter.

  These are still rejected *semantically*, so you will likely receive an error
  but these changes can be seen and parsed by procedural macros and
  conditional compilation.

Compiler
--------
- [Added tier 2\* support for `armv7a-none-eabi`.][68253]
- [Added tier 2 support for `riscv64gc-unknown-linux-gnu`.][68339]
- [`Option::{expect,unwrap}` and
   `Result::{expect, expect_err, unwrap, unwrap_err}` now produce panic messages
   pointing to the location where they were called, rather than
   `core`'s internals. ][67887]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
---------
- [`iter::Empty<T>` now implements `Send` and `Sync` for any `T`.][68348]
- [`Pin::{map_unchecked, map_unchecked_mut}` no longer require the return type
   to implement `Sized`.][67935]
- [`io::Cursor` now derives `PartialEq` and `Eq`.][67233]
- [`Layout::new` is now `const`.][66254]
- [Added Standard Library support for `riscv64gc-unknown-linux-gnu`.][66899]


Stabilized APIs
---------------
- [`CondVar::wait_while`]
- [`CondVar::wait_timeout_while`]
- [`DebugMap::key`]
- [`DebugMap::value`]
- [`ManuallyDrop::take`]
- [`matches!`]
- [`ptr::slice_from_raw_parts_mut`]
- [`ptr::slice_from_raw_parts`]

Cargo
-----
- [You no longer need to include `extern crate proc_macro;` to be able to
  `use proc_macro;` in the `2018` edition.][cargo/7700]

Compatibility Notes
-------------------
- [`Error::description` has been deprecated, and its use will now produce a
  warning.][66919] It's recommended to use `Display`/`to_string` instead.

[68253]: https://github.com/crablang/crablang/pull/68253/
[68348]: https://github.com/crablang/crablang/pull/68348/
[67935]: https://github.com/crablang/crablang/pull/67935/
[68339]: https://github.com/crablang/crablang/pull/68339/
[68122]: https://github.com/crablang/crablang/pull/68122/
[64273]: https://github.com/crablang/crablang/pull/64273/
[67712]: https://github.com/crablang/crablang/pull/67712/
[67887]: https://github.com/crablang/crablang/pull/67887/
[67131]: https://github.com/crablang/crablang/pull/67131/
[67233]: https://github.com/crablang/crablang/pull/67233/
[66899]: https://github.com/crablang/crablang/pull/66899/
[66919]: https://github.com/crablang/crablang/pull/66919/
[66254]: https://github.com/crablang/crablang/pull/66254/
[cargo/7700]: https://github.com/crablang/cargo/pull/7700
[`DebugMap::key`]: https://doc.crablang.org/stable/std/fmt/struct.DebugMap.html#method.key
[`DebugMap::value`]: https://doc.crablang.org/stable/std/fmt/struct.DebugMap.html#method.value
[`ManuallyDrop::take`]: https://doc.crablang.org/stable/std/mem/struct.ManuallyDrop.html#method.take
[`matches!`]: https://doc.crablang.org/stable/std/macro.matches.html
[`ptr::slice_from_raw_parts_mut`]: https://doc.crablang.org/stable/std/ptr/fn.slice_from_raw_parts_mut.html
[`ptr::slice_from_raw_parts`]: https://doc.crablang.org/stable/std/ptr/fn.slice_from_raw_parts.html
[`CondVar::wait_while`]: https://doc.crablang.org/stable/std/sync/struct.Condvar.html#method.wait_while
[`CondVar::wait_timeout_while`]: https://doc.crablang.org/stable/std/sync/struct.Condvar.html#method.wait_timeout_while


Version 1.41.1 (2020-02-27)
===========================

* [Always check types of static items][69145]
* [Always check lifetime bounds of `Copy` impls][69145]
* [Fix miscompilation in callers of `Layout::repeat`][69225]
* [CrabLang 1.41.0 was announced as the last CrabLang release with tier 1 or tier 2 support for 32-bit Apple targets][apple-32bit-drop].
  That announcement did not expect a patch release. 1.41.1 also includes release binaries for these targets.

[69225]: https://github.com/crablang/crablang/issues/69225
[69145]: https://github.com/crablang/crablang/pull/69145


Version 1.41.0 (2020-01-30)
===========================

Language
--------

- [You can now pass type parameters to foreign items when implementing
  traits.][65879] E.g. You can now write `impl<T> From<Foo> for Vec<T> {}`.
- [You can now arbitrarily nest receiver types in the `self` position.][64325] E.g. you can
  now write `fn foo(self: Box<Box<Self>>) {}`. Previously only `Self`, `&Self`,
  `&mut Self`, `Arc<Self>`, `Rc<Self>`, and `Box<Self>` were allowed.
- [You can now use any valid identifier in a `format_args` macro.][66847]
  Previously identifiers starting with an underscore were not allowed.
- [Visibility modifiers (e.g. `pub`) are now syntactically allowed on trait items and
  enum variants.][66183] These are still rejected semantically, but
  can be seen and parsed by procedural macros and conditional compilation.
- [You can now define a CrabLang `extern "C"` function with `Box<T>` and use `T*` as the corresponding
  type on the C side.][62514] Please see [the documentation][box-memory-layout] for more information,
  including the important caveat about preferring to avoid `Box<T>` in CrabLang signatures for functions defined in C.

[box-memory-layout]: https://doc.crablang.org/std/boxed/index.html#memory-layout

Compiler
--------

- [CrabLangc will now warn if you have unused loop `'label`s.][66325]
- [Removed support for the `i686-unknown-dragonfly` target.][67255]
- [Added tier 3 support\* for the `riscv64gc-unknown-linux-gnu` target.][66661]
- [You can now pass an arguments file passing the `@path` syntax
  to crablangc.][66172] Note that the format differs somewhat from what is
  found in other tooling; please see [the documentation][argfile-docs] for
  more information.
- [You can now provide `--extern` flag without a path, indicating that it is
  available from the search path or specified with an `-L` flag.][64882]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

[argfile-docs]: https://doc.crablang.org/nightly/crablangc/command-line-arguments.html#path-load-command-line-flags-from-a-path

Libraries
---------

- [The `core::panic` module is now stable.][66771] It was already stable
  through `std`.
- [`NonZero*` numerics now implement `From<NonZero*>` if it's a smaller integer
  width.][66277] E.g. `NonZeroU16` now implements `From<NonZeroU8>`.
- [`MaybeUninit<T>` now implements `fmt::Debug`.][65013]

Stabilized APIs
---------------

- [`Result::map_or`]
- [`Result::map_or_else`]
- [`std::rc::Weak::weak_count`]
- [`std::rc::Weak::strong_count`]
- [`std::sync::Weak::weak_count`]
- [`std::sync::Weak::strong_count`]

Cargo
-----

- [Cargo will now document all the private items for binary crates
  by default.][cargo/7593]
- [`cargo-install` will now reinstall the package if it detects that it is out
  of date.][cargo/7560]
- [Cargo.lock now uses a more git friendly format that should help to reduce
  merge conflicts.][cargo/7579]
- [You can now override specific dependencies's build settings][cargo/7591] E.g.
  `[profile.dev.package.image] opt-level = 2` sets the `image` crate's
  optimisation level to `2` for debug builds. You can also use
  `[profile.<profile>.build-override]` to override build scripts and
  their dependencies.

Misc
----

- [You can now specify `edition` in documentation code blocks to compile the block
  for that edition.][66238] E.g. `edition2018` tells crablangdoc that the code sample
  should be compiled the 2018 edition of CrabLang.
- [You can now provide custom themes to crablangdoc with `--theme`, and check the
  current theme with `--check-theme`.][54733]
- [You can use `#[cfg(doc)]` to compile an item when building documentation.][61351]

Compatibility Notes
-------------------

- [As previously announced 1.41 will be the last tier 1 release for 32-bit
  Apple targets.][apple-32bit-drop] This means that the source code is still
  available to build, but the targets are no longer being tested and release
  binaries for those platforms will no longer be distributed by the CrabLang project.
  Please refer to the linked blog post for more information.

[54733]: https://github.com/crablang/crablang/pull/54733/
[61351]: https://github.com/crablang/crablang/pull/61351/
[62514]: https://github.com/crablang/crablang/pull/62514/
[67255]: https://github.com/crablang/crablang/pull/67255/
[66661]: https://github.com/crablang/crablang/pull/66661/
[66771]: https://github.com/crablang/crablang/pull/66771/
[66847]: https://github.com/crablang/crablang/pull/66847/
[66238]: https://github.com/crablang/crablang/pull/66238/
[66277]: https://github.com/crablang/crablang/pull/66277/
[66325]: https://github.com/crablang/crablang/pull/66325/
[66172]: https://github.com/crablang/crablang/pull/66172/
[66183]: https://github.com/crablang/crablang/pull/66183/
[65879]: https://github.com/crablang/crablang/pull/65879/
[65013]: https://github.com/crablang/crablang/pull/65013/
[64882]: https://github.com/crablang/crablang/pull/64882/
[64325]: https://github.com/crablang/crablang/pull/64325/
[cargo/7560]: https://github.com/crablang/cargo/pull/7560/
[cargo/7579]: https://github.com/crablang/cargo/pull/7579/
[cargo/7591]: https://github.com/crablang/cargo/pull/7591/
[cargo/7593]: https://github.com/crablang/cargo/pull/7593/
[`Result::map_or_else`]: https://doc.crablang.org/std/result/enum.Result.html#method.map_or_else
[`Result::map_or`]: https://doc.crablang.org/std/result/enum.Result.html#method.map_or
[`std::rc::Weak::weak_count`]: https://doc.crablang.org/std/rc/struct.Weak.html#method.weak_count
[`std::rc::Weak::strong_count`]: https://doc.crablang.org/std/rc/struct.Weak.html#method.strong_count
[`std::sync::Weak::weak_count`]: https://doc.crablang.org/std/sync/struct.Weak.html#method.weak_count
[`std::sync::Weak::strong_count`]: https://doc.crablang.org/std/sync/struct.Weak.html#method.strong_count
[apple-32bit-drop]: https://blog.crablang.org/2020/01/03/reducing-support-for-32-bit-apple-targets.html

Version 1.40.0 (2019-12-19)
===========================

Language
--------
- [You can now use tuple `struct`s and tuple `enum` variant's constructors in
  `const` contexts.][65188] e.g.

  ```crablang
  pub struct Point(i32, i32);

  const ORIGIN: Point = {
      let constructor = Point;

      constructor(0, 0)
  };
  ```

- [You can now mark `struct`s, `enum`s, and `enum` variants with the `#[non_exhaustive]` attribute to
  indicate that there may be variants or fields added in the future.][64639]
  For example this requires adding a wild-card branch (`_ => {}`) to any match
  statements on a non-exhaustive `enum`. [(RFC 2008)]
- [You can now use function-like procedural macros in `extern` blocks and in
  type positions.][63931] e.g. `type Generated = macro!();`
- [Function-like and attribute procedural macros can now emit
  `macro_rules!` items, so you can now have your macros generate macros.][64035]
- [The `meta` pattern matcher in `macro_rules!` now correctly matches the modern
  attribute syntax.][63674] For example `(#[$m:meta])` now matches `#[attr]`,
  `#[attr{tokens}]`, `#[attr[tokens]]`, and `#[attr(tokens)]`.

Compiler
--------
- [Added tier 3 support\* for the
  `thumbv7neon-unknown-linux-musleabihf` target.][66103]
- [Added tier 3 support for the
  `aarch64-unknown-none-softfloat` target.][64589]
- [Added tier 3 support for the `mips64-unknown-linux-muslabi64`, and
  `mips64el-unknown-linux-muslabi64` targets.][65843]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
  information on CrabLang's tiered platform support.

Libraries
---------
- [The `is_power_of_two` method on unsigned numeric types is now a `const` function.][65092]

Stabilized APIs
---------------
- [`BTreeMap::get_key_value`]
- [`HashMap::get_key_value`]
- [`Option::as_deref_mut`]
- [`Option::as_deref`]
- [`Option::flatten`]
- [`UdpSocket::peer_addr`]
- [`f32::to_be_bytes`]
- [`f32::to_le_bytes`]
- [`f32::to_ne_bytes`]
- [`f64::to_be_bytes`]
- [`f64::to_le_bytes`]
- [`f64::to_ne_bytes`]
- [`f32::from_be_bytes`]
- [`f32::from_le_bytes`]
- [`f32::from_ne_bytes`]
- [`f64::from_be_bytes`]
- [`f64::from_le_bytes`]
- [`f64::from_ne_bytes`]
- [`mem::take`]
- [`slice::repeat`]
- [`todo!`]

Cargo
-----
- [Cargo will now always display warnings, rather than only on
  fresh builds.][cargo/7450]
- [Feature flags (except `--all-features`) passed to a virtual workspace will
  now produce an error.][cargo/7507] Previously these flags were ignored.
- [You can now publish `dev-dependencies` without including
  a `version`.][cargo/7333]

Misc
----
- [You can now specify the `#[cfg(doctest)]` attribute to include an item only
  when running documentation tests with `crablangdoc`.][63803]

Compatibility Notes
-------------------
- [As previously announced, any previous NLL warnings in the 2015 edition are
  now hard errors.][64221]
- [The `include!` macro will now warn if it failed to include the
  entire file.][64284] The `include!` macro unintentionally only includes the
  first _expression_ in a file, and this can be unintuitive. This will become
  either a hard error in a future release, or the behavior may be fixed to include all expressions as expected.
- [Using `#[inline]` on function prototypes and consts now emits a warning under
  `unused_attribute` lint.][65294] Using `#[inline]` anywhere else inside traits
  or `extern` blocks now correctly emits a hard error.

[65294]: https://github.com/crablang/crablang/pull/65294/
[66103]: https://github.com/crablang/crablang/pull/66103/
[65843]: https://github.com/crablang/crablang/pull/65843/
[65188]: https://github.com/crablang/crablang/pull/65188/
[65092]: https://github.com/crablang/crablang/pull/65092/
[64589]: https://github.com/crablang/crablang/pull/64589/
[64639]: https://github.com/crablang/crablang/pull/64639/
[64221]: https://github.com/crablang/crablang/pull/64221/
[64284]: https://github.com/crablang/crablang/pull/64284/
[63931]: https://github.com/crablang/crablang/pull/63931/
[64035]: https://github.com/crablang/crablang/pull/64035/
[63674]: https://github.com/crablang/crablang/pull/63674/
[63803]: https://github.com/crablang/crablang/pull/63803/
[cargo/7450]: https://github.com/crablang/cargo/pull/7450/
[cargo/7507]: https://github.com/crablang/cargo/pull/7507/
[cargo/7333]: https://github.com/crablang/cargo/pull/7333/
[(rfc 2008)]: https://crablang.github.io/rfcs/2008-non-exhaustive.html
[`f32::to_be_bytes`]: https://doc.crablang.org/std/primitive.f32.html#method.to_be_bytes
[`f32::to_le_bytes`]: https://doc.crablang.org/std/primitive.f32.html#method.to_le_bytes
[`f32::to_ne_bytes`]: https://doc.crablang.org/std/primitive.f32.html#method.to_ne_bytes
[`f64::to_be_bytes`]: https://doc.crablang.org/std/primitive.f64.html#method.to_be_bytes
[`f64::to_le_bytes`]: https://doc.crablang.org/std/primitive.f64.html#method.to_le_bytes
[`f64::to_ne_bytes`]: https://doc.crablang.org/std/primitive.f64.html#method.to_ne_bytes
[`f32::from_be_bytes`]: https://doc.crablang.org/std/primitive.f32.html#method.from_be_bytes
[`f32::from_le_bytes`]: https://doc.crablang.org/std/primitive.f32.html#method.from_le_bytes
[`f32::from_ne_bytes`]: https://doc.crablang.org/std/primitive.f32.html#method.from_ne_bytes
[`f64::from_be_bytes`]: https://doc.crablang.org/std/primitive.f64.html#method.from_be_bytes
[`f64::from_le_bytes`]: https://doc.crablang.org/std/primitive.f64.html#method.from_le_bytes
[`f64::from_ne_bytes`]: https://doc.crablang.org/std/primitive.f64.html#method.from_ne_bytes
[`option::flatten`]: https://doc.crablang.org/std/option/enum.Option.html#method.flatten
[`option::as_deref`]: https://doc.crablang.org/std/option/enum.Option.html#method.as_deref
[`option::as_deref_mut`]: https://doc.crablang.org/std/option/enum.Option.html#method.as_deref_mut
[`hashmap::get_key_value`]: https://doc.crablang.org/std/collections/struct.HashMap.html#method.get_key_value
[`btreemap::get_key_value`]: https://doc.crablang.org/std/collections/struct.BTreeMap.html#method.get_key_value
[`slice::repeat`]: https://doc.crablang.org/std/primitive.slice.html#method.repeat
[`mem::take`]: https://doc.crablang.org/std/mem/fn.take.html
[`udpsocket::peer_addr`]: https://doc.crablang.org/std/net/struct.UdpSocket.html#method.peer_addr
[`todo!`]: https://doc.crablang.org/std/macro.todo.html


Version 1.39.0 (2019-11-07)
===========================

Language
--------
- [You can now create `async` functions and blocks with `async fn`, `async move {}`, and
  `async {}` respectively, and you can now call `.await` on async expressions.][63209]
- [You can now use certain attributes on function, closure, and function pointer
  parameters.][64010] These attributes include `cfg`, `cfg_attr`, `allow`, `warn`,
  `deny`, `forbid` as well as inert helper attributes used by procedural macro
  attributes applied to items. e.g.
  ```crablang
  fn len(
      #[cfg(windows)] slice: &[u16],
      #[cfg(not(windows))] slice: &[u8],
  ) -> usize {
      slice.len()
  }
  ```
- [You can now take shared references to bind-by-move patterns in the `if` guards
  of `match` arms.][63118] e.g.
  ```crablang
  fn main() {
      let array: Box<[u8; 4]> = Box::new([1, 2, 3, 4]);

      match array {
          nums
  //      ---- `nums` is bound by move.
              if nums.iter().sum::<u8>() == 10
  //                 ^------ `.iter()` implicitly takes a reference to `nums`.
          => {
              drop(nums);
  //          ----------- Legal as `nums` was bound by move and so we have ownership.
          }
          _ => unreachable!(),
      }
  }
  ```



Compiler
--------
- [Added tier 3\* support for the `i686-unknown-uefi` target.][64334]
- [Added tier 3 support for the `sparc64-unknown-openbsd` target.][63595]
- [crablangc will now trim code snippets in diagnostics to fit in your terminal.][63402]
  **Note** Cargo currently doesn't use this feature. Refer to
  [cargo#7315][cargo/7315] to track this feature's progress.
- [You can now pass `--show-output` argument to test binaries to print the
  output of successful tests.][62600]


\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
---------
- [`Vec::new` and `String::new` are now `const` functions.][64028]
- [`LinkedList::new` is now a `const` function.][63684]
- [`str::len`, `[T]::len` and `str::as_bytes` are now `const` functions.][63770]
- [The `abs`, `wrapping_abs`, and `overflowing_abs` numeric functions are
  now `const`.][63786]

Stabilized APIs
---------------
- [`Pin::into_inner`]
- [`Instant::checked_duration_since`]
- [`Instant::saturating_duration_since`]

Cargo
-----
- [You can now publish git dependencies if supplied with a `version`.][cargo/7237]
- [The `--all` flag has been renamed to `--workspace`.][cargo/7241] Using
  `--all` is now deprecated.

Misc
----
- [You can now pass `-Clinker` to crablangdoc to control the linker used
  for compiling doctests.][63834]

Compatibility Notes
-------------------
- [Code that was previously accepted by the old borrow checker, but rejected by
  the NLL borrow checker is now a hard error in CrabLang 2018.][63565] This was
  previously a warning, and will also become a hard error in the CrabLang 2015
  edition in the 1.40.0 release.
- [`crablangdoc` now requires `crablangc` to be installed and in the same directory to
  run tests.][63827] This should improve performance when running a large
  amount of doctests.
- [The `try!` macro will now issue a deprecation warning.][62672] It is
  recommended to use the `?` operator instead.
- [`asinh(-0.0)` now correctly returns `-0.0`.][63698] Previously this
  returned `0.0`.

[62600]: https://github.com/crablang/crablang/pull/62600/
[62672]: https://github.com/crablang/crablang/pull/62672/
[63118]: https://github.com/crablang/crablang/pull/63118/
[63209]: https://github.com/crablang/crablang/pull/63209/
[63402]: https://github.com/crablang/crablang/pull/63402/
[63565]: https://github.com/crablang/crablang/pull/63565/
[63595]: https://github.com/crablang/crablang/pull/63595/
[63684]: https://github.com/crablang/crablang/pull/63684/
[63698]: https://github.com/crablang/crablang/pull/63698/
[63770]: https://github.com/crablang/crablang/pull/63770/
[63786]: https://github.com/crablang/crablang/pull/63786/
[63827]: https://github.com/crablang/crablang/pull/63827/
[63834]: https://github.com/crablang/crablang/pull/63834/
[64010]: https://github.com/crablang/crablang/pull/64010/
[64028]: https://github.com/crablang/crablang/pull/64028/
[64334]: https://github.com/crablang/crablang/pull/64334/
[cargo/7237]: https://github.com/crablang/cargo/pull/7237/
[cargo/7241]: https://github.com/crablang/cargo/pull/7241/
[cargo/7315]: https://github.com/crablang/cargo/pull/7315/
[`Pin::into_inner`]: https://doc.crablang.org/std/pin/struct.Pin.html#method.into_inner
[`Instant::checked_duration_since`]: https://doc.crablang.org/std/time/struct.Instant.html#method.checked_duration_since
[`Instant::saturating_duration_since`]: https://doc.crablang.org/std/time/struct.Instant.html#method.saturating_duration_since

Version 1.38.0 (2019-09-26)
==========================

Language
--------
- [The `#[global_allocator]` attribute can now be used in submodules.][62735]
- [The `#[deprecated]` attribute can now be used on macros.][62042]

Compiler
--------
- [Added pipelined compilation support to `crablangc`.][62766] This will
  improve compilation times in some cases. For further information please refer
  to the [_"Evaluating pipelined crablangc compilation"_][pipeline-internals] thread.
- [Added tier 3\* support for the `aarch64-uwp-windows-msvc`, `i686-uwp-windows-gnu`,
  `i686-uwp-windows-msvc`, `x86_64-uwp-windows-gnu`, and
  `x86_64-uwp-windows-msvc` targets.][60260]
- [Added tier 3 support for the `armv7-unknown-linux-gnueabi` and
  `armv7-unknown-linux-musleabi` targets.][63107]
- [Added tier 3 support for the `hexagon-unknown-linux-musl` target.][62814]
- [Added tier 3 support for the `riscv32i-unknown-none-elf` target.][62784]
- [Upgraded to LLVM 9.][62592]

\* Refer to CrabLang's [platform support page][platform-support-doc] for more
information on CrabLang's tiered platform support.

Libraries
---------
- [`ascii::EscapeDefault` now implements `Clone` and `Display`.][63421]
- [Derive macros for prelude traits (e.g. `Clone`, `Debug`, `Hash`) are now
  available at the same path as the trait.][63056] (e.g. The `Clone` derive macro
  is available at `std::clone::Clone`). This also makes all built-in macros
  available in `std`/`core` root. e.g. `std::include_bytes!`.
- [`str::Chars` now implements `Debug`.][63000]
- [`slice::{concat, connect, join}` now accepts `&[T]` in addition to `&T`.][62528]
- [`*const T` and `*mut T` now implement `marker::Unpin`.][62583]
- [`Arc<[T]>` and `Rc<[T]>` now implement `FromIterator<T>`.][61953]
- [Added euclidean remainder and division operations (`div_euclid`,
  `rem_euclid`) to all numeric primitives.][61884] Additionally `checked`,
  `overflowing`, and `wrapping` versions are available for all
  integer primitives.
- [`thread::AccessError` now implements `Clone`, `Copy`, `Eq`, `Error`, and
  `PartialEq`.][61491]
- [`iter::{StepBy, Peekable, Take}` now implement `DoubleEndedIterator`.][61457]

Stabilized APIs
---------------
- [`<*const T>::cast`]
- [`<*mut T>::cast`]
- [`Duration::as_secs_f32`]
- [`Duration::as_secs_f64`]
- [`Duration::div_f32`]
- [`Duration::div_f64`]
- [`Duration::from_secs_f32`]
- [`Duration::from_secs_f64`]
- [`Duration::mul_f32`]
- [`Duration::mul_f64`]
- [`any::type_name`]

Cargo
-----
- [Added pipelined compilation support to `cargo`.][cargo/7143]
- [You can now pass the `--features` option multiple times to enable
  multiple features.][cargo/7084]

CrabLangdoc
-------

- [Documentation on `pub use` statements is prepended to the documentation of the re-exported item][63048]

Misc
----
- [`crablangc` will now warn about some incorrect uses of
  `mem::{uninitialized, zeroed}` that are known to cause undefined behaviour.][63346]

Compatibility Notes
-------------------
- The [`x86_64-unknown-uefi` platform can not be built][62785] with crablangc
  1.38.0.
- The [`armv7-unknown-linux-gnueabihf` platform is known to have
  issues][62896] with certain crates such as libc.

[60260]: https://github.com/crablang/crablang/pull/60260/
[61457]: https://github.com/crablang/crablang/pull/61457/
[61491]: https://github.com/crablang/crablang/pull/61491/
[61884]: https://github.com/crablang/crablang/pull/61884/
[61953]: https://github.com/crablang/crablang/pull/61953/
[62042]: https://github.com/crablang/crablang/pull/62042/
[62528]: https://github.com/crablang/crablang/pull/62528/
[62583]: https://github.com/crablang/crablang/pull/62583/
[62735]: https://github.com/crablang/crablang/pull/62735/
[62766]: https://github.com/crablang/crablang/pull/62766/
[62784]: https://github.com/crablang/crablang/pull/62784/
[62592]: https://github.com/crablang/crablang/pull/62592/
[62785]: https://github.com/crablang/crablang/issues/62785/
[62814]: https://github.com/crablang/crablang/pull/62814/
[62896]: https://github.com/crablang/crablang/issues/62896/
[63000]: https://github.com/crablang/crablang/pull/63000/
[63056]: https://github.com/crablang/crablang/pull/63056/
[63107]: https://github.com/crablang/crablang/pull/63107/
[63346]: https://github.com/crablang/crablang/pull/63346/
[63421]: https://github.com/crablang/crablang/pull/63421/
[cargo/7084]: https://github.com/crablang/cargo/pull/7084/
[cargo/7143]: https://github.com/crablang/cargo/pull/7143/
[63048]: https://github.com/crablang/crablang/pull/63048
[`<*const T>::cast`]: https://doc.crablang.org/std/primitive.pointer.html#method.cast
[`<*mut T>::cast`]: https://doc.crablang.org/std/primitive.pointer.html#method.cast
[`Duration::as_secs_f32`]: https://doc.crablang.org/std/time/struct.Duration.html#method.as_secs_f32
[`Duration::as_secs_f64`]: https://doc.crablang.org/std/time/struct.Duration.html#method.as_secs_f64
[`Duration::div_f32`]: https://doc.crablang.org/std/time/struct.Duration.html#method.div_f32
[`Duration::div_f64`]: https://doc.crablang.org/std/time/struct.Duration.html#method.div_f64
[`Duration::from_secs_f32`]: https://doc.crablang.org/std/time/struct.Duration.html#method.from_secs_f32
[`Duration::from_secs_f64`]: https://doc.crablang.org/std/time/struct.Duration.html#method.from_secs_f64
[`Duration::mul_f32`]: https://doc.crablang.org/std/time/struct.Duration.html#method.mul_f32
[`Duration::mul_f64`]: https://doc.crablang.org/std/time/struct.Duration.html#method.mul_f64
[`any::type_name`]: https://doc.crablang.org/std/any/fn.type_name.html
[platform-support-doc]: https://doc.crablang.org/nightly/crablangc/platform-support.html
[pipeline-internals]: https://internals.crablang.org/t/evaluating-pipelined-crablangc-compilation/10199

Version 1.37.0 (2019-08-15)
==========================

Language
--------
- `#[must_use]` will now warn if the type is contained in a [tuple][61100],
  [`Box`][62228], or an [array][62235] and unused.
- [You can now use the `cfg` and `cfg_attr` attributes on
  generic parameters.][61547]
- [You can now use enum variants through type alias.][61682] e.g. You can
  write the following:
  ```crablang
  type MyOption = Option<u8>;

  fn increment_or_zero(x: MyOption) -> u8 {
      match x {
          MyOption::Some(y) => y + 1,
          MyOption::None => 0,
      }
  }
  ```
- [You can now use `_` as an identifier for consts.][61347] e.g. You can write
  `const _: u32 = 5;`.
- [You can now use `#[repr(align(X)]` on enums.][61229]
- [The  `?` Kleene macro operator is now available in the
  2015 edition.][60932]

Compiler
--------
- [You can now enable Profile-Guided Optimization with the `-C profile-generate`
  and `-C profile-use` flags.][61268] For more information on how to use profile
  guided optimization, please refer to the [crablangc book][crablangc-book-pgo].
- [The `crablang-lldb` wrapper script should now work again.][61827]

Libraries
---------
- [`mem::MaybeUninit<T>` is now ABI-compatible with `T`.][61802]

Stabilized APIs
---------------
- [`BufReader::buffer`]
- [`BufWriter::buffer`]
- [`Cell::from_mut`]
- [`Cell<[T]>::as_slice_of_cells`][`Cell<slice>::as_slice_of_cells`]
- [`DoubleEndedIterator::nth_back`]
- [`Option::xor`]
- [`Wrapping::reverse_bits`]
- [`i128::reverse_bits`]
- [`i16::reverse_bits`]
- [`i32::reverse_bits`]
- [`i64::reverse_bits`]
- [`i8::reverse_bits`]
- [`isize::reverse_bits`]
- [`slice::copy_within`]
- [`u128::reverse_bits`]
- [`u16::reverse_bits`]
- [`u32::reverse_bits`]
- [`u64::reverse_bits`]
- [`u8::reverse_bits`]
- [`usize::reverse_bits`]

Cargo
-----
- [`Cargo.lock` files are now included by default when publishing executable crates
  with executables.][cargo/7026]
- [You can now specify `default-run="foo"` in `[package]` to specify the
  default executable to use for `cargo run`.][cargo/7056]

Misc
----

Compatibility Notes
-------------------
- [Using `...` for inclusive range patterns will now warn by default.][61342]
  Please transition your code to using the `..=` syntax for inclusive
  ranges instead.
- [Using a trait object without the `dyn` will now warn by default.][61203]
  Please transition your code to use `dyn Trait` for trait objects instead.

[62228]: https://github.com/crablang/crablang/pull/62228/
[62235]: https://github.com/crablang/crablang/pull/62235/
[61802]: https://github.com/crablang/crablang/pull/61802/
[61827]: https://github.com/crablang/crablang/pull/61827/
[61547]: https://github.com/crablang/crablang/pull/61547/
[61682]: https://github.com/crablang/crablang/pull/61682/
[61268]: https://github.com/crablang/crablang/pull/61268/
[61342]: https://github.com/crablang/crablang/pull/61342/
[61347]: https://github.com/crablang/crablang/pull/61347/
[61100]: https://github.com/crablang/crablang/pull/61100/
[61203]: https://github.com/crablang/crablang/pull/61203/
[61229]: https://github.com/crablang/crablang/pull/61229/
[60932]: https://github.com/crablang/crablang/pull/60932/
[cargo/7026]: https://github.com/crablang/cargo/pull/7026/
[cargo/7056]: https://github.com/crablang/cargo/pull/7056/
[`BufReader::buffer`]: https://doc.crablang.org/std/io/struct.BufReader.html#method.buffer
[`BufWriter::buffer`]: https://doc.crablang.org/std/io/struct.BufWriter.html#method.buffer
[`Cell::from_mut`]: https://doc.crablang.org/std/cell/struct.Cell.html#method.from_mut
[`Cell<slice>::as_slice_of_cells`]: https://doc.crablang.org/std/cell/struct.Cell.html#method.as_slice_of_cells
[`DoubleEndedIterator::nth_back`]: https://doc.crablang.org/std/iter/trait.DoubleEndedIterator.html#method.nth_back
[`Option::xor`]: https://doc.crablang.org/std/option/enum.Option.html#method.xor
[`Wrapping::reverse_bits`]: https://doc.crablang.org/std/num/struct.Wrapping.html#method.reverse_bits
[`i128::reverse_bits`]: https://doc.crablang.org/std/primitive.i128.html#method.reverse_bits
[`i16::reverse_bits`]: https://doc.crablang.org/std/primitive.i16.html#method.reverse_bits
[`i32::reverse_bits`]: https://doc.crablang.org/std/primitive.i32.html#method.reverse_bits
[`i64::reverse_bits`]: https://doc.crablang.org/std/primitive.i64.html#method.reverse_bits
[`i8::reverse_bits`]: https://doc.crablang.org/std/primitive.i8.html#method.reverse_bits
[`isize::reverse_bits`]: https://doc.crablang.org/std/primitive.isize.html#method.reverse_bits
[`slice::copy_within`]: https://doc.crablang.org/std/primitive.slice.html#method.copy_within
[`u128::reverse_bits`]: https://doc.crablang.org/std/primitive.u128.html#method.reverse_bits
[`u16::reverse_bits`]: https://doc.crablang.org/std/primitive.u16.html#method.reverse_bits
[`u32::reverse_bits`]: https://doc.crablang.org/std/primitive.u32.html#method.reverse_bits
[`u64::reverse_bits`]: https://doc.crablang.org/std/primitive.u64.html#method.reverse_bits
[`u8::reverse_bits`]: https://doc.crablang.org/std/primitive.u8.html#method.reverse_bits
[`usize::reverse_bits`]: https://doc.crablang.org/std/primitive.usize.html#method.reverse_bits
[crablangc-book-pgo]: https://doc.crablang.org/crablangc/profile-guided-optimization.html


Version 1.36.0 (2019-07-04)
==========================

Language
--------
- [Non-Lexical Lifetimes are now enabled on the 2015 edition.][59114]
- [The order of traits in trait objects no longer affects the semantics of that
  object.][59445] e.g. `dyn Send + fmt::Debug` is now equivalent to
  `dyn fmt::Debug + Send`, where this was previously not the case.

Libraries
---------
- [`HashMap`'s implementation has been replaced with `hashbrown::HashMap` implementation.][58623]
- [`TryFromSliceError` now implements `From<Infallible>`.][60318]
- [`mem::needs_drop` is now available as a const fn.][60364]
- [`alloc::Layout::from_size_align_unchecked` is now available as a const fn.][60370]
- [`String` now implements `BorrowMut<str>`.][60404]
- [`io::Cursor` now implements `Default`.][60234]
- [Both `NonNull::{dangling, cast}` are now const fns.][60244]
- [The `alloc` crate is now stable.][59675] `alloc` allows you to use a subset
  of `std` (e.g. `Vec`, `Box`, `Arc`) in `#![no_std]` environments if the
  environment has access to heap memory allocation.
- [`String` now implements `From<&String>`.][59825]
- [You can now pass multiple arguments to the `dbg!` macro.][59826] `dbg!` will
  return a tuple of each argument when there is multiple arguments.
- [`Result::{is_err, is_ok}` are now `#[must_use]` and will produce a warning if
  not used.][59648]

Stabilized APIs
---------------
- [`VecDeque::rotate_left`]
- [`VecDeque::rotate_right`]
- [`Iterator::copied`]
- [`io::IoSlice`]
- [`io::IoSliceMut`]
- [`Read::read_vectored`]
- [`Write::write_vectored`]
- [`str::as_mut_ptr`]
- [`mem::MaybeUninit`]
- [`pointer::align_offset`]
- [`future::Future`]
- [`task::Context`]
- [`task::RawWaker`]
- [`task::RawWakerVTable`]
- [`task::Waker`]
- [`task::Poll`]

Cargo
-----
- [Cargo will now produce an error if you attempt to use the name of a required dependency as a feature.][cargo/6860]
- [You can now pass the `--offline` flag to run cargo without accessing the network.][cargo/6934]

You can find further change's in [Cargo's 1.36.0 release notes][cargo-1-36-0].

Clippy
------
There have been numerous additions and fixes to clippy, see [Clippy's 1.36.0 release notes][clippy-1-36-0] for more details.

Misc
----

Compatibility Notes
-------------------
- With the stabilisation of `mem::MaybeUninit`, `mem::uninitialized` use is no
  longer recommended, and will be deprecated in 1.39.0.

[60318]: https://github.com/crablang/crablang/pull/60318/
[60364]: https://github.com/crablang/crablang/pull/60364/
[60370]: https://github.com/crablang/crablang/pull/60370/
[60404]: https://github.com/crablang/crablang/pull/60404/
[60234]: https://github.com/crablang/crablang/pull/60234/
[60244]: https://github.com/crablang/crablang/pull/60244/
[58623]: https://github.com/crablang/crablang/pull/58623/
[59648]: https://github.com/crablang/crablang/pull/59648/
[59675]: https://github.com/crablang/crablang/pull/59675/
[59825]: https://github.com/crablang/crablang/pull/59825/
[59826]: https://github.com/crablang/crablang/pull/59826/
[59445]: https://github.com/crablang/crablang/pull/59445/
[59114]: https://github.com/crablang/crablang/pull/59114/
[cargo/6860]: https://github.com/crablang/cargo/pull/6860/
[cargo/6934]: https://github.com/crablang/cargo/pull/6934/
[`VecDeque::rotate_left`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.rotate_left
[`VecDeque::rotate_right`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.rotate_right
[`Iterator::copied`]: https://doc.crablang.org/std/iter/trait.Iterator.html#tymethod.copied
[`io::IoSlice`]: https://doc.crablang.org/std/io/struct.IoSlice.html
[`io::IoSliceMut`]: https://doc.crablang.org/std/io/struct.IoSliceMut.html
[`Read::read_vectored`]: https://doc.crablang.org/std/io/trait.Read.html#method.read_vectored
[`Write::write_vectored`]: https://doc.crablang.org/std/io/trait.Write.html#method.write_vectored
[`str::as_mut_ptr`]: https://doc.crablang.org/std/primitive.str.html#method.as_mut_ptr
[`mem::MaybeUninit`]: https://doc.crablang.org/std/mem/union.MaybeUninit.html
[`pointer::align_offset`]: https://doc.crablang.org/std/primitive.pointer.html#method.align_offset
[`future::Future`]: https://doc.crablang.org/std/future/trait.Future.html
[`task::Context`]: https://doc.crablang.org/beta/std/task/struct.Context.html
[`task::RawWaker`]: https://doc.crablang.org/beta/std/task/struct.RawWaker.html
[`task::RawWakerVTable`]: https://doc.crablang.org/beta/std/task/struct.RawWakerVTable.html
[`task::Waker`]: https://doc.crablang.org/beta/std/task/struct.Waker.html
[`task::Poll`]: https://doc.crablang.org/beta/std/task/enum.Poll.html
[clippy-1-36-0]: https://github.com/crablang/crablang-clippy/blob/master/CHANGELOG.md#crablang-136
[cargo-1-36-0]: https://github.com/crablang/cargo/blob/master/CHANGELOG.md#cargo-136-2019-07-04


Version 1.35.0 (2019-05-23)
==========================

Language
--------
- [`FnOnce`, `FnMut`, and the `Fn` traits are now implemented for `Box<FnOnce>`,
  `Box<FnMut>`, and `Box<Fn>` respectively.][59500]
- [You can now coerce closures into unsafe function pointers.][59580] e.g.
  ```crablang
  unsafe fn call_unsafe(func: unsafe fn()) {
      func()
  }

  pub fn main() {
      unsafe { call_unsafe(|| {}); }
  }
  ```


Compiler
--------
- [Added the `armv6-unknown-freebsd-gnueabihf` and
  `armv7-unknown-freebsd-gnueabihf` targets.][58080]
- [Added the `wasm32-unknown-wasi` target.][59464]


Libraries
---------
- [`Thread` will now show its ID in `Debug` output.][59460]
- [`StdinLock`, `StdoutLock`, and `StderrLock` now implement `AsRawFd`.][59512]
- [`alloc::System` now implements `Default`.][59451]
- [Expanded `Debug` output (`{:#?}`) for structs now has a trailing comma on the
  last field.][59076]
- [`char::{ToLowercase, ToUppercase}` now
  implement `ExactSizeIterator`.][58778]
- [All `NonZero` numeric types now implement `FromStr`.][58717]
- [Removed the `Read` trait bounds
  on the `BufReader::{get_ref, get_mut, into_inner}` methods.][58423]
- [You can now call the `dbg!` macro without any parameters to print the file
  and line where it is called.][57847]
- [In place ASCII case conversions are now up to 4× faster.][59283]
  e.g. `str::make_ascii_lowercase`
- [`hash_map::{OccupiedEntry, VacantEntry}` now implement `Sync`
  and `Send`.][58369]

Stabilized APIs
---------------
- [`f32::copysign`]
- [`f64::copysign`]
- [`RefCell::replace_with`]
- [`RefCell::map_split`]
- [`ptr::hash`]
- [`Range::contains`]
- [`RangeFrom::contains`]
- [`RangeTo::contains`]
- [`RangeInclusive::contains`]
- [`RangeToInclusive::contains`]
- [`Option::copied`]

Cargo
-----
- [You can now set `cargo:crablangc-cdylib-link-arg` at build time to pass custom
  linker arguments when building a `cdylib`.][cargo/6298] Its usage is highly
  platform specific.

Misc
----
- [The CrabLang toolchain is now available natively for musl based distros.][58575]

[59460]: https://github.com/crablang/crablang/pull/59460/
[59464]: https://github.com/crablang/crablang/pull/59464/
[59500]: https://github.com/crablang/crablang/pull/59500/
[59512]: https://github.com/crablang/crablang/pull/59512/
[59580]: https://github.com/crablang/crablang/pull/59580/
[59283]: https://github.com/crablang/crablang/pull/59283/
[59451]: https://github.com/crablang/crablang/pull/59451/
[59076]: https://github.com/crablang/crablang/pull/59076/
[58778]: https://github.com/crablang/crablang/pull/58778/
[58717]: https://github.com/crablang/crablang/pull/58717/
[58369]: https://github.com/crablang/crablang/pull/58369/
[58423]: https://github.com/crablang/crablang/pull/58423/
[58080]: https://github.com/crablang/crablang/pull/58080/
[57847]: https://github.com/crablang/crablang/pull/57847/
[58575]: https://github.com/crablang/crablang/pull/58575
[cargo/6298]: https://github.com/crablang/cargo/pull/6298/
[`f32::copysign`]: https://doc.crablang.org/stable/std/primitive.f32.html#method.copysign
[`f64::copysign`]: https://doc.crablang.org/stable/std/primitive.f64.html#method.copysign
[`RefCell::replace_with`]: https://doc.crablang.org/stable/std/cell/struct.RefCell.html#method.replace_with
[`RefCell::map_split`]: https://doc.crablang.org/stable/std/cell/struct.RefCell.html#method.map_split
[`ptr::hash`]: https://doc.crablang.org/stable/std/ptr/fn.hash.html
[`Range::contains`]: https://doc.crablang.org/std/ops/struct.Range.html#method.contains
[`RangeFrom::contains`]: https://doc.crablang.org/std/ops/struct.RangeFrom.html#method.contains
[`RangeTo::contains`]: https://doc.crablang.org/std/ops/struct.RangeTo.html#method.contains
[`RangeInclusive::contains`]: https://doc.crablang.org/std/ops/struct.RangeInclusive.html#method.contains
[`RangeToInclusive::contains`]: https://doc.crablang.org/std/ops/struct.RangeToInclusive.html#method.contains
[`Option::copied`]: https://doc.crablang.org/std/option/enum.Option.html#method.copied

Version 1.34.2 (2019-05-14)
===========================

* [Destabilize the `Error::type_id` function due to a security
   vulnerability][60785] ([CVE-2019-12083])

[60785]: https://github.com/crablang/crablang/pull/60785
[CVE-2019-12083]: https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2019-12083

Version 1.34.1 (2019-04-25)
===========================

* [Fix false positives for the `redundant_closure` Clippy lint][clippy/3821]
* [Fix false positives for the `missing_const_for_fn` Clippy lint][clippy/3844]
* [Fix Clippy panic when checking some macros][clippy/3805]

[clippy/3821]: https://github.com/crablang/crablang-clippy/pull/3821
[clippy/3844]: https://github.com/crablang/crablang-clippy/pull/3844
[clippy/3805]: https://github.com/crablang/crablang-clippy/pull/3805

Version 1.34.0 (2019-04-11)
==========================

Language
--------
- [You can now use `#[deprecated = "reason"]`][58166] as a shorthand for
  `#[deprecated(note = "reason")]`. This was previously allowed by mistake
  but had no effect.
- [You can now accept token streams in `#[attr()]`,`#[attr[]]`, and
  `#[attr{}]` procedural macros.][57367]
- [You can now write `extern crate self as foo;`][57407] to import your
  crate's root into the extern prelude.


Compiler
--------
- [You can now target `riscv64imac-unknown-none-elf` and
  `riscv64gc-unknown-none-elf`.][58406]
- [You can now enable linker plugin LTO optimisations with
  `-C linker-plugin-lto`.][58057] This allows crablangc to compile your CrabLang code
  into LLVM bitcode allowing LLVM to perform LTO optimisations across C/C++ FFI
  boundaries.
- [You can now target `powerpc64-unknown-freebsd`.][57809]


Libraries
---------
- [The trait bounds have been removed on some of `HashMap<K, V, S>`'s and
  `HashSet<T, S>`'s basic methods.][58370] Most notably you no longer require
  the `Hash` trait to create an iterator.
- [The `Ord` trait bounds have been removed on some of `BinaryHeap<T>`'s basic
  methods.][58421] Most notably you no longer require the `Ord` trait to create
  an iterator.
- [The methods `overflowing_neg` and `wrapping_neg` are now `const` functions
  for all numeric types.][58044]
- [Indexing a `str` is now generic over all types that
  implement `SliceIndex<str>`.][57604]
- [`str::trim`, `str::trim_matches`, `str::trim_{start, end}`, and
  `str::trim_{start, end}_matches` are now `#[must_use]`][57106] and will
  produce a warning if their returning type is unused.
- [The methods `checked_pow`, `saturating_pow`, `wrapping_pow`, and
  `overflowing_pow` are now available for all numeric types.][57873] These are
  equivalent to methods such as `wrapping_add` for the `pow` operation.


Stabilized APIs
---------------

#### std & core
* [`Any::type_id`]
* [`Error::type_id`]
* [`atomic::AtomicI16`]
* [`atomic::AtomicI32`]
* [`atomic::AtomicI64`]
* [`atomic::AtomicI8`]
* [`atomic::AtomicU16`]
* [`atomic::AtomicU32`]
* [`atomic::AtomicU64`]
* [`atomic::AtomicU8`]
* [`convert::Infallible`]
* [`convert::TryFrom`]
* [`convert::TryInto`]
* [`iter::from_fn`]
* [`iter::successors`]
* [`num::NonZeroI128`]
* [`num::NonZeroI16`]
* [`num::NonZeroI32`]
* [`num::NonZeroI64`]
* [`num::NonZeroI8`]
* [`num::NonZeroIsize`]
* [`slice::sort_by_cached_key`]
* [`str::escape_debug`]
* [`str::escape_default`]
* [`str::escape_unicode`]
* [`str::split_ascii_whitespace`]

#### std
* [`Instant::checked_add`]
* [`Instant::checked_sub`]
* [`SystemTime::checked_add`]
* [`SystemTime::checked_sub`]

Cargo
-----
- [You can now use alternative registries to crates.io.][cargo/6654]

Misc
----
- [You can now use the `?` operator in your documentation tests without manually
  adding `fn main() -> Result<(), _> {}`.][56470]

Compatibility Notes
-------------------
- [`Command::before_exec` is being replaced by the unsafe method
  `Command::pre_exec`][58059] and will be deprecated with CrabLang 1.37.0.
- [Use of `ATOMIC_{BOOL, ISIZE, USIZE}_INIT` is now deprecated][57425] as you
  can now use `const` functions in `static` variables.

[58370]: https://github.com/crablang/crablang/pull/58370/
[58406]: https://github.com/crablang/crablang/pull/58406/
[58421]: https://github.com/crablang/crablang/pull/58421/
[58166]: https://github.com/crablang/crablang/pull/58166/
[58044]: https://github.com/crablang/crablang/pull/58044/
[58057]: https://github.com/crablang/crablang/pull/58057/
[58059]: https://github.com/crablang/crablang/pull/58059/
[57809]: https://github.com/crablang/crablang/pull/57809/
[57873]: https://github.com/crablang/crablang/pull/57873/
[57604]: https://github.com/crablang/crablang/pull/57604/
[57367]: https://github.com/crablang/crablang/pull/57367/
[57407]: https://github.com/crablang/crablang/pull/57407/
[57425]: https://github.com/crablang/crablang/pull/57425/
[57106]: https://github.com/crablang/crablang/pull/57106/
[56470]: https://github.com/crablang/crablang/pull/56470/
[cargo/6654]: https://github.com/crablang/cargo/pull/6654/
[`Any::type_id`]: https://doc.crablang.org/std/any/trait.Any.html#tymethod.type_id
[`Error::type_id`]: https://doc.crablang.org/std/error/trait.Error.html#method.type_id
[`atomic::AtomicI16`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicI16.html
[`atomic::AtomicI32`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicI32.html
[`atomic::AtomicI64`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicI64.html
[`atomic::AtomicI8`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicI8.html
[`atomic::AtomicU16`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicU16.html
[`atomic::AtomicU32`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicU32.html
[`atomic::AtomicU64`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicU64.html
[`atomic::AtomicU8`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicU8.html
[`convert::Infallible`]: https://doc.crablang.org/std/convert/enum.Infallible.html
[`convert::TryFrom`]: https://doc.crablang.org/std/convert/trait.TryFrom.html
[`convert::TryInto`]: https://doc.crablang.org/std/convert/trait.TryInto.html
[`iter::from_fn`]: https://doc.crablang.org/std/iter/fn.from_fn.html
[`iter::successors`]: https://doc.crablang.org/std/iter/fn.successors.html
[`num::NonZeroI128`]: https://doc.crablang.org/std/num/struct.NonZeroI128.html
[`num::NonZeroI16`]: https://doc.crablang.org/std/num/struct.NonZeroI16.html
[`num::NonZeroI32`]: https://doc.crablang.org/std/num/struct.NonZeroI32.html
[`num::NonZeroI64`]: https://doc.crablang.org/std/num/struct.NonZeroI64.html
[`num::NonZeroI8`]: https://doc.crablang.org/std/num/struct.NonZeroI8.html
[`num::NonZeroIsize`]: https://doc.crablang.org/std/num/struct.NonZeroIsize.html
[`slice::sort_by_cached_key`]: https://doc.crablang.org/std/primitive.slice.html#method.sort_by_cached_key
[`str::escape_debug`]: https://doc.crablang.org/std/primitive.str.html#method.escape_debug
[`str::escape_default`]: https://doc.crablang.org/std/primitive.str.html#method.escape_default
[`str::escape_unicode`]: https://doc.crablang.org/std/primitive.str.html#method.escape_unicode
[`str::split_ascii_whitespace`]: https://doc.crablang.org/std/primitive.str.html#method.split_ascii_whitespace
[`Instant::checked_add`]: https://doc.crablang.org/std/time/struct.Instant.html#method.checked_add
[`Instant::checked_sub`]: https://doc.crablang.org/std/time/struct.Instant.html#method.checked_sub
[`SystemTime::checked_add`]: https://doc.crablang.org/std/time/struct.SystemTime.html#method.checked_add
[`SystemTime::checked_sub`]: https://doc.crablang.org/std/time/struct.SystemTime.html#method.checked_sub


Version 1.33.0 (2019-02-28)
==========================

Language
--------
- [You can now use the `cfg(target_vendor)` attribute.][57465] E.g.
  `#[cfg(target_vendor="apple")] fn main() { println!("Hello Apple!"); }`
- [Integer patterns such as in a match expression can now be exhaustive.][56362]
  E.g. You can have match statement on a `u8` that covers `0..=255` and
  you would no longer be required to have a `_ => unreachable!()` case.
- [You can now have multiple patterns in `if let` and `while let`
  expressions.][57532] You can do this with the same syntax as a `match`
  expression. E.g.
  ```crablang
  enum Creature {
      Crab(String),
      Lobster(String),
      Person(String),
  }

  fn main() {
      let state = Creature::Crab("Ferris");

      if let Creature::Crab(name) | Creature::Person(name) = state {
          println!("This creature's name is: {}", name);
      }
  }
  ```
- [You can now have irrefutable `if let` and `while let` patterns.][57535] Using
  this feature will by default produce a warning as this behaviour can be
  unintuitive. E.g. `if let _ = 5 {}`
- [You can now use `let` bindings, assignments, expression statements,
  and irrefutable pattern destructuring in const functions.][57175]
- [You can now call unsafe const functions.][57067] E.g.
  ```crablang
  const unsafe fn foo() -> i32 { 5 }
  const fn bar() -> i32 {
      unsafe { foo() }
  }
  ```
- [You can now specify multiple attributes in a `cfg_attr` attribute.][57332]
  E.g. `#[cfg_attr(all(), must_use, optimize)]`
- [You can now specify a specific alignment with the `#[repr(packed)]`
  attribute.][57049] E.g. `#[repr(packed(2))] struct Foo(i16, i32);` is a struct
  with an alignment of 2 bytes and a size of 6 bytes.
- [You can now import an item from a module as an `_`.][56303] This allows you to
  import a trait's impls, and not have the name in the namespace. E.g.
  ```crablang
  use std::io::Read as _;

  // Allowed as there is only one `Read` in the module.
  pub trait Read {}
  ```
- [You may now use `Rc`, `Arc`, and `Pin` as method receivers][56805].

Compiler
--------
- [You can now set a linker flavor for `crablangc` with the `-Clinker-flavor`
  command line argument.][56351]
- [The minimum required LLVM version has been bumped to 6.0.][56642]
- [Added support for the PowerPC64 architecture on FreeBSD.][57615]
- [The `x86_64-fortanix-unknown-sgx` target support has been upgraded to
  tier 2 support.][57130] Visit the [platform support][platform-support] page for
  information on CrabLang's platform support.
- [Added support for the `thumbv7neon-linux-androideabi` and
  `thumbv7neon-unknown-linux-gnueabihf` targets.][56947]
- [Added support for the `x86_64-unknown-uefi` target.][56769]

Libraries
---------
- [The methods `overflowing_{add, sub, mul, shl, shr}` are now `const`
  functions for all numeric types.][57566]
- [The methods `rotate_left`, `rotate_right`, and `wrapping_{add, sub, mul, shl, shr}`
  are now `const` functions for all numeric types.][57105]
- [The methods `is_positive` and `is_negative` are now `const` functions for
  all signed numeric types.][57105]
- [The `get` method for all `NonZero` types is now `const`.][57167]
- [The methods `count_ones`, `count_zeros`, `leading_zeros`, `trailing_zeros`,
  `swap_bytes`, `from_be`, `from_le`, `to_be`, `to_le` are now `const` for all
  numeric types.][57234]
- [`Ipv4Addr::new` is now a `const` function][57234]

Stabilized APIs
---------------
- [`unix::FileExt::read_exact_at`]
- [`unix::FileExt::write_all_at`]
- [`Option::transpose`]
- [`Result::transpose`]
- [`convert::identity`]
- [`pin::Pin`]
- [`marker::Unpin`]
- [`marker::PhantomPinned`]
- [`Vec::resize_with`]
- [`VecDeque::resize_with`]
- [`Duration::as_millis`]
- [`Duration::as_micros`]
- [`Duration::as_nanos`]


Cargo
-----
- [You can now publish crates that require a feature flag to compile with
  `cargo publish --features` or `cargo publish --all-features`.][cargo/6453]
- [Cargo should now rebuild a crate if a file was modified during the initial
  build.][cargo/6484]

Compatibility Notes
-------------------
- The methods `str::{trim_left, trim_right, trim_left_matches, trim_right_matches}`
  are now deprecated in the standard library, and their usage will now produce a warning.
  Please use the `str::{trim_start, trim_end, trim_start_matches, trim_end_matches}`
  methods instead.
- The `Error::cause` method has been deprecated in favor of `Error::source` which supports
  downcasting.
- [Libtest no longer creates a new thread for each test when
  `--test-threads=1`.  It also runs the tests in deterministic order][56243]

[56243]: https://github.com/crablang/crablang/pull/56243
[56303]: https://github.com/crablang/crablang/pull/56303/
[56351]: https://github.com/crablang/crablang/pull/56351/
[56362]: https://github.com/crablang/crablang/pull/56362
[56642]: https://github.com/crablang/crablang/pull/56642/
[56769]: https://github.com/crablang/crablang/pull/56769/
[56805]: https://github.com/crablang/crablang/pull/56805
[56947]: https://github.com/crablang/crablang/pull/56947/
[57049]: https://github.com/crablang/crablang/pull/57049/
[57067]: https://github.com/crablang/crablang/pull/57067/
[57105]: https://github.com/crablang/crablang/pull/57105
[57130]: https://github.com/crablang/crablang/pull/57130/
[57167]: https://github.com/crablang/crablang/pull/57167/
[57175]: https://github.com/crablang/crablang/pull/57175/
[57234]: https://github.com/crablang/crablang/pull/57234/
[57332]: https://github.com/crablang/crablang/pull/57332/
[57465]: https://github.com/crablang/crablang/pull/57465/
[57532]: https://github.com/crablang/crablang/pull/57532/
[57535]: https://github.com/crablang/crablang/pull/57535/
[57566]: https://github.com/crablang/crablang/pull/57566/
[57615]: https://github.com/crablang/crablang/pull/57615/
[cargo/6453]: https://github.com/crablang/cargo/pull/6453/
[cargo/6484]: https://github.com/crablang/cargo/pull/6484/
[`unix::FileExt::read_exact_at`]: https://doc.crablang.org/std/os/unix/fs/trait.FileExt.html#method.read_exact_at
[`unix::FileExt::write_all_at`]: https://doc.crablang.org/std/os/unix/fs/trait.FileExt.html#method.write_all_at
[`Option::transpose`]: https://doc.crablang.org/std/option/enum.Option.html#method.transpose
[`Result::transpose`]: https://doc.crablang.org/std/result/enum.Result.html#method.transpose
[`convert::identity`]: https://doc.crablang.org/std/convert/fn.identity.html
[`pin::Pin`]: https://doc.crablang.org/std/pin/struct.Pin.html
[`marker::Unpin`]: https://doc.crablang.org/stable/std/marker/trait.Unpin.html
[`marker::PhantomPinned`]: https://doc.crablang.org/nightly/std/marker/struct.PhantomPinned.html
[`Vec::resize_with`]: https://doc.crablang.org/std/vec/struct.Vec.html#method.resize_with
[`VecDeque::resize_with`]: https://doc.crablang.org/std/collections/struct.VecDeque.html#method.resize_with
[`Duration::as_millis`]: https://doc.crablang.org/std/time/struct.Duration.html#method.as_millis
[`Duration::as_micros`]: https://doc.crablang.org/std/time/struct.Duration.html#method.as_micros
[`Duration::as_nanos`]: https://doc.crablang.org/std/time/struct.Duration.html#method.as_nanos
[platform-support]: https://forge.crablang.org/platform-support.html

Version 1.32.0 (2019-01-17)
==========================

Language
--------
#### 2018 edition
- [You can now use the `?` operator in macro definitions.][56245] The `?`
  operator allows you to specify zero or one repetitions similar to the `*` and
  `+` operators.
- [Module paths with no leading keyword like `super`, `self`, or `crate`, will
  now always resolve to the item (`enum`, `struct`, etc.) available in the
  module if present, before resolving to a external crate or an item the prelude.][56759]
  E.g.
  ```crablang
  enum Color { Red, Green, Blue }

  use Color::*;
  ```

#### All editions
- [You can now match against `PhantomData<T>` types.][55837]
- [You can now match against literals in macros with the `literal`
  specifier.][56072] This will match against a literal of any type.
  E.g. `1`, `'A'`, `"Hello World"`
- [Self can now be used as a constructor and pattern for unit and tuple structs.][56365] E.g.
  ```crablang
  struct Point(i32, i32);

  impl Point {
      pub fn new(x: i32, y: i32) -> Self {
          Self(x, y)
      }

      pub fn is_origin(&self) -> bool {
          match self {
              Self(0, 0) => true,
              _ => false,
          }
      }
  }
  ```
- [Self can also now be used in type definitions.][56366] E.g.
  ```crablang
  enum List<T>
  where
      Self: PartialOrd<Self> // can write `Self` instead of `List<T>`
  {
      Nil,
      Cons(T, Box<Self>) // likewise here
  }
  ```
- [You can now mark traits with `#[must_use]`.][55663] This provides a warning if
  a `impl Trait` or `dyn Trait` is returned and unused in the program.

Compiler
--------
- [The default allocator has changed from jemalloc to the default allocator on
  your system.][55238] The compiler itself on Linux & macOS will still use
  jemalloc, but programs compiled with it will use the system allocator.
- [Added the `aarch64-pc-windows-msvc` target.][55702]

Libraries
---------
- [`PathBuf` now implements `FromStr`.][55148]
- [`Box<[T]>` now implements `FromIterator<T>`.][55843]
- [The `dbg!` macro has been stabilized.][56395] This macro enables you to
  easily debug expressions in your crablang program. E.g.
  ```crablang
  let a = 2;
  let b = dbg!(a * 2) + 1;
  //      ^-- prints: [src/main.rs:4] a * 2 = 4
  assert_eq!(b, 5);
  ```

The following APIs are now `const` functions and can be used in a
`const` context.

- [`Cell::as_ptr`]
- [`UnsafeCell::get`]
- [`char::is_ascii`]
- [`iter::empty`]
- [`ManuallyDrop::new`]
- [`ManuallyDrop::into_inner`]
- [`RangeInclusive::start`]
- [`RangeInclusive::end`]
- [`NonNull::as_ptr`]
- [`slice::as_ptr`]
- [`str::as_ptr`]
- [`Duration::as_secs`]
- [`Duration::subsec_millis`]
- [`Duration::subsec_micros`]
- [`Duration::subsec_nanos`]
- [`CStr::as_ptr`]
- [`Ipv4Addr::is_unspecified`]
- [`Ipv6Addr::new`]
- [`Ipv6Addr::octets`]

Stabilized APIs
---------------
- [`i8::to_be_bytes`]
- [`i8::to_le_bytes`]
- [`i8::to_ne_bytes`]
- [`i8::from_be_bytes`]
- [`i8::from_le_bytes`]
- [`i8::from_ne_bytes`]
- [`i16::to_be_bytes`]
- [`i16::to_le_bytes`]
- [`i16::to_ne_bytes`]
- [`i16::from_be_bytes`]
- [`i16::from_le_bytes`]
- [`i16::from_ne_bytes`]
- [`i32::to_be_bytes`]
- [`i32::to_le_bytes`]
- [`i32::to_ne_bytes`]
- [`i32::from_be_bytes`]
- [`i32::from_le_bytes`]
- [`i32::from_ne_bytes`]
- [`i64::to_be_bytes`]
- [`i64::to_le_bytes`]
- [`i64::to_ne_bytes`]
- [`i64::from_be_bytes`]
- [`i64::from_le_bytes`]
- [`i64::from_ne_bytes`]
- [`i128::to_be_bytes`]
- [`i128::to_le_bytes`]
- [`i128::to_ne_bytes`]
- [`i128::from_be_bytes`]
- [`i128::from_le_bytes`]
- [`i128::from_ne_bytes`]
- [`isize::to_be_bytes`]
- [`isize::to_le_bytes`]
- [`isize::to_ne_bytes`]
- [`isize::from_be_bytes`]
- [`isize::from_le_bytes`]
- [`isize::from_ne_bytes`]
- [`u8::to_be_bytes`]
- [`u8::to_le_bytes`]
- [`u8::to_ne_bytes`]
- [`u8::from_be_bytes`]
- [`u8::from_le_bytes`]
- [`u8::from_ne_bytes`]
- [`u16::to_be_bytes`]
- [`u16::to_le_bytes`]
- [`u16::to_ne_bytes`]
- [`u16::from_be_bytes`]
- [`u16::from_le_bytes`]
- [`u16::from_ne_bytes`]
- [`u32::to_be_bytes`]
- [`u32::to_le_bytes`]
- [`u32::to_ne_bytes`]
- [`u32::from_be_bytes`]
- [`u32::from_le_bytes`]
- [`u32::from_ne_bytes`]
- [`u64::to_be_bytes`]
- [`u64::to_le_bytes`]
- [`u64::to_ne_bytes`]
- [`u64::from_be_bytes`]
- [`u64::from_le_bytes`]
- [`u64::from_ne_bytes`]
- [`u128::to_be_bytes`]
- [`u128::to_le_bytes`]
- [`u128::to_ne_bytes`]
- [`u128::from_be_bytes`]
- [`u128::from_le_bytes`]
- [`u128::from_ne_bytes`]
- [`usize::to_be_bytes`]
- [`usize::to_le_bytes`]
- [`usize::to_ne_bytes`]
- [`usize::from_be_bytes`]
- [`usize::from_le_bytes`]
- [`usize::from_ne_bytes`]

Cargo
-----
- [You can now run `cargo c` as an alias for `cargo check`.][cargo/6218]
- [Usernames are now allowed in alt registry URLs.][cargo/6242]

Misc
----
- [`libproc_macro` has been added to the `crablang-src` distribution.][55280]

Compatibility Notes
-------------------
- [The argument types for AVX's
  `_mm256_stream_si256`, `_mm256_stream_pd`, `_mm256_stream_ps`][55610] have
  been changed from `*const` to `*mut` as the previous implementation
  was unsound.


[55148]: https://github.com/crablang/crablang/pull/55148/
[55238]: https://github.com/crablang/crablang/pull/55238/
[55280]: https://github.com/crablang/crablang/pull/55280/
[55610]: https://github.com/crablang/crablang/pull/55610/
[55663]: https://github.com/crablang/crablang/pull/55663/
[55702]: https://github.com/crablang/crablang/pull/55702/
[55837]: https://github.com/crablang/crablang/pull/55837/
[55843]: https://github.com/crablang/crablang/pull/55843/
[56072]: https://github.com/crablang/crablang/pull/56072/
[56245]: https://github.com/crablang/crablang/pull/56245/
[56365]: https://github.com/crablang/crablang/pull/56365/
[56366]: https://github.com/crablang/crablang/pull/56366/
[56395]: https://github.com/crablang/crablang/pull/56395/
[56759]: https://github.com/crablang/crablang/pull/56759/
[cargo/6218]: https://github.com/crablang/cargo/pull/6218/
[cargo/6242]: https://github.com/crablang/cargo/pull/6242/
[`CStr::as_ptr`]: https://doc.crablang.org/std/ffi/struct.CStr.html#method.as_ptr
[`Cell::as_ptr`]: https://doc.crablang.org/std/cell/struct.Cell.html#method.as_ptr
[`Duration::as_secs`]: https://doc.crablang.org/std/time/struct.Duration.html#method.as_secs
[`Duration::subsec_micros`]: https://doc.crablang.org/std/time/struct.Duration.html#method.subsec_micros
[`Duration::subsec_millis`]: https://doc.crablang.org/std/time/struct.Duration.html#method.subsec_millis
[`Duration::subsec_nanos`]: https://doc.crablang.org/std/time/struct.Duration.html#method.subsec_nanos
[`Ipv4Addr::is_unspecified`]: https://doc.crablang.org/std/net/struct.Ipv4Addr.html#method.is_unspecified
[`Ipv6Addr::new`]: https://doc.crablang.org/std/net/struct.Ipv6Addr.html#method.new
[`Ipv6Addr::octets`]: https://doc.crablang.org/std/net/struct.Ipv6Addr.html#method.octets
[`ManuallyDrop::into_inner`]: https://doc.crablang.org/std/mem/struct.ManuallyDrop.html#method.into_inner
[`ManuallyDrop::new`]: https://doc.crablang.org/std/mem/struct.ManuallyDrop.html#method.new
[`NonNull::as_ptr`]: https://doc.crablang.org/std/ptr/struct.NonNull.html#method.as_ptr
[`RangeInclusive::end`]: https://doc.crablang.org/std/ops/struct.RangeInclusive.html#method.end
[`RangeInclusive::start`]: https://doc.crablang.org/std/ops/struct.RangeInclusive.html#method.start
[`UnsafeCell::get`]: https://doc.crablang.org/std/cell/struct.UnsafeCell.html#method.get
[`slice::as_ptr`]: https://doc.crablang.org/std/primitive.slice.html#method.as_ptr
[`char::is_ascii`]: https://doc.crablang.org/std/primitive.char.html#method.is_ascii
[`i128::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i128.html#method.from_be_bytes
[`i128::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i128.html#method.from_le_bytes
[`i128::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i128.html#method.from_ne_bytes
[`i128::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i128.html#method.to_be_bytes
[`i128::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i128.html#method.to_le_bytes
[`i128::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i128.html#method.to_ne_bytes
[`i16::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i16.html#method.from_be_bytes
[`i16::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i16.html#method.from_le_bytes
[`i16::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i16.html#method.from_ne_bytes
[`i16::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i16.html#method.to_be_bytes
[`i16::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i16.html#method.to_le_bytes
[`i16::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i16.html#method.to_ne_bytes
[`i32::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i32.html#method.from_be_bytes
[`i32::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i32.html#method.from_le_bytes
[`i32::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i32.html#method.from_ne_bytes
[`i32::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i32.html#method.to_be_bytes
[`i32::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i32.html#method.to_le_bytes
[`i32::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i32.html#method.to_ne_bytes
[`i64::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i64.html#method.from_be_bytes
[`i64::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i64.html#method.from_le_bytes
[`i64::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i64.html#method.from_ne_bytes
[`i64::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i64.html#method.to_be_bytes
[`i64::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i64.html#method.to_le_bytes
[`i64::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i64.html#method.to_ne_bytes
[`i8::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i8.html#method.from_be_bytes
[`i8::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i8.html#method.from_le_bytes
[`i8::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i8.html#method.from_ne_bytes
[`i8::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.i8.html#method.to_be_bytes
[`i8::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.i8.html#method.to_le_bytes
[`i8::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.i8.html#method.to_ne_bytes
[`isize::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.isize.html#method.from_be_bytes
[`isize::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.isize.html#method.from_le_bytes
[`isize::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.isize.html#method.from_ne_bytes
[`isize::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.isize.html#method.to_be_bytes
[`isize::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.isize.html#method.to_le_bytes
[`isize::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.isize.html#method.to_ne_bytes
[`iter::empty`]: https://doc.crablang.org/std/iter/fn.empty.html
[`str::as_ptr`]: https://doc.crablang.org/std/primitive.str.html#method.as_ptr
[`u128::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u128.html#method.from_be_bytes
[`u128::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u128.html#method.from_le_bytes
[`u128::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u128.html#method.from_ne_bytes
[`u128::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u128.html#method.to_be_bytes
[`u128::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u128.html#method.to_le_bytes
[`u128::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u128.html#method.to_ne_bytes
[`u16::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u16.html#method.from_be_bytes
[`u16::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u16.html#method.from_le_bytes
[`u16::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u16.html#method.from_ne_bytes
[`u16::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u16.html#method.to_be_bytes
[`u16::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u16.html#method.to_le_bytes
[`u16::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u16.html#method.to_ne_bytes
[`u32::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u32.html#method.from_be_bytes
[`u32::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u32.html#method.from_le_bytes
[`u32::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u32.html#method.from_ne_bytes
[`u32::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u32.html#method.to_be_bytes
[`u32::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u32.html#method.to_le_bytes
[`u32::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u32.html#method.to_ne_bytes
[`u64::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u64.html#method.from_be_bytes
[`u64::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u64.html#method.from_le_bytes
[`u64::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u64.html#method.from_ne_bytes
[`u64::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u64.html#method.to_be_bytes
[`u64::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u64.html#method.to_le_bytes
[`u64::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u64.html#method.to_ne_bytes
[`u8::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u8.html#method.from_be_bytes
[`u8::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u8.html#method.from_le_bytes
[`u8::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u8.html#method.from_ne_bytes
[`u8::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.u8.html#method.to_be_bytes
[`u8::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.u8.html#method.to_le_bytes
[`u8::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.u8.html#method.to_ne_bytes
[`usize::from_be_bytes`]: https://doc.crablang.org/stable/std/primitive.usize.html#method.from_be_bytes
[`usize::from_le_bytes`]: https://doc.crablang.org/stable/std/primitive.usize.html#method.from_le_bytes
[`usize::from_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.usize.html#method.from_ne_bytes
[`usize::to_be_bytes`]: https://doc.crablang.org/stable/std/primitive.usize.html#method.to_be_bytes
[`usize::to_le_bytes`]: https://doc.crablang.org/stable/std/primitive.usize.html#method.to_le_bytes
[`usize::to_ne_bytes`]: https://doc.crablang.org/stable/std/primitive.usize.html#method.to_ne_bytes


Version 1.31.1 (2018-12-20)
===========================

- [Fix CrabLang failing to build on `powerpc-unknown-netbsd`][56562]
- [Fix broken go-to-definition in RLS][rls/1171]
- [Fix infinite loop on hover in RLS][rls/1170]

[56562]: https://github.com/crablang/crablang/pull/56562
[rls/1171]: https://github.com/crablang/rls/issues/1171
[rls/1170]: https://github.com/crablang/rls/pull/1170

Version 1.31.0 (2018-12-06)
==========================

Language
--------
- 🎉 [This version marks the release of the 2018 edition of CrabLang.][54057] 🎉
- [New lifetime elision rules now allow for eliding lifetimes in functions and
  impl headers.][54778] E.g. `impl<'a> Reader for BufReader<'a> {}` can now be
  `impl Reader for BufReader<'_> {}`. Lifetimes are still required to be defined
  in structs.
- [You can now define and use `const` functions.][54835] These are currently
  a strict minimal subset of the [const fn RFC][RFC-911]. Refer to the
  [language reference][const-reference] for what exactly is available.
- [You can now use tool lints, which allow you to scope lints from external
  tools using attributes.][54870] E.g. `#[allow(clippy::filter_map)]`.
- [`#[no_mangle]` and `#[export_name]` attributes can now be located anywhere in
  a crate, not just in exported functions.][54451]
- [You can now use parentheses in pattern matches.][54497]

Compiler
--------
- [Updated musl to 1.1.20][54430]

Libraries
---------
- [You can now convert `num::NonZero*` types to their raw equivalents using the
  `From` trait.][54240] E.g. `u8` now implements `From<NonZeroU8>`.
- [You can now convert a `&Option<T>` into `Option<&T>` and `&mut Option<T>`
  into `Option<&mut T>` using the `From` trait.][53218]
- [You can now multiply (`*`) a `time::Duration` by a `u32`.][52813]


Stabilized APIs
---------------
- [`slice::align_to`]
- [`slice::align_to_mut`]
- [`slice::chunks_exact`]
- [`slice::chunks_exact_mut`]
- [`slice::rchunks`]
- [`slice::rchunks_mut`]
- [`slice::rchunks_exact`]
- [`slice::rchunks_exact_mut`]
- [`Option::replace`]

Cargo
-----
- [Cargo will now download crates in parallel using HTTP/2.][cargo/6005]
- [You can now rename packages in your Cargo.toml][cargo/6319] We have a guide
  on [how to use the `package` key in your dependencies.][cargo-rename-reference]

[52813]: https://github.com/crablang/crablang/pull/52813/
[53218]: https://github.com/crablang/crablang/pull/53218/
[54057]: https://github.com/crablang/crablang/pull/54057/
[54240]: https://github.com/crablang/crablang/pull/54240/
[54430]: https://github.com/crablang/crablang/pull/54430/
[54451]: https://github.com/crablang/crablang/pull/54451/
[54497]: https://github.com/crablang/crablang/pull/54497/
[54778]: https://github.com/crablang/crablang/pull/54778/
[54835]: https://github.com/crablang/crablang/pull/54835/
[54870]: https://github.com/crablang/crablang/pull/54870/
[RFC-911]: https://github.com/crablang/rfcs/pull/911
[`Option::replace`]: https://doc.crablang.org/std/option/enum.Option.html#method.replace
[`slice::align_to_mut`]: https://doc.crablang.org/std/primitive.slice.html#method.align_to_mut
[`slice::align_to`]: https://doc.crablang.org/std/primitive.slice.html#method.align_to
[`slice::chunks_exact_mut`]: https://doc.crablang.org/std/primitive.slice.html#method.chunks_exact_mut
[`slice::chunks_exact`]: https://doc.crablang.org/std/primitive.slice.html#method.chunks_exact
[`slice::rchunks_exact_mut`]: https://doc.crablang.org/std/primitive.slice.html#method.rchunks_mut
[`slice::rchunks_exact`]: https://doc.crablang.org/std/primitive.slice.html#method.rchunks_exact
[`slice::rchunks_mut`]: https://doc.crablang.org/std/primitive.slice.html#method.rchunks_mut
[`slice::rchunks`]: https://doc.crablang.org/std/primitive.slice.html#method.rchunks
[cargo/6005]: https://github.com/crablang/cargo/pull/6005/
[cargo/6319]: https://github.com/crablang/cargo/pull/6319/
[cargo-rename-reference]: https://doc.crablang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml
[const-reference]: https://doc.crablang.org/reference/items/functions.html#const-functions

Version 1.30.1 (2018-11-08)
===========================

- [Fixed overflow ICE in crablangdoc][54199]
- [Cap Cargo progress bar width at 60 in MSYS terminals][cargo/6122]

[54199]: https://github.com/crablang/crablang/pull/54199
[cargo/6122]: https://github.com/crablang/cargo/pull/6122

Version 1.30.0 (2018-10-25)
==========================

Language
--------
- [Procedural macros are now available.][52081] These kinds of macros allow for
  more powerful code generation. There is a [new chapter available][proc-macros]
  in the CrabLang Programming Language book that goes further in depth.
- [You can now use keywords as identifiers using the raw identifiers
  syntax (`r#`),][53236] e.g. `let r#for = true;`
- [Using anonymous parameters in traits is now deprecated with a warning and
  will be a hard error in the 2018 edition.][53272]
- [You can now use `crate` in paths.][54404] This allows you to refer to the
  crate root in the path, e.g. `use crate::foo;` refers to `foo` in `src/lib.rs`.
- [Using a external crate no longer requires being prefixed with `::`.][54404]
  Previously, using a external crate in a module without a use statement
  required `let json = ::serde_json::from_str(foo);` but can now be written
  as `let json = serde_json::from_str(foo);`.
- [You can now apply the `#[used]` attribute to static items to prevent the
  compiler from optimising them away, even if they appear to be unused,][51363]
  e.g. `#[used] static FOO: u32 = 1;`
- [You can now import and reexport macros from other crates with the `use`
  syntax.][50911] Macros exported with `#[macro_export]` are now placed into
  the root module of the crate. If your macro relies on calling other local
  macros, it is recommended to export with the
  `#[macro_export(local_inner_macros)]` attribute so users won't have to import
  those macros.
- [You can now catch visibility keywords (e.g. `pub`, `pub(crate)`) in macros
  using the `vis` specifier.][53370]
- [Non-macro attributes now allow all forms of literals, not just
  strings.][53044] Previously, you would write `#[attr("true")]`, and you can now
  write `#[attr(true)]`.
- [You can now specify a function to handle a panic in the CrabLang runtime with the
  `#[panic_handler]` attribute.][51366]

Compiler
--------
- [Added the `riscv32imc-unknown-none-elf` target.][53822]
- [Added the `aarch64-unknown-netbsd` target][53165]
- [Upgraded to LLVM 8.][53611]

Libraries
---------
- [`ManuallyDrop` now allows the inner type to be unsized.][53033]

Stabilized APIs
---------------
- [`Ipv4Addr::BROADCAST`]
- [`Ipv4Addr::LOCALHOST`]
- [`Ipv4Addr::UNSPECIFIED`]
- [`Ipv6Addr::LOCALHOST`]
- [`Ipv6Addr::UNSPECIFIED`]
- [`Iterator::find_map`]

  The following methods are replacement methods for `trim_left`, `trim_right`,
  `trim_left_matches`, and `trim_right_matches`, which will be deprecated
  in 1.33.0:
- [`str::trim_end_matches`]
- [`str::trim_end`]
- [`str::trim_start_matches`]
- [`str::trim_start`]

Cargo
----
- [`cargo run` doesn't require specifying a package in workspaces.][cargo/5877]
- [`cargo doc` now supports `--message-format=json`.][cargo/5878] This is
  equivalent to calling `crablangdoc --error-format=json`.
- [Cargo will now provide a progress bar for builds.][cargo/5995]

Misc
----
- [`crablangdoc` allows you to specify what edition to treat your code as with the
  `--edition` option.][54057]
- [`crablangdoc` now has the `--color` (specify whether to output color) and
  `--error-format` (specify error format, e.g. `json`) options.][53003]
- [We now distribute a `crablang-gdbgui` script that invokes `gdbgui` with CrabLang
  debug symbols.][53774]
- [Attributes from CrabLang tools such as `crablangfmt` or `clippy` are now
  available,][53459] e.g. `#[crablangfmt::skip]` will skip formatting the next item.

[50911]: https://github.com/crablang/crablang/pull/50911/
[51363]: https://github.com/crablang/crablang/pull/51363/
[51366]: https://github.com/crablang/crablang/pull/51366/
[52081]: https://github.com/crablang/crablang/pull/52081/
[53003]: https://github.com/crablang/crablang/pull/53003/
[53033]: https://github.com/crablang/crablang/pull/53033/
[53044]: https://github.com/crablang/crablang/pull/53044/
[53165]: https://github.com/crablang/crablang/pull/53165/
[53611]: https://github.com/crablang/crablang/pull/53611/
[53236]: https://github.com/crablang/crablang/pull/53236/
[53272]: https://github.com/crablang/crablang/pull/53272/
[53370]: https://github.com/crablang/crablang/pull/53370/
[53459]: https://github.com/crablang/crablang/pull/53459/
[53774]: https://github.com/crablang/crablang/pull/53774/
[53822]: https://github.com/crablang/crablang/pull/53822/
[54057]: https://github.com/crablang/crablang/pull/54057/
[54404]: https://github.com/crablang/crablang/pull/54404/
[cargo/5877]: https://github.com/crablang/cargo/pull/5877/
[cargo/5878]: https://github.com/crablang/cargo/pull/5878/
[cargo/5995]: https://github.com/crablang/cargo/pull/5995/
[proc-macros]: https://doc.crablang.org/nightly/book/2018-edition/ch19-06-macros.html

[`Ipv4Addr::BROADCAST`]: https://doc.crablang.org/nightly/std/net/struct.Ipv4Addr.html#associatedconstant.BROADCAST
[`Ipv4Addr::LOCALHOST`]: https://doc.crablang.org/nightly/std/net/struct.Ipv4Addr.html#associatedconstant.LOCALHOST
[`Ipv4Addr::UNSPECIFIED`]: https://doc.crablang.org/nightly/std/net/struct.Ipv4Addr.html#associatedconstant.UNSPECIFIED
[`Ipv6Addr::LOCALHOST`]: https://doc.crablang.org/nightly/std/net/struct.Ipv6Addr.html#associatedconstant.LOCALHOST
[`Ipv6Addr::UNSPECIFIED`]: https://doc.crablang.org/nightly/std/net/struct.Ipv6Addr.html#associatedconstant.UNSPECIFIED
[`Iterator::find_map`]: https://doc.crablang.org/std/iter/trait.Iterator.html#method.find_map
[`str::trim_end_matches`]: https://doc.crablang.org/nightly/std/primitive.str.html#method.trim_end_matches
[`str::trim_end`]: https://doc.crablang.org/nightly/std/primitive.str.html#method.trim_end
[`str::trim_start_matches`]: https://doc.crablang.org/nightly/std/primitive.str.html#method.trim_start_matches
[`str::trim_start`]: https://doc.crablang.org/nightly/std/primitive.str.html#method.trim_start


Version 1.29.2 (2018-10-11)
===========================

- [Workaround for an aliasing-related LLVM bug, which caused miscompilation.][54639]
- The `rls-preview` component on the windows-gnu targets has been restored.

[54639]: https://github.com/crablang/crablang/pull/54639


Version 1.29.1 (2018-09-25)
===========================

Security Notes
--------------

- The standard library's `str::repeat` function contained an out of bounds write
  caused by an integer overflow. This has been fixed by deterministically
  panicking when an overflow happens.

  Thank you to Scott McMurray for responsibly disclosing this vulnerability to
  us.


Version 1.29.0 (2018-09-13)
==========================

Compiler
--------
- [Bumped minimum LLVM version to 5.0.][51899]
- [Added `powerpc64le-unknown-linux-musl` target.][51619]
- [Added `aarch64-unknown-hermit` and `x86_64-unknown-hermit` targets.][52861]
- [Upgraded to LLVM 7.][51966]

Libraries
---------
- [`Once::call_once` no longer requires `Once` to be `'static`.][52239]
- [`BuildHasherDefault` now implements `PartialEq` and `Eq`.][52402]
- [`Box<CStr>`, `Box<OsStr>`, and `Box<Path>` now implement `Clone`.][51912]
- [Implemented `PartialEq<&str>` for `OsString` and `PartialEq<OsString>`
  for `&str`.][51178]
- [`Cell<T>` now allows `T` to be unsized.][50494]
- [`SocketAddr` is now stable on Redox.][52656]

Stabilized APIs
---------------
- [`Arc::downcast`]
- [`Iterator::flatten`]
- [`Rc::downcast`]

Cargo
-----
- [Cargo can silently fix some bad lockfiles.][cargo/5831] You can use
  `--locked` to disable this behavior.
- [`cargo-install` will now allow you to cross compile an install
  using `--target`.][cargo/5614]
- [Added the `cargo-fix` subcommand to automatically move project code from
  2015 edition to 2018.][cargo/5723]
- [`cargo doc` can now optionally document private types using the
  `--document-private-items` flag.][cargo/5543]

Misc
----
- [`crablangdoc` now has the `--cap-lints` option which demotes all lints above
  the specified level to that level.][52354] For example `--cap-lints warn`
  will demote `deny` and `forbid` lints to `warn`.
- [`crablangc` and `crablangdoc` will now have the exit code of `1` if compilation
  fails and `101` if there is a panic.][52197]
- [A preview of clippy has been made available through crablangup.][51122]
  You can install the preview with `crablangup component add clippy-preview`.

Compatibility Notes
-------------------
- [`str::{slice_unchecked, slice_unchecked_mut}` are now deprecated.][51807]
  Use `str::get_unchecked(begin..end)` instead.
- [`std::env::home_dir` is now deprecated for its unintuitive behavior.][51656]
  Consider using the `home_dir` function from
  https://crates.io/crates/dirs instead.
- [`crablangc` will no longer silently ignore invalid data in target spec.][52330]
- [`cfg` attributes and `--cfg` command line flags are now more
  strictly validated.][53893]

[53893]: https://github.com/crablang/crablang/pull/53893/
[52861]: https://github.com/crablang/crablang/pull/52861/
[51966]: https://github.com/crablang/crablang/pull/51966/
[52656]: https://github.com/crablang/crablang/pull/52656/
[52239]: https://github.com/crablang/crablang/pull/52239/
[52330]: https://github.com/crablang/crablang/pull/52330/
[52354]: https://github.com/crablang/crablang/pull/52354/
[52402]: https://github.com/crablang/crablang/pull/52402/
[52197]: https://github.com/crablang/crablang/pull/52197/
[51807]: https://github.com/crablang/crablang/pull/51807/
[51899]: https://github.com/crablang/crablang/pull/51899/
[51912]: https://github.com/crablang/crablang/pull/51912/
[51619]: https://github.com/crablang/crablang/pull/51619/
[51656]: https://github.com/crablang/crablang/pull/51656/
[51178]: https://github.com/crablang/crablang/pull/51178/
[51122]: https://github.com/crablang/crablang/pull/51122
[50494]: https://github.com/crablang/crablang/pull/50494/
[cargo/5543]: https://github.com/crablang/cargo/pull/5543
[cargo/5614]: https://github.com/crablang/cargo/pull/5614/
[cargo/5723]: https://github.com/crablang/cargo/pull/5723/
[cargo/5831]: https://github.com/crablang/cargo/pull/5831/
[`Arc::downcast`]: https://doc.crablang.org/std/sync/struct.Arc.html#method.downcast
[`Iterator::flatten`]: https://doc.crablang.org/std/iter/trait.Iterator.html#method.flatten
[`Rc::downcast`]: https://doc.crablang.org/std/rc/struct.Rc.html#method.downcast


Version 1.28.0 (2018-08-02)
===========================

Language
--------
- [The `#[repr(transparent)]` attribute is now stable.][51562] This attribute
  allows a CrabLang newtype wrapper (`struct NewType<T>(T);`) to be represented as
  the inner type across Foreign Function Interface (FFI) boundaries.
- [The keywords `pure`, `sizeof`, `alignof`, and `offsetof` have been unreserved
  and can now be used as identifiers.][51196]
- [The `GlobalAlloc` trait and `#[global_allocator]` attribute are now
  stable.][51241] This will allow users to specify a global allocator for
  their program.
- [Unit test functions marked with the `#[test]` attribute can now return
  `Result<(), E: Debug>` in addition to `()`.][51298]
- [The `lifetime` specifier for `macro_rules!` is now stable.][50385] This
  allows macros to easily target lifetimes.

Compiler
--------
- [The `s` and `z` optimisation levels are now stable.][50265] These optimisations
  prioritise making smaller binary sizes. `z` is the same as `s` with the
  exception that it does not vectorise loops, which typically results in an even
  smaller binary.
- [The short error format is now stable.][49546] Specified with
  `--error-format=short` this option will provide a more compressed output of
  crablang error messages.
- [Added a lint warning when you have duplicated `macro_export`s.][50143]
- [Reduced the number of allocations in the macro parser.][50855] This can
  improve compile times of macro heavy crates on average by 5%.

Libraries
---------
- [Implemented `Default` for `&mut str`.][51306]
- [Implemented `From<bool>` for all integer and unsigned number types.][50554]
- [Implemented `Extend` for `()`.][50234]
- [The `Debug` implementation of `time::Duration` should now be more easily
  human readable.][50364] Previously a `Duration` of one second would printed as
  `Duration { secs: 1, nanos: 0 }` and will now be printed as `1s`.
- [Implemented `From<&String>` for `Cow<str>`, `From<&Vec<T>>` for `Cow<[T]>`,
  `From<Cow<CStr>>` for `CString`, `From<CString>, From<CStr>, From<&CString>`
  for `Cow<CStr>`, `From<OsString>, From<OsStr>, From<&OsString>` for
  `Cow<OsStr>`, `From<&PathBuf>` for `Cow<Path>`, and `From<Cow<Path>>`
  for `PathBuf`.][50170]
- [Implemented `Shl` and `Shr` for `Wrapping<u128>`
  and `Wrapping<i128>`.][50465]
- [`DirEntry::metadata` now uses `fstatat` instead of `lstat` when
  possible.][51050] This can provide up to a 40% speed increase.
- [Improved error messages when using `format!`.][50610]

Stabilized APIs
---------------
- [`Iterator::step_by`]
- [`Path::ancestors`]
- [`SystemTime::UNIX_EPOCH`]
- [`alloc::GlobalAlloc`]
- [`alloc::Layout`]
- [`alloc::LayoutErr`]
- [`alloc::System`]
- [`alloc::alloc`]
- [`alloc::alloc_zeroed`]
- [`alloc::dealloc`]
- [`alloc::realloc`]
- [`alloc::handle_alloc_error`]
- [`btree_map::Entry::or_default`]
- [`fmt::Alignment`]
- [`hash_map::Entry::or_default`]
- [`iter::repeat_with`]
- [`num::NonZeroUsize`]
- [`num::NonZeroU128`]
- [`num::NonZeroU16`]
- [`num::NonZeroU32`]
- [`num::NonZeroU64`]
- [`num::NonZeroU8`]
- [`ops::RangeBounds`]
- [`slice::SliceIndex`]
- [`slice::from_mut`]
- [`slice::from_ref`]
- [`{Any + Send + Sync}::downcast_mut`]
- [`{Any + Send + Sync}::downcast_ref`]
- [`{Any + Send + Sync}::is`]

Cargo
-----
- [Cargo will now no longer allow you to publish crates with build scripts that
  modify the `src` directory.][cargo/5584] The `src` directory in a crate should be
  considered to be immutable.

Misc
----
- [The `suggestion_applicability` field in `crablangc`'s json output is now
  stable.][50486] This will allow dev tools to check whether a code suggestion
  would apply to them.

Compatibility Notes
-------------------
- [CrabLang will consider trait objects with duplicated constraints to be the same
  type as without the duplicated constraint.][51276] For example the below code will
  now fail to compile.
  ```crablang
  trait Trait {}

  impl Trait + Send {
      fn test(&self) { println!("one"); } //~ ERROR duplicate definitions with name `test`
  }

  impl Trait + Send + Send {
      fn test(&self) { println!("two"); }
  }
  ```

[49546]: https://github.com/crablang/crablang/pull/49546/
[50143]: https://github.com/crablang/crablang/pull/50143/
[50170]: https://github.com/crablang/crablang/pull/50170/
[50234]: https://github.com/crablang/crablang/pull/50234/
[50265]: https://github.com/crablang/crablang/pull/50265/
[50364]: https://github.com/crablang/crablang/pull/50364/
[50385]: https://github.com/crablang/crablang/pull/50385/
[50465]: https://github.com/crablang/crablang/pull/50465/
[50486]: https://github.com/crablang/crablang/pull/50486/
[50554]: https://github.com/crablang/crablang/pull/50554/
[50610]: https://github.com/crablang/crablang/pull/50610/
[50855]: https://github.com/crablang/crablang/pull/50855/
[51050]: https://github.com/crablang/crablang/pull/51050/
[51196]: https://github.com/crablang/crablang/pull/51196/
[51241]: https://github.com/crablang/crablang/pull/51241/
[51276]: https://github.com/crablang/crablang/pull/51276/
[51298]: https://github.com/crablang/crablang/pull/51298/
[51306]: https://github.com/crablang/crablang/pull/51306/
[51562]: https://github.com/crablang/crablang/pull/51562/
[cargo/5584]: https://github.com/crablang/cargo/pull/5584/
[`Iterator::step_by`]: https://doc.crablang.org/std/iter/trait.Iterator.html#method.step_by
[`Path::ancestors`]: https://doc.crablang.org/std/path/struct.Path.html#method.ancestors
[`SystemTime::UNIX_EPOCH`]: https://doc.crablang.org/std/time/struct.SystemTime.html#associatedconstant.UNIX_EPOCH
[`alloc::GlobalAlloc`]: https://doc.crablang.org/std/alloc/trait.GlobalAlloc.html
[`alloc::Layout`]: https://doc.crablang.org/std/alloc/struct.Layout.html
[`alloc::LayoutErr`]: https://doc.crablang.org/std/alloc/struct.LayoutErr.html
[`alloc::System`]: https://doc.crablang.org/std/alloc/struct.System.html
[`alloc::alloc`]: https://doc.crablang.org/std/alloc/fn.alloc.html
[`alloc::alloc_zeroed`]: https://doc.crablang.org/std/alloc/fn.alloc_zeroed.html
[`alloc::dealloc`]: https://doc.crablang.org/std/alloc/fn.dealloc.html
[`alloc::realloc`]: https://doc.crablang.org/std/alloc/fn.realloc.html
[`alloc::handle_alloc_error`]: https://doc.crablang.org/std/alloc/fn.handle_alloc_error.html
[`btree_map::Entry::or_default`]: https://doc.crablang.org/std/collections/btree_map/enum.Entry.html#method.or_default
[`fmt::Alignment`]: https://doc.crablang.org/std/fmt/enum.Alignment.html
[`hash_map::Entry::or_default`]: https://doc.crablang.org/std/collections/hash_map/enum.Entry.html#method.or_default
[`iter::repeat_with`]: https://doc.crablang.org/std/iter/fn.repeat_with.html
[`num::NonZeroUsize`]: https://doc.crablang.org/std/num/struct.NonZeroUsize.html
[`num::NonZeroU128`]: https://doc.crablang.org/std/num/struct.NonZeroU128.html
[`num::NonZeroU16`]: https://doc.crablang.org/std/num/struct.NonZeroU16.html
[`num::NonZeroU32`]: https://doc.crablang.org/std/num/struct.NonZeroU32.html
[`num::NonZeroU64`]: https://doc.crablang.org/std/num/struct.NonZeroU64.html
[`num::NonZeroU8`]: https://doc.crablang.org/std/num/struct.NonZeroU8.html
[`ops::RangeBounds`]: https://doc.crablang.org/std/ops/trait.RangeBounds.html
[`slice::SliceIndex`]: https://doc.crablang.org/std/slice/trait.SliceIndex.html
[`slice::from_mut`]: https://doc.crablang.org/std/slice/fn.from_mut.html
[`slice::from_ref`]: https://doc.crablang.org/std/slice/fn.from_ref.html
[`{Any + Send + Sync}::downcast_mut`]: https://doc.crablang.org/std/any/trait.Any.html#method.downcast_mut-2
[`{Any + Send + Sync}::downcast_ref`]: https://doc.crablang.org/std/any/trait.Any.html#method.downcast_ref-2
[`{Any + Send + Sync}::is`]: https://doc.crablang.org/std/any/trait.Any.html#method.is-2

Version 1.27.2 (2018-07-20)
===========================

Compatibility Notes
-------------------

- The borrow checker was fixed to avoid potential unsoundness when using
  match ergonomics: [#52213][52213].

[52213]: https://github.com/crablang/crablang/issues/52213

Version 1.27.1 (2018-07-10)
===========================

Security Notes
--------------

- crablangdoc would execute plugins in the /tmp/crablangdoc/plugins directory
  when running, which enabled executing code as some other user on a
  given machine. This release fixes that vulnerability; you can read
  more about this on the [blog][crablangdoc-sec]. The associated CVE is [CVE-2018-1000622].

  Thank you to Red Hat for responsibly disclosing this vulnerability to us.

Compatibility Notes
-------------------

- The borrow checker was fixed to avoid an additional potential unsoundness when using
  match ergonomics: [#51415][51415], [#49534][49534].

[51415]: https://github.com/crablang/crablang/issues/51415
[49534]: https://github.com/crablang/crablang/issues/49534
[crablangdoc-sec]: https://blog.crablang.org/2018/07/06/security-advisory-for-crablangdoc.html
[CVE-2018-1000622]: https://cve.mitre.org/cgi-bin/cvename.cgi?name=%20CVE-2018-1000622

Version 1.27.0 (2018-06-21)
==========================

Language
--------
- [Removed 'proc' from the reserved keywords list.][49699] This allows `proc` to
  be used as an identifier.
- [The dyn syntax is now available.][49968] This syntax is equivalent to the
  bare `Trait` syntax, and should make it clearer when being used in tandem with
  `impl Trait` because it is equivalent to the following syntax:
  `&Trait == &dyn Trait`, `&mut Trait == &mut dyn Trait`, and
  `Box<Trait> == Box<dyn Trait>`.
- [Attributes on generic parameters such as types and lifetimes are
  now stable.][48851] e.g.
  `fn foo<#[lifetime_attr] 'a, #[type_attr] T: 'a>() {}`
- [The `#[must_use]` attribute can now also be used on functions as well as
  types.][48925] It provides a lint that by default warns users when the
  value returned by a function has not been used.

Compiler
--------
- [Added the `armv5te-unknown-linux-musleabi` target.][50423]

Libraries
---------
- [SIMD (Single Instruction Multiple Data) on x86/x86_64 is now stable.][49664]
  This includes [`arch::x86`] & [`arch::x86_64`] modules which contain
  SIMD intrinsics, a new macro called `is_x86_feature_detected!`, the
  `#[target_feature(enable="")]` attribute, and adding `target_feature = ""` to
  the `cfg` attribute.
- [A lot of methods for `[u8]`, `f32`, and `f64` previously only available in
  std are now available in core.][49896]
- [The generic `Rhs` type parameter on `ops::{Shl, ShlAssign, Shr}` now defaults
  to `Self`.][49630]
- [`std::str::replace` now has the `#[must_use]` attribute][50177] to clarify
  that the operation isn't done in place.
- [`Clone::clone`, `Iterator::collect`, and `ToOwned::to_owned` now have
  the `#[must_use]` attribute][49533] to warn about unused potentially
  expensive allocations.

Stabilized APIs
---------------
- [`DoubleEndedIterator::rfind`]
- [`DoubleEndedIterator::rfold`]
- [`DoubleEndedIterator::try_rfold`]
- [`Duration::from_micros`]
- [`Duration::from_nanos`]
- [`Duration::subsec_micros`]
- [`Duration::subsec_millis`]
- [`HashMap::remove_entry`]
- [`Iterator::try_fold`]
- [`Iterator::try_for_each`]
- [`NonNull::cast`]
- [`Option::filter`]
- [`String::replace_range`]
- [`Take::set_limit`]
- [`hint::unreachable_unchecked`]
- [`os::unix::process::parent_id`]
- [`ptr::swap_nonoverlapping`]
- [`slice::rsplit_mut`]
- [`slice::rsplit`]
- [`slice::swap_with_slice`]

Cargo
-----
- [`cargo-metadata` now includes `authors`, `categories`, `keywords`,
  `readme`, and `repository` fields.][cargo/5386]
- [`cargo-metadata` now includes a package's `metadata` table.][cargo/5360]
- [Added the `--target-dir` optional argument.][cargo/5393] This allows you to specify
  a different directory than `target` for placing compilation artifacts.
- [Cargo will be adding automatic target inference for binaries, benchmarks,
  examples, and tests in the CrabLang 2018 edition.][cargo/5335] If your project specifies
  specific targets, e.g. using `[[bin]]`, and have other binaries in locations
  where cargo would infer a binary, Cargo will produce a warning. You can
  disable this feature ahead of time by setting any of the following to false:
  `autobins`, `autobenches`, `autoexamples`, `autotests`.
- [Cargo will now cache compiler information.][cargo/5359] This can be disabled by
  setting `CARGO_CACHE_CRABLANGC_INFO=0` in your environment.

Misc
----
- [Added “The CrabLangc book” into the official documentation.][49707]
  [“The CrabLangc book”] documents and teaches how to use the crablangc compiler.
- [All books available on `doc.crablang.org` are now searchable.][49623]

Compatibility Notes
-------------------
- [Calling a `CharExt` or `StrExt` method directly on core will no longer
  work.][49896] e.g. `::core::prelude::v1::StrExt::is_empty("")` will not
  compile, `"".is_empty()` will still compile.
- [`Debug` output on `atomic::{AtomicBool, AtomicIsize, AtomicPtr, AtomicUsize}`
  will only print the inner type.][48553] E.g.
  `print!("{:?}", AtomicBool::new(true))` will print `true`,
  not `AtomicBool(true)`.
- [The maximum number for `repr(align(N))` is now 2²⁹.][50378] Previously you
  could enter higher numbers but they were not supported by LLVM. Up to 512MB
  alignment should cover all use cases.
- The `.description()` method on the `std::error::Error` trait
  [has been soft-deprecated][50163]. It is no longer required to implement it.

[48553]: https://github.com/crablang/crablang/pull/48553/
[48851]: https://github.com/crablang/crablang/pull/48851/
[48925]: https://github.com/crablang/crablang/pull/48925/
[49533]: https://github.com/crablang/crablang/pull/49533/
[49623]: https://github.com/crablang/crablang/pull/49623/
[49630]: https://github.com/crablang/crablang/pull/49630/
[49664]: https://github.com/crablang/crablang/pull/49664/
[49699]: https://github.com/crablang/crablang/pull/49699/
[49707]: https://github.com/crablang/crablang/pull/49707/
[49896]: https://github.com/crablang/crablang/pull/49896/
[49968]: https://github.com/crablang/crablang/pull/49968/
[50163]: https://github.com/crablang/crablang/pull/50163
[50177]: https://github.com/crablang/crablang/pull/50177/
[50378]: https://github.com/crablang/crablang/pull/50378/
[50423]: https://github.com/crablang/crablang/pull/50423/
[cargo/5335]: https://github.com/crablang/cargo/pull/5335/
[cargo/5359]: https://github.com/crablang/cargo/pull/5359/
[cargo/5360]: https://github.com/crablang/cargo/pull/5360/
[cargo/5386]: https://github.com/crablang/cargo/pull/5386/
[cargo/5393]: https://github.com/crablang/cargo/pull/5393/
[`DoubleEndedIterator::rfind`]: https://doc.crablang.org/std/iter/trait.DoubleEndedIterator.html#method.rfind
[`DoubleEndedIterator::rfold`]: https://doc.crablang.org/std/iter/trait.DoubleEndedIterator.html#method.rfold
[`DoubleEndedIterator::try_rfold`]: https://doc.crablang.org/std/iter/trait.DoubleEndedIterator.html#method.try_rfold
[`Duration::from_micros`]: https://doc.crablang.org/std/time/struct.Duration.html#method.from_micros
[`Duration::from_nanos`]: https://doc.crablang.org/std/time/struct.Duration.html#method.from_nanos
[`Duration::subsec_micros`]: https://doc.crablang.org/std/time/struct.Duration.html#method.subsec_micros
[`Duration::subsec_millis`]: https://doc.crablang.org/std/time/struct.Duration.html#method.subsec_millis
[`HashMap::remove_entry`]: https://doc.crablang.org/std/collections/struct.HashMap.html#method.remove_entry
[`Iterator::try_fold`]: https://doc.crablang.org/std/iter/trait.Iterator.html#method.try_fold
[`Iterator::try_for_each`]: https://doc.crablang.org/std/iter/trait.Iterator.html#method.try_for_each
[`NonNull::cast`]: https://doc.crablang.org/std/ptr/struct.NonNull.html#method.cast
[`Option::filter`]: https://doc.crablang.org/std/option/enum.Option.html#method.filter
[`String::replace_range`]: https://doc.crablang.org/std/string/struct.String.html#method.replace_range
[`Take::set_limit`]: https://doc.crablang.org/std/io/struct.Take.html#method.set_limit
[`hint::unreachable_unchecked`]: https://doc.crablang.org/std/hint/fn.unreachable_unchecked.html
[`os::unix::process::parent_id`]: https://doc.crablang.org/std/os/unix/process/fn.parent_id.html
[`process::id`]: https://doc.crablang.org/std/process/fn.id.html
[`ptr::swap_nonoverlapping`]: https://doc.crablang.org/std/ptr/fn.swap_nonoverlapping.html
[`slice::rsplit_mut`]: https://doc.crablang.org/std/primitive.slice.html#method.rsplit_mut
[`slice::rsplit`]: https://doc.crablang.org/std/primitive.slice.html#method.rsplit
[`slice::swap_with_slice`]: https://doc.crablang.org/std/primitive.slice.html#method.swap_with_slice
[`arch::x86_64`]: https://doc.crablang.org/std/arch/x86_64/index.html
[`arch::x86`]: https://doc.crablang.org/std/arch/x86/index.html
[“The CrabLangc book”]: https://doc.crablang.org/crablangc


Version 1.26.2 (2018-06-05)
==========================

Compatibility Notes
-------------------

- [The borrow checker was fixed to avoid unsoundness when using match ergonomics.][51117]

[51117]: https://github.com/crablang/crablang/issues/51117


Version 1.26.1 (2018-05-29)
==========================

Tools
-----

- [RLS now works on Windows.][50646]
- [CrabLangfmt stopped badly formatting text in some cases.][crablangfmt/2695]


Compatibility Notes
--------

- [`fn main() -> impl Trait` no longer works for non-Termination
  trait.][50656]
  This reverts an accidental stabilization.
- [`NaN > NaN` no longer returns true in const-fn contexts.][50812]
- [Prohibit using turbofish for `impl Trait` in method arguments.][50950]

[50646]: https://github.com/crablang/crablang/issues/50646
[50656]: https://github.com/crablang/crablang/pull/50656
[50812]: https://github.com/crablang/crablang/pull/50812
[50950]: https://github.com/crablang/crablang/issues/50950
[crablangfmt/2695]: https://github.com/crablang-nursery/crablangfmt/issues/2695

Version 1.26.0 (2018-05-10)
==========================

Language
--------
- [Closures now implement `Copy` and/or `Clone` if all captured variables
  implement either or both traits.][49299]
- [The inclusive range syntax e.g. `for x in 0..=10` is now stable.][47813]
- [The `'_` lifetime is now stable. The underscore lifetime can be used anywhere a
  lifetime can be elided.][49458]
- [`impl Trait` is now stable allowing you to have abstract types in returns
   or in function parameters.][49255] E.g. `fn foo() -> impl Iterator<Item=u8>` or
  `fn open(path: impl AsRef<Path>)`.
- [Pattern matching will now automatically apply dereferences.][49394]
- [128-bit integers in the form of `u128` and `i128` are now stable.][49101]
- [`main` can now return `Result<(), E: Debug>`][49162] in addition to `()`.
- [A lot of operations are now available in a const context.][46882] E.g. You
  can now index into constant arrays, reference and dereference into constants,
  and use tuple struct constructors.
- [Fixed entry slice patterns are now stable.][48516] E.g.
  ```crablang
  let points = [1, 2, 3, 4];
  match points {
      [1, 2, 3, 4] => println!("All points were sequential."),
      _ => println!("Not all points were sequential."),
  }
  ```


Compiler
--------
- [LLD is now used as the default linker for `wasm32-unknown-unknown`.][48125]
- [Fixed exponential projection complexity on nested types.][48296]
  This can provide up to a ~12% reduction in compile times for certain crates.
- [Added the `--remap-path-prefix` option to crablangc.][48359] Allowing you
  to remap path prefixes outputted by the compiler.
- [Added `powerpc-unknown-netbsd` target.][48281]

Libraries
---------
- [Implemented `From<u16> for usize` & `From<{u8, i16}> for isize`.][49305]
- [Added hexadecimal formatting for integers with fmt::Debug][48978]
  e.g. `assert!(format!("{:02x?}", b"Foo\0") == "[46, 6f, 6f, 00]")`
- [Implemented `Default, Hash` for `cmp::Reverse`.][48628]
- [Optimized `str::repeat` being 8x faster in large cases.][48657]
- [`ascii::escape_default` is now available in libcore.][48735]
- [Trailing commas are now supported in std and core macros.][48056]
- [Implemented `Copy, Clone` for `cmp::Reverse`][47379]
- [Implemented `Clone` for `char::{ToLowercase, ToUppercase}`.][48629]

Stabilized APIs
---------------
- [`*const T::add`]
- [`*const T::copy_to_nonoverlapping`]
- [`*const T::copy_to`]
- [`*const T::read_unaligned`]
- [`*const T::read_volatile`]
- [`*const T::read`]
- [`*const T::sub`]
- [`*const T::wrapping_add`]
- [`*const T::wrapping_sub`]
- [`*mut T::add`]
- [`*mut T::copy_to_nonoverlapping`]
- [`*mut T::copy_to`]
- [`*mut T::read_unaligned`]
- [`*mut T::read_volatile`]
- [`*mut T::read`]
- [`*mut T::replace`]
- [`*mut T::sub`]
- [`*mut T::swap`]
- [`*mut T::wrapping_add`]
- [`*mut T::wrapping_sub`]
- [`*mut T::write_bytes`]
- [`*mut T::write_unaligned`]
- [`*mut T::write_volatile`]
- [`*mut T::write`]
- [`Box::leak`]
- [`FromUtf8Error::as_bytes`]
- [`LocalKey::try_with`]
- [`Option::cloned`]
- [`btree_map::Entry::and_modify`]
- [`fs::read_to_string`]
- [`fs::read`]
- [`fs::write`]
- [`hash_map::Entry::and_modify`]
- [`iter::FusedIterator`]
- [`ops::RangeInclusive`]
- [`ops::RangeToInclusive`]
- [`process::id`]
- [`slice::rotate_left`]
- [`slice::rotate_right`]
- [`String::retain`]


Cargo
-----
- [Cargo will now output path to custom commands when `-v` is
  passed with `--list`][cargo/5041]
- [The Cargo binary version is now the same as the CrabLang version][cargo/5083]

Misc
----
- [The second edition of "The CrabLang Programming Language" book is now recommended
  over the first.][48404]

Compatibility Notes
-------------------

- [aliasing a `Fn` trait as `dyn` no longer works.][48481] E.g. the following
  syntax is now invalid.
  ```
  use std::ops::Fn as dyn;
  fn g(_: Box<dyn(std::fmt::Debug)>) {}
  ```
- [The result of dereferences are no longer promoted to `'static`.][47408]
  e.g.
  ```crablang
  fn main() {
      const PAIR: &(i32, i32) = &(0, 1);
      let _reversed_pair: &'static _ = &(PAIR.1, PAIR.0); // Doesn't work
  }
  ```
- [Deprecate `AsciiExt` trait in favor of inherent methods.][49109]
- [`".e0"` will now no longer parse as `0.0` and will instead cause
  an error.][48235]
- [Removed hoedown from crablangdoc.][48274]
- [Bounds on higher-kinded lifetimes a hard error.][48326]

[46882]: https://github.com/crablang/crablang/pull/46882
[47379]: https://github.com/crablang/crablang/pull/47379
[47408]: https://github.com/crablang/crablang/pull/47408
[47813]: https://github.com/crablang/crablang/pull/47813
[48056]: https://github.com/crablang/crablang/pull/48056
[48125]: https://github.com/crablang/crablang/pull/48125
[48235]: https://github.com/crablang/crablang/pull/48235
[48274]: https://github.com/crablang/crablang/pull/48274
[48281]: https://github.com/crablang/crablang/pull/48281
[48296]: https://github.com/crablang/crablang/pull/48296
[48326]: https://github.com/crablang/crablang/pull/48326
[48359]: https://github.com/crablang/crablang/pull/48359
[48404]: https://github.com/crablang/crablang/pull/48404
[48481]: https://github.com/crablang/crablang/pull/48481
[48516]: https://github.com/crablang/crablang/pull/48516
[48628]: https://github.com/crablang/crablang/pull/48628
[48629]: https://github.com/crablang/crablang/pull/48629
[48657]: https://github.com/crablang/crablang/pull/48657
[48735]: https://github.com/crablang/crablang/pull/48735
[48978]: https://github.com/crablang/crablang/pull/48978
[49101]: https://github.com/crablang/crablang/pull/49101
[49109]: https://github.com/crablang/crablang/pull/49109
[49162]: https://github.com/crablang/crablang/pull/49162
[49255]: https://github.com/crablang/crablang/pull/49255
[49299]: https://github.com/crablang/crablang/pull/49299
[49305]: https://github.com/crablang/crablang/pull/49305
[49394]: https://github.com/crablang/crablang/pull/49394
[49458]: https://github.com/crablang/crablang/pull/49458
[`*const T::add`]: https://doc.crablang.org/std/primitive.pointer.html#method.add
[`*const T::copy_to_nonoverlapping`]: https://doc.crablang.org/std/primitive.pointer.html#method.copy_to_nonoverlapping
[`*const T::copy_to`]: https://doc.crablang.org/std/primitive.pointer.html#method.copy_to
[`*const T::read_unaligned`]: https://doc.crablang.org/std/primitive.pointer.html#method.read_unaligned
[`*const T::read_volatile`]: https://doc.crablang.org/std/primitive.pointer.html#method.read_volatile
[`*const T::read`]: https://doc.crablang.org/std/primitive.pointer.html#method.read
[`*const T::sub`]: https://doc.crablang.org/std/primitive.pointer.html#method.sub
[`*const T::wrapping_add`]: https://doc.crablang.org/std/primitive.pointer.html#method.wrapping_add
[`*const T::wrapping_sub`]: https://doc.crablang.org/std/primitive.pointer.html#method.wrapping_sub
[`*mut T::add`]: https://doc.crablang.org/std/primitive.pointer.html#method.add-1
[`*mut T::copy_to_nonoverlapping`]: https://doc.crablang.org/std/primitive.pointer.html#method.copy_to_nonoverlapping-1
[`*mut T::copy_to`]: https://doc.crablang.org/std/primitive.pointer.html#method.copy_to-1
[`*mut T::read_unaligned`]: https://doc.crablang.org/std/primitive.pointer.html#method.read_unaligned-1
[`*mut T::read_volatile`]: https://doc.crablang.org/std/primitive.pointer.html#method.read_volatile-1
[`*mut T::read`]: https://doc.crablang.org/std/primitive.pointer.html#method.read-1
[`*mut T::replace`]: https://doc.crablang.org/std/primitive.pointer.html#method.replace
[`*mut T::sub`]: https://doc.crablang.org/std/primitive.pointer.html#method.sub-1
[`*mut T::swap`]: https://doc.crablang.org/std/primitive.pointer.html#method.swap
[`*mut T::wrapping_add`]: https://doc.crablang.org/std/primitive.pointer.html#method.wrapping_add-1
[`*mut T::wrapping_sub`]: https://doc.crablang.org/std/primitive.pointer.html#method.wrapping_sub-1
[`*mut T::write_bytes`]: https://doc.crablang.org/std/primitive.pointer.html#method.write_bytes
[`*mut T::write_unaligned`]: https://doc.crablang.org/std/primitive.pointer.html#method.write_unaligned
[`*mut T::write_volatile`]: https://doc.crablang.org/std/primitive.pointer.html#method.write_volatile
[`*mut T::write`]: https://doc.crablang.org/std/primitive.pointer.html#method.write
[`Box::leak`]: https://doc.crablang.org/std/boxed/struct.Box.html#method.leak
[`FromUtf8Error::as_bytes`]: https://doc.crablang.org/std/string/struct.FromUtf8Error.html#method.as_bytes
[`LocalKey::try_with`]: https://doc.crablang.org/std/thread/struct.LocalKey.html#method.try_with
[`Option::cloned`]: https://doc.crablang.org/std/option/enum.Option.html#method.cloned
[`btree_map::Entry::and_modify`]: https://doc.crablang.org/std/collections/btree_map/enum.Entry.html#method.and_modify
[`fs::read_to_string`]: https://doc.crablang.org/std/fs/fn.read_to_string.html
[`fs::read`]: https://doc.crablang.org/std/fs/fn.read.html
[`fs::write`]: https://doc.crablang.org/std/fs/fn.write.html
[`hash_map::Entry::and_modify`]: https://doc.crablang.org/std/collections/hash_map/enum.Entry.html#method.and_modify
[`iter::FusedIterator`]: https://doc.crablang.org/std/iter/trait.FusedIterator.html
[`ops::RangeInclusive`]: https://doc.crablang.org/std/ops/struct.RangeInclusive.html
[`ops::RangeToInclusive`]: https://doc.crablang.org/std/ops/struct.RangeToInclusive.html
[`process::id`]: https://doc.crablang.org/std/process/fn.id.html
[`slice::rotate_left`]: https://doc.crablang.org/std/primitive.slice.html#method.rotate_left
[`slice::rotate_right`]: https://doc.crablang.org/std/primitive.slice.html#method.rotate_right
[`String::retain`]: https://doc.crablang.org/std/string/struct.String.html#method.retain
[cargo/5041]: https://github.com/crablang/cargo/pull/5041
[cargo/5083]: https://github.com/crablang/cargo/pull/5083


Version 1.25.0 (2018-03-29)
==========================

Language
--------
- [The `#[repr(align(x))]` attribute is now stable.][47006] [RFC 1358]
- [You can now use nested groups of imports.][47948]
  e.g. `use std::{fs::File, io::Read, path::{Path, PathBuf}};`
- [You can now have `|` at the start of a match arm.][47947] e.g.
```crablang
enum Foo { A, B, C }

fn main() {
    let x = Foo::A;
    match x {
        | Foo::A
        | Foo::B => println!("AB"),
        | Foo::C => println!("C"),
    }
}
```

Compiler
--------
- [Upgraded to LLVM 6.][47828]
- [Added `-C lto=val` option.][47521]
- [Added `i586-unknown-linux-musl` target][47282]

Libraries
---------
- [Impl Send for `process::Command` on Unix.][47760]
- [Impl PartialEq and Eq for `ParseCharError`.][47790]
- [`UnsafeCell::into_inner` is now safe.][47204]
- [Implement libstd for CloudABI.][47268]
- [`Float::{from_bits, to_bits}` is now available in libcore.][46931]
- [Implement `AsRef<Path>` for Component][46985]
- [Implemented `Write` for `Cursor<&mut Vec<u8>>`][46830]
- [Moved `Duration` to libcore.][46666]

Stabilized APIs
---------------
- [`Location::column`]
- [`ptr::NonNull`]

The following functions can now be used in a constant expression.
eg. `static MINUTE: Duration = Duration::from_secs(60);`
- [`Duration::new`][47300]
- [`Duration::from_secs`][47300]
- [`Duration::from_millis`][47300]

Cargo
-----
- [`cargo new` no longer removes `crablang` or `rs` prefixs/suffixs.][cargo/5013]
- [`cargo new` now defaults to creating a binary crate, instead of a
  library crate.][cargo/5029]

Misc
----
- [CrabLang by example is now shipped with new releases][46196]

Compatibility Notes
-------------------
- [Deprecated `net::lookup_host`.][47510]
- [`crablangdoc` has switched to pulldown as the default markdown renderer.][47398]
- The borrow checker was sometimes incorrectly permitting overlapping borrows
  around indexing operations (see [#47349][47349]). This has been fixed (which also
  enabled some correct code that used to cause errors (e.g. [#33903][33903] and [#46095][46095]).
- [Removed deprecated unstable attribute `#[simd]`.][47251]

[33903]: https://github.com/crablang/crablang/pull/33903
[47947]: https://github.com/crablang/crablang/pull/47947
[47948]: https://github.com/crablang/crablang/pull/47948
[47760]: https://github.com/crablang/crablang/pull/47760
[47790]: https://github.com/crablang/crablang/pull/47790
[47828]: https://github.com/crablang/crablang/pull/47828
[47398]: https://github.com/crablang/crablang/pull/47398
[47510]: https://github.com/crablang/crablang/pull/47510
[47521]: https://github.com/crablang/crablang/pull/47521
[47204]: https://github.com/crablang/crablang/pull/47204
[47251]: https://github.com/crablang/crablang/pull/47251
[47268]: https://github.com/crablang/crablang/pull/47268
[47282]: https://github.com/crablang/crablang/pull/47282
[47300]: https://github.com/crablang/crablang/pull/47300
[47349]: https://github.com/crablang/crablang/pull/47349
[46931]: https://github.com/crablang/crablang/pull/46931
[46985]: https://github.com/crablang/crablang/pull/46985
[47006]: https://github.com/crablang/crablang/pull/47006
[46830]: https://github.com/crablang/crablang/pull/46830
[46095]: https://github.com/crablang/crablang/pull/46095
[46666]: https://github.com/crablang/crablang/pull/46666
[46196]: https://github.com/crablang/crablang/pull/46196
[cargo/5013]: https://github.com/crablang/cargo/pull/5013
[cargo/5029]: https://github.com/crablang/cargo/pull/5029
[RFC 1358]: https://github.com/crablang/rfcs/pull/1358
[`Location::column`]: https://doc.crablang.org/std/panic/struct.Location.html#method.column
[`ptr::NonNull`]: https://doc.crablang.org/std/ptr/struct.NonNull.html


Version 1.24.1 (2018-03-01)
==========================

 - [Do not abort when unwinding through FFI][48251]
 - [Emit UTF-16 files for linker arguments on Windows][48318]
 - [Make the error index generator work again][48308]
 - [Cargo will warn on Windows 7 if an update is needed][cargo/5069].

[48251]: https://github.com/crablang/crablang/issues/48251
[48308]: https://github.com/crablang/crablang/issues/48308
[48318]: https://github.com/crablang/crablang/issues/48318
[cargo/5069]: https://github.com/crablang/cargo/pull/5069


Version 1.24.0 (2018-02-15)
==========================

Language
--------
- [External `sysv64` ffi is now available.][46528]
  eg. `extern "sysv64" fn foo () {}`

Compiler
--------
- [crablangc now uses 16 codegen units by default for release builds.][46910]
  For the fastest builds, utilize `codegen-units=1`.
- [Added `armv4t-unknown-linux-gnueabi` target.][47018]
- [Add `aarch64-unknown-openbsd` support][46760]

Libraries
---------
- [`str::find::<char>` now uses memchr.][46735] This should lead to a 10x
  improvement in performance in the majority of cases.
- [`OsStr`'s `Debug` implementation is now lossless and consistent
  with Windows.][46798]
- [`time::{SystemTime, Instant}` now implement `Hash`.][46828]
- [impl `From<bool>` for `AtomicBool`][46293]
- [impl `From<{CString, &CStr}>` for `{Arc<CStr>, Rc<CStr>}`][45990]
- [impl `From<{OsString, &OsStr}>` for `{Arc<OsStr>, Rc<OsStr>}`][45990]
- [impl `From<{PathBuf, &Path}>` for `{Arc<Path>, Rc<Path>}`][45990]
- [float::from_bits now just uses transmute.][46012] This provides
  some optimisations from LLVM.
- [Copied `AsciiExt` methods onto `char`][46077]
- [Remove `T: Sized` requirement on `ptr::is_null()`][46094]
- [impl `From<RecvError>` for `{TryRecvError, RecvTimeoutError}`][45506]
- [Optimised `f32::{min, max}` to generate more efficient x86 assembly][47080]
- [`[u8]::contains` now uses memchr which provides a 3x speed improvement][46713]

Stabilized APIs
---------------
- [`RefCell::replace`]
- [`RefCell::swap`]
- [`atomic::spin_loop_hint`]

The following functions can now be used in a constant expression.
eg. `let buffer: [u8; size_of::<usize>()];`, `static COUNTER: AtomicUsize = AtomicUsize::new(1);`

- [`AtomicBool::new`][46287]
- [`AtomicUsize::new`][46287]
- [`AtomicIsize::new`][46287]
- [`AtomicPtr::new`][46287]
- [`Cell::new`][46287]
- [`{integer}::min_value`][46287]
- [`{integer}::max_value`][46287]
- [`mem::size_of`][46287]
- [`mem::align_of`][46287]
- [`ptr::null`][46287]
- [`ptr::null_mut`][46287]
- [`RefCell::new`][46287]
- [`UnsafeCell::new`][46287]

Cargo
-----
- [Added a `workspace.default-members` config that
  overrides implied `--all` in virtual workspaces.][cargo/4743]
- [Enable incremental by default on development builds.][cargo/4817] Also added
  configuration keys to `Cargo.toml` and `.cargo/config` to disable on a
  per-project or global basis respectively.

Misc
----

Compatibility Notes
-------------------
- [Floating point types `Debug` impl now always prints a decimal point.][46831]
- [`Ipv6Addr` now rejects superfluous `::`'s in IPv6 addresses][46671] This is
  in accordance with IETF RFC 4291 §2.2.
- [Unwinding will no longer go past FFI boundaries, and will instead abort.][46833]
- [`Formatter::flags` method is now deprecated.][46284] The `sign_plus`,
  `sign_minus`, `alternate`, and `sign_aware_zero_pad` should be used instead.
- [Leading zeros in tuple struct members is now an error][47084]
- [`column!()` macro is one-based instead of zero-based][46977]
- [`fmt::Arguments` can no longer be shared across threads][45198]
- [Access to `#[repr(packed)]` struct fields is now unsafe][44884]
- [Cargo sets a different working directory for the compiler][cargo/4788]

[44884]: https://github.com/crablang/crablang/pull/44884
[45198]: https://github.com/crablang/crablang/pull/45198
[45506]: https://github.com/crablang/crablang/pull/45506
[45990]: https://github.com/crablang/crablang/pull/45990
[46012]: https://github.com/crablang/crablang/pull/46012
[46077]: https://github.com/crablang/crablang/pull/46077
[46094]: https://github.com/crablang/crablang/pull/46094
[46284]: https://github.com/crablang/crablang/pull/46284
[46287]: https://github.com/crablang/crablang/pull/46287
[46293]: https://github.com/crablang/crablang/pull/46293
[46528]: https://github.com/crablang/crablang/pull/46528
[46671]: https://github.com/crablang/crablang/pull/46671
[46713]: https://github.com/crablang/crablang/pull/46713
[46735]: https://github.com/crablang/crablang/pull/46735
[46760]: https://github.com/crablang/crablang/pull/46760
[46798]: https://github.com/crablang/crablang/pull/46798
[46828]: https://github.com/crablang/crablang/pull/46828
[46831]: https://github.com/crablang/crablang/pull/46831
[46833]: https://github.com/crablang/crablang/pull/46833
[46910]: https://github.com/crablang/crablang/pull/46910
[46977]: https://github.com/crablang/crablang/pull/46977
[47018]: https://github.com/crablang/crablang/pull/47018
[47080]: https://github.com/crablang/crablang/pull/47080
[47084]: https://github.com/crablang/crablang/pull/47084
[cargo/4743]: https://github.com/crablang/cargo/pull/4743
[cargo/4788]: https://github.com/crablang/cargo/pull/4788
[cargo/4817]: https://github.com/crablang/cargo/pull/4817
[`RefCell::replace`]: https://doc.crablang.org/std/cell/struct.RefCell.html#method.replace
[`RefCell::swap`]: https://doc.crablang.org/std/cell/struct.RefCell.html#method.swap
[`atomic::spin_loop_hint`]: https://doc.crablang.org/std/sync/atomic/fn.spin_loop_hint.html


Version 1.23.0 (2018-01-04)
==========================

Language
--------
- [Arbitrary `auto` traits are now permitted in trait objects.][45772]
- [crablangc now uses subtyping on the left hand side of binary operations.][45435]
  Which should fix some confusing errors in some operations.

Compiler
--------
- [Enabled `TrapUnreachable` in LLVM which should mitigate the impact of
  undefined behavior.][45920]
- [crablangc now suggests renaming import if names clash.][45660]
- [Display errors/warnings correctly when there are zero-width or
  wide characters.][45711]
- [crablangc now avoids unnecessary copies of arguments that are
  simple bindings][45380] This should improve memory usage on average by 5-10%.
- [Updated musl used to build musl crablangc to 1.1.17][45393]

Libraries
---------
- [Allow a trailing comma in `assert_eq/ne` macro][45887]
- [Implement Hash for raw pointers to unsized types][45483]
- [impl `From<*mut T>` for `AtomicPtr<T>`][45610]
- [impl `From<usize/isize>` for `AtomicUsize/AtomicIsize`.][45610]
- [Removed the `T: Sync` requirement for `RwLock<T>: Send`][45267]
- [Removed `T: Sized` requirement for `{<*const T>, <*mut T>}::as_ref`
  and `<*mut T>::as_mut`][44932]
- [Optimized `Thread::{park, unpark}` implementation][45524]
- [Improved `SliceExt::binary_search` performance.][45333]
- [impl `FromIterator<()>` for `()`][45379]
- [Copied `AsciiExt` trait methods to primitive types.][44042] Use of `AsciiExt`
  is now deprecated.

Stabilized APIs
---------------

Cargo
-----
- [Cargo now supports uninstallation of multiple packages][cargo/4561]
  eg. `cargo uninstall foo bar` uninstalls `foo` and `bar`.
- [Added unit test checking to `cargo check`][cargo/4592]
- [Cargo now lets you install a specific version
  using `cargo install --version`][cargo/4637]

Misc
----
- [Releases now ship with the Cargo book documentation.][45692]
- [crablangdoc now prints rendering warnings on every run.][45324]

Compatibility Notes
-------------------
- [Changes have been made to type equality to make it more correct,
  in rare cases this could break some code.][45853] [Tracking issue for
  further information][45852]
- [`char::escape_debug` now uses Unicode 10 over 9.][45571]
- [Upgraded Android SDK to 27, and NDK to r15c.][45580] This drops support for
  Android 9, the minimum supported version is Android 14.
- [Bumped the minimum LLVM to 3.9][45326]

[44042]: https://github.com/crablang/crablang/pull/44042
[44932]: https://github.com/crablang/crablang/pull/44932
[45267]: https://github.com/crablang/crablang/pull/45267
[45324]: https://github.com/crablang/crablang/pull/45324
[45326]: https://github.com/crablang/crablang/pull/45326
[45333]: https://github.com/crablang/crablang/pull/45333
[45379]: https://github.com/crablang/crablang/pull/45379
[45380]: https://github.com/crablang/crablang/pull/45380
[45393]: https://github.com/crablang/crablang/pull/45393
[45435]: https://github.com/crablang/crablang/pull/45435
[45483]: https://github.com/crablang/crablang/pull/45483
[45524]: https://github.com/crablang/crablang/pull/45524
[45571]: https://github.com/crablang/crablang/pull/45571
[45580]: https://github.com/crablang/crablang/pull/45580
[45610]: https://github.com/crablang/crablang/pull/45610
[45660]: https://github.com/crablang/crablang/pull/45660
[45692]: https://github.com/crablang/crablang/pull/45692
[45711]: https://github.com/crablang/crablang/pull/45711
[45772]: https://github.com/crablang/crablang/pull/45772
[45852]: https://github.com/crablang/crablang/issues/45852
[45853]: https://github.com/crablang/crablang/pull/45853
[45887]: https://github.com/crablang/crablang/pull/45887
[45920]: https://github.com/crablang/crablang/pull/45920
[cargo/4561]: https://github.com/crablang/cargo/pull/4561
[cargo/4592]: https://github.com/crablang/cargo/pull/4592
[cargo/4637]: https://github.com/crablang/cargo/pull/4637


Version 1.22.1 (2017-11-22)
==========================

- [Update Cargo to fix an issue with macOS 10.13 "High Sierra"][46183]

[46183]: https://github.com/crablang/crablang/pull/46183

Version 1.22.0 (2017-11-22)
==========================

Language
--------
- [`non_snake_case` lint now allows extern no-mangle functions][44966]
- [Now accepts underscores in unicode escapes][43716]
- [`T op= &T` now works for numeric types.][44287] eg. `let mut x = 2; x += &8;`
- [types that impl `Drop` are now allowed in `const` and `static` types][44456]

Compiler
--------
- [crablangc now defaults to having 16 codegen units at debug on supported platforms.][45064]
- [crablangc will no longer inline in codegen units when compiling for debug][45075]
  This should decrease compile times for debug builds.
- [strict memory alignment now enabled on ARMv6][45094]
- [Remove support for the PNaCl target `le32-unknown-nacl`][45041]

Libraries
---------
- [Allow atomic operations up to 32 bits
  on `armv5te_unknown_linux_gnueabi`][44978]
- [`Box<Error>` now impls `From<Cow<str>>`][44466]
- [`std::mem::Discriminant` is now guaranteed to be `Send + Sync`][45095]
- [`fs::copy` now returns the length of the main stream on NTFS.][44895]
- [Properly detect overflow in `Instant += Duration`.][44220]
- [impl `Hasher` for `{&mut Hasher, Box<Hasher>}`][44015]
- [impl `fmt::Debug` for `SplitWhitespace`.][44303]
- [`Option<T>` now impls `Try`][42526] This allows for using `?` with `Option` types.

Stabilized APIs
---------------

Cargo
-----
- [Cargo will now build multi file examples in subdirectories of the `examples`
  folder that have a `main.rs` file.][cargo/4496]
- [Changed `[root]` to `[package]` in `Cargo.lock`][cargo/4571] Packages with
  the old format will continue to work and can be updated with `cargo update`.
- [Now supports vendoring git repositories][cargo/3992]

Misc
----
- [`libbacktrace` is now available on Apple platforms.][44251]
- [Stabilised the `compile_fail` attribute for code fences in doc-comments.][43949]
  This now lets you specify that a given code example will fail to compile.

Compatibility Notes
-------------------
- [The minimum Android version that crablangc can build for has been bumped
  to `4.0` from `2.3`][45656]
- [Allowing `T op= &T` for numeric types has broken some type
  inference cases][45480]


[42526]: https://github.com/crablang/crablang/pull/42526
[43716]: https://github.com/crablang/crablang/pull/43716
[43949]: https://github.com/crablang/crablang/pull/43949
[44015]: https://github.com/crablang/crablang/pull/44015
[44220]: https://github.com/crablang/crablang/pull/44220
[44251]: https://github.com/crablang/crablang/pull/44251
[44287]: https://github.com/crablang/crablang/pull/44287
[44303]: https://github.com/crablang/crablang/pull/44303
[44456]: https://github.com/crablang/crablang/pull/44456
[44466]: https://github.com/crablang/crablang/pull/44466
[44895]: https://github.com/crablang/crablang/pull/44895
[44966]: https://github.com/crablang/crablang/pull/44966
[44978]: https://github.com/crablang/crablang/pull/44978
[45041]: https://github.com/crablang/crablang/pull/45041
[45064]: https://github.com/crablang/crablang/pull/45064
[45075]: https://github.com/crablang/crablang/pull/45075
[45094]: https://github.com/crablang/crablang/pull/45094
[45095]: https://github.com/crablang/crablang/pull/45095
[45480]: https://github.com/crablang/crablang/issues/45480
[45656]: https://github.com/crablang/crablang/pull/45656
[cargo/3992]: https://github.com/crablang/cargo/pull/3992
[cargo/4496]: https://github.com/crablang/cargo/pull/4496
[cargo/4571]: https://github.com/crablang/cargo/pull/4571






Version 1.21.0 (2017-10-12)
==========================

Language
--------
- [You can now use static references for literals.][43838]
  Example:
  ```crablang
  fn main() {
      let x: &'static u32 = &0;
  }
  ```
- [Relaxed path syntax. Optional `::` before `<` is now allowed in all contexts.][43540]
  Example:
  ```crablang
  my_macro!(Vec<i32>::new); // Always worked
  my_macro!(Vec::<i32>::new); // Now works
  ```

Compiler
--------
- [Upgraded jemalloc to 4.5.0][43911]
- [Enabled unwinding panics on Redox][43917]
- [Now runs LLVM in parallel during translation phase.][43506]
  This should reduce peak memory usage.

Libraries
---------
- [Generate builtin impls for `Clone` for all arrays and tuples that
  are `T: Clone`][43690]
- [`Stdin`, `Stdout`, and `Stderr` now implement `AsRawFd`.][43459]
- [`Rc` and `Arc` now implement `From<&[T]> where T: Clone`, `From<str>`,
  `From<String>`, `From<Box<T>> where T: ?Sized`, and `From<Vec<T>>`.][42565]

Stabilized APIs
---------------

[`std::mem::discriminant`]

Cargo
-----
- [You can now call `cargo install` with multiple package names][cargo/4216]
- [Cargo commands inside a virtual workspace will now implicitly
  pass `--all`][cargo/4335]
- [Added a `[patch]` section to `Cargo.toml` to handle
  prepublication dependencies][cargo/4123] [RFC 1969]
- [`include` & `exclude` fields in `Cargo.toml` now accept gitignore
  like patterns][cargo/4270]
- [Added the `--all-targets` option][cargo/4400]
- [Using required dependencies as a feature is now deprecated and emits
  a warning][cargo/4364]


Misc
----
- [Cargo docs are moving][43916]
  to [doc.crablang.org/cargo](https://doc.crablang.org/cargo)
- [The crablangdoc book is now available][43863]
  at [doc.crablang.org/crablangdoc](https://doc.crablang.org/crablangdoc)
- [Added a preview of RLS has been made available through crablangup][44204]
  Install with `crablangup component add rls-preview`
- [`std::os` documentation for Unix, Linux, and Windows now appears on doc.crablang.org][43348]
  Previously only showed `std::os::unix`.

Compatibility Notes
-------------------
- [Changes in method matching against higher-ranked types][43880] This may cause
  breakage in subtyping corner cases. [A more in-depth explanation is available.][info/43880]
- [crablangc's JSON error output's byte position start at top of file.][42973]
  Was previously relative to the crablangc's internal `CodeMap` struct which
  required the unstable library `libsyntax` to correctly use.
- [`unused_results` lint no longer ignores booleans][43728]

[42565]: https://github.com/crablang/crablang/pull/42565
[42973]: https://github.com/crablang/crablang/pull/42973
[43348]: https://github.com/crablang/crablang/pull/43348
[43459]: https://github.com/crablang/crablang/pull/43459
[43506]: https://github.com/crablang/crablang/pull/43506
[43540]: https://github.com/crablang/crablang/pull/43540
[43690]: https://github.com/crablang/crablang/pull/43690
[43728]: https://github.com/crablang/crablang/pull/43728
[43838]: https://github.com/crablang/crablang/pull/43838
[43863]: https://github.com/crablang/crablang/pull/43863
[43880]: https://github.com/crablang/crablang/pull/43880
[43911]: https://github.com/crablang/crablang/pull/43911
[43916]: https://github.com/crablang/crablang/pull/43916
[43917]: https://github.com/crablang/crablang/pull/43917
[44204]: https://github.com/crablang/crablang/pull/44204
[cargo/4123]: https://github.com/crablang/cargo/pull/4123
[cargo/4216]: https://github.com/crablang/cargo/pull/4216
[cargo/4270]: https://github.com/crablang/cargo/pull/4270
[cargo/4335]: https://github.com/crablang/cargo/pull/4335
[cargo/4364]: https://github.com/crablang/cargo/pull/4364
[cargo/4400]: https://github.com/crablang/cargo/pull/4400
[RFC 1969]: https://github.com/crablang/rfcs/pull/1969
[info/43880]: https://github.com/crablang/crablang/issues/44224#issuecomment-330058902
[`std::mem::discriminant`]: https://doc.crablang.org/std/mem/fn.discriminant.html

Version 1.20.0 (2017-08-31)
===========================

Language
--------
- [Associated constants are now stabilised.][42809]
- [A lot of macro bugs are now fixed.][42913]

Compiler
--------

- [Struct fields are now properly coerced to the expected field type.][42807]
- [Enabled wasm LLVM backend][42571] WASM can now be built with the
  `wasm32-experimental-emscripten` target.
- [Changed some of the error messages to be more helpful.][42033]
- [Add support for RELRO(RELocation Read-Only) for platforms that support
  it.][43170]
- [crablangc now reports the total number of errors on compilation failure][43015]
  previously this was only the number of errors in the pass that failed.
- [Expansion in crablangc has been sped up 29x.][42533]
- [added `msp430-none-elf` target.][43099]
- [crablangc will now suggest one-argument enum variant to fix type mismatch when
  applicable][43178]
- [Fixes backtraces on Redox][43228]
- [crablangc now identifies different versions of same crate when absolute paths of
  different types match in an error message.][42826]

Libraries
---------


- [Relaxed Debug constraints on `{HashMap,BTreeMap}::{Keys,Values}`.][42854]
- [Impl `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Debug`, `Hash` for unsized
  tuples.][43011]
- [Impl `fmt::{Display, Debug}` for `Ref`, `RefMut`, `MutexGuard`,
  `RwLockReadGuard`, `RwLockWriteGuard`][42822]
- [Impl `Clone` for `DefaultHasher`.][42799]
- [Impl `Sync` for `SyncSender`.][42397]
- [Impl `FromStr` for `char`][42271]
- [Fixed how `{f32, f64}::{is_sign_negative, is_sign_positive}` handles
  NaN.][42431]
- [allow messages in the `unimplemented!()` macro.][42155]
  ie. `unimplemented!("Waiting for 1.21 to be stable")`
- [`pub(restricted)` is now supported in the `thread_local!` macro.][43185]
- [Upgrade to Unicode 10.0.0][42999]
- [Reimplemented `{f32, f64}::{min, max}` in CrabLang instead of using CMath.][42430]
- [Skip the main thread's manual stack guard on Linux][43072]
- [Iterator::nth for `ops::{Range, RangeFrom}` is now done in *O*(1) time][43077]
- [`#[repr(align(N))]` attribute max number is now 2^31 - 1.][43097] This was
  previously 2^15.
- [`{OsStr, Path}::Display` now avoids allocations where possible][42613]

Stabilized APIs
---------------

- [`CStr::into_c_string`]
- [`CString::as_c_str`]
- [`CString::into_boxed_c_str`]
- [`Chain::get_mut`]
- [`Chain::get_ref`]
- [`Chain::into_inner`]
- [`Option::get_or_insert_with`]
- [`Option::get_or_insert`]
- [`OsStr::into_os_string`]
- [`OsString::into_boxed_os_str`]
- [`Take::get_mut`]
- [`Take::get_ref`]
- [`Utf8Error::error_len`]
- [`char::EscapeDebug`]
- [`char::escape_debug`]
- [`compile_error!`]
- [`f32::from_bits`]
- [`f32::to_bits`]
- [`f64::from_bits`]
- [`f64::to_bits`]
- [`mem::ManuallyDrop`]
- [`slice::sort_unstable_by_key`]
- [`slice::sort_unstable_by`]
- [`slice::sort_unstable`]
- [`str::from_boxed_utf8_unchecked`]
- [`str::as_bytes_mut`]
- [`str::as_bytes_mut`]
- [`str::from_utf8_mut`]
- [`str::from_utf8_unchecked_mut`]
- [`str::get_mut`]
- [`str::get_unchecked_mut`]
- [`str::get_unchecked`]
- [`str::get`]
- [`str::into_boxed_bytes`]


Cargo
-----
- [Cargo API token location moved from `~/.cargo/config` to
  `~/.cargo/credentials`.][cargo/3978]
- [Cargo will now build `main.rs` binaries that are in sub-directories of
  `src/bin`.][cargo/4214] ie. Having `src/bin/server/main.rs` and
  `src/bin/client/main.rs` generates `target/debug/server` and `target/debug/client`
- [You can now specify version of a binary when installed through
  `cargo install` using `--vers`.][cargo/4229]
- [Added `--no-fail-fast` flag to cargo to run all benchmarks regardless of
  failure.][cargo/4248]
- [Changed the convention around which file is the crate root.][cargo/4259]

Compatibility Notes
-------------------

- [Functions with `'static` in their return types will now not be as usable as
  if they were using lifetime parameters instead.][42417]
- [The reimplementation of `{f32, f64}::is_sign_{negative, positive}` now
  takes the sign of NaN into account where previously didn't.][42430]

[42033]: https://github.com/crablang/crablang/pull/42033
[42155]: https://github.com/crablang/crablang/pull/42155
[42271]: https://github.com/crablang/crablang/pull/42271
[42397]: https://github.com/crablang/crablang/pull/42397
[42417]: https://github.com/crablang/crablang/pull/42417
[42430]: https://github.com/crablang/crablang/pull/42430
[42431]: https://github.com/crablang/crablang/pull/42431
[42533]: https://github.com/crablang/crablang/pull/42533
[42571]: https://github.com/crablang/crablang/pull/42571
[42613]: https://github.com/crablang/crablang/pull/42613
[42799]: https://github.com/crablang/crablang/pull/42799
[42807]: https://github.com/crablang/crablang/pull/42807
[42809]: https://github.com/crablang/crablang/pull/42809
[42822]: https://github.com/crablang/crablang/pull/42822
[42826]: https://github.com/crablang/crablang/pull/42826
[42854]: https://github.com/crablang/crablang/pull/42854
[42913]: https://github.com/crablang/crablang/pull/42913
[42999]: https://github.com/crablang/crablang/pull/42999
[43011]: https://github.com/crablang/crablang/pull/43011
[43015]: https://github.com/crablang/crablang/pull/43015
[43072]: https://github.com/crablang/crablang/pull/43072
[43077]: https://github.com/crablang/crablang/pull/43077
[43097]: https://github.com/crablang/crablang/pull/43097
[43099]: https://github.com/crablang/crablang/pull/43099
[43170]: https://github.com/crablang/crablang/pull/43170
[43178]: https://github.com/crablang/crablang/pull/43178
[43185]: https://github.com/crablang/crablang/pull/43185
[43228]: https://github.com/crablang/crablang/pull/43228
[cargo/3978]: https://github.com/crablang/cargo/pull/3978
[cargo/4214]: https://github.com/crablang/cargo/pull/4214
[cargo/4229]: https://github.com/crablang/cargo/pull/4229
[cargo/4248]: https://github.com/crablang/cargo/pull/4248
[cargo/4259]: https://github.com/crablang/cargo/pull/4259
[`CStr::into_c_string`]: https://doc.crablang.org/std/ffi/struct.CStr.html#method.into_c_string
[`CString::as_c_str`]: https://doc.crablang.org/std/ffi/struct.CString.html#method.as_c_str
[`CString::into_boxed_c_str`]: https://doc.crablang.org/std/ffi/struct.CString.html#method.into_boxed_c_str
[`Chain::get_mut`]: https://doc.crablang.org/std/io/struct.Chain.html#method.get_mut
[`Chain::get_ref`]: https://doc.crablang.org/std/io/struct.Chain.html#method.get_ref
[`Chain::into_inner`]: https://doc.crablang.org/std/io/struct.Chain.html#method.into_inner
[`Option::get_or_insert_with`]: https://doc.crablang.org/std/option/enum.Option.html#method.get_or_insert_with
[`Option::get_or_insert`]: https://doc.crablang.org/std/option/enum.Option.html#method.get_or_insert
[`OsStr::into_os_string`]: https://doc.crablang.org/std/ffi/struct.OsStr.html#method.into_os_string
[`OsString::into_boxed_os_str`]: https://doc.crablang.org/std/ffi/struct.OsString.html#method.into_boxed_os_str
[`Take::get_mut`]: https://doc.crablang.org/std/io/struct.Take.html#method.get_mut
[`Take::get_ref`]: https://doc.crablang.org/std/io/struct.Take.html#method.get_ref
[`Utf8Error::error_len`]: https://doc.crablang.org/std/str/struct.Utf8Error.html#method.error_len
[`char::EscapeDebug`]: https://doc.crablang.org/std/char/struct.EscapeDebug.html
[`char::escape_debug`]: https://doc.crablang.org/std/primitive.char.html#method.escape_debug
[`compile_error!`]: https://doc.crablang.org/std/macro.compile_error.html
[`f32::from_bits`]: https://doc.crablang.org/std/primitive.f32.html#method.from_bits
[`f32::to_bits`]: https://doc.crablang.org/std/primitive.f32.html#method.to_bits
[`f64::from_bits`]: https://doc.crablang.org/std/primitive.f64.html#method.from_bits
[`f64::to_bits`]: https://doc.crablang.org/std/primitive.f64.html#method.to_bits
[`mem::ManuallyDrop`]: https://doc.crablang.org/std/mem/union.ManuallyDrop.html
[`slice::sort_unstable_by_key`]: https://doc.crablang.org/std/primitive.slice.html#method.sort_unstable_by_key
[`slice::sort_unstable_by`]: https://doc.crablang.org/std/primitive.slice.html#method.sort_unstable_by
[`slice::sort_unstable`]: https://doc.crablang.org/std/primitive.slice.html#method.sort_unstable
[`str::from_boxed_utf8_unchecked`]: https://doc.crablang.org/std/str/fn.from_boxed_utf8_unchecked.html
[`str::as_bytes_mut`]: https://doc.crablang.org/std/primitive.str.html#method.as_bytes_mut
[`str::from_utf8_mut`]: https://doc.crablang.org/std/str/fn.from_utf8_mut.html
[`str::from_utf8_unchecked_mut`]: https://doc.crablang.org/std/str/fn.from_utf8_unchecked_mut.html
[`str::get_mut`]: https://doc.crablang.org/std/primitive.str.html#method.get_mut
[`str::get_unchecked_mut`]: https://doc.crablang.org/std/primitive.str.html#method.get_unchecked_mut
[`str::get_unchecked`]: https://doc.crablang.org/std/primitive.str.html#method.get_unchecked
[`str::get`]: https://doc.crablang.org/std/primitive.str.html#method.get
[`str::into_boxed_bytes`]: https://doc.crablang.org/std/primitive.str.html#method.into_boxed_bytes


Version 1.19.0 (2017-07-20)
===========================

Language
--------

- [Numeric fields can now be used for creating tuple structs.][41145] [RFC 1506]
  For example `struct Point(u32, u32); let x = Point { 0: 7, 1: 0 };`.
- [Macro recursion limit increased to 1024 from 64.][41676]
- [Added lint for detecting unused macros.][41907]
- [`loop` can now return a value with `break`.][42016] [RFC 1624]
  For example: `let x = loop { break 7; };`
- [C compatible `union`s are now available.][42068] [RFC 1444] They can only
  contain `Copy` types and cannot have a `Drop` implementation.
  Example: `union Foo { bar: u8, baz: usize }`
- [Non capturing closures can now be coerced into `fn`s,][42162] [RFC 1558]
  Example: `let foo: fn(u8) -> u8 = |v: u8| { v };`

Compiler
--------

- [Add support for bootstrapping the CrabLang compiler toolchain on Android.][41370]
- [Change `arm-linux-androideabi` to correspond to the `armeabi`
  official ABI.][41656] If you wish to continue targeting the `armeabi-v7a` ABI
  you should use `--target armv7-linux-androideabi`.
- [Fixed ICE when removing a source file between compilation sessions.][41873]
- [Minor optimisation of string operations.][42037]
- [Compiler error message is now `aborting due to previous error(s)` instead of
  `aborting due to N previous errors`][42150] This was previously inaccurate and
  would only count certain kinds of errors.
- [The compiler now supports Visual Studio 2017][42225]
- [The compiler is now built against LLVM 4.0.1 by default][42948]
- [Added a lot][42264] of [new error codes][42302]
- [Added `target-feature=+crt-static` option][37406] [RFC 1721] Which allows
  libraries with C Run-time Libraries(CRT) to be statically linked.
- [Fixed various ARM codegen bugs][42740]

Libraries
---------

- [`String` now implements `FromIterator<Cow<'a, str>>` and
  `Extend<Cow<'a, str>>`][41449]
- [`Vec` now implements `From<&mut [T]>`][41530]
- [`Box<[u8]>` now implements `From<Box<str>>`][41258]
- [`SplitWhitespace` now implements `Clone`][41659]
- [`[u8]::reverse` is now 5x faster and `[u16]::reverse` is now
  1.5x faster][41764]
- [`eprint!` and `eprintln!` macros added to prelude.][41192] Same as the `print!`
  macros, but for printing to stderr.

Stabilized APIs
---------------

- [`OsString::shrink_to_fit`]
- [`cmp::Reverse`]
- [`Command::envs`]
- [`thread::ThreadId`]

Cargo
-----

- [Build scripts can now add environment variables to the environment
  the crate is being compiled in.
  Example: `println!("cargo:crablangc-env=FOO=bar");`][cargo/3929]
- [Subcommands now replace the current process rather than spawning a new
  child process][cargo/3970]
- [Workspace members can now accept glob file patterns][cargo/3979]
- [Added `--all` flag to the `cargo bench` subcommand to run benchmarks of all
  the members in a given workspace.][cargo/3988]
- [Updated `libssh2-sys` to 0.2.6][cargo/4008]
- [Target directory path is now in the cargo metadata][cargo/4022]
- [Cargo no longer checks out a local working directory for the
  crates.io index][cargo/4026] This should provide smaller file size for the
  registry, and improve cloning times, especially on Windows machines.
- [Added an `--exclude` option for excluding certain packages when using the
  `--all` option][cargo/4031]
- [Cargo will now automatically retry when receiving a 5xx error
  from crates.io][cargo/4032]
- [The `--features` option now accepts multiple comma or space
  delimited values.][cargo/4084]
- [Added support for custom target specific runners][cargo/3954]

Misc
----

- [Added `crablang-windbg.cmd`][39983] for loading crablang `.natvis` files in the
  Windows Debugger.
- [CrabLang will now release XZ compressed packages][crablang-installer/57]
- [crablangup will now prefer to download crablang packages with
  XZ compression][crablangup/1100] over GZip packages.
- [Added the ability to escape `#` in crablang documentation][41785] By adding
  additional `#`'s ie. `##` is now `#`

Compatibility Notes
-------------------

- [`MutexGuard<T>` may only be `Sync` if `T` is `Sync`.][41624]
- [`-Z` flags are now no longer allowed to be used on the stable
  compiler.][41751] This has been a warning for a year previous to this.
- [As a result of the `-Z` flag change, the `cargo-check` plugin no
  longer works][42844]. Users should migrate to the built-in `check`
  command, which has been available since 1.16.
- [Ending a float literal with `._` is now a hard error.
  Example: `42._` .][41946]
- [Any use of a private `extern crate` outside of its module is now a
  hard error.][36886] This was previously a warning.
- [`use ::self::foo;` is now a hard error.][36888] `self` paths are always
  relative while the `::` prefix makes a path absolute, but was ignored and the
  path was relative regardless.
- [Floating point constants in match patterns is now a hard error][36890]
  This was previously a warning.
- [Struct or enum constants that don't derive `PartialEq` & `Eq` used
  match patterns is now a hard error][36891] This was previously a warning.
- [Lifetimes named `'_` are no longer allowed.][36892] This was previously
  a warning.
- [From the pound escape, lines consisting of multiple `#`s are
  now visible][41785]
- [It is an error to re-export private enum variants][42460]. This is
  known to break a number of crates that depend on an older version of
  mustache.
- [On Windows, if `VCINSTALLDIR` is set incorrectly, `crablangc` will try
  to use it to find the linker, and the build will fail where it did
  not previously][42607]

[36886]: https://github.com/crablang/crablang/issues/36886
[36888]: https://github.com/crablang/crablang/issues/36888
[36890]: https://github.com/crablang/crablang/issues/36890
[36891]: https://github.com/crablang/crablang/issues/36891
[36892]: https://github.com/crablang/crablang/issues/36892
[37406]: https://github.com/crablang/crablang/issues/37406
[39983]: https://github.com/crablang/crablang/pull/39983
[41145]: https://github.com/crablang/crablang/pull/41145
[41192]: https://github.com/crablang/crablang/pull/41192
[41258]: https://github.com/crablang/crablang/pull/41258
[41370]: https://github.com/crablang/crablang/pull/41370
[41449]: https://github.com/crablang/crablang/pull/41449
[41530]: https://github.com/crablang/crablang/pull/41530
[41624]: https://github.com/crablang/crablang/pull/41624
[41656]: https://github.com/crablang/crablang/pull/41656
[41659]: https://github.com/crablang/crablang/pull/41659
[41676]: https://github.com/crablang/crablang/pull/41676
[41751]: https://github.com/crablang/crablang/pull/41751
[41764]: https://github.com/crablang/crablang/pull/41764
[41785]: https://github.com/crablang/crablang/pull/41785
[41873]: https://github.com/crablang/crablang/pull/41873
[41907]: https://github.com/crablang/crablang/pull/41907
[41946]: https://github.com/crablang/crablang/pull/41946
[42016]: https://github.com/crablang/crablang/pull/42016
[42037]: https://github.com/crablang/crablang/pull/42037
[42068]: https://github.com/crablang/crablang/pull/42068
[42150]: https://github.com/crablang/crablang/pull/42150
[42162]: https://github.com/crablang/crablang/pull/42162
[42225]: https://github.com/crablang/crablang/pull/42225
[42264]: https://github.com/crablang/crablang/pull/42264
[42302]: https://github.com/crablang/crablang/pull/42302
[42460]: https://github.com/crablang/crablang/issues/42460
[42607]: https://github.com/crablang/crablang/issues/42607
[42740]: https://github.com/crablang/crablang/pull/42740
[42844]: https://github.com/crablang/crablang/issues/42844
[42948]: https://github.com/crablang/crablang/pull/42948
[RFC 1444]: https://github.com/crablang/rfcs/pull/1444
[RFC 1506]: https://github.com/crablang/rfcs/pull/1506
[RFC 1558]: https://github.com/crablang/rfcs/pull/1558
[RFC 1624]: https://github.com/crablang/rfcs/pull/1624
[RFC 1721]: https://github.com/crablang/rfcs/pull/1721
[`Command::envs`]: https://doc.crablang.org/std/process/struct.Command.html#method.envs
[`OsString::shrink_to_fit`]: https://doc.crablang.org/std/ffi/struct.OsString.html#method.shrink_to_fit
[`cmp::Reverse`]: https://doc.crablang.org/std/cmp/struct.Reverse.html
[`thread::ThreadId`]: https://doc.crablang.org/std/thread/struct.ThreadId.html
[cargo/3929]: https://github.com/crablang/cargo/pull/3929
[cargo/3954]: https://github.com/crablang/cargo/pull/3954
[cargo/3970]: https://github.com/crablang/cargo/pull/3970
[cargo/3979]: https://github.com/crablang/cargo/pull/3979
[cargo/3988]: https://github.com/crablang/cargo/pull/3988
[cargo/4008]: https://github.com/crablang/cargo/pull/4008
[cargo/4022]: https://github.com/crablang/cargo/pull/4022
[cargo/4026]: https://github.com/crablang/cargo/pull/4026
[cargo/4031]: https://github.com/crablang/cargo/pull/4031
[cargo/4032]: https://github.com/crablang/cargo/pull/4032
[cargo/4084]: https://github.com/crablang/cargo/pull/4084
[crablang-installer/57]: https://github.com/crablang/crablang-installer/pull/57
[crablangup/1100]: https://github.com/crablang-nursery/crablangup.rs/pull/1100


Version 1.18.0 (2017-06-08)
===========================

Language
--------

- [Stabilize pub(restricted)][40556] `pub` can now accept a module path to
  make the item visible to just that module tree. Also accepts the keyword
  `crate` to make something public to the whole crate but not users of the
  library. Example: `pub(crate) mod utils;`. [RFC 1422].
- [Stabilize `#![windows_subsystem]` attribute][40870] conservative exposure of the
  `/SUBSYSTEM` linker flag on Windows platforms. [RFC 1665].
- [Refactor of trait object type parsing][40043] Now `ty` in macros can accept
  types like `Write + Send`, trailing `+` are now supported in trait objects,
  and better error reporting for trait objects starting with `?Sized`.
- [0e+10 is now a valid floating point literal][40589]
- [Now warns if you bind a lifetime parameter to 'static][40734]
- [Tuples, Enum variant fields, and structs with no `repr` attribute or with
  `#[repr(CrabLang)]` are reordered to minimize padding and produce a smaller
  representation in some cases.][40377]

Compiler
--------

- [crablangc can now emit mir with `--emit mir`][39891]
- [Improved LLVM IR for trivial functions][40367]
- [Added explanation for E0090(Wrong number of lifetimes are supplied)][40723]
- [crablangc compilation is now 15%-20% faster][41469] Thanks to optimisation
  opportunities found through profiling
- [Improved backtrace formatting when panicking][38165]

Libraries
---------

- [Specialized `Vec::from_iter` being passed `vec::IntoIter`][40731] if the
  iterator hasn't been advanced the original `Vec` is reassembled with no actual
  iteration or reallocation.
- [Simplified HashMap Bucket interface][40561] provides performance
  improvements for iterating and cloning.
- [Specialize Vec::from_elem to use calloc][40409]
- [Fixed Race condition in fs::create_dir_all][39799]
- [No longer caching stdio on Windows][40516]
- [Optimized insertion sort in slice][40807] insertion sort in some cases
  2.50%~ faster and in one case now 12.50% faster.
- [Optimized `AtomicBool::fetch_nand`][41143]

Stabilized APIs
---------------

- [`Child::try_wait`]
- [`HashMap::retain`]
- [`HashSet::retain`]
- [`PeekMut::pop`]
- [`TcpStream::peek`]
- [`UdpSocket::peek`]
- [`UdpSocket::peek_from`]

Cargo
-----

- [Added partial Pijul support][cargo/3842] Pijul is a version control system in CrabLang.
  You can now create new cargo projects with Pijul using `cargo new --vcs pijul`
- [Now always emits build script warnings for crates that fail to build][cargo/3847]
- [Added Android build support][cargo/3885]
- [Added `--bins` and `--tests` flags][cargo/3901] now you can build all programs
  of a certain type, for example `cargo build --bins` will build all
  binaries.
- [Added support for haiku][cargo/3952]

Misc
----

- [crablangdoc can now use pulldown-cmark with the `--enable-commonmark` flag][40338]
- [CrabLang now uses the official cross compiler for NetBSD][40612]
- [crablangdoc now accepts `#` at the start of files][40828]
- [Fixed jemalloc support for musl][41168]

Compatibility Notes
-------------------

- [Changes to how the `0` flag works in format!][40241] Padding zeroes are now
  always placed after the sign if it exists and before the digits. With the `#`
  flag the zeroes are placed after the prefix and before the digits.
- [Due to the struct field optimisation][40377], using `transmute` on structs
  that have no `repr` attribute or `#[repr(CrabLang)]` will no longer work. This has
  always been undefined behavior, but is now more likely to break in practice.
- [The refactor of trait object type parsing][40043] fixed a bug where `+` was
  receiving the wrong priority parsing things like `&for<'a> Tr<'a> + Send` as
  `&(for<'a> Tr<'a> + Send)` instead of `(&for<'a> Tr<'a>) + Send`
- [Overlapping inherent `impl`s are now a hard error][40728]
- [`PartialOrd` and `Ord` must agree on the ordering.][41270]
- [`crablangc main.rs -o out --emit=asm,llvm-ir`][41085] Now will output
  `out.asm` and `out.ll` instead of only one of the filetypes.
- [ calling a function that returns `Self` will no longer work][41805] when
  the size of `Self` cannot be statically determined.
- [crablangc now builds with a "pthreads" flavour of MinGW for Windows GNU][40805]
  this has caused a few regressions namely:

  - Changed the link order of local static/dynamic libraries (respecting the
    order on given rather than having the compiler reorder).
  - Changed how MinGW is linked, native code linked to dynamic libraries
    may require manually linking to the gcc support library (for the native
    code itself)

[38165]: https://github.com/crablang/crablang/pull/38165
[39799]: https://github.com/crablang/crablang/pull/39799
[39891]: https://github.com/crablang/crablang/pull/39891
[40043]: https://github.com/crablang/crablang/pull/40043
[40241]: https://github.com/crablang/crablang/pull/40241
[40338]: https://github.com/crablang/crablang/pull/40338
[40367]: https://github.com/crablang/crablang/pull/40367
[40377]: https://github.com/crablang/crablang/pull/40377
[40409]: https://github.com/crablang/crablang/pull/40409
[40516]: https://github.com/crablang/crablang/pull/40516
[40556]: https://github.com/crablang/crablang/pull/40556
[40561]: https://github.com/crablang/crablang/pull/40561
[40589]: https://github.com/crablang/crablang/pull/40589
[40612]: https://github.com/crablang/crablang/pull/40612
[40723]: https://github.com/crablang/crablang/pull/40723
[40728]: https://github.com/crablang/crablang/pull/40728
[40731]: https://github.com/crablang/crablang/pull/40731
[40734]: https://github.com/crablang/crablang/pull/40734
[40805]: https://github.com/crablang/crablang/pull/40805
[40807]: https://github.com/crablang/crablang/pull/40807
[40828]: https://github.com/crablang/crablang/pull/40828
[40870]: https://github.com/crablang/crablang/pull/40870
[41085]: https://github.com/crablang/crablang/pull/41085
[41143]: https://github.com/crablang/crablang/pull/41143
[41168]: https://github.com/crablang/crablang/pull/41168
[41270]: https://github.com/crablang/crablang/issues/41270
[41469]: https://github.com/crablang/crablang/pull/41469
[41805]: https://github.com/crablang/crablang/issues/41805
[RFC 1422]: https://github.com/crablang/rfcs/blob/master/text/1422-pub-restricted.md
[RFC 1665]: https://github.com/crablang/rfcs/blob/master/text/1665-windows-subsystem.md
[`Child::try_wait`]: https://doc.crablang.org/std/process/struct.Child.html#method.try_wait
[`HashMap::retain`]: https://doc.crablang.org/std/collections/struct.HashMap.html#method.retain
[`HashSet::retain`]: https://doc.crablang.org/std/collections/struct.HashSet.html#method.retain
[`PeekMut::pop`]: https://doc.crablang.org/std/collections/binary_heap/struct.PeekMut.html#method.pop
[`TcpStream::peek`]: https://doc.crablang.org/std/net/struct.TcpStream.html#method.peek
[`UdpSocket::peek_from`]: https://doc.crablang.org/std/net/struct.UdpSocket.html#method.peek_from
[`UdpSocket::peek`]: https://doc.crablang.org/std/net/struct.UdpSocket.html#method.peek
[cargo/3842]: https://github.com/crablang/cargo/pull/3842
[cargo/3847]: https://github.com/crablang/cargo/pull/3847
[cargo/3885]: https://github.com/crablang/cargo/pull/3885
[cargo/3901]: https://github.com/crablang/cargo/pull/3901
[cargo/3952]: https://github.com/crablang/cargo/pull/3952


Version 1.17.0 (2017-04-27)
===========================

Language
--------

* [The lifetime of statics and consts defaults to `'static`][39265]. [RFC 1623]
* [Fields of structs may be initialized without duplicating the field/variable
  names][39761]. [RFC 1682]
* [`Self` may be included in the `where` clause of `impls`][38864]. [RFC 1647]
* [When coercing to an unsized type lifetimes must be equal][40319]. That is,
  there is no subtyping between `T` and `U` when `T: Unsize<U>`. For example,
  coercing `&mut [&'a X; N]` to `&mut [&'b X]` requires `'a` be equal to
  `'b`. Soundness fix.
* [Values passed to the indexing operator, `[]`, automatically coerce][40166]
* [Static variables may contain references to other statics][40027]

Compiler
--------

* [Exit quickly on only `--emit dep-info`][40336]
* [Make `-C relocation-model` more correctly determine whether the linker
  creates a position-independent executable][40245]
* [Add `-C overflow-checks` to directly control whether integer overflow
  panics][40037]
* [The crablangc type checker now checks items on demand instead of in a single
  in-order pass][40008]. This is mostly an internal refactoring in support of
  future work, including incremental type checking, but also resolves [RFC
  1647], allowing `Self` to appear in `impl` `where` clauses.
* [Optimize vtable loads][39995]
* [Turn off vectorization for Emscripten targets][39990]
* [Provide suggestions for unknown macros imported with `use`][39953]
* [Fix ICEs in path resolution][39939]
* [Strip exception handling code on Emscripten when `panic=abort`][39193]
* [Add clearer error message using `&str + &str`][39116]

Stabilized APIs
---------------

* [`Arc::into_raw`]
* [`Arc::from_raw`]
* [`Arc::ptr_eq`]
* [`Rc::into_raw`]
* [`Rc::from_raw`]
* [`Rc::ptr_eq`]
* [`Ordering::then`]
* [`Ordering::then_with`]
* [`BTreeMap::range`]
* [`BTreeMap::range_mut`]
* [`collections::Bound`]
* [`process::abort`]
* [`ptr::read_unaligned`]
* [`ptr::write_unaligned`]
* [`Result::expect_err`]
* [`Cell::swap`]
* [`Cell::replace`]
* [`Cell::into_inner`]
* [`Cell::take`]

Libraries
---------

* [`BTreeMap` and `BTreeSet` can iterate over ranges][27787]
* [`Cell` can store non-`Copy` types][39793]. [RFC 1651]
* [`String` implements `FromIterator<&char>`][40028]
* `Box` [implements][40009] a number of new conversions:
  `From<Box<str>> for String`,
  `From<Box<[T]>> for Vec<T>`,
  `From<Box<CStr>> for CString`,
  `From<Box<OsStr>> for OsString`,
  `From<Box<Path>> for PathBuf`,
  `Into<Box<str>> for String`,
  `Into<Box<[T]>> for Vec<T>`,
  `Into<Box<CStr>> for CString`,
  `Into<Box<OsStr>> for OsString`,
  `Into<Box<Path>> for PathBuf`,
  `Default for Box<str>`,
  `Default for Box<CStr>`,
  `Default for Box<OsStr>`,
  `From<&CStr> for Box<CStr>`,
  `From<&OsStr> for Box<OsStr>`,
  `From<&Path> for Box<Path>`
* [`ffi::FromBytesWithNulError` implements `Error` and `Display`][39960]
* [Specialize `PartialOrd<A> for [A] where A: Ord`][39642]
* [Slightly optimize `slice::sort`][39538]
* [Add `ToString` trait specialization for `Cow<'a, str>` and `String`][39440]
* [`Box<[T]>` implements `From<&[T]> where T: Copy`,
  `Box<str>` implements `From<&str>`][39438]
* [`IpAddr` implements `From` for various arrays. `SocketAddr` implements
  `From<(I, u16)> where I: Into<IpAddr>`][39372]
* [`format!` estimates the needed capacity before writing a string][39356]
* [Support unprivileged symlink creation in Windows][38921]
* [`PathBuf` implements `Default`][38764]
* [Implement `PartialEq<[A]>` for `VecDeque<A>`][38661]
* [`HashMap` resizes adaptively][38368] to guard against DOS attacks
  and poor hash functions.

Cargo
-----

* [Add `cargo check --all`][cargo/3731]
* [Add an option to ignore SSL revocation checking][cargo/3699]
* [Add `cargo run --package`][cargo/3691]
* [Add `required_features`][cargo/3667]
* [Assume `build.rs` is a build script][cargo/3664]
* [Find workspace via `workspace_root` link in containing member][cargo/3562]

Misc
----

* [Documentation is rendered with mdbook instead of the obsolete, in-tree
  `crablangbook`][39633]
* [The "Unstable Book" documents nightly-only features][ubook]
* [Improve the style of the sidebar in crablangdoc output][40265]
* [Configure build correctly on 64-bit CPU's with the armhf ABI][40261]
* [Fix MSP430 breakage due to `i128`][40257]
* [Preliminary Solaris/SPARCv9 support][39903]
* [`crablangc` is linked statically on Windows MSVC targets][39837], allowing it to
  run without installing the MSVC runtime.
* [`crablangdoc --test` includes file names in test names][39788]
* This release includes builds of `std` for `sparc64-unknown-linux-gnu`,
  `aarch64-unknown-linux-fuchsia`, and `x86_64-unknown-linux-fuchsia`.
* [Initial support for `aarch64-unknown-freebsd`][39491]
* [Initial support for `i686-unknown-netbsd`][39426]
* [This release no longer includes the old makefile build system][39431]. CrabLang
  is built with a custom build system, written in CrabLang, and with Cargo.
* [Add Debug implementations for libcollection structs][39002]
* [`TypeId` implements `PartialOrd` and `Ord`][38981]
* [`--test-threads=0` produces an error][38945]
* [`crablangup` installs documentation by default][40526]
* [The CrabLang source includes NatVis visualizations][39843]. These can be used by
  WinDbg and Visual Studio to improve the debugging experience.

Compatibility Notes
-------------------

* [CrabLang 1.17 does not correctly detect the MSVC 2017 linker][38584]. As a
  workaround, either use MSVC 2015 or run vcvars.bat.
* [When coercing to an unsized type lifetimes must be equal][40319]. That is,
  disallow subtyping between `T` and `U` when `T: Unsize<U>`, e.g. coercing
  `&mut [&'a X; N]` to `&mut [&'b X]` requires `'a` be equal to `'b`. Soundness
  fix.
* [`format!` and `Display::to_string` panic if an underlying formatting
  implementation returns an error][40117]. Previously the error was silently
  ignored. It is incorrect for `write_fmt` to return an error when writing
  to a string.
* [In-tree crates are verified to be unstable][39851]. Previously, some minor
  crates were marked stable and could be accessed from the stable toolchain.
* [CrabLang git source no longer includes vendored crates][39728]. Those that need
  to build with vendored crates should build from release tarballs.
* [Fix inert attributes from `proc_macro_derives`][39572]
* [During crate resolution, crablangc prefers a crate in the sysroot if two crates
  are otherwise identical][39518]. Unlikely to be encountered outside the CrabLang
  build system.
* [Fixed bugs around how type inference interacts with dead-code][39485]. The
  existing code generally ignores the type of dead-code unless a type-hint is
  provided; this can cause surprising inference interactions particularly around
  defaulting. The new code uniformly ignores the result type of dead-code.
* [Tuple-struct constructors with private fields are no longer visible][38932]
* [Lifetime parameters that do not appear in the arguments are now considered
  early-bound][38897], resolving a soundness bug (#[32330]). The
  `hr_lifetime_in_assoc_type` future-compatibility lint has been in effect since
  April of 2016.
* [crablangdoc: fix doctests with non-feature crate attributes][38161]
* [Make transmuting from fn item types to pointer-sized types a hard
  error][34198]

[27787]: https://github.com/crablang/crablang/issues/27787
[32330]: https://github.com/crablang/crablang/issues/32330
[34198]: https://github.com/crablang/crablang/pull/34198
[38161]: https://github.com/crablang/crablang/pull/38161
[38368]: https://github.com/crablang/crablang/pull/38368
[38584]: https://github.com/crablang/crablang/issues/38584
[38661]: https://github.com/crablang/crablang/pull/38661
[38764]: https://github.com/crablang/crablang/pull/38764
[38864]: https://github.com/crablang/crablang/issues/38864
[38897]: https://github.com/crablang/crablang/pull/38897
[38921]: https://github.com/crablang/crablang/pull/38921
[38932]: https://github.com/crablang/crablang/pull/38932
[38945]: https://github.com/crablang/crablang/pull/38945
[38981]: https://github.com/crablang/crablang/pull/38981
[39002]: https://github.com/crablang/crablang/pull/39002
[39116]: https://github.com/crablang/crablang/pull/39116
[39193]: https://github.com/crablang/crablang/pull/39193
[39265]: https://github.com/crablang/crablang/pull/39265
[39356]: https://github.com/crablang/crablang/pull/39356
[39372]: https://github.com/crablang/crablang/pull/39372
[39426]: https://github.com/crablang/crablang/pull/39426
[39431]: https://github.com/crablang/crablang/pull/39431
[39438]: https://github.com/crablang/crablang/pull/39438
[39440]: https://github.com/crablang/crablang/pull/39440
[39485]: https://github.com/crablang/crablang/pull/39485
[39491]: https://github.com/crablang/crablang/pull/39491
[39518]: https://github.com/crablang/crablang/pull/39518
[39538]: https://github.com/crablang/crablang/pull/39538
[39572]: https://github.com/crablang/crablang/pull/39572
[39633]: https://github.com/crablang/crablang/pull/39633
[39642]: https://github.com/crablang/crablang/pull/39642
[39728]: https://github.com/crablang/crablang/pull/39728
[39761]: https://github.com/crablang/crablang/pull/39761
[39788]: https://github.com/crablang/crablang/pull/39788
[39793]: https://github.com/crablang/crablang/pull/39793
[39837]: https://github.com/crablang/crablang/pull/39837
[39843]: https://github.com/crablang/crablang/pull/39843
[39851]: https://github.com/crablang/crablang/pull/39851
[39903]: https://github.com/crablang/crablang/pull/39903
[39939]: https://github.com/crablang/crablang/pull/39939
[39953]: https://github.com/crablang/crablang/pull/39953
[39960]: https://github.com/crablang/crablang/pull/39960
[39990]: https://github.com/crablang/crablang/pull/39990
[39995]: https://github.com/crablang/crablang/pull/39995
[40008]: https://github.com/crablang/crablang/pull/40008
[40009]: https://github.com/crablang/crablang/pull/40009
[40027]: https://github.com/crablang/crablang/pull/40027
[40028]: https://github.com/crablang/crablang/pull/40028
[40037]: https://github.com/crablang/crablang/pull/40037
[40117]: https://github.com/crablang/crablang/pull/40117
[40166]: https://github.com/crablang/crablang/pull/40166
[40245]: https://github.com/crablang/crablang/pull/40245
[40257]: https://github.com/crablang/crablang/pull/40257
[40261]: https://github.com/crablang/crablang/pull/40261
[40265]: https://github.com/crablang/crablang/pull/40265
[40319]: https://github.com/crablang/crablang/pull/40319
[40336]: https://github.com/crablang/crablang/pull/40336
[40526]: https://github.com/crablang/crablang/pull/40526
[RFC 1623]: https://github.com/crablang/rfcs/blob/master/text/1623-static.md
[RFC 1647]: https://github.com/crablang/rfcs/blob/master/text/1647-allow-self-in-where-clauses.md
[RFC 1651]: https://github.com/crablang/rfcs/blob/master/text/1651-movecell.md
[RFC 1682]: https://github.com/crablang/rfcs/blob/master/text/1682-field-init-shorthand.md
[`Arc::from_raw`]: https://doc.crablang.org/std/sync/struct.Arc.html#method.from_raw
[`Arc::into_raw`]: https://doc.crablang.org/std/sync/struct.Arc.html#method.into_raw
[`Arc::ptr_eq`]: https://doc.crablang.org/std/sync/struct.Arc.html#method.ptr_eq
[`BTreeMap::range_mut`]: https://doc.crablang.org/std/collections/btree_map/struct.BTreeMap.html#method.range_mut
[`BTreeMap::range`]: https://doc.crablang.org/std/collections/btree_map/struct.BTreeMap.html#method.range
[`Cell::into_inner`]: https://doc.crablang.org/std/cell/struct.Cell.html#method.into_inner
[`Cell::replace`]: https://doc.crablang.org/std/cell/struct.Cell.html#method.replace
[`Cell::swap`]: https://doc.crablang.org/std/cell/struct.Cell.html#method.swap
[`Cell::take`]: https://doc.crablang.org/std/cell/struct.Cell.html#method.take
[`Ordering::then_with`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.then_with
[`Ordering::then`]: https://doc.crablang.org/std/cmp/enum.Ordering.html#method.then
[`Rc::from_raw`]: https://doc.crablang.org/std/rc/struct.Rc.html#method.from_raw
[`Rc::into_raw`]: https://doc.crablang.org/std/rc/struct.Rc.html#method.into_raw
[`Rc::ptr_eq`]: https://doc.crablang.org/std/rc/struct.Rc.html#method.ptr_eq
[`Result::expect_err`]: https://doc.crablang.org/std/result/enum.Result.html#method.expect_err
[`collections::Bound`]: https://doc.crablang.org/std/collections/enum.Bound.html
[`process::abort`]: https://doc.crablang.org/std/process/fn.abort.html
[`ptr::read_unaligned`]: https://doc.crablang.org/std/ptr/fn.read_unaligned.html
[`ptr::write_unaligned`]: https://doc.crablang.org/std/ptr/fn.write_unaligned.html
[cargo/3562]: https://github.com/crablang/cargo/pull/3562
[cargo/3664]: https://github.com/crablang/cargo/pull/3664
[cargo/3667]: https://github.com/crablang/cargo/pull/3667
[cargo/3691]: https://github.com/crablang/cargo/pull/3691
[cargo/3699]: https://github.com/crablang/cargo/pull/3699
[cargo/3731]: https://github.com/crablang/cargo/pull/3731
[ubook]: https://doc.crablang.org/unstable-book/


Version 1.16.0 (2017-03-16)
===========================

Language
--------

* [The compiler's `dead_code` lint now accounts for type aliases][38051].
* [Uninhabitable enums (those without any variants) no longer permit wildcard
  match patterns][38069]
* [Clean up semantics of `self` in an import list][38313]
* [`Self` may appear in `impl` headers][38920]
* [`Self` may appear in struct expressions][39282]

Compiler
--------

* [`crablangc` now supports `--emit=metadata`, which causes crablangc to emit
  a `.rmeta` file containing only crate metadata][38571]. This can be
  used by tools like the CrabLang Language Service to perform
  metadata-only builds.
* [Levenshtein based typo suggestions now work in most places, while
  previously they worked only for fields and sometimes for local
  variables][38927]. Together with the overhaul of "no
  resolution"/"unexpected resolution" errors (#[38154]) they result in
  large and systematic improvement in resolution diagnostics.
* [Fix `transmute::<T, U>` where `T` requires a bigger alignment than
  `U`][38670]
* [crablangc: use -Xlinker when specifying an rpath with ',' in it][38798]
* [`crablangc` no longer attempts to provide "consider using an explicit
  lifetime" suggestions][37057]. They were inaccurate.

Stabilized APIs
---------------

* [`VecDeque::truncate`]
* [`VecDeque::resize`]
* [`String::insert_str`]
* [`Duration::checked_add`]
* [`Duration::checked_sub`]
* [`Duration::checked_div`]
* [`Duration::checked_mul`]
* [`str::replacen`]
* [`str::repeat`]
* [`SocketAddr::is_ipv4`]
* [`SocketAddr::is_ipv6`]
* [`IpAddr::is_ipv4`]
* [`IpAddr::is_ipv6`]
* [`Vec::dedup_by`]
* [`Vec::dedup_by_key`]
* [`Result::unwrap_or_default`]
* [`<*const T>::wrapping_offset`]
* [`<*mut T>::wrapping_offset`]
* `CommandExt::creation_flags`
* [`File::set_permissions`]
* [`String::split_off`]

Libraries
---------

* [`[T]::binary_search` and `[T]::binary_search_by_key` now take
  their argument by `Borrow` parameter][37761]
* [All public types in std implement `Debug`][38006]
* [`IpAddr` implements `From<Ipv4Addr>` and `From<Ipv6Addr>`][38327]
* [`Ipv6Addr` implements `From<[u16; 8]>`][38131]
* [Ctrl-Z returns from `Stdin.read()` when reading from the console on
  Windows][38274]
* [std: Fix partial writes in `LineWriter`][38062]
* [std: Clamp max read/write sizes on Unix][38622]
* [Use more specific panic message for `&str` slicing errors][38066]
* [`TcpListener::set_only_v6` is deprecated][38304]. This
  functionality cannot be achieved in std currently.
* [`writeln!`, like `println!`, now accepts a form with no string
  or formatting arguments, to just print a newline][38469]
* [Implement `iter::Sum` and `iter::Product` for `Result`][38580]
* [Reduce the size of static data in `std_unicode::tables`][38781]
* [`char::EscapeDebug`, `EscapeDefault`, `EscapeUnicode`,
  `CaseMappingIter`, `ToLowercase`, `ToUppercase`, implement
  `Display`][38909]
* [`Duration` implements `Sum`][38712]
* [`String` implements `ToSocketAddrs`][39048]

Cargo
-----

* [The `cargo check` command does a type check of a project without
  building it][cargo/3296]
* [crates.io will display CI badges from Travis and AppVeyor, if
  specified in Cargo.toml][cargo/3546]
* [crates.io will display categories listed in Cargo.toml][cargo/3301]
* [Compilation profiles accept integer values for `debug`, in addition
  to `true` and `false`. These are passed to `crablangc` as the value to
  `-C debuginfo`][cargo/3534]
* [Implement `cargo --version --verbose`][cargo/3604]
* [All builds now output 'dep-info' build dependencies compatible with
  make and ninja][cargo/3557]
* [Build all workspace members with `build --all`][cargo/3511]
* [Document all workspace members with `doc --all`][cargo/3515]
* [Path deps outside workspace are not members][cargo/3443]

Misc
----

* [`crablangdoc` has a `--sysroot` argument that, like `crablangc`, specifies
  the path to the CrabLang implementation][38589]
* [The `armv7-linux-androideabi` target no longer enables NEON
  extensions, per Google's ABI guide][38413]
* [The stock standard library can be compiled for Redox OS][38401]
* [CrabLang has initial SPARC support][38726]. Tier 3. No builds
  available.
* [CrabLang has experimental support for Nvidia PTX][38559]. Tier 3. No
  builds available.
* [Fix backtraces on i686-pc-windows-gnu by disabling FPO][39379]

Compatibility Notes
-------------------

* [Uninhabitable enums (those without any variants) no longer permit wildcard
  match patterns][38069]
* In this release, references to uninhabited types can not be
  pattern-matched. This was accidentally allowed in 1.15.
* [The compiler's `dead_code` lint now accounts for type aliases][38051].
* [Ctrl-Z returns from `Stdin.read()` when reading from the console on
  Windows][38274]
* [Clean up semantics of `self` in an import list][38313]
* Reimplemented lifetime elision. This change was almost entirely compatible
  with existing code, but it did close a number of small bugs and loopholes,
  as well as being more accepting in some other [cases][41105].

[37057]: https://github.com/crablang/crablang/pull/37057
[37761]: https://github.com/crablang/crablang/pull/37761
[38006]: https://github.com/crablang/crablang/pull/38006
[38051]: https://github.com/crablang/crablang/pull/38051
[38062]: https://github.com/crablang/crablang/pull/38062
[38622]: https://github.com/crablang/crablang/pull/38622
[38066]: https://github.com/crablang/crablang/pull/38066
[38069]: https://github.com/crablang/crablang/pull/38069
[38131]: https://github.com/crablang/crablang/pull/38131
[38154]: https://github.com/crablang/crablang/pull/38154
[38274]: https://github.com/crablang/crablang/pull/38274
[38304]: https://github.com/crablang/crablang/pull/38304
[38313]: https://github.com/crablang/crablang/pull/38313
[38327]: https://github.com/crablang/crablang/pull/38327
[38401]: https://github.com/crablang/crablang/pull/38401
[38413]: https://github.com/crablang/crablang/pull/38413
[38469]: https://github.com/crablang/crablang/pull/38469
[38559]: https://github.com/crablang/crablang/pull/38559
[38571]: https://github.com/crablang/crablang/pull/38571
[38580]: https://github.com/crablang/crablang/pull/38580
[38589]: https://github.com/crablang/crablang/pull/38589
[38670]: https://github.com/crablang/crablang/pull/38670
[38712]: https://github.com/crablang/crablang/pull/38712
[38726]: https://github.com/crablang/crablang/pull/38726
[38781]: https://github.com/crablang/crablang/pull/38781
[38798]: https://github.com/crablang/crablang/pull/38798
[38909]: https://github.com/crablang/crablang/pull/38909
[38920]: https://github.com/crablang/crablang/pull/38920
[38927]: https://github.com/crablang/crablang/pull/38927
[39048]: https://github.com/crablang/crablang/pull/39048
[39282]: https://github.com/crablang/crablang/pull/39282
[39379]: https://github.com/crablang/crablang/pull/39379
[41105]: https://github.com/crablang/crablang/issues/41105
[`<*const T>::wrapping_offset`]: https://doc.crablang.org/std/primitive.pointer.html#method.wrapping_offset
[`<*mut T>::wrapping_offset`]: https://doc.crablang.org/std/primitive.pointer.html#method.wrapping_offset
[`Duration::checked_add`]: https://doc.crablang.org/std/time/struct.Duration.html#method.checked_add
[`Duration::checked_div`]: https://doc.crablang.org/std/time/struct.Duration.html#method.checked_div
[`Duration::checked_mul`]: https://doc.crablang.org/std/time/struct.Duration.html#method.checked_mul
[`Duration::checked_sub`]: https://doc.crablang.org/std/time/struct.Duration.html#method.checked_sub
[`File::set_permissions`]: https://doc.crablang.org/std/fs/struct.File.html#method.set_permissions
[`IpAddr::is_ipv4`]: https://doc.crablang.org/std/net/enum.IpAddr.html#method.is_ipv4
[`IpAddr::is_ipv6`]: https://doc.crablang.org/std/net/enum.IpAddr.html#method.is_ipv6
[`Result::unwrap_or_default`]: https://doc.crablang.org/std/result/enum.Result.html#method.unwrap_or_default
[`SocketAddr::is_ipv4`]: https://doc.crablang.org/std/net/enum.SocketAddr.html#method.is_ipv4
[`SocketAddr::is_ipv6`]: https://doc.crablang.org/std/net/enum.SocketAddr.html#method.is_ipv6
[`String::insert_str`]: https://doc.crablang.org/std/string/struct.String.html#method.insert_str
[`String::split_off`]: https://doc.crablang.org/std/string/struct.String.html#method.split_off
[`Vec::dedup_by_key`]: https://doc.crablang.org/std/vec/struct.Vec.html#method.dedup_by_key
[`Vec::dedup_by`]: https://doc.crablang.org/std/vec/struct.Vec.html#method.dedup_by
[`VecDeque::resize`]:  https://doc.crablang.org/std/collections/vec_deque/struct.VecDeque.html#method.resize
[`VecDeque::truncate`]: https://doc.crablang.org/std/collections/vec_deque/struct.VecDeque.html#method.truncate
[`str::repeat`]: https://doc.crablang.org/std/primitive.str.html#method.repeat
[`str::replacen`]: https://doc.crablang.org/std/primitive.str.html#method.replacen
[cargo/3296]: https://github.com/crablang/cargo/pull/3296
[cargo/3301]: https://github.com/crablang/cargo/pull/3301
[cargo/3443]: https://github.com/crablang/cargo/pull/3443
[cargo/3511]: https://github.com/crablang/cargo/pull/3511
[cargo/3515]: https://github.com/crablang/cargo/pull/3515
[cargo/3534]: https://github.com/crablang/cargo/pull/3534
[cargo/3546]: https://github.com/crablang/cargo/pull/3546
[cargo/3557]: https://github.com/crablang/cargo/pull/3557
[cargo/3604]: https://github.com/crablang/cargo/pull/3604


Version 1.15.1 (2017-02-09)
===========================

* [Fix IntoIter::as_mut_slice's signature][39466]
* [Compile compiler builtins with `-fPIC` on 32-bit platforms][39523]

[39466]: https://github.com/crablang/crablang/pull/39466
[39523]: https://github.com/crablang/crablang/pull/39523


Version 1.15.0 (2017-02-02)
===========================

Language
--------

* Basic procedural macros allowing custom `#[derive]`, aka "macros 1.1", are
  stable. This allows popular code-generating crates like Serde and Diesel to
  work ergonomically. [RFC 1681].
* [Tuple structs may be empty. Unary and empty tuple structs may be instantiated
  with curly braces][36868]. Part of [RFC 1506].
* [A number of minor changes to name resolution have been activated][37127].
  They add up to more consistent semantics, allowing for future evolution of
  CrabLang macros. Specified in [RFC 1560], see its section on ["changes"] for
  details of what is different. The breaking changes here have been transitioned
  through the [`legacy_imports`] lint since 1.14, with no known regressions.
* [In `macro_rules`, `path` fragments can now be parsed as type parameter
  bounds][38279]
* [`?Sized` can be used in `where` clauses][37791]
* [There is now a limit on the size of monomorphized types and it can be
  modified with the `#![type_size_limit]` crate attribute, similarly to
  the `#![recursion_limit]` attribute][37789]

Compiler
--------

* [On Windows, the compiler will apply dllimport attributes when linking to
  extern functions][37973]. Additional attributes and flags can control which
  library kind is linked and its name. [RFC 1717].
* [CrabLang-ABI symbols are no longer exported from cdylibs][38117]
* [The `--test` flag works with procedural macro crates][38107]
* [Fix `extern "aapcs" fn` ABI][37814]
* [The `-C no-stack-check` flag is deprecated][37636]. It does nothing.
* [The `format!` expander recognizes incorrect `printf` and shell-style
  formatting directives and suggests the correct format][37613].
* [Only report one error for all unused imports in an import list][37456]

Compiler Performance
--------------------

* [Avoid unnecessary `mk_ty` calls in `Ty::super_fold_with`][37705]
* [Avoid more unnecessary `mk_ty` calls in `Ty::super_fold_with`][37979]
* [Don't clone in `UnificationTable::probe`][37848]
* [Remove `scope_auxiliary` to cut RSS by 10%][37764]
* [Use small vectors in type walker][37760]
* [Macro expansion performance was improved][37701]
* [Change `HirVec<P<T>>` to `HirVec<T>` in `hir::Expr`][37642]
* [Replace FNV with a faster hash function][37229]

Stabilized APIs
---------------

* [`std::iter::Iterator::min_by`]
* [`std::iter::Iterator::max_by`]
* [`std::os::*::fs::FileExt`]
* [`std::sync::atomic::Atomic*::get_mut`]
* [`std::sync::atomic::Atomic*::into_inner`]
* [`std::vec::IntoIter::as_slice`]
* [`std::vec::IntoIter::as_mut_slice`]
* [`std::sync::mpsc::Receiver::try_iter`]
* [`std::os::unix::process::CommandExt::before_exec`]
* [`std::rc::Rc::strong_count`]
* [`std::rc::Rc::weak_count`]
* [`std::sync::Arc::strong_count`]
* [`std::sync::Arc::weak_count`]
* [`std::char::encode_utf8`]
* [`std::char::encode_utf16`]
* [`std::cell::Ref::clone`]
* [`std::io::Take::into_inner`]

Libraries
---------

* [The standard sorting algorithm has been rewritten for dramatic performance
  improvements][38192]. It is a hybrid merge sort, drawing influences from
  Timsort. Previously it was a naive merge sort.
* [`Iterator::nth` no longer has a `Sized` bound][38134]
* [`Extend<&T>` is specialized for `Vec` where `T: Copy`][38182] to improve
  performance.
* [`chars().count()` is much faster][37888] and so are [`chars().last()`
  and `char_indices().last()`][37882]
* [Fix ARM Objective-C ABI in `std::env::args`][38146]
* [Chinese characters display correctly in `fmt::Debug`][37855]
* [Derive `Default` for `Duration`][37699]
* [Support creation of anonymous pipes on WinXP/2k][37677]
* [`mpsc::RecvTimeoutError` implements `Error`][37527]
* [Don't pass overlapped handles to processes][38835]

Cargo
-----

* [In this release, Cargo build scripts no longer have access to the `OUT_DIR`
  environment variable at build time via `env!("OUT_DIR")`][cargo/3368]. They
  should instead check the variable at runtime with `std::env`. That the value
  was set at build time was a bug, and incorrect when cross-compiling. This
  change is known to cause breakage.
* [Add `--all` flag to `cargo test`][cargo/3221]
* [Compile statically against the MSVC CRT][cargo/3363]
* [Mix feature flags into fingerprint/metadata shorthash][cargo/3102]
* [Link OpenSSL statically on OSX][cargo/3311]
* [Apply new fingerprinting to build dir outputs][cargo/3310]
* [Test for bad path overrides with summaries][cargo/3336]
* [Require `cargo install --vers` to take a semver version][cargo/3338]
* [Fix retrying crate downloads for network errors][cargo/3348]
* [Implement string lookup for `build.crablangflags` config key][cargo/3356]
* [Emit more info on --message-format=json][cargo/3319]
* [Assume `build.rs` in the same directory as `Cargo.toml` is a build script][cargo/3361]
* [Don't ignore errors in workspace manifest][cargo/3409]
* [Fix `--message-format JSON` when crablangc emits non-JSON warnings][cargo/3410]

Tooling
-------

* [Test runners (binaries built with `--test`) now support a `--list` argument
  that lists the tests it contains][38185]
* [Test runners now support a `--exact` argument that makes the test filter
  match exactly, instead of matching only a substring of the test name][38181]
* [crablangdoc supports a `--playground-url` flag][37763]
* [crablangdoc provides more details about `#[should_panic]` errors][37749]

Misc
----

* [The CrabLang build system is now written in CrabLang][37817]. The Makefiles may
  continue to be used in this release by passing `--disable-crablangbuild` to the
  configure script, but they will be deleted soon. Note that the new build
  system uses a different on-disk layout that will likely affect any scripts
  building CrabLang.
* [CrabLang supports i686-unknown-openbsd][38086]. Tier 3 support. No testing or
  releases.
* [CrabLang supports the MSP430][37627]. Tier 3 support. No testing or releases.
* [CrabLang supports the ARMv5TE architecture][37615]. Tier 3 support. No testing or
  releases.

Compatibility Notes
-------------------

* [A number of minor changes to name resolution have been activated][37127].
  They add up to more consistent semantics, allowing for future evolution of
  CrabLang macros. Specified in [RFC 1560], see its section on ["changes"] for
  details of what is different. The breaking changes here have been transitioned
  through the [`legacy_imports`] lint since 1.14, with no known regressions.
* [In this release, Cargo build scripts no longer have access to the `OUT_DIR`
  environment variable at build time via `env!("OUT_DIR")`][cargo/3368]. They
  should instead check the variable at runtime with `std::env`. That the value
  was set at build time was a bug, and incorrect when cross-compiling. This
  change is known to cause breakage.
* [Higher-ranked lifetimes are no longer allowed to appear _only_ in associated
  types][33685]. The [`hr_lifetime_in_assoc_type` lint] has been a warning since
  1.10 and is now an error by default. It will become a hard error in the near
  future.
* [The semantics relating modules to file system directories are changing in
  minor ways][37602]. This is captured in the new `legacy_directory_ownership`
  lint, which is a warning in this release, and will become a hard error in the
  future.
* [CrabLang-ABI symbols are no longer exported from cdylibs][38117]
* [Once `Peekable` peeks a `None` it will return that `None` without re-querying
  the underlying iterator][37834]

["changes"]: https://github.com/crablang/rfcs/blob/master/text/1560-name-resolution.md#changes-to-name-resolution-rules
[33685]: https://github.com/crablang/crablang/issues/33685
[36868]: https://github.com/crablang/crablang/pull/36868
[37127]: https://github.com/crablang/crablang/pull/37127
[37229]: https://github.com/crablang/crablang/pull/37229
[37456]: https://github.com/crablang/crablang/pull/37456
[37527]: https://github.com/crablang/crablang/pull/37527
[37602]: https://github.com/crablang/crablang/pull/37602
[37613]: https://github.com/crablang/crablang/pull/37613
[37615]: https://github.com/crablang/crablang/pull/37615
[37636]: https://github.com/crablang/crablang/pull/37636
[37627]: https://github.com/crablang/crablang/pull/37627
[37642]: https://github.com/crablang/crablang/pull/37642
[37677]: https://github.com/crablang/crablang/pull/37677
[37699]: https://github.com/crablang/crablang/pull/37699
[37701]: https://github.com/crablang/crablang/pull/37701
[37705]: https://github.com/crablang/crablang/pull/37705
[37749]: https://github.com/crablang/crablang/pull/37749
[37760]: https://github.com/crablang/crablang/pull/37760
[37763]: https://github.com/crablang/crablang/pull/37763
[37764]: https://github.com/crablang/crablang/pull/37764
[37789]: https://github.com/crablang/crablang/pull/37789
[37791]: https://github.com/crablang/crablang/pull/37791
[37814]: https://github.com/crablang/crablang/pull/37814
[37817]: https://github.com/crablang/crablang/pull/37817
[37834]: https://github.com/crablang/crablang/pull/37834
[37848]: https://github.com/crablang/crablang/pull/37848
[37855]: https://github.com/crablang/crablang/pull/37855
[37882]: https://github.com/crablang/crablang/pull/37882
[37888]: https://github.com/crablang/crablang/pull/37888
[37973]: https://github.com/crablang/crablang/pull/37973
[37979]: https://github.com/crablang/crablang/pull/37979
[38086]: https://github.com/crablang/crablang/pull/38086
[38107]: https://github.com/crablang/crablang/pull/38107
[38117]: https://github.com/crablang/crablang/pull/38117
[38134]: https://github.com/crablang/crablang/pull/38134
[38146]: https://github.com/crablang/crablang/pull/38146
[38181]: https://github.com/crablang/crablang/pull/38181
[38182]: https://github.com/crablang/crablang/pull/38182
[38185]: https://github.com/crablang/crablang/pull/38185
[38192]: https://github.com/crablang/crablang/pull/38192
[38279]: https://github.com/crablang/crablang/pull/38279
[38835]: https://github.com/crablang/crablang/pull/38835
[RFC 1506]: https://github.com/crablang/rfcs/blob/master/text/1506-adt-kinds.md
[RFC 1560]: https://github.com/crablang/rfcs/blob/master/text/1560-name-resolution.md
[RFC 1681]: https://github.com/crablang/rfcs/blob/master/text/1681-macros-1.1.md
[RFC 1717]: https://github.com/crablang/rfcs/blob/master/text/1717-dllimport.md
[`hr_lifetime_in_assoc_type` lint]: https://github.com/crablang/crablang/issues/33685
[`legacy_imports`]: https://github.com/crablang/crablang/pull/38271
[cargo/3102]: https://github.com/crablang/cargo/pull/3102
[cargo/3221]: https://github.com/crablang/cargo/pull/3221
[cargo/3310]: https://github.com/crablang/cargo/pull/3310
[cargo/3311]: https://github.com/crablang/cargo/pull/3311
[cargo/3319]: https://github.com/crablang/cargo/pull/3319
[cargo/3336]: https://github.com/crablang/cargo/pull/3336
[cargo/3338]: https://github.com/crablang/cargo/pull/3338
[cargo/3348]: https://github.com/crablang/cargo/pull/3348
[cargo/3356]: https://github.com/crablang/cargo/pull/3356
[cargo/3361]: https://github.com/crablang/cargo/pull/3361
[cargo/3363]: https://github.com/crablang/cargo/pull/3363
[cargo/3368]: https://github.com/crablang/cargo/issues/3368
[cargo/3409]: https://github.com/crablang/cargo/pull/3409
[cargo/3410]: https://github.com/crablang/cargo/pull/3410
[`std::iter::Iterator::min_by`]: https://doc.crablang.org/std/iter/trait.Iterator.html#method.min_by
[`std::iter::Iterator::max_by`]: https://doc.crablang.org/std/iter/trait.Iterator.html#method.max_by
[`std::os::*::fs::FileExt`]: https://doc.crablang.org/std/os/unix/fs/trait.FileExt.html
[`std::sync::atomic::Atomic*::get_mut`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicU8.html#method.get_mut
[`std::sync::atomic::Atomic*::into_inner`]: https://doc.crablang.org/std/sync/atomic/struct.AtomicU8.html#method.into_inner
[`std::vec::IntoIter::as_slice`]: https://doc.crablang.org/std/vec/struct.IntoIter.html#method.as_slice
[`std::vec::IntoIter::as_mut_slice`]: https://doc.crablang.org/std/vec/struct.IntoIter.html#method.as_mut_slice
[`std::sync::mpsc::Receiver::try_iter`]: https://doc.crablang.org/std/sync/mpsc/struct.Receiver.html#method.try_iter
[`std::os::unix::process::CommandExt::before_exec`]: https://doc.crablang.org/std/os/unix/process/trait.CommandExt.html#tymethod.before_exec
[`std::rc::Rc::strong_count`]: https://doc.crablang.org/std/rc/struct.Rc.html#method.strong_count
[`std::rc::Rc::weak_count`]: https://doc.crablang.org/std/rc/struct.Rc.html#method.weak_count
[`std::sync::Arc::strong_count`]: https://doc.crablang.org/std/sync/struct.Arc.html#method.strong_count
[`std::sync::Arc::weak_count`]: https://doc.crablang.org/std/sync/struct.Arc.html#method.weak_count
[`std::char::encode_utf8`]: https://doc.crablang.org/std/primitive.char.html#method.encode_utf8
[`std::char::encode_utf16`]: https://doc.crablang.org/std/primitive.char.html#method.encode_utf16
[`std::cell::Ref::clone`]: https://doc.crablang.org/std/cell/struct.Ref.html#method.clone
[`std::io::Take::into_inner`]: https://doc.crablang.org/std/io/struct.Take.html#method.into_inner


Version 1.14.0 (2016-12-22)
===========================

Language
--------

* [`..` matches multiple tuple fields in enum variants, structs
  and tuples][36843]. [RFC 1492].
* [Safe `fn` items can be coerced to `unsafe fn` pointers][37389]
* [`use *` and `use ::*` both glob-import from the crate root][37367]
* [It's now possible to call a `Vec<Box<Fn()>>` without explicit
  dereferencing][36822]

Compiler
--------

* [Mark enums with non-zero discriminant as non-zero][37224]
* [Lower-case `static mut` names are linted like other
  statics and consts][37162]
* [Fix ICE on some macros in const integer positions
   (e.g. `[u8; m!()]`)][36819]
* [Improve error message and snippet for "did you mean `x`"][36798]
* [Add a panic-strategy field to the target specification][36794]
* [Include LLVM version in `--version --verbose`][37200]

Compile-time Optimizations
--------------------------

* [Improve macro expansion performance][37569]
* [Shrink `Expr_::ExprInlineAsm`][37445]
* [Replace all uses of SHA-256 with BLAKE2b][37439]
* [Reduce the number of bytes hashed by `IchHasher`][37427]
* [Avoid more allocations when compiling html5ever][37373]
* [Use `SmallVector` in `CombineFields::instantiate`][37322]
* [Avoid some allocations in the macro parser][37318]
* [Use a faster deflate setting][37298]
* [Add `ArrayVec` and `AccumulateVec` to reduce heap allocations
  during interning of slices][37270]
* [Optimize `write_metadata`][37267]
* [Don't process obligation forest cycles when stalled][37231]
* [Avoid many `CrateConfig` clones][37161]
* [Optimize `Substs::super_fold_with`][37108]
* [Optimize `ObligationForest`'s `NodeState` handling][36993]
* [Speed up `plug_leaks`][36917]

Libraries
---------

* [`println!()`, with no arguments, prints newline][36825].
  Previously, an empty string was required to achieve the same.
* [`Wrapping` impls standard binary and unary operators, as well as
   the `Sum` and `Product` iterators][37356]
* [Implement `From<Cow<str>> for String` and `From<Cow<[T]>> for
  Vec<T>`][37326]
* [Improve `fold` performance for `chain`, `cloned`, `map`, and
  `VecDeque` iterators][37315]
* [Improve `SipHasher` performance on small values][37312]
* [Add Iterator trait TcrablangedLen to enable better FromIterator /
  Extend][37306]
* [Expand `.zip()` specialization to `.map()` and `.cloned()`][37230]
* [`ReadDir` implements `Debug`][37221]
* [Implement `RefUnwindSafe` for atomic types][37178]
* [Specialize `Vec::extend` to `Vec::extend_from_slice`][37094]
* [Avoid allocations in `Decoder::read_str`][37064]
* [`io::Error` implements `From<io::ErrorKind>`][37037]
* [Impl `Debug` for raw pointers to unsized data][36880]
* [Don't reuse `HashMap` random seeds][37470]
* [The internal memory layout of `HashMap` is more cache-friendly, for
  significant improvements in some operations][36692]
* [`HashMap` uses less memory on 32-bit architectures][36595]
* [Impl `Add<{str, Cow<str>}>` for `Cow<str>`][36430]

Cargo
-----

* [Expose crablangc cfg values to build scripts][cargo/3243]
* [Allow cargo to work with read-only `CARGO_HOME`][cargo/3259]
* [Fix passing --features when testing multiple packages][cargo/3280]
* [Use a single profile set per workspace][cargo/3249]
* [Load `replace` sections from lock files][cargo/3220]
* [Ignore `panic` configuration for test/bench profiles][cargo/3175]

Tooling
-------

* [crablangup is the recommended CrabLang installation method][1.14crablangup]
* This release includes host (crablangc) builds for Linux on MIPS, PowerPC, and
  S390x. These are [tier 2] platforms and may have major defects. Follow the
  instructions on the website to install, or add the targets to an existing
  installation with `crablangup target add`. The new target triples are:
  - `mips-unknown-linux-gnu`
  - `mipsel-unknown-linux-gnu`
  - `mips64-unknown-linux-gnuabi64`
  - `mips64el-unknown-linux-gnuabi64 `
  - `powerpc-unknown-linux-gnu`
  - `powerpc64-unknown-linux-gnu`
  - `powerpc64le-unknown-linux-gnu`
  - `s390x-unknown-linux-gnu `
* This release includes target (std) builds for ARM Linux running MUSL
  libc. These are [tier 2] platforms and may have major defects. Add the
  following triples to an existing crablangup installation with `crablangup target add`:
  - `arm-unknown-linux-musleabi`
  - `arm-unknown-linux-musleabihf`
  - `armv7-unknown-linux-musleabihf`
* This release includes [experimental support for WebAssembly][1.14wasm], via
  the `wasm32-unknown-emscripten` target. This target is known to have major
  defects. Please test, report, and fix.
* crablangup no longer installs documentation by default. Run `crablangup
  component add crablang-docs` to install.
* [Fix line stepping in debugger][37310]
* [Enable line number debuginfo in releases][37280]

Misc
----

* [Disable jemalloc on aarch64/powerpc/mips][37392]
* [Add support for Fuchsia OS][37313]
* [Detect local-rebuild by only MAJOR.MINOR version][37273]

Compatibility Notes
-------------------

* [A number of forward-compatibility lints used by the compiler
  to gradually introduce language changes have been converted
  to deny by default][36894]:
  - ["use of inaccessible extern crate erroneously allowed"][36886]
  - ["type parameter default erroneously allowed in invalid location"][36887]
  - ["detects super or self keywords at the beginning of global path"][36888]
  - ["two overlapping inherent impls define an item with the same name
    were erroneously allowed"][36889]
  - ["floating-point constants cannot be used in patterns"][36890]
  - ["constants of struct or enum type can only be used in a pattern if
     the struct or enum has `#[derive(PartialEq, Eq)]`"][36891]
  - ["lifetimes or labels named `'_` were erroneously allowed"][36892]
* [Prohibit patterns in trait methods without bodies][37378]
* [The atomic `Ordering` enum may not be matched exhaustively][37351]
* [Future-proofing `#[no_link]` breaks some obscure cases][37247]
* [The `$crate` macro variable is accepted in fewer locations][37213]
* [Impls specifying extra region requirements beyond the trait
  they implement are rejected][37167]
* [Enums may not be unsized][37111]. Unsized enums are intended to
  work but never have. For now they are forbidden.
* [Enforce the shadowing restrictions from RFC 1560 for today's macros][36767]

[tier 2]: https://forge.crablang.org/platform-support.html
[1.14crablangup]: https://internals.crablang.org/t/beta-testing-crablangup-rs/3316/204
[1.14wasm]: https://users.crablang.org/t/compiling-to-the-web-with-crablang-and-emscripten/7627
[36430]: https://github.com/crablang/crablang/pull/36430
[36595]: https://github.com/crablang/crablang/pull/36595
[36692]: https://github.com/crablang/crablang/pull/36692
[36767]: https://github.com/crablang/crablang/pull/36767
[36794]: https://github.com/crablang/crablang/pull/36794
[36798]: https://github.com/crablang/crablang/pull/36798
[36819]: https://github.com/crablang/crablang/pull/36819
[36822]: https://github.com/crablang/crablang/pull/36822
[36825]: https://github.com/crablang/crablang/pull/36825
[36843]: https://github.com/crablang/crablang/pull/36843
[36880]: https://github.com/crablang/crablang/pull/36880
[36886]: https://github.com/crablang/crablang/issues/36886
[36887]: https://github.com/crablang/crablang/issues/36887
[36888]: https://github.com/crablang/crablang/issues/36888
[36889]: https://github.com/crablang/crablang/issues/36889
[36890]: https://github.com/crablang/crablang/issues/36890
[36891]: https://github.com/crablang/crablang/issues/36891
[36892]: https://github.com/crablang/crablang/issues/36892
[36894]: https://github.com/crablang/crablang/pull/36894
[36917]: https://github.com/crablang/crablang/pull/36917
[36993]: https://github.com/crablang/crablang/pull/36993
[37037]: https://github.com/crablang/crablang/pull/37037
[37064]: https://github.com/crablang/crablang/pull/37064
[37094]: https://github.com/crablang/crablang/pull/37094
[37108]: https://github.com/crablang/crablang/pull/37108
[37111]: https://github.com/crablang/crablang/pull/37111
[37161]: https://github.com/crablang/crablang/pull/37161
[37162]: https://github.com/crablang/crablang/pull/37162
[37167]: https://github.com/crablang/crablang/pull/37167
[37178]: https://github.com/crablang/crablang/pull/37178
[37200]: https://github.com/crablang/crablang/pull/37200
[37213]: https://github.com/crablang/crablang/pull/37213
[37221]: https://github.com/crablang/crablang/pull/37221
[37224]: https://github.com/crablang/crablang/pull/37224
[37230]: https://github.com/crablang/crablang/pull/37230
[37231]: https://github.com/crablang/crablang/pull/37231
[37247]: https://github.com/crablang/crablang/pull/37247
[37267]: https://github.com/crablang/crablang/pull/37267
[37270]: https://github.com/crablang/crablang/pull/37270
[37273]: https://github.com/crablang/crablang/pull/37273
[37280]: https://github.com/crablang/crablang/pull/37280
[37298]: https://github.com/crablang/crablang/pull/37298
[37306]: https://github.com/crablang/crablang/pull/37306
[37310]: https://github.com/crablang/crablang/pull/37310
[37312]: https://github.com/crablang/crablang/pull/37312
[37313]: https://github.com/crablang/crablang/pull/37313
[37315]: https://github.com/crablang/crablang/pull/37315
[37318]: https://github.com/crablang/crablang/pull/37318
[37322]: https://github.com/crablang/crablang/pull/37322
[37326]: https://github.com/crablang/crablang/pull/37326
[37351]: https://github.com/crablang/crablang/pull/37351
[37356]: https://github.com/crablang/crablang/pull/37356
[37367]: https://github.com/crablang/crablang/pull/37367
[37373]: https://github.com/crablang/crablang/pull/37373
[37378]: https://github.com/crablang/crablang/pull/37378
[37389]: https://github.com/crablang/crablang/pull/37389
[37392]: https://github.com/crablang/crablang/pull/37392
[37427]: https://github.com/crablang/crablang/pull/37427
[37439]: https://github.com/crablang/crablang/pull/37439
[37445]: https://github.com/crablang/crablang/pull/37445
[37470]: https://github.com/crablang/crablang/pull/37470
[37569]: https://github.com/crablang/crablang/pull/37569
[RFC 1492]: https://github.com/crablang/rfcs/blob/master/text/1492-dotdot-in-patterns.md
[cargo/3175]: https://github.com/crablang/cargo/pull/3175
[cargo/3220]: https://github.com/crablang/cargo/pull/3220
[cargo/3243]: https://github.com/crablang/cargo/pull/3243
[cargo/3249]: https://github.com/crablang/cargo/pull/3249
[cargo/3259]: https://github.com/crablang/cargo/pull/3259
[cargo/3280]: https://github.com/crablang/cargo/pull/3280


Version 1.13.0 (2016-11-10)
===========================

Language
--------

* [Stabilize the `?` operator][36995]. `?` is a simple way to propagate
  errors, like the `try!` macro, described in [RFC 0243].
* [Stabilize macros in type position][36014]. Described in [RFC 873].
* [Stabilize attributes on statements][36995]. Described in [RFC 0016].
* [Fix `#[derive]` for empty tuple structs/variants][35728]
* [Fix lifetime rules for 'if' conditions][36029]
* [Avoid loading and parsing unconfigured non-inline modules][36482]

Compiler
--------

* [Add the `-C link-arg` argument][36574]
* [Remove the old AST-based backend from crablangc_trans][35764]
* [Don't enable NEON by default on armv7 Linux][35814]
* [Fix debug line number info for macro expansions][35238]
* [Do not emit "class method" debuginfo for types that are not
  DICompositeType][36008]
* [Warn about multiple conflicting #[repr] hints][34623]
* [When sizing DST, don't double-count nested struct prefixes][36351]
* [Default CRABLANG_MIN_STACK to 16MiB for now][36505]
* [Improve rlib metadata format][36551]. Reduces rlib size significantly.
* [Reject macros with empty repetitions to avoid infinite loop][36721]
* [Expand macros without recursing to avoid stack overflows][36214]

Diagnostics
-----------

* [Replace macro backtraces with labeled local uses][35702]
* [Improve error message for misplaced doc comments][33922]
* [Buffer unix and lock windows to prevent message interleaving][35975]
* [Update lifetime errors to specifically note temporaries][36171]
* [Special case a few colors for Windows][36178]
* [Suggest `use self` when such an import resolves][36289]
* [Be more specific when type parameter shadows primitive type][36338]
* Many minor improvements

Compile-time Optimizations
--------------------------

* [Compute and cache HIR hashes at beginning][35854]
* [Don't hash types in loan paths][36004]
* [Cache projections in trans][35761]
* [Optimize the parser's last token handling][36527]
* [Only instantiate #[inline] functions in codegen units referencing
  them][36524]. This leads to big improvements in cases where crates export
  define many inline functions without using them directly.
* [Lazily allocate TypedArena's first chunk][36592]
* [Don't allocate during default HashSet creation][36734]

Stabilized APIs
---------------

* [`checked_abs`]
* [`wrapping_abs`]
* [`overflowing_abs`]
* [`RefCell::try_borrow`]
* [`RefCell::try_borrow_mut`]

Libraries
---------

* [Add `assert_ne!` and `debug_assert_ne!`][35074]
* [Make `vec_deque::Drain`, `hash_map::Drain`, and `hash_set::Drain`
  covariant][35354]
* [Implement `AsRef<[T]>` for `std::slice::Iter`][35559]
* [Implement `Debug` for `std::vec::IntoIter`][35707]
* [`CString`: avoid excessive growth just to 0-terminate][35871]
* [Implement `CoerceUnsized` for `{Cell, RefCell, UnsafeCell}`][35627]
* [Use arc4rand on FreeBSD][35884]
* [memrchr: Correct aligned offset computation][35969]
* [Improve Demangling of CrabLang Symbols][36059]
* [Use monotonic time in condition variables][35048]
* [Implement `Debug` for `std::path::{Components,Iter}`][36101]
* [Implement conversion traits for `char`][35755]
* [Fix illegal instruction caused by overflow in channel cloning][36104]
* [Zero first byte of CString on drop][36264]
* [Inherit overflow checks for sum and product][36372]
* [Add missing Eq implementations][36423]
* [Implement `Debug` for `DirEntry`][36631]
* [When `getaddrinfo` returns `EAI_SYSTEM` retrieve actual error from
  `errno`][36754]
* [`SipHasher`] is deprecated. Use [`DefaultHasher`].
* [Implement more traits for `std::io::ErrorKind`][35911]
* [Optimize BinaryHeap bounds checking][36072]
* [Work around pointer aliasing issue in `Vec::extend_from_slice`,
  `extend_with_element`][36355]
* [Fix overflow checking in unsigned pow()][34942]

Cargo
-----

* This release includes security fixes to both curl and OpenSSL.
* [Fix transitive doctests when panic=abort][cargo/3021]
* [Add --all-features flag to cargo][cargo/3038]
* [Reject path-based dependencies in `cargo package`][cargo/3060]
* [Don't parse the home directory more than once][cargo/3078]
* [Don't try to generate Cargo.lock on empty workspaces][cargo/3092]
* [Update OpenSSL to 1.0.2j][cargo/3121]
* [Add license and license_file to cargo metadata output][cargo/3110]
* [Make crates-io registry URL optional in config; ignore all changes to
  source.crates-io][cargo/3089]
* [Don't download dependencies from other platforms][cargo/3123]
* [Build transitive dev-dependencies when needed][cargo/3125]
* [Add support for per-target crablangflags in .cargo/config][cargo/3157]
* [Avoid updating registry when adding existing deps][cargo/3144]
* [Warn about path overrides that won't work][cargo/3136]
* [Use workspaces during `cargo install`][cargo/3146]
* [Leak mspdbsrv.exe processes on Windows][cargo/3162]
* [Add --message-format flag][cargo/3000]
* [Pass target environment for crablangdoc][cargo/3205]
* [Use `CommandExt::exec` for `cargo run` on Unix][cargo/2818]
* [Update curl and curl-sys][cargo/3241]
* [Call crablangdoc test with the correct cfg flags of a package][cargo/3242]

Tooling
-------

* [crablangdoc: Add the `--sysroot` argument][36586]
* [crablangdoc: Fix a couple of issues with the search results][35655]
* [crablangdoc: remove the `!` from macro URLs and titles][35234]
* [gdb: Fix pretty-printing special-cased CrabLang types][35585]
* [crablangdoc: Filter more incorrect methods inherited through Deref][36266]

Misc
----

* [Remove unmaintained style guide][35124]
* [Add s390x support][36369]
* [Initial work at Haiku OS support][36727]
* [Add mips-uclibc targets][35734]
* [Crate-ify compiler-rt into compiler-builtins][35021]
* [Add crablangc version info (git hash + date) to dist tarball][36213]
* Many documentation improvements

Compatibility Notes
-------------------

* [`SipHasher`] is deprecated. Use [`DefaultHasher`].
* [Deny (by default) transmuting from fn item types to pointer-sized
  types][34923]. Continuing the long transition to zero-sized fn items,
  per [RFC 401].
* [Fix `#[derive]` for empty tuple structs/variants][35728].
  Part of [RFC 1506].
* [Issue deprecation warnings for safe accesses to extern statics][36173]
* [Fix lifetime rules for 'if' conditions][36029].
* [Inherit overflow checks for sum and product][36372].
* [Forbid user-defined macros named "macro_rules"][36730].

[33922]: https://github.com/crablang/crablang/pull/33922
[34623]: https://github.com/crablang/crablang/pull/34623
[34923]: https://github.com/crablang/crablang/pull/34923
[34942]: https://github.com/crablang/crablang/pull/34942
[35021]: https://github.com/crablang/crablang/pull/35021
[35048]: https://github.com/crablang/crablang/pull/35048
[35074]: https://github.com/crablang/crablang/pull/35074
[35124]: https://github.com/crablang/crablang/pull/35124
[35234]: https://github.com/crablang/crablang/pull/35234
[35238]: https://github.com/crablang/crablang/pull/35238
[35354]: https://github.com/crablang/crablang/pull/35354
[35559]: https://github.com/crablang/crablang/pull/35559
[35585]: https://github.com/crablang/crablang/pull/35585
[35627]: https://github.com/crablang/crablang/pull/35627
[35655]: https://github.com/crablang/crablang/pull/35655
[35702]: https://github.com/crablang/crablang/pull/35702
[35707]: https://github.com/crablang/crablang/pull/35707
[35728]: https://github.com/crablang/crablang/pull/35728
[35734]: https://github.com/crablang/crablang/pull/35734
[35755]: https://github.com/crablang/crablang/pull/35755
[35761]: https://github.com/crablang/crablang/pull/35761
[35764]: https://github.com/crablang/crablang/pull/35764
[35814]: https://github.com/crablang/crablang/pull/35814
[35854]: https://github.com/crablang/crablang/pull/35854
[35871]: https://github.com/crablang/crablang/pull/35871
[35884]: https://github.com/crablang/crablang/pull/35884
[35911]: https://github.com/crablang/crablang/pull/35911
[35969]: https://github.com/crablang/crablang/pull/35969
[35975]: https://github.com/crablang/crablang/pull/35975
[36004]: https://github.com/crablang/crablang/pull/36004
[36008]: https://github.com/crablang/crablang/pull/36008
[36014]: https://github.com/crablang/crablang/pull/36014
[36029]: https://github.com/crablang/crablang/pull/36029
[36059]: https://github.com/crablang/crablang/pull/36059
[36072]: https://github.com/crablang/crablang/pull/36072
[36101]: https://github.com/crablang/crablang/pull/36101
[36104]: https://github.com/crablang/crablang/pull/36104
[36171]: https://github.com/crablang/crablang/pull/36171
[36173]: https://github.com/crablang/crablang/pull/36173
[36178]: https://github.com/crablang/crablang/pull/36178
[36213]: https://github.com/crablang/crablang/pull/36213
[36214]: https://github.com/crablang/crablang/pull/36214
[36264]: https://github.com/crablang/crablang/pull/36264
[36266]: https://github.com/crablang/crablang/pull/36266
[36289]: https://github.com/crablang/crablang/pull/36289
[36338]: https://github.com/crablang/crablang/pull/36338
[36351]: https://github.com/crablang/crablang/pull/36351
[36355]: https://github.com/crablang/crablang/pull/36355
[36369]: https://github.com/crablang/crablang/pull/36369
[36372]: https://github.com/crablang/crablang/pull/36372
[36423]: https://github.com/crablang/crablang/pull/36423
[36482]: https://github.com/crablang/crablang/pull/36482
[36505]: https://github.com/crablang/crablang/pull/36505
[36524]: https://github.com/crablang/crablang/pull/36524
[36527]: https://github.com/crablang/crablang/pull/36527
[36551]: https://github.com/crablang/crablang/pull/36551
[36574]: https://github.com/crablang/crablang/pull/36574
[36586]: https://github.com/crablang/crablang/pull/36586
[36592]: https://github.com/crablang/crablang/pull/36592
[36631]: https://github.com/crablang/crablang/pull/36631
[36721]: https://github.com/crablang/crablang/pull/36721
[36727]: https://github.com/crablang/crablang/pull/36727
[36730]: https://github.com/crablang/crablang/pull/36730
[36734]: https://github.com/crablang/crablang/pull/36734
[36754]: https://github.com/crablang/crablang/pull/36754
[36995]: https://github.com/crablang/crablang/pull/36995
[RFC 0016]: https://github.com/crablang/rfcs/blob/master/text/0016-more-attributes.md
[RFC 0243]: https://github.com/crablang/rfcs/blob/master/text/0243-trait-based-exception-handling.md
[RFC 1506]: https://github.com/crablang/rfcs/blob/master/text/1506-adt-kinds.md
[RFC 401]: https://github.com/crablang/rfcs/blob/master/text/0401-coercions.md
[RFC 873]: https://github.com/crablang/rfcs/blob/master/text/0873-type-macros.md
[cargo/2818]: https://github.com/crablang/cargo/pull/2818
[cargo/3000]: https://github.com/crablang/cargo/pull/3000
[cargo/3021]: https://github.com/crablang/cargo/pull/3021
[cargo/3038]: https://github.com/crablang/cargo/pull/3038
[cargo/3060]: https://github.com/crablang/cargo/pull/3060
[cargo/3078]: https://github.com/crablang/cargo/pull/3078
[cargo/3089]: https://github.com/crablang/cargo/pull/3089
[cargo/3092]: https://github.com/crablang/cargo/pull/3092
[cargo/3110]: https://github.com/crablang/cargo/pull/3110
[cargo/3121]: https://github.com/crablang/cargo/pull/3121
[cargo/3123]: https://github.com/crablang/cargo/pull/3123
[cargo/3125]: https://github.com/crablang/cargo/pull/3125
[cargo/3136]: https://github.com/crablang/cargo/pull/3136
[cargo/3144]: https://github.com/crablang/cargo/pull/3144
[cargo/3146]: https://github.com/crablang/cargo/pull/3146
[cargo/3157]: https://github.com/crablang/cargo/pull/3157
[cargo/3162]: https://github.com/crablang/cargo/pull/3162
[cargo/3205]: https://github.com/crablang/cargo/pull/3205
[cargo/3241]: https://github.com/crablang/cargo/pull/3241
[cargo/3242]: https://github.com/crablang/cargo/pull/3242
[`checked_abs`]: https://doc.crablang.org/std/primitive.i32.html#method.checked_abs
[`wrapping_abs`]: https://doc.crablang.org/std/primitive.i32.html#method.wrapping_abs
[`overflowing_abs`]: https://doc.crablang.org/std/primitive.i32.html#method.overflowing_abs
[`RefCell::try_borrow`]: https://doc.crablang.org/std/cell/struct.RefCell.html#method.try_borrow
[`RefCell::try_borrow_mut`]: https://doc.crablang.org/std/cell/struct.RefCell.html#method.try_borrow_mut
[`SipHasher`]: https://doc.crablang.org/std/hash/struct.SipHasher.html
[`DefaultHasher`]: https://doc.crablang.org/std/collections/hash_map/struct.DefaultHasher.html


Version 1.12.1 (2016-10-20)
===========================

Regression Fixes
----------------

* [ICE: 'crablangc' panicked at 'assertion failed: concrete_substs.is_normalized_for_trans()' #36381][36381]
* [Confusion with double negation and booleans][36856]
* [crablangc 1.12.0 fails with SIGSEGV in release mode (syn crate 0.8.0)][36875]
* [CrabLangc 1.12.0 Windows build of `ethcore` crate fails with LLVM error][36924]
* [1.12.0: High memory usage when linking in release mode with debug info][36926]
* [Corrupted memory after updated to 1.12][36936]
* ["Let NullaryConstructor = something;" causes internal compiler error: "tried to overwrite interned AdtDef"][37026]
* [Fix ICE: inject bitcast if types mismatch for invokes/calls/stores][37112]
* [debuginfo: Handle spread_arg case in MIR-trans in a more stable way.][37153]

[36381]: https://github.com/crablang/crablang/issues/36381
[36856]: https://github.com/crablang/crablang/issues/36856
[36875]: https://github.com/crablang/crablang/issues/36875
[36924]: https://github.com/crablang/crablang/issues/36924
[36926]: https://github.com/crablang/crablang/issues/36926
[36936]: https://github.com/crablang/crablang/issues/36936
[37026]: https://github.com/crablang/crablang/issues/37026
[37112]: https://github.com/crablang/crablang/issues/37112
[37153]: https://github.com/crablang/crablang/issues/37153


Version 1.12.0 (2016-09-29)
===========================

Highlights
----------

* [`crablangc` translates code to LLVM IR via its own "middle" IR (MIR)](https://github.com/crablang/crablang/pull/34096).
  This translation pass is far simpler than the previous AST->LLVM pass, and
  creates opportunities to perform new optimizations directly on the MIR. It
  was previously described [on the CrabLang blog](https://blog.crablang.org/2016/04/19/MIR.html).
* [`crablangc` presents a new, more readable error format, along with
  machine-readable JSON error output for use by IDEs](https://github.com/crablang/crablang/pull/35401).
  Most common editors supporting CrabLang have been updated to work with it. It was
  previously described [on the CrabLang blog](https://blog.crablang.org/2016/08/10/Shape-of-errors-to-come.html).

Compiler
--------

* [`crablangc` translates code to LLVM IR via its own "middle" IR (MIR)](https://github.com/crablang/crablang/pull/34096).
  This translation pass is far simpler than the previous AST->LLVM pass, and
  creates opportunities to perform new optimizations directly on the MIR. It
  was previously described [on the CrabLang blog](https://blog.crablang.org/2016/04/19/MIR.html).
* [Print the CrabLang target name, not the LLVM target name, with
  `--print target-list`](https://github.com/crablang/crablang/pull/35489)
* [The computation of `TypeId` is correct in some cases where it was previously
  producing inconsistent results](https://github.com/crablang/crablang/pull/35267)
* [The `mips-unknown-linux-gnu` target uses hardware floating point by default](https://github.com/crablang/crablang/pull/34910)
* [The `crablangc` arguments, `--print target-cpus`, `--print target-features`,
  `--print relocation-models`, and `--print code-models` print the available
  options to the `-C target-cpu`, `-C target-feature`, `-C relocation-model` and
  `-C code-model` code generation arguments](https://github.com/crablang/crablang/pull/34845)
* [`crablangc` supports three new MUSL targets on ARM: `arm-unknown-linux-musleabi`,
  `arm-unknown-linux-musleabihf`, and `armv7-unknown-linux-musleabihf`](https://github.com/crablang/crablang/pull/35060).
  These targets produce statically-linked binaries. There are no binary release
  builds yet though.

Diagnostics
-----------

* [`crablangc` presents a new, more readable error format, along with
  machine-readable JSON error output for use by IDEs](https://github.com/crablang/crablang/pull/35401).
  Most common editors supporting CrabLang have been updated to work with it. It was
  previously described [on the CrabLang blog](https://blog.crablang.org/2016/08/10/Shape-of-errors-to-come.html).
* [In error descriptions, references are now described in plain English,
  instead of as "&-ptr"](https://github.com/crablang/crablang/pull/35611)
* [In error type descriptions, unknown numeric types are named `{integer}` or
  `{float}` instead of `_`](https://github.com/crablang/crablang/pull/35080)
* [`crablangc` emits a clearer error when inner attributes follow a doc comment](https://github.com/crablang/crablang/pull/34676)

Language
--------

* [`macro_rules!` invocations can be made within `macro_rules!` invocations](https://github.com/crablang/crablang/pull/34925)
* [`macro_rules!` meta-variables are hygienic](https://github.com/crablang/crablang/pull/35453)
* [`macro_rules!` `tt` matchers can be reparsed correctly, making them much more
  useful](https://github.com/crablang/crablang/pull/34908)
* [`macro_rules!` `stmt` matchers correctly consume the entire contents when
  inside non-braces invocations](https://github.com/crablang/crablang/pull/34886)
* [Semicolons are properly required as statement delimiters inside
  `macro_rules!` invocations](https://github.com/crablang/crablang/pull/34660)
* [`cfg_attr` works on `path` attributes](https://github.com/crablang/crablang/pull/34546)

Stabilized APIs
---------------

* [`Cell::as_ptr`](https://doc.crablang.org/std/cell/struct.Cell.html#method.as_ptr)
* [`RefCell::as_ptr`](https://doc.crablang.org/std/cell/struct.RefCell.html#method.as_ptr)
* [`IpAddr::is_unspecified`](https://doc.crablang.org/std/net/enum.IpAddr.html#method.is_unspecified)
* [`IpAddr::is_loopback`](https://doc.crablang.org/std/net/enum.IpAddr.html#method.is_loopback)
* [`IpAddr::is_multicast`](https://doc.crablang.org/std/net/enum.IpAddr.html#method.is_multicast)
* [`Ipv4Addr::is_unspecified`](https://doc.crablang.org/std/net/struct.Ipv4Addr.html#method.is_unspecified)
* [`Ipv6Addr::octets`](https://doc.crablang.org/std/net/struct.Ipv6Addr.html#method.octets)
* [`LinkedList::contains`](https://doc.crablang.org/std/collections/linked_list/struct.LinkedList.html#method.contains)
* [`VecDeque::contains`](https://doc.crablang.org/std/collections/vec_deque/struct.VecDeque.html#method.contains)
* [`ExitStatusExt::from_raw`](https://doc.crablang.org/std/os/unix/process/trait.ExitStatusExt.html#tymethod.from_raw).
  Both on Unix and Windows.
* [`Receiver::recv_timeout`](https://doc.crablang.org/std/sync/mpsc/struct.Receiver.html#method.recv_timeout)
* [`RecvTimeoutError`](https://doc.crablang.org/std/sync/mpsc/enum.RecvTimeoutError.html)
* [`BinaryHeap::peek_mut`](https://doc.crablang.org/std/collections/binary_heap/struct.BinaryHeap.html#method.peek_mut)
* [`PeekMut`](https://doc.crablang.org/std/collections/binary_heap/struct.PeekMut.html)
* [`iter::Product`](https://doc.crablang.org/std/iter/trait.Product.html)
* [`iter::Sum`](https://doc.crablang.org/std/iter/trait.Sum.html)
* [`OccupiedEntry::remove_entry`](https://doc.crablang.org/std/collections/btree_map/struct.OccupiedEntry.html#method.remove_entry)
* [`VacantEntry::into_key`](https://doc.crablang.org/std/collections/btree_map/struct.VacantEntry.html#method.into_key)

Libraries
---------

* [The `format!` macro and friends now allow a single argument to be formatted
  in multiple styles](https://github.com/crablang/crablang/pull/33642)
* [The lifetime bounds on `[T]::binary_search_by` and
  `[T]::binary_search_by_key` have been adjusted to be more flexible](https://github.com/crablang/crablang/pull/34762)
* [`Option` implements `From` for its contained type](https://github.com/crablang/crablang/pull/34828)
* [`Cell`, `RefCell` and `UnsafeCell` implement `From` for their contained type](https://github.com/crablang/crablang/pull/35392)
* [`RwLock` panics if the reader count overflows](https://github.com/crablang/crablang/pull/35378)
* [`vec_deque::Drain`, `hash_map::Drain` and `hash_set::Drain` are covariant](https://github.com/crablang/crablang/pull/35354)
* [`vec::Drain` and `binary_heap::Drain` are covariant](https://github.com/crablang/crablang/pull/34951)
* [`Cow<str>` implements `FromIterator` for `char`, `&str` and `String`](https://github.com/crablang/crablang/pull/35064)
* [Sockets on Linux are correctly closed in subprocesses via `SOCK_CLOEXEC`](https://github.com/crablang/crablang/pull/34946)
* [`hash_map::Entry`, `hash_map::VacantEntry` and `hash_map::OccupiedEntry`
  implement `Debug`](https://github.com/crablang/crablang/pull/34937)
* [`btree_map::Entry`, `btree_map::VacantEntry` and `btree_map::OccupiedEntry`
  implement `Debug`](https://github.com/crablang/crablang/pull/34885)
* [`String` implements `AddAssign`](https://github.com/crablang/crablang/pull/34890)
* [Variadic `extern fn` pointers implement the `Clone`, `PartialEq`, `Eq`,
  `PartialOrd`, `Ord`, `Hash`, `fmt::Pointer`, and `fmt::Debug` traits](https://github.com/crablang/crablang/pull/34879)
* [`FileType` implements `Debug`](https://github.com/crablang/crablang/pull/34757)
* [References to `Mutex` and `RwLock` are unwind-safe](https://github.com/crablang/crablang/pull/34756)
* [`mpsc::sync_channel` `Receiver`s return any available message before
  reporting a disconnect](https://github.com/crablang/crablang/pull/34731)
* [Unicode definitions have been updated to 9.0](https://github.com/crablang/crablang/pull/34599)
* [`env` iterators implement `DoubleEndedIterator`](https://github.com/crablang/crablang/pull/33312)

Cargo
-----

* [Support local mirrors of registries](https://github.com/crablang/cargo/pull/2857)
* [Add support for command aliases](https://github.com/crablang/cargo/pull/2679)
* [Allow `opt-level="s"` / `opt-level="z"` in profile overrides](https://github.com/crablang/cargo/pull/3007)
* [Make `cargo doc --open --target` work as expected](https://github.com/crablang/cargo/pull/2988)
* [Speed up noop registry updates](https://github.com/crablang/cargo/pull/2974)
* [Update OpenSSL](https://github.com/crablang/cargo/pull/2971)
* [Fix `--panic=abort` with plugins](https://github.com/crablang/cargo/pull/2954)
* [Always pass `-C metadata` to the compiler](https://github.com/crablang/cargo/pull/2946)
* [Fix depending on git repos with workspaces](https://github.com/crablang/cargo/pull/2938)
* [Add a `--lib` flag to `cargo new`](https://github.com/crablang/cargo/pull/2921)
* [Add `http.cainfo` for custom certs](https://github.com/crablang/cargo/pull/2917)
* [Indicate the compilation profile after compiling](https://github.com/crablang/cargo/pull/2909)
* [Allow enabling features for dependencies with `--features`](https://github.com/crablang/cargo/pull/2876)
* [Add `--jobs` flag to `cargo package`](https://github.com/crablang/cargo/pull/2867)
* [Add `--dry-run` to `cargo publish`](https://github.com/crablang/cargo/pull/2849)
* [Add support for `CRABLANGDOCFLAGS`](https://github.com/crablang/cargo/pull/2794)

Performance
-----------

* [`panic::catch_unwind` is more optimized](https://github.com/crablang/crablang/pull/35444)
* [`panic::catch_unwind` no longer accesses thread-local storage on entry](https://github.com/crablang/crablang/pull/34866)

Tooling
-------

* [Test binaries now support a `--test-threads` argument to specify the number
  of threads used to run tests, and which acts the same as the
  `CRABLANG_TEST_THREADS` environment variable](https://github.com/crablang/crablang/pull/35414)
* [The test runner now emits a warning when tests run over 60 seconds](https://github.com/crablang/crablang/pull/35405)
* [crablangdoc: Fix methods in search results](https://github.com/crablang/crablang/pull/34752)
* [`crablang-lldb` warns about unsupported versions of LLDB](https://github.com/crablang/crablang/pull/34646)
* [CrabLang releases now come with source packages that can be installed by crablangup
  via `crablangup component add crablang-src`](https://github.com/crablang/crablang/pull/34366).
  The resulting source code can be used by tools and IDES, located in the
  sysroot under `lib/crablanglib/src`.

Misc
----

* [The compiler can now be built against LLVM 3.9](https://github.com/crablang/crablang/pull/35594)
* Many minor improvements to the documentation.
* [The CrabLang exception handling "personality" routine is now written in CrabLang](https://github.com/crablang/crablang/pull/34832)

Compatibility Notes
-------------------

* [When printing Windows `OsStr`s, unpaired surrogate codepoints are escaped
  with the lowercase format instead of the uppercase](https://github.com/crablang/crablang/pull/35084)
* [When formatting strings, if "precision" is specified, the "fill",
  "align" and "width" specifiers are no longer ignored](https://github.com/crablang/crablang/pull/34544)
* [The `Debug` impl for strings no longer escapes all non-ASCII characters](https://github.com/crablang/crablang/pull/34485)


Version 1.11.0 (2016-08-18)
===========================

Language
--------

* [Support nested `cfg_attr` attributes](https://github.com/crablang/crablang/pull/34216)
* [Allow statement-generating braced macro invocations at the end of blocks](https://github.com/crablang/crablang/pull/34436)
* [Macros can be expanded inside of trait definitions](https://github.com/crablang/crablang/pull/34213)
* [`#[macro_use]` works properly when it is itself expanded from a macro](https://github.com/crablang/crablang/pull/34032)

Stabilized APIs
---------------

* [`BinaryHeap::append`](https://doc.crablang.org/std/collections/binary_heap/struct.BinaryHeap.html#method.append)
* [`BTreeMap::append`](https://doc.crablang.org/std/collections/btree_map/struct.BTreeMap.html#method.append)
* [`BTreeMap::split_off`](https://doc.crablang.org/std/collections/btree_map/struct.BTreeMap.html#method.split_off)
* [`BTreeSet::append`](https://doc.crablang.org/std/collections/btree_set/struct.BTreeSet.html#method.append)
* [`BTreeSet::split_off`](https://doc.crablang.org/std/collections/btree_set/struct.BTreeSet.html#method.split_off)
* [`f32::to_degrees`](https://doc.crablang.org/std/primitive.f32.html#method.to_degrees)
  (in libcore - previously stabilized in libstd)
* [`f32::to_radians`](https://doc.crablang.org/std/primitive.f32.html#method.to_radians)
  (in libcore - previously stabilized in libstd)
* [`f64::to_degrees`](https://doc.crablang.org/std/primitive.f64.html#method.to_degrees)
  (in libcore - previously stabilized in libstd)
* [`f64::to_radians`](https://doc.crablang.org/std/primitive.f64.html#method.to_radians)
  (in libcore - previously stabilized in libstd)
* [`Iterator::sum`](https://doc.crablang.org/std/iter/trait.Iterator.html#method.sum)
* [`Iterator::product`](https://doc.crablang.org/std/iter/trait.Iterator.html#method.sum)
* [`Cell::get_mut`](https://doc.crablang.org/std/cell/struct.Cell.html#method.get_mut)
* [`RefCell::get_mut`](https://doc.crablang.org/std/cell/struct.RefCell.html#method.get_mut)

Libraries
---------

* [The `thread_local!` macro supports multiple definitions in a single
   invocation, and can apply attributes](https://github.com/crablang/crablang/pull/34077)
* [`Cow` implements `Default`](https://github.com/crablang/crablang/pull/34305)
* [`Wrapping` implements binary, octal, lower-hex and upper-hex
  `Display` formatting](https://github.com/crablang/crablang/pull/34190)
* [The range types implement `Hash`](https://github.com/crablang/crablang/pull/34180)
* [`lookup_host` ignores unknown address types](https://github.com/crablang/crablang/pull/34067)
* [`assert_eq!` accepts a custom error message, like `assert!` does](https://github.com/crablang/crablang/pull/33976)
* [The main thread is now called "main" instead of "&lt;main&gt;"](https://github.com/crablang/crablang/pull/33803)

Cargo
-----

* [Disallow specifying features of transitive deps](https://github.com/crablang/cargo/pull/2821)
* [Add color support for Windows consoles](https://github.com/crablang/cargo/pull/2804)
* [Fix `harness = false` on `[lib]` sections](https://github.com/crablang/cargo/pull/2795)
* [Don't panic when `links` contains a '.'](https://github.com/crablang/cargo/pull/2787)
* [Build scripts can emit warnings](https://github.com/crablang/cargo/pull/2630),
  and `-vv` prints warnings for all crates.
* [Ignore file locks on OS X NFS mounts](https://github.com/crablang/cargo/pull/2720)
* [Don't warn about `package.metadata` keys](https://github.com/crablang/cargo/pull/2668).
  This provides room for expansion by arbitrary tools.
* [Add support for cdylib crate types](https://github.com/crablang/cargo/pull/2741)
* [Prevent publishing crates when files are dirty](https://github.com/crablang/cargo/pull/2781)
* [Don't fetch all crates on clean](https://github.com/crablang/cargo/pull/2704)
* [Propagate --color option to crablangc](https://github.com/crablang/cargo/pull/2779)
* [Fix `cargo doc --open` on Windows](https://github.com/crablang/cargo/pull/2780)
* [Improve autocompletion](https://github.com/crablang/cargo/pull/2772)
* [Configure colors of stderr as well as stdout](https://github.com/crablang/cargo/pull/2739)

Performance
-----------

* [Caching projections speeds up type check dramatically for some
  workloads](https://github.com/crablang/crablang/pull/33816)
* [The default `HashMap` hasher is SipHash 1-3 instead of SipHash 2-4](https://github.com/crablang/crablang/pull/33940)
  This hasher is faster, but is believed to provide sufficient
  protection from collision attacks.
* [Comparison of `Ipv4Addr` is 10x faster](https://github.com/crablang/crablang/pull/33891)

CrabLangdoc
-------

* [Fix empty implementation section on some module pages](https://github.com/crablang/crablang/pull/34536)
* [Fix inlined renamed re-exports in import lists](https://github.com/crablang/crablang/pull/34479)
* [Fix search result layout for enum variants and struct fields](https://github.com/crablang/crablang/pull/34477)
* [Fix issues with source links to external crates](https://github.com/crablang/crablang/pull/34387)
* [Fix redirect pages for renamed re-exports](https://github.com/crablang/crablang/pull/34245)

Tooling
-------

* [crablangc is better at finding the MSVC toolchain](https://github.com/crablang/crablang/pull/34492)
* [When emitting debug info, crablangc emits frame pointers for closures,
  shims and glue, as it does for all other functions](https://github.com/crablang/crablang/pull/33909)
* [crablang-lldb warns about unsupported versions of LLDB](https://github.com/crablang/crablang/pull/34646)
* Many more errors have been given error codes and extended
  explanations
* API documentation continues to be improved, with many new examples

Misc
----

* [crablangc no longer hangs when dependencies recursively re-export
  submodules](https://github.com/crablang/crablang/pull/34542)
* [crablangc requires LLVM 3.7+](https://github.com/crablang/crablang/pull/34104)
* [The 'How Safe and Unsafe Interact' chapter of The CrabLangonomicon was
  rewritten](https://github.com/crablang/crablang/pull/33895)
* [crablangc support 16-bit pointer sizes](https://github.com/crablang/crablang/pull/33460).
  No targets use this yet, but it works toward AVR support.

Compatibility Notes
-------------------

* [`const`s and `static`s may not have unsized types](https://github.com/crablang/crablang/pull/34443)
* [The new follow-set rules that place restrictions on `macro_rules!`
  in order to ensure syntax forward-compatibility have been enabled](https://github.com/crablang/crablang/pull/33982)
  This was an [amendment to RFC 550](https://github.com/crablang/rfcs/pull/1384),
  and has been a warning since 1.10.
* [`cfg` attribute process has been refactored to fix various bugs](https://github.com/crablang/crablang/pull/33706).
  This causes breakage in some corner cases.


Version 1.10.0 (2016-07-07)
===========================

Language
--------

* [`Copy` types are required to have a trivial implementation of `Clone`](https://github.com/crablang/crablang/pull/33420).
  [RFC 1521](https://github.com/crablang/rfcs/blob/master/text/1521-copy-clone-semantics.md).
* [Single-variant enums support the `#[repr(..)]` attribute](https://github.com/crablang/crablang/pull/33355).
* [Fix `#[derive(CrabLangcEncodable)]` in the presence of other `encode` methods](https://github.com/crablang/crablang/pull/32908).
* [`panic!` can be converted to a runtime abort with the
  `-C panic=abort` flag](https://github.com/crablang/crablang/pull/32900).
  [RFC 1513](https://github.com/crablang/rfcs/blob/master/text/1513-less-unwinding.md).
* [Add a new crate type, 'cdylib'](https://github.com/crablang/crablang/pull/33553).
  cdylibs are dynamic libraries suitable for loading by non-CrabLang hosts.
  [RFC 1510](https://github.com/crablang/rfcs/blob/master/text/1510-cdylib.md).
  Note that Cargo does not yet directly support cdylibs.

Stabilized APIs
---------------

* `os::windows::fs::OpenOptionsExt::access_mode`
* `os::windows::fs::OpenOptionsExt::share_mode`
* `os::windows::fs::OpenOptionsExt::custom_flags`
* `os::windows::fs::OpenOptionsExt::attributes`
* `os::windows::fs::OpenOptionsExt::security_qos_flags`
* `os::unix::fs::OpenOptionsExt::custom_flags`
* [`sync::Weak::new`](http://doc.crablang.org/alloc/arc/struct.Weak.html#method.new)
* `Default for sync::Weak`
* [`panic::set_hook`](http://doc.crablang.org/std/panic/fn.set_hook.html)
* [`panic::take_hook`](http://doc.crablang.org/std/panic/fn.take_hook.html)
* [`panic::PanicInfo`](http://doc.crablang.org/std/panic/struct.PanicInfo.html)
* [`panic::PanicInfo::payload`](http://doc.crablang.org/std/panic/struct.PanicInfo.html#method.payload)
* [`panic::PanicInfo::location`](http://doc.crablang.org/std/panic/struct.PanicInfo.html#method.location)
* [`panic::Location`](http://doc.crablang.org/std/panic/struct.Location.html)
* [`panic::Location::file`](http://doc.crablang.org/std/panic/struct.Location.html#method.file)
* [`panic::Location::line`](http://doc.crablang.org/std/panic/struct.Location.html#method.line)
* [`ffi::CStr::from_bytes_with_nul`](http://doc.crablang.org/std/ffi/struct.CStr.html#method.from_bytes_with_nul)
* [`ffi::CStr::from_bytes_with_nul_unchecked`](http://doc.crablang.org/std/ffi/struct.CStr.html#method.from_bytes_with_nul_unchecked)
* [`ffi::FromBytesWithNulError`](http://doc.crablang.org/std/ffi/struct.FromBytesWithNulError.html)
* [`fs::Metadata::modified`](http://doc.crablang.org/std/fs/struct.Metadata.html#method.modified)
* [`fs::Metadata::accessed`](http://doc.crablang.org/std/fs/struct.Metadata.html#method.accessed)
* [`fs::Metadata::created`](http://doc.crablang.org/std/fs/struct.Metadata.html#method.created)
* `sync::atomic::Atomic{Usize,Isize,Bool,Ptr}::compare_exchange`
* `sync::atomic::Atomic{Usize,Isize,Bool,Ptr}::compare_exchange_weak`
* `collections::{btree,hash}_map::{Occupied,Vacant,}Entry::key`
* `os::unix::net::{UnixStream, UnixListener, UnixDatagram, SocketAddr}`
* [`SocketAddr::is_unnamed`](http://doc.crablang.org/std/os/unix/net/struct.SocketAddr.html#method.is_unnamed)
* [`SocketAddr::as_pathname`](http://doc.crablang.org/std/os/unix/net/struct.SocketAddr.html#method.as_pathname)
* [`UnixStream::connect`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.connect)
* [`UnixStream::pair`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.pair)
* [`UnixStream::try_clone`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.try_clone)
* [`UnixStream::local_addr`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.local_addr)
* [`UnixStream::peer_addr`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.peer_addr)
* [`UnixStream::set_read_timeout`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.read_timeout)
* [`UnixStream::set_write_timeout`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.write_timeout)
* [`UnixStream::read_timeout`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.read_timeout)
* [`UnixStream::write_timeout`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.write_timeout)
* [`UnixStream::set_nonblocking`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.set_nonblocking)
* [`UnixStream::take_error`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.take_error)
* [`UnixStream::shutdown`](http://doc.crablang.org/std/os/unix/net/struct.UnixStream.html#method.shutdown)
* Read/Write/RawFd impls for `UnixStream`
* [`UnixListener::bind`](http://doc.crablang.org/std/os/unix/net/struct.UnixListener.html#method.bind)
* [`UnixListener::accept`](http://doc.crablang.org/std/os/unix/net/struct.UnixListener.html#method.accept)
* [`UnixListener::try_clone`](http://doc.crablang.org/std/os/unix/net/struct.UnixListener.html#method.try_clone)
* [`UnixListener::local_addr`](http://doc.crablang.org/std/os/unix/net/struct.UnixListener.html#method.local_addr)
* [`UnixListener::set_nonblocking`](http://doc.crablang.org/std/os/unix/net/struct.UnixListener.html#method.set_nonblocking)
* [`UnixListener::take_error`](http://doc.crablang.org/std/os/unix/net/struct.UnixListener.html#method.take_error)
* [`UnixListener::incoming`](http://doc.crablang.org/std/os/unix/net/struct.UnixListener.html#method.incoming)
* RawFd impls for `UnixListener`
* [`UnixDatagram::bind`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.bind)
* [`UnixDatagram::unbound`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.unbound)
* [`UnixDatagram::pair`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.pair)
* [`UnixDatagram::connect`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.connect)
* [`UnixDatagram::try_clone`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.try_clone)
* [`UnixDatagram::local_addr`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.local_addr)
* [`UnixDatagram::peer_addr`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.peer_addr)
* [`UnixDatagram::recv_from`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.recv_from)
* [`UnixDatagram::recv`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.recv)
* [`UnixDatagram::send_to`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.send_to)
* [`UnixDatagram::send`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.send)
* [`UnixDatagram::set_read_timeout`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.set_read_timeout)
* [`UnixDatagram::set_write_timeout`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.set_write_timeout)
* [`UnixDatagram::read_timeout`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.read_timeout)
* [`UnixDatagram::write_timeout`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.write_timeout)
* [`UnixDatagram::set_nonblocking`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.set_nonblocking)
* [`UnixDatagram::take_error`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.take_error)
* [`UnixDatagram::shutdown`](http://doc.crablang.org/std/os/unix/net/struct.UnixDatagram.html#method.shutdown)
* RawFd impls for `UnixDatagram`
* `{BTree,Hash}Map::values_mut`
* [`<[_]>::binary_search_by_key`](http://doc.crablang.org/std/primitive.slice.html#method.binary_search_by_key)

Libraries
---------

* [The `abs_sub` method of floats is deprecated](https://github.com/crablang/crablang/pull/33664).
  The semantics of this minor method are subtle and probably not what
  most people want.
* [Add implementation of Ord for Cell<T> and RefCell<T> where T: Ord](https://github.com/crablang/crablang/pull/33306).
* [On Linux, if `HashMap`s can't be initialized with `getrandom` they
  will fall back to `/dev/urandom` temporarily to avoid blocking
  during early boot](https://github.com/crablang/crablang/pull/33086).
* [Implemented negation for wrapping numerals](https://github.com/crablang/crablang/pull/33067).
* [Implement `Clone` for `binary_heap::IntoIter`](https://github.com/crablang/crablang/pull/33050).
* [Implement `Display` and `Hash` for `std::num::Wrapping`](https://github.com/crablang/crablang/pull/33023).
* [Add `Default` implementation for `&CStr`, `CString`](https://github.com/crablang/crablang/pull/32990).
* [Implement `From<Vec<T>>` and `Into<Vec<T>>` for `VecDeque<T>`](https://github.com/crablang/crablang/pull/32866).
* [Implement `Default` for `UnsafeCell`, `fmt::Error`, `Condvar`,
  `Mutex`, `RwLock`](https://github.com/crablang/crablang/pull/32785).

Cargo
-----
* [Cargo.toml supports the `profile.*.panic` option](https://github.com/crablang/cargo/pull/2687).
  This controls the runtime behavior of the `panic!` macro
  and can be either "unwind" (the default), or "abort".
  [RFC 1513](https://github.com/crablang/rfcs/blob/master/text/1513-less-unwinding.md).
* [Don't throw away errors with `-p` arguments](https://github.com/crablang/cargo/pull/2723).
* [Report status to stderr instead of stdout](https://github.com/crablang/cargo/pull/2693).
* [Build scripts are passed a `CARGO_MANIFEST_LINKS` environment
  variable that corresponds to the `links` field of the manifest](https://github.com/crablang/cargo/pull/2710).
* [Ban keywords from crate names](https://github.com/crablang/cargo/pull/2707).
* [Canonicalize `CARGO_HOME` on Windows](https://github.com/crablang/cargo/pull/2604).
* [Retry network requests](https://github.com/crablang/cargo/pull/2396).
  By default they are retried twice, which can be customized with the
  `net.retry` value in `.cargo/config`.
* [Don't print extra error info for failing subcommands](https://github.com/crablang/cargo/pull/2674).
* [Add `--force` flag to `cargo install`](https://github.com/crablang/cargo/pull/2405).
* [Don't use `flock` on NFS mounts](https://github.com/crablang/cargo/pull/2623).
* [Prefer building `cargo install` artifacts in temporary directories](https://github.com/crablang/cargo/pull/2610).
  Makes it possible to install multiple crates in parallel.
* [Add `cargo test --doc`](https://github.com/crablang/cargo/pull/2578).
* [Add `cargo --explain`](https://github.com/crablang/cargo/pull/2551).
* [Don't print warnings when `-q` is passed](https://github.com/crablang/cargo/pull/2576).
* [Add `cargo doc --lib` and `--bin`](https://github.com/crablang/cargo/pull/2577).
* [Don't require build script output to be UTF-8](https://github.com/crablang/cargo/pull/2560).
* [Correctly attempt multiple git usernames](https://github.com/crablang/cargo/pull/2584).

Performance
-----------

* [crablangc memory usage was reduced by refactoring the context used for
  type checking](https://github.com/crablang/crablang/pull/33425).
* [Speed up creation of `HashMap`s by caching the random keys used
  to initialize the hash state](https://github.com/crablang/crablang/pull/33318).
* [The `find` implementation for `Chain` iterators is 2x faster](https://github.com/crablang/crablang/pull/33289).
* [Trait selection optimizations speed up type checking by 15%](https://github.com/crablang/crablang/pull/33138).
* [Efficient trie lookup for boolean Unicode properties](https://github.com/crablang/crablang/pull/33098).
  10x faster than the previous lookup tables.
* [Special case `#[derive(Copy, Clone)]` to avoid bloat](https://github.com/crablang/crablang/pull/31414).

Usability
---------

* Many incremental improvements to documentation and crablangdoc.
* [crablangdoc: List blanket trait impls](https://github.com/crablang/crablang/pull/33514).
* [crablangdoc: Clean up ABI rendering](https://github.com/crablang/crablang/pull/33151).
* [Indexing with the wrong type produces a more informative error](https://github.com/crablang/crablang/pull/33401).
* [Improve diagnostics for constants being used in irrefutable patterns](https://github.com/crablang/crablang/pull/33406).
* [When many method candidates are in scope limit the suggestions to 10](https://github.com/crablang/crablang/pull/33338).
* [Remove confusing suggestion when calling a `fn` type](https://github.com/crablang/crablang/pull/33325).
* [Do not suggest changing `&mut self` to `&mut mut self`](https://github.com/crablang/crablang/pull/33319).

Misc
----

* [Update i686-linux-android features to match Android ABI](https://github.com/crablang/crablang/pull/33651).
* [Update aarch64-linux-android features to match Android ABI](https://github.com/crablang/crablang/pull/33500).
* [`std` no longer prints backtraces on platforms where the running
  module must be loaded with `env::current_exe`, which can't be relied
  on](https://github.com/crablang/crablang/pull/33554).
* This release includes std binaries for the i586-unknown-linux-gnu,
  i686-unknown-linux-musl, and armv7-linux-androideabi targets. The
  i586 target is for old x86 hardware without SSE2, and the armv7
  target is for Android running on modern ARM architectures.
* [The `crablang-gdb` and `crablang-lldb` scripts are distributed on all
  Unix platforms](https://github.com/crablang/crablang/pull/32835).
* [On Unix the runtime aborts by calling `libc::abort` instead of
  generating an illegal instruction](https://github.com/crablang/crablang/pull/31457).
* [CrabLang is now bootstrapped from the previous release of CrabLang,
  instead of a snapshot from an arbitrary commit](https://github.com/crablang/crablang/pull/32942).

Compatibility Notes
-------------------

* [`AtomicBool` is now bool-sized, not word-sized](https://github.com/crablang/crablang/pull/33579).
* [`target_env` for Linux ARM targets is just `gnu`, not
  `gnueabihf`, `gnueabi`, etc](https://github.com/crablang/crablang/pull/33403).
* [Consistently panic on overflow in `Duration::new`](https://github.com/crablang/crablang/pull/33072).
* [Change `String::truncate` to panic less](https://github.com/crablang/crablang/pull/32977).
* [Add `:block` to the follow set for `:ty` and `:path`](https://github.com/crablang/crablang/pull/32945).
  Affects how macros are parsed.
* [Fix macro hygiene bug](https://github.com/crablang/crablang/pull/32923).
* [Feature-gated attributes on macro-generated macro invocations are
  now rejected](https://github.com/crablang/crablang/pull/32791).
* [Suppress fallback and ambiguity errors during type inference](https://github.com/crablang/crablang/pull/32258).
  This caused some minor changes to type inference.


Version 1.9.0 (2016-05-26)
==========================

Language
--------

* The `#[deprecated]` attribute when applied to an API will generate
  warnings when used. The warnings may be suppressed with
  `#[allow(deprecated)]`. [RFC 1270].
* [`fn` item types are zero sized, and each `fn` names a unique
  type][1.9fn]. This will break code that transmutes `fn`s, so calling
  `transmute` on a `fn` type will generate a warning for a few cycles,
  then will be converted to an error.
* [Field and method resolution understand visibility, so private
  fields and methods cannot prevent the proper use of public fields
  and methods][1.9fv].
* [The parser considers unicode codepoints in the
  `PATTERN_WHITE_SPACE` category to be whitespace][1.9ws].

Stabilized APIs
---------------

* [`std::panic`]
* [`std::panic::catch_unwind`] (renamed from `recover`)
* [`std::panic::resume_unwind`] (renamed from `propagate`)
* [`std::panic::AssertUnwindSafe`] (renamed from `AssertRecoverSafe`)
* [`std::panic::UnwindSafe`] (renamed from `RecoverSafe`)
* [`str::is_char_boundary`]
* [`<*const T>::as_ref`]
* [`<*mut T>::as_ref`]
* [`<*mut T>::as_mut`]
* [`AsciiExt::make_ascii_uppercase`]
* [`AsciiExt::make_ascii_lowercase`]
* [`char::decode_utf16`]
* [`char::DecodeUtf16`]
* [`char::DecodeUtf16Error`]
* [`char::DecodeUtf16Error::unpaired_surrogate`]
* [`BTreeSet::take`]
* [`BTreeSet::replace`]
* [`BTreeSet::get`]
* [`HashSet::take`]
* [`HashSet::replace`]
* [`HashSet::get`]
* [`OsString::with_capacity`]
* [`OsString::clear`]
* [`OsString::capacity`]
* [`OsString::reserve`]
* [`OsString::reserve_exact`]
* [`OsStr::is_empty`]
* [`OsStr::len`]
* [`std::os::unix::thread`]
* [`RawPthread`]
* [`JoinHandleExt`]
* [`JoinHandleExt::as_pthread_t`]
* [`JoinHandleExt::into_pthread_t`]
* [`HashSet::hasher`]
* [`HashMap::hasher`]
* [`CommandExt::exec`]
* [`File::try_clone`]
* [`SocketAddr::set_ip`]
* [`SocketAddr::set_port`]
* [`SocketAddrV4::set_ip`]
* [`SocketAddrV4::set_port`]
* [`SocketAddrV6::set_ip`]
* [`SocketAddrV6::set_port`]
* [`SocketAddrV6::set_flowinfo`]
* [`SocketAddrV6::set_scope_id`]
* [`slice::copy_from_slice`]
* [`ptr::read_volatile`]
* [`ptr::write_volatile`]
* [`OpenOptions::create_new`]
* [`TcpStream::set_nodelay`]
* [`TcpStream::nodelay`]
* [`TcpStream::set_ttl`]
* [`TcpStream::ttl`]
* [`TcpStream::set_only_v6`]
* [`TcpStream::only_v6`]
* [`TcpStream::take_error`]
* [`TcpStream::set_nonblocking`]
* [`TcpListener::set_ttl`]
* [`TcpListener::ttl`]
* [`TcpListener::set_only_v6`]
* [`TcpListener::only_v6`]
* [`TcpListener::take_error`]
* [`TcpListener::set_nonblocking`]
* [`UdpSocket::set_broadcast`]
* [`UdpSocket::broadcast`]
* [`UdpSocket::set_multicast_loop_v4`]
* [`UdpSocket::multicast_loop_v4`]
* [`UdpSocket::set_multicast_ttl_v4`]
* [`UdpSocket::multicast_ttl_v4`]
* [`UdpSocket::set_multicast_loop_v6`]
* [`UdpSocket::multicast_loop_v6`]
* [`UdpSocket::set_multicast_ttl_v6`]
* [`UdpSocket::multicast_ttl_v6`]
* [`UdpSocket::set_ttl`]
* [`UdpSocket::ttl`]
* [`UdpSocket::set_only_v6`]
* [`UdpSocket::only_v6`]
* [`UdpSocket::join_multicast_v4`]
* [`UdpSocket::join_multicast_v6`]
* [`UdpSocket::leave_multicast_v4`]
* [`UdpSocket::leave_multicast_v6`]
* [`UdpSocket::take_error`]
* [`UdpSocket::connect`]
* [`UdpSocket::send`]
* [`UdpSocket::recv`]
* [`UdpSocket::set_nonblocking`]

Libraries
---------

* [`std::sync::Once` is poisoned if its initialization function
  fails][1.9o].
* [`cell::Ref` and `cell::RefMut` can contain unsized types][1.9cu].
* [Most types implement `fmt::Debug`][1.9db].
* [The default buffer size used by `BufReader` and `BufWriter` was
  reduced to 8K, from 64K][1.9bf]. This is in line with the buffer size
  used by other languages.
* [`Instant`, `SystemTime` and `Duration` implement `+=` and `-=`.
  `Duration` additionally implements `*=` and `/=`][1.9ta].
* [`Skip` is a `DoubleEndedIterator`][1.9sk].
* [`From<[u8; 4]>` is implemented for `Ipv4Addr`][1.9fi].
* [`Chain` implements `BufRead`][1.9ch].
* [`HashMap`, `HashSet` and iterators are covariant][1.9hc].

Cargo
-----

* [Cargo can now run concurrently][1.9cc].
* [Top-level overrides allow specific revisions of crates to be
  overridden through the entire crate graph][1.9ct].  This is intended
  to make upgrades easier for large projects, by allowing crates to be
  forked temporarily until they've been upgraded and republished.
* [Cargo exports a `CARGO_PKG_AUTHORS` environment variable][1.9cp].
* [Cargo will pass the contents of the `CRABLANGFLAGS` variable to `crablangc`
  on the commandline][1.9cf]. `crablangc` arguments can also be specified
  in the `build.crablangflags` configuration key.

Performance
-----------

* [The time complexity of comparing variables for equivalence during type
  unification is reduced from _O_(_n_!) to _O_(_n_)][1.9tu]. This leads
  to major compilation time improvement in some scenarios.
* [`ToString` is specialized for `str`, giving it the same performance
  as `to_owned`][1.9ts].
* [Spawning processes with `Command::output` no longer creates extra
  threads][1.9sp].
* [`#[derive(PartialEq)]` and `#[derive(PartialOrd)]` emit less code
  for C-like enums][1.9cl].

Misc
----

* [Passing the `--quiet` flag to a test runner will produce
  much-abbreviated output][1.9q].
* The CrabLang Project now publishes std binaries for the
  `mips-unknown-linux-musl`, `mipsel-unknown-linux-musl`, and
  `i586-pc-windows-msvc` targets.

Compatibility Notes
-------------------

* [`std::sync::Once` is poisoned if its initialization function
  fails][1.9o].
* [It is illegal to define methods with the same name in overlapping
  inherent `impl` blocks][1.9sn].
* [`fn` item types are zero sized, and each `fn` names a unique
  type][1.9fn]. This will break code that transmutes `fn`s, so calling
  `transmute` on a `fn` type will generate a warning for a few cycles,
  then will be converted to an error.
* [Improvements to const evaluation may trigger new errors when integer
  literals are out of range][1.9ce].


[1.9bf]: https://github.com/crablang/crablang/pull/32695
[1.9cc]: https://github.com/crablang/cargo/pull/2486
[1.9ce]: https://github.com/crablang/crablang/pull/30587
[1.9cf]: https://github.com/crablang/cargo/pull/2241
[1.9ch]: https://github.com/crablang/crablang/pull/32541
[1.9cl]: https://github.com/crablang/crablang/pull/31977
[1.9cp]: https://github.com/crablang/cargo/pull/2465
[1.9ct]: https://github.com/crablang/cargo/pull/2385
[1.9cu]: https://github.com/crablang/crablang/pull/32652
[1.9db]: https://github.com/crablang/crablang/pull/32054
[1.9fi]: https://github.com/crablang/crablang/pull/32050
[1.9fn]: https://github.com/crablang/crablang/pull/31710
[1.9fv]: https://github.com/crablang/crablang/pull/31938
[1.9hc]: https://github.com/crablang/crablang/pull/32635
[1.9o]: https://github.com/crablang/crablang/pull/32325
[1.9q]: https://github.com/crablang/crablang/pull/31887
[1.9sk]: https://github.com/crablang/crablang/pull/31700
[1.9sn]: https://github.com/crablang/crablang/pull/31925
[1.9sp]: https://github.com/crablang/crablang/pull/31618
[1.9ta]: https://github.com/crablang/crablang/pull/32448
[1.9ts]: https://github.com/crablang/crablang/pull/32586
[1.9tu]: https://github.com/crablang/crablang/pull/32062
[1.9ws]: https://github.com/crablang/crablang/pull/29734
[RFC 1270]: https://github.com/crablang/rfcs/blob/master/text/1270-deprecation.md
[`<*const T>::as_ref`]: http://doc.crablang.org/nightly/std/primitive.pointer.html#method.as_ref
[`<*mut T>::as_mut`]: http://doc.crablang.org/nightly/std/primitive.pointer.html#method.as_mut
[`<*mut T>::as_ref`]: http://doc.crablang.org/nightly/std/primitive.pointer.html#method.as_ref
[`slice::copy_from_slice`]: http://doc.crablang.org/nightly/std/primitive.slice.html#method.copy_from_slice
[`AsciiExt::make_ascii_lowercase`]: http://doc.crablang.org/nightly/std/ascii/trait.AsciiExt.html#tymethod.make_ascii_lowercase
[`AsciiExt::make_ascii_uppercase`]: http://doc.crablang.org/nightly/std/ascii/trait.AsciiExt.html#tymethod.make_ascii_uppercase
[`BTreeSet::get`]: http://doc.crablang.org/nightly/collections/btree/set/struct.BTreeSet.html#method.get
[`BTreeSet::replace`]: http://doc.crablang.org/nightly/collections/btree/set/struct.BTreeSet.html#method.replace
[`BTreeSet::take`]: http://doc.crablang.org/nightly/collections/btree/set/struct.BTreeSet.html#method.take
[`CommandExt::exec`]: http://doc.crablang.org/nightly/std/os/unix/process/trait.CommandExt.html#tymethod.exec
[`File::try_clone`]: http://doc.crablang.org/nightly/std/fs/struct.File.html#method.try_clone
[`HashMap::hasher`]: http://doc.crablang.org/nightly/std/collections/struct.HashMap.html#method.hasher
[`HashSet::get`]: http://doc.crablang.org/nightly/std/collections/struct.HashSet.html#method.get
[`HashSet::hasher`]: http://doc.crablang.org/nightly/std/collections/struct.HashSet.html#method.hasher
[`HashSet::replace`]: http://doc.crablang.org/nightly/std/collections/struct.HashSet.html#method.replace
[`HashSet::take`]: http://doc.crablang.org/nightly/std/collections/struct.HashSet.html#method.take
[`JoinHandleExt::as_pthread_t`]: http://doc.crablang.org/nightly/std/os/unix/thread/trait.JoinHandleExt.html#tymethod.as_pthread_t
[`JoinHandleExt::into_pthread_t`]: http://doc.crablang.org/nightly/std/os/unix/thread/trait.JoinHandleExt.html#tymethod.into_pthread_t
[`JoinHandleExt`]: http://doc.crablang.org/nightly/std/os/unix/thread/trait.JoinHandleExt.html
[`OpenOptions::create_new`]: http://doc.crablang.org/nightly/std/fs/struct.OpenOptions.html#method.create_new
[`OsStr::is_empty`]: http://doc.crablang.org/nightly/std/ffi/struct.OsStr.html#method.is_empty
[`OsStr::len`]: http://doc.crablang.org/nightly/std/ffi/struct.OsStr.html#method.len
[`OsString::capacity`]: http://doc.crablang.org/nightly/std/ffi/struct.OsString.html#method.capacity
[`OsString::clear`]: http://doc.crablang.org/nightly/std/ffi/struct.OsString.html#method.clear
[`OsString::reserve_exact`]: http://doc.crablang.org/nightly/std/ffi/struct.OsString.html#method.reserve_exact
[`OsString::reserve`]: http://doc.crablang.org/nightly/std/ffi/struct.OsString.html#method.reserve
[`OsString::with_capacity`]: http://doc.crablang.org/nightly/std/ffi/struct.OsString.html#method.with_capacity
[`RawPthread`]: http://doc.crablang.org/nightly/std/os/unix/thread/type.RawPthread.html
[`SocketAddr::set_ip`]: http://doc.crablang.org/nightly/std/net/enum.SocketAddr.html#method.set_ip
[`SocketAddr::set_port`]: http://doc.crablang.org/nightly/std/net/enum.SocketAddr.html#method.set_port
[`SocketAddrV4::set_ip`]: http://doc.crablang.org/nightly/std/net/struct.SocketAddrV4.html#method.set_ip
[`SocketAddrV4::set_port`]: http://doc.crablang.org/nightly/std/net/struct.SocketAddrV4.html#method.set_port
[`SocketAddrV6::set_flowinfo`]: http://doc.crablang.org/nightly/std/net/struct.SocketAddrV6.html#method.set_flowinfo
[`SocketAddrV6::set_ip`]: http://doc.crablang.org/nightly/std/net/struct.SocketAddrV6.html#method.set_ip
[`SocketAddrV6::set_port`]: http://doc.crablang.org/nightly/std/net/struct.SocketAddrV6.html#method.set_port
[`SocketAddrV6::set_scope_id`]: http://doc.crablang.org/nightly/std/net/struct.SocketAddrV6.html#method.set_scope_id
[`TcpListener::only_v6`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.only_v6
[`TcpListener::set_nonblocking`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_nonblocking
[`TcpListener::set_only_v6`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_only_v6
[`TcpListener::set_ttl`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_ttl
[`TcpListener::take_error`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.take_error
[`TcpListener::ttl`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.ttl
[`TcpStream::nodelay`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.nodelay
[`TcpStream::only_v6`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.only_v6
[`TcpStream::set_nodelay`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_nodelay
[`TcpStream::set_nonblocking`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_nonblocking
[`TcpStream::set_only_v6`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_only_v6
[`TcpStream::set_ttl`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_ttl
[`TcpStream::take_error`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.take_error
[`TcpStream::ttl`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.ttl
[`UdpSocket::broadcast`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.broadcast
[`UdpSocket::connect`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.connect
[`UdpSocket::join_multicast_v4`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.join_multicast_v4
[`UdpSocket::join_multicast_v6`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.join_multicast_v6
[`UdpSocket::leave_multicast_v4`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.leave_multicast_v4
[`UdpSocket::leave_multicast_v6`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.leave_multicast_v6
[`UdpSocket::multicast_loop_v4`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.multicast_loop_v4
[`UdpSocket::multicast_loop_v6`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.multicast_loop_v6
[`UdpSocket::multicast_ttl_v4`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.multicast_ttl_v4
[`UdpSocket::multicast_ttl_v6`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.multicast_ttl_v6
[`UdpSocket::only_v6`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.only_v6
[`UdpSocket::recv`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.recv
[`UdpSocket::send`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.send
[`UdpSocket::set_broadcast`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.set_broadcast
[`UdpSocket::set_multicast_loop_v4`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.set_multicast_loop_v4
[`UdpSocket::set_multicast_loop_v6`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.set_multicast_loop_v6
[`UdpSocket::set_multicast_ttl_v4`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.set_multicast_ttl_v4
[`UdpSocket::set_multicast_ttl_v6`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.set_multicast_ttl_v6
[`UdpSocket::set_nonblocking`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.set_nonblocking
[`UdpSocket::set_only_v6`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.set_only_v6
[`UdpSocket::set_ttl`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.set_ttl
[`UdpSocket::take_error`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.take_error
[`UdpSocket::ttl`]: http://doc.crablang.org/nightly/std/net/struct.UdpSocket.html#method.ttl
[`char::DecodeUtf16Error::unpaired_surrogate`]: http://doc.crablang.org/nightly/std/char/struct.DecodeUtf16Error.html#method.unpaired_surrogate
[`char::DecodeUtf16Error`]: http://doc.crablang.org/nightly/std/char/struct.DecodeUtf16Error.html
[`char::DecodeUtf16`]: http://doc.crablang.org/nightly/std/char/struct.DecodeUtf16.html
[`char::decode_utf16`]: http://doc.crablang.org/nightly/std/char/fn.decode_utf16.html
[`ptr::read_volatile`]: http://doc.crablang.org/nightly/std/ptr/fn.read_volatile.html
[`ptr::write_volatile`]: http://doc.crablang.org/nightly/std/ptr/fn.write_volatile.html
[`std::os::unix::thread`]: http://doc.crablang.org/nightly/std/os/unix/thread/index.html
[`std::panic::AssertUnwindSafe`]: http://doc.crablang.org/nightly/std/panic/struct.AssertUnwindSafe.html
[`std::panic::UnwindSafe`]: http://doc.crablang.org/nightly/std/panic/trait.UnwindSafe.html
[`std::panic::catch_unwind`]: http://doc.crablang.org/nightly/std/panic/fn.catch_unwind.html
[`std::panic::resume_unwind`]: http://doc.crablang.org/nightly/std/panic/fn.resume_unwind.html
[`std::panic`]: http://doc.crablang.org/nightly/std/panic/index.html
[`str::is_char_boundary`]: http://doc.crablang.org/nightly/std/primitive.str.html#method.is_char_boundary


Version 1.8.0 (2016-04-14)
==========================

Language
--------

* CrabLang supports overloading of compound assignment statements like
  `+=` by implementing the [`AddAssign`], [`SubAssign`],
  [`MulAssign`], [`DivAssign`], [`RemAssign`], [`BitAndAssign`],
  [`BitOrAssign`], [`BitXorAssign`], [`ShlAssign`], or [`ShrAssign`]
  traits. [RFC 953].
* Empty structs can be defined with braces, as in `struct Foo { }`, in
  addition to the non-braced form, `struct Foo;`. [RFC 218].

Libraries
---------

* Stabilized APIs:
  * [`str::encode_utf16`] (renamed from `utf16_units`)
  * [`str::EncodeUtf16`] (renamed from `Utf16Units`)
  * [`Ref::map`]
  * [`RefMut::map`]
  * [`ptr::drop_in_place`]
  * [`time::Instant`]
  * [`time::SystemTime`]
  * [`Instant::now`]
  * [`Instant::duration_since`] (renamed from `duration_from_earlier`)
  * [`Instant::elapsed`]
  * [`SystemTime::now`]
  * [`SystemTime::duration_since`] (renamed from `duration_from_earlier`)
  * [`SystemTime::elapsed`]
  * Various `Add`/`Sub` impls for `Time` and `SystemTime`
  * [`SystemTimeError`]
  * [`SystemTimeError::duration`]
  * Various impls for `SystemTimeError`
  * [`UNIX_EPOCH`]
  * [`AddAssign`], [`SubAssign`], [`MulAssign`], [`DivAssign`],
    [`RemAssign`], [`BitAndAssign`], [`BitOrAssign`],
    [`BitXorAssign`], [`ShlAssign`], [`ShrAssign`].
* [The `write!` and `writeln!` macros correctly emit errors if any of
  their arguments can't be formatted][1.8w].
* [Various I/O functions support large files on 32-bit Linux][1.8l].
* [The Unix-specific `raw` modules, which contain a number of
  redefined C types are deprecated][1.8r], including `os::raw::unix`,
  `os::raw::macos`, and `os::raw::linux`. These modules defined types
  such as `ino_t` and `dev_t`. The inconsistency of these definitions
  across platforms was making it difficult to implement `std`
  correctly. Those that need these definitions should use the `libc`
  crate. [RFC 1415].
* The Unix-specific `MetadataExt` traits, including
  `os::unix::fs::MetadataExt`, which expose values such as inode
  numbers [no longer return platform-specific types][1.8r], but
  instead return widened integers. [RFC 1415].
* [`btree_set::{IntoIter, Iter, Range}` are covariant][1.8cv].
* [Atomic loads and stores are not volatile][1.8a].
* [All types in `sync::mpsc` implement `fmt::Debug`][1.8mp].

Performance
-----------

* [Inlining hash functions lead to a 3% compile-time improvement in
  some workloads][1.8h].
* When using jemalloc, its symbols are [unprefixed so that it
  overrides the libc malloc implementation][1.8h]. This means that for
  crablangc, LLVM is now using jemalloc, which results in a 6%
  compile-time improvement on a specific workload.
* [Avoid quadratic growth in function size due to cleanups][1.8cu].

Misc
----

* [32-bit MSVC builds finally implement unwinding][1.8ms].
  i686-pc-windows-msvc is now considered a tier-1 platform.
* [The `--print targets` flag prints a list of supported targets][1.8t].
* [The `--print cfg` flag prints the `cfg`s defined for the current
  target][1.8cf].
* [`crablangc` can be built with an new Cargo-based build system, written
  in CrabLang][1.8b].  It will eventually replace CrabLang's Makefile-based
  build system. To enable it configure with `configure --crablangbuild`.
* [Errors for non-exhaustive `match` patterns now list up to 3 missing
  variants while also indicating the total number of missing variants
  if more than 3][1.8m].
* [Executable stacks are disabled on Linux and BSD][1.8nx].
* The CrabLang Project now publishes binary releases of the standard
  library for a number of tier-2 targets:
  `armv7-unknown-linux-gnueabihf`, `powerpc-unknown-linux-gnu`,
  `powerpc64-unknown-linux-gnu`, `powerpc64le-unknown-linux-gnu`
  `x86_64-rumprun-netbsd`. These can be installed with
  tools such as [multicrablang][1.8mr].

Cargo
-----

* [`cargo init` creates a new Cargo project in the current
  directory][1.8ci].  It is otherwise like `cargo new`.
* [Cargo has configuration keys for `-v` and
  `--color`][1.8cc]. `verbose` and `color`, respectively, go in the
  `[term]` section of `.cargo/config`.
* [Configuration keys that evaluate to strings or integers can be set
  via environment variables][1.8ce]. For example the `build.jobs` key
  can be set via `CARGO_BUILD_JOBS`. Environment variables take
  precedence over config files.
* [Target-specific dependencies support CrabLang `cfg` syntax for
  describing targets][1.8cfg] so that dependencies for multiple
  targets can be specified together. [RFC 1361].
* [The environment variables `CARGO_TARGET_ROOT`, `CRABLANGC`, and
  `CRABLANGDOC` take precedence over the `build.target-dir`,
  `build.crablangc`, and `build.crablangdoc` configuration values][1.8cfv].
* [The child process tree is killed on Windows when Cargo is
  killed][1.8ck].
* [The `build.target` configuration value sets the target platform,
  like `--target`][1.8ct].

Compatibility Notes
-------------------

* [Unstable compiler flags have been further restricted][1.8u]. Since
  1.0 `-Z` flags have been considered unstable, and other flags that
  were considered unstable additionally required passing `-Z
  unstable-options` to access. Unlike unstable language and library
  features though, these options have been accessible on the stable
  release channel. Going forward, *new unstable flags will not be
  available on the stable release channel*, and old unstable flags
  will warn about their usage. In the future, all unstable flags will
  be unavailable on the stable release channel.
* [It is no longer possible to `match` on empty enum variants using
  the `Variant(..)` syntax][1.8v]. This has been a warning since 1.6.
* The Unix-specific `MetadataExt` traits, including
  `os::unix::fs::MetadataExt`, which expose values such as inode
  numbers [no longer return platform-specific types][1.8r], but
  instead return widened integers. [RFC 1415].
* [Modules sourced from the filesystem cannot appear within arbitrary
  blocks, but only within other modules][1.8mf].
* [`--cfg` compiler flags are parsed strictly as identifiers][1.8c].
* On Unix, [stack overflow triggers a runtime abort instead of a
  SIGSEGV][1.8so].
* [`Command::spawn` and its equivalents return an error if any of
  its command-line arguments contain interior `NUL`s][1.8n].
* [Tuple and unit enum variants from other crates are in the type
  namespace][1.8tn].
* [On Windows `crablangc` emits `.lib` files for the `staticlib` library
  type instead of `.a` files][1.8st]. Additionally, for the MSVC
  toolchain, `crablangc` emits import libraries named `foo.dll.lib`
  instead of `foo.lib`.


[1.8a]: https://github.com/crablang/crablang/pull/30962
[1.8b]: https://github.com/crablang/crablang/pull/31123
[1.8c]: https://github.com/crablang/crablang/pull/31530
[1.8cc]: https://github.com/crablang/cargo/pull/2397
[1.8ce]: https://github.com/crablang/cargo/pull/2398
[1.8cf]: https://github.com/crablang/crablang/pull/31278
[1.8cfg]: https://github.com/crablang/cargo/pull/2328
[1.8ci]: https://github.com/crablang/cargo/pull/2081
[1.8ck]: https://github.com/crablang/cargo/pull/2370
[1.8ct]: https://github.com/crablang/cargo/pull/2335
[1.8cu]: https://github.com/crablang/crablang/pull/31390
[1.8cfv]: https://github.com/crablang/cargo/issues/2365
[1.8cv]: https://github.com/crablang/crablang/pull/30998
[1.8h]: https://github.com/crablang/crablang/pull/31460
[1.8l]: https://github.com/crablang/crablang/pull/31668
[1.8m]: https://github.com/crablang/crablang/pull/31020
[1.8mf]: https://github.com/crablang/crablang/pull/31534
[1.8mp]: https://github.com/crablang/crablang/pull/30894
[1.8mr]: https://users.crablang.org/t/multicrablang-0-8-with-cross-std-installation/4901
[1.8ms]: https://github.com/crablang/crablang/pull/30448
[1.8n]: https://github.com/crablang/crablang/pull/31056
[1.8nx]: https://github.com/crablang/crablang/pull/30859
[1.8r]: https://github.com/crablang/crablang/pull/31551
[1.8so]: https://github.com/crablang/crablang/pull/31333
[1.8st]: https://github.com/crablang/crablang/pull/29520
[1.8t]: https://github.com/crablang/crablang/pull/31358
[1.8tn]: https://github.com/crablang/crablang/pull/30882
[1.8u]: https://github.com/crablang/crablang/pull/31793
[1.8v]: https://github.com/crablang/crablang/pull/31757
[1.8w]: https://github.com/crablang/crablang/pull/31904
[RFC 1361]: https://github.com/crablang/rfcs/blob/master/text/1361-cargo-cfg-dependencies.md
[RFC 1415]: https://github.com/crablang/rfcs/blob/master/text/1415-trim-std-os.md
[RFC 218]: https://github.com/crablang/rfcs/blob/master/text/0218-empty-struct-with-braces.md
[RFC 953]: https://github.com/crablang/rfcs/blob/master/text/0953-op-assign.md
[`AddAssign`]: http://doc.crablang.org/nightly/std/ops/trait.AddAssign.html
[`BitAndAssign`]: http://doc.crablang.org/nightly/std/ops/trait.BitAndAssign.html
[`BitOrAssign`]: http://doc.crablang.org/nightly/std/ops/trait.BitOrAssign.html
[`BitXorAssign`]: http://doc.crablang.org/nightly/std/ops/trait.BitXorAssign.html
[`DivAssign`]: http://doc.crablang.org/nightly/std/ops/trait.DivAssign.html
[`Instant::duration_since`]: http://doc.crablang.org/nightly/std/time/struct.Instant.html#method.duration_since
[`Instant::elapsed`]: http://doc.crablang.org/nightly/std/time/struct.Instant.html#method.elapsed
[`Instant::now`]: http://doc.crablang.org/nightly/std/time/struct.Instant.html#method.now
[`MulAssign`]: http://doc.crablang.org/nightly/std/ops/trait.MulAssign.html
[`Ref::map`]: http://doc.crablang.org/nightly/std/cell/struct.Ref.html#method.map
[`RefMut::map`]: http://doc.crablang.org/nightly/std/cell/struct.RefMut.html#method.map
[`RemAssign`]: http://doc.crablang.org/nightly/std/ops/trait.RemAssign.html
[`ShlAssign`]: http://doc.crablang.org/nightly/std/ops/trait.ShlAssign.html
[`ShrAssign`]: http://doc.crablang.org/nightly/std/ops/trait.ShrAssign.html
[`SubAssign`]: http://doc.crablang.org/nightly/std/ops/trait.SubAssign.html
[`SystemTime::duration_since`]: http://doc.crablang.org/nightly/std/time/struct.SystemTime.html#method.duration_since
[`SystemTime::elapsed`]: http://doc.crablang.org/nightly/std/time/struct.SystemTime.html#method.elapsed
[`SystemTime::now`]: http://doc.crablang.org/nightly/std/time/struct.SystemTime.html#method.now
[`SystemTimeError::duration`]: http://doc.crablang.org/nightly/std/time/struct.SystemTimeError.html#method.duration
[`SystemTimeError`]: http://doc.crablang.org/nightly/std/time/struct.SystemTimeError.html
[`UNIX_EPOCH`]: http://doc.crablang.org/nightly/std/time/constant.UNIX_EPOCH.html
[`ptr::drop_in_place`]: http://doc.crablang.org/nightly/std/ptr/fn.drop_in_place.html
[`str::EncodeUtf16`]: http://doc.crablang.org/nightly/std/str/struct.EncodeUtf16.html
[`str::encode_utf16`]: http://doc.crablang.org/nightly/std/primitive.str.html#method.encode_utf16
[`time::Instant`]: http://doc.crablang.org/nightly/std/time/struct.Instant.html
[`time::SystemTime`]: http://doc.crablang.org/nightly/std/time/struct.SystemTime.html


Version 1.7.0 (2016-03-03)
==========================

Libraries
---------

* Stabilized APIs
  * `Path`
    * [`Path::strip_prefix`] (renamed from relative_from)
    * [`path::StripPrefixError`] (new error type returned from strip_prefix)
  * `Ipv4Addr`
    * [`Ipv4Addr::is_loopback`]
    * [`Ipv4Addr::is_private`]
    * [`Ipv4Addr::is_link_local`]
    * [`Ipv4Addr::is_multicast`]
    * [`Ipv4Addr::is_broadcast`]
    * [`Ipv4Addr::is_documentation`]
  * `Ipv6Addr`
    * [`Ipv6Addr::is_unspecified`]
    * [`Ipv6Addr::is_loopback`]
    * [`Ipv6Addr::is_multicast`]
  * `Vec`
    * [`Vec::as_slice`]
    * [`Vec::as_mut_slice`]
  * `String`
    * [`String::as_str`]
    * [`String::as_mut_str`]
  * Slices
    * `<[T]>::`[`clone_from_slice`], which now requires the two slices to
    be the same length
    * `<[T]>::`[`sort_by_key`]
  * checked, saturated, and overflowing operations
    * [`i32::checked_rem`], [`i32::checked_neg`], [`i32::checked_shl`], [`i32::checked_shr`]
    * [`i32::saturating_mul`]
    * [`i32::overflowing_add`], [`i32::overflowing_sub`], [`i32::overflowing_mul`], [`i32::overflowing_div`]
    * [`i32::overflowing_rem`], [`i32::overflowing_neg`], [`i32::overflowing_shl`], [`i32::overflowing_shr`]
    * [`u32::checked_rem`], [`u32::checked_neg`], [`u32::checked_shl`], [`u32::checked_shl`]
    * [`u32::saturating_mul`]
    * [`u32::overflowing_add`], [`u32::overflowing_sub`], [`u32::overflowing_mul`], [`u32::overflowing_div`]
    * [`u32::overflowing_rem`], [`u32::overflowing_neg`], [`u32::overflowing_shl`], [`u32::overflowing_shr`]
    * and checked, saturated, and overflowing operations for other primitive types
  * FFI
    * [`ffi::IntoStringError`]
    * [`CString::into_string`]
    * [`CString::into_bytes`]
    * [`CString::into_bytes_with_nul`]
    * `From<CString> for Vec<u8>`
  * `IntoStringError`
    * [`IntoStringError::into_cstring`]
    * [`IntoStringError::utf8_error`]
    * `Error for IntoStringError`
  * Hashing
    * [`std::hash::BuildHasher`]
    * [`BuildHasher::Hasher`]
    * [`BuildHasher::build_hasher`]
    * [`std::hash::BuildHasherDefault`]
    * [`HashMap::with_hasher`]
    * [`HashMap::with_capacity_and_hasher`]
    * [`HashSet::with_hasher`]
    * [`HashSet::with_capacity_and_hasher`]
    * [`std::collections::hash_map::RandomState`]
    * [`RandomState::new`]
* [Validating UTF-8 is faster by a factor of between 7 and 14x for
  ASCII input][1.7utf8]. This means that creating `String`s and `str`s
  from bytes is faster.
* [The performance of `LineWriter` (and thus `io::stdout`) was
  improved by using `memchr` to search for newlines][1.7m].
* [`f32::to_degrees` and `f32::to_radians` are stable][1.7f]. The
  `f64` variants were stabilized previously.
* [`BTreeMap` was rewritten to use less memory and improve the performance
  of insertion and iteration, the latter by as much as 5x][1.7bm].
* [`BTreeSet` and its iterators, `Iter`, `IntoIter`, and `Range` are
  covariant over their contained type][1.7bt].
* [`LinkedList` and its iterators, `Iter` and `IntoIter` are covariant
  over their contained type][1.7ll].
* [`str::replace` now accepts a `Pattern`][1.7rp], like other string
  searching methods.
* [`Any` is implemented for unsized types][1.7a].
* [`Hash` is implemented for `Duration`][1.7h].

Misc
----

* [When running tests with `--test`, crablangdoc will pass `--cfg`
  arguments to the compiler][1.7dt].
* [The compiler is built with RPATH information by default][1.7rpa].
  This means that it will be possible to run `crablangc` when installed in
  unusual configurations without configuring the dynamic linker search
  path explicitly.
* [`crablangc` passes `--enable-new-dtags` to GNU ld][1.7dta]. This makes
  any RPATH entries (emitted with `-C rpath`) *not* take precedence
  over `LD_LIBRARY_PATH`.

Cargo
-----

* [`cargo crablangc` accepts a `--profile` flag that runs `crablangc` under
  any of the compilation profiles, 'dev', 'bench', or 'test'][1.7cp].
* [The `rerun-if-changed` build script directive no longer causes the
  build script to incorrectly run twice in certain scenarios][1.7rr].

Compatibility Notes
-------------------

* Soundness fixes to the interactions between associated types and
  lifetimes, specified in [RFC 1214], [now generate errors][1.7sf] for
  code that violates the new rules. This is a significant change that
  is known to break existing code, so it has emitted warnings for the
  new error cases since 1.4 to give crate authors time to adapt. The
  details of what is changing are subtle; read the RFC for more.
* [Several bugs in the compiler's visibility calculations were
  fixed][1.7v]. Since this was found to break significant amounts of
  code, the new errors will be emitted as warnings for several release
  cycles, under the `private_in_public` lint.
* Defaulted type parameters were accidentally accepted in positions
  that were not intended. In this release, [defaulted type parameters
  appearing outside of type definitions will generate a
  warning][1.7d], which will become an error in future releases.
* [Parsing "." as a float results in an error instead of 0][1.7p].
  That is, `".".parse::<f32>()` returns `Err`, not `Ok(0.0)`.
* [Borrows of closure parameters may not outlive the closure][1.7bc].

[1.7a]: https://github.com/crablang/crablang/pull/30928
[1.7bc]: https://github.com/crablang/crablang/pull/30341
[1.7bm]: https://github.com/crablang/crablang/pull/30426
[1.7bt]: https://github.com/crablang/crablang/pull/30998
[1.7cp]: https://github.com/crablang/cargo/pull/2224
[1.7d]: https://github.com/crablang/crablang/pull/30724
[1.7dt]: https://github.com/crablang/crablang/pull/30372
[1.7dta]: https://github.com/crablang/crablang/pull/30394
[1.7f]: https://github.com/crablang/crablang/pull/30672
[1.7h]: https://github.com/crablang/crablang/pull/30818
[1.7ll]: https://github.com/crablang/crablang/pull/30663
[1.7m]: https://github.com/crablang/crablang/pull/30381
[1.7p]: https://github.com/crablang/crablang/pull/30681
[1.7rp]: https://github.com/crablang/crablang/pull/29498
[1.7rpa]: https://github.com/crablang/crablang/pull/30353
[1.7rr]: https://github.com/crablang/cargo/pull/2279
[1.7sf]: https://github.com/crablang/crablang/pull/30389
[1.7utf8]: https://github.com/crablang/crablang/pull/30740
[1.7v]: https://github.com/crablang/crablang/pull/29973
[RFC 1214]: https://github.com/crablang/rfcs/blob/master/text/1214-projections-lifetimes-and-wf.md
[`BuildHasher::Hasher`]: http://doc.crablang.org/nightly/std/hash/trait.Hasher.html
[`BuildHasher::build_hasher`]: http://doc.crablang.org/nightly/std/hash/trait.BuildHasher.html#tymethod.build_hasher
[`CString::into_bytes_with_nul`]: http://doc.crablang.org/nightly/std/ffi/struct.CString.html#method.into_bytes_with_nul
[`CString::into_bytes`]: http://doc.crablang.org/nightly/std/ffi/struct.CString.html#method.into_bytes
[`CString::into_string`]: http://doc.crablang.org/nightly/std/ffi/struct.CString.html#method.into_string
[`HashMap::with_capacity_and_hasher`]: http://doc.crablang.org/nightly/std/collections/struct.HashMap.html#method.with_capacity_and_hasher
[`HashMap::with_hasher`]: http://doc.crablang.org/nightly/std/collections/struct.HashMap.html#method.with_hasher
[`HashSet::with_capacity_and_hasher`]: http://doc.crablang.org/nightly/std/collections/struct.HashSet.html#method.with_capacity_and_hasher
[`HashSet::with_hasher`]: http://doc.crablang.org/nightly/std/collections/struct.HashSet.html#method.with_hasher
[`IntoStringError::into_cstring`]: http://doc.crablang.org/nightly/std/ffi/struct.IntoStringError.html#method.into_cstring
[`IntoStringError::utf8_error`]: http://doc.crablang.org/nightly/std/ffi/struct.IntoStringError.html#method.utf8_error
[`Ipv4Addr::is_broadcast`]: http://doc.crablang.org/nightly/std/net/struct.Ipv4Addr.html#method.is_broadcast
[`Ipv4Addr::is_documentation`]: http://doc.crablang.org/nightly/std/net/struct.Ipv4Addr.html#method.is_documentation
[`Ipv4Addr::is_link_local`]: http://doc.crablang.org/nightly/std/net/struct.Ipv4Addr.html#method.is_link_local
[`Ipv4Addr::is_loopback`]: http://doc.crablang.org/nightly/std/net/struct.Ipv4Addr.html#method.is_loopback
[`Ipv4Addr::is_multicast`]: http://doc.crablang.org/nightly/std/net/struct.Ipv4Addr.html#method.is_multicast
[`Ipv4Addr::is_private`]: http://doc.crablang.org/nightly/std/net/struct.Ipv4Addr.html#method.is_private
[`Ipv6Addr::is_loopback`]: http://doc.crablang.org/nightly/std/net/struct.Ipv6Addr.html#method.is_loopback
[`Ipv6Addr::is_multicast`]: http://doc.crablang.org/nightly/std/net/struct.Ipv6Addr.html#method.is_multicast
[`Ipv6Addr::is_unspecified`]: http://doc.crablang.org/nightly/std/net/struct.Ipv6Addr.html#method.is_unspecified
[`Path::strip_prefix`]: http://doc.crablang.org/nightly/std/path/struct.Path.html#method.strip_prefix
[`RandomState::new`]: http://doc.crablang.org/nightly/std/collections/hash_map/struct.RandomState.html#method.new
[`String::as_mut_str`]: http://doc.crablang.org/nightly/std/string/struct.String.html#method.as_mut_str
[`String::as_str`]: http://doc.crablang.org/nightly/std/string/struct.String.html#method.as_str
[`Vec::as_mut_slice`]: http://doc.crablang.org/nightly/std/vec/struct.Vec.html#method.as_mut_slice
[`Vec::as_slice`]: http://doc.crablang.org/nightly/std/vec/struct.Vec.html#method.as_slice
[`clone_from_slice`]: http://doc.crablang.org/nightly/std/primitive.slice.html#method.clone_from_slice
[`ffi::IntoStringError`]: http://doc.crablang.org/nightly/std/ffi/struct.IntoStringError.html
[`i32::checked_neg`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.checked_neg
[`i32::checked_rem`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.checked_rem
[`i32::checked_shl`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.checked_shl
[`i32::checked_shr`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.checked_shr
[`i32::overflowing_add`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.overflowing_add
[`i32::overflowing_div`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.overflowing_div
[`i32::overflowing_mul`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.overflowing_mul
[`i32::overflowing_neg`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.overflowing_neg
[`i32::overflowing_rem`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.overflowing_rem
[`i32::overflowing_shl`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.overflowing_shl
[`i32::overflowing_shr`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.overflowing_shr
[`i32::overflowing_sub`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.overflowing_sub
[`i32::saturating_mul`]: http://doc.crablang.org/nightly/std/primitive.i32.html#method.saturating_mul
[`path::StripPrefixError`]: http://doc.crablang.org/nightly/std/path/struct.StripPrefixError.html
[`sort_by_key`]: http://doc.crablang.org/nightly/std/primitive.slice.html#method.sort_by_key
[`std::collections::hash_map::RandomState`]: http://doc.crablang.org/nightly/std/collections/hash_map/struct.RandomState.html
[`std::hash::BuildHasherDefault`]: http://doc.crablang.org/nightly/std/hash/struct.BuildHasherDefault.html
[`std::hash::BuildHasher`]: http://doc.crablang.org/nightly/std/hash/trait.BuildHasher.html
[`u32::checked_neg`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.checked_neg
[`u32::checked_rem`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.checked_rem
[`u32::checked_neg`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.checked_neg
[`u32::checked_shl`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.checked_shl
[`u32::overflowing_add`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.overflowing_add
[`u32::overflowing_div`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.overflowing_div
[`u32::overflowing_mul`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.overflowing_mul
[`u32::overflowing_neg`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.overflowing_neg
[`u32::overflowing_rem`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.overflowing_rem
[`u32::overflowing_shl`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.overflowing_shl
[`u32::overflowing_shr`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.overflowing_shr
[`u32::overflowing_sub`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.overflowing_sub
[`u32::saturating_mul`]: http://doc.crablang.org/nightly/std/primitive.u32.html#method.saturating_mul


Version 1.6.0 (2016-01-21)
==========================

Language
--------

* The `#![no_std]` attribute causes a crate to not be linked to the
  standard library, but only the [core library][1.6co], as described
  in [RFC 1184]. The core library defines common types and traits but
  has no platform dependencies whatsoever, and is the basis for CrabLang
  software in environments that cannot support a full port of the
  standard library, such as operating systems. Most of the core
  library is now stable.

Libraries
---------

* Stabilized APIs:
  [`Read::read_exact`],
  [`ErrorKind::UnexpectedEof`] (renamed from `UnexpectedEOF`),
  [`fs::DirBuilder`], [`fs::DirBuilder::new`],
  [`fs::DirBuilder::recursive`], [`fs::DirBuilder::create`],
  [`os::unix::fs::DirBuilderExt`],
  [`os::unix::fs::DirBuilderExt::mode`], [`vec::Drain`],
  [`vec::Vec::drain`], [`string::Drain`], [`string::String::drain`],
  [`vec_deque::Drain`], [`vec_deque::VecDeque::drain`],
  [`collections::hash_map::Drain`],
  [`collections::hash_map::HashMap::drain`],
  [`collections::hash_set::Drain`],
  [`collections::hash_set::HashSet::drain`],
  [`collections::binary_heap::Drain`],
  [`collections::binary_heap::BinaryHeap::drain`],
  [`Vec::extend_from_slice`] (renamed from `push_all`),
  [`Mutex::get_mut`], [`Mutex::into_inner`], [`RwLock::get_mut`],
  [`RwLock::into_inner`],
  [`Iterator::min_by_key`] (renamed from `min_by`),
  [`Iterator::max_by_key`] (renamed from `max_by`).
* The [core library][1.6co] is stable, as are most of its APIs.
* [The `assert_eq!` macro supports arguments that don't implement
  `Sized`][1.6ae], such as arrays. In this way it behaves more like
  `assert!`.
* Several timer functions that take duration in milliseconds [are
  deprecated in favor of those that take `Duration`][1.6ms]. These
  include `Condvar::wait_timeout_ms`, `thread::sleep_ms`, and
  `thread::park_timeout_ms`.
* The algorithm by which `Vec` reserves additional elements was
  [tweaked to not allocate excessive space][1.6a] while still growing
  exponentially.
* `From` conversions are [implemented from integers to floats][1.6f]
  in cases where the conversion is lossless. Thus they are not
  implemented for 32-bit ints to `f32`, nor for 64-bit ints to `f32`
  or `f64`. They are also not implemented for `isize` and `usize`
  because the implementations would be platform-specific. `From` is
  also implemented from `f32` to `f64`.
* `From<&Path>` and `From<PathBuf>` are implemented for `Cow<Path>`.
* `From<T>` is implemented for `Box<T>`, `Rc<T>` and `Arc<T>`.
* `IntoIterator` is implemented for `&PathBuf` and `&Path`.
* [`BinaryHeap` was refactored][1.6bh] for modest performance
  improvements.
* Sorting slices that are already sorted [is 50% faster in some
  cases][1.6s].

Cargo
-----

* Cargo will look in `$CARGO_HOME/bin` for subcommands [by default][1.6c].
* Cargo build scripts can specify their dependencies by emitting the
  [`rerun-if-changed`][1.6rr] key.
* crates.io will reject publication of crates with dependencies that
  have a wildcard version constraint. Crates with wildcard
  dependencies were seen to cause a variety of problems, as described
  in [RFC 1241]. Since 1.5 publication of such crates has emitted a
  warning.
* `cargo clean` [accepts a `--release` flag][1.6cc] to clean the
  release folder.  A variety of artifacts that Cargo failed to clean
  are now correctly deleted.

Misc
----

* The `unreachable_code` lint [warns when a function call's argument
  diverges][1.6dv].
* The parser indicates [failures that may be caused by
  confusingly-similar Unicode characters][1.6uc]
* Certain macro errors [are reported at definition time][1.6m], not
  expansion.

Compatibility Notes
-------------------

* The compiler no longer makes use of the [`CRABLANG_PATH`][1.6rp]
  environment variable when locating crates. This was a pre-cargo
  feature for integrating with the package manager that was
  accidentally never removed.
* [A number of bugs were fixed in the privacy checker][1.6p] that
  could cause previously-accepted code to break.
* [Modules and unit/tuple structs may not share the same name][1.6ts].
* [Bugs in pattern matching unit structs were fixed][1.6us]. The tuple
  struct pattern syntax (`Foo(..)`) can no longer be used to match
  unit structs. This is a warning now, but will become an error in
  future releases. Patterns that share the same name as a const are
  now an error.
* A bug was fixed that causes [crablangc not to apply default type
  parameters][1.6xc] when resolving certain method implementations of
  traits defined in other crates.

[1.6a]: https://github.com/crablang/crablang/pull/29454
[1.6ae]: https://github.com/crablang/crablang/pull/29770
[1.6bh]: https://github.com/crablang/crablang/pull/29811
[1.6c]: https://github.com/crablang/cargo/pull/2192
[1.6cc]: https://github.com/crablang/cargo/pull/2131
[1.6co]: http://doc.crablang.org/core/index.html
[1.6dv]: https://github.com/crablang/crablang/pull/30000
[1.6f]: https://github.com/crablang/crablang/pull/29129
[1.6m]: https://github.com/crablang/crablang/pull/29828
[1.6ms]: https://github.com/crablang/crablang/pull/29604
[1.6p]: https://github.com/crablang/crablang/pull/29726
[1.6rp]: https://github.com/crablang/crablang/pull/30034
[1.6rr]: https://github.com/crablang/cargo/pull/2134
[1.6s]: https://github.com/crablang/crablang/pull/29675
[1.6ts]: https://github.com/crablang/crablang/issues/21546
[1.6uc]: https://github.com/crablang/crablang/pull/29837
[1.6us]: https://github.com/crablang/crablang/pull/29383
[1.6xc]: https://github.com/crablang/crablang/issues/30123
[RFC 1184]: https://github.com/crablang/rfcs/blob/master/text/1184-stabilize-no_std.md
[RFC 1241]: https://github.com/crablang/rfcs/blob/master/text/1241-no-wildcard-deps.md
[`ErrorKind::UnexpectedEof`]: http://doc.crablang.org/nightly/std/io/enum.ErrorKind.html#variant.UnexpectedEof
[`Iterator::max_by_key`]: http://doc.crablang.org/nightly/std/iter/trait.Iterator.html#method.max_by_key
[`Iterator::min_by_key`]: http://doc.crablang.org/nightly/std/iter/trait.Iterator.html#method.min_by_key
[`Mutex::get_mut`]: http://doc.crablang.org/nightly/std/sync/struct.Mutex.html#method.get_mut
[`Mutex::into_inner`]: http://doc.crablang.org/nightly/std/sync/struct.Mutex.html#method.into_inner
[`Read::read_exact`]: http://doc.crablang.org/nightly/std/io/trait.Read.html#method.read_exact
[`RwLock::get_mut`]: http://doc.crablang.org/nightly/std/sync/struct.RwLock.html#method.get_mut
[`RwLock::into_inner`]: http://doc.crablang.org/nightly/std/sync/struct.RwLock.html#method.into_inner
[`Vec::extend_from_slice`]: http://doc.crablang.org/nightly/collections/vec/struct.Vec.html#method.extend_from_slice
[`collections::binary_heap::BinaryHeap::drain`]: http://doc.crablang.org/nightly/std/collections/binary_heap/struct.BinaryHeap.html#method.drain
[`collections::binary_heap::Drain`]: http://doc.crablang.org/nightly/std/collections/binary_heap/struct.Drain.html
[`collections::hash_map::Drain`]: http://doc.crablang.org/nightly/std/collections/hash_map/struct.Drain.html
[`collections::hash_map::HashMap::drain`]: http://doc.crablang.org/nightly/std/collections/hash_map/struct.HashMap.html#method.drain
[`collections::hash_set::Drain`]: http://doc.crablang.org/nightly/std/collections/hash_set/struct.Drain.html
[`collections::hash_set::HashSet::drain`]: http://doc.crablang.org/nightly/std/collections/hash_set/struct.HashSet.html#method.drain
[`fs::DirBuilder::create`]: http://doc.crablang.org/nightly/std/fs/struct.DirBuilder.html#method.create
[`fs::DirBuilder::new`]: http://doc.crablang.org/nightly/std/fs/struct.DirBuilder.html#method.new
[`fs::DirBuilder::recursive`]: http://doc.crablang.org/nightly/std/fs/struct.DirBuilder.html#method.recursive
[`fs::DirBuilder`]: http://doc.crablang.org/nightly/std/fs/struct.DirBuilder.html
[`os::unix::fs::DirBuilderExt::mode`]: http://doc.crablang.org/nightly/std/os/unix/fs/trait.DirBuilderExt.html#tymethod.mode
[`os::unix::fs::DirBuilderExt`]: http://doc.crablang.org/nightly/std/os/unix/fs/trait.DirBuilderExt.html
[`string::Drain`]: http://doc.crablang.org/nightly/std/string/struct.Drain.html
[`string::String::drain`]: http://doc.crablang.org/nightly/std/string/struct.String.html#method.drain
[`vec::Drain`]: http://doc.crablang.org/nightly/std/vec/struct.Drain.html
[`vec::Vec::drain`]: http://doc.crablang.org/nightly/std/vec/struct.Vec.html#method.drain
[`vec_deque::Drain`]: http://doc.crablang.org/nightly/std/collections/vec_deque/struct.Drain.html
[`vec_deque::VecDeque::drain`]: http://doc.crablang.org/nightly/std/collections/vec_deque/struct.VecDeque.html#method.drain


Version 1.5.0 (2015-12-10)
==========================

* ~700 changes, numerous bugfixes

Highlights
----------

* Stabilized APIs:
  [`BinaryHeap::from`], [`BinaryHeap::into_sorted_vec`],
  [`BinaryHeap::into_vec`], [`Condvar::wait_timeout`],
  [`FileTypeExt::is_block_device`], [`FileTypeExt::is_char_device`],
  [`FileTypeExt::is_fifo`], [`FileTypeExt::is_socket`],
  [`FileTypeExt`], [`Formatter::alternate`], [`Formatter::fill`],
  [`Formatter::precision`], [`Formatter::sign_aware_zero_pad`],
  [`Formatter::sign_minus`], [`Formatter::sign_plus`],
  [`Formatter::width`], [`Iterator::cmp`], [`Iterator::eq`],
  [`Iterator::ge`], [`Iterator::gt`], [`Iterator::le`],
  [`Iterator::lt`], [`Iterator::ne`], [`Iterator::partial_cmp`],
  [`Path::canonicalize`], [`Path::exists`], [`Path::is_dir`],
  [`Path::is_file`], [`Path::metadata`], [`Path::read_dir`],
  [`Path::read_link`], [`Path::symlink_metadata`],
  [`Utf8Error::valid_up_to`], [`Vec::resize`],
  [`VecDeque::as_mut_slices`], [`VecDeque::as_slices`],
  [`VecDeque::insert`], [`VecDeque::shrink_to_fit`],
  [`VecDeque::swap_remove_back`], [`VecDeque::swap_remove_front`],
  [`slice::split_first_mut`], [`slice::split_first`],
  [`slice::split_last_mut`], [`slice::split_last`],
  [`char::from_u32_unchecked`], [`fs::canonicalize`],
  [`str::MatchIndices`], [`str::RMatchIndices`],
  [`str::match_indices`], [`str::rmatch_indices`],
  [`str::slice_mut_unchecked`], [`string::ParseError`].
* CrabLang applications hosted on crates.io can be installed locally to
  `~/.cargo/bin` with the [`cargo install`] command. Among other
  things this makes it easier to augment Cargo with new subcommands:
  when a binary named e.g. `cargo-foo` is found in `$PATH` it can be
  invoked as `cargo foo`.
* Crates with wildcard (`*`) dependencies will [emit warnings when
  published][1.5w]. In 1.6 it will no longer be possible to publish
  crates with wildcard dependencies.

Breaking Changes
----------------

* The rules determining when a particular lifetime must outlive
  a particular value (known as '[dropck]') have been [modified
  to not rely on parametricity][1.5p].
* [Implementations of `AsRef` and `AsMut` were added to `Box`, `Rc`,
  and `Arc`][1.5a]. Because these smart pointer types implement
  `Deref`, this causes breakage in cases where the interior type
  contains methods of the same name.
* [Correct a bug in Rc/Arc][1.5c] that caused [dropck] to be unaware
  that they could drop their content. Soundness fix.
* All method invocations are [properly checked][1.5wf1] for
  [well-formedness][1.5wf2]. Soundness fix.
* Traits whose supertraits contain `Self` are [not object
  safe][1.5o]. Soundness fix.
* Target specifications support a [`no_default_libraries`][1.5nd]
  setting that controls whether `-nodefaultlibs` is passed to the
  linker, and in turn the `is_like_windows` setting no longer affects
  the `-nodefaultlibs` flag.
* `#[derive(Show)]`, long-deprecated, [has been removed][1.5ds].
* The `#[inline]` and `#[repr]` attributes [can only appear
  in valid locations][1.5at].
* Native libraries linked from the local crate are [passed to
  the linker before native libraries from upstream crates][1.5nl].
* Two rarely-used attributes, `#[no_debug]` and
  `#[omit_gdb_pretty_printer_section]` [are feature gated][1.5fg].
* Negation of unsigned integers, which has been a warning for
  several releases, [is now behind a feature gate and will
  generate errors][1.5nu].
* The parser accidentally accepted visibility modifiers on
  enum variants, a bug [which has been fixed][1.5ev].
* [A bug was fixed that allowed `use` statements to import unstable
  features][1.5use].

Language
--------

* When evaluating expressions at compile-time that are not
  compile-time constants (const-evaluating expressions in non-const
  contexts), incorrect code such as overlong bitshifts and arithmetic
  overflow will [generate a warning instead of an error][1.5ce],
  delaying the error until runtime. This will allow the
  const-evaluator to be expanded in the future backwards-compatibly.
* The `improper_ctypes` lint [no longer warns about using `isize` and
  `usize` in FFI][1.5ict].

Libraries
---------

* `Arc<T>` and `Rc<T>` are [covariant with respect to `T` instead of
  invariant][1.5c].
* `Default` is [implemented for mutable slices][1.5d].
* `FromStr` is [implemented for `SockAddrV4` and `SockAddrV6`][1.5s].
* There are now `From` conversions [between floating point
  types][1.5f] where the conversions are lossless.
* There are now `From` conversions [between integer types][1.5i] where
  the conversions are lossless.
* [`fs::Metadata` implements `Clone`][1.5fs].
* The `parse` method [accepts a leading "+" when parsing
  integers][1.5pi].
* [`AsMut` is implemented for `Vec`][1.5am].
* The `clone_from` implementations for `String` and `BinaryHeap` [have
  been optimized][1.5cf] and no longer rely on the default impl.
* The `extern "CrabLang"`, `extern "C"`, `unsafe extern "CrabLang"` and
  `unsafe extern "C"` function types now [implement `Clone`,
  `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `fmt::Pointer`, and
  `fmt::Debug` for up to 12 arguments][1.5fp].
* [Dropping `Vec`s is much faster in unoptimized builds when the
  element types don't implement `Drop`][1.5dv].
* A bug that caused in incorrect behavior when [combining `VecDeque`
  with zero-sized types][1.5vdz] was resolved.
* [`PartialOrd` for slices is faster][1.5po].

Miscellaneous
-------------

* [Crate metadata size was reduced by 20%][1.5md].
* [Improvements to code generation reduced the size of libcore by 3.3
  MB and crablangc's memory usage by 18MB][1.5m].
* [Improvements to deref translation increased performance in
  unoptimized builds][1.5dr].
* Various errors in trait resolution [are deduplicated to only be
  reported once][1.5te].
* CrabLang has preliminary [support for rumprun kernels][1.5rr].
* CrabLang has preliminary [support for NetBSD on amd64][1.5na].

[1.5use]: https://github.com/crablang/crablang/pull/28364
[1.5po]: https://github.com/crablang/crablang/pull/28436
[1.5ev]: https://github.com/crablang/crablang/pull/28442
[1.5nu]: https://github.com/crablang/crablang/pull/28468
[1.5dr]: https://github.com/crablang/crablang/pull/28491
[1.5vdz]: https://github.com/crablang/crablang/pull/28494
[1.5md]: https://github.com/crablang/crablang/pull/28521
[1.5fg]: https://github.com/crablang/crablang/pull/28522
[1.5dv]: https://github.com/crablang/crablang/pull/28531
[1.5na]: https://github.com/crablang/crablang/pull/28543
[1.5fp]: https://github.com/crablang/crablang/pull/28560
[1.5rr]: https://github.com/crablang/crablang/pull/28593
[1.5cf]: https://github.com/crablang/crablang/pull/28602
[1.5nl]: https://github.com/crablang/crablang/pull/28605
[1.5te]: https://github.com/crablang/crablang/pull/28645
[1.5at]: https://github.com/crablang/crablang/pull/28650
[1.5am]: https://github.com/crablang/crablang/pull/28663
[1.5m]: https://github.com/crablang/crablang/pull/28778
[1.5ict]: https://github.com/crablang/crablang/pull/28779
[1.5a]: https://github.com/crablang/crablang/pull/28811
[1.5pi]: https://github.com/crablang/crablang/pull/28826
[1.5ce]: https://github.com/crablang/rfcs/blob/master/text/1229-compile-time-asserts.md
[1.5p]: https://github.com/crablang/rfcs/blob/master/text/1238-nonparametric-dropck.md
[1.5i]: https://github.com/crablang/crablang/pull/28921
[1.5fs]: https://github.com/crablang/crablang/pull/29021
[1.5f]: https://github.com/crablang/crablang/pull/29129
[1.5ds]: https://github.com/crablang/crablang/pull/29148
[1.5s]: https://github.com/crablang/crablang/pull/29190
[1.5d]: https://github.com/crablang/crablang/pull/29245
[1.5o]: https://github.com/crablang/crablang/pull/29259
[1.5nd]: https://github.com/crablang/crablang/pull/28578
[1.5wf2]: https://github.com/crablang/rfcs/blob/master/text/1214-projections-lifetimes-and-wf.md
[1.5wf1]: https://github.com/crablang/crablang/pull/28669
[dropck]: https://doc.crablang.org/nightly/nomicon/dropck.html
[1.5c]: https://github.com/crablang/crablang/pull/29110
[1.5w]: https://github.com/crablang/rfcs/blob/master/text/1241-no-wildcard-deps.md
[`cargo install`]: https://github.com/crablang/rfcs/blob/master/text/1200-cargo-install.md
[`BinaryHeap::from`]: http://doc.crablang.org/nightly/std/convert/trait.From.html#method.from
[`BinaryHeap::into_sorted_vec`]: http://doc.crablang.org/nightly/std/collections/struct.BinaryHeap.html#method.into_sorted_vec
[`BinaryHeap::into_vec`]: http://doc.crablang.org/nightly/std/collections/struct.BinaryHeap.html#method.into_vec
[`Condvar::wait_timeout`]: http://doc.crablang.org/nightly/std/sync/struct.Condvar.html#method.wait_timeout
[`FileTypeExt::is_block_device`]: http://doc.crablang.org/nightly/std/os/unix/fs/trait.FileTypeExt.html#tymethod.is_block_device
[`FileTypeExt::is_char_device`]: http://doc.crablang.org/nightly/std/os/unix/fs/trait.FileTypeExt.html#tymethod.is_char_device
[`FileTypeExt::is_fifo`]: http://doc.crablang.org/nightly/std/os/unix/fs/trait.FileTypeExt.html#tymethod.is_fifo
[`FileTypeExt::is_socket`]: http://doc.crablang.org/nightly/std/os/unix/fs/trait.FileTypeExt.html#tymethod.is_socket
[`FileTypeExt`]: http://doc.crablang.org/nightly/std/os/unix/fs/trait.FileTypeExt.html
[`Formatter::alternate`]: http://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.alternate
[`Formatter::fill`]: http://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.fill
[`Formatter::precision`]: http://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.precision
[`Formatter::sign_aware_zero_pad`]: http://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.sign_aware_zero_pad
[`Formatter::sign_minus`]: http://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.sign_minus
[`Formatter::sign_plus`]: http://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.sign_plus
[`Formatter::width`]: http://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.width
[`Iterator::cmp`]: http://doc.crablang.org/nightly/core/iter/trait.Iterator.html#method.cmp
[`Iterator::eq`]: http://doc.crablang.org/nightly/core/iter/trait.Iterator.html#method.eq
[`Iterator::ge`]: http://doc.crablang.org/nightly/core/iter/trait.Iterator.html#method.ge
[`Iterator::gt`]: http://doc.crablang.org/nightly/core/iter/trait.Iterator.html#method.gt
[`Iterator::le`]: http://doc.crablang.org/nightly/core/iter/trait.Iterator.html#method.le
[`Iterator::lt`]: http://doc.crablang.org/nightly/core/iter/trait.Iterator.html#method.lt
[`Iterator::ne`]: http://doc.crablang.org/nightly/core/iter/trait.Iterator.html#method.ne
[`Iterator::partial_cmp`]: http://doc.crablang.org/nightly/core/iter/trait.Iterator.html#method.partial_cmp
[`Path::canonicalize`]: http://doc.crablang.org/nightly/std/path/struct.Path.html#method.canonicalize
[`Path::exists`]: http://doc.crablang.org/nightly/std/path/struct.Path.html#method.exists
[`Path::is_dir`]: http://doc.crablang.org/nightly/std/path/struct.Path.html#method.is_dir
[`Path::is_file`]: http://doc.crablang.org/nightly/std/path/struct.Path.html#method.is_file
[`Path::metadata`]: http://doc.crablang.org/nightly/std/path/struct.Path.html#method.metadata
[`Path::read_dir`]: http://doc.crablang.org/nightly/std/path/struct.Path.html#method.read_dir
[`Path::read_link`]: http://doc.crablang.org/nightly/std/path/struct.Path.html#method.read_link
[`Path::symlink_metadata`]: http://doc.crablang.org/nightly/std/path/struct.Path.html#method.symlink_metadata
[`Utf8Error::valid_up_to`]: http://doc.crablang.org/nightly/core/str/struct.Utf8Error.html#method.valid_up_to
[`Vec::resize`]: http://doc.crablang.org/nightly/std/vec/struct.Vec.html#method.resize
[`VecDeque::as_mut_slices`]: http://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.as_mut_slices
[`VecDeque::as_slices`]: http://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.as_slices
[`VecDeque::insert`]: http://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.insert
[`VecDeque::shrink_to_fit`]: http://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.shrink_to_fit
[`VecDeque::swap_remove_back`]: http://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.swap_remove_back
[`VecDeque::swap_remove_front`]: http://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.swap_remove_front
[`slice::split_first_mut`]: http://doc.crablang.org/nightly/std/primitive.slice.html#method.split_first_mut
[`slice::split_first`]: http://doc.crablang.org/nightly/std/primitive.slice.html#method.split_first
[`slice::split_last_mut`]: http://doc.crablang.org/nightly/std/primitive.slice.html#method.split_last_mut
[`slice::split_last`]: http://doc.crablang.org/nightly/std/primitive.slice.html#method.split_last
[`char::from_u32_unchecked`]: http://doc.crablang.org/nightly/std/char/fn.from_u32_unchecked.html
[`fs::canonicalize`]: http://doc.crablang.org/nightly/std/fs/fn.canonicalize.html
[`str::MatchIndices`]: http://doc.crablang.org/nightly/std/str/struct.MatchIndices.html
[`str::RMatchIndices`]: http://doc.crablang.org/nightly/std/str/struct.RMatchIndices.html
[`str::match_indices`]: http://doc.crablang.org/nightly/std/primitive.str.html#method.match_indices
[`str::rmatch_indices`]: http://doc.crablang.org/nightly/std/primitive.str.html#method.rmatch_indices
[`str::slice_mut_unchecked`]: http://doc.crablang.org/nightly/std/primitive.str.html#method.slice_mut_unchecked
[`string::ParseError`]: http://doc.crablang.org/nightly/std/string/enum.ParseError.html

Version 1.4.0 (2015-10-29)
==========================

* ~1200 changes, numerous bugfixes

Highlights
----------

* Windows builds targeting the 64-bit MSVC ABI and linker (instead of
  GNU) are now supported and recommended for use.

Breaking Changes
----------------

* [Several changes have been made to fix type soundness and improve
  the behavior of associated types][sound]. See [RFC 1214]. Although
  we have mostly introduced these changes as warnings this release, to
  become errors next release, there are still some scenarios that will
  see immediate breakage.
* [The `str::lines` and `BufRead::lines` iterators treat `\r\n` as
  line breaks in addition to `\n`][crlf].
* [Loans of `'static` lifetime extend to the end of a function][stat].
* [`str::parse` no longer introduces avoidable rounding error when
  parsing floating point numbers. Together with earlier changes to
  float formatting/output, "round trips" like f.to_string().parse()
  now preserve the value of f exactly. Additionally, leading plus
  signs are now accepted][fp3].


Language
--------

* `use` statements that import multiple items [can now rename
  them][i], as in `use foo::{bar as kitten, baz as puppy}`.
* [Binops work correctly on fat pointers][binfat].
* `pub extern crate`, which does not behave as expected, [issues a
  warning][pec] until a better solution is found.

Libraries
---------

* [Many APIs were stabilized][stab]: `<Box<str>>::into_string`,
  [`Arc::downgrade`], [`Arc::get_mut`], [`Arc::make_mut`],
  [`Arc::try_unwrap`], [`Box::from_raw`], [`Box::into_raw`], [`CStr::to_str`],
  [`CStr::to_string_lossy`], [`CString::from_raw`], [`CString::into_raw`],
  [`IntoRawFd::into_raw_fd`], [`IntoRawFd`],
  `IntoRawHandle::into_raw_handle`, `IntoRawHandle`,
  `IntoRawSocket::into_raw_socket`, `IntoRawSocket`, [`Rc::downgrade`],
  [`Rc::get_mut`], [`Rc::make_mut`], [`Rc::try_unwrap`], [`Result::expect`],
  [`String::into_boxed_str`], [`TcpStream::read_timeout`],
  [`TcpStream::set_read_timeout`], [`TcpStream::set_write_timeout`],
  [`TcpStream::write_timeout`], [`UdpSocket::read_timeout`],
  [`UdpSocket::set_read_timeout`], [`UdpSocket::set_write_timeout`],
  [`UdpSocket::write_timeout`], `Vec::append`, `Vec::split_off`,
  [`VecDeque::append`], [`VecDeque::retain`], [`VecDeque::split_off`],
  [`rc::Weak::upgrade`], [`rc::Weak`], [`slice::Iter::as_slice`],
  [`slice::IterMut::into_slice`], [`str::CharIndices::as_str`],
  [`str::Chars::as_str`], [`str::split_at_mut`], [`str::split_at`],
  [`sync::Weak::upgrade`], [`sync::Weak`], [`thread::park_timeout`],
  [`thread::sleep`].
* [Some APIs were deprecated][dep]: `BTreeMap::with_b`,
  `BTreeSet::with_b`, `Option::as_mut_slice`, `Option::as_slice`,
  `Result::as_mut_slice`, `Result::as_slice`, `f32::from_str_radix`,
  `f64::from_str_radix`.
* [Reverse-searching strings is faster with the 'two-way'
  algorithm][s].
* [`std::io::copy` allows `?Sized` arguments][cc].
* The `Windows`, `Chunks`, and `ChunksMut` iterators over slices all
  [override `count`, `nth` and `last` with an *O*(1)
  implementation][it].
* [`Default` is implemented for arrays up to `[T; 32]`][d].
* [`IntoRawFd` has been added to the Unix-specific prelude,
  `IntoRawSocket` and `IntoRawHandle` to the Windows-specific
  prelude][pr].
* [`Extend<String>` and `FromIterator<String` are both implemented for
  `String`][es].
* [`IntoIterator` is implemented for references to `Option` and
  `Result`][into2].
* [`HashMap` and `HashSet` implement `Extend<&T>` where `T:
  Copy`][ext] as part of [RFC 839]. This will cause type inference
  breakage in rare situations.
* [`BinaryHeap` implements `Debug`][bh2].
* [`Borrow` and `BorrowMut` are implemented for fixed-size
  arrays][bm].
* [`extern fn`s with the "CrabLang" and "C" ABIs implement common
  traits including `Eq`, `Ord`, `Debug`, `Hash`][fp].
* [String comparison is faster][faststr].
* `&mut T` where `T: std::fmt::Write` [also implements
  `std::fmt::Write`][mutw].
* [A stable regression in `VecDeque::push_back` and other
  capacity-altering methods that caused panics for zero-sized types
  was fixed][vd].
* [Function pointers implement traits for up to 12 parameters][fp2].

Miscellaneous
-------------

* The compiler [no longer uses the 'morestack' feature to prevent
  stack overflow][mm]. Instead it uses guard pages and stack
  probes (though stack probes are not yet implemented on any platform
  but Windows).
* [The compiler matches traits faster when projections are involved][p].
* The 'improper_ctypes' lint [no longer warns about use of `isize` and
  `usize`][ffi].
* [Cargo now displays useful information about what its doing during
  `cargo update`][cu].

[`Arc::downgrade`]: http://doc.crablang.org/nightly/alloc/arc/struct.Arc.html#method.downgrade
[`Arc::make_mut`]: http://doc.crablang.org/nightly/alloc/arc/struct.Arc.html#method.make_mut
[`Arc::get_mut`]: http://doc.crablang.org/nightly/alloc/arc/struct.Arc.html#method.get_mut
[`Arc::try_unwrap`]: http://doc.crablang.org/nightly/alloc/arc/struct.Arc.html#method.try_unwrap
[`Box::from_raw`]: http://doc.crablang.org/nightly/alloc/boxed/struct.Box.html#method.from_raw
[`Box::into_raw`]: http://doc.crablang.org/nightly/alloc/boxed/struct.Box.html#method.into_raw
[`CStr::to_str`]: http://doc.crablang.org/nightly/std/ffi/struct.CStr.html#method.to_str
[`CStr::to_string_lossy`]: http://doc.crablang.org/nightly/std/ffi/struct.CStr.html#method.to_string_lossy
[`CString::from_raw`]: http://doc.crablang.org/nightly/std/ffi/struct.CString.html#method.from_raw
[`CString::into_raw`]: http://doc.crablang.org/nightly/std/ffi/struct.CString.html#method.into_raw
[`IntoRawFd::into_raw_fd`]: http://doc.crablang.org/nightly/std/os/unix/io/trait.IntoRawFd.html#tymethod.into_raw_fd
[`IntoRawFd`]: http://doc.crablang.org/nightly/std/os/unix/io/trait.IntoRawFd.html
[`Rc::downgrade`]: http://doc.crablang.org/nightly/alloc/rc/struct.Rc.html#method.downgrade
[`Rc::get_mut`]: http://doc.crablang.org/nightly/alloc/rc/struct.Rc.html#method.get_mut
[`Rc::make_mut`]: http://doc.crablang.org/nightly/alloc/rc/struct.Rc.html#method.make_mut
[`Rc::try_unwrap`]: http://doc.crablang.org/nightly/alloc/rc/struct.Rc.html#method.try_unwrap
[`Result::expect`]: http://doc.crablang.org/nightly/core/result/enum.Result.html#method.expect
[`String::into_boxed_str`]: http://doc.crablang.org/nightly/collections/string/struct.String.html#method.into_boxed_str
[`TcpStream::read_timeout`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.read_timeout
[`TcpStream::set_read_timeout`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_read_timeout
[`TcpStream::write_timeout`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.write_timeout
[`TcpStream::set_write_timeout`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_write_timeout
[`UdpSocket::read_timeout`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.read_timeout
[`UdpSocket::set_read_timeout`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_read_timeout
[`UdpSocket::write_timeout`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.write_timeout
[`UdpSocket::set_write_timeout`]: http://doc.crablang.org/nightly/std/net/struct.TcpStream.html#method.set_write_timeout
[`VecDeque::append`]: http://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.append
[`VecDeque::retain`]: http://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.retain
[`VecDeque::split_off`]: http://doc.crablang.org/nightly/std/collections/struct.VecDeque.html#method.split_off
[`rc::Weak::upgrade`]: http://doc.crablang.org/nightly/std/rc/struct.Weak.html#method.upgrade
[`rc::Weak`]: http://doc.crablang.org/nightly/std/rc/struct.Weak.html
[`slice::Iter::as_slice`]: http://doc.crablang.org/nightly/std/slice/struct.Iter.html#method.as_slice
[`slice::IterMut::into_slice`]: http://doc.crablang.org/nightly/std/slice/struct.IterMut.html#method.into_slice
[`str::CharIndices::as_str`]: http://doc.crablang.org/nightly/std/str/struct.CharIndices.html#method.as_str
[`str::Chars::as_str`]: http://doc.crablang.org/nightly/std/str/struct.Chars.html#method.as_str
[`str::split_at_mut`]: http://doc.crablang.org/nightly/std/primitive.str.html#method.split_at_mut
[`str::split_at`]: http://doc.crablang.org/nightly/std/primitive.str.html#method.split_at
[`sync::Weak::upgrade`]: http://doc.crablang.org/nightly/std/sync/struct.Weak.html#method.upgrade
[`sync::Weak`]: http://doc.crablang.org/nightly/std/sync/struct.Weak.html
[`thread::park_timeout`]: http://doc.crablang.org/nightly/std/thread/fn.park_timeout.html
[`thread::sleep`]: http://doc.crablang.org/nightly/std/thread/fn.sleep.html
[bh2]: https://github.com/crablang/crablang/pull/28156
[binfat]: https://github.com/crablang/crablang/pull/28270
[bm]: https://github.com/crablang/crablang/pull/28197
[cc]: https://github.com/crablang/crablang/pull/27531
[crlf]: https://github.com/crablang/crablang/pull/28034
[cu]: https://github.com/crablang/cargo/pull/1931
[d]: https://github.com/crablang/crablang/pull/27825
[dep]: https://github.com/crablang/crablang/pull/28339
[es]: https://github.com/crablang/crablang/pull/27956
[ext]: https://github.com/crablang/crablang/pull/28094
[faststr]: https://github.com/crablang/crablang/pull/28338
[ffi]: https://github.com/crablang/crablang/pull/28779
[fp]: https://github.com/crablang/crablang/pull/28268
[fp2]: https://github.com/crablang/crablang/pull/28560
[fp3]: https://github.com/crablang/crablang/pull/27307
[i]: https://github.com/crablang/crablang/pull/27451
[into2]: https://github.com/crablang/crablang/pull/28039
[it]: https://github.com/crablang/crablang/pull/27652
[mm]: https://github.com/crablang/crablang/pull/27338
[mutw]: https://github.com/crablang/crablang/pull/28368
[sound]: https://github.com/crablang/crablang/pull/27641
[p]: https://github.com/crablang/crablang/pull/27866
[pec]: https://github.com/crablang/crablang/pull/28486
[pr]: https://github.com/crablang/crablang/pull/27896
[RFC 839]: https://github.com/crablang/rfcs/blob/master/text/0839-embrace-extend-extinguish.md
[RFC 1214]: https://github.com/crablang/rfcs/blob/master/text/1214-projections-lifetimes-and-wf.md
[s]: https://github.com/crablang/crablang/pull/27474
[stab]: https://github.com/crablang/crablang/pull/28339
[stat]: https://github.com/crablang/crablang/pull/28321
[vd]: https://github.com/crablang/crablang/pull/28494

Version 1.3.0 (2015-09-17)
==============================

* ~900 changes, numerous bugfixes

Highlights
----------

* The [new object lifetime defaults][nold] have been [turned
  on][nold2] after a cycle of warnings about the change. Now types
  like `&'a Box<Trait>` (or `&'a Rc<Trait>`, etc) will change from
  being interpreted as `&'a Box<Trait+'a>` to `&'a
  Box<Trait+'static>`.
* [The CrabLangonomicon][nom] is a new book in the official documentation
  that dives into writing unsafe CrabLang.
* The [`Duration`] API, [has been stabilized][ds]. This basic unit of
  timekeeping is employed by other std APIs, as well as out-of-tree
  time crates.

Breaking Changes
----------------

* The [new object lifetime defaults][nold] have been [turned
  on][nold2] after a cycle of warnings about the change.
* There is a known [regression][lr] in how object lifetime elision is
  interpreted, the proper solution for which is undetermined.
* The `#[prelude_import]` attribute, an internal implementation
  detail, was accidentally stabilized previously. [It has been put
  behind the `prelude_import` feature gate][pi]. This change is
  believed to break no existing code.
* The behavior of [`size_of_val`][dst1] and [`align_of_val`][dst2] is
  [more sane for dynamically sized types][dst3]. Code that relied on
  the previous behavior is thought to be broken.
* The `dropck` rules, which checks that destructors can't access
  destroyed values, [have been updated][dropck] to match the
  [RFC][dropckrfc]. This fixes some soundness holes, and as such will
  cause some previously-compiling code to no longer build.

Language
--------

* The [new object lifetime defaults][nold] have been [turned
  on][nold2] after a cycle of warnings about the change.
* Semicolons may [now follow types and paths in
  macros](https://github.com/crablang/crablang/pull/27000).
* The behavior of [`size_of_val`][dst1] and [`align_of_val`][dst2] is
  [more sane for dynamically sized types][dst3]. Code that relied on
  the previous behavior is not known to exist, and suspected to be
  broken.
* `'static` variables [may now be recursive][st].
* `ref` bindings choose between [`Deref`] and [`DerefMut`]
  implementations correctly.
* The `dropck` rules, which checks that destructors can't access
  destroyed values, [have been updated][dropck] to match the
  [RFC][dropckrfc].

Libraries
---------

* The [`Duration`] API, [has been stabilized][ds], as well as the
  `std::time` module, which presently contains only `Duration`.
* `Box<str>` and `Box<[T]>` both implement `Clone`.
* The owned C string, [`CString`], implements [`Borrow`] and the
  borrowed C string, [`CStr`], implements [`ToOwned`]. The two of
  these allow C strings to be borrowed and cloned in generic code.
* [`CStr`] implements [`Debug`].
* [`AtomicPtr`] implements [`Debug`].
* [`Error`] trait objects [can be downcast to their concrete types][e]
  in many common configurations, using the [`is`], [`downcast`],
  [`downcast_ref`] and [`downcast_mut`] methods, similarly to the
  [`Any`] trait.
* Searching for substrings now [employs the two-way algorithm][search]
  instead of doing a naive search. This gives major speedups to a
  number of methods, including [`contains`][sc], [`find`][sf],
  [`rfind`][srf], [`split`][ss]. [`starts_with`][ssw] and
  [`ends_with`][sew] are also faster.
* The performance of `PartialEq` for slices is [much faster][ps].
* The [`Hash`] trait offers the default method, [`hash_slice`], which
  is overridden and optimized by the implementations for scalars.
* The [`Hasher`] trait now has a number of specialized `write_*`
  methods for primitive types, for efficiency.
* The I/O-specific error type, [`std::io::Error`][ie], gained a set of
  methods for accessing the 'inner error', if any: [`get_ref`][iegr],
  [`get_mut`][iegm], [`into_inner`][ieii]. As well, the implementation
  of [`std::error::Error::cause`][iec] also delegates to the inner
  error.
* [`process::Child`][pc] gained the [`id`] method, which returns a
  `u32` representing the platform-specific process identifier.
* The [`connect`] method on slices is deprecated, replaced by the new
  [`join`] method (note that both of these are on the *unstable*
  [`SliceConcatExt`] trait, but through the magic of the prelude are
  available to stable code anyway).
* The [`Div`] operator is implemented for [`Wrapping`] types.
* [`DerefMut` is implemented for `String`][dms].
* Performance of SipHash (the default hasher for `HashMap`) is
  [better for long data][sh].
* [`AtomicPtr`] implements [`Send`].
* The [`read_to_end`] implementations for [`Stdin`] and [`File`]
  are now [specialized to use uninitialized buffers for increased
  performance][rte].
* Lifetime parameters of foreign functions [are now resolved
  properly][f].

Misc
----

* CrabLang can now, with some coercion, [produce programs that run on
  Windows XP][xp], though XP is not considered a supported platform.
* Porting CrabLang on Windows from the GNU toolchain to MSVC continues
  ([1][win1], [2][win2], [3][win3], [4][win4]). It is still not
  recommended for use in 1.3, though should be fully-functional
  in the [64-bit 1.4 beta][b14].
* On Fedora-based systems installation will [properly configure the
  dynamic linker][fl].
* The compiler gained many new extended error descriptions, which can
  be accessed with the `--explain` flag.
* The `dropck` pass, which checks that destructors can't access
  destroyed values, [has been rewritten][27261]. This fixes some
  soundness holes, and as such will cause some previously-compiling
  code to no longer build.
* `crablangc` now uses [LLVM to write archive files where possible][ar].
  Eventually this will eliminate the compiler's dependency on the ar
  utility.
* CrabLang has [preliminary support for i686 FreeBSD][26959] (it has long
  supported FreeBSD on x86_64).
* The [`unused_mut`][lum], [`unconditional_recursion`][lur],
  [`improper_ctypes`][lic], and [`negate_unsigned`][lnu] lints are
  more strict.
* If landing pads are disabled (with `-Z no-landing-pads`), [`panic!`
  will kill the process instead of leaking][nlp].

[`Any`]: http://doc.crablang.org/nightly/std/any/trait.Any.html
[`AtomicPtr`]: http://doc.crablang.org/nightly/std/sync/atomic/struct.AtomicPtr.html
[`Borrow`]: http://doc.crablang.org/nightly/std/borrow/trait.Borrow.html
[`CStr`]: http://doc.crablang.org/nightly/std/ffi/struct.CStr.html
[`CString`]: http://doc.crablang.org/nightly/std/ffi/struct.CString.html
[`Debug`]: http://doc.crablang.org/nightly/std/fmt/trait.Debug.html
[`DerefMut`]: http://doc.crablang.org/nightly/std/ops/trait.DerefMut.html
[`Deref`]: http://doc.crablang.org/nightly/std/ops/trait.Deref.html
[`Div`]: http://doc.crablang.org/nightly/std/ops/trait.Div.html
[`Duration`]: http://doc.crablang.org/nightly/std/time/struct.Duration.html
[`Error`]: http://doc.crablang.org/nightly/std/error/trait.Error.html
[`File`]: http://doc.crablang.org/nightly/std/fs/struct.File.html
[`Hash`]: http://doc.crablang.org/nightly/std/hash/trait.Hash.html
[`Hasher`]: http://doc.crablang.org/nightly/std/hash/trait.Hasher.html
[`Send`]: http://doc.crablang.org/nightly/std/marker/trait.Send.html
[`SliceConcatExt`]: http://doc.crablang.org/nightly/std/slice/trait.SliceConcatExt.html
[`Stdin`]: http://doc.crablang.org/nightly/std/io/struct.Stdin.html
[`ToOwned`]: http://doc.crablang.org/nightly/std/borrow/trait.ToOwned.html
[`Wrapping`]: http://doc.crablang.org/nightly/std/num/struct.Wrapping.html
[`connect`]: http://doc.crablang.org/nightly/std/slice/trait.SliceConcatExt.html#method.connect
[`downcast_mut`]: http://doc.crablang.org/nightly/std/error/trait.Error.html#method.downcast_mut
[`downcast_ref`]: http://doc.crablang.org/nightly/std/error/trait.Error.html#method.downcast_ref
[`downcast`]: http://doc.crablang.org/nightly/std/error/trait.Error.html#method.downcast
[`hash_slice`]: http://doc.crablang.org/nightly/std/hash/trait.Hash.html#method.hash_slice
[`id`]: http://doc.crablang.org/nightly/std/process/struct.Child.html#method.id
[`is`]: http://doc.crablang.org/nightly/std/error/trait.Error.html#method.is
[`join`]: http://doc.crablang.org/nightly/std/slice/trait.SliceConcatExt.html#method.join
[`read_to_end`]: http://doc.crablang.org/nightly/std/io/trait.Read.html#method.read_to_end
[ar]: https://github.com/crablang/crablang/pull/26926
[b14]: https://static.crablang.org/dist/crablang-beta-x86_64-pc-windows-msvc.msi
[dms]: https://github.com/crablang/crablang/pull/26241
[27261]: https://github.com/crablang/crablang/pull/27261
[dropckrfc]: https://github.com/crablang/rfcs/blob/master/text/0769-sound-generic-drop.md
[ds]: https://github.com/crablang/crablang/pull/26818
[dst1]: http://doc.crablang.org/nightly/std/mem/fn.size_of_val.html
[dst2]: http://doc.crablang.org/nightly/std/mem/fn.align_of_val.html
[dst3]: https://github.com/crablang/crablang/pull/27351
[e]: https://github.com/crablang/crablang/pull/24793
[f]: https://github.com/crablang/crablang/pull/26588
[26959]: https://github.com/crablang/crablang/pull/26959
[fl]: https://github.com/crablang/crablang-installer/pull/41
[ie]: http://doc.crablang.org/nightly/std/io/struct.Error.html
[iec]: http://doc.crablang.org/nightly/std/io/struct.Error.html#method.cause
[iegm]: http://doc.crablang.org/nightly/std/io/struct.Error.html#method.get_mut
[iegr]: http://doc.crablang.org/nightly/std/io/struct.Error.html#method.get_ref
[ieii]: http://doc.crablang.org/nightly/std/io/struct.Error.html#method.into_inner
[lic]: https://github.com/crablang/crablang/pull/26583
[lnu]: https://github.com/crablang/crablang/pull/27026
[lr]: https://github.com/crablang/crablang/issues/27248
[lum]: https://github.com/crablang/crablang/pull/26378
[lur]: https://github.com/crablang/crablang/pull/26783
[nlp]: https://github.com/crablang/crablang/pull/27176
[nold2]: https://github.com/crablang/crablang/pull/27045
[nold]: https://github.com/crablang/rfcs/blob/master/text/1156-adjust-default-object-bounds.md
[nom]: http://doc.crablang.org/nightly/nomicon/
[pc]: http://doc.crablang.org/nightly/std/process/struct.Child.html
[pi]: https://github.com/crablang/crablang/pull/26699
[ps]: https://github.com/crablang/crablang/pull/26884
[rte]: https://github.com/crablang/crablang/pull/26950
[sc]: http://doc.crablang.org/nightly/std/primitive.str.html#method.contains
[search]: https://github.com/crablang/crablang/pull/26327
[sew]: http://doc.crablang.org/nightly/std/primitive.str.html#method.ends_with
[sf]: http://doc.crablang.org/nightly/std/primitive.str.html#method.find
[sh]: https://github.com/crablang/crablang/pull/27280
[srf]: http://doc.crablang.org/nightly/std/primitive.str.html#method.rfind
[ss]: http://doc.crablang.org/nightly/std/primitive.str.html#method.split
[ssw]: http://doc.crablang.org/nightly/std/primitive.str.html#method.starts_with
[st]: https://github.com/crablang/crablang/pull/26630
[win1]: https://github.com/crablang/crablang/pull/26569
[win2]: https://github.com/crablang/crablang/pull/26741
[win3]: https://github.com/crablang/crablang/pull/26741
[win4]: https://github.com/crablang/crablang/pull/27210
[xp]: https://github.com/crablang/crablang/pull/26569

Version 1.2.0 (2015-08-07)
==========================

* ~1200 changes, numerous bugfixes

Highlights
----------

* [Dynamically-sized-type coercions][dst] allow smart pointer types
  like `Rc` to contain types without a fixed size, arrays and trait
  objects, finally enabling use of `Rc<[T]>` and completing the
  implementation of DST.
* [Parallel codegen][parcodegen] is now working again, which can
  substantially speed up large builds in debug mode; It also gets
  another ~33% speedup when bootstrapping on a 4 core machine (using 8
  jobs). It's not enabled by default, but will be "in the near
  future". It can be activated with the `-C codegen-units=N` flag to
  `crablangc`.
* This is the first release with [experimental support for linking
  with the MSVC linker and lib C on Windows (instead of using the GNU
  variants via MinGW)][win]. It is yet recommended only for the most
  intrepid CrabLangaceans.
* Benchmark compilations are showing a 30% improvement in
  bootstrapping over 1.1.

Breaking Changes
----------------

* The [`to_uppercase`] and [`to_lowercase`] methods on `char` now do
  unicode case mapping, which is a previously-planned change in
  behavior and considered a bugfix.
* [`mem::align_of`] now specifies [the *minimum alignment* for
  T][align], which is usually the alignment programs are interested
  in, and the same value reported by clang's
  `alignof`. [`mem::min_align_of`] is deprecated. This is not known to
  break real code.
* [The `#[packed]` attribute is no longer silently accepted by the
  compiler][packed]. This attribute did nothing and code that
  mentioned it likely did not work as intended.
* Associated type defaults are [now behind the
  `associated_type_defaults` feature gate][ad]. In 1.1 associated type
  defaults *did not work*, but could be mentioned syntactically. As
  such this breakage has minimal impact.

Language
--------

* Patterns with `ref mut` now correctly invoke [`DerefMut`] when
  matching against dereferenceable values.

Libraries
---------

* The [`Extend`] trait, which grows a collection from an iterator, is
  implemented over iterators of references, for `String`, `Vec`,
  `LinkedList`, `VecDeque`, `EnumSet`, `BinaryHeap`, `VecMap`,
  `BTreeSet` and `BTreeMap`. [RFC][extend-rfc].
* The [`iter::once`] function returns an iterator that yields a single
  element, and [`iter::empty`] returns an iterator that yields no
  elements.
* The [`matches`] and [`rmatches`] methods on `str` return iterators
  over substring matches.
* [`Cell`] and [`RefCell`] both implement `Eq`.
* A number of methods for wrapping arithmetic are added to the
  integral types, [`wrapping_div`], [`wrapping_rem`],
  [`wrapping_neg`], [`wrapping_shl`], [`wrapping_shr`]. These are in
  addition to the existing [`wrapping_add`], [`wrapping_sub`], and
  [`wrapping_mul`] methods, and alternatives to the [`Wrapping`]
  type.. It is illegal for the default arithmetic operations in CrabLang
  to overflow; the desire to wrap must be explicit.
* The `{:#?}` formatting specifier [displays the alternate,
  pretty-printed][debugfmt] form of the `Debug` formatter. This
  feature was actually introduced prior to 1.0 with little
  fanfare.
* [`fmt::Formatter`] implements [`fmt::Write`], a `fmt`-specific trait
  for writing data to formatted strings, similar to [`io::Write`].
* [`fmt::Formatter`] adds 'debug builder' methods, [`debug_struct`],
  [`debug_tuple`], [`debug_list`], [`debug_set`], [`debug_map`]. These
  are used by code generators to emit implementations of [`Debug`].
* `str` has new [`to_uppercase`][strup] and [`to_lowercase`][strlow]
  methods that convert case, following Unicode case mapping.
* It is now easier to handle poisoned locks. The [`PoisonError`]
  type, returned by failing lock operations, exposes `into_inner`,
  `get_ref`, and `get_mut`, which all give access to the inner lock
  guard, and allow the poisoned lock to continue to operate. The
  `is_poisoned` method of [`RwLock`] and [`Mutex`] can poll for a
  poisoned lock without attempting to take the lock.
* On Unix the [`FromRawFd`] trait is implemented for [`Stdio`], and
  [`AsRawFd`] for [`ChildStdin`], [`ChildStdout`], [`ChildStderr`].
  On Windows the `FromRawHandle` trait is implemented for `Stdio`,
  and `AsRawHandle` for `ChildStdin`, `ChildStdout`,
  `ChildStderr`.
* [`io::ErrorKind`] has a new variant, `InvalidData`, which indicates
  malformed input.

Misc
----

* `crablangc` employs smarter heuristics for guessing at [typos].
* `crablangc` emits more efficient code for [no-op conversions between
  unsafe pointers][nop].
* Fat pointers are now [passed in pairs of immediate arguments][fat],
  resulting in faster compile times and smaller code.

[`Extend`]: https://doc.crablang.org/nightly/std/iter/trait.Extend.html
[extend-rfc]: https://github.com/crablang/rfcs/blob/master/text/0839-embrace-extend-extinguish.md
[`iter::once`]: https://doc.crablang.org/nightly/std/iter/fn.once.html
[`iter::empty`]: https://doc.crablang.org/nightly/std/iter/fn.empty.html
[`matches`]: https://doc.crablang.org/nightly/std/primitive.str.html#method.matches
[`rmatches`]: https://doc.crablang.org/nightly/std/primitive.str.html#method.rmatches
[`Cell`]: https://doc.crablang.org/nightly/std/cell/struct.Cell.html
[`RefCell`]: https://doc.crablang.org/nightly/std/cell/struct.RefCell.html
[`wrapping_add`]: https://doc.crablang.org/nightly/std/primitive.i8.html#method.wrapping_add
[`wrapping_sub`]: https://doc.crablang.org/nightly/std/primitive.i8.html#method.wrapping_sub
[`wrapping_mul`]: https://doc.crablang.org/nightly/std/primitive.i8.html#method.wrapping_mul
[`wrapping_div`]: https://doc.crablang.org/nightly/std/primitive.i8.html#method.wrapping_div
[`wrapping_rem`]: https://doc.crablang.org/nightly/std/primitive.i8.html#method.wrapping_rem
[`wrapping_neg`]: https://doc.crablang.org/nightly/std/primitive.i8.html#method.wrapping_neg
[`wrapping_shl`]: https://doc.crablang.org/nightly/std/primitive.i8.html#method.wrapping_shl
[`wrapping_shr`]: https://doc.crablang.org/nightly/std/primitive.i8.html#method.wrapping_shr
[`Wrapping`]: https://doc.crablang.org/nightly/std/num/struct.Wrapping.html
[`fmt::Formatter`]: https://doc.crablang.org/nightly/std/fmt/struct.Formatter.html
[`fmt::Write`]: https://doc.crablang.org/nightly/std/fmt/trait.Write.html
[`io::Write`]: https://doc.crablang.org/nightly/std/io/trait.Write.html
[`debug_struct`]: https://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.debug_struct
[`debug_tuple`]: https://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.debug_tuple
[`debug_list`]: https://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.debug_list
[`debug_set`]: https://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.debug_set
[`debug_map`]: https://doc.crablang.org/nightly/core/fmt/struct.Formatter.html#method.debug_map
[`Debug`]: https://doc.crablang.org/nightly/std/fmt/trait.Debug.html
[strup]: https://doc.crablang.org/nightly/std/primitive.str.html#method.to_uppercase
[strlow]: https://doc.crablang.org/nightly/std/primitive.str.html#method.to_lowercase
[`to_uppercase`]: https://doc.crablang.org/nightly/std/primitive.char.html#method.to_uppercase
[`to_lowercase`]: https://doc.crablang.org/nightly/std/primitive.char.html#method.to_lowercase
[`PoisonError`]: https://doc.crablang.org/nightly/std/sync/struct.PoisonError.html
[`RwLock`]: https://doc.crablang.org/nightly/std/sync/struct.RwLock.html
[`Mutex`]: https://doc.crablang.org/nightly/std/sync/struct.Mutex.html
[`FromRawFd`]: https://doc.crablang.org/nightly/std/os/unix/io/trait.FromRawFd.html
[`AsRawFd`]: https://doc.crablang.org/nightly/std/os/unix/io/trait.AsRawFd.html
[`Stdio`]: https://doc.crablang.org/nightly/std/process/struct.Stdio.html
[`ChildStdin`]: https://doc.crablang.org/nightly/std/process/struct.ChildStdin.html
[`ChildStdout`]: https://doc.crablang.org/nightly/std/process/struct.ChildStdout.html
[`ChildStderr`]: https://doc.crablang.org/nightly/std/process/struct.ChildStderr.html
[`io::ErrorKind`]: https://doc.crablang.org/nightly/std/io/enum.ErrorKind.html
[debugfmt]: https://www.reddit.com/r/crablang/comments/3ceaui/psa_produces_prettyprinted_debug_output/
[`DerefMut`]: https://doc.crablang.org/nightly/std/ops/trait.DerefMut.html
[`mem::align_of`]: https://doc.crablang.org/nightly/std/mem/fn.align_of.html
[align]: https://github.com/crablang/crablang/pull/25646
[`mem::min_align_of`]: https://doc.crablang.org/nightly/std/mem/fn.min_align_of.html
[typos]: https://github.com/crablang/crablang/pull/26087
[nop]: https://github.com/crablang/crablang/pull/26336
[fat]: https://github.com/crablang/crablang/pull/26411
[dst]: https://github.com/crablang/rfcs/blob/master/text/0982-dst-coercion.md
[parcodegen]: https://github.com/crablang/crablang/pull/26018
[packed]: https://github.com/crablang/crablang/pull/25541
[ad]: https://github.com/crablang/crablang/pull/27382
[win]: https://github.com/crablang/crablang/pull/25350

Version 1.1.0 (2015-06-25)
=========================

* ~850 changes, numerous bugfixes

Highlights
----------

* The [`std::fs` module has been expanded][fs] to expand the set of
  functionality exposed:
  * `DirEntry` now supports optimizations like `file_type` and `metadata` which
    don't incur a syscall on some platforms.
  * A `symlink_metadata` function has been added.
  * The `fs::Metadata` structure now lowers to its OS counterpart, providing
    access to all underlying information.
* The compiler now contains extended explanations of many errors. When an error
  with an explanation occurs the compiler suggests using the `--explain` flag
  to read the explanation. Error explanations are also [available online][err-index].
* Thanks to multiple [improvements][sk] to [type checking][pre], as
  well as other work, the time to bootstrap the compiler decreased by
  32%.

Libraries
---------

* The [`str::split_whitespace`] method splits a string on unicode
  whitespace boundaries.
* On both Windows and Unix, new extension traits provide conversion of
  I/O types to and from the underlying system handles. On Unix, these
  traits are [`FromRawFd`] and [`AsRawFd`], on Windows `FromRawHandle`
  and `AsRawHandle`. These are implemented for `File`, `TcpStream`,
  `TcpListener`, and `UpdSocket`. Further implementations for
  `std::process` will be stabilized later.
* On Unix, [`std::os::unix::symlink`] creates symlinks. On
  Windows, symlinks can be created with
  `std::os::windows::symlink_dir` and
  `std::os::windows::symlink_file`.
* The `mpsc::Receiver` type can now be converted into an iterator with
  `into_iter` on the [`IntoIterator`] trait.
* `Ipv4Addr` can be created from `u32` with the `From<u32>`
  implementation of the [`From`] trait.
* The `Debug` implementation for `RangeFull` [creates output that is
  more consistent with other implementations][rf].
* [`Debug` is implemented for `File`][file].
* The `Default` implementation for `Arc` [no longer requires `Sync +
  Send`][arc].
* [The `Iterator` methods `count`, `nth`, and `last` have been
  overridden for slices to have *O*(1) performance instead of *O*(*n*)][si].
* Incorrect handling of paths on Windows has been improved in both the
  compiler and the standard library.
* [`AtomicPtr` gained a `Default` implementation][ap].
* In accordance with CrabLang's policy on arithmetic overflow `abs` now
  [panics on overflow when debug assertions are enabled][abs].
* The [`Cloned`] iterator, which was accidentally left unstable for
  1.0 [has been stabilized][c].
* The [`Incoming`] iterator, which iterates over incoming TCP
  connections, and which was accidentally unnamable in 1.0, [is now
  properly exported][inc].
* [`BinaryHeap`] no longer corrupts itself [when functions called by
  `sift_up` or `sift_down` panic][bh].
* The [`split_off`] method of `LinkedList` [no longer corrupts
  the list in certain scenarios][ll].

Misc
----

* Type checking performance [has improved notably][sk] with
  [multiple improvements][pre].
* The compiler [suggests code changes][ch] for more errors.
* crablangc and it's build system have experimental support for [building
  toolchains against MUSL][m] instead of glibc on Linux.
* The compiler defines the `target_env` cfg value, which is used for
  distinguishing toolchains that are otherwise for the same
  platform. Presently this is set to `gnu` for common GNU Linux
  targets and for MinGW targets, and `musl` for MUSL Linux targets.
* The [`cargo crablangc`][crc] command invokes a build with custom flags
  to crablangc.
* [Android executables are always position independent][pie].
* [The `drop_with_repr_extern` lint warns about mixing `repr(C)`
  with `Drop`][24935].

[`str::split_whitespace`]: https://doc.crablang.org/nightly/std/primitive.str.html#method.split_whitespace
[`FromRawFd`]: https://doc.crablang.org/nightly/std/os/unix/io/trait.FromRawFd.html
[`AsRawFd`]: https://doc.crablang.org/nightly/std/os/unix/io/trait.AsRawFd.html
[`std::os::unix::symlink`]: https://doc.crablang.org/nightly/std/os/unix/fs/fn.symlink.html
[`IntoIterator`]: https://doc.crablang.org/nightly/std/iter/trait.IntoIterator.html
[`From`]: https://doc.crablang.org/nightly/std/convert/trait.From.html
[rf]: https://github.com/crablang/crablang/pull/24491
[err-index]: https://doc.crablang.org/error-index.html
[sk]: https://github.com/crablang/crablang/pull/24615
[pre]: https://github.com/crablang/crablang/pull/25323
[file]: https://github.com/crablang/crablang/pull/24598
[ch]: https://github.com/crablang/crablang/pull/24683
[arc]: https://github.com/crablang/crablang/pull/24695
[si]: https://github.com/crablang/crablang/pull/24701
[ap]: https://github.com/crablang/crablang/pull/24834
[m]: https://github.com/crablang/crablang/pull/24777
[fs]: https://github.com/crablang/rfcs/blob/master/text/1044-io-fs-2.1.md
[crc]: https://github.com/crablang/cargo/pull/1568
[pie]: https://github.com/crablang/crablang/pull/24953
[abs]: https://github.com/crablang/crablang/pull/25441
[c]: https://github.com/crablang/crablang/pull/25496
[`Cloned`]: https://doc.crablang.org/nightly/std/iter/struct.Cloned.html
[`Incoming`]: https://doc.crablang.org/nightly/std/net/struct.Incoming.html
[inc]: https://github.com/crablang/crablang/pull/25522
[bh]: https://github.com/crablang/crablang/pull/25856
[`BinaryHeap`]: https://doc.crablang.org/nightly/std/collections/struct.BinaryHeap.html
[ll]: https://github.com/crablang/crablang/pull/26022
[`split_off`]: https://doc.crablang.org/nightly/collections/linked_list/struct.LinkedList.html#method.split_off
[24935]: https://github.com/crablang/crablang/pull/24935

Version 1.0.0 (2015-05-15)
========================

* ~1500 changes, numerous bugfixes

Highlights
----------

* The vast majority of the standard library is now `#[stable]`. It is
  no longer possible to use unstable features with a stable build of
  the compiler.
* Many popular crates on [crates.io] now work on the stable release
  channel.
* Arithmetic on basic integer types now [checks for overflow in debug
  builds][overflow].

Language
--------

* Several [restrictions have been added to trait coherence][coh] in
  order to make it easier for upstream authors to change traits
  without breaking downstream code.
* Digits of binary and octal literals are [lexed more eagerly][lex] to
  improve error messages and macro behavior. For example, `0b1234` is
  now lexed as `0b1234` instead of two tokens, `0b1` and `234`.
* Trait bounds [are always invariant][inv], eliminating the need for
  the `PhantomFn` and `MarkerTrait` lang items, which have been
  removed.
* ["-" is no longer a valid character in crate names][cr], the `extern crate
  "foo" as bar` syntax has been replaced with `extern crate foo as
  bar`, and Cargo now automatically translates "-" in *package* names
  to underscore for the crate name.
* [Lifetime shadowing is an error][lt].
* [`Send` no longer implies `'static`][send-rfc].
* [UFCS now supports trait-less associated paths][moar-ufcs] like
  `MyType::default()`.
* Primitive types [now have inherent methods][prim-inherent],
  obviating the need for extension traits like `SliceExt`.
* Methods with `Self: Sized` in their `where` clause are [considered
  object-safe][self-sized], allowing many extension traits like
  `IteratorExt` to be merged into the traits they extended.
* You can now [refer to associated types][assoc-where] whose
  corresponding trait bounds appear only in a `where` clause.
* The final bits of [OIBIT landed][oibit-final], meaning that traits
  like `Send` and `Sync` are now library-defined.
* A [Reflect trait][reflect] was introduced, which means that
  downcasting via the `Any` trait is effectively limited to concrete
  types. This helps retain the potentially-important "parametricity"
  property: generic code cannot behave differently for different type
  arguments except in minor ways.
* The `unsafe_destructor` feature is now deprecated in favor of the
  [new `dropck`][rfc769]. This change is a major reduction in unsafe
  code.

Libraries
---------

* The `thread_local` module [has been renamed to `std::thread`][th].
* The methods of `IteratorExt` [have been moved to the `Iterator`
  trait itself][23300].
* Several traits that implement CrabLang's conventions for type
  conversions, `AsMut`, `AsRef`, `From`, and `Into` have been
  [centralized in the `std::convert` module][con].
* The `FromError` trait [was removed in favor of `From`][fe].
* The basic sleep function [has moved to
  `std::thread::sleep_ms`][slp].
* The `splitn` function now takes an `n` parameter that represents the
  number of items yielded by the returned iterator [instead of the
  number of 'splits'][spl].
* [On Unix, all file descriptors are `CLOEXEC` by default][clo].
* [Derived implementations of `PartialOrd` now order enums according
  to their explicitly-assigned discriminants][po].
* [Methods for searching strings are generic over `Pattern`s][pat],
  implemented presently by `&char`, `&str`, `FnMut(char) -> bool` and
  some others.
* [In method resolution, object methods are resolved before inherent
  methods][meth].
* [`String::from_str` has been deprecated in favor of the `From` impl,
  `String::from`][24517].
* [`io::Error` implements `Sync`][ios].
* [The `words` method on `&str` has been replaced with
  `split_whitespace`][sw], to avoid answering the tricky question, 'what is
  a word?'
* The new path and IO modules are complete and `#[stable]`. This
  was the major library focus for this cycle.
* The path API was [revised][path-normalize] to normalize `.`,
  adjusting the tradeoffs in favor of the most common usage.
* A large number of remaining APIs in `std` were also stabilized
  during this cycle; about 75% of the non-deprecated API surface
  is now stable.
* The new [string pattern API][string-pattern] landed, which makes
  the string slice API much more internally consistent and flexible.
* A new set of [generic conversion traits][conversion] replaced
  many existing ad hoc traits.
* Generic numeric traits were [completely removed][num-traits]. This
  was made possible thanks to inherent methods for primitive types,
  and the removal gives maximal flexibility for designing a numeric
  hierarchy in the future.
* The `Fn` traits are now related via [inheritance][fn-inherit]
  and provide ergonomic [blanket implementations][fn-blanket].
* The `Index` and `IndexMut` traits were changed to
  [take the index by value][index-value], enabling code like
  `hash_map["string"]` to work.
* `Copy` now [inherits][copy-clone] from `Clone`, meaning that all
  `Copy` data is known to be `Clone` as well.

Misc
----

* Many errors now have extended explanations that can be accessed with
  the `--explain` flag to `crablangc`.
* Many new examples have been added to the standard library
  documentation.
* crablangdoc has received a number of improvements focused on completion
  and polish.
* Metadata was tuned, shrinking binaries [by 27%][metadata-shrink].
* Much headway was made on ecosystem-wide CI, making it possible
  to [compare builds for breakage][ci-compare].


[crates.io]: http://crates.io
[clo]: https://github.com/crablang/crablang/pull/24034
[coh]: https://github.com/crablang/rfcs/blob/master/text/1023-rebalancing-coherence.md
[con]: https://github.com/crablang/crablang/pull/23875
[cr]: https://github.com/crablang/crablang/pull/23419
[fe]: https://github.com/crablang/crablang/pull/23879
[23300]: https://github.com/crablang/crablang/pull/23300
[inv]: https://github.com/crablang/crablang/pull/23938
[ios]: https://github.com/crablang/crablang/pull/24133
[lex]: https://github.com/crablang/rfcs/blob/master/text/0879-small-base-lexing.md
[lt]: https://github.com/crablang/crablang/pull/24057
[meth]: https://github.com/crablang/crablang/pull/24056
[pat]: https://github.com/crablang/rfcs/blob/master/text/0528-string-patterns.md
[po]: https://github.com/crablang/crablang/pull/24270
[24517]: https://github.com/crablang/crablang/pull/24517
[slp]: https://github.com/crablang/crablang/pull/23949
[spl]: https://github.com/crablang/rfcs/blob/master/text/0979-align-splitn-with-other-languages.md
[sw]: https://github.com/crablang/rfcs/blob/master/text/1054-str-words.md
[th]: https://github.com/crablang/rfcs/blob/master/text/0909-move-thread-local-to-std-thread.md
[send-rfc]: https://github.com/crablang/rfcs/blob/master/text/0458-send-improvements.md
[moar-ufcs]: https://github.com/crablang/crablang/pull/22172
[prim-inherent]: https://github.com/crablang/crablang/pull/23104
[overflow]: https://github.com/crablang/rfcs/blob/master/text/0560-integer-overflow.md
[metadata-shrink]: https://github.com/crablang/crablang/pull/22971
[self-sized]: https://github.com/crablang/crablang/pull/22301
[assoc-where]: https://github.com/crablang/crablang/pull/22512
[string-pattern]: https://github.com/crablang/crablang/pull/22466
[oibit-final]: https://github.com/crablang/crablang/pull/21689
[reflect]: https://github.com/crablang/crablang/pull/23712
[conversion]: https://github.com/crablang/rfcs/pull/529
[num-traits]: https://github.com/crablang/crablang/pull/23549
[index-value]: https://github.com/crablang/crablang/pull/23601
[rfc769]: https://github.com/crablang/rfcs/pull/769
[ci-compare]: https://gist.github.com/brson/a30a77836fbec057cbee
[fn-inherit]: https://github.com/crablang/crablang/pull/23282
[fn-blanket]: https://github.com/crablang/crablang/pull/23895
[copy-clone]: https://github.com/crablang/crablang/pull/23860
[path-normalize]: https://github.com/crablang/crablang/pull/23229


Version 1.0.0-alpha.2 (2015-02-20)
=====================================

* ~1300 changes, numerous bugfixes

* Highlights

    * The various I/O modules were [overhauled][io-rfc] to reduce
      unnecessary abstractions and provide better interoperation with
      the underlying platform. The old `io` module remains temporarily
      at `std::old_io`.
    * The standard library now [participates in feature gating][feat],
      so use of unstable libraries now requires a `#![feature(...)]`
      attribute. The impact of this change is [described on the
      forum][feat-forum]. [RFC][feat-rfc].

* Language

    * `for` loops [now operate on the `IntoIterator` trait][into],
      which eliminates the need to call `.iter()`, etc. to iterate
      over collections. There are some new subtleties to remember
      though regarding what sort of iterators various types yield, in
      particular that `for foo in bar { }` yields values from a move
      iterator, destroying the original collection. [RFC][into-rfc].
    * Objects now have [default lifetime bounds][obj], so you don't
      have to write `Box<Trait+'static>` when you don't care about
      storing references. [RFC][obj-rfc].
    * In types that implement `Drop`, [lifetimes must outlive the
      value][drop]. This will soon make it possible to safely
      implement `Drop` for types where `#[unsafe_destructor]` is now
      required. Read the [gorgeous RFC][drop-rfc] for details.
    * The fully qualified <T as Trait>::X syntax lets you set the Self
      type for a trait method or associated type. [RFC][ufcs-rfc].
    * References to types that implement `Deref<U>` now [automatically
      coerce to references][deref] to the dereferenced type `U`,
      e.g. `&T where T: Deref<U>` automatically coerces to `&U`. This
      should eliminate many unsightly uses of `&*`, as when converting
      from references to vectors into references to
      slices. [RFC][deref-rfc].
    * The explicit [closure kind syntax][close] (`|&:|`, `|&mut:|`,
      `|:|`) is obsolete and closure kind is inferred from context.
    * [`Self` is a keyword][Self].

* Libraries

    * The `Show` and `String` formatting traits [have been
      renamed][fmt] to `Debug` and `Display` to more clearly reflect
      their related purposes. Automatically getting a string
      conversion to use with `format!("{:?}", something_to_debug)` is
      now written `#[derive(Debug)]`.
    * Abstract [OS-specific string types][osstr], `std::ff::{OsString,
      OsStr}`, provide strings in platform-specific encodings for easier
      interop with system APIs. [RFC][osstr-rfc].
    * The `boxed::into_raw` and `Box::from_raw` functions [convert
      between `Box<T>` and `*mut T`][boxraw], a common pattern for
      creating raw pointers.

* Tooling

    * Certain long error messages of the form 'expected foo found bar'
      are now [split neatly across multiple
      lines][multiline]. Examples in the PR.
    * On Unix CrabLang can be [uninstalled][un] by running
      `/usr/local/lib/crablanglib/uninstall.sh`.
    * The `#[crablangc_on_unimplemented]` attribute, requiring the
      'on_unimplemented' feature, lets crablangc [display custom error
      messages when a trait is expected to be implemented for a type
      but is not][onun].

* Misc

    * CrabLang is tested against a [LALR grammar][lalr], which parses
      almost all the CrabLang files that crablangc does.

[boxraw]: https://github.com/crablang/crablang/pull/21318
[close]: https://github.com/crablang/crablang/pull/21843
[deref]: https://github.com/crablang/crablang/pull/21351
[deref-rfc]: https://github.com/crablang/rfcs/blob/master/text/0241-deref-conversions.md
[drop]: https://github.com/crablang/crablang/pull/21972
[drop-rfc]: https://github.com/crablang/rfcs/blob/master/text/0769-sound-generic-drop.md
[feat]: https://github.com/crablang/crablang/pull/21248
[feat-forum]: https://users.crablang.org/t/psa-important-info-about-crablangcs-new-feature-staging/82/5
[feat-rfc]: https://github.com/crablang/rfcs/blob/master/text/0507-release-channels.md
[fmt]: https://github.com/crablang/crablang/pull/21457
[into]: https://github.com/crablang/crablang/pull/20790
[into-rfc]: https://github.com/crablang/rfcs/blob/master/text/0235-collections-conventions.md#intoiterator-and-iterable
[io-rfc]: https://github.com/crablang/rfcs/blob/master/text/0517-io-os-reform.md
[lalr]: https://github.com/crablang/crablang/pull/21452
[multiline]: https://github.com/crablang/crablang/pull/19870
[obj]: https://github.com/crablang/crablang/pull/22230
[obj-rfc]: https://github.com/crablang/rfcs/blob/master/text/0599-default-object-bound.md
[onun]: https://github.com/crablang/crablang/pull/20889
[osstr]: https://github.com/crablang/crablang/pull/21488
[osstr-rfc]: https://github.com/crablang/rfcs/blob/master/text/0517-io-os-reform.md
[Self]: https://github.com/crablang/crablang/pull/22158
[ufcs-rfc]: https://github.com/crablang/rfcs/blob/master/text/0132-ufcs.md
[un]: https://github.com/crablang/crablang/pull/22256


Version 1.0.0-alpha (2015-01-09)
==================================

  * ~2400 changes, numerous bugfixes

  * Highlights

    * The language itself is considered feature complete for 1.0,
      though there will be many usability improvements and bugfixes
      before the final release.
    * Nearly 50% of the public API surface of the standard library has
      been declared 'stable'. Those interfaces are unlikely to change
      before 1.0.
    * The long-running debate over integer types has been
      [settled][ints]: CrabLang will ship with types named `isize` and
      `usize`, rather than `int` and `uint`, for pointer-sized
      integers. Guidelines will be rolled out during the alpha cycle.
    * Most crates that are not `std` have been moved out of the CrabLang
      distribution into the Cargo ecosystem so they can evolve
      separately and don't need to be stabilized as quickly, including
      'time', 'getopts', 'num', 'regex', and 'term'.
    * Documentation continues to be expanded with more API coverage, more
      examples, and more in-depth explanations. The guides have been
      consolidated into [The CrabLang Programming Language][trpl].
    * "[CrabLang By Example][rbe]" is now maintained by the CrabLang team.
    * All official CrabLang binary installers now come with [Cargo], the
      CrabLang package manager.

* Language

    * Closures have been [completely redesigned][unboxed] to be
      implemented in terms of traits, can now be used as generic type
      bounds and thus monomorphized and inlined, or via an opaque
      pointer (boxed) as in the old system. The new system is often
      referred to as 'unboxed' closures.
    * Traits now support [associated types][assoc], allowing families
      of related types to be defined together and used generically in
      powerful ways.
    * Enum variants are [namespaced by their type names][enum].
    * [`where` clauses][where] provide a more versatile and attractive
      syntax for specifying generic bounds, though the previous syntax
      remains valid.
    * CrabLang again picks a [fallback][fb] (either i32 or f64) for uninferred
      numeric types.
    * CrabLang [no longer has a runtime][rt] of any description, and only
      supports OS threads, not green threads.
    * At long last, CrabLang has been overhauled for 'dynamically-sized
      types' ([DST]), which integrates 'fat pointers' (object types,
      arrays, and `str`) more deeply into the type system, making it
      more consistent.
    * CrabLang now has a general [range syntax][range], `i..j`, `i..`, and
      `..j` that produce range types and which, when combined with the
      `Index` operator and multidispatch, leads to a convenient slice
      notation, `[i..j]`.
    * The new range syntax revealed an ambiguity in the fixed-length
      array syntax, so now fixed length arrays [are written `[T;
      N]`][arrays].
    * The `Copy` trait is no longer implemented automatically. Unsafe
      pointers no longer implement `Sync` and `Send` so types
      containing them don't automatically either. `Sync` and `Send`
      are now 'unsafe traits' so one can "forcibly" implement them via
      `unsafe impl` if a type confirms to the requirements for them
      even though the internals do not (e.g. structs containing unsafe
      pointers like `Arc`). These changes are intended to prevent some
      footguns and are collectively known as [opt-in built-in
      traits][oibit] (though `Sync` and `Send` will soon become pure
      library types unknown to the compiler).
    * Operator traits now take their operands [by value][ops], and
      comparison traits can use multidispatch to compare one type
      against multiple other types, allowing e.g. `String` to be
      compared with `&str`.
    * `if let` and `while let` are no longer feature-gated.
    * CrabLang has adopted a more [uniform syntax for escaping unicode
      characters][unicode].
    * `macro_rules!` [has been declared stable][mac]. Though it is a
      flawed system it is sufficiently popular that it must be usable
      for 1.0. Effort has gone into [future-proofing][mac-future] it
      in ways that will allow other macro systems to be developed in
      parallel, and won't otherwise impact the evolution of the
      language.
    * The prelude has been [pared back significantly][prelude] such
      that it is the minimum necessary to support the most pervasive
      code patterns, and through [generalized where clauses][where]
      many of the prelude extension traits have been consolidated.
    * CrabLang's rudimentary reflection [has been removed][refl], as it
      incurred too much code generation for little benefit.
    * [Struct variants][structvars] are no longer feature-gated.
    * Trait bounds can be [polymorphic over lifetimes][hrtb]. Also
      known as 'higher-ranked trait bounds', this crucially allows
      unboxed closures to work.
    * Macros invocations surrounded by parens or square brackets and
      not terminated by a semicolon are [parsed as
      expressions][macros], which makes expressions like `vec![1i32,
      2, 3].len()` work as expected.
    * Trait objects now implement their traits automatically, and
      traits that can be coerced to objects now must be [object
      safe][objsafe].
    * Automatically deriving traits is now done with `#[derive(...)]`
      not `#[deriving(...)]` for [consistency with other naming
      conventions][derive].
    * Importing the containing module or enum at the same time as
      items or variants they contain is [now done with `self` instead
      of `mod`][self], as in use `foo::{self, bar}`
    * Glob imports are no longer feature-gated.
    * The `box` operator and `box` patterns have been feature-gated
      pending a redesign. For now unique boxes should be allocated
      like other containers, with `Box::new`.

* Libraries

    * A [series][coll1] of [efforts][coll2] to establish
      [conventions][coll3] for collections types has resulted in API
      improvements throughout the standard library.
    * New [APIs for error handling][err] provide ergonomic interop
      between error types, and [new conventions][err-conv] describe
      more clearly the recommended error handling strategies in CrabLang.
    * The `fail!` macro has been renamed to [`panic!`][panic] so that
      it is easier to discuss failure in the context of error handling
      without making clarifications as to whether you are referring to
      the 'fail' macro or failure more generally.
    * On Linux, `OsRng` prefers the new, more reliable `getrandom`
      syscall when available.
    * The 'serialize' crate has been renamed 'crablangc-serialize' and
      moved out of the distribution to Cargo. Although it is widely
      used now, it is expected to be superseded in the near future.
    * The `Show` formatter, typically implemented with
      `#[derive(Show)]` is [now requested with the `{:?}`
      specifier][show] and is intended for use by all types, for uses
      such as `println!` debugging. The new `String` formatter must be
      implemented by hand, uses the `{}` specifier, and is intended
      for full-fidelity conversions of things that can logically be
      represented as strings.

* Tooling

    * [Flexible target specification][flex] allows crablangc's code
      generation to be configured to support otherwise-unsupported
      platforms.
    * CrabLang comes with crablang-gdb and crablang-lldb scripts that launch their
      respective debuggers with CrabLang-appropriate pretty-printing.
    * The Windows installation of CrabLang is distributed with the
      MinGW components currently required to link binaries on that
      platform.

* Misc

    * Nullable enum optimizations have been extended to more types so
      that e.g. `Option<Vec<T>>` and `Option<String>` take up no more
      space than the inner types themselves.
    * Work has begun on supporting AArch64.

[Cargo]: https://crates.io
[unboxed]: http://smallcultfollowing.com/babysteps/blog/2014/11/26/purging-proc/
[enum]: https://github.com/crablang/rfcs/blob/master/text/0390-enum-namespacing.md
[flex]: https://github.com/crablang/rfcs/blob/master/text/0131-target-specification.md
[err]: https://github.com/crablang/rfcs/blob/master/text/0201-error-chaining.md
[err-conv]: https://github.com/crablang/rfcs/blob/master/text/0236-error-conventions.md
[rt]: https://github.com/crablang/rfcs/blob/master/text/0230-remove-runtime.md
[mac]: https://github.com/crablang/rfcs/blob/master/text/0453-macro-reform.md
[mac-future]: https://github.com/crablang/rfcs/pull/550
[DST]: http://smallcultfollowing.com/babysteps/blog/2014/01/05/dst-take-5/
[coll1]: https://github.com/crablang/rfcs/blob/master/text/0235-collections-conventions.md
[coll2]: https://github.com/crablang/rfcs/blob/master/text/0509-collections-reform-part-2.md
[coll3]: https://github.com/crablang/rfcs/blob/master/text/0216-collection-views.md
[ops]: https://github.com/crablang/rfcs/blob/master/text/0439-cmp-ops-reform.md
[prelude]: https://github.com/crablang/rfcs/blob/master/text/0503-prelude-stabilization.md
[where]: https://github.com/crablang/rfcs/blob/master/text/0135-where.md
[refl]: https://github.com/crablang/rfcs/blob/master/text/0379-remove-reflection.md
[panic]: https://github.com/crablang/rfcs/blob/master/text/0221-panic.md
[structvars]: https://github.com/crablang/rfcs/blob/master/text/0418-struct-variants.md
[hrtb]: https://github.com/crablang/rfcs/blob/master/text/0387-higher-ranked-trait-bounds.md
[unicode]: https://github.com/crablang/rfcs/blob/master/text/0446-es6-unicode-escapes.md
[oibit]: https://github.com/crablang/rfcs/blob/master/text/0019-opt-in-builtin-traits.md
[macros]: https://github.com/crablang/rfcs/blob/master/text/0378-expr-macros.md
[range]: https://github.com/crablang/rfcs/blob/master/text/0439-cmp-ops-reform.md#indexing-and-slicing
[arrays]: https://github.com/crablang/rfcs/blob/master/text/0520-new-array-repeat-syntax.md
[show]: https://github.com/crablang/rfcs/blob/master/text/0504-show-stabilization.md
[derive]: https://github.com/crablang/rfcs/blob/master/text/0534-deriving2derive.md
[self]: https://github.com/crablang/rfcs/blob/master/text/0532-self-in-use.md
[fb]: https://github.com/crablang/rfcs/blob/master/text/0212-restore-int-fallback.md
[objsafe]: https://github.com/crablang/rfcs/blob/master/text/0255-object-safety.md
[assoc]: https://github.com/crablang/rfcs/blob/master/text/0195-associated-items.md
[ints]: https://github.com/crablang/rfcs/pull/544#issuecomment-68760871
[trpl]: https://doc.crablang.org/book/index.html
[rbe]: http://crablangbyexample.com/


Version 0.12.0 (2014-10-09)
=============================

  * ~1900 changes, numerous bugfixes

  * Highlights

    * The introductory documentation (now called The CrabLang Guide) has
      been completely rewritten, as have a number of supplementary
      guides.
    * CrabLang's package manager, Cargo, continues to improve and is
      sometimes considered to be quite awesome.
    * Many API's in `std` have been reviewed and updated for
      consistency with the in-development CrabLang coding
      guidelines. The standard library documentation tracks
      stabilization progress.
    * Minor libraries have been moved out-of-tree to the crablang org
      on GitHub: uuid, semver, glob, num, hexfloat, fourcc. They can
      be installed with Cargo.
    * Lifetime elision allows lifetime annotations to be left off of
      function declarations in many common scenarios.
    * CrabLang now works on 64-bit Windows.

  * Language
    * Indexing can be overloaded with the `Index` and `IndexMut`
      traits.
    * The `if let` construct takes a branch only if the `let` pattern
      matches, currently behind the 'if_let' feature gate.
    * 'where clauses', a more flexible syntax for specifying trait
      bounds that is more aesthetic, have been added for traits and
      free functions. Where clauses will in the future make it
      possible to constrain associated types, which would be
      impossible with the existing syntax.
    * A new slicing syntax (e.g. `[0..4]`) has been introduced behind
      the 'slicing_syntax' feature gate, and can be overloaded with
      the `Slice` or `SliceMut` traits.
    * The syntax for matching of sub-slices has been changed to use a
      postfix `..` instead of prefix (.e.g. `[a, b, c..]`), for
      consistency with other uses of `..` and to future-proof
      potential additional uses of the syntax.
    * The syntax for matching inclusive ranges in patterns has changed
      from `0..3` to `0...4` to be consistent with the exclusive range
      syntax for slicing.
    * Matching of sub-slices in non-tail positions (e.g.  `[a.., b,
      c]`) has been put behind the 'advanced_slice_patterns' feature
      gate and may be removed in the future.
    * Components of tuples and tuple structs can be extracted using
      the `value.0` syntax, currently behind the `tuple_indexing`
      feature gate.
    * The `#[crate_id]` attribute is no longer supported; versioning
      is handled by the package manager.
    * Renaming crate imports are now written `extern crate foo as bar`
      instead of `extern crate bar = foo`.
    * Renaming use statements are now written `use foo as bar` instead
      of `use bar = foo`.
    * `let` and `match` bindings and argument names in macros are now
      hygienic.
    * The new, more efficient, closure types ('unboxed closures') have
      been added under a feature gate, 'unboxed_closures'. These will
      soon replace the existing closure types, once higher-ranked
      trait lifetimes are added to the language.
    * `move` has been added as a keyword, for indicating closures
      that capture by value.
    * Mutation and assignment is no longer allowed in pattern guards.
    * Generic structs and enums can now have trait bounds.
    * The `Share` trait is now called `Sync` to free up the term
      'shared' to refer to 'shared reference' (the default reference
      type.
    * Dynamically-sized types have been mostly implemented,
      unifying the behavior of fat-pointer types with the rest of the
      type system.
    * As part of dynamically-sized types, the `Sized` trait has been
      introduced, which qualifying types implement by default, and
      which type parameters expect by default. To specify that a type
      parameter does not need to be sized, write `<Sized? T>`. Most
      types are `Sized`, notable exceptions being unsized arrays
      (`[T]`) and trait types.
    * Closures can return `!`, as in `|| -> !` or `proc() -> !`.
    * Lifetime bounds can now be applied to type parameters and object
      types.
    * The old, reference counted GC type, `Gc<T>` which was once
      denoted by the `@` sigil, has finally been removed. GC will be
      revisited in the future.

  * Libraries
    * Library documentation has been improved for a number of modules.
    * Bit-vectors, collections::bitv has been modernized.
    * The url crate is deprecated in favor of
      http://github.com/servo/crablang-url, which can be installed with
      Cargo.
    * Most I/O stream types can be cloned and subsequently closed from
      a different thread.
    * A `std::time::Duration` type has been added for use in I/O
      methods that rely on timers, as well as in the 'time' crate's
      `Timespec` arithmetic.
    * The runtime I/O abstraction layer that enabled the green thread
      scheduler to do non-thread-blocking I/O has been removed, along
      with the libuv-based implementation employed by the green thread
      scheduler. This will greatly simplify the future I/O work.
    * `collections::btree` has been rewritten to have a more
      idiomatic and efficient design.

  * Tooling
    * crablangdoc output now indicates the stability levels of API's.
    * The `--crate-name` flag can specify the name of the crate
      being compiled, like `#[crate_name]`.
    * The `-C metadata` specifies additional metadata to hash into
      symbol names, and `-C extra-filename` specifies additional
      information to put into the output filename, for use by the
      package manager for versioning.
    * debug info generation has continued to improve and should be
      more reliable under both gdb and lldb.
    * crablangc has experimental support for compiling in parallel
      using the `-C codegen-units` flag.
    * crablangc no longer encodes rpath information into binaries by
      default.

  * Misc
    * Stack usage has been optimized with LLVM lifetime annotations.
    * Official CrabLang binaries on Linux are more compatible with older
      kernels and distributions, built on CentOS 5.10.


Version 0.11.0 (2014-07-02)
==========================

  * ~1700 changes, numerous bugfixes

  * Language
    * ~[T] has been removed from the language. This type is superseded by
      the Vec<T> type.
    * ~str has been removed from the language. This type is superseded by
      the String type.
    * ~T has been removed from the language. This type is superseded by the
      Box<T> type.
    * @T has been removed from the language. This type is superseded by the
      standard library's std::gc::Gc<T> type.
    * Struct fields are now all private by default.
    * Vector indices and shift amounts are both required to be a `uint`
      instead of any integral type.
    * Byte character, byte string, and raw byte string literals are now all
      supported by prefixing the normal literal with a `b`.
    * Multiple ABIs are no longer allowed in an ABI string
    * The syntax for lifetimes on closures/procedures has been tweaked
      slightly: `<'a>|A, B|: 'b + K -> T`
    * Floating point modulus has been removed from the language; however it
      is still provided by a library implementation.
    * Private enum variants are now disallowed.
    * The `priv` keyword has been removed from the language.
    * A closure can no longer be invoked through a &-pointer.
    * The `use foo, bar, baz;` syntax has been removed from the language.
    * The transmute intrinsic no longer works on type parameters.
    * Statics now allow blocks/items in their definition.
    * Trait bounds are separated from objects with + instead of : now.
    * Objects can no longer be read while they are mutably borrowed.
    * The address of a static is now marked as insignificant unless the
      #[inline(never)] attribute is placed it.
    * The #[unsafe_destructor] attribute is now behind a feature gate.
    * Struct literals are no longer allowed in ambiguous positions such as
      if, while, match, and for..in.
    * Declaration of lang items and intrinsics are now feature-gated by
      default.
    * Integral literals no longer default to `int`, and floating point
      literals no longer default to `f64`. Literals must be suffixed with an
      appropriate type if inference cannot determine the type of the
      literal.
    * The Box<T> type is no longer implicitly borrowed to &mut T.
    * Procedures are now required to not capture borrowed references.

  * Libraries
    * The standard library is now a "facade" over a number of underlying
      libraries. This means that development on the standard library should
      be speedier due to smaller crates, as well as a clearer line between
      all dependencies.
    * A new library, libcore, lives under the standard library's facade
      which is CrabLang's "0-assumption" library, suitable for embedded and
      kernel development for example.
    * A regex crate has been added to the standard distribution. This crate
      includes statically compiled regular expressions.
    * The unwrap/unwrap_err methods on Result require a Show bound for
      better error messages.
    * The return types of the std::comm primitives have been centralized
      around the Result type.
    * A number of I/O primitives have gained the ability to time out their
      operations.
    * A number of I/O primitives have gained the ability to close their
      reading/writing halves to cancel pending operations.
    * Reverse iterator methods have been removed in favor of `rev()` on
      their forward-iteration counterparts.
    * A bitflags! macro has been added to enable easy interop with C and
      management of bit flags.
    * A debug_assert! macro is now provided which is disabled when
      `--cfg ndebug` is passed to the compiler.
    * A graphviz crate has been added for creating .dot files.
    * The std::cast module has been migrated into std::mem.
    * The std::local_data api has been migrated from freestanding functions
      to being based on methods.
    * The Pod trait has been renamed to Copy.
    * jemalloc has been added as the default allocator for types.
    * The API for allocating memory has been changed to use proper alignment
      and sized deallocation
    * Connecting a TcpStream or binding a TcpListener is now based on a
      string address and a u16 port. This allows connecting to a hostname as
      opposed to an IP.
    * The Reader trait now contains a core method, read_at_least(), which
      correctly handles many repeated 0-length reads.
    * The process-spawning API is now centered around a builder-style
      Command struct.
    * The :? printing qualifier has been moved from the standard library to
      an external libdebug crate.
    * Eq/Ord have been renamed to PartialEq/PartialOrd. TotalEq/TotalOrd
      have been renamed to Eq/Ord.
    * The select/plural methods have been removed from format!. The escapes
      for { and } have also changed from \{ and \} to {{ and }},
      respectively.
    * The TaskBuilder API has been re-worked to be a true builder, and
      extension traits for spawning native/green tasks have been added.

  * Tooling
    * All breaking changes to the language or libraries now have their
      commit message annotated with `[breaking-change]` to allow for easy
      discovery of breaking changes.
    * The compiler will now try to suggest how to annotate lifetimes if a
      lifetime-related error occurs.
    * Debug info continues to be improved greatly with general bug fixes and
      better support for situations like link time optimization (LTO).
    * Usage of syntax extensions when cross-compiling has been fixed.
    * Functionality equivalent to GCC & Clang's -ffunction-sections,
      -fdata-sections and --gc-sections has been enabled by default
    * The compiler is now stricter about where it will load module files
      from when a module is declared via `mod foo;`.
    * The #[phase(syntax)] attribute has been renamed to #[phase(plugin)].
      Syntax extensions are now discovered via a "plugin registrar" type
      which will be extended in the future to other various plugins.
    * Lints have been restructured to allow for dynamically loadable lints.
    * A number of crablangdoc improvements:
      * The HTML output has been visually redesigned.
      * Markdown is now powered by hoedown instead of sundown.
      * Searching heuristics have been greatly improved.
      * The search index has been reduced in size by a great amount.
      * Cross-crate documentation via `pub use` has been greatly improved.
      * Primitive types are now hyperlinked and documented.
    * Documentation has been moved from static.crablang.org/doc to
      doc.crablang.org
    * A new sandbox, play.crablang.org, is available for running and
      sharing crablang code examples on-line.
    * Unused attributes are now more robustly warned about.
    * The dead_code lint now warns about unused struct fields.
    * Cross-compiling to iOS is now supported.
    * Cross-compiling to mipsel is now supported.
    * Stability attributes are now inherited by default and no longer apply
      to intra-crate usage, only inter-crate usage.
    * Error message related to non-exhaustive match expressions have been
      greatly improved.


Version 0.10 (2014-04-03)
=========================

  * ~1500 changes, numerous bugfixes

  * Language
    * A new RFC process is now in place for modifying the language.
    * Patterns with `@`-pointers have been removed from the language.
    * Patterns with unique vectors (`~[T]`) have been removed from the
      language.
    * Patterns with unique strings (`~str`) have been removed from the
      language.
    * `@str` has been removed from the language.
    * `@[T]` has been removed from the language.
    * `@self` has been removed from the language.
    * `@Trait` has been removed from the language.
    * Headers on `~` allocations which contain `@` boxes inside the type for
      reference counting have been removed.
    * The semantics around the lifetimes of temporary expressions have changed,
      see #3511 and #11585 for more information.
    * Cross-crate syntax extensions are now possible, but feature gated. See
      #11151 for more information. This includes both `macro_rules!` macros as
      well as syntax extensions such as `format!`.
    * New lint modes have been added, and older ones have been turned on to be
      warn-by-default.
      * Unnecessary parentheses
      * Uppercase statics
      * Camel Case types
      * Uppercase variables
      * Publicly visible private types
      * `#[deriving]` with raw pointers
    * Unsafe functions can no longer be coerced to closures.
    * Various obscure macros such as `log_syntax!` are now behind feature gates.
    * The `#[simd]` attribute is now behind a feature gate.
    * Visibility is no longer allowed on `extern crate` statements, and
      unnecessary visibility (`priv`) is no longer allowed on `use` statements.
    * Trailing commas are now allowed in argument lists and tuple patterns.
    * The `do` keyword has been removed, it is now a reserved keyword.
    * Default type parameters have been implemented, but are feature gated.
    * Borrowed variables through captures in closures are now considered soundly.
    * `extern mod` is now `extern crate`
    * The `Freeze` trait has been removed.
    * The `Share` trait has been added for types that can be shared among
      threads.
    * Labels in macros are now hygienic.
    * Expression/statement macro invocations can be delimited with `{}` now.
    * Treatment of types allowed in `static mut` locations has been tweaked.
    * The `*` and `.` operators are now overloadable through the `Deref` and
      `DerefMut` traits.
    * `~Trait` and `proc` no longer have `Send` bounds by default.
    * Partial type hints are now supported with the `_` type marker.
    * An `Unsafe` type was introduced for interior mutability. It is now
      considered undefined to transmute from `&T` to `&mut T` without using the
      `Unsafe` type.
    * The #[linkage] attribute was implemented for extern statics/functions.
    * The inner attribute syntax has changed from `#[foo];` to `#![foo]`.
    * `Pod` was renamed to `Copy`.

  * Libraries
    * The `libextra` library has been removed. It has now been decomposed into
      component libraries with smaller and more focused nuggets of
      functionality. The full list of libraries can be found on the
      documentation index page.
    * std: `std::condition` has been removed. All I/O errors are now propagated
      through the `Result` type. In order to assist with error handling, a
      `try!` macro for unwrapping errors with an early return and a lint for
      unused results has been added. See #12039 for more information.
    * std: The `vec` module has been renamed to `slice`.
    * std: A new vector type, `Vec<T>`, has been added in preparation for DST.
      This will become the only growable vector in the future.
    * std: `std::io` now has more public re-exports. Types such as `BufferedReader`
      are now found at `std::io::BufferedReader` instead of
      `std::io::buffered::BufferedReader`.
    * std: `print` and `println` are no longer in the prelude, the `print!` and
      `println!` macros are intended to be used instead.
    * std: `Rc` now has a `Weak` pointer for breaking cycles, and it no longer
      attempts to statically prevent cycles.
    * std: The standard distribution is adopting the policy of pushing failure
      to the user rather than failing in libraries. Many functions (such as
      `slice::last()`) now return `Option<T>` instead of `T` + failing.
    * std: `fmt::Default` has been renamed to `fmt::Show`, and it now has a new
      deriving mode: `#[deriving(Show)]`.
    * std: `ToStr` is now implemented for all types implementing `Show`.
    * std: The formatting trait methods now take `&self` instead of `&T`
    * std: The `invert()` method on iterators has been renamed to `rev()`
    * std: `std::num` has seen a reduction in the genericity of its traits,
      consolidating functionality into a few core traits.
    * std: Backtraces are now printed on task failure if the environment
      variable `CRABLANG_BACKTRACE` is present.
    * std: Naming conventions for iterators have been standardized. More details
      can be found on the wiki's style guide.
    * std: `eof()` has been removed from the `Reader` trait. Specific types may
      still implement the function.
    * std: Networking types are now cloneable to allow simultaneous reads/writes.
    * std: `assert_approx_eq!` has been removed
    * std: The `e` and `E` formatting specifiers for floats have been added to
      print them in exponential notation.
    * std: The `Times` trait has been removed
    * std: Indications of variance and opting out of builtin bounds is done
      through marker types in `std::kinds::marker` now
    * std: `hash` has been rewritten, `IterBytes` has been removed, and
      `#[deriving(Hash)]` is now possible.
    * std: `SharedChan` has been removed, `Sender` is now cloneable.
    * std: `Chan` and `Port` were renamed to `Sender` and `Receiver`.
    * std: `Chan::new` is now `channel()`.
    * std: A new synchronous channel type has been implemented.
    * std: A `select!` macro is now provided for selecting over `Receiver`s.
    * std: `hashmap` and `trie` have been moved to `libcollections`
    * std: `run` has been rolled into `io::process`
    * std: `assert_eq!` now uses `{}` instead of `{:?}`
    * std: The equality and comparison traits have seen some reorganization.
    * std: `rand` has moved to `librand`.
    * std: `to_{lower,upper}case` has been implemented for `char`.
    * std: Logging has been moved to `liblog`.
    * collections: `HashMap` has been rewritten for higher performance and less
      memory usage.
    * native: The default runtime is now `libnative`. If `libgreen` is desired,
      it can be booted manually. The runtime guide has more information and
      examples.
    * native: All I/O functionality except signals has been implemented.
    * green: Task spawning with `libgreen` has been optimized with stack caching
      and various trimming of code.
    * green: Tasks spawned by `libgreen` now have an unmapped guard page.
    * sync: The `extra::sync` module has been updated to modern crablang (and moved
      to the `sync` library), tweaking and improving various interfaces while
      dropping redundant functionality.
    * sync: A new `Barrier` type has been added to the `sync` library.
    * sync: An efficient mutex for native and green tasks has been implemented.
    * serialize: The `base64` module has seen some improvement. It treats
      newlines better, has non-string error values, and has seen general
      cleanup.
    * fourcc: A `fourcc!` macro was introduced
    * hexfloat: A `hexfloat!` macro was implemented for specifying floats via a
      hexadecimal literal.

  * Tooling
    * `crablangpkg` has been deprecated and removed from the main repository. Its
      replacement, `cargo`, is under development.
    * Nightly builds of crablang are now available
    * The memory usage of crablangc has been improved many times throughout this
      release cycle.
    * The build process supports disabling rpath support for the crablangc binary
      itself.
    * Code generation has improved in some cases, giving more information to the
      LLVM optimization passes to enable more extensive optimizations.
    * Debuginfo compatibility with lldb on OSX has been restored.
    * The master branch is now gated on an android bot, making building for
      android much more reliable.
    * Output flags have been centralized into one `--emit` flag.
    * Crate type flags have been centralized into one `--crate-type` flag.
    * Codegen flags have been consolidated behind a `-C` flag.
    * Linking against outdated crates now has improved error messages.
    * Error messages with lifetimes will often suggest how to annotate the
      function to fix the error.
    * Many more types are documented in the standard library, and new guides
      were written.
    * Many `crablangdoc` improvements:
      * code blocks are syntax highlighted.
      * render standalone markdown files.
      * the --test flag tests all code blocks by default.
      * exported macros are displayed.
      * re-exported types have their documentation inlined at the location of the
        first re-export.
      * search works across crates that have been rendered to the same output
        directory.


Version 0.9 (2014-01-09)
==========================

   * ~1800 changes, numerous bugfixes

   * Language
      * The `float` type has been removed. Use `f32` or `f64` instead.
      * A new facility for enabling experimental features (feature gating) has
        been added, using the crate-level `#[feature(foo)]` attribute.
      * Managed boxes (@) are now behind a feature gate
        (`#[feature(managed_boxes)]`) in preparation for future removal. Use the
        standard library's `Gc` or `Rc` types instead.
      * `@mut` has been removed. Use `std::cell::{Cell, RefCell}` instead.
      * Jumping back to the top of a loop is now done with `continue` instead of
        `loop`.
      * Strings can no longer be mutated through index assignment.
      * Raw strings can be created via the basic `r"foo"` syntax or with matched
        hash delimiters, as in `r###"foo"###`.
      * `~fn` is now written `proc (args) -> retval { ... }` and may only be
        called once.
      * The `&fn` type is now written `|args| -> ret` to match the literal form.
      * `@fn`s have been removed.
      * `do` only works with procs in order to make it obvious what the cost
        of `do` is.
      * Single-element tuple-like structs can no longer be dereferenced to
        obtain the inner value. A more comprehensive solution for overloading
        the dereference operator will be provided in the future.
      * The `#[link(...)]` attribute has been replaced with
        `#[crate_id = "name#vers"]`.
      * Empty `impl`s must be terminated with empty braces and may not be
        terminated with a semicolon.
      * Keywords are no longer allowed as lifetime names; the `self` lifetime
        no longer has any special meaning.
      * The old `fmt!` string formatting macro has been removed.
      * `printf!` and `printfln!` (old-style formatting) removed in favor of
        `print!` and `println!`.
      * `mut` works in patterns now, as in `let (mut x, y) = (1, 2);`.
      * The `extern mod foo (name = "bar")` syntax has been removed. Use
        `extern mod foo = "bar"` instead.
      * New reserved keywords: `alignof`, `offsetof`, `sizeof`.
      * Macros can have attributes.
      * Macros can expand to items with attributes.
      * Macros can expand to multiple items.
      * The `asm!` macro is feature-gated (`#[feature(asm)]`).
      * Comments may be nested.
      * Values automatically coerce to trait objects they implement, without
        an explicit `as`.
      * Enum discriminants are no longer an entire word but as small as needed to
        contain all the variants. The `repr` attribute can be used to override
        the discriminant size, as in `#[repr(int)]` for integer-sized, and
        `#[repr(C)]` to match C enums.
      * Non-string literals are not allowed in attributes (they never worked).
      * The FFI now supports variadic functions.
      * Octal numeric literals, as in `0o7777`.
      * The `concat!` syntax extension performs compile-time string concatenation.
      * The `#[fixed_stack_segment]` and `#[crablang_stack]` attributes have been
        removed as CrabLang no longer uses segmented stacks.
      * Non-ascii identifiers are feature-gated (`#[feature(non_ascii_idents)]`).
      * Ignoring all fields of an enum variant or tuple-struct is done with `..`,
        not `*`; ignoring remaining fields of a struct is also done with `..`,
        not `_`; ignoring a slice of a vector is done with `..`, not `.._`.
      * `crablangc` supports the "win64" calling convention via `extern "win64"`.
      * `crablangc` supports the "system" calling convention, which defaults to the
        preferred convention for the target platform, "stdcall" on 32-bit Windows,
        "C" elsewhere.
      * The `type_overflow` lint (default: warn) checks literals for overflow.
      * The `unsafe_block` lint (default: allow) checks for usage of `unsafe`.
      * The `attribute_usage` lint (default: warn) warns about unknown
        attributes.
      * The `unknown_features` lint (default: warn) warns about unknown
        feature gates.
      * The `dead_code` lint (default: warn) checks for dead code.
      * CrabLang libraries can be linked statically to one another
      * `#[link_args]` is behind the `link_args` feature gate.
      * Native libraries are now linked with `#[link(name = "foo")]`
      * Native libraries can be statically linked to a crablang crate
        (`#[link(name = "foo", kind = "static")]`).
      * Native OS X frameworks are now officially supported
        (`#[link(name = "foo", kind = "framework")]`).
      * The `#[thread_local]` attribute creates thread-local (not task-local)
        variables. Currently behind the `thread_local` feature gate.
      * The `return` keyword may be used in closures.
      * Types that can be copied via a memcpy implement the `Pod` kind.
      * The `cfg` attribute can now be used on struct fields and enum variants.

   * Libraries
      * std: The `option` and `result` API's have been overhauled to make them
        simpler, more consistent, and more composable.
      * std: The entire `std::io` module has been replaced with one that is
        more comprehensive and that properly interfaces with the underlying
        scheduler. File, TCP, UDP, Unix sockets, pipes, and timers are all
        implemented.
      * std: `io::util` contains a number of useful implementations of
        `Reader` and `Writer`, including `NullReader`, `NullWriter`,
        `ZeroReader`, `TeeReader`.
      * std: The reference counted pointer type `extra::rc` moved into std.
      * std: The `Gc` type in the `gc` module will replace `@` (it is currently
        just a wrapper around it).
      * std: The `Either` type has been removed.
      * std: `fmt::Default` can be implemented for any type to provide default
        formatting to the `format!` macro, as in `format!("{}", myfoo)`.
      * std: The `rand` API continues to be tweaked.
      * std: The `crablang_begin_unwind` function, useful for inserting breakpoints
        on failure in gdb, is now named `crablang_fail`.
      * std: The `each_key` and `each_value` methods on `HashMap` have been
        replaced by the `keys` and `values` iterators.
      * std: Functions dealing with type size and alignment have moved from the
        `sys` module to the `mem` module.
      * std: The `path` module was written and API changed.
      * std: `str::from_utf8` has been changed to cast instead of allocate.
      * std: `starts_with` and `ends_with` methods added to vectors via the
        `ImmutableEqVector` trait, which is in the prelude.
      * std: Vectors can be indexed with the `get_opt` method, which returns `None`
        if the index is out of bounds.
      * std: Task failure no longer propagates between tasks, as the model was
        complex, expensive, and incompatible with thread-based tasks.
      * std: The `Any` type can be used for dynamic typing.
      * std: `~Any` can be passed to the `fail!` macro and retrieved via
        `task::try`.
      * std: Methods that produce iterators generally do not have an `_iter`
        suffix now.
      * std: `cell::Cell` and `cell::RefCell` can be used to introduce mutability
        roots (mutable fields, etc.). Use instead of e.g. `@mut`.
      * std: `util::ignore` renamed to `prelude::drop`.
      * std: Slices have `sort` and `sort_by` methods via the `MutableVector`
        trait.
      * std: `vec::raw` has seen a lot of cleanup and API changes.
      * std: The standard library no longer includes any C++ code, and very
        minimal C, eliminating the dependency on libstdc++.
      * std: Runtime scheduling and I/O functionality has been factored out into
        extensible interfaces and is now implemented by two different crates:
        libnative, for native threading and I/O; and libgreen, for green threading
        and I/O. This paves the way for using the standard library in more limited
        embedded environments.
      * std: The `comm` module has been rewritten to be much faster, have a
        simpler, more consistent API, and to work for both native and green
        threading.
      * std: All libuv dependencies have been moved into the crablanguv crate.
      * native: New implementations of runtime scheduling on top of OS threads.
      * native: New native implementations of TCP, UDP, file I/O, process spawning,
        and other I/O.
      * green: The green thread scheduler and message passing types are almost
        entirely lock-free.
      * extra: The `flatpipes` module had bitrotted and was removed.
      * extra: All crypto functions have been removed and CrabLang now has a policy of
        not reimplementing crypto in the standard library. In the future crypto
        will be provided by external crates with bindings to established libraries.
      * extra: `c_vec` has been modernized.
      * extra: The `sort` module has been removed. Use the `sort` method on
        mutable slices.

   * Tooling
      * The `crablang` and `crablangi` commands have been removed, due to lack of
        maintenance.
      * `crablangdoc` was completely rewritten.
      * `crablangdoc` can test code examples in documentation.
      * `crablangpkg` can test packages with the argument, 'test'.
      * `crablangpkg` supports arbitrary dependencies, including C libraries.
      * `crablangc`'s support for generating debug info is improved again.
      * `crablangc` has better error reporting for unbalanced delimiters.
      * `crablangc`'s JIT support was removed due to bitrot.
      * Executables and static libraries can be built with LTO (-Z lto)
      * `crablangc` adds a `--dep-info` flag for communicating dependencies to
        build tools.


Version 0.8 (2013-09-26)
============================

   * ~2200 changes, numerous bugfixes

   * Language
      * The `for` loop syntax has changed to work with the `Iterator` trait.
      * At long last, unwinding works on Windows.
      * Default methods are ready for use.
      * Many trait inheritance bugs fixed.
      * Owned and borrowed trait objects work more reliably.
      * `copy` is no longer a keyword. It has been replaced by the `Clone` trait.
      * crablangc can omit emission of code for the `debug!` macro if it is passed
        `--cfg ndebug`
      * mod.rs is now "blessed". When loading `mod foo;`, crablangc will now look
        for foo.rs, then foo/mod.rs, and will generate an error when both are
        present.
      * Strings no longer contain trailing nulls. The new `std::c_str` module
        provides new mechanisms for converting to C strings.
      * The type of foreign functions is now `extern "C" fn` instead of `*u8'.
      * The FFI has been overhauled such that foreign functions are called directly,
        instead of through a stack-switching wrapper.
      * Calling a foreign function must be done through a CrabLang function with the
        `#[fixed_stack_segment]` attribute.
      * The `externfn!` macro can be used to declare both a foreign function and
        a `#[fixed_stack_segment]` wrapper at once.
      * `pub` and `priv` modifiers on `extern` blocks are no longer parsed.
      * `unsafe` is no longer allowed on extern fns - they are all unsafe.
      * `priv` is disallowed everywhere except for struct fields and enum variants.
      * `&T` (besides `&'static T`) is no longer allowed in `@T`.
      * `ref` bindings in irrefutable patterns work correctly now.
      * `char` is now prevented from containing invalid code points.
      * Casting to `bool` is no longer allowed.
      * `\0` is now accepted as an escape in chars and strings.
      * `yield` is a reserved keyword.
      * `typeof` is a reserved keyword.
      * Crates may be imported by URL with `extern mod foo = "url";`.
      * Explicit enum discriminants may be given as uints as in `enum E { V = 0u }`
      * Static vectors can be initialized with repeating elements,
        e.g. `static foo: [u8, .. 100]: [0, .. 100];`.
      * Static structs can be initialized with functional record update,
        e.g. `static foo: Foo = Foo { a: 5, .. bar };`.
      * `cfg!` can be used to conditionally execute code based on the crate
        configuration, similarly to `#[cfg(...)]`.
      * The `unnecessary_qualification` lint detects unneeded module
        prefixes (default: allow).
      * Arithmetic operations have been implemented on the SIMD types in
        `std::unstable::simd`.
      * Exchange allocation headers were removed, reducing memory usage.
      * `format!` implements a completely new, extensible, and higher-performance
        string formatting system. It will replace `fmt!`.
      * `print!` and `println!` write formatted strings (using the `format!`
        extension) to stdout.
      * `write!` and `writeln!` write formatted strings (using the `format!`
        extension) to the new Writers in `std::rt::io`.
      * The library section in which a function or static is placed may
        be specified with `#[link_section = "..."]`.
      * The `proto!` syntax extension for defining bounded message protocols
        was removed.
      * `macro_rules!` is hygienic for `let` declarations.
      * The `#[export_name]` attribute specifies the name of a symbol.
      * `unreachable!` can be used to indicate unreachable code, and fails
        if executed.

   * Libraries
      * std: Transitioned to the new runtime, written in CrabLang.
      * std: Added an experimental I/O library, `rt::io`, based on the new
        runtime.
      * std: A new generic `range` function was added to the prelude, replacing
        `uint::range` and friends.
      * std: `range_rev` no longer exists. Since range is an iterator it can be
        reversed with `range(lo, hi).invert()`.
      * std: The `chain` method on option renamed to `and_then`; `unwrap_or_default`
        renamed to `unwrap_or`.
      * std: The `iterator` module was renamed to `iter`.
      * std: Integral types now support the `checked_add`, `checked_sub`, and
        `checked_mul` operations for detecting overflow.
      * std: Many methods in `str`, `vec`, `option, `result` were renamed for
        consistency.
      * std: Methods are standardizing on conventions for casting methods:
        `to_foo` for copying, `into_foo` for moving, `as_foo` for temporary
        and cheap casts.
      * std: The `CString` type in `c_str` provides new ways to convert to and
        from C strings.
      * std: `DoubleEndedIterator` can yield elements in two directions.
      * std: The `mut_split` method on vectors partitions an `&mut [T]` into
        two splices.
      * std: `str::from_bytes` renamed to `str::from_utf8`.
      * std: `pop_opt` and `shift_opt` methods added to vectors.
      * std: The task-local data interface no longer uses @, and keys are
        no longer function pointers.
      * std: The `swap_unwrap` method of `Option` renamed to `take_unwrap`.
      * std: Added `SharedPort` to `comm`.
      * std: `Eq` has a default method for `ne`; only `eq` is required
        in implementations.
      * std: `Ord` has default methods for `le`, `gt` and `ge`; only `lt`
        is required in implementations.
      * std: `is_utf8` performance is improved, impacting many string functions.
      * std: `os::MemoryMap` provides cross-platform mmap.
      * std: `ptr::offset` is now unsafe, but also more optimized. Offsets that
        are not 'in-bounds' are considered undefined.
      * std: Many freestanding functions in `vec` removed in favor of methods.
      * std: Many freestanding functions on scalar types removed in favor of
        methods.
      * std: Many options to task builders were removed since they don't make
        sense in the new scheduler design.
      * std: More containers implement `FromIterator` so can be created by the
        `collect` method.
      * std: More complete atomic types in `unstable::atomics`.
      * std: `comm::PortSet` removed.
      * std: Mutating methods in the `Set` and `Map` traits have been moved into
        the `MutableSet` and `MutableMap` traits. `Container::is_empty`,
        `Map::contains_key`, `MutableMap::insert`, and `MutableMap::remove` have
        default implementations.
      * std: Various `from_str` functions were removed in favor of a generic
        `from_str` which is available in the prelude.
      * std: `util::unreachable` removed in favor of the `unreachable!` macro.
      * extra: `dlist`, the doubly-linked list was modernized.
      * extra: Added a `hex` module with `ToHex` and `FromHex` traits.
      * extra: Added `glob` module, replacing `std::os::glob`.
      * extra: `rope` was removed.
      * extra: `deque` was renamed to `ringbuf`. `RingBuf` implements `Deque`.
      * extra: `net`, and `timer` were removed. The experimental replacements
        are `std::rt::io::net` and `std::rt::io::timer`.
      * extra: Iterators implemented for `SmallIntMap`.
      * extra: Iterators implemented for `Bitv` and `BitvSet`.
      * extra: `SmallIntSet` removed. Use `BitvSet`.
      * extra: Performance of JSON parsing greatly improved.
      * extra: `semver` updated to SemVer 2.0.0.
      * extra: `term` handles more terminals correctly.
      * extra: `dbg` module removed.
      * extra: `par` module removed.
      * extra: `future` was cleaned up, with some method renames.
      * extra: Most free functions in `getopts` were converted to methods.

   * Other
      * crablangc's debug info generation (`-Z debug-info`) is greatly improved.
      * crablangc accepts `--target-cpu` to compile to a specific CPU architecture,
        similarly to gcc's `--march` flag.
      * crablangc's performance compiling small crates is much better.
      * crablangpkg has received many improvements.
      * crablangpkg supports git tags as package IDs.
      * crablangpkg builds into target-specific directories so it can be used for
        cross-compiling.
      * The number of concurrent test tasks is controlled by the environment
        variable CRABLANG_TEST_TASKS.
      * The test harness can now report metrics for benchmarks.
      * All tools have man pages.
      * Programs compiled with `--test` now support the `-h` and `--help` flags.
      * The runtime uses jemalloc for allocations.
      * Segmented stacks are temporarily disabled as part of the transition to
        the new runtime. Stack overflows are possible!
      * A new documentation backend, crablangdoc_ng, is available for use. It is
        still invoked through the normal `crablangdoc` command.


Version 0.7 (2013-07-03)
=======================

   * ~2000 changes, numerous bugfixes

   * Language
      * `impl`s no longer accept a visibility qualifier. Put them on methods
        instead.
      * The borrow checker has been rewritten with flow-sensitivity, fixing
        many bugs and inconveniences.
      * The `self` parameter no longer implicitly means `&'self self`,
        and can be explicitly marked with a lifetime.
      * Overloadable compound operators (`+=`, etc.) have been temporarily
        removed due to bugs.
      * The `for` loop protocol now requires `for`-iterators to return `bool`
        so they compose better.
      * The `Durable` trait is replaced with the `'static` bounds.
      * Trait default methods work more often.
      * Structs with the `#[packed]` attribute have byte alignment and
        no padding between fields.
      * Type parameters bound by `Copy` must now be copied explicitly with
        the `copy` keyword.
      * It is now illegal to move out of a dereferenced unsafe pointer.
      * `Option<~T>` is now represented as a nullable pointer.
      * `@mut` does dynamic borrow checks correctly.
      * The `main` function is only detected at the topmost level of the crate.
        The `#[main]` attribute is still valid anywhere.
      * Struct fields may no longer be mutable. Use inherited mutability.
      * The `#[no_send]` attribute makes a type that would otherwise be
        `Send`, not.
      * The `#[no_freeze]` attribute makes a type that would otherwise be
        `Freeze`, not.
      * Unbounded recursion will abort the process after reaching the limit
        specified by the `CRABLANG_MAX_STACK` environment variable (default: 1GB).
      * The `vecs_implicitly_copyable` lint mode has been removed. Vectors
        are never implicitly copyable.
      * `#[static_assert]` makes compile-time assertions about static bools.
      * At long last, 'argument modes' no longer exist.
      * The rarely used `use mod` statement no longer exists.

   * Syntax extensions
      * `fail!` and `assert!` accept `~str`, `&'static str` or `fmt!`-style
        argument list.
      * `Encodable`, `Decodable`, `Ord`, `TotalOrd`, `TotalEq`, `DeepClone`,
        `Rand`, `Zero` and `ToStr` can all be automatically derived with
        `#[deriving(...)]`.
      * The `bytes!` macro returns a vector of bytes for string, u8, char,
        and unsuffixed integer literals.

   * Libraries
      * The `core` crate was renamed to `std`.
      * The `std` crate was renamed to `extra`.
      * More and improved documentation.
      * std: `iterator` module for external iterator objects.
      * Many old-style (internal, higher-order function) iterators replaced by
        implementations of `Iterator`.
      * std: Many old internal vector and string iterators,
        incl. `any`, `all`. removed.
      * std: The `finalize` method of `Drop` renamed to `drop`.
      * std: The `drop` method now takes `&mut self` instead of `&self`.
      * std: The prelude no longer re-exports any modules, only types and traits.
      * std: Prelude additions: `print`, `println`, `FromStr`, `ApproxEq`, `Equiv`,
        `Iterator`, `IteratorUtil`, many numeric traits, many tuple traits.
      * std: New numeric traits: `Fractional`, `Real`, `RealExt`, `Integer`, `Ratio`,
        `Algebraic`, `Trigonometric`, `Exponential`, `Primitive`.
      * std: Tuple traits and accessors defined for up to 12-tuples, e.g.
        `(0, 1, 2).n2()` or `(0, 1, 2).n2_ref()`.
      * std: Many types implement `Clone`.
      * std: `path` type renamed to `Path`.
      * std: `mut` module and `Mut` type removed.
      * std: Many standalone functions removed in favor of methods and iterators
        in `vec`, `str`. In the future methods will also work as functions.
      * std: `reinterpret_cast` removed. Use `transmute`.
      * std: ascii string handling in `std::ascii`.
      * std: `Rand` is implemented for ~/@.
      * std: `run` module for spawning processes overhauled.
      * std: Various atomic types added to `unstable::atomic`.
      * std: Various types implement `Zero`.
      * std: `LinearMap` and `LinearSet` renamed to `HashMap` and `HashSet`.
      * std: Borrowed pointer functions moved from `ptr` to `borrow`.
      * std: Added `os::mkdir_recursive`.
      * std: Added `os::glob` function performs filesystems globs.
      * std: `FuzzyEq` renamed to `ApproxEq`.
      * std: `Map` now defines `pop` and `swap` methods.
      * std: `Cell` constructors converted to static methods.
      * extra: `rc` module adds the reference counted pointers, `Rc` and `RcMut`.
      * extra: `flate` module moved from `std` to `extra`.
      * extra: `fileinput` module for iterating over a series of files.
      * extra: `Complex` number type and `complex` module.
      * extra: `Rational` number type and `rational` module.
      * extra: `BigInt`, `BigUint` implement numeric and comparison traits.
      * extra: `term` uses terminfo now, is more correct.
      * extra: `arc` functions converted to methods.
      * extra: Implementation of fixed output size variations of SHA-2.

   * Tooling
      * `unused_variables` lint mode for unused variables (default: warn).
      * `unused_unsafe` lint mode for detecting unnecessary `unsafe` blocks
        (default: warn).
      * `unused_mut` lint mode for identifying unused `mut` qualifiers
        (default: warn).
      * `dead_assignment` lint mode for unread variables (default: warn).
      * `unnecessary_allocation` lint mode detects some heap allocations that are
        immediately borrowed so could be written without allocating (default: warn).
      * `missing_doc` lint mode (default: allow).
      * `unreachable_code` lint mode (default: warn).
      * The `crablangi` command has been rewritten and a number of bugs addressed.
      * crablangc outputs in color on more terminals.
      * crablangc accepts a `--link-args` flag to pass arguments to the linker.
      * crablangc accepts a `-Z print-link-args` flag for debugging linkage.
      * Compiling with `-g` will make the binary record information about
        dynamic borrowcheck failures for debugging.
      * crablangdoc has a nicer stylesheet.
      * Various improvements to crablangdoc.
      * Improvements to crablangpkg (see the detailed release notes).


Version 0.6 (2013-04-03)
========================

   * ~2100 changes, numerous bugfixes

   * Syntax changes
      * The self type parameter in traits is now spelled `Self`
      * The `self` parameter in trait and impl methods must now be explicitly
        named (for example: `fn f(&self) { }`). Implicit self is deprecated.
      * Static methods no longer require the `static` keyword and instead
        are distinguished by the lack of a `self` parameter
      * Replaced the `Durable` trait with the `'static` lifetime
      * The old closure type syntax with the trailing sigil has been
        removed in favor of the more consistent leading sigil
      * `super` is a keyword, and may be prefixed to paths
      * Trait bounds are separated with `+` instead of whitespace
      * Traits are implemented with `impl Trait for Type`
        instead of `impl Type: Trait`
      * Lifetime syntax is now `&'l foo` instead of `&l/foo`
      * The `export` keyword has finally been removed
      * The `move` keyword has been removed (see "Semantic changes")
      * The interior mutability qualifier on vectors, `[mut T]`, has been
        removed. Use `&mut [T]`, etc.
      * `mut` is no longer valid in `~mut T`. Use inherited mutability
      * `fail` is no longer a keyword. Use `fail!()`
      * `assert` is no longer a keyword. Use `assert!()`
      * `log` is no longer a keyword. use `debug!`, etc.
      * 1-tuples may be represented as `(T,)`
      * Struct fields may no longer be `mut`. Use inherited mutability,
        `@mut T`, `core::mut` or `core::cell`
      * `extern mod { ... }` is no longer valid syntax for foreign
        function modules. Use extern blocks: `extern { ... }`
      * Newtype enums removed. Use tuple-structs.
      * Trait implementations no longer support visibility modifiers
      * Pattern matching over vectors improved and expanded
      * `const` renamed to `static` to correspond to lifetime name,
        and make room for future `static mut` unsafe mutable globals.
      * Replaced `#[deriving_eq]` with `#[deriving(Eq)]`, etc.
      * `Clone` implementations can be automatically generated with
        `#[deriving(Clone)]`
      * Casts to traits must use a pointer sigil, e.g. `@foo as @Bar`
        instead of `foo as Bar`.
      * Fixed length vector types are now written as `[int, .. 3]`
        instead of `[int * 3]`.
      * Fixed length vector types can express the length as a constant
        expression. (ex: `[int, .. GL_BUFFER_SIZE - 2]`)

   * Semantic changes
      * Types with owned pointers or custom destructors move by default,
        eliminating the `move` keyword
      * All foreign functions are considered unsafe
      * &mut is now unaliasable
      * Writes to borrowed @mut pointers are prevented dynamically
      * () has size 0
      * The name of the main function can be customized using #[main]
      * The default type of an inferred closure is &fn instead of @fn
      * `use` statements may no longer be "chained" - they cannot import
        identifiers imported by previous `use` statements
      * `use` statements are crate relative, importing from the "top"
        of the crate by default. Paths may be prefixed with `super::`
        or `self::` to change the search behavior.
      * Method visibility is inherited from the implementation declaration
      * Structural records have been removed
      * Many more types can be used in static items, including enums
        'static-lifetime pointers and vectors
      * Pattern matching over vectors improved and expanded
      * Typechecking of closure types has been overhauled to
        improve inference and eliminate unsoundness
      * Macros leave scope at the end of modules, unless that module is
        tagged with #[macro_escape]

   * Libraries
      * Added big integers to `std::bigint`
      * Removed `core::oldcomm` module
      * Added pipe-based `core::comm` module
      * Numeric traits have been reorganized under `core::num`
      * `vec::slice` finally returns a slice
      * `debug!` and friends don't require a format string, e.g. `debug!(Foo)`
      * Containers reorganized around traits in `core::container`
      * `core::dvec` removed, `~[T]` is a drop-in replacement
      * `core::send_map` renamed to `core::hashmap`
      * `std::map` removed; replaced with `core::hashmap`
      * `std::treemap` reimplemented as an owned balanced tree
      * `std::deque` and `std::smallintmap` reimplemented as owned containers
      * `core::trie` added as a fast ordered map for integer keys
      * Set types added to `core::hashmap`, `core::trie` and `std::treemap`
      * `Ord` split into `Ord` and `TotalOrd`. `Ord` is still used to
        overload the comparison operators, whereas `TotalOrd` is used
        by certain container types

   * Other
      * Replaced the 'cargo' package manager with 'crablangpkg'
      * Added all-purpose 'crablang' tool
      * `crablangc --test` now supports benchmarks with the `#[bench]` attribute
      * crablangc now *attempts* to offer spelling suggestions
      * Improved support for ARM and Android
      * Preliminary MIPS backend
      * Improved foreign function ABI implementation for x86, x86_64
      * Various memory usage improvements
      * CrabLang code may be embedded in foreign code under limited circumstances
      * Inline assembler supported by new asm!() syntax extension.


Version 0.5 (2012-12-21)
===========================

   * ~900 changes, numerous bugfixes

   * Syntax changes
      * Removed `<-` move operator
      * Completed the transition from the `#fmt` extension syntax to `fmt!`
      * Removed old fixed length vector syntax - `[T]/N`
      * New token-based quasi-quoters, `quote_tokens!`, `quote_expr!`, etc.
      * Macros may now expand to items and statements
      * `a.b()` is always parsed as a method call, never as a field projection
      * `Eq` and `IterBytes` implementations can be automatically generated
        with `#[deriving_eq]` and `#[deriving_iter_bytes]` respectively
      * Removed the special crate language for `.rc` files
      * Function arguments may consist of any irrefutable pattern

   * Semantic changes
      * `&` and `~` pointers may point to objects
      * Tuple structs - `struct Foo(Bar, Baz)`. Will replace newtype enums.
      * Enum variants may be structs
      * Destructors can be added to all nominal types with the Drop trait
      * Structs and nullary enum variants may be constants
      * Values that cannot be implicitly copied are now automatically moved
        without writing `move` explicitly
      * `&T` may now be coerced to `*T`
      * Coercions happen in `let` statements as well as function calls
      * `use` statements now take crate-relative paths
      * The module and type namespaces have been merged so that static
        method names can be resolved under the trait in which they are
        declared

   * Improved support for language features
      * Trait inheritance works in many scenarios
      * More support for explicit self arguments in methods - `self`, `&self`
        `@self`, and `~self` all generally work as expected
      * Static methods work in more situations
      * Experimental: Traits may declare default methods for the implementations
        to use

   * Libraries
      * New condition handling system in `core::condition`
      * Timsort added to `std::sort`
      * New priority queue, `std::priority_queue`
      * Pipes for serializable types, `std::flatpipes'
      * Serialization overhauled to be trait-based
      * Expanded `getopts` definitions
      * Moved futures to `std`
      * More functions are pure now
      * `core::comm` renamed to `oldcomm`. Still deprecated
      * `crablangdoc` and `cargo` are libraries now

   * Misc
      * Added a preliminary REPL, `crablangi`
      * License changed from MIT to dual MIT/APL2


Version 0.4 (2012-10-15)
==========================

   * ~2000 changes, numerous bugfixes

   * Syntax
      * All keywords are now strict and may not be used as identifiers anywhere
      * Keyword removal: 'again', 'import', 'check', 'new', 'owned', 'send',
        'of', 'with', 'to', 'class'.
      * Classes are replaced with simpler structs
      * Explicit method self types
      * `ret` became `return` and `alt` became `match`
      * `import` is now `use`; `use is now `extern mod`
      * `extern mod { ... }` is now `extern { ... }`
      * `use mod` is the recommended way to import modules
      * `pub` and `priv` replace deprecated export lists
      * The syntax of `match` pattern arms now uses fat arrow (=>)
      * `main` no longer accepts an args vector; use `os::args` instead

   * Semantics
      * Trait implementations are now coherent, ala Haskell typeclasses
      * Trait methods may be static
      * Argument modes are deprecated
      * Borrowed pointers are much more mature and recommended for use
      * Strings and vectors in the static region are stored in constant memory
      * Typestate was removed
      * Resolution rewritten to be more reliable
      * Support for 'dual-mode' data structures (freezing and thawing)

   * Libraries
      * Most binary operators can now be overloaded via the traits in
        `core::ops'
      * `std::net::url` for representing URLs
      * Sendable hash maps in `core::send_map`
      * `core::task' gained a (currently unsafe) task-local storage API

   * Concurrency
      * An efficient new intertask communication primitive called the pipe,
        along with a number of higher-level channel types, in `core::pipes`
      * `std::arc`, an atomically reference counted, immutable, shared memory
        type
      * `std::sync`, various exotic synchronization tools based on arcs and pipes
      * Futures are now based on pipes and sendable
      * More robust linked task failure
      * Improved task builder API

   * Other
      * Improved error reporting
      * Preliminary JIT support
      * Preliminary work on precise GC
      * Extensive architectural improvements to crablangc
      * Begun a transition away from buggy C++-based reflection (shape) code to
        CrabLang-based (visitor) code
      * All hash functions and tables converted to secure, randomized SipHash


Version 0.3  (2012-07-12)
========================

   * ~1900 changes, numerous bugfixes

   * New coding conveniences
      * Integer-literal suffix inference
      * Per-item control over warnings, errors
      * #[cfg(windows)] and #[cfg(unix)] attributes
      * Documentation comments
      * More compact closure syntax
      * 'do' expressions for treating higher-order functions as
        control structures
      * *-patterns (wildcard extended to all constructor fields)

   * Semantic cleanup
      * Name resolution pass and exhaustiveness checker rewritten
      * Region pointers and borrow checking supersede alias
        analysis
      * Init-ness checking is now provided by a region-based liveness
        pass instead of the typestate pass; same for last-use analysis
      * Extensive work on region pointers

   * Experimental new language features
      * Slices and fixed-size, interior-allocated vectors
      * #!-comments for lang versioning, shell execution
      * Destructors and iface implementation for classes;
        type-parameterized classes and class methods
      * 'const' type kind for types that can be used to implement
        shared-memory concurrency patterns

   * Type reflection

   * Removal of various obsolete features
      * Keywords: 'be', 'prove', 'syntax', 'note', 'mutable', 'bind',
                 'ccrablang', 'native' (now 'extern'), 'cont' (now 'again')

      * Constructs: do-while loops ('do' repurposed), fn binding,
                    resources (replaced by destructors)

   * Compiler reorganization
      * Syntax-layer of compiler split into separate crate
      * Clang (from LLVM project) integrated into build
      * Typechecker split into sub-modules

   * New library code
      * New time functions
      * Extension methods for many built-in types
      * Arc: atomic-refcount read-only / exclusive-use shared cells
      * Par: parallel map and search routines
      * Extensive work on libuv interface
      * Much vector code moved to libraries
      * Syntax extensions: #line, #col, #file, #mod, #stringify,
        #include, #include_str, #include_bin

   * Tool improvements
      * Cargo automatically resolves dependencies


Version 0.2  (2012-03-29)
=========================

   * >1500 changes, numerous bugfixes

   * New docs and doc tooling

   * New port: FreeBSD x86_64

   * Compilation model enhancements
      * Generics now specialized, multiply instantiated
      * Functions now inlined across separate crates

   * Scheduling, stack and threading fixes
      * Noticeably improved message-passing performance
      * Explicit schedulers
      * Callbacks from C
      * Helgrind clean

   * Experimental new language features
      * Operator overloading
      * Region pointers
      * Classes

   * Various language extensions
      * C-callback function types: 'ccrablang fn ...'
      * Infinite-loop construct: 'loop { ... }'
      * Shorten 'mutable' to 'mut'
      * Required mutable-local qualifier: 'let mut ...'
      * Basic glob-exporting: 'export foo::*;'
      * Alt now exhaustive, 'alt check' for runtime-checked
      * Block-function form of 'for' loop, with 'break' and 'ret'.

   * New library code
      * AST quasi-quote syntax extension
      * Revived libuv interface
      * New modules: core::{future, iter}, std::arena
      * Merged per-platform std::{os*, fs*} to core::{libc, os}
      * Extensive cleanup, regularization in libstd, libcore


Version 0.1  (2012-01-20)
===============================

   * Most language features work, including:
      * Unique pointers, unique closures, move semantics
      * Interface-constrained generics
      * Static interface dispatch
      * Stack growth
      * Multithread task scheduling
      * Typestate predicates
      * Failure unwinding, destructors
      * Pattern matching and destructuring assignment
      * Lightweight block-lambda syntax
      * Preliminary macro-by-example

   * Compiler works with the following configurations:
      * Linux: x86 and x86_64 hosts and targets
      * macOS: x86 and x86_64 hosts and targets
      * Windows: x86 hosts and targets

   * Cross compilation / multi-target configuration supported.

   * Preliminary API-documentation and package-management tools included.

Known issues:

   * Documentation is incomplete.

   * Performance is below intended target.

   * Standard library APIs are subject to extensive change, reorganization.

   * Language-level versioning is not yet operational - future code will
     break unexpectedly.
