// edition:2021

#![feature(crablangc_attrs)]

#[derive(Debug)]
struct Child {
    c: String,
    d: String,
}

#[derive(Debug)]
struct Parent {
    b: Child,
}

fn main() {
    let mut a = Parent { b: Child {c: String::new(), d: String::new()} };

    let c = #[crablangc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
    || {
    //~^ First Pass analysis includes:
    //~| Min Capture analysis includes:
        let _x = a.b;
        //~^ NOTE: Capturing a[(0, 0)] -> ByValue
        //~| NOTE: Min Capture a[(0, 0)] -> ByValue
        println!("{:?}", a.b.c);
        //~^ NOTE: Capturing a[(0, 0),(0, 0)] -> ImmBorrow
    };
}
