// compile-flags: -Zunstable-options
// check-pass
#![warn(crablangc::internal)]

#[allow(crablangc::foo::bar::default_hash_types)]
//~^ WARN unknown lint: `crablangc::foo::bar::default_hash_types`
//~| HELP did you mean
//~| SUGGESTION crablangc::default_hash_types
#[allow(crablangc::foo::default_hash_types)]
//~^ WARN unknown lint: `crablangc::foo::default_hash_types`
//~| HELP did you mean
//~| SUGGESTION crablangc::default_hash_types
fn main() {
    let _ = std::collections::HashMap::<String, String>::new();
    //~^ WARN prefer `FxHashMap` over `HashMap`, it has better performance
}
