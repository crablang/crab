# #[panic_handler]

`#[panic_handler]` is used to define the behavior of `panic!` in `#![no_std]` applications.
The `#[panic_handler]` attribute must be applied to a function with signature `fn(&PanicInfo)
-> !` and such function must appear *once* in the dependency graph of a binary / dylib / cdylib
crate. The API of `PanicInfo` can be found in the [API docs].

[API docs]: ../core/panic/struct.PanicInfo.html

Given that `#![no_std]` applications have no *standard* output and that some `#![no_std]`
applications, e.g. embedded applications, need different panicking behaviors for development and for
release it can be helpful to have panic crates, crate that only contain a `#[panic_handler]`.
This way applications can easily swap the panicking behavior by simply linking to a different panic
crate.

Below is shown an example where an application has a different panicking behavior depending on
whether is compiled using the dev profile (`cargo build`) or using the release profile (`cargo build
--release`).

`panic-semihosting` crate -- log panic messages to the host stderr using semihosting:

<!-- ignore: simplified code -->
```rust,ignore
#![no_std]

use core::fmt::{Write, self};
use core::panic::PanicInfo;

struct HStderr {
    // ..
#     _0: (),
}
#
# impl HStderr {
#     fn new() -> HStderr { HStderr { _0: () } }
# }
#
# impl fmt::Write for HStderr {
#     fn write_str(&mut self, _: &str) -> fmt::Result { Ok(()) }
# }

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut host_stderr = HStderr::new();

    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    writeln!(host_stderr, "{}", info).ok();

    loop {}
}
```

`panic-halt` crate -- halt the thread on panic; messages are discarded:

<!-- ignore: simplified code -->
```rust,ignore
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

`app` crate:

<!-- ignore: requires the above crates -->
```rust,ignore
#![no_std]

// dev profile
#[cfg(debug_assertions)]
extern crate panic_semihosting;

// release profile
#[cfg(not(debug_assertions))]
extern crate panic_halt;

fn main() {
    // ..
}
```
