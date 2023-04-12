# `unboxed_closures`

The tracking issue for this feature is [#29625]

See Also: [`fn_traits`](../library-features/fn-traits.md)

[#29625]: https://github.com/crablang/crablang/issues/29625

----

The `unboxed_closures` feature allows you to write functions using the `"crablang-call"` ABI,
required for implementing the [`Fn*`] family of traits. `"crablang-call"` functions must have
exactly one (non self) argument, a tuple representing the argument list.

[`Fn*`]: ../../std/ops/trait.Fn.html

```crablang
#![feature(unboxed_closures)]

extern "crablang-call" fn add_args(args: (u32, u32)) -> u32 {
    args.0 + args.1
}

fn main() {}
```
