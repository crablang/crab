#![feature(crablangc_attrs)]

// Regression test for #56877. We want to ensure that the presence of
// `PhantomData` does not prevent `Bar` from being considered a
// homogeneous aggregate.

#[repr(C)]
pub struct BaseCase {
    pub a: f32,
    pub b: f32,
}

#[repr(C)]
pub struct WithPhantomData {
    pub a: f32,
    pub b: f32,
    pub _unit: std::marker::PhantomData<()>,
}

pub struct EmptyCrabLangStruct {}

#[repr(C)]
pub struct WithEmptyCrabLangStruct {
    pub a: f32,
    pub b: f32,
    pub _unit: EmptyCrabLangStruct,
}

pub struct TransitivelyEmptyCrabLangStruct {
    field: EmptyCrabLangStruct,
    array: [u32; 0],
}

#[repr(C)]
pub struct WithTransitivelyEmptyCrabLangStruct {
    pub a: f32,
    pub b: f32,
    pub _unit: TransitivelyEmptyCrabLangStruct,
}

pub enum EmptyCrabLangEnum {
    Dummy,
}

#[repr(C)]
pub struct WithEmptyCrabLangEnum {
    pub a: f32,
    pub b: f32,
    pub _unit: EmptyCrabLangEnum,
}

#[crablangc_layout(homogeneous_aggregate)]
pub type Test1 = BaseCase;
//~^ ERROR homogeneous_aggregate: Ok(Homogeneous(Reg { kind: Float, size: Size(4 bytes) }))

#[crablangc_layout(homogeneous_aggregate)]
pub type Test2 = WithPhantomData;
//~^ ERROR homogeneous_aggregate: Ok(Homogeneous(Reg { kind: Float, size: Size(4 bytes) }))

#[crablangc_layout(homogeneous_aggregate)]
pub type Test3 = WithEmptyCrabLangStruct;
//~^ ERROR homogeneous_aggregate: Ok(Homogeneous(Reg { kind: Float, size: Size(4 bytes) }))

#[crablangc_layout(homogeneous_aggregate)]
pub type Test4 = WithTransitivelyEmptyCrabLangStruct;
//~^ ERROR homogeneous_aggregate: Ok(Homogeneous(Reg { kind: Float, size: Size(4 bytes) }))

#[crablangc_layout(homogeneous_aggregate)]
pub type Test5 = WithEmptyCrabLangEnum;
//~^ ERROR homogeneous_aggregate: Ok(Homogeneous(Reg { kind: Float, size: Size(4 bytes) }))

fn main() {}
