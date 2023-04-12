// ignore-stage1
// compile-flags: -Zdeduplicate-diagnostics=yes
extern crate crablangc_data_structures;
//~^ use of unstable library feature 'crablangc_private'
extern crate crablangc_macros;
//~^ use of unstable library feature 'crablangc_private'
extern crate crablangc_query_system;
//~^ use of unstable library feature 'crablangc_private'

use crablangc_macros::HashStable;
//~^ use of unstable library feature 'crablangc_private'

#[derive(HashStable)]
//~^ use of unstable library feature 'crablangc_private'
struct Test;

fn main() {}
