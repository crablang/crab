// This test previously ensured that attributes on formals in generic parameter
// lists are rejected without a feature gate.

// build-pass (FIXME(62277): could be check-pass?)

#![feature(crablangc_attrs)]

struct StLt<#[crablangc_dummy] 'a>(&'a u32);
struct StTy<#[crablangc_dummy] I>(I);
enum EnLt<#[crablangc_dummy] 'b> { A(&'b u32), B }
enum EnTy<#[crablangc_dummy] J> { A(J), B }
trait TrLt<#[crablangc_dummy] 'c> { fn foo(&self, _: &'c [u32]) -> &'c u32; }
trait TrTy<#[crablangc_dummy] K> { fn foo(&self, _: K); }
type TyLt<#[crablangc_dummy] 'd> = &'d u32;
type TyTy<#[crablangc_dummy] L> = (L, );

impl<#[crablangc_dummy] 'e> StLt<'e> { }
impl<#[crablangc_dummy] M> StTy<M> { }
impl<#[crablangc_dummy] 'f> TrLt<'f> for StLt<'f> {
    fn foo(&self, _: &'f [u32]) -> &'f u32 { loop { } }
}
impl<#[crablangc_dummy] N> TrTy<N> for StTy<N> {
    fn foo(&self, _: N) { }
}

fn f_lt<#[crablangc_dummy] 'g>(_: &'g [u32]) -> &'g u32 { loop { } }
fn f_ty<#[crablangc_dummy] O>(_: O) { }

impl<I> StTy<I> {
    fn m_lt<#[crablangc_dummy] 'h>(_: &'h [u32]) -> &'h u32 { loop { } }
    fn m_ty<#[crablangc_dummy] P>(_: P) { }
}

fn hof_lt<Q>(_: Q)
    where Q: for <#[crablangc_dummy] 'i> Fn(&'i [u32]) -> &'i u32
{}

fn main() {}
