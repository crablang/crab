use rustc_errors::{Applicability, StashKey};
use rustc_hir as hir;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::HirId;
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::util::IntTypeExt;
use rustc_middle::ty::{self, ImplTraitInTraitData, IsSuggestable, Ty, TyCtxt, TypeVisitableExt};
use rustc_span::symbol::Ident;
use rustc_span::{Span, DUMMY_SP};

use super::ItemCtxt;
use super::{bad_placeholder, is_suggestable_infer_ty};

mod opaque;

fn anon_const_type_of<'tcx>(tcx: TyCtxt<'tcx>, def_id: LocalDefId) -> Ty<'tcx> {
    use hir::*;
    use rustc_middle::ty::Ty;
    let hir_id = tcx.hir().local_def_id_to_hir_id(def_id);

    let Node::AnonConst(_) = tcx.hir().get(hir_id) else { panic!() };

    let parent_node_id = tcx.hir().parent_id(hir_id);
    let parent_node = tcx.hir().get(parent_node_id);

    let (generics, arg_idx) = match parent_node {
        // Easy case: arrays repeat expressions.
        Node::Ty(&hir::Ty { kind: TyKind::Array(_, ref constant), .. })
        | Node::Expr(&Expr { kind: ExprKind::Repeat(_, ref constant), .. })
            if constant.hir_id() == hir_id =>
        {
            return tcx.types.usize
        }
        Node::Ty(&hir::Ty { kind: TyKind::Typeof(ref e), .. }) if e.hir_id == hir_id => {
            return tcx.typeck(def_id).node_type(e.hir_id)
        }
        Node::Expr(&Expr { kind: ExprKind::InlineAsm(asm), .. })
        | Node::Item(&Item { kind: ItemKind::GlobalAsm(asm), .. })
            if asm.operands.iter().any(|(op, _op_sp)| match op {
                hir::InlineAsmOperand::Const { anon_const }
                | hir::InlineAsmOperand::SymFn { anon_const } => anon_const.hir_id == hir_id,
                _ => false,
            }) =>
        {
            return tcx.typeck(def_id).node_type(hir_id)
        }
        Node::Variant(Variant { disr_expr: Some(ref e), .. }) if e.hir_id == hir_id => {
            return tcx
                .adt_def(tcx.hir().get_parent_item(hir_id))
                .repr()
                .discr_type()
                .to_ty(tcx)
        }
        Node::GenericParam(&GenericParam {
            def_id: param_def_id,
            kind: GenericParamKind::Const { default: Some(ct), .. },
            ..
        }) if ct.hir_id == hir_id => {
            return tcx.type_of(param_def_id)
                .no_bound_vars()
                .expect("const parameter types cannot be generic")
        }

        Node::TypeBinding(binding @ &TypeBinding { hir_id: binding_id, ..  })
            if let Node::TraitRef(trait_ref) = tcx.hir().get(
                tcx.hir().parent_id(binding_id)
            ) =>
        {
            let Some(trait_def_id) = trait_ref.trait_def_id() else {
                return Ty::new_error_with_message(tcx,tcx.def_span(def_id), "Could not find trait");
            };
            let assoc_items = tcx.associated_items(trait_def_id);
            let assoc_item = assoc_items.find_by_name_and_kind(
                tcx, binding.ident, ty::AssocKind::Const, def_id.to_def_id(),
            );
            return if let Some(assoc_item) = assoc_item {
                tcx.type_of(assoc_item.def_id)
                    .no_bound_vars()
                    .expect("const parameter types cannot be generic")
            } else {
                // FIXME(associated_const_equality): add a useful error message here.
                Ty::new_error_with_message(tcx,tcx.def_span(def_id), "Could not find associated const on trait")
            }
        }

        // This match arm is for when the def_id appears in a GAT whose
        // path can't be resolved without typechecking e.g.
        //
        // trait Foo {
        //   type Assoc<const N: usize>;
        //   fn foo() -> Self::Assoc<3>;
        // }
        //
        // In the above code we would call this query with the def_id of 3 and
        // the parent_node we match on would be the hir node for Self::Assoc<3>
        //
        // `Self::Assoc<3>` cant be resolved without typechecking here as we
        // didnt write <Self as Foo>::Assoc<3>. If we did then another match
        // arm would handle this.
        //
        // I believe this match arm is only needed for GAT but I am not 100% sure - BoxyUwU
        Node::Ty(hir_ty @ hir::Ty { kind: TyKind::Path(QPath::TypeRelative(_, segment)), .. }) => {
            // Find the Item containing the associated type so we can create an ItemCtxt.
            // Using the ItemCtxt convert the HIR for the unresolved assoc type into a
            // ty which is a fully resolved projection.
            // For the code example above, this would mean converting Self::Assoc<3>
            // into a ty::Alias(ty::Projection, <Self as Foo>::Assoc<3>)
            let item_def_id = tcx
                .hir()
                .parent_owner_iter(hir_id)
                .find(|(_, node)| matches!(node, OwnerNode::Item(_)))
                .unwrap()
                .0
                .def_id;
            let item_ctxt = &ItemCtxt::new(tcx, item_def_id) as &dyn crate::astconv::AstConv<'_>;
            let ty = item_ctxt.ast_ty_to_ty(hir_ty);

            // Iterate through the generics of the projection to find the one that corresponds to
            // the def_id that this query was called with. We filter to only type and const args here
            // as a precaution for if it's ever allowed to elide lifetimes in GAT's. It currently isn't
            // but it can't hurt to be safe ^^
            if let ty::Alias(ty::Projection | ty::Inherent, projection) = ty.kind() {
                let generics = tcx.generics_of(projection.def_id);

                let arg_index = segment
                    .args
                    .and_then(|args| {
                        args.args
                            .iter()
                            .filter(|arg| arg.is_ty_or_const())
                            .position(|arg| arg.hir_id() == hir_id)
                    })
                    .unwrap_or_else(|| {
                        bug!("no arg matching AnonConst in segment");
                    });

                (generics, arg_index)
            } else {
                // I dont think it's possible to reach this but I'm not 100% sure - BoxyUwU
                return Ty::new_error_with_message(tcx,
                    tcx.def_span(def_id),
                    "unexpected non-GAT usage of an anon const",
                );
            }
        }
        Node::Expr(&Expr {
            kind:
                ExprKind::MethodCall(segment, ..) | ExprKind::Path(QPath::TypeRelative(_, segment)),
            ..
        }) => {
            let body_owner = tcx.hir().enclosing_body_owner(hir_id);
            let tables = tcx.typeck(body_owner);
            // This may fail in case the method/path does not actually exist.
            // As there is no relevant param for `def_id`, we simply return
            // `None` here.
            let Some(type_dependent_def) = tables.type_dependent_def_id(parent_node_id) else {
                return Ty::new_error_with_message(tcx,
                    tcx.def_span(def_id),
                    format!("unable to find type-dependent def for {parent_node_id:?}"),
                );
            };
            let idx = segment
                .args
                .and_then(|args| {
                    args.args
                        .iter()
                        .filter(|arg| arg.is_ty_or_const())
                        .position(|arg| arg.hir_id() == hir_id)
                })
                .unwrap_or_else(|| {
                    bug!("no arg matching AnonConst in segment");
                });

            (tcx.generics_of(type_dependent_def), idx)
        }

        Node::Ty(&hir::Ty { kind: TyKind::Path(_), .. })
        | Node::Expr(&Expr { kind: ExprKind::Path(_) | ExprKind::Struct(..), .. })
        | Node::TraitRef(..)
        | Node::Pat(_) => {
            let path = match parent_node {
                Node::Ty(&hir::Ty { kind: TyKind::Path(QPath::Resolved(_, path)), .. })
                | Node::TraitRef(&TraitRef { path, .. }) => &*path,
                Node::Expr(&Expr {
                    kind:
                        ExprKind::Path(QPath::Resolved(_, path))
                        | ExprKind::Struct(&QPath::Resolved(_, path), ..),
                    ..
                }) => {
                    let body_owner = tcx.hir().enclosing_body_owner(hir_id);
                    let _tables = tcx.typeck(body_owner);
                    &*path
                }
                Node::Pat(pat) => {
                    if let Some(path) = get_path_containing_arg_in_pat(pat, hir_id) {
                        path
                    } else {
                        return Ty::new_error_with_message(tcx,
                            tcx.def_span(def_id),
                            format!("unable to find const parent for {hir_id} in pat {pat:?}"),
                        );
                    }
                }
                _ => {
                    return Ty::new_error_with_message(tcx,
                        tcx.def_span(def_id),
                        format!("unexpected const parent path {parent_node:?}"),
                    );
                }
            };

            // We've encountered an `AnonConst` in some path, so we need to
            // figure out which generic parameter it corresponds to and return
            // the relevant type.
            let Some((arg_index, segment)) = path.segments.iter().find_map(|seg| {
                let args = seg.args?;
                args.args
                .iter()
                .filter(|arg| arg.is_ty_or_const())
                .position(|arg| arg.hir_id() == hir_id)
                .map(|index| (index, seg)).or_else(|| args.bindings
                    .iter()
                    .filter_map(TypeBinding::opt_const)
                    .position(|ct| ct.hir_id == hir_id)
                    .map(|idx| (idx, seg)))
            }) else {
                return Ty::new_error_with_message(tcx,
                    tcx.def_span(def_id),
                    "no arg matching AnonConst in path",
                );
            };

            let generics = match tcx.res_generics_def_id(segment.res) {
                Some(def_id) => tcx.generics_of(def_id),
                None => {
                    return Ty::new_error_with_message(tcx,
                        tcx.def_span(def_id),
                        format!("unexpected anon const res {:?} in path: {:?}", segment.res, path),
                    );
                }
            };

            (generics, arg_index)
        }

        _ => return Ty::new_error_with_message(tcx,
            tcx.def_span(def_id),
            format!("unexpected const parent in type_of(): {parent_node:?}"),
        ),
    };

    debug!(?parent_node);
    debug!(?generics, ?arg_idx);
    if let Some(param_def_id) = generics
        .params
        .iter()
        .filter(|param| param.kind.is_ty_or_const())
        .nth(match generics.has_self && generics.parent.is_none() {
            true => arg_idx + 1,
            false => arg_idx,
        })
        .and_then(|param| match param.kind {
            ty::GenericParamDefKind::Const { .. } => {
                debug!(?param);
                Some(param.def_id)
            }
            _ => None,
        })
    {
        tcx.type_of(param_def_id).no_bound_vars().expect("const parameter types cannot be generic")
    } else {
        return Ty::new_error_with_message(
            tcx,
            tcx.def_span(def_id),
            format!("const generic parameter not found in {generics:?} at position {arg_idx:?}"),
        );
    }
}

fn get_path_containing_arg_in_pat<'hir>(
    pat: &'hir hir::Pat<'hir>,
    arg_id: HirId,
) -> Option<&'hir hir::Path<'hir>> {
    use hir::*;

    let is_arg_in_path = |p: &hir::Path<'_>| {
        p.segments
            .iter()
            .filter_map(|seg| seg.args)
            .flat_map(|args| args.args)
            .any(|arg| arg.hir_id() == arg_id)
    };
    let mut arg_path = None;
    pat.walk(|pat| match pat.kind {
        PatKind::Struct(QPath::Resolved(_, path), _, _)
        | PatKind::TupleStruct(QPath::Resolved(_, path), _, _)
        | PatKind::Path(QPath::Resolved(_, path))
            if is_arg_in_path(path) =>
        {
            arg_path = Some(path);
            false
        }
        _ => true,
    });
    arg_path
}

pub(super) fn type_of(tcx: TyCtxt<'_>, def_id: LocalDefId) -> ty::EarlyBinder<Ty<'_>> {
    use rustc_hir::*;
    use rustc_middle::ty::Ty;

    // If we are computing `type_of` the synthesized associated type for an RPITIT in the impl
    // side, use `collect_return_position_impl_trait_in_trait_tys` to infer the value of the
    // associated type in the impl.
    if let Some(ImplTraitInTraitData::Impl { fn_def_id, .. }) =
        tcx.opt_rpitit_info(def_id.to_def_id())
    {
        match tcx.collect_return_position_impl_trait_in_trait_tys(fn_def_id) {
            Ok(map) => {
                let assoc_item = tcx.associated_item(def_id);
                return map[&assoc_item.trait_item_def_id.unwrap()];
            }
            Err(_) => {
                return ty::EarlyBinder::bind(Ty::new_error_with_message(
                    tcx,
                    DUMMY_SP,
                    "Could not collect return position impl trait in trait tys",
                ));
            }
        }
    }

    let hir_id = tcx.hir().local_def_id_to_hir_id(def_id);

    let icx = ItemCtxt::new(tcx, def_id);

    let output = match tcx.hir().get(hir_id) {
        Node::TraitItem(item) => match item.kind {
            TraitItemKind::Fn(..) => {
                let args = ty::GenericArgs::identity_for_item(tcx, def_id);
                Ty::new_fn_def(tcx, def_id.to_def_id(), args)
            }
            TraitItemKind::Const(ty, body_id) => body_id
                .and_then(|body_id| {
                    is_suggestable_infer_ty(ty).then(|| {
                        infer_placeholder_type(
                            tcx,
                            def_id,
                            body_id,
                            ty.span,
                            item.ident,
                            "associated constant",
                        )
                    })
                })
                .unwrap_or_else(|| icx.to_ty(ty)),
            TraitItemKind::Type(_, Some(ty)) => icx.to_ty(ty),
            TraitItemKind::Type(_, None) => {
                span_bug!(item.span, "associated type missing default");
            }
        },

        Node::ImplItem(item) => match item.kind {
            ImplItemKind::Fn(..) => {
                let args = ty::GenericArgs::identity_for_item(tcx, def_id);
                Ty::new_fn_def(tcx, def_id.to_def_id(), args)
            }
            ImplItemKind::Const(ty, body_id) => {
                if is_suggestable_infer_ty(ty) {
                    infer_placeholder_type(
                        tcx,
                        def_id,
                        body_id,
                        ty.span,
                        item.ident,
                        "associated constant",
                    )
                } else {
                    icx.to_ty(ty)
                }
            }
            ImplItemKind::Type(ty) => {
                if tcx.impl_trait_ref(tcx.hir().get_parent_item(hir_id)).is_none() {
                    check_feature_inherent_assoc_ty(tcx, item.span);
                }

                icx.to_ty(ty)
            }
        },

        Node::Item(item) => {
            match item.kind {
                ItemKind::Static(ty, .., body_id) => {
                    if is_suggestable_infer_ty(ty) {
                        infer_placeholder_type(
                            tcx,
                            def_id,
                            body_id,
                            ty.span,
                            item.ident,
                            "static variable",
                        )
                    } else {
                        icx.to_ty(ty)
                    }
                }
                ItemKind::Const(ty, body_id) => {
                    if is_suggestable_infer_ty(ty) {
                        infer_placeholder_type(
                            tcx, def_id, body_id, ty.span, item.ident, "constant",
                        )
                    } else {
                        icx.to_ty(ty)
                    }
                }
                ItemKind::TyAlias(self_ty, _) => icx.to_ty(self_ty),
                ItemKind::Impl(hir::Impl { self_ty, .. }) => match self_ty.find_self_aliases() {
                    spans if spans.len() > 0 => {
                        let guar = tcx.sess.emit_err(crate::errors::SelfInImplSelf {
                            span: spans.into(),
                            note: (),
                        });
                        Ty::new_error(tcx, guar)
                    }
                    _ => icx.to_ty(*self_ty),
                },
                ItemKind::Fn(..) => {
                    let args = ty::GenericArgs::identity_for_item(tcx, def_id);
                    Ty::new_fn_def(tcx, def_id.to_def_id(), args)
                }
                ItemKind::Enum(..) | ItemKind::Struct(..) | ItemKind::Union(..) => {
                    let def = tcx.adt_def(def_id);
                    let args = ty::GenericArgs::identity_for_item(tcx, def_id);
                    Ty::new_adt(tcx, def, args)
                }
                ItemKind::OpaqueTy(OpaqueTy {
                    origin: hir::OpaqueTyOrigin::TyAlias { .. },
                    ..
                }) => opaque::find_opaque_ty_constraints_for_tait(tcx, def_id),
                // Opaque types desugared from `impl Trait`.
                ItemKind::OpaqueTy(&OpaqueTy {
                    origin:
                        hir::OpaqueTyOrigin::FnReturn(owner) | hir::OpaqueTyOrigin::AsyncFn(owner),
                    in_trait,
                    ..
                }) => {
                    if in_trait && !tcx.defaultness(owner).has_value() {
                        span_bug!(
                            tcx.def_span(def_id),
                            "tried to get type of this RPITIT with no definition"
                        );
                    }
                    opaque::find_opaque_ty_constraints_for_rpit(tcx, def_id, owner)
                }
                ItemKind::Trait(..)
                | ItemKind::TraitAlias(..)
                | ItemKind::Macro(..)
                | ItemKind::Mod(..)
                | ItemKind::ForeignMod { .. }
                | ItemKind::GlobalAsm(..)
                | ItemKind::ExternCrate(..)
                | ItemKind::Use(..) => {
                    span_bug!(
                        item.span,
                        "compute_type_of_item: unexpected item type: {:?}",
                        item.kind
                    );
                }
            }
        }

        Node::ForeignItem(foreign_item) => match foreign_item.kind {
            ForeignItemKind::Fn(..) => {
                let args = ty::GenericArgs::identity_for_item(tcx, def_id);
                Ty::new_fn_def(tcx, def_id.to_def_id(), args)
            }
            ForeignItemKind::Static(t, _) => icx.to_ty(t),
            ForeignItemKind::Type => Ty::new_foreign(tcx, def_id.to_def_id()),
        },

        Node::Ctor(def) | Node::Variant(Variant { data: def, .. }) => match def {
            VariantData::Unit(..) | VariantData::Struct(..) => {
                tcx.type_of(tcx.hir().get_parent_item(hir_id)).instantiate_identity()
            }
            VariantData::Tuple(..) => {
                let args = ty::GenericArgs::identity_for_item(tcx, def_id);
                Ty::new_fn_def(tcx, def_id.to_def_id(), args)
            }
        },

        Node::Field(field) => icx.to_ty(field.ty),

        Node::Expr(&Expr { kind: ExprKind::Closure { .. }, .. }) => {
            tcx.typeck(def_id).node_type(hir_id)
        }

        Node::AnonConst(_) => anon_const_type_of(tcx, def_id),

        Node::ConstBlock(_) => {
            let args = ty::GenericArgs::identity_for_item(tcx, def_id.to_def_id());
            args.as_inline_const().ty()
        }

        Node::GenericParam(param) => match &param.kind {
            GenericParamKind::Type { default: Some(ty), .. }
            | GenericParamKind::Const { ty, .. } => icx.to_ty(ty),
            x => bug!("unexpected non-type Node::GenericParam: {:?}", x),
        },

        x => {
            bug!("unexpected sort of node in type_of(): {:?}", x);
        }
    };
    ty::EarlyBinder::bind(output)
}

fn infer_placeholder_type<'a>(
    tcx: TyCtxt<'a>,
    def_id: LocalDefId,
    body_id: hir::BodyId,
    span: Span,
    item_ident: Ident,
    kind: &'static str,
) -> Ty<'a> {
    let ty = tcx.diagnostic_only_typeck(def_id).node_type(body_id.hir_id);

    // If this came from a free `const` or `static mut?` item,
    // then the user may have written e.g. `const A = 42;`.
    // In this case, the parser has stashed a diagnostic for
    // us to improve in typeck so we do that now.
    match tcx.sess.diagnostic().steal_diagnostic(span, StashKey::ItemNoType) {
        Some(mut err) => {
            if !ty.references_error() {
                // Only suggest adding `:` if it was missing (and suggested by parsing diagnostic)
                let colon = if span == item_ident.span.shrink_to_hi() { ":" } else { "" };

                // The parser provided a sub-optimal `HasPlaceholders` suggestion for the type.
                // We are typeck and have the real type, so remove that and suggest the actual type.
                // FIXME(eddyb) this looks like it should be functionality on `Diagnostic`.
                if let Ok(suggestions) = &mut err.suggestions {
                    suggestions.clear();
                }

                if let Some(ty) = ty.make_suggestable(tcx, false) {
                    err.span_suggestion(
                        span,
                        format!("provide a type for the {kind}"),
                        format!("{colon} {ty}"),
                        Applicability::MachineApplicable,
                    );
                } else {
                    with_forced_trimmed_paths!(err.span_note(
                        tcx.hir().body(body_id).value.span,
                        format!("however, the inferred type `{ty}` cannot be named"),
                    ));
                }
            }

            err.emit();
        }
        None => {
            let mut diag = bad_placeholder(tcx, vec![span], kind);

            if !ty.references_error() {
                if let Some(ty) = ty.make_suggestable(tcx, false) {
                    diag.span_suggestion(
                        span,
                        "replace with the correct type",
                        ty,
                        Applicability::MachineApplicable,
                    );
                } else {
                    with_forced_trimmed_paths!(diag.span_note(
                        tcx.hir().body(body_id).value.span,
                        format!("however, the inferred type `{ty}` cannot be named"),
                    ));
                }
            }

            diag.emit();
        }
    }

    // Typeck doesn't expect erased regions to be returned from `type_of`.
    tcx.fold_regions(ty, |r, _| match *r {
        ty::ReErased => tcx.lifetimes.re_static,
        _ => r,
    })
}

fn check_feature_inherent_assoc_ty(tcx: TyCtxt<'_>, span: Span) {
    if !tcx.features().inherent_associated_types {
        use rustc_session::parse::feature_err;
        use rustc_span::symbol::sym;
        feature_err(
            &tcx.sess.parse_sess,
            sym::inherent_associated_types,
            span,
            "inherent associated types are unstable",
        )
        .emit();
    }
}
