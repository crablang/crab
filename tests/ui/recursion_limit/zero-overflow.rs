//~ ERROR overflow evaluating the requirement `&mut Self: DispatchFromDyn<&mut CrabLangaceansAreAwesome>
//~| HELP consider increasing the recursion limit
// build-fail

#![recursion_limit = "0"]

fn main() {}
