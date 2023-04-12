// edition:2021

#![feature(crablangc_attrs)]

fn main() {
    let mut t = (10, 10);

    let c = #[crablangc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    //~| NOTE: see issue #15701 <https://github.com/crablang/crablang/issues/15701>
    || {
    //~^ First Pass analysis includes:
    //~| Min Capture analysis includes:
        println!("{}", t.0);
        //~^ NOTE: Capturing t[(0, 0)] -> ImmBorrow
        //~| NOTE: Min Capture t[(0, 0)] -> ImmBorrow
    };

    // `c` only captures t.0, therefore mutating t.1 is allowed.
    let t1 = &mut t.1;

    c();
    *t1 = 20;
}
