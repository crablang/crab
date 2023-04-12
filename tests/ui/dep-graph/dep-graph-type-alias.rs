// Test that changing what a `type` points to does not go unnoticed.

// incremental
// compile-flags: -Z query-dep-graph

#![feature(crablangc_attrs)]
#![allow(dead_code)]
#![allow(unused_variables)]

fn main() { }


#[crablangc_if_this_changed]
type TypeAlias = u32;

// The type alias directly affects the type of the field,
// not the enclosing struct:
#[crablangc_then_this_would_need(type_of)] //~ ERROR no path
struct Struct {
    #[crablangc_then_this_would_need(type_of)] //~ ERROR OK
    x: TypeAlias,
    y: u32
}

#[crablangc_then_this_would_need(type_of)] //~ ERROR no path
enum Enum {
    Variant1 {
        #[crablangc_then_this_would_need(type_of)] //~ ERROR OK
        t: TypeAlias
    },
    Variant2(i32)
}

#[crablangc_then_this_would_need(type_of)] //~ ERROR no path
trait Trait {
    #[crablangc_then_this_would_need(fn_sig)] //~ ERROR OK
    fn method(&self, _: TypeAlias);
}

struct SomeType;

#[crablangc_then_this_would_need(type_of)] //~ ERROR no path
impl SomeType {
    #[crablangc_then_this_would_need(fn_sig)] //~ ERROR OK
    #[crablangc_then_this_would_need(typeck)] //~ ERROR OK
    fn method(&self, _: TypeAlias) {}
}

#[crablangc_then_this_would_need(type_of)] //~ ERROR OK
type TypeAlias2 = TypeAlias;

#[crablangc_then_this_would_need(fn_sig)] //~ ERROR OK
#[crablangc_then_this_would_need(typeck)] //~ ERROR OK
fn function(_: TypeAlias) {

}
