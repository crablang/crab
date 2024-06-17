# Cargo: Remove implicit features

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".

## Summary

- Optional dependencies must now be explicitly specified in the `[features]` table.

## Details

In previous editions, when an [optional dependency] is specified, Cargo would automatically add an implicit [feature] of the same name as the dependency. For example:

```toml
[dependencies]
jpeg-decoder = { version = "0.3.1", optional = true }
```

This would automatically add a feature `jpeg-decoder = ["dep:jpeg-decoder"]` to provide a way to enable the dependency.
The `dep:` entries are specific syntax for referring to optional dependencies.
This implicit feature is only added if `"dep:jpeg-decoder"` is not specified in any other feature.

In the 2024 Edition, this implicit feature is no longer added, and you are required to explicitly specify the dependency in the `[features]` table.
For example, instead of exposing the particular internal name of some dependency, you may consider using a more general term for the feature name:

```toml
[features]
graphics = ["dep:jpeg-decoder"]
```

`cargo add --optional <NAME>` automatically adds a feature for the dependency to the `[features]` table if it isn't already there.

### Motivation

One reason for requiring this to be explicit is that it encourages a conscious decision about the public exposure of the feature name, and makes it clearer when reading the `[features]` table which features exist.
This can help avoid tying the implementation details (the dependency names) to the public set of feature names.

Also, removing features is a [SemVer incompatible change][semver], which may not be obvious when removing an optional dependency that you thought was private.

## Migration

When using `cargo fix --edition`, Cargo will automatically update your `Cargo.toml` file to include the implicit features if necessary.

If you would prefer to update your `Cargo.toml` manually, add a `foo = ["dep:foo"]` entry for each optional dependency named *foo* if `dep:foo` is not already specified anywhere in the `[features]` table.

[optional dependency]: ../../cargo/reference/features.html#optional-dependencies
[feature]: ../../cargo/reference/features.html
[semver]: ../../cargo/reference/semver.html#cargo-feature-remove
