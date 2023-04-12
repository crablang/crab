// crablangfmt-trailing_comma: Always
// crablangfmt-struct_lit_single_line: false
// crablangfmt-struct_lit_width: 0

fn main() {
    let Foo {
        a,
        ..
    } = b;
}
