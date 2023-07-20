# Extern crate declarations

> **<sup>Syntax:<sup>**\
> _ExternCrate_ :\
> &nbsp;&nbsp; `extern` `crate` _CrateRef_ _AsClause_<sup>?</sup> `;`
>
> _CrateRef_ :\
> &nbsp;&nbsp; [IDENTIFIER] | `self`
>
> _AsClause_ :\
> &nbsp;&nbsp; `as` ( [IDENTIFIER] | `_` )

An _`extern crate` declaration_ specifies a dependency on an external crate.
The external crate is then bound into the declaring scope as the [identifier]
provided in the `extern crate` declaration. Additionally, if the `extern
crate` appears in the crate root, then the crate name is also added to the
[extern prelude], making it automatically in scope in all modules. The `as`
clause can be used to bind the imported crate to a different name.

The external crate is resolved to a specific `soname` at compile time, and a
runtime linkage requirement to that `soname` is passed to the linker for
loading at runtime. The `soname` is resolved at compile time by scanning the
compiler's library path and matching the optional `crate_name` provided against
the [`crate_name` attributes] that were declared on the external crate when it was
compiled. If no `crate_name` is provided, a default `name` attribute is assumed,
equal to the [identifier] given in the `extern crate` declaration.

The `self` crate may be imported which creates a binding to the current crate.
In this case the `as` clause must be used to specify the name to bind it to.

Three examples of `extern crate` declarations:

<!-- ignore: requires external crates -->
```rust,ignore
extern crate pcre;

extern crate std; // equivalent to: extern crate std as std;

extern crate std as ruststd; // linking to 'std' under another name
```

When naming Rust crates, hyphens are disallowed. However, Cargo packages may
make use of them. In such case, when `Cargo.toml` doesn't specify a crate name,
Cargo will transparently replace `-` with `_` (Refer to [RFC 940] for more
details).

Here is an example:

<!-- ignore: requires external crates -->
```rust,ignore
// Importing the Cargo package hello-world
extern crate hello_world; // hyphen replaced with an underscore
```

## Extern Prelude

This section has been moved to [Preludes — Extern Prelude](../names/preludes.md#extern-prelude).
<!-- this is to appease the linkchecker, will remove once other books are updated -->

## Underscore Imports

An external crate dependency can be declared without binding its name in scope
by using an underscore with the form `extern crate foo as _`. This may be
useful for crates that only need to be linked, but are never referenced, and
will avoid being reported as unused.

The [`macro_use` attribute] works as usual and imports the macro names
into the [`macro_use` prelude].

## The `no_link` attribute

The *`no_link` attribute* may be specified on an `extern crate` item to
prevent linking the crate into the output. This is commonly used to load a
crate to access only its macros.

[IDENTIFIER]: ../identifiers.md
[RFC 940]: https://github.com/rust-lang/rfcs/blob/master/text/0940-hyphens-considered-harmful.md
[`macro_use` attribute]: ../macros-by-example.md#the-macro_use-attribute
[extern prelude]: ../names/preludes.md#extern-prelude
[`macro_use` prelude]: ../names/preludes.md#macro_use-prelude
[`crate_name` attributes]: ../crates-and-source-files.md#the-crate_name-attribute

<script>
(function() {
    var fragments = {
        "#extern-prelude": "../names/preludes.html#extern-prelude",
    };
    var target = fragments[window.location.hash];
    if (target) {
        var url = window.location.toString();
        var base = url.substring(0, url.lastIndexOf('/'));
        window.location.replace(base + "/" + target);
    }
})();
</script>
