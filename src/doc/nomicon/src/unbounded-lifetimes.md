# Unbounded Lifetimes

Unsafe code can often end up producing references or lifetimes out of thin air.
Such lifetimes come into the world as *unbounded*. The most common source of
this is taking a reference to a dereferenced raw pointer, which produces a
reference with an unbounded lifetime. Such a lifetime becomes as big as context
demands. This is in fact more powerful than simply becoming `'static`, because
for instance `&'static &'a T` will fail to typecheck, but the unbound lifetime
will perfectly mold into `&'a &'a T` as needed. However for most intents and
purposes, such an unbounded lifetime can be regarded as `'static`.

Almost no reference is `'static`, so this is probably wrong. `transmute` and
`transmute_copy` are the two other primary offenders. One should endeavor to
bound an unbounded lifetime as quickly as possible, especially across function
boundaries.

Given a function, any output lifetimes that don't derive from inputs are
unbounded. For instance:

<!-- no_run: This example exhibits undefined behavior. -->
```rust,no_run
fn get_str<'a>(s: *const String) -> &'a str {
    unsafe { &*s }
}

fn main() {
    let soon_dropped = String::from("hello");
    let dangling = get_str(&soon_dropped);
    drop(soon_dropped);
    println!("Invalid str: {}", dangling); // Invalid str: gӚ_`
}
```

The easiest way to avoid unbounded lifetimes is to use lifetime elision at the
function boundary. If an output lifetime is elided, then it *must* be bounded by
an input lifetime. Of course it might be bounded by the *wrong* lifetime, but
this will usually just cause a compiler error, rather than allow memory safety
to be trivially violated.

Within a function, bounding lifetimes is more error-prone. The safest and easiest
way to bound a lifetime is to return it from a function with a bound lifetime.
However if this is unacceptable, the reference can be placed in a location with
a specific lifetime. Unfortunately it's impossible to name all lifetimes involved
in a function.
