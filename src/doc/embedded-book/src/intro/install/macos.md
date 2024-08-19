# macOS

All the tools can be installed using [Homebrew] or [MacPorts]:

[Homebrew]: http://brew.sh/
[MacPorts]: https://www.macports.org/

## Install tools with [Homebrew]

``` text
$ # GDB
$ brew install armmbed/formulae/arm-none-eabi-gcc

$ # OpenOCD
$ brew install openocd

$ # QEMU
$ brew install qemu
```

> **NOTE** If OpenOCD crashes you may need to install the latest version using: 
```text
$ brew install --HEAD openocd
```

## Install tools with [MacPorts]

``` text
$ # GDB
$ sudo port install arm-none-eabi-gcc

$ # OpenOCD
$ sudo port install openocd

$ # QEMU
$ sudo port install qemu
```



That's all! Go to the [next section].

[next section]: verify.md
