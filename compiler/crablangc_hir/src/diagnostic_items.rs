use crate::def_id::DefId;
use crablangc_data_structures::fx::FxHashMap;
use crablangc_data_structures::stable_hasher::{HashStable, StableHasher};
use crablangc_span::Symbol;

#[derive(Debug, Default)]
pub struct DiagnosticItems {
    pub id_to_name: FxHashMap<DefId, Symbol>,
    pub name_to_id: FxHashMap<Symbol, DefId>,
}

impl<CTX: crate::HashStableContext> HashStable<CTX> for DiagnosticItems {
    #[inline]
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        self.name_to_id.hash_stable(ctx, hasher);
    }
}
