# HAL Design Patterns Checklist

- **Naming** *(crate aligns with Rust naming conventions)*
  - [ ] The crate is named appropriately ([C-CRATE-NAME])
- **Interoperability** *(crate interacts nicely with other library functionality)*
  - [ ] Wrapper types provide a destructor method ([C-FREE])
  - [ ] HALs reexport their register access crate ([C-REEXPORT-PAC])
  - [ ] Types implement the `embedded-hal` traits ([C-HAL-TRAITS])
- **Predictability** *(crate enables legible code that acts how it looks)*
  - [ ] Constructors are used instead of extension traits ([C-CTOR])
- **GPIO Interfaces** *(GPIO Interfaces follow a common pattern)*
  - [ ] Pin types are zero-sized by default ([C-ZST-PIN])
  - [ ] Pin types provide methods to erase pin and port ([C-ERASED-PIN])
  - [ ] Pin state should be encoded as type parameters ([C-PIN-STATE])

[C-CRATE-NAME]: naming.html#c-crate-name

[C-FREE]: interoperability.html#c-free
[C-REEXPORT-PAC]: interoperability.html#c-reexport-pac
[C-HAL-TRAITS]: interoperability.html#c-hal-traits

[C-CTOR]: predictability.html#c-ctor

[C-ZST-PIN]: gpio.md#c-zst-pin
[C-ERASED-PIN]: gpio.md#c-erased-pin
[C-PIN-STATE]: gpio.md#c-pin-state
