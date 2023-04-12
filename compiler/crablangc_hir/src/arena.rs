/// This higher-order macro declares a list of types which can be allocated by `Arena`.
///
/// Specifying the `decode` modifier will add decode impls for `&T` and `&[T]`,
/// where `T` is the type listed. These impls will appear in the implement_ty_decoder! macro.
#[macro_export]
macro_rules! arena_types {
    ($macro:path) => (
        $macro!([
            // HIR types
            [] hir_krate: crablangc_hir::Crate<'tcx>,
            [] arm: crablangc_hir::Arm<'tcx>,
            [] asm_operand: (crablangc_hir::InlineAsmOperand<'tcx>, crablangc_span::Span),
            [] asm_template: crablangc_ast::InlineAsmTemplatePiece,
            [] attribute: crablangc_ast::Attribute,
            [] closure: crablangc_hir::Closure<'tcx>,
            [] block: crablangc_hir::Block<'tcx>,
            [] bare_fn_ty: crablangc_hir::BareFnTy<'tcx>,
            [] body: crablangc_hir::Body<'tcx>,
            [] generics: crablangc_hir::Generics<'tcx>,
            [] generic_arg: crablangc_hir::GenericArg<'tcx>,
            [] generic_args: crablangc_hir::GenericArgs<'tcx>,
            [] generic_bound: crablangc_hir::GenericBound<'tcx>,
            [] generic_param: crablangc_hir::GenericParam<'tcx>,
            [] expr: crablangc_hir::Expr<'tcx>,
            [] impl_: crablangc_hir::Impl<'tcx>,
            [] let_expr: crablangc_hir::Let<'tcx>,
            [] expr_field: crablangc_hir::ExprField<'tcx>,
            [] pat_field: crablangc_hir::PatField<'tcx>,
            [] fn_decl: crablangc_hir::FnDecl<'tcx>,
            [] foreign_item: crablangc_hir::ForeignItem<'tcx>,
            [] foreign_item_ref: crablangc_hir::ForeignItemRef,
            [] impl_item: crablangc_hir::ImplItem<'tcx>,
            [] impl_item_ref: crablangc_hir::ImplItemRef,
            [] item: crablangc_hir::Item<'tcx>,
            [] inline_asm: crablangc_hir::InlineAsm<'tcx>,
            [] local: crablangc_hir::Local<'tcx>,
            [] mod_: crablangc_hir::Mod<'tcx>,
            [] owner_info: crablangc_hir::OwnerInfo<'tcx>,
            [] param: crablangc_hir::Param<'tcx>,
            [] pat: crablangc_hir::Pat<'tcx>,
            [] path: crablangc_hir::Path<'tcx>,
            [] use_path: crablangc_hir::UsePath<'tcx>,
            [] path_segment: crablangc_hir::PathSegment<'tcx>,
            [] poly_trait_ref: crablangc_hir::PolyTraitRef<'tcx>,
            [] qpath: crablangc_hir::QPath<'tcx>,
            [] stmt: crablangc_hir::Stmt<'tcx>,
            [] field_def: crablangc_hir::FieldDef<'tcx>,
            [] trait_item: crablangc_hir::TraitItem<'tcx>,
            [] trait_item_ref: crablangc_hir::TraitItemRef,
            [] ty: crablangc_hir::Ty<'tcx>,
            [] type_binding: crablangc_hir::TypeBinding<'tcx>,
            [] variant: crablangc_hir::Variant<'tcx>,
            [] where_predicate: crablangc_hir::WherePredicate<'tcx>,
        ]);
    )
}
