# Disjoint capture in closures

## Summary

- `|| a.x + 1` now captures only `a.x` instead of `a`.
- This can cause things to be dropped at different times or affect whether closures implement traits like `Send` or `Clone`.
  - If possible changes are detected, `cargo fix` will insert statements like `let _ = &a` to force a closure to capture the entire variable.

## Details

[Closures](https://doc.rust-lang.org/book/ch13-01-closures.html)
automatically capture anything that you refer to from within their body.
For example, `|| a + 1` automatically captures a reference to `a` from the surrounding context.

In Rust 2018 and before, closures capture entire variables, even if the closure only uses one field.
For example, `|| a.x + 1` captures a reference to `a` and not just `a.x`.
Capturing `a` in its entirety prevents mutation or moves from other fields of `a`, so that code like this does not compile:

```rust,ignore
let a = SomeStruct::new();
drop(a.x); // Move out of one field of the struct
println!("{}", a.y); // Ok: Still use another field of the struct
let c = || println!("{}", a.y); // Error: Tries to capture all of `a`
c();
```

Starting in Rust 2021, closures captures are more precise. Typically they will only capture the fields they use (in some cases, they might capture more than just what they use, see the Rust reference for full details). Therefore, the above example will compile fine in Rust 2021.

Disjoint capture was proposed as part of [RFC 2229](https://github.com/rust-lang/rfcs/blob/master/text/2229-capture-disjoint-fields.md) and the RFC contains details about the motivation.

## Migration

As a part of the 2021 edition a migration lint, `rust_2021_incompatible_closure_captures`, has been added in order to aid in automatic migration of Rust 2018 codebases to Rust 2021.

In order to have `rustfix` migrate your code to be Rust 2021 Edition compatible, run:

```sh
cargo fix --edition
```

Below is an examination of how to manually migrate code to use closure captures that are compatible with Rust 2021 should the automatic migration fail 
or you would like to better understand how the migration works.

Changing the variables captured by a closure can cause programs to change behavior or to stop compiling in two cases:

- changes to drop order, or when destructors run ([details](#drop-order));
- changes to which traits a closure implements ([details](#trait-implementations)).

Whenever any of the scenarios below are detected, `cargo fix` will insert a "dummy let" into your closure to force it to capture the entire variable:

```rust
let x = (vec![22], vec![23]);
let c = move || {
    // "Dummy let" that forces `x` to be captured in its entirety
    let _ = &x;

    // Otherwise, only `x.0` would be captured here
    println!("{:?}", x.0);
};
```

This is a conservative analysis: in many cases, these dummy lets can be safely removed and your program will work fine.

### Wild Card Patterns

Closures now only capture data that needs to be read, which means the following closures will not capture `x`:

```rust
let x = 10;
let c = || {
    let _ = x; // no-op
};

let c = || match x {
    _ => println!("Hello World!")
};
```

The `let _ = x` statement here is a no-op, since the `_` pattern completely ignores the right-hand side, and `x` is a reference to a place in memory (in this case, a variable).

This change by itself (capturing fewer values) doesn't trigger any suggestions, but it may do so in conjunction with the "drop order" change below.

**Subtle:** There are other similar expressions, such as the "dummy lets" `let _ = &x` that we insert, which are not no-ops. This is because the right-hand side (`&x`) is not a reference to a place in memory, but rather an expression that must first be evaluated (and whose result is then discarded).

### Drop Order

When a closure takes ownership of a value from a variable `t`, that value is then dropped when the closure is dropped, and not when the variable `t` goes out of scope:

```rust
# fn move_value<T>(_: T){}
{
    let t = (vec![0], vec![0]);

    {
        let c = || move_value(t); // t is moved here
    } // c is dropped, which drops the tuple `t` as well
} // t goes out of scope here
```

The above code will run the same in both Rust 2018 and Rust 2021. However, in cases where the closure only takes ownership of _part_ of a variable, there can be differences:

```rust
# fn move_value<T>(_: T){}
{
    let t = (vec![0], vec![0]);

    {
        let c = || {
            // In Rust 2018, captures all of `t`.
            // In Rust 2021, captures only `t.0`
            move_value(t.0);
        };

        // In Rust 2018, `c` (and `t`) are both dropped when we
        // exit this block.
        //
        // In Rust 2021, `c` and `t.0` are both dropped when we
        // exit this block.
    }

// In Rust 2018, the value from `t` has been moved and is
// not dropped.
//
// In Rust 2021, the value from `t.0` has been moved, but `t.1`
// remains, so it will be dropped here.
}
```

In most cases, dropping values at different times just affects when memory is freed and is not important. However, some `Drop` impls (aka, destructors) have side-effects, and changing the drop order in those cases can alter the semantics of your program. In such cases, the compiler will suggest inserting a dummy `let` to force the entire variable to be captured.

### Trait implementations

Closures automatically implement the following traits based on what values they capture:

- [`Clone`]: if all captured values are `Clone`.
- [Auto traits] like [`Send`], [`Sync`], and [`UnwindSafe`]: if all captured values implement the given trait.

[auto traits]: https://doc.rust-lang.org/nightly/reference/special-types-and-traits.html#auto-traits
[`clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
[`send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
[`sync`]: https://doc.rust-lang.org/std/marker/trait.Sync.html
[`unwindsafe`]: https://doc.rust-lang.org/std/panic/trait.UnwindSafe.html

In Rust 2021, since different values are being captured, this can affect what traits a closure will implement. The migration lints test each closure to see whether it would have implemented a given trait before and whether it still implements it now; if they find that a trait used to be implemented but no longer is, then "dummy lets" are inserted.

For instance, a common way to allow passing around raw pointers between threads is to wrap them in a struct and then implement `Send`/`Sync` auto trait for the wrapper. The closure that is passed to `thread::spawn` uses the specific fields within the wrapper but the entire wrapper is captured regardless. Since the wrapper is `Send`/`Sync`, the code is considered safe and therefore compiles successfully.

With disjoint captures, only the specific field mentioned in the closure gets captured, which wasn't originally `Send`/`Sync` defeating the purpose of the wrapper.

```rust
use std::thread;

struct Ptr(*mut i32);
unsafe impl Send for Ptr {}


let mut x = 5;
let px = Ptr(&mut x as *mut i32);

let c = thread::spawn(move || {
    unsafe {
        *(px.0) += 10;
    }
}); // Closure captured px.0 which is not Send
```
