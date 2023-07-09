//! Helper routines for higher-ranked things. See the `doc` module at
//! the end of the file for details.

use super::combine::CombineFields;
use super::{HigherRankedType, InferCtxt};
use crate::infer::CombinedSnapshot;
use rustc_middle::ty::fold::FnMutDelegate;
use rustc_middle::ty::relate::{Relate, RelateResult, TypeRelation};
use rustc_middle::ty::{self, Binder, Ty, TyCtxt, TypeFoldable};

impl<'a, 'tcx> CombineFields<'a, 'tcx> {
    /// Checks whether `for<..> sub <: for<..> sup` holds.
    ///
    /// For this to hold, **all** instantiations of the super type
    /// have to be a super type of **at least one** instantiation of
    /// the subtype.
    ///
    /// This is implemented by first entering a new universe.
    /// We then replace all bound variables in `sup` with placeholders,
    /// and all bound variables in `sub` with inference vars.
    /// We can then just relate the two resulting types as normal.
    ///
    /// Note: this is a subtle algorithm. For a full explanation, please see
    /// the [rustc dev guide][rd]
    ///
    /// [rd]: https://rustc-dev-guide.rust-lang.org/borrow_check/region_inference/placeholders_and_universes.html
    #[instrument(skip(self), level = "debug")]
    pub fn higher_ranked_sub<T>(
        &mut self,
        sub: Binder<'tcx, T>,
        sup: Binder<'tcx, T>,
        sub_is_expected: bool,
    ) -> RelateResult<'tcx, ()>
    where
        T: Relate<'tcx>,
    {
        let span = self.trace.cause.span;
        // First, we instantiate each bound region in the supertype with a
        // fresh placeholder region. Note that this automatically creates
        // a new universe if needed.
        let sup_prime = self.infcx.instantiate_binder_with_placeholders(sup);

        // Next, we instantiate each bound region in the subtype
        // with a fresh region variable. These region variables --
        // but no other preexisting region variables -- can name
        // the placeholders.
        let sub_prime = self.infcx.instantiate_binder_with_fresh_vars(span, HigherRankedType, sub);

        debug!("a_prime={:?}", sub_prime);
        debug!("b_prime={:?}", sup_prime);

        // Compare types now that bound regions have been replaced.
        let result = self.sub(sub_is_expected).relate(sub_prime, sup_prime)?;

        debug!("OK result={result:?}");
        // NOTE: returning the result here would be dangerous as it contains
        // placeholders which **must not** be named afterwards.
        Ok(())
    }
}

impl<'tcx> InferCtxt<'tcx> {
    /// Replaces all bound variables (lifetimes, types, and constants) bound by
    /// `binder` with placeholder variables in a new universe. This means that the
    /// new placeholders can only be named by inference variables created after
    /// this method has been called.
    ///
    /// This is the first step of checking subtyping when higher-ranked things are involved.
    /// For more details visit the relevant sections of the [rustc dev guide].
    ///
    /// [rustc dev guide]: https://rustc-dev-guide.rust-lang.org/traits/hrtb.html
    #[instrument(level = "debug", skip(self), ret)]
    pub fn instantiate_binder_with_placeholders<T>(&self, binder: ty::Binder<'tcx, T>) -> T
    where
        T: TypeFoldable<TyCtxt<'tcx>> + Copy,
    {
        if let Some(inner) = binder.no_bound_vars() {
            return inner;
        }

        let next_universe = self.create_next_universe();

        let delegate = FnMutDelegate {
            regions: &mut |br: ty::BoundRegion| {
                ty::Region::new_placeholder(
                    self.tcx,
                    ty::PlaceholderRegion { universe: next_universe, bound: br },
                )
            },
            types: &mut |bound_ty: ty::BoundTy| {
                Ty::new_placeholder(
                    self.tcx,
                    ty::PlaceholderType { universe: next_universe, bound: bound_ty },
                )
            },
            consts: &mut |bound_var: ty::BoundVar, ty| {
                ty::Const::new_placeholder(
                    self.tcx,
                    ty::PlaceholderConst { universe: next_universe, bound: bound_var },
                    ty,
                )
            },
        };

        debug!(?next_universe);
        self.tcx.replace_bound_vars_uncached(binder, delegate)
    }

    /// See [RegionConstraintCollector::leak_check][1]. We only check placeholder
    /// leaking into `outer_universe`, i.e. placeholders which cannot be named by that
    /// universe.
    ///
    /// [1]: crate::infer::region_constraints::RegionConstraintCollector::leak_check
    pub fn leak_check(
        &self,
        outer_universe: ty::UniverseIndex,
        only_consider_snapshot: Option<&CombinedSnapshot<'tcx>>,
    ) -> RelateResult<'tcx, ()> {
        // If the user gave `-Zno-leak-check`, or we have been
        // configured to skip the leak check, then skip the leak check
        // completely. The leak check is deprecated. Any legitimate
        // subtyping errors that it would have caught will now be
        // caught later on, during region checking. However, we
        // continue to use it for a transition period.
        if self.tcx.sess.opts.unstable_opts.no_leak_check || self.skip_leak_check {
            return Ok(());
        }

        self.inner.borrow_mut().unwrap_region_constraints().leak_check(
            self.tcx,
            outer_universe,
            self.universe(),
            only_consider_snapshot,
        )
    }
}
