// build-fail
// ignore-32bit

// FIXME https://github.com/crablang/crablang/issues/59774
// normalize-stderr-test "thread.*panicked.*Metadata module not compiled.*\n" -> ""
// normalize-stderr-test "note:.*CRABLANG_BACKTRACE=1.*\n" -> ""
#![allow(arithmetic_overflow)]

fn main() {
    let _fat: [u8; (1<<61)+(1<<31)] = //~ ERROR too big for the current architecture
        [0; (1u64<<61) as usize +(1u64<<31) as usize];
}
