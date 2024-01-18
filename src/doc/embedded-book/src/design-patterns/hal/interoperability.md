# Interoperability


<a id="c-free"></a>
## Wrapper types provide a destructor method (C-FREE)

Any non-`Copy` wrapper type provided by the HAL should provide a `free` method
that consumes the wrapper and returns back the raw peripheral (and possibly
other objects) it was created from.

The method should shut down and reset the peripheral if necessary. Calling `new`
with the raw peripheral returned by `free` should not fail due to an unexpected
state of the peripheral.

If the HAL type requires other non-`Copy` objects to be constructed (for example
I/O pins), any such object should be released and returned by `free` as well.
`free` should return a tuple in that case.

For example:

```rust
# pub struct TIMER0;
pub struct Timer(TIMER0);

impl Timer {
    pub fn new(periph: TIMER0) -> Self {
        Self(periph)
    }

    pub fn free(self) -> TIMER0 {
        self.0
    }
}
```

<a id="c-reexport-pac"></a>
## HALs reexport their register access crate (C-REEXPORT-PAC)

HALs can be written on top of [svd2rust]-generated PACs, or on top of other
crates that provide raw register access. HALs should always reexport the
register access crate they are based on in their crate root.

A PAC should be reexported under the name `pac`, regardless of the actual name
of the crate, as the name of the HAL should already make it clear what PAC is
being accessed.

[svd2rust]: https://github.com/rust-embedded/svd2rust

<a id="c-hal-traits"></a>
## Types implement the `embedded-hal` traits (C-HAL-TRAITS)

Types provided by the HAL should implement all applicable traits provided by the
[`embedded-hal`] crate.

Multiple traits may be implemented for the same type.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
