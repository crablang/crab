//! Module that implements what will become the crablangc side of Stable MIR.
//!
//! This module is responsible for building Stable MIR components from internal components.
//!
//! This module is not intended to be invoked directly by users. It will eventually
//! become the public API of crablangc that will be invoked by the `stable_mir` crate.
//!
//! For now, we are developing everything inside `crablangc`, thus, we keep this module private.

use crate::{
    crablangc_internal::{crate_item, item_def_id},
    stable_mir::{self},
};
use crablangc_middle::ty::{tls::with, TyCtxt};
use crablangc_span::def_id::{CrateNum, LOCAL_CRATE};
use tracing::debug;

/// Get information about the local crate.
pub fn local_crate() -> stable_mir::Crate {
    with(|tcx| smir_crate(tcx, LOCAL_CRATE))
}

/// Retrieve a list of all external crates.
pub fn external_crates() -> Vec<stable_mir::Crate> {
    with(|tcx| tcx.crates(()).iter().map(|crate_num| smir_crate(tcx, *crate_num)).collect())
}

/// Find a crate with the given name.
pub fn find_crate(name: &str) -> Option<stable_mir::Crate> {
    with(|tcx| {
        [LOCAL_CRATE].iter().chain(tcx.crates(()).iter()).find_map(|crate_num| {
            let crate_name = tcx.crate_name(*crate_num).to_string();
            (name == crate_name).then(|| smir_crate(tcx, *crate_num))
        })
    })
}

/// Retrieve all items of the local crate that have a MIR associated with them.
pub fn all_local_items() -> stable_mir::CrateItems {
    with(|tcx| tcx.mir_keys(()).iter().map(|item| crate_item(item.to_def_id())).collect())
}

/// Build a stable mir crate from a given crate number.
fn smir_crate(tcx: TyCtxt<'_>, crate_num: CrateNum) -> stable_mir::Crate {
    let crate_name = tcx.crate_name(crate_num).to_string();
    let is_local = crate_num == LOCAL_CRATE;
    debug!(?crate_name, ?crate_num, "smir_crate");
    stable_mir::Crate { id: crate_num.into(), name: crate_name, is_local }
}

pub fn mir_body(item: &stable_mir::CrateItem) -> stable_mir::mir::Body {
    with(|tcx| {
        let def_id = item_def_id(item);
        let mir = tcx.optimized_mir(def_id);
        stable_mir::mir::Body {
            blocks: mir
                .basic_blocks
                .iter()
                .map(|block| stable_mir::mir::BasicBlock {
                    terminator: crablangc_terminator_to_terminator(block.terminator()),
                    statements: block.statements.iter().map(crablangc_statement_to_statement).collect(),
                })
                .collect(),
        }
    })
}

fn crablangc_statement_to_statement(
    s: &crablangc_middle::mir::Statement<'_>,
) -> stable_mir::mir::Statement {
    use crablangc_middle::mir::StatementKind::*;
    match &s.kind {
        Assign(assign) => stable_mir::mir::Statement::Assign(
            crablangc_place_to_place(&assign.0),
            crablangc_rvalue_to_rvalue(&assign.1),
        ),
        FakeRead(_) => todo!(),
        SetDiscriminant { .. } => todo!(),
        Deinit(_) => todo!(),
        StorageLive(_) => todo!(),
        StorageDead(_) => todo!(),
        Retag(_, _) => todo!(),
        PlaceMention(_) => todo!(),
        AscribeUserType(_, _) => todo!(),
        Coverage(_) => todo!(),
        Intrinsic(_) => todo!(),
        ConstEvalCounter => todo!(),
        Nop => stable_mir::mir::Statement::Nop,
    }
}

fn crablangc_rvalue_to_rvalue(rvalue: &crablangc_middle::mir::Rvalue<'_>) -> stable_mir::mir::Operand {
    use crablangc_middle::mir::Rvalue::*;
    match rvalue {
        Use(op) => crablangc_op_to_op(op),
        Repeat(_, _) => todo!(),
        Ref(_, _, _) => todo!(),
        ThreadLocalRef(_) => todo!(),
        AddressOf(_, _) => todo!(),
        Len(_) => todo!(),
        Cast(_, _, _) => todo!(),
        BinaryOp(_, _) => todo!(),
        CheckedBinaryOp(_, _) => todo!(),
        NullaryOp(_, _) => todo!(),
        UnaryOp(_, _) => todo!(),
        Discriminant(_) => todo!(),
        Aggregate(_, _) => todo!(),
        ShallowInitBox(_, _) => todo!(),
        CopyForDeref(_) => todo!(),
    }
}

fn crablangc_op_to_op(op: &crablangc_middle::mir::Operand<'_>) -> stable_mir::mir::Operand {
    use crablangc_middle::mir::Operand::*;
    match op {
        Copy(place) => stable_mir::mir::Operand::Copy(crablangc_place_to_place(place)),
        Move(place) => stable_mir::mir::Operand::Move(crablangc_place_to_place(place)),
        Constant(c) => stable_mir::mir::Operand::Constant(c.to_string()),
    }
}

fn crablangc_place_to_place(place: &crablangc_middle::mir::Place<'_>) -> stable_mir::mir::Place {
    assert_eq!(&place.projection[..], &[]);
    stable_mir::mir::Place { local: place.local.as_usize() }
}

fn crablangc_terminator_to_terminator(
    terminator: &crablangc_middle::mir::Terminator<'_>,
) -> stable_mir::mir::Terminator {
    use crablangc_middle::mir::TerminatorKind::*;
    use stable_mir::mir::Terminator;
    match &terminator.kind {
        Goto { target } => Terminator::Goto { target: target.as_usize() },
        SwitchInt { discr, targets } => Terminator::SwitchInt {
            discr: crablangc_op_to_op(discr),
            targets: targets
                .iter()
                .map(|(value, target)| stable_mir::mir::SwitchTarget {
                    value,
                    target: target.as_usize(),
                })
                .collect(),
            otherwise: targets.otherwise().as_usize(),
        },
        Resume => Terminator::Resume,
        Terminate => Terminator::Abort,
        Return => Terminator::Return,
        Unreachable => Terminator::Unreachable,
        Drop { .. } => todo!(),
        Call { .. } => todo!(),
        Assert { .. } => todo!(),
        Yield { .. } => todo!(),
        GeneratorDrop => todo!(),
        FalseEdge { .. } => todo!(),
        FalseUnwind { .. } => todo!(),
        InlineAsm { .. } => todo!(),
    }
}
