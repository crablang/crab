# Default Cargo feature resolver

## Summary

- `edition = "2021"` implies `resolver = "2"` in `Cargo.toml`.

## Details

Since Rust 1.51.0, Cargo has opt-in support for a [new feature resolver][4]
which can be activated with `resolver = "2"` in `Cargo.toml`.

Starting in Rust 2021, this will be the default.
That is, writing `edition = "2021"` in `Cargo.toml` will imply `resolver = "2"`.

The resolver is a global setting for a [workspace], and the setting is ignored in dependencies.
The setting is only honored for the top-level package of the workspace.
If you are using a [virtual workspace], you will still need to explicitly set the [`resolver` field]
in the `[workspace]` definition if you want to opt-in to the new resolver.

The new feature resolver no longer merges all requested features for
crates that are depended on in multiple ways.
See [the announcement of Rust 1.51][5] for details.

[4]: ../../cargo/reference/resolver.html#feature-resolver-version-2
[5]: https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html#cargos-new-feature-resolver
[workspace]: ../../cargo/reference/workspaces.html
[virtual workspace]: ../../cargo/reference/workspaces.html#virtual-workspace
[`resolver` field]: ../../cargo/reference/resolver.html#resolver-versions

## Migration

There are no automated migration tools for updating for the new resolver.
For most projects, there are usually few or no changes as a result of updating.

When updating with `cargo fix --edition`, Cargo will display a report if the new resolver will build dependencies with different features.
It may look something like this:

> note: Switching to Edition 2021 will enable the use of the version 2 feature resolver in Cargo.
> This may cause some dependencies to be built with fewer features enabled than previously.
> More information about the resolver changes may be found at <https://doc.rust-lang.org/nightly/edition-guide/rust-2021/default-cargo-resolver.html><br>
> When building the following dependencies, the given features will no longer be used:
>
> ```text
>   bstr v0.2.16: default, lazy_static, regex-automata, unicode
>   libz-sys v1.1.3 (as host dependency): libc
> ```

This lets you know that certain dependencies will no longer be built with the given features.

### Build failures

There may be some circumstances where your project may not build correctly after the change.
If a dependency declaration in one package assumes that certain features are enabled in another, and those features are now disabled, it may fail to compile.

For example, let's say we have a dependency like this:

```toml
# Cargo.toml

[dependencies]
bstr = { version = "0.2.16", default-features = false }
# ...
```

And somewhere in our dependency tree, another package has this:

```toml
# Another package's Cargo.toml

[build-dependencies]
bstr = "0.2.16"
```

In our package, we've been using the [`words_with_breaks`](https://docs.rs/bstr/0.2.16/bstr/trait.ByteSlice.html#method.words_with_breaks) method from `bstr`, which requires `bstr`'s  "unicode" feature to be enabled.
This has historically worked because Cargo unified the features of `bstr` between the two packages.
However, after updating to Rust 2021, the new resolver will build `bstr` twice, once with the default features (as a build dependency), and once with no features (as our normal dependency).
Since `bstr` is now being built without the "unicode" feature, the `words_with_breaks` method doesn't exist, and the build will fail with an error that the method is missing.

The solution here is to ensure that the dependency is declared with the features you are actually using.
For example:

```toml
[dependencies]
bstr = { version = "0.2.16", default-features = false, features = ["unicode"] }
```

In some cases, this may be a problem with a third-party dependency that you don't have direct control over.
You can consider submitting a patch to that project to try to declare the correct set of features for the problematic dependency.
Alternatively, you can add features to any dependency from within your own `Cargo.toml` file.
For example, if the `bstr` example given above was declared in some third-party dependency, you can just copy the correct dependency declaration into your own project.
The features will be unified, as long as they match the unification rules of the new resolver. Those are:

* Features enabled on platform-specific dependencies for targets not currently being built are ignored.
* Build-dependencies and proc-macros do not share features with normal dependencies.
* Dev-dependencies do not activate features unless building a target that needs them (like tests or examples).

A real-world example is using [`diesel`](https://crates.io/crates/diesel) and [`diesel_migrations`](https://crates.io/crates/diesel_migrations).
These packages provide database support, and the database is selected using a feature, like this:

```toml
[dependencies]
diesel = { version = "1.4.7", features = ["postgres"] }
diesel_migrations = "1.4.0"
```

The problem is that `diesel_migrations` has an internal proc-macro which itself depends on `diesel`, and the proc-macro assumes its own copy of `diesel` has the same features enabled as the rest of the dependency graph.
After updating to the new resolver, it fails to build because now there are two copies of `diesel`, and the one built for the proc-macro is missing the "postgres" feature.

A solution here is to add `diesel` as a build-dependency with the required features, for example:

```toml
[build-dependencies]
diesel = { version = "1.4.7", features = ["postgres"] }
```

This causes Cargo to add "postgres" as a feature for host dependencies (proc-macros and build-dependencies).
Now, the `diesel_migrations` proc-macro will get the "postgres" feature enabled, and it will build correctly.

The 2.0 release of `diesel` (currently in development) does not have this problem as it has been restructured to not have this dependency requirement.

### Exploring features

The [`cargo tree`] command has had substantial improvements to help with the migration to the new resolver.
`cargo tree` can be used to explore the dependency graph, and to see which features are being enabled, and importantly *why* they are being enabled.

One option is to use the `--duplicates` flag (`-d` for short), which will tell you when a package is being built multiple times.
Taking the `bstr` example from earlier, we might see:

```console
> cargo tree -d
bstr v0.2.16
└── foo v0.1.0 (/MyProjects/foo)

bstr v0.2.16
[build-dependencies]
└── bar v0.1.0
    └── foo v0.1.0 (/MyProjects/foo)

```

This output tells us that `bstr` is built twice, and shows the chain of dependencies that led to its inclusion in both cases.

You can print which features each package is using with the `-f` flag, like this:

```console
cargo tree -f '{p} {f}'
```

This tells Cargo to change the "format" of the output, where it will print both the package and the enabled features.

You can also use the `-e` flag to tell it which "edges" to display.
For example, `cargo tree -e features` will show in-between each dependency which features are being added by each dependency.
This option becomes more useful with the `-i` flag which can be used to "invert" the tree.
This allows you to see how features *flow* into a given dependency.
For example, let's say the dependency graph is large, and we're not quite sure who is depending on `bstr`, the following command will show that:

```console
> cargo tree -e features -i bstr
bstr v0.2.16
├── bstr feature "default"
│   [build-dependencies]
│   └── bar v0.1.0
│       └── bar feature "default"
│           └── foo v0.1.0 (/MyProjects/foo)
├── bstr feature "lazy_static"
│   └── bstr feature "unicode"
│       └── bstr feature "default" (*)
├── bstr feature "regex-automata"
│   └── bstr feature "unicode" (*)
├── bstr feature "std"
│   └── bstr feature "default" (*)
└── bstr feature "unicode" (*)
```

This snippet of output shows that the project `foo` depends on `bar` with the "default" feature.
Then, `bar` depends on `bstr` as a build-dependency with the "default" feature.
We can further see that `bstr`'s  "default" feature enables "unicode" (among other features).

[`cargo tree`]: ../../cargo/commands/cargo-tree.html
