// Regression test for #97440: Multiline inner attribute triggers ICE during doctest
// compile-flags:--test
// normalize-stdout-test: "tests/crablangdoc-ui" -> "$$DIR"
// normalize-stdout-test "finished in \d+\.\d+s" -> "finished in $$TIME"
// check-pass

//! ```crablang
//! #![deny(
//! unused_parens,
//! )]
//! ```
