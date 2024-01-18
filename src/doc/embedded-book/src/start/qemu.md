# QEMU

We'll start writing a program for the [LM3S6965], a Cortex-M3 microcontroller.
We have chosen this as our initial target because it [can be emulated](https://wiki.qemu.org/Documentation/Platforms/ARM#Supported_in_qemu-system-arm) using QEMU
so you don't need to fiddle with hardware in this section and we can focus on
the tooling and the development process.

[LM3S6965]: http://www.ti.com/product/LM3S6965

**IMPORTANT**
We'll use the name "app" for the project name in this tutorial.
Whenever you see the word "app" you should replace it with the name you selected
for your project. Or, you could also name your project "app" and avoid the
substitutions.

## Creating a non standard Rust program

We'll use the [`cortex-m-quickstart`] project template to generate a new
project from it. The created project will contain a barebone application: a good
starting point for a new embedded rust application. In addition, the project will
contain an `examples` directory, with several separate applications, highlighting
some of the key embedded rust functionality. 

[`cortex-m-quickstart`]: https://github.com/rust-embedded/cortex-m-quickstart

### Using `cargo-generate`
First install cargo-generate
```console
cargo install cargo-generate
```
Then generate a new project
```console
cargo generate --git https://github.com/rust-embedded/cortex-m-quickstart
```

```text
 Project Name: app
 Creating project called `app`...
 Done! New project created /tmp/app
```

```console
cd app
```

### Using `git`

Clone the repository

```console
git clone https://github.com/rust-embedded/cortex-m-quickstart app
cd app
```

And then fill in the placeholders in the `Cargo.toml` file

```toml
[package]
authors = ["{{authors}}"] # "{{authors}}" -> "John Smith"
edition = "2018"
name = "{{project-name}}" # "{{project-name}}" -> "app"
version = "0.1.0"

# ..

[[bin]]
name = "{{project-name}}" # "{{project-name}}" -> "app"
test = false
bench = false
```

### Using neither

Grab the latest snapshot of the `cortex-m-quickstart` template and extract it.

```console
curl -LO https://github.com/rust-embedded/cortex-m-quickstart/archive/master.zip
unzip master.zip
mv cortex-m-quickstart-master app
cd app
```

Or you can browse to [`cortex-m-quickstart`], click the green "Clone or
download" button and then click "Download ZIP".

Then fill in the placeholders in the `Cargo.toml` file as done in the second
part of the "Using `git`" version.

## Program Overview

For convenience here are the most important parts of the source code in `src/main.rs`:

```rust,ignore
#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {
        // your code goes here
    }
}
```

This program is a bit different from a standard Rust program so let's take a
closer look.

`#![no_std]` indicates that this program will *not* link to the standard crate,
`std`. Instead it will link to its subset: the `core` crate.

`#![no_main]` indicates that this program won't use the standard `main`
interface that most Rust programs use. The main (no pun intended) reason to go
with `no_main` is that using the `main` interface in `no_std` context requires
nightly.

`use panic_halt as _;`. This crate provides a `panic_handler` that defines
the panicking behavior of the program. We will cover this in more detail in the
[Panicking](panicking.md) chapter of the book.

[`#[entry]`][entry] is an attribute provided by the [`cortex-m-rt`] crate that's used
to mark the entry point of the program. As we are not using the standard `main`
interface we need another way to indicate the entry point of the program and
that'd be `#[entry]`.

[entry]: https://docs.rs/cortex-m-rt-macros/latest/cortex_m_rt_macros/attr.entry.html
[`cortex-m-rt`]: https://crates.io/crates/cortex-m-rt

`fn main() -> !`. Our program will be the *only* process running on the target
hardware so we don't want it to end! We use a [divergent function](https://doc.rust-lang.org/rust-by-example/fn/diverging.html) (the `-> !`
bit in the function signature) to ensure at compile time that'll be the case.

## Cross compiling

The next step is to *cross* compile the program for the Cortex-M3 architecture.
That's as simple as running `cargo build --target $TRIPLE` if you know what the
compilation target (`$TRIPLE`) should be. Luckily, the `.cargo/config.toml` in the
template has the answer:

```console
tail -n6 .cargo/config.toml
```

```toml
[build]
# Pick ONE of these compilation targets
# target = "thumbv6m-none-eabi"    # Cortex-M0 and Cortex-M0+
target = "thumbv7m-none-eabi"    # Cortex-M3
# target = "thumbv7em-none-eabi"   # Cortex-M4 and Cortex-M7 (no FPU)
# target = "thumbv7em-none-eabihf" # Cortex-M4F and Cortex-M7F (with FPU)
```

To cross compile for the Cortex-M3 architecture we have to use
`thumbv7m-none-eabi`. That target is not automatically installed when installing
the Rust toolchain, it would now be a good time to add that target to the toolchain,
if you haven't done it yet:
``` console
rustup target add thumbv7m-none-eabi
```
 Since the `thumbv7m-none-eabi` compilation target has been set as the default in 
 your `.cargo/config.toml` file, the two commands below do the same:

```console
cargo build --target thumbv7m-none-eabi
cargo build
```

## Inspecting

Now we have a non-native ELF binary in `target/thumbv7m-none-eabi/debug/app`. We
can inspect it using `cargo-binutils`.

With `cargo-readobj` we can print the ELF headers to confirm that this is an ARM
binary.

``` console
cargo readobj --bin app -- --file-headers
```

Note that:
* `--bin app` is sugar for inspect the binary at `target/$TRIPLE/debug/app`
* `--bin app` will also (re)compile the binary, if necessary


``` text
ELF Header:
  Magic:   7f 45 4c 46 01 01 01 00 00 00 00 00 00 00 00 00
  Class:                             ELF32
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI Version:                       0x0
  Type:                              EXEC (Executable file)
  Machine:                           ARM
  Version:                           0x1
  Entry point address:               0x405
  Start of program headers:          52 (bytes into file)
  Start of section headers:          153204 (bytes into file)
  Flags:                             0x5000200
  Size of this header:               52 (bytes)
  Size of program headers:           32 (bytes)
  Number of program headers:         2
  Size of section headers:           40 (bytes)
  Number of section headers:         19
  Section header string table index: 18
```

`cargo-size` can print the size of the linker sections of the binary.


```console
cargo size --bin app --release -- -A
```
we use `--release` to inspect the optimized version

``` text
app  :
section             size        addr
.vector_table       1024         0x0
.text                 92       0x400
.rodata                0       0x45c
.data                  0  0x20000000
.bss                   0  0x20000000
.debug_str          2958         0x0
.debug_loc            19         0x0
.debug_abbrev        567         0x0
.debug_info         4929         0x0
.debug_ranges         40         0x0
.debug_macinfo         1         0x0
.debug_pubnames     2035         0x0
.debug_pubtypes     1892         0x0
.ARM.attributes       46         0x0
.debug_frame         100         0x0
.debug_line          867         0x0
Total              14570
```

> A refresher on ELF linker sections
>
> - `.text` contains the program instructions
> - `.rodata` contains constant values like strings
> - `.data` contains statically allocated variables whose initial values are
>   *not* zero
> - `.bss` also contains statically allocated variables whose initial values
>   *are* zero
> - `.vector_table` is a *non*-standard section that we use to store the vector
>   (interrupt) table
> - `.ARM.attributes` and the `.debug_*` sections contain metadata and will
>   *not* be loaded onto the target when flashing the binary.

**IMPORTANT**: ELF files contain metadata like debug information so their *size
on disk* does *not* accurately reflect the space the program will occupy when
flashed on a device. *Always* use `cargo-size` to check how big a binary really
is.

`cargo-objdump` can be used to disassemble the binary.

```console
cargo objdump --bin app --release -- --disassemble --no-show-raw-insn --print-imm-hex
```

> **NOTE** if the above command complains about `Unknown command line argument` see
> the following bug report: https://github.com/rust-embedded/book/issues/269

> **NOTE** this output can differ on your system. New versions of rustc, LLVM
> and libraries can generate different assembly. We truncated some of the instructions
> to keep the snippet small.

```text
app:  file format ELF32-arm-little

Disassembly of section .text:
main:
     400: bl  #0x256
     404: b #-0x4 <main+0x4>

Reset:
     406: bl  #0x24e
     40a: movw  r0, #0x0
     < .. truncated any more instructions .. >

DefaultHandler_:
     656: b #-0x4 <DefaultHandler_>

UsageFault:
     657: strb  r7, [r4, #0x3]

DefaultPreInit:
     658: bx  lr

__pre_init:
     659: strb  r7, [r0, #0x1]

__nop:
     65a: bx  lr

HardFaultTrampoline:
     65c: mrs r0, msp
     660: b #-0x2 <HardFault_>

HardFault_:
     662: b #-0x4 <HardFault_>

HardFault:
     663: <unknown>
```

## Running

Next, let's see how to run an embedded program on QEMU! This time we'll use the
`hello` example which actually does something.

For convenience here's the source code of `examples/hello.rs`:

```rust,ignore
//! Prints "Hello, world!" on the host console using semihosting

#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!").unwrap();

    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}
```

This program uses something called semihosting to print text to the *host*
console. When using real hardware this requires a debug session but when using
QEMU this Just Works.

Let's start by compiling the example:

```console
cargo build --example hello
```

The output binary will be located at
`target/thumbv7m-none-eabi/debug/examples/hello`.

To run this binary on QEMU run the following command:

```console
qemu-system-arm \
  -cpu cortex-m3 \
  -machine lm3s6965evb \
  -nographic \
  -semihosting-config enable=on,target=native \
  -kernel target/thumbv7m-none-eabi/debug/examples/hello
```

```text
Hello, world!
```

The command should successfully exit (exit code = 0) after printing the text. On
*nix you can check that with the following command:

```console
echo $?
```

```text
0
```

Let's break down that QEMU command:

- `qemu-system-arm`. This is the QEMU emulator. There are a few variants of
  these QEMU binaries; this one does full *system* emulation of *ARM* machines
  hence the name.

- `-cpu cortex-m3`. This tells QEMU to emulate a Cortex-M3 CPU. Specifying the
  CPU model lets us catch some miscompilation errors: for example, running a
  program compiled for the Cortex-M4F, which has a hardware FPU, will make QEMU
  error during its execution.

- `-machine lm3s6965evb`. This tells QEMU to emulate the LM3S6965EVB, an
  evaluation board that contains a LM3S6965 microcontroller.

- `-nographic`. This tells QEMU to not launch its GUI.

- `-semihosting-config (..)`. This tells QEMU to enable semihosting. Semihosting
  lets the emulated device, among other things, use the host stdout, stderr and
  stdin and create files on the host.

- `-kernel $file`. This tells QEMU which binary to load and run on the emulated
  machine.

Typing out that long QEMU command is too much work! We can set a custom runner
to simplify the process. `.cargo/config.toml` has a commented out runner that invokes
QEMU; let's uncomment it:

```console
head -n3 .cargo/config.toml
```

```toml
[target.thumbv7m-none-eabi]
# uncomment this to make `cargo run` execute programs on QEMU
runner = "qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel"
```

This runner only applies to the `thumbv7m-none-eabi` target, which is our
default compilation target. Now `cargo run` will compile the program and run it
on QEMU:

```console
cargo run --example hello --release
```

```text
   Compiling app v0.1.0 (file:///tmp/app)
    Finished release [optimized + debuginfo] target(s) in 0.26s
     Running `qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel target/thumbv7m-none-eabi/release/examples/hello`
Hello, world!
```

## Debugging

Debugging is critical to embedded development. Let's see how it's done.

Debugging an embedded device involves *remote* debugging as the program that we
want to debug won't be running on the machine that's running the debugger
program (GDB or LLDB).

Remote debugging involves a client and a server. In a QEMU setup, the client
will be a GDB (or LLDB) process and the server will be the QEMU process that's
also running the embedded program.

In this section we'll use the `hello` example we already compiled.

The first debugging step is to launch QEMU in debugging mode:

```console
qemu-system-arm \
  -cpu cortex-m3 \
  -machine lm3s6965evb \
  -nographic \
  -semihosting-config enable=on,target=native \
  -gdb tcp::3333 \
  -S \
  -kernel target/thumbv7m-none-eabi/debug/examples/hello
```

This command won't print anything to the console and will block the terminal. We
have passed two extra flags this time:

- `-gdb tcp::3333`. This tells QEMU to wait for a GDB connection on TCP
  port 3333.

- `-S`. This tells QEMU to freeze the machine at startup. Without this the
  program would have reached the end of main before we had a chance to launch
  the debugger!

Next we launch GDB in another terminal and tell it to load the debug symbols of
the example:

```console
gdb-multiarch -q target/thumbv7m-none-eabi/debug/examples/hello
```

**NOTE**: you might need another version of gdb instead of `gdb-multiarch` depending
on which one you installed in the installation chapter. This could also be
`arm-none-eabi-gdb` or just `gdb`.

Then within the GDB shell we connect to QEMU, which is waiting for a connection
on TCP port 3333.

```console
target remote :3333
```

```text
Remote debugging using :3333
Reset () at $REGISTRY/cortex-m-rt-0.6.1/src/lib.rs:473
473     pub unsafe extern "C" fn Reset() -> ! {
```


You'll see that the process is halted and that the program counter is pointing
to a function named `Reset`. That is the reset handler: what Cortex-M cores
execute upon booting.

>  Note that on some setup, instead of displaying the line `Reset () at $REGISTRY/cortex-m-rt-0.6.1/src/lib.rs:473` as shown above, gdb may print some warnings like : 
>
>`core::num::bignum::Big32x40::mul_small () at src/libcore/num/bignum.rs:254`
> `    src/libcore/num/bignum.rs: No such file or directory.`
> 
> That's a known glitch. You can safely ignore those warnings, you're most likely at Reset(). 


This reset handler will eventually call our main function. Let's skip all the
way there using a breakpoint and the `continue` command. To set the breakpoint, let's first take a look where we would like to break in our code, with the `list` command.

```console
list main
```
This will show the source code, from the file examples/hello.rs. 

```text
6       use panic_halt as _;
7
8       use cortex_m_rt::entry;
9       use cortex_m_semihosting::{debug, hprintln};
10
11      #[entry]
12      fn main() -> ! {
13          hprintln!("Hello, world!").unwrap();
14
15          // exit QEMU
```
We would like to add a breakpoint just before the "Hello, world!", which is on line 13. We do that with the `break` command:

```console
break 13
```
We can now instruct gdb to run up to our main function, with the `continue` command:

```console
continue
```

```text
Continuing.

Breakpoint 1, hello::__cortex_m_rt_main () at examples\hello.rs:13
13          hprintln!("Hello, world!").unwrap();
```

We are now close to the code that prints "Hello, world!". Let's move forward
using the `next` command.

``` console
next
```

```text
16          debug::exit(debug::EXIT_SUCCESS);
```

At this point you should see "Hello, world!" printed on the terminal that's
running `qemu-system-arm`.

```text
$ qemu-system-arm (..)
Hello, world!
```

Calling `next` again will terminate the QEMU process.

```console
next
```

```text
[Inferior 1 (Remote target) exited normally]
```

You can now exit the GDB session.

``` console
quit
```
