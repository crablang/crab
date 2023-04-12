#[crablangc_doc_primitive = "usize"]
//~^ ERROR `crablangc_doc_primitive` is a crablangc internal attribute
/// Some docs
mod usize {}

fn main() {}
