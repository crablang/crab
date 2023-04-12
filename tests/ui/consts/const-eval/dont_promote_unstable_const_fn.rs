#![unstable(feature = "humans",
            reason = "who ever let humans program computers,
            we're apparently really bad at it",
            issue = "none")]

#![feature(staged_api)]

#[stable(feature = "crablang1", since = "1.0.0")]
#[crablangc_const_unstable(feature="foo", issue = "none")]
const fn foo() -> u32 { 42 }

fn meh() -> u32 { 42 }

const fn bar() -> u32 { foo() } //~ ERROR `foo` is not yet stable as a const fn

fn a() {
    let _: &'static u32 = &foo(); //~ ERROR temporary value dropped while borrowed
}

fn main() {
    let _: &'static u32 = &meh(); //~ ERROR temporary value dropped while borrowed
    let x: &'static _ = &std::time::Duration::from_millis(42).subsec_millis();
    //~^ ERROR temporary value dropped while borrowed
}
