//! The compiler code necessary to implement the `#[derive]` extensions.

use crablangc_ast as ast;
use crablangc_ast::ptr::P;
use crablangc_ast::{GenericArg, Impl, ItemKind, MetaItem};
use crablangc_expand::base::{Annotatable, ExpandResult, ExtCtxt, MultiItemModifier};
use crablangc_span::symbol::{sym, Ident, Symbol};
use crablangc_span::Span;
use thin_vec::{thin_vec, ThinVec};

macro path_local($x:ident) {
    generic::ty::Path::new_local(sym::$x)
}

macro pathvec_std($($rest:ident)::+) {{
    vec![ $( sym::$rest ),+ ]
}}

macro path_std($($x:tt)*) {
    generic::ty::Path::new( pathvec_std!( $($x)* ) )
}

pub mod bounds;
pub mod clone;
pub mod debug;
pub mod decodable;
pub mod default;
pub mod encodable;
pub mod hash;

#[path = "cmp/eq.rs"]
pub mod eq;
#[path = "cmp/ord.rs"]
pub mod ord;
#[path = "cmp/partial_eq.rs"]
pub mod partial_eq;
#[path = "cmp/partial_ord.rs"]
pub mod partial_ord;

pub mod generic;

pub(crate) type BuiltinDeriveFn =
    fn(&mut ExtCtxt<'_>, Span, &MetaItem, &Annotatable, &mut dyn FnMut(Annotatable), bool);

pub(crate) struct BuiltinDerive(pub(crate) BuiltinDeriveFn);

impl MultiItemModifier for BuiltinDerive {
    fn expand(
        &self,
        ecx: &mut ExtCtxt<'_>,
        span: Span,
        meta_item: &MetaItem,
        item: Annotatable,
        is_derive_const: bool,
    ) -> ExpandResult<Vec<Annotatable>, Annotatable> {
        // FIXME: Built-in derives often forget to give spans contexts,
        // so we are doing it here in a centralized way.
        let span = ecx.with_def_site_ctxt(span);
        let mut items = Vec::new();
        match item {
            Annotatable::Stmt(stmt) => {
                if let ast::StmtKind::Item(item) = stmt.into_inner().kind {
                    (self.0)(
                        ecx,
                        span,
                        meta_item,
                        &Annotatable::Item(item),
                        &mut |a| {
                            // Cannot use 'ecx.stmt_item' here, because we need to pass 'ecx'
                            // to the function
                            items.push(Annotatable::Stmt(P(ast::Stmt {
                                id: ast::DUMMY_NODE_ID,
                                kind: ast::StmtKind::Item(a.expect_item()),
                                span,
                            })));
                        },
                        is_derive_const,
                    );
                } else {
                    unreachable!("should have already errored on non-item statement")
                }
            }
            _ => {
                (self.0)(ecx, span, meta_item, &item, &mut |a| items.push(a), is_derive_const);
            }
        }
        ExpandResult::Ready(items)
    }
}

/// Constructs an expression that calls an intrinsic
fn call_intrinsic(
    cx: &ExtCtxt<'_>,
    span: Span,
    intrinsic: Symbol,
    args: ThinVec<P<ast::Expr>>,
) -> P<ast::Expr> {
    let span = cx.with_def_site_ctxt(span);
    let path = cx.std_path(&[sym::intrinsics, intrinsic]);
    cx.expr_call_global(span, path, args)
}

/// Constructs an expression that calls the `unreachable` intrinsic.
fn call_unreachable(cx: &ExtCtxt<'_>, span: Span) -> P<ast::Expr> {
    let span = cx.with_def_site_ctxt(span);
    let path = cx.std_path(&[sym::intrinsics, sym::unreachable]);
    let call = cx.expr_call_global(span, path, ThinVec::new());

    cx.expr_block(P(ast::Block {
        stmts: thin_vec![cx.stmt_expr(call)],
        id: ast::DUMMY_NODE_ID,
        rules: ast::BlockCheckMode::Unsafe(ast::CompilerGenerated),
        span,
        tokens: None,
        could_be_bare_literal: false,
    }))
}

// Injects `impl<...> Structural for ItemType<...> { }`. In particular,
// does *not* add `where T: Structural` for parameters `T` in `...`.
// (That's the main reason we cannot use TraitDef here.)
fn inject_impl_of_structural_trait(
    cx: &mut ExtCtxt<'_>,
    span: Span,
    item: &Annotatable,
    structural_path: generic::ty::Path,
    push: &mut dyn FnMut(Annotatable),
) {
    let Annotatable::Item(item) = item else {
        unreachable!();
    };

    let generics = match &item.kind {
        ItemKind::Struct(_, generics) | ItemKind::Enum(_, generics) => generics,
        // Do not inject `impl Structural for Union`. (`PartialEq` does not
        // support unions, so we will see error downstream.)
        ItemKind::Union(..) => return,
        _ => unreachable!(),
    };

    // Create generics param list for where clauses and impl headers
    let mut generics = generics.clone();

    let ctxt = span.ctxt();

    // Create the type of `self`.
    //
    // in addition, remove defaults from generic params (impls cannot have them).
    let self_params: Vec<_> = generics
        .params
        .iter_mut()
        .map(|param| match &mut param.kind {
            ast::GenericParamKind::Lifetime => ast::GenericArg::Lifetime(
                cx.lifetime(param.ident.span.with_ctxt(ctxt), param.ident),
            ),
            ast::GenericParamKind::Type { default } => {
                *default = None;
                ast::GenericArg::Type(cx.ty_ident(param.ident.span.with_ctxt(ctxt), param.ident))
            }
            ast::GenericParamKind::Const { ty: _, kw_span: _, default } => {
                *default = None;
                ast::GenericArg::Const(
                    cx.const_ident(param.ident.span.with_ctxt(ctxt), param.ident),
                )
            }
        })
        .collect();

    let type_ident = item.ident;

    let trait_ref = cx.trait_ref(structural_path.to_path(cx, span, type_ident, &generics));
    let self_type = cx.ty_path(cx.path_all(span, false, vec![type_ident], self_params));

    // It would be nice to also encode constraint `where Self: Eq` (by adding it
    // onto `generics` cloned above). Unfortunately, that strategy runs afoul of
    // crablang/crablang#48214. So we perform that additional check in the compiler
    // itself, instead of encoding it here.

    // Keep the lint and stability attributes of the original item, to control
    // how the generated implementation is linted.
    let mut attrs = ast::AttrVec::new();
    attrs.extend(
        item.attrs
            .iter()
            .filter(|a| {
                [sym::allow, sym::warn, sym::deny, sym::forbid, sym::stable, sym::unstable]
                    .contains(&a.name_or_empty())
            })
            .cloned(),
    );
    // Mark as `automatically_derived` to avoid some silly lints.
    attrs.push(cx.attr_word(sym::automatically_derived, span));

    let newitem = cx.item(
        span,
        Ident::empty(),
        attrs,
        ItemKind::Impl(Box::new(Impl {
            unsafety: ast::Unsafe::No,
            polarity: ast::ImplPolarity::Positive,
            defaultness: ast::Defaultness::Final,
            constness: ast::Const::No,
            generics,
            of_trait: Some(trait_ref),
            self_ty: self_type,
            items: ThinVec::new(),
        })),
    );

    push(Annotatable::Item(newitem));
}

fn assert_ty_bounds(
    cx: &mut ExtCtxt<'_>,
    stmts: &mut ThinVec<ast::Stmt>,
    ty: P<ast::Ty>,
    span: Span,
    assert_path: &[Symbol],
) {
    // Generate statement `let _: assert_path<ty>;`.
    let span = cx.with_def_site_ctxt(span);
    let assert_path = cx.path_all(span, true, cx.std_path(assert_path), vec![GenericArg::Type(ty)]);
    stmts.push(cx.stmt_let_type_only(span, cx.ty_path(assert_path)));
}
