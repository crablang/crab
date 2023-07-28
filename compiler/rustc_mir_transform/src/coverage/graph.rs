use super::Error;

use itertools::Itertools;
use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::graph::dominators::{self, Dominators};
use rustc_data_structures::graph::{self, GraphSuccessors, WithNumNodes, WithStartNode};
use rustc_index::bit_set::BitSet;
use rustc_index::{IndexSlice, IndexVec};
use rustc_middle::mir::coverage::*;
use rustc_middle::mir::{self, BasicBlock, BasicBlockData, Terminator, TerminatorKind};

use std::cmp::Ordering;
use std::ops::{Index, IndexMut};

const ID_SEPARATOR: &str = ",";

/// A coverage-specific simplification of the MIR control flow graph (CFG). The `CoverageGraph`s
/// nodes are `BasicCoverageBlock`s, which encompass one or more MIR `BasicBlock`s, plus a
/// `CoverageKind` counter (to be added by `CoverageCounters::make_bcb_counters`), and an optional
/// set of additional counters--if needed--to count incoming edges, if there are more than one.
/// (These "edge counters" are eventually converted into new MIR `BasicBlock`s.)
#[derive(Debug)]
pub(super) struct CoverageGraph {
    bcbs: IndexVec<BasicCoverageBlock, BasicCoverageBlockData>,
    bb_to_bcb: IndexVec<BasicBlock, Option<BasicCoverageBlock>>,
    pub successors: IndexVec<BasicCoverageBlock, Vec<BasicCoverageBlock>>,
    pub predecessors: IndexVec<BasicCoverageBlock, Vec<BasicCoverageBlock>>,
    dominators: Option<Dominators<BasicCoverageBlock>>,
}

impl CoverageGraph {
    pub fn from_mir(mir_body: &mir::Body<'_>) -> Self {
        let (bcbs, bb_to_bcb) = Self::compute_basic_coverage_blocks(mir_body);

        // Pre-transform MIR `BasicBlock` successors and predecessors into the BasicCoverageBlock
        // equivalents. Note that since the BasicCoverageBlock graph has been fully simplified, the
        // each predecessor of a BCB leader_bb should be in a unique BCB. It is possible for a
        // `SwitchInt` to have multiple targets to the same destination `BasicBlock`, so
        // de-duplication is required. This is done without reordering the successors.

        let mut seen = IndexVec::from_elem(false, &bcbs);
        let successors = IndexVec::from_fn_n(
            |bcb| {
                for b in seen.iter_mut() {
                    *b = false;
                }
                let bcb_data = &bcbs[bcb];
                let mut bcb_successors = Vec::new();
                for successor in
                    bcb_filtered_successors(&mir_body, &bcb_data.terminator(mir_body).kind)
                        .filter_map(|successor_bb| bb_to_bcb[successor_bb])
                {
                    if !seen[successor] {
                        seen[successor] = true;
                        bcb_successors.push(successor);
                    }
                }
                bcb_successors
            },
            bcbs.len(),
        );

        let mut predecessors = IndexVec::from_elem(Vec::new(), &bcbs);
        for (bcb, bcb_successors) in successors.iter_enumerated() {
            for &successor in bcb_successors {
                predecessors[successor].push(bcb);
            }
        }

        let mut basic_coverage_blocks =
            Self { bcbs, bb_to_bcb, successors, predecessors, dominators: None };
        let dominators = dominators::dominators(&basic_coverage_blocks);
        basic_coverage_blocks.dominators = Some(dominators);
        basic_coverage_blocks
    }

    fn compute_basic_coverage_blocks(
        mir_body: &mir::Body<'_>,
    ) -> (
        IndexVec<BasicCoverageBlock, BasicCoverageBlockData>,
        IndexVec<BasicBlock, Option<BasicCoverageBlock>>,
    ) {
        let num_basic_blocks = mir_body.basic_blocks.len();
        let mut bcbs = IndexVec::with_capacity(num_basic_blocks);
        let mut bb_to_bcb = IndexVec::from_elem_n(None, num_basic_blocks);

        // Walk the MIR CFG using a Preorder traversal, which starts from `START_BLOCK` and follows
        // each block terminator's `successors()`. Coverage spans must map to actual source code,
        // so compiler generated blocks and paths can be ignored. To that end, the CFG traversal
        // intentionally omits unwind paths.
        // FIXME(#78544): MIR InstrumentCoverage: Improve coverage of `#[should_panic]` tests and
        // `catch_unwind()` handlers.
        let mir_cfg_without_unwind = ShortCircuitPreorder::new(&mir_body, bcb_filtered_successors);

        let mut basic_blocks = Vec::new();
        for (bb, data) in mir_cfg_without_unwind {
            if let Some(last) = basic_blocks.last() {
                let predecessors = &mir_body.basic_blocks.predecessors()[bb];
                if predecessors.len() > 1 || !predecessors.contains(last) {
                    // The `bb` has more than one _incoming_ edge, and should start its own
                    // `BasicCoverageBlockData`. (Note, the `basic_blocks` vector does not yet
                    // include `bb`; it contains a sequence of one or more sequential basic_blocks
                    // with no intermediate branches in or out. Save these as a new
                    // `BasicCoverageBlockData` before starting the new one.)
                    Self::add_basic_coverage_block(
                        &mut bcbs,
                        &mut bb_to_bcb,
                        basic_blocks.split_off(0),
                    );
                    debug!(
                        "  because {}",
                        if predecessors.len() > 1 {
                            "predecessors.len() > 1".to_owned()
                        } else {
                            format!("bb {} is not in predecessors: {:?}", bb.index(), predecessors)
                        }
                    );
                }
            }
            basic_blocks.push(bb);

            let term = data.terminator();

            match term.kind {
                TerminatorKind::Return { .. }
                | TerminatorKind::Terminate
                | TerminatorKind::Yield { .. }
                | TerminatorKind::SwitchInt { .. } => {
                    // The `bb` has more than one _outgoing_ edge, or exits the function. Save the
                    // current sequence of `basic_blocks` gathered to this point, as a new
                    // `BasicCoverageBlockData`.
                    Self::add_basic_coverage_block(
                        &mut bcbs,
                        &mut bb_to_bcb,
                        basic_blocks.split_off(0),
                    );
                    debug!("  because term.kind = {:?}", term.kind);
                    // Note that this condition is based on `TerminatorKind`, even though it
                    // theoretically boils down to `successors().len() != 1`; that is, either zero
                    // (e.g., `Return`, `Terminate`) or multiple successors (e.g., `SwitchInt`), but
                    // since the BCB CFG ignores things like unwind branches (which exist in the
                    // `Terminator`s `successors()` list) checking the number of successors won't
                    // work.
                }

                // The following `TerminatorKind`s are either not expected outside an unwind branch,
                // or they should not (under normal circumstances) branch. Coverage graphs are
                // simplified by assuring coverage results are accurate for program executions that
                // don't panic.
                //
                // Programs that panic and unwind may record slightly inaccurate coverage results
                // for a coverage region containing the `Terminator` that began the panic. This
                // is as intended. (See Issue #78544 for a possible future option to support
                // coverage in test programs that panic.)
                TerminatorKind::Goto { .. }
                | TerminatorKind::Resume
                | TerminatorKind::Unreachable
                | TerminatorKind::Drop { .. }
                | TerminatorKind::Call { .. }
                | TerminatorKind::GeneratorDrop
                | TerminatorKind::Assert { .. }
                | TerminatorKind::FalseEdge { .. }
                | TerminatorKind::FalseUnwind { .. }
                | TerminatorKind::InlineAsm { .. } => {}
            }
        }

        if !basic_blocks.is_empty() {
            // process any remaining basic_blocks into a final `BasicCoverageBlockData`
            Self::add_basic_coverage_block(&mut bcbs, &mut bb_to_bcb, basic_blocks.split_off(0));
            debug!("  because the end of the MIR CFG was reached while traversing");
        }

        (bcbs, bb_to_bcb)
    }

    fn add_basic_coverage_block(
        bcbs: &mut IndexVec<BasicCoverageBlock, BasicCoverageBlockData>,
        bb_to_bcb: &mut IndexSlice<BasicBlock, Option<BasicCoverageBlock>>,
        basic_blocks: Vec<BasicBlock>,
    ) {
        let bcb = bcbs.next_index();
        for &bb in basic_blocks.iter() {
            bb_to_bcb[bb] = Some(bcb);
        }
        let bcb_data = BasicCoverageBlockData::from(basic_blocks);
        debug!("adding bcb{}: {:?}", bcb.index(), bcb_data);
        bcbs.push(bcb_data);
    }

    #[inline(always)]
    pub fn iter_enumerated(
        &self,
    ) -> impl Iterator<Item = (BasicCoverageBlock, &BasicCoverageBlockData)> {
        self.bcbs.iter_enumerated()
    }

    #[inline(always)]
    pub fn iter_enumerated_mut(
        &mut self,
    ) -> impl Iterator<Item = (BasicCoverageBlock, &mut BasicCoverageBlockData)> {
        self.bcbs.iter_enumerated_mut()
    }

    #[inline(always)]
    pub fn bcb_from_bb(&self, bb: BasicBlock) -> Option<BasicCoverageBlock> {
        if bb.index() < self.bb_to_bcb.len() { self.bb_to_bcb[bb] } else { None }
    }

    #[inline(always)]
    pub fn dominates(&self, dom: BasicCoverageBlock, node: BasicCoverageBlock) -> bool {
        self.dominators.as_ref().unwrap().dominates(dom, node)
    }

    #[inline(always)]
    pub fn rank_partial_cmp(
        &self,
        a: BasicCoverageBlock,
        b: BasicCoverageBlock,
    ) -> Option<Ordering> {
        self.dominators.as_ref().unwrap().rank_partial_cmp(a, b)
    }
}

impl Index<BasicCoverageBlock> for CoverageGraph {
    type Output = BasicCoverageBlockData;

    #[inline]
    fn index(&self, index: BasicCoverageBlock) -> &BasicCoverageBlockData {
        &self.bcbs[index]
    }
}

impl IndexMut<BasicCoverageBlock> for CoverageGraph {
    #[inline]
    fn index_mut(&mut self, index: BasicCoverageBlock) -> &mut BasicCoverageBlockData {
        &mut self.bcbs[index]
    }
}

impl graph::DirectedGraph for CoverageGraph {
    type Node = BasicCoverageBlock;
}

impl graph::WithNumNodes for CoverageGraph {
    #[inline]
    fn num_nodes(&self) -> usize {
        self.bcbs.len()
    }
}

impl graph::WithStartNode for CoverageGraph {
    #[inline]
    fn start_node(&self) -> Self::Node {
        self.bcb_from_bb(mir::START_BLOCK)
            .expect("mir::START_BLOCK should be in a BasicCoverageBlock")
    }
}

type BcbSuccessors<'graph> = std::slice::Iter<'graph, BasicCoverageBlock>;

impl<'graph> graph::GraphSuccessors<'graph> for CoverageGraph {
    type Item = BasicCoverageBlock;
    type Iter = std::iter::Cloned<BcbSuccessors<'graph>>;
}

impl graph::WithSuccessors for CoverageGraph {
    #[inline]
    fn successors(&self, node: Self::Node) -> <Self as GraphSuccessors<'_>>::Iter {
        self.successors[node].iter().cloned()
    }
}

impl<'graph> graph::GraphPredecessors<'graph> for CoverageGraph {
    type Item = BasicCoverageBlock;
    type Iter = std::iter::Copied<std::slice::Iter<'graph, BasicCoverageBlock>>;
}

impl graph::WithPredecessors for CoverageGraph {
    #[inline]
    fn predecessors(&self, node: Self::Node) -> <Self as graph::GraphPredecessors<'_>>::Iter {
        self.predecessors[node].iter().copied()
    }
}

rustc_index::newtype_index! {
    /// A node in the control-flow graph of CoverageGraph.
    #[debug_format = "bcb{}"]
    pub(super) struct BasicCoverageBlock {
        const START_BCB = 0;
    }
}

/// `BasicCoverageBlockData` holds the data indexed by a `BasicCoverageBlock`.
///
/// A `BasicCoverageBlock` (BCB) represents the maximal-length sequence of MIR `BasicBlock`s without
/// conditional branches, and form a new, simplified, coverage-specific Control Flow Graph, without
/// altering the original MIR CFG.
///
/// Note that running the MIR `SimplifyCfg` transform is not sufficient (and therefore not
/// necessary). The BCB-based CFG is a more aggressive simplification. For example:
///
///   * The BCB CFG ignores (trims) branches not relevant to coverage, such as unwind-related code,
///     that is injected by the Rust compiler but has no physical source code to count. This also
///     means a BasicBlock with a `Call` terminator can be merged into its primary successor target
///     block, in the same BCB. (But, note: Issue #78544: "MIR InstrumentCoverage: Improve coverage
///     of `#[should_panic]` tests and `catch_unwind()` handlers")
///   * Some BasicBlock terminators support Rust-specific concerns--like borrow-checking--that are
///     not relevant to coverage analysis. `FalseUnwind`, for example, can be treated the same as
///     a `Goto`, and merged with its successor into the same BCB.
///
/// Each BCB with at least one computed `CoverageSpan` will have no more than one `Counter`.
/// In some cases, a BCB's execution count can be computed by `Expression`. Additional
/// disjoint `CoverageSpan`s in a BCB can also be counted by `Expression` (by adding `ZERO`
/// to the BCB's primary counter or expression).
///
/// The BCB CFG is critical to simplifying the coverage analysis by ensuring graph path-based
/// queries (`dominates()`, `predecessors`, `successors`, etc.) have branch (control flow)
/// significance.
#[derive(Debug, Clone)]
pub(super) struct BasicCoverageBlockData {
    pub basic_blocks: Vec<BasicBlock>,
    pub counter_kind: Option<CoverageKind>,
    edge_from_bcbs: Option<FxHashMap<BasicCoverageBlock, CoverageKind>>,
}

impl BasicCoverageBlockData {
    pub fn from(basic_blocks: Vec<BasicBlock>) -> Self {
        assert!(basic_blocks.len() > 0);
        Self { basic_blocks, counter_kind: None, edge_from_bcbs: None }
    }

    #[inline(always)]
    pub fn leader_bb(&self) -> BasicBlock {
        self.basic_blocks[0]
    }

    #[inline(always)]
    pub fn last_bb(&self) -> BasicBlock {
        *self.basic_blocks.last().unwrap()
    }

    #[inline(always)]
    pub fn terminator<'a, 'tcx>(&self, mir_body: &'a mir::Body<'tcx>) -> &'a Terminator<'tcx> {
        &mir_body[self.last_bb()].terminator()
    }

    pub fn set_counter(
        &mut self,
        counter_kind: CoverageKind,
    ) -> Result<ExpressionOperandId, Error> {
        debug_assert!(
            // If the BCB has an edge counter (to be injected into a new `BasicBlock`), it can also
            // have an expression (to be injected into an existing `BasicBlock` represented by this
            // `BasicCoverageBlock`).
            self.edge_from_bcbs.is_none() || counter_kind.is_expression(),
            "attempt to add a `Counter` to a BCB target with existing incoming edge counters"
        );
        let operand = counter_kind.as_operand_id();
        if let Some(replaced) = self.counter_kind.replace(counter_kind) {
            Error::from_string(format!(
                "attempt to set a BasicCoverageBlock coverage counter more than once; \
                {self:?} already had counter {replaced:?}",
            ))
        } else {
            Ok(operand)
        }
    }

    #[inline(always)]
    pub fn counter(&self) -> Option<&CoverageKind> {
        self.counter_kind.as_ref()
    }

    #[inline(always)]
    pub fn take_counter(&mut self) -> Option<CoverageKind> {
        self.counter_kind.take()
    }

    pub fn set_edge_counter_from(
        &mut self,
        from_bcb: BasicCoverageBlock,
        counter_kind: CoverageKind,
    ) -> Result<ExpressionOperandId, Error> {
        if level_enabled!(tracing::Level::DEBUG) {
            // If the BCB has an edge counter (to be injected into a new `BasicBlock`), it can also
            // have an expression (to be injected into an existing `BasicBlock` represented by this
            // `BasicCoverageBlock`).
            if self.counter_kind.as_ref().is_some_and(|c| !c.is_expression()) {
                return Error::from_string(format!(
                    "attempt to add an incoming edge counter from {from_bcb:?} when the target BCB already \
                    has a `Counter`"
                ));
            }
        }
        let operand = counter_kind.as_operand_id();
        if let Some(replaced) =
            self.edge_from_bcbs.get_or_insert_default().insert(from_bcb, counter_kind)
        {
            Error::from_string(format!(
                "attempt to set an edge counter more than once; from_bcb: \
                {from_bcb:?} already had counter {replaced:?}",
            ))
        } else {
            Ok(operand)
        }
    }

    #[inline]
    pub fn edge_counter_from(&self, from_bcb: BasicCoverageBlock) -> Option<&CoverageKind> {
        if let Some(edge_from_bcbs) = &self.edge_from_bcbs {
            edge_from_bcbs.get(&from_bcb)
        } else {
            None
        }
    }

    #[inline]
    pub fn take_edge_counters(
        &mut self,
    ) -> Option<impl Iterator<Item = (BasicCoverageBlock, CoverageKind)>> {
        self.edge_from_bcbs.take().map(|m| m.into_iter())
    }

    pub fn id(&self) -> String {
        format!("@{}", self.basic_blocks.iter().map(|bb| bb.index().to_string()).join(ID_SEPARATOR))
    }
}

/// Represents a successor from a branching BasicCoverageBlock (such as the arms of a `SwitchInt`)
/// as either the successor BCB itself, if it has only one incoming edge, or the successor _plus_
/// the specific branching BCB, representing the edge between the two. The latter case
/// distinguishes this incoming edge from other incoming edges to the same `target_bcb`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct BcbBranch {
    pub edge_from_bcb: Option<BasicCoverageBlock>,
    pub target_bcb: BasicCoverageBlock,
}

impl BcbBranch {
    pub fn from_to(
        from_bcb: BasicCoverageBlock,
        to_bcb: BasicCoverageBlock,
        basic_coverage_blocks: &CoverageGraph,
    ) -> Self {
        let edge_from_bcb = if basic_coverage_blocks.predecessors[to_bcb].len() > 1 {
            Some(from_bcb)
        } else {
            None
        };
        Self { edge_from_bcb, target_bcb: to_bcb }
    }

    pub fn counter<'a>(
        &self,
        basic_coverage_blocks: &'a CoverageGraph,
    ) -> Option<&'a CoverageKind> {
        if let Some(from_bcb) = self.edge_from_bcb {
            basic_coverage_blocks[self.target_bcb].edge_counter_from(from_bcb)
        } else {
            basic_coverage_blocks[self.target_bcb].counter()
        }
    }

    pub fn is_only_path_to_target(&self) -> bool {
        self.edge_from_bcb.is_none()
    }
}

impl std::fmt::Debug for BcbBranch {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(from_bcb) = self.edge_from_bcb {
            write!(fmt, "{:?}->{:?}", from_bcb, self.target_bcb)
        } else {
            write!(fmt, "{:?}", self.target_bcb)
        }
    }
}

// Returns the `Terminator`s non-unwind successors.
// FIXME(#78544): MIR InstrumentCoverage: Improve coverage of `#[should_panic]` tests and
// `catch_unwind()` handlers.
fn bcb_filtered_successors<'a, 'tcx>(
    body: &'a mir::Body<'tcx>,
    term_kind: &'a TerminatorKind<'tcx>,
) -> Box<dyn Iterator<Item = BasicBlock> + 'a> {
    Box::new(
        match &term_kind {
            // SwitchInt successors are never unwind, and all of them should be traversed.
            TerminatorKind::SwitchInt { ref targets, .. } => {
                None.into_iter().chain(targets.all_targets().into_iter().copied())
            }
            // For all other kinds, return only the first successor, if any, and ignore unwinds.
            // NOTE: `chain(&[])` is required to coerce the `option::iter` (from
            // `next().into_iter()`) into the `mir::Successors` aliased type.
            _ => term_kind.successors().next().into_iter().chain((&[]).into_iter().copied()),
        }
        .filter(move |&successor| body[successor].terminator().kind != TerminatorKind::Unreachable),
    )
}

/// Maintains separate worklists for each loop in the BasicCoverageBlock CFG, plus one for the
/// CoverageGraph outside all loops. This supports traversing the BCB CFG in a way that
/// ensures a loop is completely traversed before processing Blocks after the end of the loop.
#[derive(Debug)]
pub(super) struct TraversalContext {
    /// From one or more backedges returning to a loop header.
    pub loop_backedges: Option<(Vec<BasicCoverageBlock>, BasicCoverageBlock)>,

    /// worklist, to be traversed, of CoverageGraph in the loop with the given loop
    /// backedges, such that the loop is the inner inner-most loop containing these
    /// CoverageGraph
    pub worklist: Vec<BasicCoverageBlock>,
}

pub(super) struct TraverseCoverageGraphWithLoops {
    pub backedges: IndexVec<BasicCoverageBlock, Vec<BasicCoverageBlock>>,
    pub context_stack: Vec<TraversalContext>,
    visited: BitSet<BasicCoverageBlock>,
}

impl TraverseCoverageGraphWithLoops {
    pub fn new(basic_coverage_blocks: &CoverageGraph) -> Self {
        let start_bcb = basic_coverage_blocks.start_node();
        let backedges = find_loop_backedges(basic_coverage_blocks);
        let context_stack =
            vec![TraversalContext { loop_backedges: None, worklist: vec![start_bcb] }];
        // `context_stack` starts with a `TraversalContext` for the main function context (beginning
        // with the `start` BasicCoverageBlock of the function). New worklists are pushed to the top
        // of the stack as loops are entered, and popped off of the stack when a loop's worklist is
        // exhausted.
        let visited = BitSet::new_empty(basic_coverage_blocks.num_nodes());
        Self { backedges, context_stack, visited }
    }

    pub fn next(&mut self, basic_coverage_blocks: &CoverageGraph) -> Option<BasicCoverageBlock> {
        debug!(
            "TraverseCoverageGraphWithLoops::next - context_stack: {:?}",
            self.context_stack.iter().rev().collect::<Vec<_>>()
        );

        while let Some(context) = self.context_stack.last_mut() {
            if let Some(next_bcb) = context.worklist.pop() {
                if !self.visited.insert(next_bcb) {
                    debug!("Already visited: {:?}", next_bcb);
                    continue;
                }
                debug!("Visiting {:?}", next_bcb);
                if self.backedges[next_bcb].len() > 0 {
                    debug!("{:?} is a loop header! Start a new TraversalContext...", next_bcb);
                    self.context_stack.push(TraversalContext {
                        loop_backedges: Some((self.backedges[next_bcb].clone(), next_bcb)),
                        worklist: Vec::new(),
                    });
                }
                self.extend_worklist(basic_coverage_blocks, next_bcb);
                return Some(next_bcb);
            } else {
                // Strip contexts with empty worklists from the top of the stack
                self.context_stack.pop();
            }
        }

        None
    }

    pub fn extend_worklist(
        &mut self,
        basic_coverage_blocks: &CoverageGraph,
        bcb: BasicCoverageBlock,
    ) {
        let successors = &basic_coverage_blocks.successors[bcb];
        debug!("{:?} has {} successors:", bcb, successors.len());
        for &successor in successors {
            if successor == bcb {
                debug!(
                    "{:?} has itself as its own successor. (Note, the compiled code will \
                    generate an infinite loop.)",
                    bcb
                );
                // Don't re-add this successor to the worklist. We are already processing it.
                break;
            }
            for context in self.context_stack.iter_mut().rev() {
                // Add successors of the current BCB to the appropriate context. Successors that
                // stay within a loop are added to the BCBs context worklist. Successors that
                // exit the loop (they are not dominated by the loop header) must be reachable
                // from other BCBs outside the loop, and they will be added to a different
                // worklist.
                //
                // Branching blocks (with more than one successor) must be processed before
                // blocks with only one successor, to prevent unnecessarily complicating
                // `Expression`s by creating a Counter in a `BasicCoverageBlock` that the
                // branching block would have given an `Expression` (or vice versa).
                let (some_successor_to_add, some_loop_header) =
                    if let Some((_, loop_header)) = context.loop_backedges {
                        if basic_coverage_blocks.dominates(loop_header, successor) {
                            (Some(successor), Some(loop_header))
                        } else {
                            (None, None)
                        }
                    } else {
                        (Some(successor), None)
                    };
                if let Some(successor_to_add) = some_successor_to_add {
                    if basic_coverage_blocks.successors[successor_to_add].len() > 1 {
                        debug!(
                            "{:?} successor is branching. Prioritize it at the beginning of \
                            the {}",
                            successor_to_add,
                            if let Some(loop_header) = some_loop_header {
                                format!("worklist for the loop headed by {loop_header:?}")
                            } else {
                                String::from("non-loop worklist")
                            },
                        );
                        context.worklist.insert(0, successor_to_add);
                    } else {
                        debug!(
                            "{:?} successor is non-branching. Defer it to the end of the {}",
                            successor_to_add,
                            if let Some(loop_header) = some_loop_header {
                                format!("worklist for the loop headed by {loop_header:?}")
                            } else {
                                String::from("non-loop worklist")
                            },
                        );
                        context.worklist.push(successor_to_add);
                    }
                    break;
                }
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.visited.count() == self.visited.domain_size()
    }

    pub fn unvisited(&self) -> Vec<BasicCoverageBlock> {
        let mut unvisited_set: BitSet<BasicCoverageBlock> =
            BitSet::new_filled(self.visited.domain_size());
        unvisited_set.subtract(&self.visited);
        unvisited_set.iter().collect::<Vec<_>>()
    }
}

pub(super) fn find_loop_backedges(
    basic_coverage_blocks: &CoverageGraph,
) -> IndexVec<BasicCoverageBlock, Vec<BasicCoverageBlock>> {
    let num_bcbs = basic_coverage_blocks.num_nodes();
    let mut backedges = IndexVec::from_elem_n(Vec::<BasicCoverageBlock>::new(), num_bcbs);

    // Identify loops by their backedges.
    for (bcb, _) in basic_coverage_blocks.iter_enumerated() {
        for &successor in &basic_coverage_blocks.successors[bcb] {
            if basic_coverage_blocks.dominates(successor, bcb) {
                let loop_header = successor;
                let backedge_from_bcb = bcb;
                debug!(
                    "Found BCB backedge: {:?} -> loop_header: {:?}",
                    backedge_from_bcb, loop_header
                );
                backedges[loop_header].push(backedge_from_bcb);
            }
        }
    }
    backedges
}

pub struct ShortCircuitPreorder<
    'a,
    'tcx,
    F: Fn(&'a mir::Body<'tcx>, &'a TerminatorKind<'tcx>) -> Box<dyn Iterator<Item = BasicBlock> + 'a>,
> {
    body: &'a mir::Body<'tcx>,
    visited: BitSet<BasicBlock>,
    worklist: Vec<BasicBlock>,
    filtered_successors: F,
}

impl<
    'a,
    'tcx,
    F: Fn(&'a mir::Body<'tcx>, &'a TerminatorKind<'tcx>) -> Box<dyn Iterator<Item = BasicBlock> + 'a>,
> ShortCircuitPreorder<'a, 'tcx, F>
{
    pub fn new(
        body: &'a mir::Body<'tcx>,
        filtered_successors: F,
    ) -> ShortCircuitPreorder<'a, 'tcx, F> {
        let worklist = vec![mir::START_BLOCK];

        ShortCircuitPreorder {
            body,
            visited: BitSet::new_empty(body.basic_blocks.len()),
            worklist,
            filtered_successors,
        }
    }
}

impl<
    'a,
    'tcx,
    F: Fn(&'a mir::Body<'tcx>, &'a TerminatorKind<'tcx>) -> Box<dyn Iterator<Item = BasicBlock> + 'a>,
> Iterator for ShortCircuitPreorder<'a, 'tcx, F>
{
    type Item = (BasicBlock, &'a BasicBlockData<'tcx>);

    fn next(&mut self) -> Option<(BasicBlock, &'a BasicBlockData<'tcx>)> {
        while let Some(idx) = self.worklist.pop() {
            if !self.visited.insert(idx) {
                continue;
            }

            let data = &self.body[idx];

            if let Some(ref term) = data.terminator {
                self.worklist.extend((self.filtered_successors)(&self.body, &term.kind));
            }

            return Some((idx, data));
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.body.basic_blocks.len() - self.visited.count();
        (size, Some(size))
    }
}
