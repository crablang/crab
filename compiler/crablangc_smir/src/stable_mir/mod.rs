//! Module that implements the public interface to the Stable MIR.
//!
//! This module shall contain all type definitions and APIs that we expect 3P tools to invoke to
//! interact with the compiler.
//!
//! The goal is to eventually move this module to its own crate which shall be published on
//! [crates.io](https://crates.io).
//!
//! ## Note:
//!
//! There shouldn't be any direct references to internal compiler constructs in this module.
//! If you need an internal construct, consider using `crablangc_internal` or `crablangc_smir`.

pub mod mir;

/// Use String for now but we should replace it.
pub type Symbol = String;

/// The number that identifies a crate.
pub type CrateNum = usize;

/// A unique identification number for each item accessible for the current compilation unit.
pub type DefId = usize;

/// A list of crate items.
pub type CrateItems = Vec<CrateItem>;

/// Holds information about a crate.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Crate {
    pub(crate) id: CrateNum,
    pub name: Symbol,
    pub is_local: bool,
}

/// Holds information about an item in the crate.
/// For now, it only stores the item DefId. Use functions inside `crablangc_internal` module to
/// use this item.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CrateItem(pub(crate) DefId);

impl CrateItem {
    pub fn body(&self) -> mir::Body {
        crate::crablangc_smir::mir_body(self)
    }
}

/// Access to the local crate.
pub fn local_crate() -> Crate {
    crate::crablangc_smir::local_crate()
}

/// Try to find a crate with the given name.
pub fn find_crate(name: &str) -> Option<Crate> {
    crate::crablangc_smir::find_crate(name)
}

/// Try to find a crate with the given name.
pub fn external_crates() -> Vec<Crate> {
    crate::crablangc_smir::external_crates()
}

/// Retrieve all items in the local crate that have a MIR associated with them.
pub fn all_local_items() -> CrateItems {
    crate::crablangc_smir::all_local_items()
}
