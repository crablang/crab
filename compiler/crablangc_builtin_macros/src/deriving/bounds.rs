use crate::deriving::generic::*;
use crate::deriving::path_std;

use crablangc_ast::MetaItem;
use crablangc_expand::base::{Annotatable, ExtCtxt};
use crablangc_span::Span;

pub fn expand_deriving_copy(
    cx: &mut ExtCtxt<'_>,
    span: Span,
    mitem: &MetaItem,
    item: &Annotatable,
    push: &mut dyn FnMut(Annotatable),
    is_const: bool,
) {
    let trait_def = TraitDef {
        span,
        path: path_std!(marker::Copy),
        skip_path_as_bound: false,
        needs_copy_as_bound_if_packed: false,
        additional_bounds: Vec::new(),
        supports_unions: true,
        methods: Vec::new(),
        associated_types: Vec::new(),
        is_const,
    };

    trait_def.expand(cx, mitem, item, push);
}
