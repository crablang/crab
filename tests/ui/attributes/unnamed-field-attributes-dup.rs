// Duplicate non-builtin attributes can be used on unnamed fields.

// check-pass

struct S (
    #[crablangfmt::skip]
    #[crablangfmt::skip]
    u8
);

fn main() {}
