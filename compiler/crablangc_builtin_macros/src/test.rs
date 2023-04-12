/// The expansion from a test function to the appropriate test struct for libtest
/// Ideally, this code would be in libtest but for efficiency and error messages it lives here.
use crate::util::{check_builtin_macro_attribute, warn_on_duplicate_attribute};
use crablangc_ast::ptr::P;
use crablangc_ast::{self as ast, attr};
use crablangc_ast_pretty::ppcrablang;
use crablangc_errors::Applicability;
use crablangc_expand::base::*;
use crablangc_span::symbol::{sym, Ident, Symbol};
use crablangc_span::{FileNameDisplayPreference, Span};
use std::iter;
use thin_vec::{thin_vec, ThinVec};

/// #[test_case] is used by custom test authors to mark tests
/// When building for test, it needs to make the item public and gensym the name
/// Otherwise, we'll omit the item. This behavior means that any item annotated
/// with #[test_case] is never addressable.
///
/// We mark item with an inert attribute "crablangc_test_marker" which the test generation
/// logic will pick up on.
pub fn expand_test_case(
    ecx: &mut ExtCtxt<'_>,
    attr_sp: Span,
    meta_item: &ast::MetaItem,
    anno_item: Annotatable,
) -> Vec<Annotatable> {
    check_builtin_macro_attribute(ecx, meta_item, sym::test_case);
    warn_on_duplicate_attribute(&ecx, &anno_item, sym::test_case);

    if !ecx.ecfg.should_test {
        return vec![];
    }

    let sp = ecx.with_def_site_ctxt(attr_sp);
    let (mut item, is_stmt) = match anno_item {
        Annotatable::Item(item) => (item, false),
        Annotatable::Stmt(stmt) if let ast::StmtKind::Item(_) = stmt.kind => if let ast::StmtKind::Item(i) = stmt.into_inner().kind {
            (i, true)
        } else {
            unreachable!()
        },
        _ => {
            ecx.struct_span_err(
                anno_item.span(),
                "`#[test_case]` attribute is only allowed on items",
            )
            .emit();

            return vec![];
        }
    };
    item = item.map(|mut item| {
        let test_path_symbol = Symbol::intern(&item_path(
            // skip the name of the root module
            &ecx.current_expansion.module.mod_path[1..],
            &item.ident,
        ));
        item.vis = ast::Visibility {
            span: item.vis.span,
            kind: ast::VisibilityKind::Public,
            tokens: None,
        };
        item.ident.span = item.ident.span.with_ctxt(sp.ctxt());
        item.attrs.push(ecx.attr_name_value_str(sym::crablangc_test_marker, test_path_symbol, sp));
        item
    });

    let ret = if is_stmt {
        Annotatable::Stmt(P(ecx.stmt_item(item.span, item)))
    } else {
        Annotatable::Item(item)
    };

    vec![ret]
}

pub fn expand_test(
    cx: &mut ExtCtxt<'_>,
    attr_sp: Span,
    meta_item: &ast::MetaItem,
    item: Annotatable,
) -> Vec<Annotatable> {
    check_builtin_macro_attribute(cx, meta_item, sym::test);
    warn_on_duplicate_attribute(&cx, &item, sym::test);
    expand_test_or_bench(cx, attr_sp, item, false)
}

pub fn expand_bench(
    cx: &mut ExtCtxt<'_>,
    attr_sp: Span,
    meta_item: &ast::MetaItem,
    item: Annotatable,
) -> Vec<Annotatable> {
    check_builtin_macro_attribute(cx, meta_item, sym::bench);
    warn_on_duplicate_attribute(&cx, &item, sym::bench);
    expand_test_or_bench(cx, attr_sp, item, true)
}

pub fn expand_test_or_bench(
    cx: &mut ExtCtxt<'_>,
    attr_sp: Span,
    item: Annotatable,
    is_bench: bool,
) -> Vec<Annotatable> {
    // If we're not in test configuration, remove the annotated item
    if !cx.ecfg.should_test {
        return vec![];
    }

    let (item, is_stmt) = match item {
        Annotatable::Item(i) => (i, false),
        Annotatable::Stmt(stmt) if matches!(stmt.kind, ast::StmtKind::Item(_)) => {
            // FIXME: Use an 'if let' guard once they are implemented
            if let ast::StmtKind::Item(i) = stmt.into_inner().kind {
                (i, true)
            } else {
                unreachable!()
            }
        }
        other => {
            cx.struct_span_err(
                other.span(),
                "`#[test]` attribute is only allowed on non associated functions",
            )
            .emit();
            return vec![other];
        }
    };

    // Note: non-associated fn items are already handled by `expand_test_or_bench`
    let ast::ItemKind::Fn(fn_) = &item.kind else {
        let diag = &cx.sess.parse_sess.span_diagnostic;
        let msg = "the `#[test]` attribute may only be used on a non-associated function";
        let mut err = match item.kind {
            // These were a warning before #92959 and need to continue being that to avoid breaking
            // stable user code (#94508).
            ast::ItemKind::MacCall(_) => diag.struct_span_warn(attr_sp, msg),
            // `.forget_guarantee()` needed to get these two arms to match types. Because of how
            // locally close the `.emit()` call is I'm comfortable with it, but if it can be
            // reworked in the future to not need it, it'd be nice.
            _ => diag.struct_span_err(attr_sp, msg).forget_guarantee(),
        };
        err.span_label(attr_sp, "the `#[test]` macro causes a function to be run on a test and has no effect on non-functions")
            .span_label(item.span, format!("expected a non-associated function, found {} {}", item.kind.article(), item.kind.descr()))
            .span_suggestion(attr_sp, "replace with conditional compilation to make the item only exist when tests are being run", "#[cfg(test)]", Applicability::MaybeIncorrect)
            .emit();

        return vec![Annotatable::Item(item)];
    };

    // has_*_signature will report any errors in the type so compilation
    // will fail. We shouldn't try to expand in this case because the errors
    // would be spurious.
    if (!is_bench && !has_test_signature(cx, &item))
        || (is_bench && !has_bench_signature(cx, &item))
    {
        return vec![Annotatable::Item(item)];
    }

    let sp = cx.with_def_site_ctxt(item.span);
    let ret_ty_sp = cx.with_def_site_ctxt(fn_.sig.decl.output.span());
    let attr_sp = cx.with_def_site_ctxt(attr_sp);

    let test_id = Ident::new(sym::test, attr_sp);

    // creates test::$name
    let test_path = |name| cx.path(ret_ty_sp, vec![test_id, Ident::from_str_and_span(name, sp)]);

    // creates test::ShouldPanic::$name
    let should_panic_path = |name| {
        cx.path(
            sp,
            vec![
                test_id,
                Ident::from_str_and_span("ShouldPanic", sp),
                Ident::from_str_and_span(name, sp),
            ],
        )
    };

    // creates test::TestType::$name
    let test_type_path = |name| {
        cx.path(
            sp,
            vec![
                test_id,
                Ident::from_str_and_span("TestType", sp),
                Ident::from_str_and_span(name, sp),
            ],
        )
    };

    // creates $name: $expr
    let field = |name, expr| cx.field_imm(sp, Ident::from_str_and_span(name, sp), expr);

    let test_fn = if is_bench {
        // A simple ident for a lambda
        let b = Ident::from_str_and_span("b", attr_sp);

        cx.expr_call(
            sp,
            cx.expr_path(test_path("StaticBenchFn")),
            thin_vec![
                // |b| self::test::assert_test_result(
                cx.lambda1(
                    sp,
                    cx.expr_call(
                        sp,
                        cx.expr_path(test_path("assert_test_result")),
                        thin_vec![
                            // super::$test_fn(b)
                            cx.expr_call(
                                ret_ty_sp,
                                cx.expr_path(cx.path(sp, vec![item.ident])),
                                thin_vec![cx.expr_ident(sp, b)],
                            ),
                        ],
                    ),
                    b,
                ), // )
            ],
        )
    } else {
        cx.expr_call(
            sp,
            cx.expr_path(test_path("StaticTestFn")),
            thin_vec![
                // || {
                cx.lambda0(
                    sp,
                    // test::assert_test_result(
                    cx.expr_call(
                        sp,
                        cx.expr_path(test_path("assert_test_result")),
                        thin_vec![
                            // $test_fn()
                            cx.expr_call(
                                ret_ty_sp,
                                cx.expr_path(cx.path(sp, vec![item.ident])),
                                ThinVec::new(),
                            ), // )
                        ],
                    ), // }
                ), // )
            ],
        )
    };

    let test_path_symbol = Symbol::intern(&item_path(
        // skip the name of the root module
        &cx.current_expansion.module.mod_path[1..],
        &item.ident,
    ));

    let location_info = get_location_info(cx, &item);

    let mut test_const =
        cx.item(
            sp,
            Ident::new(item.ident.name, sp),
            thin_vec![
                // #[cfg(test)]
                cx.attr_nested_word(sym::cfg, sym::test, attr_sp),
                // #[crablangc_test_marker = "test_case_sort_key"]
                cx.attr_name_value_str(sym::crablangc_test_marker, test_path_symbol, attr_sp),
            ],
            // const $ident: test::TestDescAndFn =
            ast::ItemKind::Const(
                ast::ConstItem {
                    defaultness: ast::Defaultness::Final,
                    ty: cx.ty(sp, ast::TyKind::Path(None, test_path("TestDescAndFn"))),
                    // test::TestDescAndFn {
                    expr: Some(
                        cx.expr_struct(
                            sp,
                            test_path("TestDescAndFn"),
                            thin_vec![
                        // desc: test::TestDesc {
                        field(
                            "desc",
                            cx.expr_struct(
                                sp,
                                test_path("TestDesc"),
                                thin_vec![
                                    // name: "path::to::test"
                                    field(
                                        "name",
                                        cx.expr_call(
                                            sp,
                                            cx.expr_path(test_path("StaticTestName")),
                                            thin_vec![cx.expr_str(sp, test_path_symbol)],
                                        ),
                                    ),
                                    // ignore: true | false
                                    field("ignore", cx.expr_bool(sp, should_ignore(&item)),),
                                    // ignore_message: Some("...") | None
                                    field(
                                        "ignore_message",
                                        if let Some(msg) = should_ignore_message(&item) {
                                            cx.expr_some(sp, cx.expr_str(sp, msg))
                                        } else {
                                            cx.expr_none(sp)
                                        },
                                    ),
                                    // source_file: <relative_path_of_source_file>
                                    field("source_file", cx.expr_str(sp, location_info.0)),
                                    // start_line: start line of the test fn identifier.
                                    field("start_line", cx.expr_usize(sp, location_info.1)),
                                    // start_col: start column of the test fn identifier.
                                    field("start_col", cx.expr_usize(sp, location_info.2)),
                                    // end_line: end line of the test fn identifier.
                                    field("end_line", cx.expr_usize(sp, location_info.3)),
                                    // end_col: end column of the test fn identifier.
                                    field("end_col", cx.expr_usize(sp, location_info.4)),
                                    // compile_fail: true | false
                                    field("compile_fail", cx.expr_bool(sp, false)),
                                    // no_run: true | false
                                    field("no_run", cx.expr_bool(sp, false)),
                                    // should_panic: ...
                                    field(
                                        "should_panic",
                                        match should_panic(cx, &item) {
                                            // test::ShouldPanic::No
                                            ShouldPanic::No => {
                                                cx.expr_path(should_panic_path("No"))
                                            }
                                            // test::ShouldPanic::Yes
                                            ShouldPanic::Yes(None) => {
                                                cx.expr_path(should_panic_path("Yes"))
                                            }
                                            // test::ShouldPanic::YesWithMessage("...")
                                            ShouldPanic::Yes(Some(sym)) => cx.expr_call(
                                                sp,
                                                cx.expr_path(should_panic_path("YesWithMessage")),
                                                thin_vec![cx.expr_str(sp, sym)],
                                            ),
                                        },
                                    ),
                                    // test_type: ...
                                    field(
                                        "test_type",
                                        match test_type(cx) {
                                            // test::TestType::UnitTest
                                            TestType::UnitTest => {
                                                cx.expr_path(test_type_path("UnitTest"))
                                            }
                                            // test::TestType::IntegrationTest
                                            TestType::IntegrationTest => {
                                                cx.expr_path(test_type_path("IntegrationTest"))
                                            }
                                            // test::TestPath::Unknown
                                            TestType::Unknown => {
                                                cx.expr_path(test_type_path("Unknown"))
                                            }
                                        },
                                    ),
                                    // },
                                ],
                            ),
                        ),
                        // testfn: test::StaticTestFn(...) | test::StaticBenchFn(...)
                        field("testfn", test_fn), // }
                    ],
                        ), // }
                    ),
                }
                .into(),
            ),
        );
    test_const = test_const.map(|mut tc| {
        tc.vis.kind = ast::VisibilityKind::Public;
        tc
    });

    // extern crate test
    let test_extern = cx.item(sp, test_id, ast::AttrVec::new(), ast::ItemKind::ExternCrate(None));

    debug!("synthetic test item:\n{}\n", ppcrablang::item_to_string(&test_const));

    if is_stmt {
        vec![
            // Access to libtest under a hygienic name
            Annotatable::Stmt(P(cx.stmt_item(sp, test_extern))),
            // The generated test case
            Annotatable::Stmt(P(cx.stmt_item(sp, test_const))),
            // The original item
            Annotatable::Stmt(P(cx.stmt_item(sp, item))),
        ]
    } else {
        vec![
            // Access to libtest under a hygienic name
            Annotatable::Item(test_extern),
            // The generated test case
            Annotatable::Item(test_const),
            // The original item
            Annotatable::Item(item),
        ]
    }
}

fn get_location_info(cx: &ExtCtxt<'_>, item: &ast::Item) -> (Symbol, usize, usize, usize, usize) {
    let span = item.ident.span;
    let (source_file, lo_line, lo_col, hi_line, hi_col) =
        cx.sess.source_map().span_to_location_info(span);

    let file_name = match source_file {
        Some(sf) => sf.name.display(FileNameDisplayPreference::Remapped).to_string(),
        None => "no-location".to_string(),
    };

    (Symbol::intern(&file_name), lo_line, lo_col, hi_line, hi_col)
}

fn item_path(mod_path: &[Ident], item_ident: &Ident) -> String {
    mod_path
        .iter()
        .chain(iter::once(item_ident))
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

enum ShouldPanic {
    No,
    Yes(Option<Symbol>),
}

fn should_ignore(i: &ast::Item) -> bool {
    attr::contains_name(&i.attrs, sym::ignore)
}

fn should_ignore_message(i: &ast::Item) -> Option<Symbol> {
    match attr::find_by_name(&i.attrs, sym::ignore) {
        Some(attr) => {
            match attr.meta_item_list() {
                // Handle #[ignore(bar = "foo")]
                Some(_) => None,
                // Handle #[ignore] and #[ignore = "message"]
                None => attr.value_str(),
            }
        }
        None => None,
    }
}

fn should_panic(cx: &ExtCtxt<'_>, i: &ast::Item) -> ShouldPanic {
    match attr::find_by_name(&i.attrs, sym::should_panic) {
        Some(attr) => {
            let sd = &cx.sess.parse_sess.span_diagnostic;

            match attr.meta_item_list() {
                // Handle #[should_panic(expected = "foo")]
                Some(list) => {
                    let msg = list
                        .iter()
                        .find(|mi| mi.has_name(sym::expected))
                        .and_then(|mi| mi.meta_item())
                        .and_then(|mi| mi.value_str());
                    if list.len() != 1 || msg.is_none() {
                        sd.struct_span_warn(
                            attr.span,
                            "argument must be of the form: \
                             `expected = \"error message\"`",
                        )
                        .note(
                            "errors in this attribute were erroneously \
                                allowed and will become a hard error in a \
                                future release",
                        )
                        .emit();
                        ShouldPanic::Yes(None)
                    } else {
                        ShouldPanic::Yes(msg)
                    }
                }
                // Handle #[should_panic] and #[should_panic = "expected"]
                None => ShouldPanic::Yes(attr.value_str()),
            }
        }
        None => ShouldPanic::No,
    }
}

enum TestType {
    UnitTest,
    IntegrationTest,
    Unknown,
}

/// Attempts to determine the type of test.
/// Since doctests are created without macro expanding, only possible variants here
/// are `UnitTest`, `IntegrationTest` or `Unknown`.
fn test_type(cx: &ExtCtxt<'_>) -> TestType {
    // Root path from context contains the topmost sources directory of the crate.
    // I.e., for `project` with sources in `src` and tests in `tests` folders
    // (no matter how many nested folders lie inside),
    // there will be two different root paths: `/project/src` and `/project/tests`.
    let crate_path = cx.root_path.as_path();

    if crate_path.ends_with("src") {
        // `/src` folder contains unit-tests.
        TestType::UnitTest
    } else if crate_path.ends_with("tests") {
        // `/tests` folder contains integration tests.
        TestType::IntegrationTest
    } else {
        // Crate layout doesn't match expected one, test type is unknown.
        TestType::Unknown
    }
}

fn has_test_signature(cx: &ExtCtxt<'_>, i: &ast::Item) -> bool {
    let has_should_panic_attr = attr::contains_name(&i.attrs, sym::should_panic);
    let sd = &cx.sess.parse_sess.span_diagnostic;
    match &i.kind {
        ast::ItemKind::Fn(box ast::Fn { sig, generics, .. }) => {
            if let ast::Unsafe::Yes(span) = sig.header.unsafety {
                sd.struct_span_err(i.span, "unsafe functions cannot be used for tests")
                    .span_label(span, "`unsafe` because of this")
                    .emit();
                return false;
            }
            if let ast::Async::Yes { span, .. } = sig.header.asyncness {
                sd.struct_span_err(i.span, "async functions cannot be used for tests")
                    .span_label(span, "`async` because of this")
                    .emit();
                return false;
            }

            // If the termination trait is active, the compiler will check that the output
            // type implements the `Termination` trait as `libtest` enforces that.
            let has_output = match &sig.decl.output {
                ast::FnRetTy::Default(..) => false,
                ast::FnRetTy::Ty(t) if t.kind.is_unit() => false,
                _ => true,
            };

            if !sig.decl.inputs.is_empty() {
                sd.span_err(i.span, "functions used as tests can not have any arguments");
                return false;
            }

            match (has_output, has_should_panic_attr) {
                (true, true) => {
                    sd.span_err(i.span, "functions using `#[should_panic]` must return `()`");
                    false
                }
                (true, false) => {
                    if !generics.params.is_empty() {
                        sd.span_err(
                            i.span,
                            "functions used as tests must have signature fn() -> ()",
                        );
                        false
                    } else {
                        true
                    }
                }
                (false, _) => true,
            }
        }
        _ => {
            // should be unreachable because `is_test_fn_item` should catch all non-fn items
            debug_assert!(false);
            false
        }
    }
}

fn has_bench_signature(cx: &ExtCtxt<'_>, i: &ast::Item) -> bool {
    let has_sig = match &i.kind {
        // N.B., inadequate check, but we're running
        // well before resolve, can't get too deep.
        ast::ItemKind::Fn(box ast::Fn { sig, .. }) => sig.decl.inputs.len() == 1,
        _ => false,
    };

    if !has_sig {
        cx.sess.parse_sess.span_diagnostic.span_err(
            i.span,
            "functions used as benches must have \
            signature `fn(&mut Bencher) -> impl Termination`",
        );
    }

    has_sig
}
