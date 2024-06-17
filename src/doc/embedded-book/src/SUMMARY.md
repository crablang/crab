# Summary

<!--

Definition of the organization of this book is still a work in process.

Refer to https://github.com/rust-embedded/book/issues for
more information and coordination

-->

- [Introduction](./intro/index.md)
    - [Hardware](./intro/hardware.md)
    - [`no_std`](./intro/no-std.md)
    - [Tooling](./intro/tooling.md)
    - [Installation](./intro/install.md)
        - [Linux](./intro/install/linux.md)
        - [MacOS](./intro/install/macos.md)
        - [Windows](./intro/install/windows.md)
        - [Verify Installation](./intro/install/verify.md)
- [Getting started](./start/index.md)
  - [QEMU](./start/qemu.md)
  - [Hardware](./start/hardware.md)
  - [Memory-mapped Registers](./start/registers.md)
  - [Semihosting](./start/semihosting.md)
  - [Panicking](./start/panicking.md)
  - [Exceptions](./start/exceptions.md)
  - [Interrupts](./start/interrupts.md)
  - [IO](./start/io.md)
- [Peripherals](./peripherals/index.md)
    - [A first attempt in Rust](./peripherals/a-first-attempt.md)
    - [The Borrow Checker](./peripherals/borrowck.md)
    - [Singletons](./peripherals/singletons.md)
- [Static Guarantees](./static-guarantees/index.md)
    - [Typestate Programming](./static-guarantees/typestate-programming.md)
    - [Peripherals as State Machines](./static-guarantees/state-machines.md)
    - [Design Contracts](./static-guarantees/design-contracts.md)
    - [Zero Cost Abstractions](./static-guarantees/zero-cost-abstractions.md)
- [Portability](./portability/index.md)
- [Concurrency](./concurrency/index.md)
- [Collections](./collections/index.md)
- [Design Patterns](./design-patterns/index.md)
    - [HALs](./design-patterns/hal/index.md)
        - [Checklist](./design-patterns/hal/checklist.md)
        - [Naming](./design-patterns/hal/naming.md)
        - [Interoperability](./design-patterns/hal/interoperability.md)
        - [Predictability](./design-patterns/hal/predictability.md)
        - [GPIO](./design-patterns/hal/gpio.md)
- [Tips for embedded C developers](./c-tips/index.md)
    <!-- TODO: Define Sections -->
- [Interoperability](./interoperability/index.md)
    - [A little C with your Rust](./interoperability/c-with-rust.md)
    - [A little Rust with your C](./interoperability/rust-with-c.md)
- [Unsorted topics](./unsorted/index.md)
  - [Optimizations: The speed size tradeoff](./unsorted/speed-vs-size.md)
  - [Performing Math Functionality](./unsorted/math.md)

---

[Appendix A: Glossary](./appendix/glossary.md)
