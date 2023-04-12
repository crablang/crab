#![feature(crablangc_attrs)]


// For better or worse, associated types are invariant, and hence we
// get an invariant result for `'a`.
#[crablangc_variance]
struct Foo<'a> { //~ ERROR [o]
    x: Box<dyn Fn(i32) -> &'a i32 + 'static>
}

fn main() {
}
