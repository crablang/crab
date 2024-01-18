# C-string literals

## Summary

- Literals of the form `c"foo"` or `cr"foo"` represent a string of type [`&core::ffi::CStr`][CStr].

[CStr]: ../../core/ffi/struct.CStr.html

## Details

Starting with Rust 1.76, C-strings can be written using C-string literal syntax with the `c` or `cr` prefix.

Previously, it was challenging to properly produce a valid string literal that could interoperate with C APIs which terminate with a NUL byte.
The [`cstr`] crate was a popular solution, but that required compiling a proc-macro which was quite expensive.
Now, C-strings can be written directly using literal syntax notation, which will generate a value of type [`&core::ffi::CStr`][CStr] which is automatically terminated with a NUL byte.

```rust,edition2021
# use core::ffi::CStr;

assert_eq!(c"hello", CStr::from_bytes_with_nul(b"hello\0").unwrap());
assert_eq!(
    c"byte escapes \xff work",
    CStr::from_bytes_with_nul(b"byte escapes \xff work\0").unwrap()
);
assert_eq!(
    c"unicode escapes \u{00E6} work",
    CStr::from_bytes_with_nul(b"unicode escapes \xc3\xa6 work\0").unwrap()
);
assert_eq!(
    c"unicode characters αβγ encoded as UTF-8",
    CStr::from_bytes_with_nul(
        b"unicode characters \xce\xb1\xce\xb2\xce\xb3 encoded as UTF-8\0"
    )
    .unwrap()
);
assert_eq!(
    c"strings can continue \
        on multiple lines",
    CStr::from_bytes_with_nul(b"strings can continue on multiple lines\0").unwrap()
);
```

C-strings do not allow interior NUL bytes (such as with a `\0` escape).

Similar to regular strings, C-strings also support "raw" syntax with the `cr` prefix.
These raw C-strings do not process backslash escapes which can make it easier to write strings that contain backslashes.
Double-quotes can be included by surrounding the quotes with the `#` character.
Multiple `#` characters can be used to avoid ambiguity with internal `"#` sequences.

```rust,edition2021
assert_eq!(cr"foo", c"foo");
// Number signs can be used to embed interior double quotes.
assert_eq!(cr#""foo""#, c"\"foo\"");
// This requires two #.
assert_eq!(cr##""foo"#"##, c"\"foo\"#");
// Escapes are not processed.
assert_eq!(cr"C:\foo", c"C:\\foo");
```

See [The Reference] for more details.

[`cstr`]: https://crates.io/crates/cstr
[The Reference]: ../../reference/tokens.html#c-string-and-raw-c-string-literals

## Migration

Migration is only necessary for macros which may have been assuming a sequence of tokens that looks similar to `c"…"` or `cr"…"`, which previous to the 2021 edition would tokenize as two separate tokens, but in 2021 appears as a single token.

As part of the [syntax reservation] for the 2021 edition, any macro input which may run into this issue should issue a warning from the `rust_2021_prefixes_incompatible_syntax` migration lint.
See that chapter for more detail.

[syntax reservation]: reserving-syntax.md
