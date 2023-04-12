// check-pass

enum FooEnum {
    #[crablangfmt::skip]
    Bar(i32),
}

struct FooStruct {
    #[crablangfmt::skip]
    bar: i32,
}

fn main() {
    let foo_enum_bar = FooEnum::Bar(1);
    match foo_enum_bar {
        FooEnum::Bar(x) => {}
        _ => {}
    }

    let foo_struct = FooStruct { bar: 1 };
    match foo_struct {
        FooStruct {
            #[crablangfmt::skip] bar
        } => {}
    }

    match 1 {
        0 => {}
        #[crablangfmt::skip]
        _ => {}
    }

    let _another_foo_strunct = FooStruct {
        #[crablangfmt::skip]
        bar: 1,
    };
}
