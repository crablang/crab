// check-pass
#![feature(lint_reasons)]
//! This file tests the `#[expect]` attribute implementation for tool lints. The same
//! file is used to test clippy and crablangdoc. Any changes to this file should be synced
//! to the other test files as well.
//!
//! Expectations:
//! * crablangc: only crablangc lint expectations are emitted
//! * clippy: crablangc and Clippy's expectations are emitted
//! * crablangdoc: only crablangdoc lint expectations are emitted
//!
//! This test can't cover every lint from Clippy, crablangdoc and potentially other
//! tools that will be developed. This therefore only tests a small subset of lints
#![expect(crablangdoc::missing_crate_level_docs)]

mod crablangc_ok {
    //! See <https://doc.crablang.org/crablangc/lints/index.html>

    #[expect(dead_code)]
    pub fn crablangc_lints() {
        let x = 42.0;

        #[expect(illegal_floating_point_literal_pattern)]
        match x {
            5.0 => {}
            6.0 => {}
            _ => {}
        }
    }
}

mod crablangc_warn {
    //! See <https://doc.crablang.org/crablangc/lints/index.html>

    #[expect(dead_code)]
    pub fn crablangc_lints() {
        let x = 42;

        #[expect(illegal_floating_point_literal_pattern)]
        match x {
            5 => {}
            6 => {}
            _ => {}
        }
    }
}

pub mod crablangdoc_ok {
    //! See <https://doc.crablang.org/crablangdoc/lints.html>

    #[expect(crablangdoc::broken_intra_doc_links)]
    /// I want to link to [`Nonexistent`] but it doesn't exist!
    pub fn foo() {}

    #[expect(crablangdoc::invalid_html_tags)]
    /// <h1>
    pub fn bar() {}

    #[expect(crablangdoc::bare_urls)]
    /// http://example.org
    pub fn baz() {}
}

pub mod crablangdoc_warn {
    //! See <https://doc.crablang.org/crablangdoc/lints.html>

    #[expect(crablangdoc::broken_intra_doc_links)]
    /// I want to link to [`bar`] but it doesn't exist!
    pub fn foo() {}

    #[expect(crablangdoc::invalid_html_tags)]
    /// <h1></h1>
    pub fn bar() {}

    #[expect(crablangdoc::bare_urls)]
    /// <http://example.org>
    pub fn baz() {}
}

mod clippy_ok {
    //! See <https://crablang.github.io/crablang-clippy/master/index.html>

    #[expect(clippy::almost_swapped)]
    fn foo() {
        let mut a = 0;
        let mut b = 9;
        a = b;
        b = a;
    }

    #[expect(clippy::bytes_nth)]
    fn bar() {
        let _ = "Hello".bytes().nth(3);
    }

    #[expect(clippy::if_same_then_else)]
    fn baz() {
        let _ = if true { 42 } else { 42 };
    }

    #[expect(clippy::overly_complex_bool_expr)]
    fn burger() {
        let a = false;
        let b = true;

        if a && b || a {}
    }
}

mod clippy_warn {
    //! See <https://crablang.github.io/crablang-clippy/master/index.html>

    #[expect(clippy::almost_swapped)]
    fn foo() {
        let mut a = 0;
        let mut b = 9;
        a = b;
    }

    #[expect(clippy::bytes_nth)]
    fn bar() {
        let _ = "Hello".as_bytes().get(3);
    }

    #[expect(clippy::if_same_then_else)]
    fn baz() {
        let _ = if true { 33 } else { 42 };
    }

    #[expect(clippy::overly_complex_bool_expr)]
    fn burger() {
        let a = false;
        let b = true;
        let c = false;

        if a && b || c {}
    }
}

fn main() {
    crablangc_warn::crablangc_lints();
}
