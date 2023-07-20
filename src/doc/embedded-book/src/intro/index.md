# Introduction

Welcome to The Embedded Rust Book: An introductory book about using the Rust
Programming Language on "Bare Metal" embedded systems, such as Microcontrollers.

## Who Embedded Rust is For
Embedded Rust is for everyone who wants to do embedded programming while taking advantage of the higher-level concepts and safety guarantees the Rust language provides.
(See also [Who Rust Is For](https://doc.rust-lang.org/book/ch00-00-introduction.html))

## Scope

The goals of this book are:

* Get developers up to speed with embedded Rust development. i.e. How to set
  up a development environment.

* Share *current* best practices about using Rust for embedded development. i.e.
  How to best use Rust language features to write more correct embedded
  software.

* Serve as a cookbook in some cases. e.g. How do I mix C and Rust in a single
  project?

This book tries to be as general as possible but to make things easier for both
the readers and the writers it uses the ARM Cortex-M architecture in all its
examples. However, the book doesn't assume that the reader is familiar with this
particular architecture and explains details particular to this architecture
where required.

## Who This Book is For
This book caters towards people with either some embedded background or some Rust background, however we believe
everybody curious about embedded Rust programming can get something out of this book. For those without any prior knowledge
we suggest you read the "Assumptions and Prerequisites" section and catch up on missing knowledge to get more out of the book
and improve your reading experience. You can check out the "Other Resources" section to find resources on topics
you might want to catch up on.

### Assumptions and Prerequisites

* You are comfortable using the Rust Programming Language, and have written,
  run, and debugged Rust applications on a desktop environment. You should also
  be familiar with the idioms of the [2018 edition] as this book targets
  Rust 2018.

[2018 edition]: https://doc.rust-lang.org/edition-guide/

* You are comfortable developing and debugging embedded systems in another
  language such as C, C++, or Ada, and are familiar with concepts such as:
    * Cross Compilation
    * Memory Mapped Peripherals
    * Interrupts
    * Common interfaces such as I2C, SPI, Serial, etc.

### Other Resources
If you are unfamiliar with anything mentioned above or if you want more information about a specific topic mentioned in this book you might find some of these resources helpful.

| Topic        | Resource | Description |
|--------------|----------|-------------|
| Rust         | [Rust Book](https://doc.rust-lang.org/book/) | If you are not yet comfortable with Rust, we highly suggest reading this book. |
| Rust, Embedded | [Discovery Book](https://docs.rust-embedded.org/discovery/) | If you have never done any embedded programming, this book might be a better start |
| Rust, Embedded | [Embedded Rust Bookshelf](https://docs.rust-embedded.org) | Here you can find several other resources provided by Rust's Embedded Working Group. |
| Rust, Embedded | [Embedonomicon](https://docs.rust-embedded.org/embedonomicon/) | The nitty gritty details when doing embedded programming in Rust. |
| Rust, Embedded | [embedded FAQ](https://docs.rust-embedded.org/faq.html) | Frequently asked questions about Rust in an embedded context. |
| Rust, Embedded | [Comprehensive Rust 🦀: Bare Metal](https://google.github.io/comprehensive-rust/bare-metal.html) | Teaching material for a 1-day class on bare-metal Rust development |
| Interrupts | [Interrupt](https://en.wikipedia.org/wiki/Interrupt) | - |
| Memory-mapped IO/Peripherals | [Memory-mapped I/O](https://en.wikipedia.org/wiki/Memory-mapped_I/O) | - |
| SPI, UART, RS232, USB, I2C, TTL | [Stack Exchange about SPI, UART, and other interfaces](https://electronics.stackexchange.com/questions/37814/usart-uart-rs232-usb-spi-i2c-ttl-etc-what-are-all-of-these-and-how-do-th) | - |

### Translations

This book has been translated by generous volunteers. If you would like your
translation listed here, please open a PR to add it.

* [Japanese](https://tomoyuki-nakabayashi.github.io/book/)
  ([repository](https://github.com/tomoyuki-nakabayashi/book))

* [Chinese](https://xxchang.github.io/book/)
  ([repository](https://github.com/XxChang/book))

## How to Use This Book

This book generally assumes that you’re reading it front-to-back. Later
chapters build on concepts in earlier chapters, and earlier chapters may
not dig into details on a topic, revisiting the topic in a later chapter.

This book will be using the [STM32F3DISCOVERY] development board from
STMicroelectronics for the majority of the examples contained within. This board
is based on the ARM Cortex-M architecture, and while basic functionality is
the same across most CPUs based on this architecture, peripherals and other
implementation details of Microcontrollers are different between different
vendors, and often even different between Microcontroller families from the same
vendor.

For this reason, we suggest purchasing the [STM32F3DISCOVERY] development board
for the purpose of following the examples in this book.

[STM32F3DISCOVERY]: http://www.st.com/en/evaluation-tools/stm32f3discovery.html

## Contributing to This Book

The work on this book is coordinated in [this repository] and is mainly
developed by the [resources team].

[this repository]: https://github.com/rust-embedded/book
[resources team]: https://github.com/rust-embedded/wg#the-resources-team

If you have trouble following the instructions in this book or find that some
section of the book is not clear enough or hard to follow then that's a bug and
it should be reported in [the issue tracker] of this book.

[the issue tracker]: https://github.com/rust-embedded/book/issues/

Pull requests fixing typos and adding new content are very welcome!

## Re-using this material

This book is distributed under the following licenses:

* The code samples and free-standing Cargo projects contained within this book are licensed under the terms of both the [MIT License] and the [Apache License v2.0].
* The written prose, pictures and diagrams contained within this book are licensed under the terms of the Creative Commons [CC-BY-SA v4.0] license.

[MIT License]: https://opensource.org/licenses/MIT
[Apache License v2.0]: http://www.apache.org/licenses/LICENSE-2.0
[CC-BY-SA v4.0]: https://creativecommons.org/licenses/by-sa/4.0/legalcode

TL;DR: If you want to use our text or images in your work, you need to:

* Give the appropriate credit (i.e. mention this book on your slide, and provide a link to the relevant page)
* Provide a link to the [CC-BY-SA v4.0] licence
* Indicate if you have changed the material in any way, and make any changes to our material available under the same licence

Also, please do let us know if you find this book useful!
