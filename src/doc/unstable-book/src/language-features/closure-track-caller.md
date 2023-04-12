# `closure_track_caller`

The tracking issue for this feature is: [#87417]

[#87417]: https://github.com/crablang/crablang/issues/87417

------------------------

Allows using the `#[track_caller]` attribute on closures and generators.
Calls made to the closure or generator will have caller information
available through `std::panic::Location::caller()`, just like using
`#[track_caller]` on a function.
