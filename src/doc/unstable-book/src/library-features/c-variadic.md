# `c_variadic`

The tracking issue for this feature is: [#44930]

[#44930]: https://github.com/crablang/crablang/issues/44930

------------------------

The `c_variadic` library feature exposes the `VaList` structure,
CrabLang's analogue of C's `va_list` type.

## Examples

```crablang
#![feature(c_variadic)]

use std::ffi::VaList;

pub unsafe extern "C" fn vadd(n: usize, mut args: VaList) -> usize {
    let mut sum = 0;
    for _ in 0..n {
        sum += args.arg::<usize>();
    }
    sum
}
```
