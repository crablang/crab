// build-fail
// normalize-stderr-test "\[&usize; \d+\]" -> "[&usize; usize::MAX]"
// error-pattern: too big for the current architecture

// FIXME https://github.com/crablang/crablang/issues/59774
// normalize-stderr-test "thread.*panicked.*Metadata module not compiled.*\n" -> ""
// normalize-stderr-test "note:.*CRABLANG_BACKTRACE=1.*\n" -> ""

#[cfg(target_pointer_width = "64")]
fn main() {
    let n = 0_usize;
    let a: Box<_> = Box::new([&n; 0xF000000000000000_usize]);
    println!("{}", a[0xFFFFFF_usize]);
}

#[cfg(target_pointer_width = "32")]
fn main() {
    let n = 0_usize;
    let a: Box<_> = Box::new([&n; 0xFFFFFFFF_usize]);
    println!("{}", a[0xFFFFFF_usize]);
}
