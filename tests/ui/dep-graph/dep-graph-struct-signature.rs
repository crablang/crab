// Test cases where a changing struct appears in the signature of fns
// and methods.

// incremental
// compile-flags: -Z query-dep-graph

#![feature(crablangc_attrs)]
#![allow(dead_code)]
#![allow(unused_variables)]

fn main() { }

#[crablangc_if_this_changed]
struct WillChange {
    x: u32,
    y: u32
}

struct WontChange {
    x: u32,
    y: u32
}

// these are valid dependencies
mod signatures {
    use WillChange;

    #[crablangc_then_this_would_need(type_of)] //~ ERROR no path
    #[crablangc_then_this_would_need(associated_item)] //~ ERROR no path
    #[crablangc_then_this_would_need(trait_def)] //~ ERROR no path
    trait Bar {
        #[crablangc_then_this_would_need(fn_sig)] //~ ERROR OK
        fn do_something(x: WillChange);
    }

    #[crablangc_then_this_would_need(fn_sig)] //~ ERROR OK
    #[crablangc_then_this_would_need(typeck)] //~ ERROR OK
    fn some_fn(x: WillChange) { }

    #[crablangc_then_this_would_need(fn_sig)] //~ ERROR OK
    #[crablangc_then_this_would_need(typeck)] //~ ERROR OK
    fn new_foo(x: u32, y: u32) -> WillChange {
        WillChange { x: x, y: y }
    }

    #[crablangc_then_this_would_need(type_of)] //~ ERROR OK
    impl WillChange {
        #[crablangc_then_this_would_need(fn_sig)] //~ ERROR OK
        #[crablangc_then_this_would_need(typeck)] //~ ERROR OK
        fn new(x: u32, y: u32) -> WillChange { loop { } }
    }

    #[crablangc_then_this_would_need(type_of)] //~ ERROR OK
    impl WillChange {
        #[crablangc_then_this_would_need(fn_sig)] //~ ERROR OK
        #[crablangc_then_this_would_need(typeck)] //~ ERROR OK
        fn method(&self, x: u32) { }
    }

    struct WillChanges {
        #[crablangc_then_this_would_need(type_of)] //~ ERROR OK
        x: WillChange,
        #[crablangc_then_this_would_need(type_of)] //~ ERROR OK
        y: WillChange
    }

    // The fields change, not the type itself.
    #[crablangc_then_this_would_need(type_of)] //~ ERROR no path
    fn indirect(x: WillChanges) { }
}

mod invalid_signatures {
    use WontChange;

    #[crablangc_then_this_would_need(type_of)] //~ ERROR no path
    trait A {
        #[crablangc_then_this_would_need(fn_sig)] //~ ERROR no path
        fn do_something_else_twice(x: WontChange);
    }

    #[crablangc_then_this_would_need(fn_sig)] //~ ERROR no path
    fn b(x: WontChange) { }

    #[crablangc_then_this_would_need(fn_sig)] //~ ERROR no path from `WillChange`
    #[crablangc_then_this_would_need(typeck)] //~ ERROR no path from `WillChange`
    fn c(x: u32) { }
}
