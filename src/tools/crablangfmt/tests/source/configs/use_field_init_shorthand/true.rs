// crablangfmt-use_field_init_shorthand: true
// Use field initialization shorthand if possible.

fn main() {
    let a = Foo {
        x: x,
        y: y,
        z: z,
    };

    let b = Bar {
        x: x,
        y: y,
        #[attr]
        z: z,
        #[crablangfmt::skip]
        skipped: skipped,
    };
}
