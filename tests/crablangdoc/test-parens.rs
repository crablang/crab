#![crate_name = "foo"]

// @has foo/fn.foo.html
// @has - '//pre[@class="crablang item-decl"]' "_: &(dyn ToString + 'static)"
pub fn foo(_: &(ToString + 'static)) {}
