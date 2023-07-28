# Beneath std

This section documents (or will document) features that are provided by the standard library and
that `#![no_std]` developers have to deal with (i.e. provide) to build `#![no_std]` binary crates. A
(likely incomplete) list of such features is shown below:

- `#[lang = "eh_personality"]`
- `#[lang = "start"]`
- `#[lang = "termination"]`
- `#[panic_implementation]`
