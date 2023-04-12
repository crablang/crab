// Test that using crablangc_clean/dirty/if_this_changed/then_this_would_need
// are forbidden when `-Z query-dep-graph` is not enabled.

#![feature(crablangc_attrs)]
#![allow(dead_code)]
#![allow(unused_variables)]

#[crablangc_clean(hir_owner)] //~ ERROR attribute requires -Z query-dep-graph
fn main() {}

#[crablangc_if_this_changed(hir_owner)] //~ ERROR attribute requires -Z query-dep-graph
struct Foo<T> {
    f: T,
}

#[crablangc_clean(hir_owner)] //~ ERROR attribute requires -Z query-dep-graph
type TypeAlias<T> = Foo<T>;

#[crablangc_then_this_would_need(variances_of)] //~ ERROR attribute requires -Z query-dep-graph
trait Use<T> {}
