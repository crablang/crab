# Windows

## `arm-none-eabi-gdb`

ARM provides `.exe` installers for Windows. Grab one from [here][gcc], and follow the instructions.
Just before the installation process finishes tick/select the "Add path to environment variable"
option. Then verify that the tools are in your `%PATH%`:

``` text
$ arm-none-eabi-gdb -v
GNU gdb (GNU Tools for Arm Embedded Processors 7-2018-q2-update) 8.1.0.20180315-git
(..)
```

[gcc]: https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads

## OpenOCD

There's no official binary release of OpenOCD for Windows but if you're not in the mood to compile
it yourself, the xPack project provides a binary distribution, [here][openocd]. Follow the
provided installation instructions. Then update your `%PATH%` environment variable to
include the path where the binaries were installed. (`C:\Users\USERNAME\AppData\Roaming\xPacks\@xpack-dev-tools\openocd\0.10.0-13.1\.content\bin\`,
if you've been using the easy install) 

[openocd]: https://xpack.github.io/openocd/

Verify that OpenOCD is in your `%PATH%` with:

``` text
$ openocd -v
Open On-Chip Debugger 0.10.0
(..)
```

## QEMU

Grab QEMU from [the official website][qemu].

[qemu]: https://www.qemu.org/download/#windows

## ST-LINK USB driver

You'll also need to install [this USB driver] or OpenOCD won't work. Follow the installer
instructions and make sure you install the right version (32-bit or 64-bit) of the driver.

[this USB driver]: http://www.st.com/en/embedded-software/stsw-link009.html

That's all! Go to the [next section].

[next section]: verify.md
