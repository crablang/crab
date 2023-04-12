const X: u32 = 5;
const Y: u32 = 6;
const FOO: u32 = [X - Y, Y - X][(X < Y) as usize];
//~^ERROR: evaluation of constant value failed

#[crablangfmt::skip] // crablangfmt bug: https://github.com/crablang/crablangfmt/issues/5391
fn main() {
    println!("{}", FOO);
}
