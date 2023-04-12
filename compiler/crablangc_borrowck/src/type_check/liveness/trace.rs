use crablangc_data_structures::fx::{FxIndexMap, FxIndexSet};
use crablangc_index::bit_set::HybridBitSet;
use crablangc_index::interval::IntervalSet;
use crablangc_infer::infer::canonical::QueryRegionConstraints;
use crablangc_middle::mir::{BasicBlock, Body, ConstraintCategory, Local, Location};
use crablangc_middle::ty::{Ty, TyCtxt, TypeVisitable, TypeVisitableExt};
use crablangc_trait_selection::traits::query::dropck_outlives::DropckOutlivesResult;
use crablangc_trait_selection::traits::query::type_op::outlives::DropckOutlives;
use crablangc_trait_selection::traits::query::type_op::{TypeOp, TypeOpOutput};
use std::rc::Rc;

use crablangc_mir_dataflow::impls::MaybeInitializedPlaces;
use crablangc_mir_dataflow::move_paths::{HasMoveData, MoveData, MovePathIndex};
use crablangc_mir_dataflow::ResultsCursor;

use crate::{
    region_infer::values::{self, PointIndex, RegionValueElements},
    type_check::liveness::local_use_map::LocalUseMap,
    type_check::liveness::polonius,
    type_check::NormalizeLocation,
    type_check::TypeChecker,
};

/// This is the heart of the liveness computation. For each variable X
/// that requires a liveness computation, it walks over all the uses
/// of X and does a reverse depth-first search ("trace") through the
/// MIR. This search stops when we find a definition of that variable.
/// The points visited in this search is the USE-LIVE set for the variable;
/// of those points is added to all the regions that appear in the variable's
/// type.
///
/// We then also walks through each *drop* of those variables and does
/// another search, stopping when we reach a use or definition. This
/// is the DROP-LIVE set of points. Each of the points in the
/// DROP-LIVE set are to the liveness sets for regions found in the
/// `dropck_outlives` result of the variable's type (in particular,
/// this respects `#[may_dangle]` annotations).
pub(super) fn trace<'mir, 'tcx>(
    typeck: &mut TypeChecker<'_, 'tcx>,
    body: &Body<'tcx>,
    elements: &Rc<RegionValueElements>,
    flow_inits: &mut ResultsCursor<'mir, 'tcx, MaybeInitializedPlaces<'mir, 'tcx>>,
    move_data: &MoveData<'tcx>,
    relevant_live_locals: Vec<Local>,
    boring_locals: Vec<Local>,
    polonius_drop_used: Option<Vec<(Local, Location)>>,
) {
    debug!("trace()");

    let local_use_map = &LocalUseMap::build(&relevant_live_locals, elements, body);

    let cx = LivenessContext {
        typeck,
        body,
        flow_inits,
        elements,
        local_use_map,
        move_data,
        drop_data: FxIndexMap::default(),
    };

    let mut results = LivenessResults::new(cx);

    if let Some(drop_used) = polonius_drop_used {
        results.add_extra_drop_facts(drop_used, relevant_live_locals.iter().copied().collect())
    }

    results.compute_for_all_locals(relevant_live_locals);

    results.dropck_boring_locals(boring_locals);
}

/// Contextual state for the type-liveness generator.
struct LivenessContext<'me, 'typeck, 'flow, 'tcx> {
    /// Current type-checker, giving us our inference context etc.
    typeck: &'me mut TypeChecker<'typeck, 'tcx>,

    /// Defines the `PointIndex` mapping
    elements: &'me RegionValueElements,

    /// MIR we are analyzing.
    body: &'me Body<'tcx>,

    /// Mapping to/from the various indices used for initialization tracking.
    move_data: &'me MoveData<'tcx>,

    /// Cache for the results of `dropck_outlives` query.
    drop_data: FxIndexMap<Ty<'tcx>, DropData<'tcx>>,

    /// Results of dataflow tracking which variables (and paths) have been
    /// initialized.
    flow_inits: &'me mut ResultsCursor<'flow, 'tcx, MaybeInitializedPlaces<'flow, 'tcx>>,

    /// Index indicating where each variable is assigned, used, or
    /// dropped.
    local_use_map: &'me LocalUseMap,
}

struct DropData<'tcx> {
    dropck_result: DropckOutlivesResult<'tcx>,
    region_constraint_data: Option<&'tcx QueryRegionConstraints<'tcx>>,
}

struct LivenessResults<'me, 'typeck, 'flow, 'tcx> {
    cx: LivenessContext<'me, 'typeck, 'flow, 'tcx>,

    /// Set of points that define the current local.
    defs: HybridBitSet<PointIndex>,

    /// Points where the current variable is "use live" -- meaning
    /// that there is a future "full use" that may use its value.
    use_live_at: IntervalSet<PointIndex>,

    /// Points where the current variable is "drop live" -- meaning
    /// that there is no future "full use" that may use its value, but
    /// there is a future drop.
    drop_live_at: IntervalSet<PointIndex>,

    /// Locations where drops may occur.
    drop_locations: Vec<Location>,

    /// Stack used when doing (reverse) DFS.
    stack: Vec<PointIndex>,
}

impl<'me, 'typeck, 'flow, 'tcx> LivenessResults<'me, 'typeck, 'flow, 'tcx> {
    fn new(cx: LivenessContext<'me, 'typeck, 'flow, 'tcx>) -> Self {
        let num_points = cx.elements.num_points();
        LivenessResults {
            cx,
            defs: HybridBitSet::new_empty(num_points),
            use_live_at: IntervalSet::new(num_points),
            drop_live_at: IntervalSet::new(num_points),
            drop_locations: vec![],
            stack: vec![],
        }
    }

    fn compute_for_all_locals(&mut self, relevant_live_locals: Vec<Local>) {
        for local in relevant_live_locals {
            self.reset_local_state();
            self.add_defs_for(local);
            self.compute_use_live_points_for(local);
            self.compute_drop_live_points_for(local);

            let local_ty = self.cx.body.local_decls[local].ty;

            if !self.use_live_at.is_empty() {
                self.cx.add_use_live_facts_for(local_ty, &self.use_live_at);
            }

            if !self.drop_live_at.is_empty() {
                self.cx.add_drop_live_facts_for(
                    local,
                    local_ty,
                    &self.drop_locations,
                    &self.drop_live_at,
                );
            }
        }
    }

    // Runs dropck for locals whose liveness isn't relevant. This is
    // necessary to eagerly detect unbound recursion during drop glue computation.
    fn dropck_boring_locals(&mut self, boring_locals: Vec<Local>) {
        for local in boring_locals {
            let local_ty = self.cx.body.local_decls[local].ty;
            let drop_data = self.cx.drop_data.entry(local_ty).or_insert_with({
                let typeck = &mut self.cx.typeck;
                move || LivenessContext::compute_drop_data(typeck, local_ty)
            });

            drop_data.dropck_result.report_overflows(
                self.cx.typeck.infcx.tcx,
                self.cx.body.local_decls[local].source_info.span,
                local_ty,
            );
        }
    }

    /// Add extra drop facts needed for Polonius.
    ///
    /// Add facts for all locals with free regions, since regions may outlive
    /// the function body only at certain nodes in the CFG.
    fn add_extra_drop_facts(
        &mut self,
        drop_used: Vec<(Local, Location)>,
        relevant_live_locals: FxIndexSet<Local>,
    ) {
        let locations = IntervalSet::new(self.cx.elements.num_points());

        for (local, location) in drop_used {
            if !relevant_live_locals.contains(&local) {
                let local_ty = self.cx.body.local_decls[local].ty;
                if local_ty.has_free_regions() {
                    self.cx.add_drop_live_facts_for(local, local_ty, &[location], &locations);
                }
            }
        }
    }

    /// Clear the value of fields that are "per local variable".
    fn reset_local_state(&mut self) {
        self.defs.clear();
        self.use_live_at.clear();
        self.drop_live_at.clear();
        self.drop_locations.clear();
        assert!(self.stack.is_empty());
    }

    /// Adds the definitions of `local` into `self.defs`.
    fn add_defs_for(&mut self, local: Local) {
        for def in self.cx.local_use_map.defs(local) {
            debug!("- defined at {:?}", def);
            self.defs.insert(def);
        }
    }

    /// Computes all points where local is "use live" -- meaning its
    /// current value may be used later (except by a drop). This is
    /// done by walking backwards from each use of `local` until we
    /// find a `def` of local.
    ///
    /// Requires `add_defs_for(local)` to have been executed.
    fn compute_use_live_points_for(&mut self, local: Local) {
        debug!("compute_use_live_points_for(local={:?})", local);

        self.stack.extend(self.cx.local_use_map.uses(local));
        while let Some(p) = self.stack.pop() {
            // We are live in this block from the closest to us of:
            //
            //  * Inclusively, the block start
            //  * Exclusively, the previous definition (if it's in this block)
            //  * Exclusively, the previous live_at setting (an optimization)
            let block_start = self.cx.elements.to_block_start(p);
            let previous_defs = self.defs.last_set_in(block_start..=p);
            let previous_live_at = self.use_live_at.last_set_in(block_start..=p);

            let exclusive_start = match (previous_defs, previous_live_at) {
                (Some(a), Some(b)) => Some(std::cmp::max(a, b)),
                (Some(a), None) | (None, Some(a)) => Some(a),
                (None, None) => None,
            };

            if let Some(exclusive) = exclusive_start {
                self.use_live_at.insert_range(exclusive + 1..=p);

                // If we have a bound after the start of the block, we should
                // not add the predecessors for this block.
                continue;
            } else {
                // Add all the elements of this block.
                self.use_live_at.insert_range(block_start..=p);

                // Then add the predecessors for this block, which are the
                // terminators of predecessor basic blocks. Push those onto the
                // stack so that the next iteration(s) will process them.

                let block = self.cx.elements.to_location(block_start).block;
                self.stack.extend(
                    self.cx.body.basic_blocks.predecessors()[block]
                        .iter()
                        .map(|&pred_bb| self.cx.body.terminator_loc(pred_bb))
                        .map(|pred_loc| self.cx.elements.point_from_location(pred_loc)),
                );
            }
        }
    }

    /// Computes all points where local is "drop live" -- meaning its
    /// current value may be dropped later (but not used). This is
    /// done by iterating over the drops of `local` where `local` (or
    /// some subpart of `local`) is initialized. For each such drop,
    /// we walk backwards until we find a point where `local` is
    /// either defined or use-live.
    ///
    /// Requires `compute_use_live_points_for` and `add_defs_for` to
    /// have been executed.
    fn compute_drop_live_points_for(&mut self, local: Local) {
        debug!("compute_drop_live_points_for(local={:?})", local);

        let mpi = self.cx.move_data.rev_lookup.find_local(local);
        debug!("compute_drop_live_points_for: mpi = {:?}", mpi);

        // Find the drops where `local` is initialized.
        for drop_point in self.cx.local_use_map.drops(local) {
            let location = self.cx.elements.to_location(drop_point);
            debug_assert_eq!(self.cx.body.terminator_loc(location.block), location,);

            if self.cx.initialized_at_terminator(location.block, mpi) {
                if self.drop_live_at.insert(drop_point) {
                    self.drop_locations.push(location);
                    self.stack.push(drop_point);
                }
            }
        }

        debug!("compute_drop_live_points_for: drop_locations={:?}", self.drop_locations);

        // Reverse DFS. But for drops, we do it a bit differently.
        // The stack only ever stores *terminators of blocks*. Within
        // a block, we walk back the statements in an inner loop.
        while let Some(term_point) = self.stack.pop() {
            self.compute_drop_live_points_for_block(mpi, term_point);
        }
    }

    /// Executes one iteration of the drop-live analysis loop.
    ///
    /// The parameter `mpi` is the `MovePathIndex` of the local variable
    /// we are currently analyzing.
    ///
    /// The point `term_point` represents some terminator in the MIR,
    /// where the local `mpi` is drop-live on entry to that terminator.
    ///
    /// This method adds all drop-live points within the block and --
    /// where applicable -- pushes the terminators of preceding blocks
    /// onto `self.stack`.
    fn compute_drop_live_points_for_block(&mut self, mpi: MovePathIndex, term_point: PointIndex) {
        debug!(
            "compute_drop_live_points_for_block(mpi={:?}, term_point={:?})",
            self.cx.move_data.move_paths[mpi].place,
            self.cx.elements.to_location(term_point),
        );

        // We are only invoked with terminators where `mpi` is
        // drop-live on entry.
        debug_assert!(self.drop_live_at.contains(term_point));

        // Otherwise, scan backwards through the statements in the
        // block. One of them may be either a definition or use
        // live point.
        let term_location = self.cx.elements.to_location(term_point);
        debug_assert_eq!(self.cx.body.terminator_loc(term_location.block), term_location,);
        let block = term_location.block;
        let entry_point = self.cx.elements.entry_point(term_location.block);
        for p in (entry_point..term_point).rev() {
            debug!("compute_drop_live_points_for_block: p = {:?}", self.cx.elements.to_location(p));

            if self.defs.contains(p) {
                debug!("compute_drop_live_points_for_block: def site");
                return;
            }

            if self.use_live_at.contains(p) {
                debug!("compute_drop_live_points_for_block: use-live at {:?}", p);
                return;
            }

            if !self.drop_live_at.insert(p) {
                debug!("compute_drop_live_points_for_block: already drop-live");
                return;
            }
        }

        let body = self.cx.body;
        for &pred_block in body.basic_blocks.predecessors()[block].iter() {
            debug!("compute_drop_live_points_for_block: pred_block = {:?}", pred_block,);

            // Check whether the variable is (at least partially)
            // initialized at the exit of this predecessor. If so, we
            // want to enqueue it on our list. If not, go check the
            // next block.
            //
            // Note that we only need to check whether `live_local`
            // became de-initialized at basic block boundaries. If it
            // were to become de-initialized within the block, that
            // would have been a "use-live" transition in the earlier
            // loop, and we'd have returned already.
            //
            // NB. It's possible that the pred-block ends in a call
            // which stores to the variable; in that case, the
            // variable may be uninitialized "at exit" because this
            // call only considers the *unconditional effects* of the
            // terminator. *But*, in that case, the terminator is also
            // a *definition* of the variable, in which case we want
            // to stop the search anyhow. (But see Note 1 below.)
            if !self.cx.initialized_at_exit(pred_block, mpi) {
                debug!("compute_drop_live_points_for_block: not initialized");
                continue;
            }

            let pred_term_loc = self.cx.body.terminator_loc(pred_block);
            let pred_term_point = self.cx.elements.point_from_location(pred_term_loc);

            // If the terminator of this predecessor either *assigns*
            // our value or is a "normal use", then stop.
            if self.defs.contains(pred_term_point) {
                debug!("compute_drop_live_points_for_block: defined at {:?}", pred_term_loc);
                continue;
            }

            if self.use_live_at.contains(pred_term_point) {
                debug!("compute_drop_live_points_for_block: use-live at {:?}", pred_term_loc);
                continue;
            }

            // Otherwise, we are drop-live on entry to the terminator,
            // so walk it.
            if self.drop_live_at.insert(pred_term_point) {
                debug!("compute_drop_live_points_for_block: pushed to stack");
                self.stack.push(pred_term_point);
            }
        }

        // Note 1. There is a weird scenario that you might imagine
        // being problematic here, but which actually cannot happen.
        // The problem would be if we had a variable that *is* initialized
        // (but dead) on entry to the terminator, and where the current value
        // will be dropped in the case of unwind. In that case, we ought to
        // consider `X` to be drop-live in between the last use and call.
        // Here is the example:
        //
        // ```
        // BB0 {
        //   X = ...
        //   use(X); // last use
        //   ...     // <-- X ought to be drop-live here
        //   X = call() goto BB1 unwind BB2
        // }
        //
        // BB1 {
        //   DROP(X)
        // }
        //
        // BB2 {
        //   DROP(X)
        // }
        // ```
        //
        // However, the current code would, when walking back from BB2,
        // simply stop and never explore BB0. This seems bad! But it turns
        // out this code is flawed anyway -- note that the existing value of
        // `X` would leak in the case where unwinding did *not* occur.
        //
        // What we *actually* generate is a store to a temporary
        // for the call (`TMP = call()...`) and then a
        // `Drop(X)` followed by `X = TMP`  to swap that with `X`.
    }
}

impl<'tcx> LivenessContext<'_, '_, '_, 'tcx> {
    /// Returns `true` if the local variable (or some part of it) is initialized at the current
    /// cursor position. Callers should call one of the `seek` methods immediately before to point
    /// the cursor to the desired location.
    fn initialized_at_curr_loc(&self, mpi: MovePathIndex) -> bool {
        let state = self.flow_inits.get();
        if state.contains(mpi) {
            return true;
        }

        let move_paths = &self.flow_inits.analysis().move_data().move_paths;
        move_paths[mpi].find_descendant(&move_paths, |mpi| state.contains(mpi)).is_some()
    }

    /// Returns `true` if the local variable (or some part of it) is initialized in
    /// the terminator of `block`. We need to check this to determine if a
    /// DROP of some local variable will have an effect -- note that
    /// drops, as they may unwind, are always terminators.
    fn initialized_at_terminator(&mut self, block: BasicBlock, mpi: MovePathIndex) -> bool {
        self.flow_inits.seek_before_primary_effect(self.body.terminator_loc(block));
        self.initialized_at_curr_loc(mpi)
    }

    /// Returns `true` if the path `mpi` (or some part of it) is initialized at
    /// the exit of `block`.
    ///
    /// **Warning:** Does not account for the result of `Call`
    /// instructions.
    fn initialized_at_exit(&mut self, block: BasicBlock, mpi: MovePathIndex) -> bool {
        self.flow_inits.seek_after_primary_effect(self.body.terminator_loc(block));
        self.initialized_at_curr_loc(mpi)
    }

    /// Stores the result that all regions in `value` are live for the
    /// points `live_at`.
    fn add_use_live_facts_for(
        &mut self,
        value: impl TypeVisitable<TyCtxt<'tcx>>,
        live_at: &IntervalSet<PointIndex>,
    ) {
        debug!("add_use_live_facts_for(value={:?})", value);

        Self::make_all_regions_live(self.elements, &mut self.typeck, value, live_at)
    }

    /// Some variable with type `live_ty` is "drop live" at `location`
    /// -- i.e., it may be dropped later. This means that *some* of
    /// the regions in its type must be live at `location`. The
    /// precise set will depend on the dropck constraints, and in
    /// particular this takes `#[may_dangle]` into account.
    fn add_drop_live_facts_for(
        &mut self,
        dropped_local: Local,
        dropped_ty: Ty<'tcx>,
        drop_locations: &[Location],
        live_at: &IntervalSet<PointIndex>,
    ) {
        debug!(
            "add_drop_live_constraint(\
             dropped_local={:?}, \
             dropped_ty={:?}, \
             drop_locations={:?}, \
             live_at={:?})",
            dropped_local,
            dropped_ty,
            drop_locations,
            values::location_set_str(self.elements, live_at.iter()),
        );

        let drop_data = self.drop_data.entry(dropped_ty).or_insert_with({
            let typeck = &mut self.typeck;
            move || Self::compute_drop_data(typeck, dropped_ty)
        });

        if let Some(data) = &drop_data.region_constraint_data {
            for &drop_location in drop_locations {
                self.typeck.push_region_constraints(
                    drop_location.to_locations(),
                    ConstraintCategory::Boring,
                    data,
                );
            }
        }

        drop_data.dropck_result.report_overflows(
            self.typeck.infcx.tcx,
            self.body.source_info(*drop_locations.first().unwrap()).span,
            dropped_ty,
        );

        // All things in the `outlives` array may be touched by
        // the destructor and must be live at this point.
        for &kind in &drop_data.dropck_result.kinds {
            Self::make_all_regions_live(self.elements, &mut self.typeck, kind, live_at);

            polonius::add_drop_of_var_derefs_origin(&mut self.typeck, dropped_local, &kind);
        }
    }

    fn make_all_regions_live(
        elements: &RegionValueElements,
        typeck: &mut TypeChecker<'_, 'tcx>,
        value: impl TypeVisitable<TyCtxt<'tcx>>,
        live_at: &IntervalSet<PointIndex>,
    ) {
        debug!("make_all_regions_live(value={:?})", value);
        debug!(
            "make_all_regions_live: live_at={}",
            values::location_set_str(elements, live_at.iter()),
        );

        let tcx = typeck.tcx();
        tcx.for_each_free_region(&value, |live_region| {
            let live_region_vid =
                typeck.borrowck_context.universal_regions.to_region_vid(live_region);
            typeck
                .borrowck_context
                .constraints
                .liveness_constraints
                .add_elements(live_region_vid, live_at);
        });
    }

    fn compute_drop_data(
        typeck: &mut TypeChecker<'_, 'tcx>,
        dropped_ty: Ty<'tcx>,
    ) -> DropData<'tcx> {
        debug!("compute_drop_data(dropped_ty={:?})", dropped_ty,);

        let param_env = typeck.param_env;
        let TypeOpOutput { output, constraints, .. } =
            param_env.and(DropckOutlives::new(dropped_ty)).fully_perform(typeck.infcx).unwrap();

        DropData { dropck_result: output, region_constraint_data: constraints }
    }
}
