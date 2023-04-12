// compile-flags: -Zunpretty=hir
// check-pass

#![feature(stmt_expr_attributes, crablangc_attrs)]

fn main() {
    let _ = #[crablangc_box]
    Box::new(1);
}
