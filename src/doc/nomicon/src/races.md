# Data Races and Race Conditions

Safe Rust guarantees an absence of data races, which are defined as:

* two or more threads concurrently accessing a location of memory
* one or more of them is a write
* one or more of them is unsynchronized

A data race has Undefined Behavior, and is therefore impossible to perform in
Safe Rust. Data races are *mostly* prevented through Rust's ownership system:
it's impossible to alias a mutable reference, so it's impossible to perform a
data race. Interior mutability makes this more complicated, which is largely why
we have the Send and Sync traits (see the next section for more on this).

**However Rust does not prevent general race conditions.**

This is mathematically impossible in situations where you do not control the
scheduler, which is true for the normal OS environment. If you do control
preemption, it _can be_ possible to prevent general races - this technique is
used by frameworks such as [RTIC](https://github.com/rtic-rs/rtic). However,
actually having control over scheduling is a very uncommon case.

For this reason, it is considered "safe" for Rust to get deadlocked or do
something nonsensical with incorrect synchronization: this is known as a general
race condition or resource race. Obviously such a program isn't very good, but
Rust of course cannot prevent all logic errors.

In any case, a race condition cannot violate memory safety in a Rust program on
its own. Only in conjunction with some other unsafe code can a race condition
actually violate memory safety. For instance, a correct program looks like this:

```rust,no_run
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

let data = vec![1, 2, 3, 4];
// Arc so that the memory the AtomicUsize is stored in still exists for
// the other thread to increment, even if we completely finish executing
// before it. Rust won't compile the program without it, because of the
// lifetime requirements of thread::spawn!
let idx = Arc::new(AtomicUsize::new(0));
let other_idx = idx.clone();

// `move` captures other_idx by-value, moving it into this thread
thread::spawn(move || {
    // It's ok to mutate idx because this value
    // is an atomic, so it can't cause a Data Race.
    other_idx.fetch_add(10, Ordering::SeqCst);
});

// Index with the value loaded from the atomic. This is safe because we
// read the atomic memory only once, and then pass a copy of that value
// to the Vec's indexing implementation. This indexing will be correctly
// bounds checked, and there's no chance of the value getting changed
// in the middle. However our program may panic if the thread we spawned
// managed to increment before this ran. A race condition because correct
// program execution (panicking is rarely correct) depends on order of
// thread execution.
println!("{}", data[idx.load(Ordering::SeqCst)]);
```

We can cause a data race if we instead do the bound check in advance, and then
unsafely access the data with an unchecked value:

```rust,no_run
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

let data = vec![1, 2, 3, 4];

let idx = Arc::new(AtomicUsize::new(0));
let other_idx = idx.clone();

// `move` captures other_idx by-value, moving it into this thread
thread::spawn(move || {
    // It's ok to mutate idx because this value
    // is an atomic, so it can't cause a Data Race.
    other_idx.fetch_add(10, Ordering::SeqCst);
});

if idx.load(Ordering::SeqCst) < data.len() {
    unsafe {
        // Incorrectly loading the idx after we did the bounds check.
        // It could have changed. This is a race condition, *and dangerous*
        // because we decided to do `get_unchecked`, which is `unsafe`.
        println!("{}", data.get_unchecked(idx.load(Ordering::SeqCst)));
    }
}
```
