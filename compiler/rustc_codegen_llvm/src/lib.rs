//! The Rust compiler.
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

#![doc(html_root_url = "https://doc.rust-lang.org/nightly/nightly-rustc/")]
#![feature(extern_types)]
#![feature(hash_raw_entry)]
#![feature(iter_intersperse)]
#![feature(let_chains)]
#![feature(never_type)]
#![feature(impl_trait_in_assoc_type)]
#![recursion_limit = "256"]
#![allow(rustc::potential_query_instability)]
#![deny(rustc::untranslatable_diagnostic)]
#![deny(rustc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate rustc_macros;
#[macro_use]
extern crate tracing;

use back::write::{create_informational_target_machine, create_target_machine};

use errors::ParseTargetMachineConfig;
pub use llvm_util::target_features;
use rustc_ast::expand::allocator::AllocatorKind;
use rustc_codegen_ssa::back::lto::{LtoModuleCodegen, SerializedModule, ThinModule};
use rustc_codegen_ssa::back::write::{
    CodegenContext, FatLTOInput, ModuleConfig, TargetMachineFactoryConfig, TargetMachineFactoryFn,
};
use rustc_codegen_ssa::traits::*;
use rustc_codegen_ssa::ModuleCodegen;
use rustc_codegen_ssa::{CodegenResults, CompiledModule};
use rustc_data_structures::fx::FxIndexMap;
use rustc_errors::{DiagnosticMessage, ErrorGuaranteed, FatalError, Handler, SubdiagnosticMessage};
use rustc_fluent_macro::fluent_messages;
use rustc_metadata::EncodedMetadata;
use rustc_middle::dep_graph::{WorkProduct, WorkProductId};
use rustc_middle::query::Providers;
use rustc_middle::ty::TyCtxt;
use rustc_session::config::{OptLevel, OutputFilenames, PrintRequest};
use rustc_session::Session;
use rustc_span::symbol::Symbol;

use std::any::Any;
use std::ffi::CStr;

mod back {
    pub mod archive;
    pub mod lto;
    mod profiling;
    pub mod write;
}

mod abi;
mod allocator;
mod asm;
mod attributes;
mod base;
mod builder;
mod callee;
mod common;
mod consts;
mod context;
mod coverageinfo;
mod debuginfo;
mod declare;
mod errors;
mod intrinsic;

// The following is a workaround that replaces `pub mod llvm;` and that fixes issue 53912.
#[path = "llvm/mod.rs"]
mod llvm_;
pub mod llvm {
    pub use super::llvm_::*;
}

mod llvm_util;
mod mono_item;
mod type_;
mod type_of;
mod va_arg;
mod value;

fluent_messages! { "../messages.ftl" }

#[derive(Clone)]
pub struct LlvmCodegenBackend(());

struct TimeTraceProfiler {
    enabled: bool,
}

impl TimeTraceProfiler {
    fn new(enabled: bool) -> Self {
        if enabled {
            unsafe { llvm::LLVMTimeTraceProfilerInitialize() }
        }
        TimeTraceProfiler { enabled }
    }
}

impl Drop for TimeTraceProfiler {
    fn drop(&mut self) {
        if self.enabled {
            unsafe { llvm::LLVMTimeTraceProfilerFinishThread() }
        }
    }
}

impl ExtraBackendMethods for LlvmCodegenBackend {
    fn codegen_allocator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        module_name: &str,
        kind: AllocatorKind,
        alloc_error_handler_kind: AllocatorKind,
    ) -> ModuleLlvm {
        let mut module_llvm = ModuleLlvm::new_metadata(tcx, module_name);
        unsafe {
            allocator::codegen(tcx, &mut module_llvm, module_name, kind, alloc_error_handler_kind);
        }
        module_llvm
    }
    fn compile_codegen_unit(
        &self,
        tcx: TyCtxt<'_>,
        cgu_name: Symbol,
    ) -> (ModuleCodegen<ModuleLlvm>, u64) {
        base::compile_codegen_unit(tcx, cgu_name)
    }
    fn target_machine_factory(
        &self,
        sess: &Session,
        optlvl: OptLevel,
        target_features: &[String],
    ) -> TargetMachineFactoryFn<Self> {
        back::write::target_machine_factory(sess, optlvl, target_features)
    }

    fn spawn_thread<F, T>(time_trace: bool, f: F) -> std::thread::JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        std::thread::spawn(move || {
            let _profiler = TimeTraceProfiler::new(time_trace);
            f()
        })
    }

    fn spawn_named_thread<F, T>(
        time_trace: bool,
        name: String,
        f: F,
    ) -> std::io::Result<std::thread::JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        std::thread::Builder::new().name(name).spawn(move || {
            let _profiler = TimeTraceProfiler::new(time_trace);
            f()
        })
    }
}

impl WriteBackendMethods for LlvmCodegenBackend {
    type Module = ModuleLlvm;
    type ModuleBuffer = back::lto::ModuleBuffer;
    type TargetMachine = &'static mut llvm::TargetMachine;
    type TargetMachineError = crate::errors::LlvmError<'static>;
    type ThinData = back::lto::ThinData;
    type ThinBuffer = back::lto::ThinBuffer;
    fn print_pass_timings(&self) {
        unsafe {
            llvm::LLVMRustPrintPassTimings();
        }
    }
    fn run_link(
        cgcx: &CodegenContext<Self>,
        diag_handler: &Handler,
        modules: Vec<ModuleCodegen<Self::Module>>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        back::write::link(cgcx, diag_handler, modules)
    }
    fn run_fat_lto(
        cgcx: &CodegenContext<Self>,
        modules: Vec<FatLTOInput<Self>>,
        cached_modules: Vec<(SerializedModule<Self::ModuleBuffer>, WorkProduct)>,
    ) -> Result<LtoModuleCodegen<Self>, FatalError> {
        back::lto::run_fat(cgcx, modules, cached_modules)
    }
    fn run_thin_lto(
        cgcx: &CodegenContext<Self>,
        modules: Vec<(String, Self::ThinBuffer)>,
        cached_modules: Vec<(SerializedModule<Self::ModuleBuffer>, WorkProduct)>,
    ) -> Result<(Vec<LtoModuleCodegen<Self>>, Vec<WorkProduct>), FatalError> {
        back::lto::run_thin(cgcx, modules, cached_modules)
    }
    unsafe fn optimize(
        cgcx: &CodegenContext<Self>,
        diag_handler: &Handler,
        module: &ModuleCodegen<Self::Module>,
        config: &ModuleConfig,
    ) -> Result<(), FatalError> {
        back::write::optimize(cgcx, diag_handler, module, config)
    }
    fn optimize_fat(
        cgcx: &CodegenContext<Self>,
        module: &mut ModuleCodegen<Self::Module>,
    ) -> Result<(), FatalError> {
        let diag_handler = cgcx.create_diag_handler();
        back::lto::run_pass_manager(cgcx, &diag_handler, module, false)
    }
    unsafe fn optimize_thin(
        cgcx: &CodegenContext<Self>,
        thin: ThinModule<Self>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        back::lto::optimize_thin_module(thin, cgcx)
    }
    unsafe fn codegen(
        cgcx: &CodegenContext<Self>,
        diag_handler: &Handler,
        module: ModuleCodegen<Self::Module>,
        config: &ModuleConfig,
    ) -> Result<CompiledModule, FatalError> {
        back::write::codegen(cgcx, diag_handler, module, config)
    }
    fn prepare_thin(module: ModuleCodegen<Self::Module>) -> (String, Self::ThinBuffer) {
        back::lto::prepare_thin(module)
    }
    fn serialize_module(module: ModuleCodegen<Self::Module>) -> (String, Self::ModuleBuffer) {
        (module.name, back::lto::ModuleBuffer::new(module.module_llvm.llmod()))
    }
}

unsafe impl Send for LlvmCodegenBackend {} // Llvm is on a per-thread basis
unsafe impl Sync for LlvmCodegenBackend {}

impl LlvmCodegenBackend {
    pub fn new() -> Box<dyn CodegenBackend> {
        Box::new(LlvmCodegenBackend(()))
    }
}

impl CodegenBackend for LlvmCodegenBackend {
    fn locale_resource(&self) -> &'static str {
        crate::DEFAULT_LOCALE_RESOURCE
    }

    fn init(&self, sess: &Session) {
        llvm_util::init(sess); // Make sure llvm is inited
    }

    fn provide(&self, providers: &mut Providers) {
        providers.global_backend_features =
            |tcx, ()| llvm_util::global_llvm_features(tcx.sess, true)
    }

    fn print(&self, req: PrintRequest, sess: &Session) {
        match req {
            PrintRequest::RelocationModels => {
                println!("Available relocation models:");
                for name in &[
                    "static",
                    "pic",
                    "pie",
                    "dynamic-no-pic",
                    "ropi",
                    "rwpi",
                    "ropi-rwpi",
                    "default",
                ] {
                    println!("    {}", name);
                }
                println!();
            }
            PrintRequest::CodeModels => {
                println!("Available code models:");
                for name in &["tiny", "small", "kernel", "medium", "large"] {
                    println!("    {}", name);
                }
                println!();
            }
            PrintRequest::TlsModels => {
                println!("Available TLS models:");
                for name in &["global-dynamic", "local-dynamic", "initial-exec", "local-exec"] {
                    println!("    {}", name);
                }
                println!();
            }
            PrintRequest::StackProtectorStrategies => {
                println!(
                    r#"Available stack protector strategies:
    all
        Generate stack canaries in all functions.

    strong
        Generate stack canaries in a function if it either:
        - has a local variable of `[T; N]` type, regardless of `T` and `N`
        - takes the address of a local variable.

          (Note that a local variable being borrowed is not equivalent to its
          address being taken: e.g. some borrows may be removed by optimization,
          while by-value argument passing may be implemented with reference to a
          local stack variable in the ABI.)

    basic
        Generate stack canaries in functions with local variables of `[T; N]`
        type, where `T` is byte-sized and `N` >= 8.

    none
        Do not generate stack canaries.
"#
                );
            }
            req => llvm_util::print(req, sess),
        }
    }

    fn print_passes(&self) {
        llvm_util::print_passes();
    }

    fn print_version(&self) {
        llvm_util::print_version();
    }

    fn target_features(&self, sess: &Session, allow_unstable: bool) -> Vec<Symbol> {
        target_features(sess, allow_unstable)
    }

    fn codegen_crate<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        metadata: EncodedMetadata,
        need_metadata_module: bool,
    ) -> Box<dyn Any> {
        Box::new(rustc_codegen_ssa::base::codegen_crate(
            LlvmCodegenBackend(()),
            tcx,
            crate::llvm_util::target_cpu(tcx.sess).to_string(),
            metadata,
            need_metadata_module,
        ))
    }

    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn Any>,
        sess: &Session,
        outputs: &OutputFilenames,
    ) -> Result<(CodegenResults, FxIndexMap<WorkProductId, WorkProduct>), ErrorGuaranteed> {
        let (codegen_results, work_products) = ongoing_codegen
            .downcast::<rustc_codegen_ssa::back::write::OngoingCodegen<LlvmCodegenBackend>>()
            .expect("Expected LlvmCodegenBackend's OngoingCodegen, found Box<Any>")
            .join(sess);

        if sess.opts.unstable_opts.llvm_time_trace {
            sess.time("llvm_dump_timing_file", || {
                let file_name = outputs.with_extension("llvm_timings.json");
                llvm_util::time_trace_profiler_finish(&file_name);
            });
        }

        Ok((codegen_results, work_products))
    }

    fn link(
        &self,
        sess: &Session,
        codegen_results: CodegenResults,
        outputs: &OutputFilenames,
    ) -> Result<(), ErrorGuaranteed> {
        use crate::back::archive::LlvmArchiveBuilderBuilder;
        use rustc_codegen_ssa::back::link::link_binary;

        // Run the linker on any artifacts that resulted from the LLVM run.
        // This should produce either a finished executable or library.
        link_binary(sess, &LlvmArchiveBuilderBuilder, &codegen_results, outputs)
    }
}

pub struct ModuleLlvm {
    llcx: &'static mut llvm::Context,
    llmod_raw: *const llvm::Module,
    tm: &'static mut llvm::TargetMachine,
}

unsafe impl Send for ModuleLlvm {}
unsafe impl Sync for ModuleLlvm {}

impl ModuleLlvm {
    fn new(tcx: TyCtxt<'_>, mod_name: &str) -> Self {
        unsafe {
            let llcx = llvm::LLVMRustContextCreate(tcx.sess.fewer_names());
            let llmod_raw = context::create_module(tcx, llcx, mod_name) as *const _;
            ModuleLlvm { llmod_raw, llcx, tm: create_target_machine(tcx, mod_name) }
        }
    }

    fn new_metadata(tcx: TyCtxt<'_>, mod_name: &str) -> Self {
        unsafe {
            let llcx = llvm::LLVMRustContextCreate(tcx.sess.fewer_names());
            let llmod_raw = context::create_module(tcx, llcx, mod_name) as *const _;
            ModuleLlvm { llmod_raw, llcx, tm: create_informational_target_machine(tcx.sess) }
        }
    }

    fn parse(
        cgcx: &CodegenContext<LlvmCodegenBackend>,
        name: &CStr,
        buffer: &[u8],
        handler: &Handler,
    ) -> Result<Self, FatalError> {
        unsafe {
            let llcx = llvm::LLVMRustContextCreate(cgcx.fewer_names);
            let llmod_raw = back::lto::parse_module(llcx, name, buffer, handler)?;
            let tm_factory_config = TargetMachineFactoryConfig::new(cgcx, name.to_str().unwrap());
            let tm = match (cgcx.tm_factory)(tm_factory_config) {
                Ok(m) => m,
                Err(e) => {
                    return Err(handler.emit_almost_fatal(ParseTargetMachineConfig(e)));
                }
            };

            Ok(ModuleLlvm { llmod_raw, llcx, tm })
        }
    }

    fn llmod(&self) -> &llvm::Module {
        unsafe { &*self.llmod_raw }
    }
}

impl Drop for ModuleLlvm {
    fn drop(&mut self) {
        unsafe {
            llvm::LLVMRustDisposeTargetMachine(&mut *(self.tm as *mut _));
            llvm::LLVMContextDispose(&mut *(self.llcx as *mut _));
        }
    }
}
