mod cache;
mod overflow;

pub(super) use overflow::OverflowHandler;
use rustc_middle::traits::solve::inspect::CacheHit;

use self::cache::ProvisionalEntry;
use cache::ProvisionalCache;
use overflow::OverflowData;
use rustc_index::IndexVec;
use rustc_middle::dep_graph::DepKind;
use rustc_middle::traits::solve::{CanonicalInput, Certainty, MaybeCause, QueryResult};
use rustc_middle::ty::TyCtxt;
use std::{collections::hash_map::Entry, mem};

use super::inspect::ProofTreeBuilder;
use super::SolverMode;

rustc_index::newtype_index! {
    pub struct StackDepth {}
}

struct StackElem<'tcx> {
    input: CanonicalInput<'tcx>,
    has_been_used: bool,
}

pub(super) struct SearchGraph<'tcx> {
    mode: SolverMode,
    /// The stack of goals currently being computed.
    ///
    /// An element is *deeper* in the stack if its index is *lower*.
    stack: IndexVec<StackDepth, StackElem<'tcx>>,
    overflow_data: OverflowData,
    provisional_cache: ProvisionalCache<'tcx>,
}

impl<'tcx> SearchGraph<'tcx> {
    pub(super) fn new(tcx: TyCtxt<'tcx>, mode: SolverMode) -> SearchGraph<'tcx> {
        Self {
            mode,
            stack: Default::default(),
            overflow_data: OverflowData::new(tcx),
            provisional_cache: ProvisionalCache::empty(),
        }
    }

    pub(super) fn solver_mode(&self) -> SolverMode {
        self.mode
    }

    /// We do not use the global cache during coherence.
    ///
    /// The trait solver behavior is different for coherence
    /// so we would have to add the solver mode to the cache key.
    /// This is probably not worth it as trait solving during
    /// coherence tends to already be incredibly fast.
    ///
    /// We could add another global cache for coherence instead,
    /// but that's effort so let's only do it if necessary.
    pub(super) fn should_use_global_cache(&self) -> bool {
        match self.mode {
            SolverMode::Normal => true,
            SolverMode::Coherence => false,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.stack.is_empty() && self.provisional_cache.is_empty()
    }

    /// Whether we're currently in a cycle. This should only be used
    /// for debug assertions.
    pub(super) fn in_cycle(&self) -> bool {
        if let Some(stack_depth) = self.stack.last_index() {
            // Either the current goal on the stack is the root of a cycle...
            if self.stack[stack_depth].has_been_used {
                return true;
            }

            // ...or it depends on a goal with a lower depth.
            let current_goal = self.stack[stack_depth].input;
            let entry_index = self.provisional_cache.lookup_table[&current_goal];
            self.provisional_cache.entries[entry_index].depth != stack_depth
        } else {
            false
        }
    }

    /// Tries putting the new goal on the stack, returning an error if it is already cached.
    ///
    /// This correctly updates the provisional cache if there is a cycle.
    #[instrument(level = "debug", skip(self, tcx, inspect), ret)]
    fn try_push_stack(
        &mut self,
        tcx: TyCtxt<'tcx>,
        input: CanonicalInput<'tcx>,
        inspect: &mut ProofTreeBuilder<'tcx>,
    ) -> Result<(), QueryResult<'tcx>> {
        // Look at the provisional cache to check for cycles.
        let cache = &mut self.provisional_cache;
        match cache.lookup_table.entry(input) {
            // No entry, simply push this goal on the stack after dealing with overflow.
            Entry::Vacant(v) => {
                if self.overflow_data.has_overflow(self.stack.len()) {
                    return Err(self.deal_with_overflow(tcx, input));
                }

                let depth = self.stack.push(StackElem { input, has_been_used: false });
                let response = super::response_no_constraints(tcx, input, Certainty::Yes);
                let entry_index = cache.entries.push(ProvisionalEntry { response, depth, input });
                v.insert(entry_index);
                Ok(())
            }
            // We have a nested goal which relies on a goal `root` deeper in the stack.
            //
            // We first store that we may have to rerun `evaluate_goal` for `root` in case the
            // provisional response is not equal to the final response. We also update the depth
            // of all goals which recursively depend on our current goal to depend on `root`
            // instead.
            //
            // Finally we can return either the provisional response for that goal if we have a
            // coinductive cycle or an ambiguous result if the cycle is inductive.
            Entry::Occupied(entry_index) => {
                inspect.cache_hit(CacheHit::Provisional);

                let entry_index = *entry_index.get();

                let stack_depth = cache.depth(entry_index);
                debug!("encountered cycle with depth {stack_depth:?}");

                cache.add_dependency_of_leaf_on(entry_index);

                self.stack[stack_depth].has_been_used = true;
                // NOTE: The goals on the stack aren't the only goals involved in this cycle.
                // We can also depend on goals which aren't part of the stack but coinductively
                // depend on the stack themselves. We already checked whether all the goals
                // between these goals and their root on the stack. This means that as long as
                // each goal in a cycle is checked for coinductivity by itself, simply checking
                // the stack is enough.
                if self.stack.raw[stack_depth.index()..]
                    .iter()
                    .all(|g| g.input.value.goal.predicate.is_coinductive(tcx))
                {
                    Err(cache.provisional_result(entry_index))
                } else {
                    Err(super::response_no_constraints(
                        tcx,
                        input,
                        Certainty::Maybe(MaybeCause::Overflow),
                    ))
                }
            }
        }
    }

    /// We cannot simply store the result of [super::EvalCtxt::compute_goal] as we have to deal with
    /// coinductive cycles.
    ///
    /// When we encounter a coinductive cycle, we have to prove the final result of that cycle
    /// while we are still computing that result. Because of this we continuously recompute the
    /// cycle until the result of the previous iteration is equal to the final result, at which
    /// point we are done.
    ///
    /// This function returns `true` if we were able to finalize the goal and `false` if it has
    /// updated the provisional cache and we have to recompute the current goal.
    ///
    /// FIXME: Refer to the rustc-dev-guide entry once it exists.
    #[instrument(level = "debug", skip(self, actual_input), ret)]
    fn try_finalize_goal(
        &mut self,
        actual_input: CanonicalInput<'tcx>,
        response: QueryResult<'tcx>,
    ) -> bool {
        let stack_elem = self.stack.pop().unwrap();
        let StackElem { input, has_been_used } = stack_elem;
        assert_eq!(input, actual_input);

        let cache = &mut self.provisional_cache;
        let provisional_entry_index = *cache.lookup_table.get(&input).unwrap();
        let provisional_entry = &mut cache.entries[provisional_entry_index];
        // We eagerly update the response in the cache here. If we have to reevaluate
        // this goal we use the new response when hitting a cycle, and we definitely
        // want to access the final response whenever we look at the cache.
        let prev_response = mem::replace(&mut provisional_entry.response, response);

        // Was the current goal the root of a cycle and was the provisional response
        // different from the final one.
        if has_been_used && prev_response != response {
            // If so, remove all entries whose result depends on this goal
            // from the provisional cache...
            //
            // That's not completely correct, as a nested goal can also
            // depend on a goal which is lower in the stack so it doesn't
            // actually depend on the current goal. This should be fairly
            // rare and is hopefully not relevant for performance.
            #[allow(rustc::potential_query_instability)]
            cache.lookup_table.retain(|_key, index| *index <= provisional_entry_index);
            cache.entries.truncate(provisional_entry_index.index() + 1);

            // ...and finally push our goal back on the stack and reevaluate it.
            self.stack.push(StackElem { input, has_been_used: false });
            false
        } else {
            true
        }
    }

    pub(super) fn with_new_goal(
        &mut self,
        tcx: TyCtxt<'tcx>,
        canonical_input: CanonicalInput<'tcx>,
        inspect: &mut ProofTreeBuilder<'tcx>,
        mut loop_body: impl FnMut(&mut Self, &mut ProofTreeBuilder<'tcx>) -> QueryResult<'tcx>,
    ) -> QueryResult<'tcx> {
        if self.should_use_global_cache() && inspect.use_global_cache() {
            if let Some(result) = tcx.new_solver_evaluation_cache.get(&canonical_input, tcx) {
                debug!(?canonical_input, ?result, "cache hit");
                inspect.cache_hit(CacheHit::Global);
                return result;
            }
        }

        match self.try_push_stack(tcx, canonical_input, inspect) {
            Ok(()) => {}
            // Our goal is already on the stack, eager return.
            Err(response) => return response,
        }

        // This is for global caching, so we properly track query dependencies.
        // Everything that affects the `Result` should be performed within this
        // `with_anon_task` closure.
        let (result, dep_node) = tcx.dep_graph.with_anon_task(tcx, DepKind::TraitSelect, || {
            self.repeat_while_none(
                |this| {
                    let result = this.deal_with_overflow(tcx, canonical_input);
                    let _ = this.stack.pop().unwrap();
                    result
                },
                |this| {
                    let result = loop_body(this, inspect);
                    this.try_finalize_goal(canonical_input, result).then(|| result)
                },
            )
        });

        let cache = &mut self.provisional_cache;
        let provisional_entry_index = *cache.lookup_table.get(&canonical_input).unwrap();
        let provisional_entry = &mut cache.entries[provisional_entry_index];
        let depth = provisional_entry.depth;

        // If not, we're done with this goal.
        //
        // Check whether that this goal doesn't depend on a goal deeper on the stack
        // and if so, move it to the global cache.
        //
        // Note that if any nested goal were to depend on something deeper on the stack,
        // this would have also updated the depth of the current goal.
        if depth == self.stack.next_index() {
            // If the current goal is the head of a cycle, we drop all other
            // cycle participants without moving them to the global cache.
            let other_cycle_participants = provisional_entry_index.index() + 1;
            for (i, entry) in cache.entries.drain_enumerated(other_cycle_participants..) {
                let actual_index = cache.lookup_table.remove(&entry.input);
                debug_assert_eq!(Some(i), actual_index);
                debug_assert!(entry.depth == depth);
            }

            let current_goal = cache.entries.pop().unwrap();
            let actual_index = cache.lookup_table.remove(&current_goal.input);
            debug_assert_eq!(Some(provisional_entry_index), actual_index);
            debug_assert!(current_goal.depth == depth);

            // We move the root goal to the global cache if we either did not hit an overflow or if it's
            // the root goal as that will now always hit the same overflow limit.
            //
            // NOTE: We cannot move any non-root goals to the global cache. When replaying the root goal's
            // dependencies, our non-root goal may no longer appear as child of the root goal.
            //
            // See https://github.com/rust-lang/rust/pull/108071 for some additional context.
            let can_cache = !self.overflow_data.did_overflow() || self.stack.is_empty();
            if self.should_use_global_cache() && can_cache {
                tcx.new_solver_evaluation_cache.insert(
                    current_goal.input,
                    dep_node,
                    current_goal.response,
                );
            }
        }

        result
    }
}
