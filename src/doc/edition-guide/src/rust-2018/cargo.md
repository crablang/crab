# Cargo changes

## Summary

- If there is a target definition in a `Cargo.toml` manifest, it no longer
  automatically disables automatic discovery of other targets.
- Target paths of the form `src/{target_name}.rs` are no longer inferred for
  targets where the `path` field is not set.
- `cargo install` for the current directory is no longer allowed, you must
  specify `cargo install --path .` to install the current package.