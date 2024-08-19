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
llvm-tools`) regardless of your OS and (b) tools like `objdump` support
all the architectures that `rustc` supports -- from ARM to x86_64 -- because
they both share the same LLVM backend.

## `qemu-system-arm`

QEMU is an emulator. In this case we use the variant that can fully emulate ARM
systems. We use QEMU to run embedded programs on the host. Thanks to this you
can follow some parts of this book even if you don't have any hardware with you!

# Tooling for Embedded Rust Debugging

## Overview

Debugging embedded systems in Rust requires specialized tools including software to manage the debugging process, debuggers to inspect and control program execution, and hardware probes to facilitate interaction between the host and the embedded device. This document outlines essential software tools like Probe-rs and OpenOCD, which simplify and support the debugging process, alongside prominent debuggers such as GDB and the Probe-rs Visual Studio Code extension. Additionally, it covers key hardware probes such as Rusty-probe, ST-Link, J-Link, and MCU-Link, which are integral for effective debugging and programming of embedded devices.

## Software that drives debugging tools

### Probe-rs

Probe-rs is a modern, Rust-focused software designed to work with debuggers in embedded systems. Unlike OpenOCD, Probe-rs is built with simplicity in mind and aims to reduce the configuration burden often found in other debugging solutions. It supports various probes and targets, providing a high-level interface for interacting with embedded hardware. Probe-rs integrates directly with Rust tooling, and integrates with Visual Studio Code through its extension, allowing developers to streamline their debugging workflow.


### OpenOCD (Open On-Chip Debugger)

OpenOCD is an open-source software tool used for debugging, testing, and programming embedded systems. It provides an interface between the host system and embedded hardware, supporting various transport layers like JTAG and SWD (Serial Wire Debug). OpenOCD integrates with GDB, which is a debugger. OpenOCD is widely supported, with extensive documentation and a large community, but may require complex configuration, especially for custom embedded setups.

## Debuggers

A debugger allows developers to inspect and control the execution of a program in order to identify and correct errors or bugs. It provides functionalities such as setting breakpoints, stepping through code line by line, and examining the values of variables and memory states. Debuggers are essential for thorough software development and maintenance, enabling developers to ensure that their code behaves as intended under various conditions.

Debuggers know how to:
 * Interact with the memory mapped registers. 
 * Set Breakpoints/Watchpoints.
 * Read and write to the memory mapped registers.
 * Detect when the MCU has been halted for a debug event.
 * Continue MCU execution after a debug event has been encountered.
 * Erase and write to the microcontroller's FLASH.

### Probe-rs Visual Studio Code Extension

Probe-rs has a Visual Studio Code extension, providing a seamless debugging experience without extensive setup. Through this connection, developers can use Rust-specific features like pretty printing and detailed error messages, ensuring that their debugging process aligns with the Rust ecosystem. 

### GDB (GNU Debugger) 

GDB is a versatile debugging tool that allows developers to examine the state of programs while they run or after they crash. For embedded Rust, GDB connects to the target system via OpenOCD or other debugging servers to interact with the embedded code. GDB is highly configurable and supports features like remote debugging, variable inspection, and conditional breakpoints. It can be used on a variety of platforms, and has extensive support for Rust-specific debugging needs, such as pretty printing and integration with IDEs.


## Probes

A hardware probe is a device used in the development and debugging of embedded systems to facilitate communication between a host computer and the target embedded device. It typically supports protocols like JTAG or SWD, enabling it to program, debug, and analyze the microcontroller or microprocessor on the embedded system. Hardware probes are crucial for developers to set breakpoints, step through code, and inspect memory and processor registers, effectively allowing them to diagnose and fix issues in real-time.

### Rusty-probe

Rusty-probe is an open-sourced USB-based hardware debugging probe designed to work with probe-rs. The combination of Rusty-Probe and probe-rs provides an easy-to-use, cost-effective solution for developers working with embedded Rust applications.

### ST-Link

The ST-Link is a popular debugging and programming probe developed by STMicroelectronics primarily for their STM32 and STM8 microcontroller series. It supports both debugging and programming via JTAG or SWD (Serial Wire Debug) interfaces. ST-Link is widely used due to its direct support from STMicroelectronics' extensive range of development boards and its integration into major IDEs, making it a convenient choice for developers working with STM microcontrollers.

### J-Link

J-Link, developed by SEGGER Microcontroller, is a robust and versatile debugger supporting a wide range of CPU cores and devices beyond just ARM, such as RISC-V. Known for its high performance and reliability, J-Link supports various communication interfaces, including JTAG, SWD, and fine-pitch JTAG interfaces. It is favored for its advanced features like unlimited breakpoints in flash memory and its compatibility with a multitude of development environments.

### MCU-Link

MCU-Link is a debugging probe that also functions as a programmer, provided by NXP Semiconductors. It supports a variety of ARM Cortex microcontrollers and interfaces seamlessly with development tools like MCUXpresso IDE. MCU-Link is particularly notable for its versatility and affordability, making it an accessible option for hobbyists, educators, and professional developers alike.