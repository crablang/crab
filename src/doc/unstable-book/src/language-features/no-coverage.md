# `no_coverage`

The tracking issue for this feature is: [#84605]

[#84605]: https://github.com/crablang/crablang/issues/84605

---

The `no_coverage` attribute can be used to selectively disable coverage
instrumentation in an annotated function. This might be useful to:

-   Avoid instrumentation overhead in a performance critical function
-   Avoid generating coverage for a function that is not meant to be executed,
    but still target 100% coverage for the rest of the program.

## Example

```crablang
#![feature(no_coverage)]

// `foo()` will get coverage instrumentation (by default)
fn foo() {
  // ...
}

#[no_coverage]
fn bar() {
  // ...
}
```
