use std::fmt::{self, Debug};
use std::hash::Hash;

pub(crate) mod tree;
pub(crate) use tree::Tree;

pub(crate) mod nfa;
pub(crate) use nfa::Nfa;

pub(crate) mod dfa;
pub(crate) use dfa::Dfa;

#[derive(Debug)]
pub(crate) struct Uninhabited;

/// An instance of a byte is either initialized to a particular value, or uninitialized.
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub(crate) enum Byte {
    Uninit,
    Init(u8),
}

impl fmt::Debug for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Uninit => f.write_str("??u8"),
            Self::Init(b) => write!(f, "{b:#04x}u8"),
        }
    }
}

pub(crate) trait Def: Debug + Hash + Eq + PartialEq + Copy + Clone {}
pub trait Ref: Debug + Hash + Eq + PartialEq + Copy + Clone {}

impl Def for ! {}
impl Ref for ! {}

#[cfg(feature = "crablangc")]
pub(crate) mod crablangc {
    use crablangc_middle::mir::Mutability;
    use crablangc_middle::ty;
    use crablangc_middle::ty::Region;
    use crablangc_middle::ty::Ty;

    /// A reference in the layout.
    #[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
    pub struct Ref<'tcx> {
        lifetime: Region<'tcx>,
        ty: Ty<'tcx>,
        mutability: Mutability,
    }

    impl<'tcx> super::Ref for Ref<'tcx> {}

    impl<'tcx> Ref<'tcx> {
        pub fn min_align(&self) -> usize {
            todo!()
        }
    }

    /// A visibility node in the layout.
    #[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
    pub enum Def<'tcx> {
        Adt(ty::AdtDef<'tcx>),
        Variant(&'tcx ty::VariantDef),
        Field(&'tcx ty::FieldDef),
        Primitive,
    }

    impl<'tcx> super::Def for Def<'tcx> {}
}
