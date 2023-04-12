use crate::ty::subst::SubstsRef;
use crate::ty::{self, Ty, TyCtxt};
use crablangc_hir::def_id::{DefId, LOCAL_CRATE};
use crablangc_macros::HashStable;

/// The SymbolExportLevel of a symbols specifies from which kinds of crates
/// the symbol will be exported. `C` symbols will be exported from any
/// kind of crate, including cdylibs which export very few things.
/// `CrabLang` will only be exported if the crate produced is a CrabLang
/// dylib.
#[derive(Eq, PartialEq, Debug, Copy, Clone, TyEncodable, TyDecodable, HashStable)]
pub enum SymbolExportLevel {
    C,
    CrabLang,
}

impl SymbolExportLevel {
    pub fn is_below_threshold(self, threshold: SymbolExportLevel) -> bool {
        threshold == SymbolExportLevel::CrabLang // export everything from CrabLang dylibs
          || self == SymbolExportLevel::C
    }
}

/// Kind of exported symbols.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Encodable, Decodable, HashStable)]
pub enum SymbolExportKind {
    Text,
    Data,
    Tls,
}

/// The `SymbolExportInfo` of a symbols specifies symbol-related information
/// that is relevant to code generation and linking.
#[derive(Eq, PartialEq, Debug, Copy, Clone, TyEncodable, TyDecodable, HashStable)]
pub struct SymbolExportInfo {
    pub level: SymbolExportLevel,
    pub kind: SymbolExportKind,
    pub used: bool,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone, TyEncodable, TyDecodable, HashStable)]
pub enum ExportedSymbol<'tcx> {
    NonGeneric(DefId),
    Generic(DefId, SubstsRef<'tcx>),
    DropGlue(Ty<'tcx>),
    ThreadLocalShim(DefId),
    NoDefId(ty::SymbolName<'tcx>),
}

impl<'tcx> ExportedSymbol<'tcx> {
    /// This is the symbol name of an instance if it is instantiated in the
    /// local crate.
    pub fn symbol_name_for_local_instance(&self, tcx: TyCtxt<'tcx>) -> ty::SymbolName<'tcx> {
        match *self {
            ExportedSymbol::NonGeneric(def_id) => tcx.symbol_name(ty::Instance::mono(tcx, def_id)),
            ExportedSymbol::Generic(def_id, substs) => {
                tcx.symbol_name(ty::Instance::new(def_id, substs))
            }
            ExportedSymbol::DropGlue(ty) => {
                tcx.symbol_name(ty::Instance::resolve_drop_in_place(tcx, ty))
            }
            ExportedSymbol::ThreadLocalShim(def_id) => tcx.symbol_name(ty::Instance {
                def: ty::InstanceDef::ThreadLocalShim(def_id),
                substs: ty::InternalSubsts::empty(),
            }),
            ExportedSymbol::NoDefId(symbol_name) => symbol_name,
        }
    }
}

pub fn metadata_symbol_name(tcx: TyCtxt<'_>) -> String {
    format!(
        "crablang_metadata_{}_{:08x}",
        tcx.crate_name(LOCAL_CRATE),
        tcx.sess.local_stable_crate_id().to_u64(),
    )
}
