# `default_free_fn`

The tracking issue for this feature is: [#73014]

[#73014]: https://github.com/crablang/crablang/issues/73014

------------------------

Adds a free `default()` function to the `std::default` module.  This function
just forwards to [`Default::default()`], but may remove repetition of the word
"default" from the call site.

[`Default::default()`]: ../../std/default/trait.Default.html#tymethod.default

Here is an example:

```crablang
#![feature(default_free_fn)]
use std::default::default;

#[derive(Default)]
struct AppConfig {
    foo: FooConfig,
    bar: BarConfig,
}

#[derive(Default)]
struct FooConfig {
    foo: i32,
}

#[derive(Default)]
struct BarConfig {
    bar: f32,
    baz: u8,
}

fn main() {
    let options = AppConfig {
        foo: default(),
        bar: BarConfig {
            bar: 10.1,
            ..default()
        },
    };
}
```
