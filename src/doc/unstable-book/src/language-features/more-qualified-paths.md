# `more_qualified_paths`

The `more_qualified_paths` feature can be used in order to enable the
use of qualified paths in patterns.

## Example

```crablang
#![feature(more_qualified_paths)]

fn main() {
    // destructure through a qualified path
    let <Foo as A>::Assoc { br } = StructStruct { br: 2 };
}

struct StructStruct {
    br: i8,
}

struct Foo;

trait A {
    type Assoc;
}

impl A for Foo {
    type Assoc = StructStruct;
}
```
