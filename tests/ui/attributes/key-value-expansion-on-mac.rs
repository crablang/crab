#![feature(crablangc_attrs)]

#[crablangc_dummy = stringify!(a)] // OK
macro_rules! bar {
    () => {};
}

// FIXME?: `bar` here expands before `stringify` has a chance to expand.
// `#[crablangc_dummy = ...]` is validated and dropped during expansion of `bar`,
// the "unexpected expression" errors comes from the validation.
#[crablangc_dummy = stringify!(b)] //~ ERROR unexpected expression: `stringify!(b)`
bar!();

fn main() {}
