# Performing math functionality with `#[no_std]`

If you want to perform math related functionality like calculating the squareroot or
the exponential of a number and you have the full standard library available, your code
might look like this:

```rs
//! Some mathematical functions with standard support available

fn main() {
    let float: f32 = 4.82832;
    let floored_float = float.floor();

    let sqrt_of_four = floored_float.sqrt();

    let sinus_of_four = floored_float.sin();

    let exponential_of_four = floored_float.exp();
    println!("Floored test float {} to {}", float, floored_float);
    println!("The square root of {} is {}", floored_float, sqrt_of_four);
    println!("The sinus of four is {}", sinus_of_four);
    println!(
        "The exponential of four to the base e is {}",
        exponential_of_four
    )
}
```

Without standard library support, these functions are not available.
An external crate like [`libm`](https://crates.io/crates/libm) can be used instead. The example code
would then look like this:

```rs
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use libm::{exp, floorf, sin, sqrtf};

#[entry]
fn main() -> ! {
    let float = 4.82832;
    let floored_float = floorf(float);

    let sqrt_of_four = sqrtf(floored_float);

    let sinus_of_four = sin(floored_float.into());

    let exponential_of_four = exp(floored_float.into());
    hprintln!("Floored test float {} to {}", float, floored_float).unwrap();
    hprintln!("The square root of {} is {}", floored_float, sqrt_of_four).unwrap();
    hprintln!("The sinus of four is {}", sinus_of_four).unwrap();
    hprintln!(
        "The exponential of four to the base e is {}",
        exponential_of_four
    )
    .unwrap();
    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    // debug::exit(debug::EXIT_SUCCESS);

    loop {}
}
```

If you need to perform more complex operations like DSP signal processing or advanced linear
algebra on your MCU, the following crates might help you

- [CMSIS DSP library binding](https://github.com/jacobrosenthal/cmsis-dsp-sys)
- [`constgebra`](https://crates.io/crates/constgebra)
- [`micromath`](https://github.com/tarcieri/micromath)
- [`microfft`](https://crates.io/crates/microfft)
- [`nalgebra`](https://github.com/dimforge/nalgebra)
