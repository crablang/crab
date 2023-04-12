// check-pass

struct S(
    #[crablangfmt::skip] u8,
    u16,
    #[crablangfmt::skip] u32,
);

fn main() {}
