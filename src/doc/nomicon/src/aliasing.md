# Aliasing

First off, let's get some important caveats out of the way:

* We will be using the broadest possible definition of aliasing for the sake
of discussion. Rust's definition will probably be more restricted to factor
in mutations and liveness.

* We will be assuming a single-threaded, interrupt-free, execution. We will also
be ignoring things like memory-mapped hardware. Rust assumes these things
don't happen unless you tell it otherwise. For more details, see the
[Concurrency Chapter](concurrency.html).

With that said, here's our working definition: variables and pointers *alias*
if they refer to overlapping regions of memory.

## Why Aliasing Matters

So why should we care about aliasing?

Consider this simple function:

```rust
fn compute(input: &u32, output: &mut u32) {
    if *input > 10 {
        *output = 1;
    }
    if *input > 5 {
        *output *= 2;
    }
    // remember that `output` will be `2` if `input > 10`
}
```

We would *like* to be able to optimize it to the following function:

```rust
fn compute(input: &u32, output: &mut u32) {
    let cached_input = *input; // keep `*input` in a register
    if cached_input > 10 {
        // If the input is greater than 10, the previous code would set the output to 1 and then double it,
        // resulting in an output of 2 (because `>10` implies `>5`).
        // Here, we avoid the double assignment and just set it directly to 2.
        *output = 2;
    } else if cached_input > 5 {
        *output *= 2;
    }
}
```

In Rust, this optimization should be sound. For almost any other language, it
wouldn't be (barring global analysis). This is because the optimization relies
on knowing that aliasing doesn't occur, which most languages are fairly liberal
with. Specifically, we need to worry about function arguments that make `input`
and `output` overlap, such as `compute(&x, &mut x)`.

With that input, we could get this execution:

<!-- ignore: expanded code -->
```rust,ignore
                    //  input ==  output == 0xabad1dea
                    // *input == *output == 20
if *input > 10 {    // true  (*input == 20)
    *output = 1;    // also overwrites *input, because they are the same
}
if *input > 5 {     // false (*input == 1)
    *output *= 2;
}
                    // *input == *output == 1
```

Our optimized function would produce `*output == 2` for this input, so the
correctness of our optimization relies on this input being impossible.

In Rust we know this input should be impossible because `&mut` isn't allowed to be
aliased. So we can safely reject its possibility and perform this optimization.
In most other languages, this input would be entirely possible, and must be considered.

This is why alias analysis is important: it lets the compiler perform useful
optimizations! Some examples:

* keeping values in registers by proving no pointers access the value's memory
* eliminating reads by proving some memory hasn't been written to since last we read it
* eliminating writes by proving some memory is never read before the next write to it
* moving or reordering reads and writes by proving they don't depend on each other

These optimizations also tend to prove the soundness of bigger optimizations
such as loop vectorization, constant propagation, and dead code elimination.

In the previous example, we used the fact that `&mut u32` can't be aliased to prove
that writes to `*output` can't possibly affect `*input`. This lets us cache `*input`
in a register, eliminating a read.

By caching this read, we knew that the write in the `> 10` branch couldn't
affect whether we take the `> 5` branch, allowing us to also eliminate a
read-modify-write (doubling `*output`) when `*input > 10`.

The key thing to remember about alias analysis is that writes are the primary
hazard for optimizations. That is, the only thing that prevents us
from moving a read to any other part of the program is the possibility of us
re-ordering it with a write to the same location.

For instance, we have no concern for aliasing in the following modified version
of our function, because we've moved the only write to `*output` to the very
end of our function. This allows us to freely reorder the reads of `*input` that
occur before it:

```rust
fn compute(input: &u32, output: &mut u32) {
    let mut temp = *output;
    if *input > 10 {
        temp = 1;
    }
    if *input > 5 {
        temp *= 2;
    }
    *output = temp;
}
```

We're still relying on alias analysis to assume that `input` doesn't alias
`temp`, but the proof is much simpler: the value of a local variable can't be
aliased by things that existed before it was declared. This is an assumption
every language freely makes, and so this version of the function could be
optimized the way we want in any language.

This is why the definition of "alias" that Rust will use likely involves some
notion of liveness and mutation: we don't actually care if aliasing occurs if
there aren't any actual writes to memory happening.

Of course, a full aliasing model for Rust must also take into consideration things like
function calls (which may mutate things we don't see), raw pointers (which have
no aliasing requirements on their own), and UnsafeCell (which lets the referent
of an `&` be mutated).
