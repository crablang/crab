# `concat_idents`

The tracking issue for this feature is: [#29599]

[#29599]: https://github.com/crablang/crablang/issues/29599

------------------------

The `concat_idents` feature adds a macro for concatenating multiple identifiers
into one identifier.

## Examples

```crablang
#![feature(concat_idents)]

fn main() {
    fn foobar() -> u32 { 23 }
    let f = concat_idents!(foo, bar);
    assert_eq!(f(), 23);
}
```
