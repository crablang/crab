# Semihosting

Semihosting is a mechanism that lets embedded devices do I/O on the host and is
mainly used to log messages to the host console. Semihosting requires a debug
session and pretty much nothing else (no extra wires!) so it's super convenient
to use. The downside is that it's super slow: each write operation can take
several milliseconds depending on the hardware debugger (e.g. ST-Link) you use.

The [`cortex-m-semihosting`] crate provides an API to do semihosting operations
on Cortex-M devices. The program below is the semihosting version of "Hello,
world!":

[`cortex-m-semihosting`]: https://crates.io/crates/cortex-m-semihosting

```rust,ignore
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!").unwrap();

    loop {}
}
```

If you run this program on hardware you'll see the "Hello, world!" message
within the OpenOCD logs.

``` text
$ openocd
(..)
Hello, world!
(..)
```

You do need to enable semihosting in OpenOCD from GDB first:
``` console
(gdb) monitor arm semihosting enable
semihosting is enabled
```

QEMU understands semihosting operations so the above program will also work with
`qemu-system-arm` without having to start a debug session. Note that you'll
need to pass the `-semihosting-config` flag to QEMU to enable semihosting
support; these flags are already included in the `.cargo/config.toml` file of the
template.

``` text
$ # this program will block the terminal
$ cargo run
     Running `qemu-system-arm (..)
Hello, world!
```

There's also an `exit` semihosting operation that can be used to terminate the
QEMU process. Important: do **not** use `debug::exit` on hardware; this function
can corrupt your OpenOCD session and you will not be able to debug more programs
until you restart it.

```rust,ignore
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;

#[entry]
fn main() -> ! {
    let roses = "blue";

    if roses == "red" {
        debug::exit(debug::EXIT_SUCCESS);
    } else {
        debug::exit(debug::EXIT_FAILURE);
    }

    loop {}
}
```

``` text
$ cargo run
     Running `qemu-system-arm (..)

$ echo $?
1
```

One last tip: you can set the panicking behavior to `exit(EXIT_FAILURE)`. This
will let you write `no_std` run-pass tests that you can run on QEMU.

For convenience, the `panic-semihosting` crate has an "exit" feature that when
enabled invokes `exit(EXIT_FAILURE)` after logging the panic message to the host
stderr.

```rust,ignore
#![no_main]
#![no_std]

use panic_semihosting as _; // features = ["exit"]

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;

#[entry]
fn main() -> ! {
    let roses = "blue";

    assert_eq!(roses, "red");

    loop {}
}
```

``` text
$ cargo run
     Running `qemu-system-arm (..)
panicked at 'assertion failed: `(left == right)`
  left: `"blue"`,
 right: `"red"`', examples/hello.rs:15:5

$ echo $?
1
```

**NOTE**: To enable this feature on `panic-semihosting`, edit your
`Cargo.toml` dependencies section where `panic-semihosting` is specified with:

``` toml
panic-semihosting = { version = "VERSION", features = ["exit"] }
```

where `VERSION` is the version desired. For more information on dependencies
features check the [`specifying dependencies`] section of the Cargo book.

[`specifying dependencies`]:
https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
