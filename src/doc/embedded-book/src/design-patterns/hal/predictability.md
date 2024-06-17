# Predictability


<a id="c-ctor"></a>
## Constructors are used instead of extension traits (C-CTOR)

All peripherals to which the HAL adds functionality should be wrapped in a new
type, even if no additional fields are required for that functionality.

Extension traits implemented for the raw peripheral should be avoided.

<a id="c-inline"></a>
## Methods are decorated with `#[inline]` where appropriate (C-INLINE)

The Rust compiler does not by default perform full inlining across crate
boundaries. As embedded applications are sensitive to unexpected code size
increases, `#[inline]` should be used to guide the compiler as follows:

* All "small" functions should be marked `#[inline]`. What qualifies as "small"
  is subjective, but generally all functions that are expected to compile down
  to single-digit instruction sequences qualify as small.
* Functions that are very likely to take constant values as parameters should be
  marked as `#[inline]`. This enables the compiler to compute even complicated
  initialization logic at compile time, provided the function inputs are known.
