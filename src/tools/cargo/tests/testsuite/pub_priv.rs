//! Tests for public/private dependencies.

use cargo_test_support::project;
use cargo_test_support::registry::{Dependency, Package};

#[cargo_test(nightly, reason = "exported_private_dependencies lint is unstable")]
fn exported_priv_warning() {
    Package::new("priv_dep", "0.1.0")
        .file("src/lib.rs", "pub struct FromPriv;")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]

                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies]
                priv_dep = "0.1.0"
            "#,
        )
        .file(
            "src/lib.rs",
            "
            extern crate priv_dep;
            pub fn use_priv(_: priv_dep::FromPriv) {}
        ",
        )
        .build();

    p.cargo("check --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_stderr_contains(
            "\
src/lib.rs:3:13: warning: type `[..]FromPriv` from private dependency 'priv_dep' in public interface
",
        )
        .run()
}

#[cargo_test(nightly, reason = "exported_private_dependencies lint is unstable")]
fn exported_pub_dep() {
    Package::new("pub_dep", "0.1.0")
        .file("src/lib.rs", "pub struct FromPub;")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]

                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies]
                pub_dep = {version = "0.1.0", public = true}
            "#,
        )
        .file(
            "src/lib.rs",
            "
            extern crate pub_dep;
            pub fn use_pub(_: pub_dep::FromPub) {}
        ",
        )
        .build();

    p.cargo("check --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] pub_dep v0.1.0 ([..])
[CHECKING] pub_dep v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run()
}

#[cargo_test]
pub fn requires_nightly_cargo() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("check --message-format=short")
        .with_status(101)
        .with_stderr(
            "\
error: failed to parse manifest at `[..]`

Caused by:
  the cargo feature `public-dependency` requires a nightly version of Cargo, but this is the `stable` channel
  See https://doc.rust-lang.org/book/appendix-07-nightly-rust.html for more information about Rust release channels.
  See https://doc.rust-lang.org/[..]cargo/reference/unstable.html#public-dependency for more information about using this feature.
"
        )
        .run()
}

#[cargo_test]
fn requires_feature() {
    Package::new("pub_dep", "0.1.0")
        .file("src/lib.rs", "")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies]
                pub_dep = { version = "0.1.0", public = true }
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("check --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_status(101)
        .with_stderr(
            "\
error: failed to parse manifest at `[..]`

Caused by:
  feature `public-dependency` is required

  The package requires the Cargo feature called `public-dependency`, \
  but that feature is not stabilized in this version of Cargo (1.[..]).
  Consider adding `cargo-features = [\"public-dependency\"]` to the top of Cargo.toml \
  (above the [package] table) to tell Cargo you are opting in to use this unstable feature.
  See https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#public-dependency \
  for more information about the status of this feature.
",
        )
        .run()
}

#[cargo_test]
fn pub_dev_dependency() {
    Package::new("pub_dep", "0.1.0")
        .file("src/lib.rs", "pub struct FromPub;")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]

                [package]
                name = "foo"
                version = "0.0.1"

                [dev-dependencies]
                pub_dep = {version = "0.1.0", public = true}
            "#,
        )
        .file(
            "src/lib.rs",
            "
            extern crate pub_dep;
            pub fn use_pub(_: pub_dep::FromPub) {}
        ",
        )
        .build();

    p.cargo("check --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_status(101)
        .with_stderr(
            "\
error: failed to parse manifest at `[..]`

Caused by:
  'public' specifier can only be used on regular dependencies, not Development dependencies
",
        )
        .run()
}

#[cargo_test(nightly, reason = "exported_private_dependencies lint is unstable")]
fn workspace_pub_disallowed() {
    Package::new("foo1", "0.1.0")
        .file("src/lib.rs", "pub struct FromFoo;")
        .publish();
    Package::new("foo2", "0.1.0")
        .file("src/lib.rs", "pub struct FromFoo;")
        .publish();
    Package::new("foo3", "0.1.0")
        .file("src/lib.rs", "pub struct FromFoo;")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]

                [package]
                name = "foo"
                version = "0.0.1"

                [workspace.dependencies]
                foo1 = "0.1.0"
                foo2 = { version = "0.1.0", public = true }
                foo3 = { version = "0.1.0", public = false }

                [dependencies]
                foo1 = { workspace = true, public = true }
                foo2 = { workspace = true }
                foo3 = { workspace = true, public = true }
            "#,
        )
        .file(
            "src/lib.rs",
            "
                #![deny(exported_private_dependencies)]
                pub fn use_priv1(_: foo1::FromFoo) {}
                pub fn use_priv2(_: foo2::FromFoo) {}
                pub fn use_priv3(_: foo3::FromFoo) {}
            ",
        )
        .build();

    p.cargo("check")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_status(101)
        .with_stderr(
            "\
error: failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  foo2 is public, but workspace dependencies cannot be public
",
        )
        .run()
}

#[cargo_test(nightly, reason = "exported_private_dependencies lint is unstable")]
fn allow_priv_in_tests() {
    Package::new("priv_dep", "0.1.0")
        .file("src/lib.rs", "pub struct FromPriv;")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]

                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies]
                priv_dep = {version = "0.1.0", public = false}
            "#,
        )
        .file(
            "tests/mod.rs",
            "
            extern crate priv_dep;
            pub fn use_priv(_: priv_dep::FromPriv) {}
        ",
        )
        .build();

    p.cargo("check --tests --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] priv_dep v0.1.0 ([..])
[CHECKING] priv_dep v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run()
}

#[cargo_test(nightly, reason = "exported_private_dependencies lint is unstable")]
fn allow_priv_in_benchs() {
    Package::new("priv_dep", "0.1.0")
        .file("src/lib.rs", "pub struct FromPriv;")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]

                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies]
                priv_dep = {version = "0.1.0", public = false}
            "#,
        )
        .file(
            "benches/mod.rs",
            "
            extern crate priv_dep;
            pub fn use_priv(_: priv_dep::FromPriv) {}
        ",
        )
        .build();

    p.cargo("check --benches --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] priv_dep v0.1.0 ([..])
[CHECKING] priv_dep v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run()
}

#[cargo_test(nightly, reason = "exported_private_dependencies lint is unstable")]
fn allow_priv_in_bins() {
    Package::new("priv_dep", "0.1.0")
        .file("src/lib.rs", "pub struct FromPriv;")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]

                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies]
                priv_dep = {version = "0.1.0", public = false}
            "#,
        )
        .file(
            "src/main.rs",
            "
            extern crate priv_dep;
            pub fn use_priv(_: priv_dep::FromPriv) {}
            fn main() {}
        ",
        )
        .build();

    p.cargo("check --bins --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] priv_dep v0.1.0 ([..])
[CHECKING] priv_dep v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run()
}

#[cargo_test(nightly, reason = "exported_private_dependencies lint is unstable")]
fn allow_priv_in_examples() {
    Package::new("priv_dep", "0.1.0")
        .file("src/lib.rs", "pub struct FromPriv;")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]

                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies]
                priv_dep = {version = "0.1.0", public = false}
            "#,
        )
        .file(
            "examples/lib.rs",
            "
            extern crate priv_dep;
            pub fn use_priv(_: priv_dep::FromPriv) {}
            fn main() {}
        ",
        )
        .build();

    p.cargo("check --examples --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] priv_dep v0.1.0 ([..])
[CHECKING] priv_dep v0.1.0
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run()
}

#[cargo_test(nightly, reason = "exported_private_dependencies lint is unstable")]
fn allow_priv_in_custom_build() {
    Package::new("priv_dep", "0.1.0")
        .file("src/lib.rs", "pub struct FromPriv;")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                cargo-features = ["public-dependency"]

                [package]
                name = "foo"
                version = "0.0.1"

                [build-dependencies]
                priv_dep = "0.1.0"
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "build.rs",
            "
            extern crate priv_dep;
            pub fn use_priv(_: priv_dep::FromPriv) {}
            fn main() {}
        ",
        )
        .build();

    p.cargo("check --all-targets --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] priv_dep v0.1.0 ([..])
[COMPILING] priv_dep v0.1.0
[COMPILING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run()
}

#[cargo_test(nightly, reason = "exported_private_dependencies lint is unstable")]
fn publish_package_with_public_dependency() {
    Package::new("pub_bar", "0.1.0")
        .file("src/lib.rs", "pub struct FromPub;")
        .publish();
    Package::new("bar", "0.1.0")
        .cargo_feature("public-dependency")
        .add_dep(Dependency::new("pub_bar", "0.1.0").public(true))
        .file(
            "src/lib.rs",
            "
            extern crate pub_bar;
            pub use pub_bar::FromPub as BarFromPub;
        ",
        )
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            cargo-features = ["public-dependency"]
            [package]
            name = "foo"
            version = "0.0.1"
            [dependencies]
            bar = {version = "0.1.0", public = true}
        "#,
        )
        .file(
            "src/lib.rs",
            "
            extern crate bar;
            pub fn use_pub(_: bar::BarFromPub) {}
        ",
        )
        .build();

    p.cargo("check --message-format=short")
        .masquerade_as_nightly_cargo(&["public-dependency"])
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] pub_bar v0.1.0 ([..])
[DOWNLOADED] bar v0.1.0 ([..])
[CHECKING] pub_bar v0.1.0
[CHECKING] bar v0.1.0
[CHECKING] foo v0.0.1 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run()
}
