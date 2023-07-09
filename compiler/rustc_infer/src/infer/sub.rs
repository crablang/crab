use super::combine::CombineFields;
use super::{DefineOpaqueTypes, ObligationEmittingRelation, SubregionOrigin};

use crate::traits::{Obligation, PredicateObligations};
use rustc_middle::ty::relate::{Cause, Relate, RelateResult, TypeRelation};
use rustc_middle::ty::visit::TypeVisitableExt;
use rustc_middle::ty::TyVar;
use rustc_middle::ty::{self, Ty, TyCtxt};
use std::mem;

/// Ensures `a` is made a subtype of `b`. Returns `a` on success.
pub struct Sub<'combine, 'a, 'tcx> {
    fields: &'combine mut CombineFields<'a, 'tcx>,
    a_is_expected: bool,
}

impl<'combine, 'infcx, 'tcx> Sub<'combine, 'infcx, 'tcx> {
    pub fn new(
        f: &'combine mut CombineFields<'infcx, 'tcx>,
        a_is_expected: bool,
    ) -> Sub<'combine, 'infcx, 'tcx> {
        Sub { fields: f, a_is_expected }
    }

    fn with_expected_switched<R, F: FnOnce(&mut Self) -> R>(&mut self, f: F) -> R {
        self.a_is_expected = !self.a_is_expected;
        let result = f(self);
        self.a_is_expected = !self.a_is_expected;
        result
    }
}

impl<'tcx> TypeRelation<'tcx> for Sub<'_, '_, 'tcx> {
    fn tag(&self) -> &'static str {
        "Sub"
    }

    fn tcx(&self) -> TyCtxt<'tcx> {
        self.fields.infcx.tcx
    }

    fn param_env(&self) -> ty::ParamEnv<'tcx> {
        self.fields.param_env
    }

    fn a_is_expected(&self) -> bool {
        self.a_is_expected
    }

    fn with_cause<F, R>(&mut self, cause: Cause, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        debug!("sub with_cause={:?}", cause);
        let old_cause = mem::replace(&mut self.fields.cause, Some(cause));
        let r = f(self);
        debug!("sub old_cause={:?}", old_cause);
        self.fields.cause = old_cause;
        r
    }

    fn relate_with_variance<T: Relate<'tcx>>(
        &mut self,
        variance: ty::Variance,
        _info: ty::VarianceDiagInfo<'tcx>,
        a: T,
        b: T,
    ) -> RelateResult<'tcx, T> {
        match variance {
            ty::Invariant => self.fields.equate(self.a_is_expected).relate(a, b),
            ty::Covariant => self.relate(a, b),
            ty::Bivariant => Ok(a),
            ty::Contravariant => self.with_expected_switched(|this| this.relate(b, a)),
        }
    }

    #[instrument(skip(self), level = "debug")]
    fn tys(&mut self, a: Ty<'tcx>, b: Ty<'tcx>) -> RelateResult<'tcx, Ty<'tcx>> {
        if a == b {
            return Ok(a);
        }

        let infcx = self.fields.infcx;
        let a = infcx.inner.borrow_mut().type_variables().replace_if_possible(a);
        let b = infcx.inner.borrow_mut().type_variables().replace_if_possible(b);

        match (a.kind(), b.kind()) {
            (&ty::Infer(TyVar(_)), &ty::Infer(TyVar(_))) => {
                // Shouldn't have any LBR here, so we can safely put
                // this under a binder below without fear of accidental
                // capture.
                assert!(!a.has_escaping_bound_vars());
                assert!(!b.has_escaping_bound_vars());

                // can't make progress on `A <: B` if both A and B are
                // type variables, so record an obligation.
                self.fields.obligations.push(Obligation::new(
                    self.tcx(),
                    self.fields.trace.cause.clone(),
                    self.fields.param_env,
                    ty::Binder::dummy(ty::PredicateKind::Subtype(ty::SubtypePredicate {
                        a_is_expected: self.a_is_expected,
                        a,
                        b,
                    })),
                ));

                Ok(a)
            }
            (&ty::Infer(TyVar(a_id)), _) => {
                self.fields.instantiate(b, ty::Contravariant, a_id, !self.a_is_expected)?;
                Ok(a)
            }
            (_, &ty::Infer(TyVar(b_id))) => {
                self.fields.instantiate(a, ty::Covariant, b_id, self.a_is_expected)?;
                Ok(a)
            }

            (&ty::Error(e), _) | (_, &ty::Error(e)) => {
                infcx.set_tainted_by_errors(e);
                Ok(Ty::new_error(self.tcx(), e))
            }

            (
                &ty::Alias(ty::Opaque, ty::AliasTy { def_id: a_def_id, .. }),
                &ty::Alias(ty::Opaque, ty::AliasTy { def_id: b_def_id, .. }),
            ) if a_def_id == b_def_id => {
                self.fields.infcx.super_combine_tys(self, a, b)?;
                Ok(a)
            }
            (&ty::Alias(ty::Opaque, ty::AliasTy { def_id, .. }), _)
            | (_, &ty::Alias(ty::Opaque, ty::AliasTy { def_id, .. }))
                if self.fields.define_opaque_types == DefineOpaqueTypes::Yes
                    && def_id.is_local()
                    && !self.fields.infcx.next_trait_solver() =>
            {
                self.fields.obligations.extend(
                    infcx
                        .handle_opaque_type(
                            a,
                            b,
                            self.a_is_expected,
                            &self.fields.trace.cause,
                            self.param_env(),
                        )?
                        .obligations,
                );
                Ok(a)
            }
            // Optimization of GeneratorWitness relation since we know that all
            // free regions are replaced with bound regions during construction.
            // This greatly speeds up subtyping of GeneratorWitness.
            (&ty::GeneratorWitness(a_types), &ty::GeneratorWitness(b_types)) => {
                let a_types = infcx.tcx.anonymize_bound_vars(a_types);
                let b_types = infcx.tcx.anonymize_bound_vars(b_types);
                if a_types.bound_vars() == b_types.bound_vars() {
                    let (a_types, b_types) = infcx.instantiate_binder_with_placeholders(
                        a_types.map_bound(|a_types| (a_types, b_types.skip_binder())),
                    );
                    for (a, b) in std::iter::zip(a_types, b_types) {
                        self.relate(a, b)?;
                    }
                    Ok(a)
                } else {
                    Err(ty::error::TypeError::Sorts(ty::relate::expected_found(self, a, b)))
                }
            }

            _ => {
                self.fields.infcx.super_combine_tys(self, a, b)?;
                Ok(a)
            }
        }
    }

    fn regions(
        &mut self,
        a: ty::Region<'tcx>,
        b: ty::Region<'tcx>,
    ) -> RelateResult<'tcx, ty::Region<'tcx>> {
        debug!("{}.regions({:?}, {:?}) self.cause={:?}", self.tag(), a, b, self.fields.cause);

        // FIXME -- we have more fine-grained information available
        // from the "cause" field, we could perhaps give more tailored
        // error messages.
        let origin = SubregionOrigin::Subtype(Box::new(self.fields.trace.clone()));
        // Subtype(&'a u8, &'b u8) => Outlives('a: 'b) => SubRegion('b, 'a)
        self.fields
            .infcx
            .inner
            .borrow_mut()
            .unwrap_region_constraints()
            .make_subregion(origin, b, a);

        Ok(a)
    }

    fn consts(
        &mut self,
        a: ty::Const<'tcx>,
        b: ty::Const<'tcx>,
    ) -> RelateResult<'tcx, ty::Const<'tcx>> {
        self.fields.infcx.super_combine_consts(self, a, b)
    }

    fn binders<T>(
        &mut self,
        a: ty::Binder<'tcx, T>,
        b: ty::Binder<'tcx, T>,
    ) -> RelateResult<'tcx, ty::Binder<'tcx, T>>
    where
        T: Relate<'tcx>,
    {
        // A binder is always a subtype of itself if it's structurally equal to itself
        if a == b {
            return Ok(a);
        }

        self.fields.higher_ranked_sub(a, b, self.a_is_expected)?;
        Ok(a)
    }
}

impl<'tcx> ObligationEmittingRelation<'tcx> for Sub<'_, '_, 'tcx> {
    fn register_predicates(&mut self, obligations: impl IntoIterator<Item: ty::ToPredicate<'tcx>>) {
        self.fields.register_predicates(obligations);
    }

    fn register_obligations(&mut self, obligations: PredicateObligations<'tcx>) {
        self.fields.register_obligations(obligations);
    }

    fn alias_relate_direction(&self) -> ty::AliasRelationDirection {
        ty::AliasRelationDirection::Subtype
    }
}
