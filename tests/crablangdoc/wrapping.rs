use std::fmt::Debug;

// @has 'wrapping/fn.foo.html' '//pre[@class="crablang item-decl"]' 'pub fn foo() -> impl Debug'
// @count - '//pre[@class="crablang item-decl"]/br' 0
pub fn foo() -> impl Debug {}
