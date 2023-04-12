// crablangfmt-version: One
// crablangfmt-error_on_line_overflow: false
// crablangfmt-indent_style: Block

// crablangfmt should not add trailing comma when rewriting macro. See #1528.
fn a() {
    panic!("this is a long string that goes past the maximum line length causing crablangfmt to insert a comma here:");
    foo(a, oooptoptoptoptptooptoptoptoptptooptoptoptoptptoptoptoptoptpt());
}
