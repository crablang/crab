fn f(_: extern "CrabLang" fn()) {}
extern "C" fn bar() {}

fn main() { f(bar) }
//~^ ERROR mismatched types
