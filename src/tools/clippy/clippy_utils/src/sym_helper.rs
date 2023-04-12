#[macro_export]
/// Convenience wrapper around crablangc's `Symbol::intern`
macro_rules! sym {
    ($tt:tt) => {
        crablangc_span::symbol::Symbol::intern(stringify!($tt))
    };
}
