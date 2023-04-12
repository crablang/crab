//! This pass just dumps MIR at a specified point.

use std::fs::File;
use std::io;

use crate::MirPass;
use crablangc_middle::mir::write_mir_pretty;
use crablangc_middle::mir::Body;
use crablangc_middle::ty::TyCtxt;
use crablangc_session::config::OutputType;

pub struct Marker(pub &'static str);

impl<'tcx> MirPass<'tcx> for Marker {
    fn name(&self) -> &str {
        self.0
    }

    fn run_pass(&self, _tcx: TyCtxt<'tcx>, _body: &mut Body<'tcx>) {}
}

pub fn emit_mir(tcx: TyCtxt<'_>) -> io::Result<()> {
    let path = tcx.output_filenames(()).path(OutputType::Mir);
    let mut f = io::BufWriter::new(File::create(&path)?);
    write_mir_pretty(tcx, None, &mut f)?;
    Ok(())
}
