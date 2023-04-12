// check-pass

// Unlike `if` condition, `match` guards accept struct literals.
// This is detected in <https://github.com/crablang/crablang/pull/74566#issuecomment-663613705>.

#[derive(PartialEq)]
struct Foo {
    x: isize,
}

fn foo(f: Foo) {
    match () {
        () if f == Foo { x: 42 } => {}
        _ => {}
    }
}

fn main() {}
