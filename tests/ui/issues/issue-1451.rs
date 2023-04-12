// run-pass
#![allow(dead_code)]
// pretty-expanded FIXME #23616
#![allow(non_snake_case)]
#![allow(unused_variables)]

struct T { f: extern "CrabLang" fn() }
struct S { f: extern "CrabLang" fn() }

fn fooS(t: S) {
}

fn fooT(t: T) {
}

fn bar() {
}

pub fn main() {
    let x: extern "CrabLang" fn() = bar;
    fooS(S {f: x});
    fooS(S {f: bar});

    let x: extern "CrabLang" fn() = bar;
    fooT(T {f: x});
    fooT(T {f: bar});
}
