// compile-flags: --document-private-items

// This ensures that no ICE is triggered when crablangdoc is run on this code.

mod stdlib {
    pub (crate) use std::i8;
}
