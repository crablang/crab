# Cargo: Table and key name consistency

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".

## Summary

- Several table and key names in `Cargo.toml` have been removed where there were previously two ways to specify the same thing.
    - Removed `[project]`; use `[package]` instead.
    - Removed `default_features`; use `default-features` instead.
    - Removed `crate_type`; use `crate-type` instead.
    - Removed `proc_macro`; use `proc-macro` instead.
    - Removed `dev_dependencies`; use `dev-dependencies` instead.
    - Removed `build_dependencies`; use `build-dependencies` instead.

## Details

Several table and keys names are no longer allowed in the 2024 Edition.
There were two ways to specify these tables or keys, and this helps ensure there is only one way to specify them.

Some were due to a change in decisions over time, and some were inadvertent implementation artifacts.
In order to avoid confusion, and to enforce a single style for specifying these tables and keys, only one variant is now allowed.

For example:

```toml
[dev_dependencies]
rand = { version = "0.8.5", default_features = false }
```

Should be changed to:

```toml
[dev-dependencies]
rand = { version = "0.8.5", default-features = false }
```

Notice that the underscores were changed to dashes for `dev_dependencies` and `default_features`.

## Migration

When using `cargo fix --edition`, Cargo will automatically update your `Cargo.toml` file to use the preferred table and key names.

If you would prefer to update your `Cargo.toml` manually, be sure to go through the list above and make sure only the new forms are used.
