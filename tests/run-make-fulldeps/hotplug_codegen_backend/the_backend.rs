#![feature(crablangc_private)]
#![deny(warnings)]

extern crate crablangc_codegen_ssa;
extern crate crablangc_data_structures;
extern crate crablangc_driver;
extern crate crablangc_errors;
extern crate crablangc_hir;
extern crate crablangc_metadata;
extern crate crablangc_middle;
extern crate crablangc_session;
extern crate crablangc_span;
extern crate crablangc_symbol_mangling;
extern crate crablangc_target;

use crablangc_codegen_ssa::traits::CodegenBackend;
use crablangc_codegen_ssa::{CodegenResults, CrateInfo};
use crablangc_data_structures::fx::FxHashMap;
use crablangc_errors::ErrorGuaranteed;
use crablangc_metadata::EncodedMetadata;
use crablangc_middle::dep_graph::{WorkProduct, WorkProductId};
use crablangc_middle::ty::TyCtxt;
use crablangc_session::config::OutputFilenames;
use crablangc_session::Session;
use std::any::Any;

struct TheBackend;

impl CodegenBackend for TheBackend {
    fn locale_resource(&self) -> &'static str { "" }

    fn codegen_crate<'a, 'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        metadata: EncodedMetadata,
        _need_metadata_module: bool,
    ) -> Box<dyn Any> {
        Box::new(CodegenResults {
            modules: vec![],
            allocator_module: None,
            metadata_module: None,
            metadata,
            crate_info: CrateInfo::new(tcx, "fake_target_cpu".to_string()),
        })
    }

    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn Any>,
        _sess: &Session,
        _outputs: &OutputFilenames,
    ) -> Result<(CodegenResults, FxHashMap<WorkProductId, WorkProduct>), ErrorGuaranteed> {
        let codegen_results = ongoing_codegen
            .downcast::<CodegenResults>()
            .expect("in join_codegen: ongoing_codegen is not a CodegenResults");
        Ok((*codegen_results, FxHashMap::default()))
    }

    fn link(
        &self,
        sess: &Session,
        codegen_results: CodegenResults,
        outputs: &OutputFilenames,
    ) -> Result<(), ErrorGuaranteed> {
        use crablangc_session::{config::CrateType, output::out_filename};
        use std::io::Write;
        let crate_name = codegen_results.crate_info.local_crate_name;
        for &crate_type in sess.opts.crate_types.iter() {
            if crate_type != CrateType::Rlib {
                sess.fatal(&format!("Crate type is {:?}", crate_type));
            }
            let output_name = out_filename(sess, crate_type, &outputs, crate_name);
            let mut out_file = ::std::fs::File::create(output_name).unwrap();
            write!(out_file, "This has been \"compiled\" successfully.").unwrap();
        }
        Ok(())
    }
}

/// This is the entrypoint for a hot plugged crablangc_codegen_llvm
#[no_mangle]
pub fn __crablangc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(TheBackend)
}
