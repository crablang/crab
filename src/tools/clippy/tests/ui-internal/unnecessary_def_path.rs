// run-crablangfix
// aux-build:paths.rs
#![deny(clippy::internal)]
#![feature(crablangc_private)]

extern crate clippy_utils;
extern crate paths;
extern crate crablangc_hir;
extern crate crablangc_lint;
extern crate crablangc_middle;
extern crate crablangc_span;

#[allow(unused)]
use clippy_utils::ty::{is_type_diagnostic_item, is_type_lang_item, match_type};
#[allow(unused)]
use clippy_utils::{
    is_expr_path_def_path, is_path_diagnostic_item, is_res_diagnostic_ctor, is_res_lang_ctor, is_trait_method,
    match_def_path, match_trait_method, path_res,
};

#[allow(unused)]
use crablangc_hir::LangItem;
#[allow(unused)]
use crablangc_span::sym;

use crablangc_hir::def_id::DefId;
use crablangc_hir::Expr;
use crablangc_lint::LateContext;
use crablangc_middle::ty::Ty;

#[allow(unused, clippy::unnecessary_def_path)]
static OPTION: [&str; 3] = ["core", "option", "Option"];
#[allow(unused, clippy::unnecessary_def_path)]
const RESULT: &[&str] = &["core", "result", "Result"];

fn _f<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>, did: DefId, expr: &Expr<'_>) {
    let _ = match_type(cx, ty, &OPTION);
    let _ = match_type(cx, ty, RESULT);
    let _ = match_type(cx, ty, &["core", "result", "Result"]);

    #[allow(unused, clippy::unnecessary_def_path)]
    let rc_path = &["alloc", "rc", "Rc"];
    let _ = clippy_utils::ty::match_type(cx, ty, rc_path);

    let _ = match_type(cx, ty, &paths::OPTION);
    let _ = match_type(cx, ty, paths::RESULT);

    let _ = match_type(cx, ty, &["alloc", "boxed", "Box"]);
    let _ = match_type(cx, ty, &["core", "mem", "maybe_uninit", "MaybeUninit", "uninit"]);

    let _ = match_def_path(cx, did, &["alloc", "boxed", "Box"]);
    let _ = match_def_path(cx, did, &["core", "option", "Option"]);
    let _ = match_def_path(cx, did, &["core", "option", "Option", "Some"]);

    let _ = match_trait_method(cx, expr, &["core", "convert", "AsRef"]);

    let _ = is_expr_path_def_path(cx, expr, &["core", "option", "Option"]);
    let _ = is_expr_path_def_path(cx, expr, &["core", "iter", "traits", "Iterator", "next"]);
    let _ = is_expr_path_def_path(cx, expr, &["core", "option", "Option", "Some"]);
}

fn main() {}
