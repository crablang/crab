# Exceptions

Exceptions, and interrupts, are a hardware mechanism by which the processor
handles asynchronous events and fatal errors (e.g. executing an invalid
instruction). Exceptions imply preemption and involve exception handlers,
subroutines executed in response to the signal that triggered the event.

The `cortex-m-rt` crate provides an [`exception`] attribute to declare exception
handlers.

[`exception`]: https://docs.rs/cortex-m-rt-macros/latest/cortex_m_rt_macros/attr.exception.html

``` rust,ignore
// Exception handler for the SysTick (System Timer) exception
#[exception]
fn SysTick() {
    // ..
}
```

Other than the `exception` attribute exception handlers look like plain
functions but there's one more difference: `exception` handlers can *not* be
called by software. Following the previous example, the statement `SysTick();`
would result in a compilation error.

This behavior is pretty much intended and it's required to provide a feature:
`static mut` variables declared *inside* `exception` handlers are *safe* to use.

``` rust,ignore
#[exception]
fn SysTick() {
    static mut COUNT: u32 = 0;

    // `COUNT` has transformed to type `&mut u32` and it's safe to use
    *COUNT += 1;
}
```

As you may know, using `static mut` variables in a function makes it
[*non-reentrant*](https://en.wikipedia.org/wiki/Reentrancy_(computing)). It's undefined behavior to call a non-reentrant function,
directly or indirectly, from more than one exception / interrupt handler or from
`main` and one or more exception / interrupt handlers.

Safe Rust must never result in undefined behavior so non-reentrant functions
must be marked as `unsafe`. Yet I just told that `exception` handlers can safely
use `static mut` variables. How is this possible? This is possible because
`exception` handlers can *not* be called by software thus reentrancy is not
possible.

> Note that the `exception` attribute transforms definitions of static variables
> inside the function by wrapping them into `unsafe` blocks and providing us
> with new appropriate variables of type `&mut` of the same name.
> Thus we can dereference the reference via `*` to access the values of the variables without
> needing to wrap them in an `unsafe` block.

## A complete example

Here's an example that uses the system timer to raise a `SysTick` exception
roughly every second. The `SysTick` exception handler keeps track of how many
times it has been called in the `COUNT` variable and then prints the value of
`COUNT` to the host console using semihosting.

> **NOTE**: You can run this example on any Cortex-M device; you can also run it
> on QEMU

```rust,ignore
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_halt as _;

use core::fmt::Write;

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::{
    debug,
    hio::{self, HStdout},
};

#[entry]
fn main() -> ! {
    let p = cortex_m::Peripherals::take().unwrap();
    let mut syst = p.SYST;

    // configures the system timer to trigger a SysTick exception every second
    syst.set_clock_source(SystClkSource::Core);
    // this is configured for the LM3S6965 which has a default CPU clock of 12 MHz
    syst.set_reload(12_000_000);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();

    loop {}
}

#[exception]
fn SysTick() {
    static mut COUNT: u32 = 0;
    static mut STDOUT: Option<HStdout> = None;

    *COUNT += 1;

    // Lazy initialization
    if STDOUT.is_none() {
        *STDOUT = hio::hstdout().ok();
    }

    if let Some(hstdout) = STDOUT.as_mut() {
        write!(hstdout, "{}", *COUNT).ok();
    }

    // IMPORTANT omit this `if` block if running on real hardware or your
    // debugger will end in an inconsistent state
    if *COUNT == 9 {
        // This will terminate the QEMU process
        debug::exit(debug::EXIT_SUCCESS);
    }
}
```

``` console
tail -n5 Cargo.toml
```

``` toml
[dependencies]
cortex-m = "0.5.7"
cortex-m-rt = "0.6.3"
panic-halt = "0.2.0"
cortex-m-semihosting = "0.3.1"
```

``` text
$ cargo run --release
     Running `qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb (..)
123456789
```

If you run this on the Discovery board you'll see the output on the OpenOCD
console. Also, the program will *not* stop when the count reaches 9.

## The default exception handler

What the `exception` attribute actually does is *override* the default exception
handler for a specific exception. If you don't override the handler for a
particular exception it will be handled by the `DefaultHandler` function, which
defaults to:

``` rust,ignore
fn DefaultHandler() {
    loop {}
}
```

This function is provided by the `cortex-m-rt` crate and marked as
`#[no_mangle]` so you can put a breakpoint on "DefaultHandler" and catch
*unhandled* exceptions.

It's possible to override this `DefaultHandler` using the `exception` attribute:

``` rust,ignore
#[exception]
fn DefaultHandler(irqn: i16) {
    // custom default handler
}
```

The `irqn` argument indicates which exception is being serviced. A negative
value indicates that a Cortex-M exception is being serviced; and zero or a
positive value indicate that a device specific exception, AKA interrupt, is
being serviced.

## The hard fault handler

The `HardFault` exception is a bit special. This exception is fired when the
program enters an invalid state so its handler can *not* return as that could
result in undefined behavior. Also, the runtime crate does a bit of work before
the user defined `HardFault` handler is invoked to improve debuggability.

The result is that the `HardFault` handler must have the following signature:
`fn(&ExceptionFrame) -> !`. The argument of the handler is a pointer to
registers that were pushed into the stack by the exception. These registers are
a snapshot of the processor state at the moment the exception was triggered and
are useful to diagnose a hard fault.

Here's an example that performs an illegal operation: a read to a nonexistent
memory location.

> **NOTE**: This program won't work, i.e. it won't crash, on QEMU because
> `qemu-system-arm -machine lm3s6965evb` doesn't check memory loads and will
> happily return `0 `on reads to invalid memory.

```rust,ignore
#![no_main]
#![no_std]

use panic_halt as _;

use core::fmt::Write;
use core::ptr;

use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::hio;

#[entry]
fn main() -> ! {
    // read a nonexistent memory location
    unsafe {
        ptr::read_volatile(0x3FFF_FFFE as *const u32);
    }

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    if let Ok(mut hstdout) = hio::hstdout() {
        writeln!(hstdout, "{:#?}", ef).ok();
    }

    loop {}
}
```

The `HardFault` handler prints the `ExceptionFrame` value. If you run this
you'll see something like this on the OpenOCD console.

``` text
$ openocd
(..)
ExceptionFrame {
    r0: 0x3ffffffe,
    r1: 0x00f00000,
    r2: 0x20000000,
    r3: 0x00000000,
    r12: 0x00000000,
    lr: 0x080008f7,
    pc: 0x0800094a,
    xpsr: 0x61000000
}
```

The `pc` value is the value of the Program Counter at the time of the exception
and it points to the instruction that triggered the exception.

If you look at the disassembly of the program:


``` text
$ cargo objdump --bin app --release -- -d --no-show-raw-insn --print-imm-hex
(..)
ResetTrampoline:
 8000942:       movw    r0, #0xfffe
 8000946:       movt    r0, #0x3fff
 800094a:       ldr     r0, [r0]
 800094c:       b       #-0x4 <ResetTrampoline+0xa>
```

You can lookup the value of the program counter `0x0800094a` in the disassembly.
You'll see that a load operation (`ldr r0, [r0]` ) caused the exception.
The `r0` field of `ExceptionFrame` will tell you the value of register `r0`
was `0x3fff_fffe` at that time.
