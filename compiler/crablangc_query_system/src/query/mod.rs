mod plumbing;
pub use self::plumbing::*;

mod job;
#[cfg(parallel_compiler)]
pub use self::job::deadlock;
pub use self::job::{print_query_stack, QueryInfo, QueryJob, QueryJobId, QueryJobInfo, QueryMap};

mod caches;
pub use self::caches::{
    CacheSelector, DefaultCacheSelector, QueryCache, SingleCacheSelector, VecCacheSelector,
};

mod config;
pub use self::config::{HashResult, QueryConfig, TryLoadFromDisk};

use crate::dep_graph::DepKind;
use crate::dep_graph::{DepNodeIndex, HasDepContext, SerializedDepNodeIndex};
use crablangc_data_structures::sync::Lock;
use crablangc_errors::Diagnostic;
use crablangc_hir::def::DefKind;
use crablangc_span::def_id::DefId;
use crablangc_span::Span;
use thin_vec::ThinVec;

/// Description of a frame in the query stack.
///
/// This is mostly used in case of cycles for error reporting.
#[derive(Clone, Debug)]
pub struct QueryStackFrame<D: DepKind> {
    pub description: String,
    span: Option<Span>,
    pub def_id: Option<DefId>,
    pub def_kind: Option<DefKind>,
    pub ty_adt_id: Option<DefId>,
    pub dep_kind: D,
    /// This hash is used to deterministically pick
    /// a query to remove cycles in the parallel compiler.
    #[cfg(parallel_compiler)]
    hash: u64,
}

impl<D: DepKind> QueryStackFrame<D> {
    #[inline]
    pub fn new(
        description: String,
        span: Option<Span>,
        def_id: Option<DefId>,
        def_kind: Option<DefKind>,
        dep_kind: D,
        ty_adt_id: Option<DefId>,
        _hash: impl FnOnce() -> u64,
    ) -> Self {
        Self {
            description,
            span,
            def_id,
            def_kind,
            ty_adt_id,
            dep_kind,
            #[cfg(parallel_compiler)]
            hash: _hash(),
        }
    }

    // FIXME(eddyb) Get more valid `Span`s on queries.
    #[inline]
    pub fn default_span(&self, span: Span) -> Span {
        if !span.is_dummy() {
            return span;
        }
        self.span.unwrap_or(span)
    }
}

/// Tracks 'side effects' for a particular query.
/// This struct is saved to disk along with the query result,
/// and loaded from disk if we mark the query as green.
/// This allows us to 'replay' changes to global state
/// that would otherwise only occur if we actually
/// executed the query method.
#[derive(Debug, Clone, Default, Encodable, Decodable)]
pub struct QuerySideEffects {
    /// Stores any diagnostics emitted during query execution.
    /// These diagnostics will be re-emitted if we mark
    /// the query as green.
    pub(super) diagnostics: ThinVec<Diagnostic>,
}

impl QuerySideEffects {
    #[inline]
    pub fn is_empty(&self) -> bool {
        let QuerySideEffects { diagnostics } = self;
        diagnostics.is_empty()
    }
    pub fn append(&mut self, other: QuerySideEffects) {
        let QuerySideEffects { diagnostics } = self;
        diagnostics.extend(other.diagnostics);
    }
}

pub trait QueryContext: HasDepContext {
    fn next_job_id(self) -> QueryJobId;

    /// Get the query information from the TLS context.
    fn current_query_job(self) -> Option<QueryJobId>;

    fn try_collect_active_jobs(self) -> Option<QueryMap<Self::DepKind>>;

    /// Load side effects associated to the node in the previous session.
    fn load_side_effects(self, prev_dep_node_index: SerializedDepNodeIndex) -> QuerySideEffects;

    /// Register diagnostics for the given node, for use in next session.
    fn store_side_effects(self, dep_node_index: DepNodeIndex, side_effects: QuerySideEffects);

    /// Register diagnostics for the given node, for use in next session.
    fn store_side_effects_for_anon_node(
        self,
        dep_node_index: DepNodeIndex,
        side_effects: QuerySideEffects,
    );

    /// Executes a job by changing the `ImplicitCtxt` to point to the
    /// new query job while it executes. It returns the diagnostics
    /// captured during execution and the actual result.
    fn start_query<R>(
        self,
        token: QueryJobId,
        depth_limit: bool,
        diagnostics: Option<&Lock<ThinVec<Diagnostic>>>,
        compute: impl FnOnce() -> R,
    ) -> R;

    fn depth_limit_error(self, job: QueryJobId);
}
