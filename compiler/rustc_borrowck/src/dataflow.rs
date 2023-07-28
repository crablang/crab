#![deny(rustc::untranslatable_diagnostic)]
#![deny(rustc::diagnostic_outside_of_impl)]
use rustc_data_structures::fx::FxIndexMap;
use rustc_index::bit_set::BitSet;
use rustc_middle::mir::{self, BasicBlock, Body, Location, Place};
use rustc_middle::ty::RegionVid;
use rustc_middle::ty::TyCtxt;
use rustc_mir_dataflow::impls::{EverInitializedPlaces, MaybeUninitializedPlaces};
use rustc_mir_dataflow::ResultsVisitable;
use rustc_mir_dataflow::{self, fmt::DebugWithContext, CallReturnPlaces, GenKill};
use rustc_mir_dataflow::{Analysis, Direction, Results};
use std::fmt;

use crate::{places_conflict, BorrowSet, PlaceConflictBias, PlaceExt, RegionInferenceContext};

/// A tuple with named fields that can hold either the results or the transient state of the
/// dataflow analyses used by the borrow checker.
#[derive(Debug)]
pub struct BorrowckAnalyses<B, U, E> {
    pub borrows: B,
    pub uninits: U,
    pub ever_inits: E,
}

/// The results of the dataflow analyses used by the borrow checker.
pub type BorrowckResults<'mir, 'tcx> = BorrowckAnalyses<
    Results<'tcx, Borrows<'mir, 'tcx>>,
    Results<'tcx, MaybeUninitializedPlaces<'mir, 'tcx>>,
    Results<'tcx, EverInitializedPlaces<'mir, 'tcx>>,
>;

/// The transient state of the dataflow analyses used by the borrow checker.
pub type BorrowckFlowState<'mir, 'tcx> =
    <BorrowckResults<'mir, 'tcx> as ResultsVisitable<'tcx>>::FlowState;

macro_rules! impl_visitable {
    ( $(
        $T:ident { $( $field:ident : $A:ident ),* $(,)? }
    )* ) => { $(
        impl<'tcx, $($A),*, D: Direction> ResultsVisitable<'tcx> for $T<$( Results<'tcx, $A> ),*>
        where
            $( $A: Analysis<'tcx, Direction = D>, )*
        {
            type Direction = D;
            type FlowState = $T<$( $A::Domain ),*>;

            fn new_flow_state(&self, body: &mir::Body<'tcx>) -> Self::FlowState {
                $T {
                    $( $field: self.$field.analysis.bottom_value(body) ),*
                }
            }

            fn reset_to_block_entry(
                &self,
                state: &mut Self::FlowState,
                block: BasicBlock,
            ) {
                $( state.$field.clone_from(&self.$field.entry_set_for_block(block)); )*
            }

            fn reconstruct_before_statement_effect(
                &mut self,
                state: &mut Self::FlowState,
                stmt: &mir::Statement<'tcx>,
                loc: Location,
            ) {
                $( self.$field.analysis
                    .apply_before_statement_effect(&mut state.$field, stmt, loc); )*
            }

            fn reconstruct_statement_effect(
                &mut self,
                state: &mut Self::FlowState,
                stmt: &mir::Statement<'tcx>,
                loc: Location,
            ) {
                $( self.$field.analysis
                    .apply_statement_effect(&mut state.$field, stmt, loc); )*
            }

            fn reconstruct_before_terminator_effect(
                &mut self,
                state: &mut Self::FlowState,
                term: &mir::Terminator<'tcx>,
                loc: Location,
            ) {
                $( self.$field.analysis
                    .apply_before_terminator_effect(&mut state.$field, term, loc); )*
            }

            fn reconstruct_terminator_effect(
                &mut self,
                state: &mut Self::FlowState,
                term: &mir::Terminator<'tcx>,
                loc: Location,
            ) {
                $( self.$field.analysis
                    .apply_terminator_effect(&mut state.$field, term, loc); )*
            }
        }
    )* }
}

impl_visitable! {
    BorrowckAnalyses { borrows: B, uninits: U, ever_inits: E }
}

rustc_index::newtype_index! {
    #[debug_format = "bw{}"]
    pub struct BorrowIndex {}
}

/// `Borrows` stores the data used in the analyses that track the flow
/// of borrows.
///
/// It uniquely identifies every borrow (`Rvalue::Ref`) by a
/// `BorrowIndex`, and maps each such index to a `BorrowData`
/// describing the borrow. These indexes are used for representing the
/// borrows in compact bitvectors.
pub struct Borrows<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    body: &'a Body<'tcx>,

    borrow_set: &'a BorrowSet<'tcx>,
    borrows_out_of_scope_at_location: FxIndexMap<Location, Vec<BorrowIndex>>,
}

struct OutOfScopePrecomputer<'a, 'tcx> {
    visited: BitSet<mir::BasicBlock>,
    visit_stack: Vec<mir::BasicBlock>,
    body: &'a Body<'tcx>,
    regioncx: &'a RegionInferenceContext<'tcx>,
    borrows_out_of_scope_at_location: FxIndexMap<Location, Vec<BorrowIndex>>,
}

impl<'a, 'tcx> OutOfScopePrecomputer<'a, 'tcx> {
    fn new(body: &'a Body<'tcx>, regioncx: &'a RegionInferenceContext<'tcx>) -> Self {
        OutOfScopePrecomputer {
            visited: BitSet::new_empty(body.basic_blocks.len()),
            visit_stack: vec![],
            body,
            regioncx,
            borrows_out_of_scope_at_location: FxIndexMap::default(),
        }
    }
}

impl<'tcx> OutOfScopePrecomputer<'_, 'tcx> {
    fn precompute_borrows_out_of_scope(
        &mut self,
        borrow_index: BorrowIndex,
        borrow_region: RegionVid,
        first_location: Location,
    ) {
        let first_block = first_location.block;
        let first_bb_data = &self.body.basic_blocks[first_block];

        // This is the first block, we only want to visit it from the creation of the borrow at
        // `first_location`.
        let first_lo = first_location.statement_index;
        let first_hi = first_bb_data.statements.len();

        if let Some(kill_stmt) = self.regioncx.first_non_contained_inclusive(
            borrow_region,
            first_block,
            first_lo,
            first_hi,
        ) {
            let kill_location = Location { block: first_block, statement_index: kill_stmt };
            // If region does not contain a point at the location, then add to list and skip
            // successor locations.
            debug!("borrow {:?} gets killed at {:?}", borrow_index, kill_location);
            self.borrows_out_of_scope_at_location
                .entry(kill_location)
                .or_default()
                .push(borrow_index);

            // The borrow is already dead, there is no need to visit other blocks.
            return;
        }

        // The borrow is not dead. Add successor BBs to the work list, if necessary.
        for succ_bb in first_bb_data.terminator().successors() {
            if self.visited.insert(succ_bb) {
                self.visit_stack.push(succ_bb);
            }
        }

        // We may end up visiting `first_block` again. This is not an issue: we know at this point
        // that it does not kill the borrow in the `first_lo..=first_hi` range, so checking the
        // `0..first_lo` range and the `0..first_hi` range give the same result.
        while let Some(block) = self.visit_stack.pop() {
            let bb_data = &self.body[block];
            let num_stmts = bb_data.statements.len();
            if let Some(kill_stmt) =
                self.regioncx.first_non_contained_inclusive(borrow_region, block, 0, num_stmts)
            {
                let kill_location = Location { block, statement_index: kill_stmt };
                // If region does not contain a point at the location, then add to list and skip
                // successor locations.
                debug!("borrow {:?} gets killed at {:?}", borrow_index, kill_location);
                self.borrows_out_of_scope_at_location
                    .entry(kill_location)
                    .or_default()
                    .push(borrow_index);

                // We killed the borrow, so we do not visit this block's successors.
                continue;
            }

            // Add successor BBs to the work list, if necessary.
            for succ_bb in bb_data.terminator().successors() {
                if self.visited.insert(succ_bb) {
                    self.visit_stack.push(succ_bb);
                }
            }
        }

        self.visited.clear();
    }
}

pub fn calculate_borrows_out_of_scope_at_location<'tcx>(
    body: &Body<'tcx>,
    regioncx: &RegionInferenceContext<'tcx>,
    borrow_set: &BorrowSet<'tcx>,
) -> FxIndexMap<Location, Vec<BorrowIndex>> {
    let mut prec = OutOfScopePrecomputer::new(body, regioncx);
    for (borrow_index, borrow_data) in borrow_set.iter_enumerated() {
        let borrow_region = borrow_data.region;
        let location = borrow_data.reserve_location;

        prec.precompute_borrows_out_of_scope(borrow_index, borrow_region, location);
    }

    prec.borrows_out_of_scope_at_location
}

impl<'a, 'tcx> Borrows<'a, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        body: &'a Body<'tcx>,
        nonlexical_regioncx: &'a RegionInferenceContext<'tcx>,
        borrow_set: &'a BorrowSet<'tcx>,
    ) -> Self {
        let borrows_out_of_scope_at_location =
            calculate_borrows_out_of_scope_at_location(body, nonlexical_regioncx, borrow_set);
        Borrows { tcx, body, borrow_set, borrows_out_of_scope_at_location }
    }

    pub fn location(&self, idx: BorrowIndex) -> &Location {
        &self.borrow_set[idx].reserve_location
    }

    /// Add all borrows to the kill set, if those borrows are out of scope at `location`.
    /// That means they went out of a nonlexical scope
    fn kill_loans_out_of_scope_at_location(
        &self,
        trans: &mut impl GenKill<BorrowIndex>,
        location: Location,
    ) {
        // NOTE: The state associated with a given `location`
        // reflects the dataflow on entry to the statement.
        // Iterate over each of the borrows that we've precomputed
        // to have went out of scope at this location and kill them.
        //
        // We are careful always to call this function *before* we
        // set up the gen-bits for the statement or
        // terminator. That way, if the effect of the statement or
        // terminator *does* introduce a new loan of the same
        // region, then setting that gen-bit will override any
        // potential kill introduced here.
        if let Some(indices) = self.borrows_out_of_scope_at_location.get(&location) {
            trans.kill_all(indices.iter().copied());
        }
    }

    /// Kill any borrows that conflict with `place`.
    fn kill_borrows_on_place(&self, trans: &mut impl GenKill<BorrowIndex>, place: Place<'tcx>) {
        debug!("kill_borrows_on_place: place={:?}", place);

        let other_borrows_of_local = self
            .borrow_set
            .local_map
            .get(&place.local)
            .into_iter()
            .flat_map(|bs| bs.iter())
            .copied();

        // If the borrowed place is a local with no projections, all other borrows of this
        // local must conflict. This is purely an optimization so we don't have to call
        // `places_conflict` for every borrow.
        if place.projection.is_empty() {
            if !self.body.local_decls[place.local].is_ref_to_static() {
                trans.kill_all(other_borrows_of_local);
            }
            return;
        }

        // By passing `PlaceConflictBias::NoOverlap`, we conservatively assume that any given
        // pair of array indices are not equal, so that when `places_conflict` returns true, we
        // will be assured that two places being compared definitely denotes the same sets of
        // locations.
        let definitely_conflicting_borrows = other_borrows_of_local.filter(|&i| {
            places_conflict(
                self.tcx,
                self.body,
                self.borrow_set[i].borrowed_place,
                place,
                PlaceConflictBias::NoOverlap,
            )
        });

        trans.kill_all(definitely_conflicting_borrows);
    }
}

impl<'tcx> rustc_mir_dataflow::AnalysisDomain<'tcx> for Borrows<'_, 'tcx> {
    type Domain = BitSet<BorrowIndex>;

    const NAME: &'static str = "borrows";

    fn bottom_value(&self, _: &mir::Body<'tcx>) -> Self::Domain {
        // bottom = nothing is reserved or activated yet;
        BitSet::new_empty(self.borrow_set.len())
    }

    fn initialize_start_block(&self, _: &mir::Body<'tcx>, _: &mut Self::Domain) {
        // no borrows of code region_scopes have been taken prior to
        // function execution, so this method has no effect.
    }
}

impl<'tcx> rustc_mir_dataflow::GenKillAnalysis<'tcx> for Borrows<'_, 'tcx> {
    type Idx = BorrowIndex;

    fn before_statement_effect(
        &mut self,
        trans: &mut impl GenKill<Self::Idx>,
        _statement: &mir::Statement<'tcx>,
        location: Location,
    ) {
        self.kill_loans_out_of_scope_at_location(trans, location);
    }

    fn statement_effect(
        &mut self,
        trans: &mut impl GenKill<Self::Idx>,
        stmt: &mir::Statement<'tcx>,
        location: Location,
    ) {
        match &stmt.kind {
            mir::StatementKind::Assign(box (lhs, rhs)) => {
                if let mir::Rvalue::Ref(_, _, place) = rhs {
                    if place.ignore_borrow(
                        self.tcx,
                        self.body,
                        &self.borrow_set.locals_state_at_exit,
                    ) {
                        return;
                    }
                    let index = self.borrow_set.get_index_of(&location).unwrap_or_else(|| {
                        panic!("could not find BorrowIndex for location {location:?}");
                    });

                    trans.gen(index);
                }

                // Make sure there are no remaining borrows for variables
                // that are assigned over.
                self.kill_borrows_on_place(trans, *lhs);
            }

            mir::StatementKind::StorageDead(local) => {
                // Make sure there are no remaining borrows for locals that
                // are gone out of scope.
                self.kill_borrows_on_place(trans, Place::from(*local));
            }

            mir::StatementKind::FakeRead(..)
            | mir::StatementKind::SetDiscriminant { .. }
            | mir::StatementKind::Deinit(..)
            | mir::StatementKind::StorageLive(..)
            | mir::StatementKind::Retag { .. }
            | mir::StatementKind::PlaceMention(..)
            | mir::StatementKind::AscribeUserType(..)
            | mir::StatementKind::Coverage(..)
            | mir::StatementKind::Intrinsic(..)
            | mir::StatementKind::ConstEvalCounter
            | mir::StatementKind::Nop => {}
        }
    }

    fn before_terminator_effect(
        &mut self,
        trans: &mut impl GenKill<Self::Idx>,
        _terminator: &mir::Terminator<'tcx>,
        location: Location,
    ) {
        self.kill_loans_out_of_scope_at_location(trans, location);
    }

    fn terminator_effect(
        &mut self,
        trans: &mut impl GenKill<Self::Idx>,
        terminator: &mir::Terminator<'tcx>,
        _location: Location,
    ) {
        if let mir::TerminatorKind::InlineAsm { operands, .. } = &terminator.kind {
            for op in operands {
                if let mir::InlineAsmOperand::Out { place: Some(place), .. }
                | mir::InlineAsmOperand::InOut { out_place: Some(place), .. } = *op
                {
                    self.kill_borrows_on_place(trans, place);
                }
            }
        }
    }

    fn call_return_effect(
        &mut self,
        _trans: &mut impl GenKill<Self::Idx>,
        _block: mir::BasicBlock,
        _return_places: CallReturnPlaces<'_, 'tcx>,
    ) {
    }
}

impl DebugWithContext<Borrows<'_, '_>> for BorrowIndex {
    fn fmt_with(&self, ctxt: &Borrows<'_, '_>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", ctxt.location(*self))
    }
}
