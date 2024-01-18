# Tooling

Dealing with microcontrollers involves using several different tools as we'll be
dealing with an architecture different than your laptop's and we'll have to run
and debug programs on a *remote* device.

We'll use all the tools listed below. Any recent version should work when a
minimum version is not specified, but we have listed the versions we have
tested.

- Rust 1.31, 1.31-beta, or a newer toolchain PLUS ARM Cortex-M compilation
  support.
- [`cargo-binutils`](https://github.com/rust-embedded/cargo-binutils) ~0.1.4
- [`qemu-system-arm`](https://www.qemu.org/). Tested versions: 3.0.0
- OpenOCD >=0.8. Tested versions: v0.9.0 and v0.10.0
- GDB with ARM support. Version 7.12 or newer highly recommended. Tested
  versions: 7.10, 7.11, 7.12 and 8.1
- [`cargo-generate`](https://github.com/ashleygwilliams/cargo-generate) or `git`.
  These tools are optional but will make it easier to follow along with the book.

The text below explains why we are using these tools. Installation instructions
can be found on the next page.

## `cargo-generate` OR `git`

Bare metal programs are non-standard (`no_std`) Rust programs that require some
adjustments to the linking process in order to get the memory layout of the program
right. This requires some additional files (like linker scripts) and 
settings (like linker flags). We have packaged those for you in a template
such that you only need to fill in the missing information (such as the project name and the
characteristics of your target hardware).

Our template is compatible with `cargo-generate`: a Cargo subcommand for
creating new Cargo projects from templates. You can also download the
template using `git`, `curl`, `wget`, or your web browser.

## `cargo-binutils`

`cargo-binutils` is a collection of Cargo subcommands that make it easy to use
the LLVM tools that are shipped with the Rust toolchain. These tools include the
LLVM versions of `objdump`, `nm` and `size` and are used for inspecting
binaries.

The advantage of using these tools over GNU binutils is that (a) installing the
LLVM tools is the same one-command installation (`rustup component add
llvm-tools-preview`) regardless of your OS and (b) tools like `objdump` support
all the architectures that `rustc` supports -- from ARM to x86_64 -- because
they both share the same LLVM backend.

## `qemu-system-arm`

QEMU is an emulator. In this case we use the variant that can fully emulate ARM
systems. We use QEMU to run embedded programs on the host. Thanks to this you
can follow some parts of this book even if you don't have any hardware with you!

## GDB

A debugger is a very important component of embedded development as you may not
always have the luxury to log stuff to the host console. In some cases, you may
not even have LEDs to blink on your hardware!

In general, LLDB works as well as GDB when it comes to debugging but we haven't
found an LLDB counterpart to GDB's `load` command, which uploads the program to
the target hardware, so currently we recommend that you use GDB.

## OpenOCD

GDB isn't able to communicate directly with the ST-Link debugging hardware on
your STM32F3DISCOVERY development board. It needs a translator and the Open
On-Chip Debugger, OpenOCD, is that translator. OpenOCD is a program that runs
on your laptop/PC and translates between GDB's TCP/IP based remote debug
protocol and ST-Link's USB based protocol.

OpenOCD also performs other important work as part of its translation for the
debugging of the ARM Cortex-M based microcontroller on your STM32F3DISCOVERY
development board:
* It knows how to interact with the memory mapped registers used by the ARM
  CoreSight debug peripheral. It is these CoreSight registers that allow for:
  * Breakpoint/Watchpoint manipulation
  * Reading and writing of the CPU registers
  * Detecting when the CPU has been halted for a debug event
  * Continuing CPU execution after a debug event has been encountered
  * etc.
* It also knows how to erase and write to the microcontroller's FLASH
