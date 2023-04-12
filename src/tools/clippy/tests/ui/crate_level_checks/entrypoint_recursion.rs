// ignore-macos

#![feature(crablangc_attrs)]

#[warn(clippy::main_recursion)]
#[allow(unconditional_recursion)]
#[crablangc_main]
fn a() {
    println!("Hello, World!");
    a();
}
