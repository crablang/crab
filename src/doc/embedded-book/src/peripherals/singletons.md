# Singletons

> In software engineering, the singleton pattern is a software design pattern that restricts the instantiation of a class to one object.
>
> *Wikipedia: [Singleton Pattern]*

[Singleton Pattern]: https://en.wikipedia.org/wiki/Singleton_pattern


## But why can't we just use global variable(s)?

We could make everything a public static, like this

```rust,ignore
static mut THE_SERIAL_PORT: SerialPort = SerialPort;

fn main() {
    let _ = unsafe {
        THE_SERIAL_PORT.read_speed();
    };
}
```

But this has a few problems. It is a mutable global variable, and in Rust, these are always unsafe to interact with. These variables are also visible across your whole program, which means the borrow checker is unable to help you track references and ownership of these variables.

## How do we do this in Rust?

Instead of just making our peripheral a global variable, we might instead decide to make a structure, in this case called `PERIPHERALS`, which contains an `Option<T>` for each of our peripherals.

```rust,ignore
struct Peripherals {
    serial: Option<SerialPort>,
}
impl Peripherals {
    fn take_serial(&mut self) -> SerialPort {
        let p = replace(&mut self.serial, None);
        p.unwrap()
    }
}
static mut PERIPHERALS: Peripherals = Peripherals {
    serial: Some(SerialPort),
};
```

This structure allows us to obtain a single instance of our peripheral. If we try to call `take_serial()` more than once, our code will panic!

```rust,ignore
fn main() {
    let serial_1 = unsafe { PERIPHERALS.take_serial() };
    // This panics!
    // let serial_2 = unsafe { PERIPHERALS.take_serial() };
}
```

Although interacting with this structure is `unsafe`, once we have the `SerialPort` it contained, we no longer need to use `unsafe`, or the `PERIPHERALS` structure at all.

This has a small runtime overhead because we must wrap the `SerialPort` structure in an option, and we'll need to call `take_serial()` once, however this small up-front cost allows us to leverage the borrow checker throughout the rest of our program.

## Existing library support

Although we created our own `Peripherals` structure above, it is not necessary to do this for your code. the `cortex_m` crate contains a macro called `singleton!()` that will perform this action for you.

```rust,ignore
use cortex_m::singleton;

fn main() {
    // OK if `main` is executed only once
    let x: &'static mut bool =
        singleton!(: bool = false).unwrap();
}
```

[cortex_m docs](https://docs.rs/cortex-m/latest/cortex_m/macro.singleton.html)

Additionally, if you use [`cortex-m-rtic`](https://github.com/rtic-rs/cortex-m-rtic), the entire process of defining and obtaining these peripherals are abstracted for you, and you are instead handed a `Peripherals` structure that contains a non-`Option<T>` version of all of the items you define.

```rust,ignore
// cortex-m-rtic v0.5.x
#[rtic::app(device = lm3s6965, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        static mut X: u32 = 0;
         
        // Cortex-M peripherals
        let core: cortex_m::Peripherals = cx.core;
        
        // Device specific peripherals
        let device: lm3s6965::Peripherals = cx.device;
    }
}
```

## But why?

But how do these Singletons make a noticeable difference in how our Rust code works?

```rust,ignore
impl SerialPort {
    const SER_PORT_SPEED_REG: *mut u32 = 0x4000_1000 as _;

    fn read_speed(
        &self // <------ This is really, really important
    ) -> u32 {
        unsafe {
            ptr::read_volatile(Self::SER_PORT_SPEED_REG)
        }
    }
}
```

There are two important factors in play here:

* Because we are using a singleton, there is only one way or place to obtain a `SerialPort` structure
* To call the `read_speed()` method, we must have ownership or a reference to a `SerialPort` structure

These two factors put together means that it is only possible to access the hardware if we have appropriately satisfied the borrow checker, meaning that at no point do we have multiple mutable references to the same hardware!

```rust,ignore
fn main() {
    // missing reference to `self`! Won't work.
    // SerialPort::read_speed();

    let serial_1 = unsafe { PERIPHERALS.take_serial() };

    // you can only read what you have access to
    let _ = serial_1.read_speed();
}
```

## Treat your hardware like data

Additionally, because some references are mutable, and some are immutable, it becomes possible to see whether a function or method could potentially modify the state of the hardware. For example,

This is allowed to change hardware settings:

```rust,ignore
fn setup_spi_port(
    spi: &mut SpiPort,
    cs_pin: &mut GpioPin
) -> Result<()> {
    // ...
}
```

This isn't:

```rust,ignore
fn read_button(gpio: &GpioPin) -> bool {
    // ...
}
```

This allows us to enforce whether code should or should not make changes to hardware at **compile time**, rather than at runtime. As a note, this generally only works across one application, but for bare metal systems, our software will be compiled into a single application, so this is not usually a restriction.
