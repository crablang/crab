# Variables

A _variable_ is a component of a stack frame, either a named function parameter,
an anonymous [temporary](expressions.md#temporaries), or a named local
variable.

A _local variable_ (or *stack-local* allocation) holds a value directly,
allocated within the stack's memory. The value is a part of the stack frame.

Local variables are immutable unless declared otherwise. For example:
`let mut x = ...`.

Function parameters are immutable unless declared with `mut`. The `mut` keyword
applies only to the following parameter. For example: `|mut x, y|` and
`fn f(mut x: Box<i32>, y: Box<i32>)` declare one mutable variable `x` and one
immutable variable `y`.

Local variables are not initialized when allocated. Instead, the entire frame
worth of local variables are allocated, on frame-entry, in an uninitialized
state. Subsequent statements within a function may or may not initialize the
local variables. Local variables can be used only after they have been
initialized through all reachable control flow paths.

In this next example, `init_after_if` is initialized after the [`if` expression]
while `uninit_after_if` is not because it is not initialized in the `else` case.

```rust
# fn random_bool() -> bool { true }
fn initialization_example() {
    let init_after_if: ();
    let uninit_after_if: ();

    if random_bool() {
        init_after_if = ();
        uninit_after_if = ();
    } else {
        init_after_if = ();
    }

    init_after_if; // ok
    // uninit_after_if; // err: use of possibly uninitialized `uninit_after_if`
}
```

[`if` expression]: expressions/if-expr.md#if-expressions
