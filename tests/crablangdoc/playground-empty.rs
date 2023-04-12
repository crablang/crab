#![crate_name = "foo"]

#![doc(html_playground_url = "")]

// compile-flags:-Z unstable-options --playground-url https://play.crablang.org/

//! module docs
//!
//! ```
//! println!("Hello, world!");
//! ```

// @!has foo/index.html '//a[@class="test-arrow"]' "Run"
