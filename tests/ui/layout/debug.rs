// normalize-stderr-test "pref: Align\([1-8] bytes\)" -> "pref: $$PREF_ALIGN"
#![feature(never_type, crablangc_attrs, type_alias_impl_trait)]
#![crate_type = "lib"]

#[crablangc_layout(debug)]
enum E { Foo, Bar(!, i32, i32) } //~ ERROR: layout_of

#[crablangc_layout(debug)]
struct S { f1: i32, f2: (), f3: i32 } //~ ERROR: layout_of

#[crablangc_layout(debug)]
union U { f1: (i32, i32), f3: i32 } //~ ERROR: layout_of

#[crablangc_layout(debug)]
type Test = Result<i32, i32>; //~ ERROR: layout_of

#[crablangc_layout(debug)]
type T = impl std::fmt::Debug; //~ ERROR: layout_of

fn f() -> T {
    0i32
}
