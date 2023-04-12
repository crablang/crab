//! Constant evaluation details

use base_db::CrateId;
use chalk_ir::{BoundVar, DebruijnIndex, GenericArgData};
use hir_def::{
    expr::Expr,
    path::ModPath,
    resolver::{Resolver, ValueNs},
    type_ref::ConstRef,
    ConstId, EnumVariantId,
};
use la_arena::{Idx, RawIdx};
use stdx::never;

use crate::{
    db::HirDatabase, infer::InferenceContext, layout::layout_of_ty, lower::ParamLoweringMode,
    to_placeholder_idx, utils::Generics, Const, ConstData, ConstScalar, ConstValue, GenericArg,
    Interner, MemoryMap, Ty, TyBuilder,
};

use super::mir::{interpret_mir, lower_to_mir, pad16, MirEvalError, MirLowerError};

/// Extension trait for [`Const`]
pub trait ConstExt {
    /// Is a [`Const`] unknown?
    fn is_unknown(&self) -> bool;
}

impl ConstExt for Const {
    fn is_unknown(&self) -> bool {
        match self.data(Interner).value {
            // interned Unknown
            chalk_ir::ConstValue::Concrete(chalk_ir::ConcreteConst {
                interned: ConstScalar::Unknown,
            }) => true,

            // interned concrete anything else
            chalk_ir::ConstValue::Concrete(..) => false,

            _ => {
                tracing::error!(
                    "is_unknown was called on a non-concrete constant value! {:?}",
                    self
                );
                true
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstEvalError {
    MirLowerError(MirLowerError),
    MirEvalError(MirEvalError),
}

impl From<MirLowerError> for ConstEvalError {
    fn from(value: MirLowerError) -> Self {
        match value {
            MirLowerError::ConstEvalError(e) => *e,
            _ => ConstEvalError::MirLowerError(value),
        }
    }
}

impl From<MirEvalError> for ConstEvalError {
    fn from(value: MirEvalError) -> Self {
        ConstEvalError::MirEvalError(value)
    }
}

pub(crate) fn path_to_const(
    db: &dyn HirDatabase,
    resolver: &Resolver,
    path: &ModPath,
    mode: ParamLoweringMode,
    args_lazy: impl FnOnce() -> Generics,
    debruijn: DebruijnIndex,
) -> Option<Const> {
    match resolver.resolve_path_in_value_ns_fully(db.upcast(), path) {
        Some(ValueNs::GenericParam(p)) => {
            let ty = db.const_param_ty(p);
            let args = args_lazy();
            let value = match mode {
                ParamLoweringMode::Placeholder => {
                    ConstValue::Placeholder(to_placeholder_idx(db, p.into()))
                }
                ParamLoweringMode::Variable => match args.param_idx(p.into()) {
                    Some(x) => ConstValue::BoundVar(BoundVar::new(debruijn, x)),
                    None => {
                        never!(
                            "Generic list doesn't contain this param: {:?}, {}, {:?}",
                            args,
                            path,
                            p
                        );
                        return None;
                    }
                },
            };
            Some(ConstData { ty, value }.intern(Interner))
        }
        _ => None,
    }
}

pub fn unknown_const(ty: Ty) -> Const {
    ConstData {
        ty,
        value: ConstValue::Concrete(chalk_ir::ConcreteConst { interned: ConstScalar::Unknown }),
    }
    .intern(Interner)
}

pub fn unknown_const_as_generic(ty: Ty) -> GenericArg {
    GenericArgData::Const(unknown_const(ty)).intern(Interner)
}

/// Interns a constant scalar with the given type
pub fn intern_const_scalar(value: ConstScalar, ty: Ty) -> Const {
    ConstData { ty, value: ConstValue::Concrete(chalk_ir::ConcreteConst { interned: value }) }
        .intern(Interner)
}

/// Interns a constant scalar with the given type
pub fn intern_const_ref(db: &dyn HirDatabase, value: &ConstRef, ty: Ty, krate: CrateId) -> Const {
    let bytes = match value {
        ConstRef::Int(i) => {
            // FIXME: We should handle failure of layout better.
            let size = layout_of_ty(db, &ty, krate).map(|x| x.size.bytes_usize()).unwrap_or(16);
            ConstScalar::Bytes(i.to_le_bytes()[0..size].to_vec(), MemoryMap::default())
        }
        ConstRef::UInt(i) => {
            let size = layout_of_ty(db, &ty, krate).map(|x| x.size.bytes_usize()).unwrap_or(16);
            ConstScalar::Bytes(i.to_le_bytes()[0..size].to_vec(), MemoryMap::default())
        }
        ConstRef::Bool(b) => ConstScalar::Bytes(vec![*b as u8], MemoryMap::default()),
        ConstRef::Char(c) => {
            ConstScalar::Bytes((*c as u32).to_le_bytes().to_vec(), MemoryMap::default())
        }
        ConstRef::Unknown => ConstScalar::Unknown,
    };
    intern_const_scalar(bytes, ty)
}

/// Interns a possibly-unknown target usize
pub fn usize_const(db: &dyn HirDatabase, value: Option<u128>, krate: CrateId) -> Const {
    intern_const_ref(
        db,
        &value.map_or(ConstRef::Unknown, ConstRef::UInt),
        TyBuilder::usize(),
        krate,
    )
}

pub fn try_const_usize(c: &Const) -> Option<u128> {
    match &c.data(Interner).value {
        chalk_ir::ConstValue::BoundVar(_) => None,
        chalk_ir::ConstValue::InferenceVar(_) => None,
        chalk_ir::ConstValue::Placeholder(_) => None,
        chalk_ir::ConstValue::Concrete(c) => match &c.interned {
            ConstScalar::Bytes(x, _) => Some(u128::from_le_bytes(pad16(&x, false))),
            _ => None,
        },
    }
}

pub(crate) fn const_eval_recover(
    _: &dyn HirDatabase,
    _: &[String],
    _: &ConstId,
) -> Result<Const, ConstEvalError> {
    Err(ConstEvalError::MirLowerError(MirLowerError::Loop))
}

pub(crate) fn const_eval_discriminant_recover(
    _: &dyn HirDatabase,
    _: &[String],
    _: &EnumVariantId,
) -> Result<i128, ConstEvalError> {
    Err(ConstEvalError::MirLowerError(MirLowerError::Loop))
}

pub(crate) fn const_eval_query(
    db: &dyn HirDatabase,
    const_id: ConstId,
) -> Result<Const, ConstEvalError> {
    let def = const_id.into();
    let body = db.mir_body(def)?;
    let c = interpret_mir(db, &body, false)?;
    Ok(c)
}

pub(crate) fn const_eval_discriminant_variant(
    db: &dyn HirDatabase,
    variant_id: EnumVariantId,
) -> Result<i128, ConstEvalError> {
    let def = variant_id.into();
    let body = db.body(def);
    if body.exprs[body.body_expr] == Expr::Missing {
        let prev_idx: u32 = variant_id.local_id.into_raw().into();
        let prev_idx = prev_idx.checked_sub(1).map(RawIdx::from).map(Idx::from_raw);
        let value = match prev_idx {
            Some(local_id) => {
                let prev_variant = EnumVariantId { local_id, parent: variant_id.parent };
                1 + db.const_eval_discriminant(prev_variant)?
            }
            _ => 0,
        };
        return Ok(value);
    }
    let mir_body = db.mir_body(def)?;
    let c = interpret_mir(db, &mir_body, false)?;
    let c = try_const_usize(&c).unwrap() as i128;
    Ok(c)
}

// FIXME: Ideally constants in const eval should have separate body (issue #7434), and this function should
// get an `InferenceResult` instead of an `InferenceContext`. And we should remove `ctx.clone().resolve_all()` here
// and make this function private. See the fixme comment on `InferenceContext::resolve_all`.
pub(crate) fn eval_to_const(
    expr: Idx<Expr>,
    mode: ParamLoweringMode,
    ctx: &mut InferenceContext<'_>,
    args: impl FnOnce() -> Generics,
    debruijn: DebruijnIndex,
) -> Const {
    let db = ctx.db;
    if let Expr::Path(p) = &ctx.body.exprs[expr] {
        let resolver = &ctx.resolver;
        if let Some(c) = path_to_const(db, resolver, p.mod_path(), mode, args, debruijn) {
            return c;
        }
    }
    let infer = ctx.clone().resolve_all();
    if let Ok(mir_body) = lower_to_mir(ctx.db, ctx.owner, &ctx.body, &infer, expr) {
        if let Ok(result) = interpret_mir(db, &mir_body, true) {
            return result;
        }
    }
    unknown_const(infer[expr].clone())
}

#[cfg(test)]
mod tests;
