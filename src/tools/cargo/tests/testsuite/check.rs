//! Tests for the `cargo check` command.

use std::fmt::{self, Write};

use crate::messages::raw_rustc_output;
use cargo_test_support::install::exe;
use cargo_test_support::paths::CargoPathExt;
use cargo_test_support::registry::Package;
use cargo_test_support::{basic_bin_manifest, basic_manifest, git, project};
use cargo_test_support::{tools, wrapped_clippy_driver};

#[cargo_test]
fn check_success() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies.bar]
                path = "../bar"
            "#,
        )
        .file(
            "src/main.rs",
            "extern crate bar; fn main() { ::bar::baz(); }",
        )
        .build();
    let _bar = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "pub fn baz() {}")
        .build();

    foo.cargo("check").run();
}

#[cargo_test]
fn check_fail() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies.bar]
                path = "../bar"
            "#,
        )
        .file(
            "src/main.rs",
            "extern crate bar; fn main() { ::bar::baz(42); }",
        )
        .build();
    let _bar = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "pub fn baz() {}")
        .build();

    foo.cargo("check")
        .with_status(101)
        .with_stderr_contains("[..]this function takes 0[..]")
        .run();
}

#[cargo_test]
fn custom_derive() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies.bar]
                path = "../bar"
            "#,
        )
        .file(
            "src/main.rs",
            r#"
            #[macro_use]
            extern crate bar;

            trait B {
                fn b(&self);
            }

            #[derive(B)]
            struct A;

            fn main() {
                let a = A;
                a.b();
            }
            "#,
        )
        .build();
    let _bar = project()
        .at("bar")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.1.0"
                authors = []
                [lib]
                proc-macro = true
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
            extern crate proc_macro;

            use proc_macro::TokenStream;

            #[proc_macro_derive(B)]
            pub fn derive(_input: TokenStream) -> TokenStream {
                format!("impl B for A {{ fn b(&self) {{}} }}").parse().unwrap()
            }
            "#,
        )
        .build();

    foo.cargo("check").run();
}

#[cargo_test]
fn check_build() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies.bar]
                path = "../bar"
            "#,
        )
        .file(
            "src/main.rs",
            "extern crate bar; fn main() { ::bar::baz(); }",
        )
        .build();

    let _bar = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "pub fn baz() {}")
        .build();

    foo.cargo("check").run();
    foo.cargo("build").run();
}

#[cargo_test]
fn build_check() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies.bar]
                path = "../bar"
            "#,
        )
        .file(
            "src/main.rs",
            "extern crate bar; fn main() { ::bar::baz(); }",
        )
        .build();

    let _bar = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "pub fn baz() {}")
        .build();

    foo.cargo("build -v").run();
    foo.cargo("check -v").run();
}

// Checks that where a project has both a lib and a bin, the lib is only checked
// not built.
#[cargo_test]
fn issue_3418() {
    let foo = project()
        .file("src/lib.rs", "")
        .file("src/main.rs", "fn main() {}")
        .build();

    foo.cargo("check -v")
        .with_stderr_contains("[..] --emit=[..]metadata [..]")
        .run();
}

// Some weirdness that seems to be caused by a crate being built as well as
// checked, but in this case with a proc macro too.
#[cargo_test]
fn issue_3419() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies]
                rustc-serialize = "*"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                extern crate rustc_serialize;

                use rustc_serialize::Decodable;

                pub fn take<T: Decodable>() {}
            "#,
        )
        .file(
            "src/main.rs",
            r#"
                extern crate rustc_serialize;

                extern crate foo;

                #[derive(RustcDecodable)]
                pub struct Foo;

                fn main() {
                    foo::take::<Foo>();
                }
            "#,
        )
        .build();

    Package::new("rustc-serialize", "1.0.0")
        .file(
            "src/lib.rs",
            r#"
                pub trait Decodable: Sized {
                    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error>;
                }
                pub trait Decoder {
                   type Error;
                   fn read_struct<T, F>(&mut self, s_name: &str, len: usize, f: F)
                                        -> Result<T, Self::Error>
                   where F: FnOnce(&mut Self) -> Result<T, Self::Error>;
                }
            "#,
        )
        .publish();

    p.cargo("check").run();
}

// Check on a dylib should have a different metadata hash than build.
#[cargo_test]
fn dylib_check_preserves_build_cache() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [lib]
                crate-type = ["dylib"]

                [dependencies]
            "#,
        )
        .file("src/lib.rs", "")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[..]Compiling foo v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("check").run();

    p.cargo("build")
        .with_stderr("[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]")
        .run();
}

// test `cargo rustc --profile check`
#[cargo_test]
fn rustc_check() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies.bar]
                path = "../bar"
            "#,
        )
        .file(
            "src/main.rs",
            "extern crate bar; fn main() { ::bar::baz(); }",
        )
        .build();
    let _bar = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "pub fn baz() {}")
        .build();

    foo.cargo("rustc --profile check -- --emit=metadata").run();

    // Verify compatible usage of --profile with --release, issue #7488
    foo.cargo("rustc --profile check --release -- --emit=metadata")
        .run();
    foo.cargo("rustc --profile test --release -- --emit=metadata")
        .run();
}

#[cargo_test]
fn rustc_check_err() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dependencies.bar]
                path = "../bar"
            "#,
        )
        .file(
            "src/main.rs",
            "extern crate bar; fn main() { ::bar::qux(); }",
        )
        .build();
    let _bar = project()
        .at("bar")
        .file("Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("src/lib.rs", "pub fn baz() {}")
        .build();

    foo.cargo("rustc --profile check -- --emit=metadata")
        .with_status(101)
        .with_stderr_contains("[CHECKING] bar [..]")
        .with_stderr_contains("[CHECKING] foo [..]")
        .with_stderr_contains("[..]cannot find function `qux` in [..] `bar`")
        .run();
}

#[cargo_test]
fn check_all() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [workspace]
                [dependencies]
                b = { path = "b" }
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("examples/a.rs", "fn main() {}")
        .file("tests/a.rs", "")
        .file("src/lib.rs", "")
        .file("b/Cargo.toml", &basic_manifest("b", "0.0.1"))
        .file("b/src/main.rs", "fn main() {}")
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("check --workspace -v")
        .with_stderr_contains("[..] --crate-name foo src/lib.rs [..]")
        .with_stderr_contains("[..] --crate-name foo src/main.rs [..]")
        .with_stderr_contains("[..] --crate-name b b/src/lib.rs [..]")
        .with_stderr_contains("[..] --crate-name b b/src/main.rs [..]")
        .run();
}

#[cargo_test]
fn check_all_exclude() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() { break_the_build(); }")
        .build();

    p.cargo("check --workspace --exclude baz")
        .with_stderr_does_not_contain("[CHECKING] baz v0.1.0 [..]")
        .with_stderr(
            "\
[CHECKING] bar v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn check_all_exclude_glob() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() { break_the_build(); }")
        .build();

    p.cargo("check --workspace --exclude '*z'")
        .with_stderr_does_not_contain("[CHECKING] baz v0.1.0 [..]")
        .with_stderr(
            "\
[CHECKING] bar v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn check_virtual_all_implied() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    p.cargo("check -v")
        .with_stderr_contains("[..] --crate-name bar bar/src/lib.rs [..]")
        .with_stderr_contains("[..] --crate-name baz baz/src/lib.rs [..]")
        .run();
}

#[cargo_test]
fn check_virtual_manifest_one_project() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() { break_the_build(); }")
        .build();

    p.cargo("check -p bar")
        .with_stderr_does_not_contain("[CHECKING] baz v0.1.0 [..]")
        .with_stderr(
            "\
[CHECKING] bar v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn check_virtual_manifest_glob() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {  break_the_build(); }")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    p.cargo("check -p '*z'")
        .with_stderr_does_not_contain("[CHECKING] bar v0.1.0 [..]")
        .with_stderr(
            "\
[CHECKING] baz v0.1.0 ([..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn exclude_warns_on_non_existing_package() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("check --workspace --exclude bar")
        .with_stdout("")
        .with_stderr(
            "\
[WARNING] excluded package(s) `bar` not found in workspace `[CWD]`
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn targets_selected_default() {
    let foo = project()
        .file("src/main.rs", "fn main() {}")
        .file("src/lib.rs", "pub fn smth() {}")
        .file("examples/example1.rs", "fn main() {}")
        .file("tests/test2.rs", "#[test] fn t() {}")
        .file("benches/bench3.rs", "")
        .build();

    foo.cargo("check -v")
        .with_stderr_contains("[..] --crate-name foo src/lib.rs [..]")
        .with_stderr_contains("[..] --crate-name foo src/main.rs [..]")
        .with_stderr_does_not_contain("[..] --crate-name example1 examples/example1.rs [..]")
        .with_stderr_does_not_contain("[..] --crate-name test2 tests/test2.rs [..]")
        .with_stderr_does_not_contain("[..] --crate-name bench3 benches/bench3.rs [..]")
        .run();
}

#[cargo_test]
fn targets_selected_all() {
    let foo = project()
        .file("src/main.rs", "fn main() {}")
        .file("src/lib.rs", "pub fn smth() {}")
        .file("examples/example1.rs", "fn main() {}")
        .file("tests/test2.rs", "#[test] fn t() {}")
        .file("benches/bench3.rs", "")
        .build();

    foo.cargo("check --all-targets -v")
        .with_stderr_contains("[..] --crate-name foo src/lib.rs [..]")
        .with_stderr_contains("[..] --crate-name foo src/main.rs [..]")
        .with_stderr_contains("[..] --crate-name example1 examples/example1.rs [..]")
        .with_stderr_contains("[..] --crate-name test2 tests/test2.rs [..]")
        .with_stderr_contains("[..] --crate-name bench3 benches/bench3.rs [..]")
        .run();
}

#[cargo_test]
fn check_unit_test_profile() {
    let foo = project()
        .file(
            "src/lib.rs",
            r#"
                #[cfg(test)]
                mod tests {
                    #[test]
                    fn it_works() {
                        badtext
                    }
                }
            "#,
        )
        .build();

    foo.cargo("check").run();
    foo.cargo("check --profile test")
        .with_status(101)
        .with_stderr_contains("[..]badtext[..]")
        .run();
}

// Verify what is checked with various command-line filters.
#[cargo_test]
fn check_filters() {
    let p = project()
        .file(
            "src/lib.rs",
            r#"
                fn unused_normal_lib() {}
                #[cfg(test)]
                mod tests {
                    fn unused_unit_lib() {}
                }
            "#,
        )
        .file(
            "src/main.rs",
            r#"
                fn main() {}
                fn unused_normal_bin() {}
                #[cfg(test)]
                mod tests {
                    fn unused_unit_bin() {}
                }
            "#,
        )
        .file(
            "tests/t1.rs",
            r#"
                fn unused_normal_t1() {}
                #[cfg(test)]
                mod tests {
                    fn unused_unit_t1() {}
                }
            "#,
        )
        .file(
            "examples/ex1.rs",
            r#"
                fn main() {}
                fn unused_normal_ex1() {}
                #[cfg(test)]
                mod tests {
                    fn unused_unit_ex1() {}
                }
            "#,
        )
        .file(
            "benches/b1.rs",
            r#"
                fn unused_normal_b1() {}
                #[cfg(test)]
                mod tests {
                    fn unused_unit_b1() {}
                }
            "#,
        )
        .build();

    p.cargo("check")
        .with_stderr_contains("[..]unused_normal_lib[..]")
        .with_stderr_contains("[..]unused_normal_bin[..]")
        .with_stderr_does_not_contain("[..]unused_normal_t1[..]")
        .with_stderr_does_not_contain("[..]unused_normal_ex1[..]")
        .with_stderr_does_not_contain("[..]unused_normal_b1[..]")
        .with_stderr_does_not_contain("[..]unused_unit_[..]")
        .run();
    p.root().join("target").rm_rf();
    p.cargo("check --tests -v")
        .with_stderr_contains("[..] --crate-name foo src/lib.rs [..] --test [..]")
        .with_stderr_contains("[..] --crate-name foo src/lib.rs [..] --crate-type lib [..]")
        .with_stderr_contains("[..] --crate-name foo src/main.rs [..] --test [..]")
        .with_stderr_contains("[..]unused_unit_lib[..]")
        .with_stderr_contains("[..]unused_unit_bin[..]")
        .with_stderr_contains("[..]unused_normal_lib[..]")
        .with_stderr_contains("[..]unused_normal_bin[..]")
        .with_stderr_contains("[..]unused_unit_t1[..]")
        .with_stderr_does_not_contain("[..]unused_normal_ex1[..]")
        .with_stderr_does_not_contain("[..]unused_unit_ex1[..]")
        .with_stderr_does_not_contain("[..]unused_normal_b1[..]")
        .with_stderr_does_not_contain("[..]unused_unit_b1[..]")
        .with_stderr_does_not_contain("[..]--crate-type bin[..]")
        .run();
    p.root().join("target").rm_rf();
    p.cargo("check --test t1 -v")
        .with_stderr_contains("[..]unused_normal_lib[..]")
        .with_stderr_contains("[..]unused_unit_t1[..]")
        .with_stderr_does_not_contain("[..]unused_unit_lib[..]")
        .with_stderr_does_not_contain("[..]unused_normal_bin[..]")
        .with_stderr_does_not_contain("[..]unused_unit_bin[..]")
        .with_stderr_does_not_contain("[..]unused_normal_ex1[..]")
        .with_stderr_does_not_contain("[..]unused_normal_b1[..]")
        .with_stderr_does_not_contain("[..]unused_unit_ex1[..]")
        .with_stderr_does_not_contain("[..]unused_unit_b1[..]")
        .run();
    p.root().join("target").rm_rf();
    p.cargo("check --all-targets -v")
        .with_stderr_contains("[..]unused_normal_lib[..]")
        .with_stderr_contains("[..]unused_normal_bin[..]")
        .with_stderr_contains("[..]unused_normal_t1[..]")
        .with_stderr_contains("[..]unused_normal_ex1[..]")
        .with_stderr_contains("[..]unused_normal_b1[..]")
        .with_stderr_contains("[..]unused_unit_b1[..]")
        .with_stderr_contains("[..]unused_unit_t1[..]")
        .with_stderr_contains("[..]unused_unit_lib[..]")
        .with_stderr_contains("[..]unused_unit_bin[..]")
        .with_stderr_does_not_contain("[..]unused_unit_ex1[..]")
        .run();
}

#[cargo_test]
fn check_artifacts() {
    // Verify which artifacts are created when running check (#4059).
    let p = project()
        .file("src/lib.rs", "")
        .file("src/main.rs", "fn main() {}")
        .file("tests/t1.rs", "")
        .file("examples/ex1.rs", "fn main() {}")
        .file("benches/b1.rs", "")
        .build();

    p.cargo("check").run();
    assert!(!p.root().join("target/debug/libfoo.rmeta").is_file());
    assert!(!p.root().join("target/debug/libfoo.rlib").is_file());
    assert!(!p.root().join("target/debug").join(exe("foo")).is_file());
    assert_eq!(p.glob("target/debug/deps/libfoo-*.rmeta").count(), 2);

    p.root().join("target").rm_rf();
    p.cargo("check --lib").run();
    assert!(!p.root().join("target/debug/libfoo.rmeta").is_file());
    assert!(!p.root().join("target/debug/libfoo.rlib").is_file());
    assert!(!p.root().join("target/debug").join(exe("foo")).is_file());
    assert_eq!(p.glob("target/debug/deps/libfoo-*.rmeta").count(), 1);

    p.root().join("target").rm_rf();
    p.cargo("check --bin foo").run();
    assert!(!p.root().join("target/debug/libfoo.rmeta").is_file());
    assert!(!p.root().join("target/debug/libfoo.rlib").is_file());
    assert!(!p.root().join("target/debug").join(exe("foo")).is_file());
    assert_eq!(p.glob("target/debug/deps/libfoo-*.rmeta").count(), 2);

    p.root().join("target").rm_rf();
    p.cargo("check --test t1").run();
    assert!(!p.root().join("target/debug/libfoo.rmeta").is_file());
    assert!(!p.root().join("target/debug/libfoo.rlib").is_file());
    assert!(!p.root().join("target/debug").join(exe("foo")).is_file());
    assert_eq!(p.glob("target/debug/t1-*").count(), 0);
    assert_eq!(p.glob("target/debug/deps/libfoo-*.rmeta").count(), 1);
    assert_eq!(p.glob("target/debug/deps/libt1-*.rmeta").count(), 1);

    p.root().join("target").rm_rf();
    p.cargo("check --example ex1").run();
    assert!(!p.root().join("target/debug/libfoo.rmeta").is_file());
    assert!(!p.root().join("target/debug/libfoo.rlib").is_file());
    assert!(!p
        .root()
        .join("target/debug/examples")
        .join(exe("ex1"))
        .is_file());
    assert_eq!(p.glob("target/debug/deps/libfoo-*.rmeta").count(), 1);
    assert_eq!(p.glob("target/debug/examples/libex1-*.rmeta").count(), 1);

    p.root().join("target").rm_rf();
    p.cargo("check --bench b1").run();
    assert!(!p.root().join("target/debug/libfoo.rmeta").is_file());
    assert!(!p.root().join("target/debug/libfoo.rlib").is_file());
    assert!(!p.root().join("target/debug").join(exe("foo")).is_file());
    assert_eq!(p.glob("target/debug/b1-*").count(), 0);
    assert_eq!(p.glob("target/debug/deps/libfoo-*.rmeta").count(), 1);
    assert_eq!(p.glob("target/debug/deps/libb1-*.rmeta").count(), 1);
}

#[cargo_test]
fn short_message_format() {
    let foo = project()
        .file("src/lib.rs", "fn foo() { let _x: bool = 'a'; }")
        .build();
    foo.cargo("check --message-format=short")
        .with_status(101)
        .with_stderr_contains(
            "\
src/lib.rs:1:27: error[E0308]: mismatched types
error: could not compile `foo` (lib) due to previous error
",
        )
        .run();
}

#[cargo_test]
fn proc_macro() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "demo"
                version = "0.0.1"

                [lib]
                proc-macro = true
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                extern crate proc_macro;

                use proc_macro::TokenStream;

                #[proc_macro_derive(Foo)]
                pub fn demo(_input: TokenStream) -> TokenStream {
                    "".parse().unwrap()
                }
            "#,
        )
        .file(
            "src/main.rs",
            r#"
                #[macro_use]
                extern crate demo;

                #[derive(Foo)]
                struct A;

                fn main() {}
            "#,
        )
        .build();
    p.cargo("check -v").env("CARGO_LOG", "cargo=trace").run();
}

#[cargo_test]
fn check_keep_going() {
    let foo = project()
        .file("src/bin/one.rs", "compile_error!(\"ONE\"); fn main() {}")
        .file("src/bin/two.rs", "compile_error!(\"TWO\"); fn main() {}")
        .build();

    // Due to -j1, without --keep-going only one of the two bins would be built.
    foo.cargo("check -j1 --keep-going -Zunstable-options")
        .masquerade_as_nightly_cargo(&["keep-going"])
        .with_status(101)
        .with_stderr_contains("error: ONE")
        .with_stderr_contains("error: TWO")
        .run();
}

#[cargo_test]
fn does_not_use_empty_rustc_wrapper() {
    // An empty RUSTC_WRAPPER environment variable won't be used.
    // The env var will also override the config, essentially unsetting it.
    let p = project()
        .file("src/lib.rs", "")
        .file(
            ".cargo/config.toml",
            r#"
                [build]
                rustc-wrapper = "do-not-execute-me"
            "#,
        )
        .build();
    p.cargo("check").env("RUSTC_WRAPPER", "").run();
}

#[cargo_test]
fn does_not_use_empty_rustc_workspace_wrapper() {
    let p = project().file("src/lib.rs", "").build();
    p.cargo("check").env("RUSTC_WORKSPACE_WRAPPER", "").run();
}

#[cargo_test]
fn error_from_deep_recursion() -> Result<(), fmt::Error> {
    let mut big_macro = String::new();
    writeln!(big_macro, "macro_rules! m {{")?;
    for i in 0..130 {
        writeln!(big_macro, "({}) => {{ m!({}); }};", i, i + 1)?;
    }
    writeln!(big_macro, "}}")?;
    writeln!(big_macro, "m!(0);")?;

    let p = project().file("src/lib.rs", &big_macro).build();
    p.cargo("check --message-format=json")
        .with_status(101)
        .with_stdout_contains(
            "[..]\"message\":\"recursion limit reached while expanding [..]`m[..]`\"[..]",
        )
        .run();

    Ok(())
}

#[cargo_test]
fn rustc_workspace_wrapper_affects_all_workspace_members() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    p.cargo("check")
        .env("RUSTC_WORKSPACE_WRAPPER", tools::echo_wrapper())
        .with_stderr_contains("WRAPPER CALLED: rustc --crate-name bar [..]")
        .with_stderr_contains("WRAPPER CALLED: rustc --crate-name baz [..]")
        .run();
}

#[cargo_test]
fn rustc_workspace_wrapper_includes_path_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [workspace]
                members = ["bar"]

                [dependencies]
                baz = { path = "baz" }
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    p.cargo("check --workspace")
        .env("RUSTC_WORKSPACE_WRAPPER", tools::echo_wrapper())
        .with_stderr_contains("WRAPPER CALLED: rustc --crate-name foo [..]")
        .with_stderr_contains("WRAPPER CALLED: rustc --crate-name bar [..]")
        .with_stderr_contains("WRAPPER CALLED: rustc --crate-name baz [..]")
        .run();
}

#[cargo_test]
fn rustc_workspace_wrapper_respects_primary_units() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["bar", "baz"]
            "#,
        )
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .file("baz/Cargo.toml", &basic_manifest("baz", "0.1.0"))
        .file("baz/src/lib.rs", "pub fn baz() {}")
        .build();

    p.cargo("check -p bar")
        .env("RUSTC_WORKSPACE_WRAPPER", tools::echo_wrapper())
        .with_stderr_contains("WRAPPER CALLED: rustc --crate-name bar [..]")
        .with_stdout_does_not_contain("WRAPPER CALLED: rustc --crate-name baz [..]")
        .run();
}

#[cargo_test]
fn rustc_workspace_wrapper_excludes_published_deps() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [workspace]
                members = ["bar"]

                [dependencies]
                baz = "1.0.0"
            "#,
        )
        .file("src/lib.rs", "")
        .file("bar/Cargo.toml", &basic_manifest("bar", "0.1.0"))
        .file("bar/src/lib.rs", "pub fn bar() {}")
        .build();

    Package::new("baz", "1.0.0").publish();

    p.cargo("check --workspace -v")
        .env("RUSTC_WORKSPACE_WRAPPER", tools::echo_wrapper())
        .with_stderr_contains("WRAPPER CALLED: rustc --crate-name foo [..]")
        .with_stderr_contains("WRAPPER CALLED: rustc --crate-name bar [..]")
        .with_stderr_contains("[CHECKING] baz [..]")
        .with_stdout_does_not_contain("WRAPPER CALLED: rustc --crate-name baz [..]")
        .run();
}

#[cargo_test]
fn warn_manifest_package_and_project() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"

                [project]
                name = "foo"
                version = "0.0.1"
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[WARNING] manifest at `[CWD]` contains both `project` and `package`, this could become a hard error in the future
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn git_manifest_package_and_project() {
    let p = project();
    let git_project = git::new("bar", |p| {
        p.file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.0.1"

            [project]
            name = "bar"
            version = "0.0.1"
            "#,
        )
        .file("src/lib.rs", "")
    });

    let p = p
        .file(
            "Cargo.toml",
            &format!(
                r#"
                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies.bar]
                version = "0.0.1"
                git  = '{}'

            "#,
                git_project.url()
            ),
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] git repository `[..]`
[CHECKING] bar v0.0.1 ([..])
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn warn_manifest_with_project() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [project]
                name = "foo"
                version = "0.0.1"
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[WARNING] manifest at `[CWD]` contains `[project]` instead of `[package]`, this could become a hard error in the future
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn git_manifest_with_project() {
    let p = project();
    let git_project = git::new("bar", |p| {
        p.file(
            "Cargo.toml",
            r#"
            [project]
            name = "bar"
            version = "0.0.1"
            "#,
        )
        .file("src/lib.rs", "")
    });

    let p = p
        .file(
            "Cargo.toml",
            &format!(
                r#"
                [package]
                name = "foo"
                version = "0.0.1"

                [dependencies.bar]
                version = "0.0.1"
                git  = '{}'

            "#,
                git_project.url()
            ),
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[UPDATING] git repository `[..]`
[CHECKING] bar v0.0.1 ([..])
[CHECKING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn check_fixable_warning() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
            "#,
        )
        .file("src/lib.rs", "use std::io;")
        .build();

    foo.cargo("check")
        .with_stderr_contains("[..] (run `cargo fix --lib -p foo` to apply 1 suggestion)")
        .run();
}

#[cargo_test]
fn check_fixable_test_warning() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
            "#,
        )
        .file(
            "src/lib.rs",
            "\
mod tests {
    #[test]
    fn t1() {
        use std::io;
    }
}
            ",
        )
        .build();

    foo.cargo("check --all-targets")
        .with_stderr_contains("[..] (run `cargo fix --lib -p foo --tests` to apply 1 suggestion)")
        .run();
    foo.cargo("fix --lib -p foo --tests --allow-no-vcs").run();
    assert!(!foo.read_file("src/lib.rs").contains("use std::io;"));
}

#[cargo_test]
fn check_fixable_error_no_fix() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
            "#,
        )
        .file(
            "src/lib.rs",
            "use std::io;\n#[derive(Debug(x))]\nstruct Foo;",
        )
        .build();

    let rustc_message = raw_rustc_output(&foo, "src/lib.rs", &[]);
    let expected_output = format!(
        "\
[CHECKING] foo v0.0.1 ([..])
{}\
[WARNING] `foo` (lib) generated 1 warning
[ERROR] could not compile `foo` (lib) due to previous error; 1 warning emitted
",
        rustc_message
    );
    foo.cargo("check")
        .with_status(101)
        .with_stderr(expected_output)
        .run();
}

#[cargo_test]
fn check_fixable_warning_workspace() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["foo", "bar"]
            "#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
            "#,
        )
        .file("foo/src/lib.rs", "use std::io;")
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.0.1"

                [dependencies]
                foo = { path = "../foo" }
            "#,
        )
        .file("bar/src/lib.rs", "use std::io;")
        .build();

    p.cargo("check")
        .with_stderr_contains("[..] (run `cargo fix --lib -p foo` to apply 1 suggestion)")
        .with_stderr_contains("[..] (run `cargo fix --lib -p bar` to apply 1 suggestion)")
        .run();
}

#[cargo_test]
fn check_fixable_example() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file(
            "src/main.rs",
            r#"
            fn hello() -> &'static str {
                "hello"
            }

            pub fn main() {
                println!("{}", hello())
            }
            "#,
        )
        .file("examples/ex1.rs", "use std::fmt; fn main() {}")
        .build();
    p.cargo("check --all-targets")
        .with_stderr_contains("[..] (run `cargo fix --example \"ex1\"` to apply 1 suggestion)")
        .run();
}

#[cargo_test(nightly, reason = "bench")]
fn check_fixable_bench() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file(
            "src/main.rs",
            r#"
            #![feature(test)]
            #[cfg(test)]
            extern crate test;

            fn hello() -> &'static str {
                "hello"
            }

            pub fn main() {
                println!("{}", hello())
            }

            #[bench]
            fn bench_hello(_b: &mut test::Bencher) {
                use std::io;
                assert_eq!(hello(), "hello")
            }
            "#,
        )
        .file(
            "benches/bench.rs",
            "
            #![feature(test)]
            extern crate test;

            #[bench]
            fn bench(_b: &mut test::Bencher) { use std::fmt; }
        ",
        )
        .build();
    p.cargo("check --all-targets")
        .with_stderr_contains("[..] (run `cargo fix --bench \"bench\"` to apply 1 suggestion)")
        .run();
}

#[cargo_test(nightly, reason = "bench")]
fn check_fixable_mixed() {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("foo"))
        .file(
            "src/main.rs",
            r#"
            #![feature(test)]
            #[cfg(test)]
            extern crate test;

            fn hello() -> &'static str {
                "hello"
            }

            pub fn main() {
                println!("{}", hello())
            }

            #[bench]
            fn bench_hello(_b: &mut test::Bencher) {
                use std::io;
                assert_eq!(hello(), "hello")
            }
            #[test]
            fn t1() {
                use std::fmt;
            }
            "#,
        )
        .file("examples/ex1.rs", "use std::fmt; fn main() {}")
        .file(
            "benches/bench.rs",
            "
            #![feature(test)]
            extern crate test;

            #[bench]
            fn bench(_b: &mut test::Bencher) { use std::fmt; }
        ",
        )
        .build();
    p.cargo("check --all-targets")
        .with_stderr_contains("[..] (run `cargo fix --bin \"foo\" --tests` to apply 2 suggestions)")
        .with_stderr_contains("[..] (run `cargo fix --example \"ex1\"` to apply 1 suggestion)")
        .with_stderr_contains("[..] (run `cargo fix --bench \"bench\"` to apply 1 suggestion)")
        .run();
}

#[cargo_test]
fn check_fixable_warning_for_clippy() {
    let foo = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
            "#,
        )
        // We don't want to show a warning that is `clippy`
        // specific since we are using a `rustc` wrapper
        // inplace of `clippy`
        .file("src/lib.rs", "use std::io;")
        .build();

    foo.cargo("check")
        // We can't use `clippy` so we use a `rustc` workspace wrapper instead
        .env("RUSTC_WORKSPACE_WRAPPER", wrapped_clippy_driver())
        .with_stderr_contains("[..] (run `cargo clippy --fix --lib -p foo` to apply 1 suggestion)")
        .run();
}

#[cargo_test]
fn check_unused_manifest_keys() {
    Package::new("dep", "0.1.0").publish();
    Package::new("foo", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "0.2.0"
            authors = []

            [dependencies]
            dep = { version = "0.1.0", wxz = "wxz" }
            foo = { version = "0.1.0", abc = "abc" }

            [dev-dependencies]
            foo = { version = "0.1.0", wxz = "wxz" }

            [build-dependencies]
            foo = { version = "0.1.0", wxz = "wxz" }

            [target.'cfg(windows)'.dependencies]
            foo = { version = "0.1.0", wxz = "wxz" }

            [target.x86_64-pc-windows-gnu.dev-dependencies]
            foo = { version = "0.1.0", wxz = "wxz" }

            [target.bar.build-dependencies]
            foo = { version = "0.1.0", wxz = "wxz" }
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("check")
        .with_stderr(
            "\
[WARNING] unused manifest key: dependencies.dep.wxz
[WARNING] unused manifest key: dependencies.foo.abc
[WARNING] unused manifest key: dev-dependencies.foo.wxz
[WARNING] unused manifest key: build-dependencies.foo.wxz
[WARNING] unused manifest key: target.bar.build-dependencies.foo.wxz
[WARNING] unused manifest key: target.cfg(windows).dependencies.foo.wxz
[WARNING] unused manifest key: target.x86_64-pc-windows-gnu.dev-dependencies.foo.wxz
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] foo v0.1.0 ([..])
[DOWNLOADED] dep v0.1.0 ([..])
[CHECKING] [..]
[CHECKING] [..]
[CHECKING] bar v0.2.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}
