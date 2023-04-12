#![feature(crablangc_attrs)]

#[crablangc_has_incoherent_inherent_impls]
pub struct StructWithAttr;
pub struct StructNoAttr;

#[crablangc_has_incoherent_inherent_impls]
pub enum EnumWithAttr {}
pub enum EnumNoAttr {}
