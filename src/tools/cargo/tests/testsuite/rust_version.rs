//! Tests for targets with `rust-version`.

use cargo_test_support::{project, registry::Package};

#[cargo_test]
fn rust_version_satisfied() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "1.1.1"
            [[bin]]
            name = "foo"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("check").run();
    p.cargo("check --ignore-rust-version").run();
}

#[cargo_test]
fn rust_version_bad_caret() {
    project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "^1.43"
            [[bin]]
            name = "foo"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build()
        .cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] unexpected version requirement, expected a version like \"1.32\"
 --> Cargo.toml:6:28
  |
6 |             rust-version = \"^1.43\"
  |                            ^^^^^^^
  |
",
        )
        .run();
}

#[cargo_test]
fn rust_version_good_pre_release() {
    project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "1.43.0-beta.1"
            [[bin]]
            name = "foo"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build()
        .cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] unexpected prerelease field, expected a version like \"1.32\"
 --> Cargo.toml:6:28
  |
6 |             rust-version = \"1.43.0-beta.1\"
  |                            ^^^^^^^^^^^^^^^
  |
",
        )
        .run();
}

#[cargo_test]
fn rust_version_bad_pre_release() {
    project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "1.43-beta.1"
            [[bin]]
            name = "foo"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build()
        .cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] unexpected prerelease field, expected a version like \"1.32\"
 --> Cargo.toml:6:28
  |
6 |             rust-version = \"1.43-beta.1\"
  |                            ^^^^^^^^^^^^^
  |
",
        )
        .run();
}

#[cargo_test]
fn rust_version_bad_nonsense() {
    project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "foodaddle"
            [[bin]]
            name = "foo"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build()
        .cargo("check")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] expected a version like \"1.32\"
 --> Cargo.toml:6:28
  |
6 |             rust-version = \"foodaddle\"
  |                            ^^^^^^^^^^^
  |
",
        )
        .run();
}

#[cargo_test]
fn rust_version_too_high() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "1.9876.0"
            [[bin]]
            name = "foo"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "error: package `foo v0.0.1 ([..])` cannot be built because it requires \
             rustc 1.9876.0 or newer, while the currently active rustc version is [..]\n\n",
        )
        .run();
    p.cargo("check --ignore-rust-version").run();
}

#[cargo_test]
fn dependency_rust_version_newer_than_rustc() {
    Package::new("bar", "0.0.1")
        .rust_version("1.2345.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            [dependencies]
            bar = "0.0.1"
        "#,
        )
        .file("src/main.rs", "fn main(){}")
        .build();

    p.cargo("check")
        .with_status(101)
        .with_stderr(
            "    Updating `[..]` index\n \
             Downloading crates ...\n  \
             Downloaded bar v0.0.1 (registry `[..]`)\n\
             error: package `bar v0.0.1` cannot be built because it requires \
             rustc 1.2345.0 or newer, while the currently active rustc version is [..]\n\
             Either upgrade to rustc 1.2345.0 or newer, or use\n\
             cargo update bar@0.0.1 --precise ver\n\
             where `ver` is the latest version of `bar` supporting rustc [..]",
        )
        .run();
    p.cargo("check --ignore-rust-version").run();
}

#[cargo_test]
fn dependency_rust_version_newer_than_package() {
    Package::new("bar", "1.6.0")
        .rust_version("1.65.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "1.60.0"
            [dependencies]
            bar = "1.0.0"
        "#,
        )
        .file("src/main.rs", "fn main(){}")
        .build();

    p.cargo("check")
        .arg("-Zmsrv-policy")
        .masquerade_as_nightly_cargo(&["msrv-policy"])
        .run();
    p.cargo("check --ignore-rust-version")
        .arg("-Zmsrv-policy")
        .masquerade_as_nightly_cargo(&["msrv-policy"])
        .run();
}

#[cargo_test]
fn dependency_rust_version_older_and_newer_than_package() {
    Package::new("bar", "1.5.0")
        .rust_version("1.55.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .publish();
    Package::new("bar", "1.6.0")
        .rust_version("1.65.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "1.60.0"
            [dependencies]
            bar = "1.0.0"
        "#,
        )
        .file("src/main.rs", "fn main(){}")
        .build();

    p.cargo("check --ignore-rust-version")
        .arg("-Zmsrv-policy")
        .masquerade_as_nightly_cargo(&["msrv-policy"])
        // This should pick 1.6.0
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v1.5.0 (registry `dummy-registry`)
[CHECKING] bar v1.5.0
[CHECKING] [..]
[FINISHED] [..]
",
        )
        .run();
    p.cargo("check")
        .arg("-Zmsrv-policy")
        .masquerade_as_nightly_cargo(&["msrv-policy"])
        .with_stderr(
            "\
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn dependency_rust_version_backtracking() {
    Package::new("has-rust-version", "1.6.0")
        .rust_version("1.65.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .publish();
    Package::new("no-rust-version", "2.1.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .publish();
    Package::new("no-rust-version", "2.2.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .dep("has-rust-version", "1.6.0")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "1.60.0"
            [dependencies]
            no-rust-version = "2"
        "#,
        )
        .file("src/main.rs", "fn main(){}")
        .build();

    p.cargo("check --ignore-rust-version")
        .arg("-Zmsrv-policy")
        .masquerade_as_nightly_cargo(&["msrv-policy"])
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] no-rust-version v2.2.0 (registry `dummy-registry`)
[DOWNLOADED] has-rust-version v1.6.0 (registry `dummy-registry`)
[CHECKING] has-rust-version v1.6.0
[CHECKING] no-rust-version v2.2.0
[CHECKING] [..]
[FINISHED] [..]
",
        )
        .run();
    p.cargo("check")
        .arg("-Zmsrv-policy")
        .masquerade_as_nightly_cargo(&["msrv-policy"])
        .with_stderr(
            "\
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn workspace_with_mixed_rust_version() {
    Package::new("bar", "1.4.0")
        .rust_version("1.45.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .publish();
    Package::new("bar", "1.5.0")
        .rust_version("1.55.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .publish();
    Package::new("bar", "1.6.0")
        .rust_version("1.65.0")
        .file("src/lib.rs", "fn other_stuff() {}")
        .publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["lower"]

            [package]
            name = "higher"
            version = "0.0.1"
            authors = []
            rust-version = "1.60.0"
            [dependencies]
            bar = "1.0.0"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "lower/Cargo.toml",
            r#"
            [package]
            name = "lower"
            version = "0.0.1"
            authors = []
            rust-version = "1.50.0"
            [dependencies]
            bar = "1.0.0"
        "#,
        )
        .file("lower/src/main.rs", "fn main() {}")
        .build();

    p.cargo("check --ignore-rust-version")
        .arg("-Zmsrv-policy")
        .masquerade_as_nightly_cargo(&["msrv-policy"])
        // This should pick 1.6.0
        .with_stderr(
            "\
[UPDATING] `dummy-registry` index
[DOWNLOADING] crates ...
[DOWNLOADED] bar v1.4.0 (registry `dummy-registry`)
[CHECKING] bar v1.4.0
[CHECKING] [..]
[FINISHED] [..]
",
        )
        .run();
    p.cargo("check")
        .arg("-Zmsrv-policy")
        .masquerade_as_nightly_cargo(&["msrv-policy"])
        .with_stderr(
            "\
[FINISHED] [..]
",
        )
        .run();
}

#[cargo_test]
fn rust_version_older_than_edition() {
    project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []
            rust-version = "1.1"
            edition = "2018"
            [[bin]]
            name = "foo"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build()
        .cargo("check")
        .with_status(101)
        .with_stderr_contains("  rust-version 1.1 is older than first version (1.31.0) required by the specified edition (2018)",
        )
        .run();
}
