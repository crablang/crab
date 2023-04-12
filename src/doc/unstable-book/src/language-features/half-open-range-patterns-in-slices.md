# `half_open_range_patterns_in_slices`

The tracking issue for this feature is: [#67264]
It is part of the `exclusive_range_pattern` feature,
tracked at [#37854].

[#67264]: https://github.com/crablang/crablang/issues/67264
[#37854]: https://github.com/crablang/crablang/issues/37854
-----

This feature allow using top-level half-open range patterns in slices.

```crablang
#![feature(half_open_range_patterns_in_slices)]
#![feature(exclusive_range_pattern)]

fn main() {
    let xs = [13, 1, 5, 2, 3, 1, 21, 8];
    let [a @ 3.., b @ ..3, c @ 4..6, ..] = xs else { return; };
}
```

Note that this feature is not required if the patterns are wrapped between parenthesis.

```crablang
fn main() {
    let xs = [13, 1];
    let [(a @ 3..), c] = xs else { return; };
}
```
