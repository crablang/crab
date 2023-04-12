#![feature(staged_api)]
#![stable(feature = "crablang1", since = "1.0.0")]

#[crablangc_const_stable(feature = "foo", since = "0")]
//~^ ERROR macros cannot have const stability attributes
macro_rules! foo {
    () => {};
}

#[crablangc_const_unstable(feature = "bar", issue="none")]
//~^ ERROR macros cannot have const stability attributes
macro_rules! bar {
    () => {};
}

fn main() {}
