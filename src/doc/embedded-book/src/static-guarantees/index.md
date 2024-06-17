# Static Guarantees

Rust's type system prevents data races at compile time (see [`Send`] and
[`Sync`] traits). The type system can also be used to check other properties at
compile time; reducing the need for runtime checks in some cases.

[`Send`]: https://doc.rust-lang.org/core/marker/trait.Send.html
[`Sync`]: https://doc.rust-lang.org/core/marker/trait.Sync.html

When applied to embedded programs these *static checks* can be used, for
example, to enforce that configuration of I/O interfaces is done properly. For
instance, one can design an API where it is only possible to initialize a serial
interface by first configuring the pins that will be used by the interface.

One can also statically check that operations, like setting a pin low, can only
be performed on correctly configured peripherals. For example, trying to change
the output state of a pin configured in floating input mode would raise a
compile error.

And, as seen in the previous chapter, the concept of ownership can be applied
to peripherals to ensure that only certain parts of a program can modify a
peripheral. This *access control* makes software easier to reason about
compared to the alternative of treating peripherals as global mutable state.
