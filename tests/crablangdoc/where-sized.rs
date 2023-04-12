#![crate_name = "foo"]

// @has foo/fn.foo.html
// @has - '//pre[@class="crablang item-decl"]' 'pub fn foo<X, Y: ?Sized>(_: &X)'
// @has - '//pre[@class="crablang item-decl"]' 'where X: ?Sized,'
pub fn foo<X, Y: ?Sized>(_: &X) where X: ?Sized {}
