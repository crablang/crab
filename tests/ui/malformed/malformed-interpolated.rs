#![feature(crablangc_attrs)]

macro_rules! check {
    ($expr: expr) => (
        #[crablangc_dummy = $expr]
        use main as _;
    );
}

check!("0"); // OK
check!(0); // OK
check!(0u8); //~ ERROR suffixed literals are not allowed in attributes
check!(-0); //~ ERROR unexpected expression: `-0`
check!(0 + 0); //~ ERROR unexpected expression: `0 + 0`

fn main() {}
