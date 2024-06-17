# Cargo: Reject unused inherited default-features

🚧 The 2024 Edition has not yet been released and hence this section is still "under construction".

## Summary

- `default-features = false` is no longer allowed in an inherited workspace dependency if the workspace dependency specifies `default-features = true` (or does not specify `default-features`).

## Details

[Workspace inheritance] allows you to specify dependencies in one place (the workspace), and then to refer to those workspace dependencies from within a package.
There was an inadvertent interaction with how `default-features` is specified that is no longer allowed in the 2024 Edition.

Unless the workspace specifies `default-features = false`, it is no longer allowed to specify `default-features = false` in an inherited package dependency.
For example, with a workspace that specifies:

```toml
[workspace.dependencies]
regex = "1.10.4"
```

The following is now an error:

```toml
[package]
name = "foo"
version = "1.0.0"
edition = "2024"

[dependencies]
regex = { workspace = true, default-features = false }  # ERROR
```

The reason for this change is to avoid confusion when specifying `default-features = false` when the default feature is already enabled, since it has no effect.

If you want the flexibility of deciding whether or not a dependency enables the default-features of a dependency, be sure to set `default-features = false` in the workspace definition.
Just beware that if you build multiple workspace members at the same time, the features will be unified so that if one member sets `default-features = true` (which is the default if not explicitly set), the default-features will be enabled for all members using that dependency.

## Migration

When using `cargo fix --edition`, Cargo will automatically update your `Cargo.toml` file to remove `default-features = false` in this situation.

If you would prefer to update your `Cargo.toml` manually, check for any warnings when running a build and remove the corresponding entries.
Previous editions should display something like:

```text
warning: /home/project/Cargo.toml: `default-features` is ignored for regex,
since `default-features` was not specified for `workspace.dependencies.regex`,
this could become a hard error in the future
```

[workspace inheritance]: ../../cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace
