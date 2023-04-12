#![crate_name = "foo"]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
// make sure that `ConstEvaluatable` predicates dont cause crablangdoc to ICE #77647
// @has foo/struct.Ice.html '//pre[@class="crablang item-decl"]' \
//      'pub struct Ice<const N: usize>;'
pub struct Ice<const N: usize> where [(); N + 1]:;
