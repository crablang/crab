use rustc_attr as attr;
use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_middle::query::Providers;
use rustc_middle::ty::TyCtxt;
use rustc_span::symbol::Symbol;

/// Whether the `def_id` is an unstable const fn and what feature gate(s) are necessary to enable
/// it.
pub fn is_unstable_const_fn(tcx: TyCtxt<'_>, def_id: DefId) -> Option<(Symbol, Option<Symbol>)> {
    if tcx.is_const_fn_raw(def_id) {
        let const_stab = tcx.lookup_const_stability(def_id)?;
        match const_stab.level {
            attr::StabilityLevel::Unstable { implied_by, .. } => {
                Some((const_stab.feature, implied_by))
            }
            attr::StabilityLevel::Stable { .. } => None,
        }
    } else {
        None
    }
}

pub fn is_parent_const_impl_raw(tcx: TyCtxt<'_>, def_id: LocalDefId) -> bool {
    let parent_id = tcx.local_parent(def_id);
    matches!(tcx.def_kind(parent_id), DefKind::Impl { .. })
        && tcx.constness(parent_id) == hir::Constness::Const
}

/// Checks whether an item is considered to be `const`. If it is a constructor, anonymous const,
/// const block, const item or associated const, it is const. If it is a trait impl/function,
/// return if it has a `const` modifier. If it is an intrinsic, report whether said intrinsic
/// has a `rustc_const_{un,}stable` attribute. Otherwise, return `Constness::NotConst`.
fn constness(tcx: TyCtxt<'_>, def_id: LocalDefId) -> hir::Constness {
    let node = tcx.hir().get_by_def_id(def_id);

    match node {
        hir::Node::Ctor(_)
        | hir::Node::AnonConst(_)
        | hir::Node::ConstBlock(_)
        | hir::Node::ImplItem(hir::ImplItem { kind: hir::ImplItemKind::Const(..), .. }) => hir::Constness::Const,
        hir::Node::Item(hir::Item { kind: hir::ItemKind::Impl(impl_), .. }) => impl_.constness,
        hir::Node::ForeignItem(hir::ForeignItem { kind: hir::ForeignItemKind::Fn(..), .. }) => {
            // Intrinsics use `rustc_const_{un,}stable` attributes to indicate constness. All other
            // foreign items cannot be evaluated at compile-time.
            let is_const = if tcx.is_intrinsic(def_id) {
                tcx.lookup_const_stability(def_id).is_some()
            } else {
                false
            };
            if is_const { hir::Constness::Const } else { hir::Constness::NotConst }
        }
        hir::Node::Expr(e) if let hir::ExprKind::Closure(c) = e.kind => c.constness,
        _ => {
            if let Some(fn_kind) = node.fn_kind() {
                if fn_kind.constness() == hir::Constness::Const {
                    return hir::Constness::Const;
                }

                // If the function itself is not annotated with `const`, it may still be a `const fn`
                // if it resides in a const trait impl.
                let is_const = is_parent_const_impl_raw(tcx, def_id);
                if is_const { hir::Constness::Const } else { hir::Constness::NotConst }
            } else {
                hir::Constness::NotConst
            }
        }
    }
}

fn is_promotable_const_fn(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    tcx.is_const_fn(def_id)
        && match tcx.lookup_const_stability(def_id) {
            Some(stab) => {
                if cfg!(debug_assertions) && stab.promotable {
                    let sig = tcx.fn_sig(def_id);
                    assert_eq!(
                        sig.skip_binder().unsafety(),
                        hir::Unsafety::Normal,
                        "don't mark const unsafe fns as promotable",
                        // https://github.com/rust-lang/rust/pull/53851#issuecomment-418760682
                    );
                }
                stab.promotable
            }
            None => false,
        }
}

pub fn provide(providers: &mut Providers) {
    *providers = Providers { constness, is_promotable_const_fn, ..*providers };
}
