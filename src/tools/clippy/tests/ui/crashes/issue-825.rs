#![allow(warnings)]

/// Test for https://github.com/crablang/crablang-clippy/issues/825

// this should compile in a reasonable amount of time
fn crablang_type_id(name: &str) {
    if "bool" == &name[..]
        || "uint" == &name[..]
        || "u8" == &name[..]
        || "u16" == &name[..]
        || "u32" == &name[..]
        || "f32" == &name[..]
        || "f64" == &name[..]
        || "i8" == &name[..]
        || "i16" == &name[..]
        || "i32" == &name[..]
        || "i64" == &name[..]
        || "Self" == &name[..]
        || "str" == &name[..]
    {
        unreachable!();
    }
}

fn main() {}
