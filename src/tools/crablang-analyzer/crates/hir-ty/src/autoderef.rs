//! In certain situations, crablang automatically inserts derefs as necessary: for
//! example, field accesses `foo.bar` still work when `foo` is actually a
//! reference to a type with the field `bar`. This is an approximation of the
//! logic in crablangc (which lives in crablangc_hir_analysis/check/autoderef.rs).

use std::sync::Arc;

use chalk_ir::cast::Cast;
use hir_def::lang_item::LangItem;
use hir_expand::name::name;
use limit::Limit;

use crate::{
    db::HirDatabase, infer::unify::InferenceTable, Canonical, Goal, Interner, ProjectionTyExt,
    TraitEnvironment, Ty, TyBuilder, TyKind,
};

static AUTODEREF_RECURSION_LIMIT: Limit = Limit::new(10);

#[derive(Debug)]
pub(crate) enum AutoderefKind {
    Builtin,
    Overloaded,
}

#[derive(Debug)]
pub(crate) struct Autoderef<'a, 'db> {
    pub(crate) table: &'a mut InferenceTable<'db>,
    ty: Ty,
    at_start: bool,
    steps: Vec<(AutoderefKind, Ty)>,
}

impl<'a, 'db> Autoderef<'a, 'db> {
    pub(crate) fn new(table: &'a mut InferenceTable<'db>, ty: Ty) -> Self {
        let ty = table.resolve_ty_shallow(&ty);
        Autoderef { table, ty, at_start: true, steps: Vec::new() }
    }

    pub(crate) fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub(crate) fn steps(&self) -> &[(AutoderefKind, Ty)] {
        &self.steps
    }

    pub(crate) fn final_ty(&self) -> Ty {
        self.ty.clone()
    }
}

impl Iterator for Autoderef<'_, '_> {
    type Item = (Ty, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_start {
            self.at_start = false;
            return Some((self.ty.clone(), 0));
        }

        if AUTODEREF_RECURSION_LIMIT.check(self.steps.len() + 1).is_err() {
            return None;
        }

        let (kind, new_ty) = autoderef_step(self.table, self.ty.clone())?;

        self.steps.push((kind, self.ty.clone()));
        self.ty = new_ty;

        Some((self.ty.clone(), self.step_count()))
    }
}

pub(crate) fn autoderef_step(
    table: &mut InferenceTable<'_>,
    ty: Ty,
) -> Option<(AutoderefKind, Ty)> {
    if let Some(derefed) = builtin_deref(&ty) {
        Some((AutoderefKind::Builtin, table.resolve_ty_shallow(derefed)))
    } else {
        Some((AutoderefKind::Overloaded, deref_by_trait(table, ty)?))
    }
}

// FIXME: replace uses of this with Autoderef above
pub fn autoderef(
    db: &dyn HirDatabase,
    env: Arc<TraitEnvironment>,
    ty: Canonical<Ty>,
) -> impl Iterator<Item = Canonical<Ty>> + '_ {
    let mut table = InferenceTable::new(db, env);
    let ty = table.instantiate_canonical(ty);
    let mut autoderef = Autoderef::new(&mut table, ty);
    let mut v = Vec::new();
    while let Some((ty, _steps)) = autoderef.next() {
        v.push(autoderef.table.canonicalize(ty).value);
    }
    v.into_iter()
}

pub(crate) fn deref(table: &mut InferenceTable<'_>, ty: Ty) -> Option<Ty> {
    let _p = profile::span("deref");
    autoderef_step(table, ty).map(|(_, ty)| ty)
}

fn builtin_deref(ty: &Ty) -> Option<&Ty> {
    match ty.kind(Interner) {
        TyKind::Ref(.., ty) | TyKind::Raw(.., ty) => Some(ty),
        _ => None,
    }
}

fn deref_by_trait(table: &mut InferenceTable<'_>, ty: Ty) -> Option<Ty> {
    let _p = profile::span("deref_by_trait");
    if table.resolve_ty_shallow(&ty).inference_var(Interner).is_some() {
        // don't try to deref unknown variables
        return None;
    }

    let db = table.db;
    let deref_trait =
        db.lang_item(table.trait_env.krate, LangItem::Deref).and_then(|l| l.as_trait())?;
    let target = db.trait_data(deref_trait).associated_type_by_name(&name![Target])?;

    let projection = {
        let b = TyBuilder::subst_for_def(db, deref_trait, None);
        if b.remaining() != 1 {
            // the Target type + Deref trait should only have one generic parameter,
            // namely Deref's Self type
            return None;
        }
        let deref_subst = b.push(ty).build();
        TyBuilder::assoc_type_projection(db, target, Some(deref_subst)).build()
    };

    // Check that the type implements Deref at all
    let trait_ref = projection.trait_ref(db);
    let implements_goal: Goal = trait_ref.cast(Interner);
    table.try_obligation(implements_goal.clone())?;

    table.register_obligation(implements_goal);

    let result = table.normalize_projection_ty(projection);
    Some(table.resolve_ty_shallow(&result))
}
