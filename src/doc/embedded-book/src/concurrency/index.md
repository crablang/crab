# Concurrency

Concurrency happens whenever different parts of your program might execute
at different times or out of order. In an embedded context, this includes:

* interrupt handlers, which run whenever the associated interrupt happens,
* various forms of multithreading, where your microprocessor regularly swaps
  between parts of your program,
* and in some systems, multiple-core microprocessors, where each core can be
  independently running a different part of your program at the same time.

Since many embedded programs need to deal with interrupts, concurrency will
usually come up sooner or later, and it's also where many subtle and difficult
bugs can occur. Luckily, Rust provides a number of abstractions and safety
guarantees to help us write correct code.

## No Concurrency

The simplest concurrency for an embedded program is no concurrency: your
software consists of a single main loop which just keeps running, and there
are no interrupts at all. Sometimes this is perfectly suited to the problem
at hand! Typically your loop will read some inputs, perform some processing,
and write some outputs.

```rust,ignore
#[entry]
fn main() {
    let peripherals = setup_peripherals();
    loop {
        let inputs = read_inputs(&peripherals);
        let outputs = process(inputs);
        write_outputs(&peripherals, outputs);
    }
}
```

Since there's no concurrency, there's no need to worry about sharing data
between parts of your program or synchronising access to peripherals. If
you can get away with such a simple approach this can be a great solution.

## Global Mutable Data

Unlike non-embedded Rust, we will not usually have the luxury of creating
heap allocations and passing references to that data into a newly-created
thread. Instead, our interrupt handlers might be called at any time and must
know how to access whatever shared memory we are using. At the lowest level,
this means we must have _statically allocated_ mutable memory, which
both the interrupt handler and the main code can refer to.

In Rust, such [`static mut`] variables are always unsafe to read or write,
because without taking special care, you might trigger a race condition,
where your access to the variable is interrupted halfway through by an
interrupt which also accesses that variable.

[`static mut`]: https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#accessing-or-modifying-a-mutable-static-variable

For an example of how this behaviour can cause subtle errors in your code,
consider an embedded program which counts rising edges of some input signal
in each one-second period (a frequency counter):

```rust,ignore
static mut COUNTER: u32 = 0;

#[entry]
fn main() -> ! {
    set_timer_1hz();
    let mut last_state = false;
    loop {
        let state = read_signal_level();
        if state && !last_state {
            // DANGER - Not actually safe! Could cause data races.
            unsafe { COUNTER += 1 };
        }
        last_state = state;
    }
}

#[interrupt]
fn timer() {
    unsafe { COUNTER = 0; }
}
```

Each second, the timer interrupt sets the counter back to 0. Meanwhile, the
main loop continually measures the signal, and incremements the counter when
it sees a change from low to high. We've had to use `unsafe` to access
`COUNTER`, as it's `static mut`, and that means we're promising the compiler
we won't cause any undefined behaviour. Can you spot the race condition? The
increment on `COUNTER` is _not_ guaranteed to be atomic — in fact, on most
embedded platforms, it will be split into a load, then the increment, then
a store. If the interrupt fired after the load but before the store, the
reset back to 0 would be ignored after the interrupt returns — and we would
count twice as many transitions for that period.

## Critical Sections

So, what can we do about data races? A simple approach is to use _critical
sections_, a context where interrupts are disabled. By wrapping the access to
`COUNTER` in `main` in a critical section, we can be sure the timer interrupt
will not fire until we're finished incrementing `COUNTER`:

```rust,ignore
static mut COUNTER: u32 = 0;

#[entry]
fn main() -> ! {
    set_timer_1hz();
    let mut last_state = false;
    loop {
        let state = read_signal_level();
        if state && !last_state {
            // New critical section ensures synchronised access to COUNTER
            cortex_m::interrupt::free(|_| {
                unsafe { COUNTER += 1 };
            });
        }
        last_state = state;
    }
}

#[interrupt]
fn timer() {
    unsafe { COUNTER = 0; }
}
```

In this example, we use `cortex_m::interrupt::free`, but other platforms will
have similar mechanisms for executing code in a critical section. This is also
the same as disabling interrupts, running some code, and then re-enabling
interrupts.

Note we didn't need to put a critical section inside the timer interrupt,
for two reasons:

  * Writing 0 to `COUNTER` can't be affected by a race since we don't read it
  * It will never be interrupted by the `main` thread anyway

If `COUNTER` was being shared by multiple interrupt handlers that might
_preempt_ each other, then each one might require a critical section as well.

This solves our immediate problem, but we're still left writing a lot of unsafe code which we need to carefully reason about, and we might be using critical sections needlessly. Since each critical section temporarily pauses interrupt processing, there is an associated cost of some extra code size and higher interrupt latency and jitter (interrupts may take longer to be processed, and the time until they are processed will be more variable). Whether this is a problem depends on your system, but in general, we'd like to avoid it.

It's worth noting that while a critical section guarantees no interrupts will
fire, it does not provide an exclusivity guarantee on multi-core systems!  The
other core could be happily accessing the same memory as your core, even
without interrupts. You will need stronger synchronisation primitives if you
are using multiple cores.

## Atomic Access

On some platforms, special atomic instructions are available, which provide
guarantees about read-modify-write operations. Specifically for Cortex-M: `thumbv6`
(Cortex-M0, Cortex-M0+) only provide atomic load and store instructions,
while `thumbv7` (Cortex-M3 and above) provide full Compare and Swap (CAS)
instructions. These CAS instructions give an alternative to the heavy-handed
disabling of all interrupts: we can attempt the increment, it will succeed most
of the time, but if it was interrupted it will automatically retry the entire
increment operation. These atomic operations are safe even across multiple
cores.

```rust,ignore
use core::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[entry]
fn main() -> ! {
    set_timer_1hz();
    let mut last_state = false;
    loop {
        let state = read_signal_level();
        if state && !last_state {
            // Use `fetch_add` to atomically add 1 to COUNTER
            COUNTER.fetch_add(1, Ordering::Relaxed);
        }
        last_state = state;
    }
}

#[interrupt]
fn timer() {
    // Use `store` to write 0 directly to COUNTER
    COUNTER.store(0, Ordering::Relaxed)
}
```

This time `COUNTER` is a safe `static` variable. Thanks to the `AtomicUsize`
type `COUNTER` can be safely modified from both the interrupt handler and the
main thread without disabling interrupts. When possible, this is a better
solution — but it may not be supported on your platform.

A note on [`Ordering`]: this affects how the compiler and hardware may reorder
instructions, and also has consequences on cache visibility. Assuming that the
target is a single core platform `Relaxed` is sufficient and the most efficient
choice in this particular case. Stricter ordering will cause the compiler to
emit memory barriers around the atomic operations; depending on what you're
using atomics for you may or may not need this! The precise details of the
atomic model are complicated and best described elsewhere.

For more details on atomics and ordering, see the [nomicon].

[`Ordering`]: https://doc.rust-lang.org/core/sync/atomic/enum.Ordering.html
[nomicon]: https://doc.rust-lang.org/nomicon/atomics.html


## Abstractions, Send, and Sync

None of the above solutions are especially satisfactory. They require `unsafe`
blocks which must be very carefully checked and are not ergonomic. Surely we
can do better in Rust!

We can abstract our counter into a safe interface which can be safely used
anywhere else in our code. For this example, we'll use the critical-section
counter, but you could do something very similar with atomics.

```rust,ignore
use core::cell::UnsafeCell;
use cortex_m::interrupt;

// Our counter is just a wrapper around UnsafeCell<u32>, which is the heart
// of interior mutability in Rust. By using interior mutability, we can have
// COUNTER be `static` instead of `static mut`, but still able to mutate
// its counter value.
struct CSCounter(UnsafeCell<u32>);

const CS_COUNTER_INIT: CSCounter = CSCounter(UnsafeCell::new(0));

impl CSCounter {
    pub fn reset(&self, _cs: &interrupt::CriticalSection) {
        // By requiring a CriticalSection be passed in, we know we must
        // be operating inside a CriticalSection, and so can confidently
        // use this unsafe block (required to call UnsafeCell::get).
        unsafe { *self.0.get() = 0 };
    }

    pub fn increment(&self, _cs: &interrupt::CriticalSection) {
        unsafe { *self.0.get() += 1 };
    }
}

// Required to allow static CSCounter. See explanation below.
unsafe impl Sync for CSCounter {}

// COUNTER is no longer `mut` as it uses interior mutability;
// therefore it also no longer requires unsafe blocks to access.
static COUNTER: CSCounter = CS_COUNTER_INIT;

#[entry]
fn main() -> ! {
    set_timer_1hz();
    let mut last_state = false;
    loop {
        let state = read_signal_level();
        if state && !last_state {
            // No unsafe here!
            interrupt::free(|cs| COUNTER.increment(cs));
        }
        last_state = state;
    }
}

#[interrupt]
fn timer() {
    // We do need to enter a critical section here just to obtain a valid
    // cs token, even though we know no other interrupt could pre-empt
    // this one.
    interrupt::free(|cs| COUNTER.reset(cs));

    // We could use unsafe code to generate a fake CriticalSection if we
    // really wanted to, avoiding the overhead:
    // let cs = unsafe { interrupt::CriticalSection::new() };
}
```

We've moved our `unsafe` code to inside our carefully-planned abstraction,
and now our application code does not contain any `unsafe` blocks.

This design requires that the application pass a `CriticalSection` token in:
these tokens are only safely generated by `interrupt::free`, so by requiring
one be passed in, we ensure we are operating inside a critical section, without
having to actually do the lock ourselves. This guarantee is provided statically
by the compiler: there won't be any runtime overhead associated with `cs`.
If we had multiple counters, they could all be given the same `cs`, without
requiring multiple nested critical sections.

This also brings up an important topic for concurrency in Rust: the
[`Send` and `Sync`] traits. To summarise the Rust book, a type is Send
when it can safely be moved to another thread, while it is Sync when
it can be safely shared between multiple threads. In an embedded context,
we consider interrupts to be executing in a separate thread to the application
code, so variables accessed by both an interrupt and the main code must be
Sync.

[`Send` and `Sync`]: https://doc.rust-lang.org/nomicon/send-and-sync.html

For most types in Rust, both of these traits are automatically derived for you
by the compiler. However, because `CSCounter` contains an [`UnsafeCell`], it is
not Sync, and therefore we could not make a `static CSCounter`: `static`
variables _must_ be Sync, since they can be accessed by multiple threads.

[`UnsafeCell`]: https://doc.rust-lang.org/core/cell/struct.UnsafeCell.html

To tell the compiler we have taken care that the `CSCounter` is in fact safe
to share between threads, we implement the Sync trait explicitly. As with the
previous use of critical sections, this is only safe on single-core platforms:
with multiple cores, you would need to go to greater lengths to ensure safety.

## Mutexes

We've created a useful abstraction specific to our counter problem, but
there are many common abstractions used for concurrency.

One such _synchronisation primitive_ is a mutex, short for mutual exclusion.
These constructs ensure exclusive access to a variable, such as our counter. A
thread can attempt to _lock_ (or _acquire_) the mutex, and either succeeds
immediately, or blocks waiting for the lock to be acquired, or returns an error
that the mutex could not be locked. While that thread holds the lock, it is
granted access to the protected data. When the thread is done, it _unlocks_ (or
_releases_) the mutex, allowing another thread to lock it. In Rust, we would
usually implement the unlock using the [`Drop`] trait to ensure it is always
released when the mutex goes out of scope.

[`Drop`]: https://doc.rust-lang.org/core/ops/trait.Drop.html

Using a mutex with interrupt handlers can be tricky: it is not normally
acceptable for the interrupt handler to block, and it would be especially
disastrous for it to block waiting for the main thread to release a lock,
since we would then _deadlock_ (the main thread will never release the lock
because execution stays in the interrupt handler). Deadlocking is not
considered unsafe: it is possible even in safe Rust.

To avoid this behaviour entirely, we could implement a mutex which requires
a critical section to lock, just like our counter example. So long as the
critical section must last as long as the lock, we can be sure we have
exclusive access to the wrapped variable without even needing to track
the lock/unlock state of the mutex.

This is in fact done for us in the `cortex_m` crate! We could have written
our counter using it:

```rust,ignore
use core::cell::Cell;
use cortex_m::interrupt::Mutex;

static COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[entry]
fn main() -> ! {
    set_timer_1hz();
    let mut last_state = false;
    loop {
        let state = read_signal_level();
        if state && !last_state {
            interrupt::free(|cs|
                COUNTER.borrow(cs).set(COUNTER.borrow(cs).get() + 1));
        }
        last_state = state;
    }
}

#[interrupt]
fn timer() {
    // We still need to enter a critical section here to satisfy the Mutex.
    interrupt::free(|cs| COUNTER.borrow(cs).set(0));
}
```

We're now using [`Cell`], which along with its sibling `RefCell` is used to
provide safe interior mutability. We've already seen `UnsafeCell` which is
the bottom layer of interior mutability in Rust: it allows you to obtain
multiple mutable references to its value, but only with unsafe code. A `Cell`
is like an `UnsafeCell` but it provides a safe interface: it only permits
taking a copy of the current value or replacing it, not taking a reference,
and since it is not Sync, it cannot be shared between threads. These
constraints mean it's safe to use, but we couldn't use it directly in a
`static` variable as a `static` must be Sync.

[`Cell`]: https://doc.rust-lang.org/core/cell/struct.Cell.html

So why does the example above work? The `Mutex<T>` implements Sync for any
`T` which is Send — such as a `Cell`. It can do this safely because it only
gives access to its contents during a critical section. We're therefore able
to get a safe counter with no unsafe code at all!

This is great for simple types like the `u32` of our counter, but what about
more complex types which are not Copy? An extremely common example in an
embedded context is a peripheral struct, which generally is not Copy.
For that, we can turn to `RefCell`.

## Sharing Peripherals

Device crates generated using `svd2rust` and similar abstractions provide
safe access to peripherals by enforcing that only one instance of the
peripheral struct can exist at a time. This ensures safety, but makes it
difficult to access a peripheral from both the main thread and an interrupt
handler.

To safely share peripheral access, we can use the `Mutex` we saw before. We'll
also need to use [`RefCell`], which uses a runtime check to ensure only one
reference to a peripheral is given out at a time. This has more overhead than
the plain `Cell`, but since we are giving out references rather than copies,
we must be sure only one exists at a time.

[`RefCell`]: https://doc.rust-lang.org/core/cell/struct.RefCell.html

Finally, we'll also have to account for somehow moving the peripheral into
the shared variable after it has been initialised in the main code. To do
this we can use the `Option` type, initialised to `None` and later set to
the instance of the peripheral.

```rust,ignore
use core::cell::RefCell;
use cortex_m::interrupt::{self, Mutex};
use stm32f4::stm32f405;

static MY_GPIO: Mutex<RefCell<Option<stm32f405::GPIOA>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Obtain the peripheral singletons and configure it.
    // This example is from an svd2rust-generated crate, but
    // most embedded device crates will be similar.
    let dp = stm32f405::Peripherals::take().unwrap();
    let gpioa = &dp.GPIOA;

    // Some sort of configuration function.
    // Assume it sets PA0 to an input and PA1 to an output.
    configure_gpio(gpioa);

    // Store the GPIOA in the mutex, moving it.
    interrupt::free(|cs| MY_GPIO.borrow(cs).replace(Some(dp.GPIOA)));
    // We can no longer use `gpioa` or `dp.GPIOA`, and instead have to
    // access it via the mutex.

    // Be careful to enable the interrupt only after setting MY_GPIO:
    // otherwise the interrupt might fire while it still contains None,
    // and as-written (with `unwrap()`), it would panic.
    set_timer_1hz();
    let mut last_state = false;
    loop {
        // We'll now read state as a digital input, via the mutex
        let state = interrupt::free(|cs| {
            let gpioa = MY_GPIO.borrow(cs).borrow();
            gpioa.as_ref().unwrap().idr.read().idr0().bit_is_set()
        });

        if state && !last_state {
            // Set PA1 high if we've seen a rising edge on PA0.
            interrupt::free(|cs| {
                let gpioa = MY_GPIO.borrow(cs).borrow();
                gpioa.as_ref().unwrap().odr.modify(|_, w| w.odr1().set_bit());
            });
        }
        last_state = state;
    }
}

#[interrupt]
fn timer() {
    // This time in the interrupt we'll just clear PA0.
    interrupt::free(|cs| {
        // We can use `unwrap()` because we know the interrupt wasn't enabled
        // until after MY_GPIO was set; otherwise we should handle the potential
        // for a None value.
        let gpioa = MY_GPIO.borrow(cs).borrow();
        gpioa.as_ref().unwrap().odr.modify(|_, w| w.odr1().clear_bit());
    });
}
```

That's quite a lot to take in, so let's break down the important lines.

```rust,ignore
static MY_GPIO: Mutex<RefCell<Option<stm32f405::GPIOA>>> =
    Mutex::new(RefCell::new(None));
```

Our shared variable is now a `Mutex` around a `RefCell` which contains an
`Option`. The `Mutex` ensures we only have access during a critical section,
and therefore makes the variable Sync, even though a plain `RefCell` would not
be Sync. The `RefCell` gives us interior mutability with references, which
we'll need to use our `GPIOA`. The `Option` lets us initialise this variable
to something empty, and only later actually move the variable in. We cannot
access the peripheral singleton statically, only at runtime, so this is
required.

```rust,ignore
interrupt::free(|cs| MY_GPIO.borrow(cs).replace(Some(dp.GPIOA)));
```

Inside a critical section we can call `borrow()` on the mutex, which gives us
a reference to the `RefCell`. We then call `replace()` to move our new value
into the `RefCell`.

```rust,ignore
interrupt::free(|cs| {
    let gpioa = MY_GPIO.borrow(cs).borrow();
    gpioa.as_ref().unwrap().odr.modify(|_, w| w.odr1().set_bit());
});
```

Finally, we use `MY_GPIO` in a safe and concurrent fashion. The critical section
prevents the interrupt firing as usual, and lets us borrow the mutex.  The
`RefCell` then gives us an `&Option<GPIOA>`, and tracks how long it remains
borrowed - once that reference goes out of scope, the `RefCell` will be updated
to indicate it is no longer borrowed.

Since we can't move the `GPIOA` out of the `&Option`, we need to convert it to
an `&Option<&GPIOA>` with `as_ref()`, which we can finally `unwrap()` to obtain
the `&GPIOA` which lets us modify the peripheral.

If we need a mutable reference to a shared resource, then `borrow_mut` and `deref_mut`
should be used instead. The following code shows an example using the TIM2 timer.

```rust,ignore
use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m::interrupt::{self, Mutex};
use cortex_m::asm::wfi;
use stm32f4::stm32f405;

static G_TIM: Mutex<RefCell<Option<Timer<stm32::TIM2>>>> =
	Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut cp = cm::Peripherals::take().unwrap();
    let dp = stm32f405::Peripherals::take().unwrap();

    // Some sort of timer configuration function.
    // Assume it configures the TIM2 timer, its NVIC interrupt,
    // and finally starts the timer.
    let tim = configure_timer_interrupt(&mut cp, dp);

    interrupt::free(|cs| {
        G_TIM.borrow(cs).replace(Some(tim));
    });

    loop {
        wfi();
    }
}

#[interrupt]
fn timer() {
    interrupt::free(|cs| {
        if let Some(ref mut tim)) =  G_TIM.borrow(cs).borrow_mut().deref_mut() {
            tim.start(1.hz());
        }
    });
}

```

Whew! This is safe, but it is also a little unwieldy. Is there anything else
we can do?

## RTIC

One alternative is the [RTIC framework], short for Real Time Interrupt-driven Concurrency. It
enforces static priorities and tracks accesses to `static mut` variables
("resources") to statically ensure that shared resources are always accessed
safely, without requiring the overhead of always entering critical sections and
using reference counting (as in `RefCell`). This has a number of advantages such
as guaranteeing no deadlocks and giving extremely low time and memory overhead.

[RTIC framework]: https://github.com/rtic-rs/cortex-m-rtic

The framework also includes other features like message passing, which reduces
the need for explicit shared state, and the ability to schedule tasks to run at
a given time, which can be used to implement periodic tasks. Check out [the
documentation] for more information!

[the documentation]: https://rtic.rs

## Real Time Operating Systems

Another common model for embedded concurrency is the real-time operating system
(RTOS). While currently less well explored in Rust, they are widely used in
traditional embedded development. Open source examples include [FreeRTOS] and
[ChibiOS]. These RTOSs provide support for running multiple application threads
which the CPU swaps between, either when the threads yield control (called
cooperative multitasking) or based on a regular timer or interrupts (preemptive
multitasking). The RTOS typically provide mutexes and other synchronisation
primitives, and often interoperate with hardware features such as DMA engines.

[FreeRTOS]: https://freertos.org/
[ChibiOS]: http://chibios.org/

At the time of writing, there are not many Rust RTOS examples to point to,
but it's an interesting area so watch this space!

## Multiple Cores

It is becoming more common to have two or more cores in embedded processors,
which adds an extra layer of complexity to concurrency. All the examples using
a critical section (including the `cortex_m::interrupt::Mutex`) assume the only
other execution thread is the interrupt thread, but on a multi-core system
that's no longer true. Instead, we'll need synchronisation primitives designed
for multiple cores (also called SMP, for symmetric multi-processing).

These typically use the atomic instructions we saw earlier, since the
processing system will ensure that atomicity is maintained over all cores.

Covering these topics in detail is currently beyond the scope of this book,
but the general patterns are the same as for the single-core case.
