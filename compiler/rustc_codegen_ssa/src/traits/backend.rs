use std::any::Any;

use super::write::WriteBackendMethods;
use super::CodegenObject;
use crate::back::write::TargetMachineFactoryFn;
use crate::{CodegenResults, ModuleCodegen};

use rustc_ast::expand::allocator::AllocatorKind;
use rustc_data_structures::fx::FxIndexMap;
use rustc_data_structures::sync::{DynSend, DynSync};
use rustc_errors::ErrorGuaranteed;
use rustc_metadata::EncodedMetadata;
use rustc_middle::dep_graph::{WorkProduct, WorkProductId};
use rustc_middle::query::{ExternProviders, Providers};
use rustc_middle::ty::layout::{FnAbiOf, HasTyCtxt, LayoutOf, TyAndLayout};
use rustc_middle::ty::{Ty, TyCtxt};
use rustc_session::{
    config::{self, OutputFilenames, PrintRequest},
    cstore::MetadataLoaderDyn,
    Session,
};
use rustc_span::symbol::Symbol;
use rustc_target::abi::call::FnAbi;
use rustc_target::spec::Target;

use std::fmt;

pub trait BackendTypes {
    type Value: CodegenObject;
    type Function: CodegenObject;

    type BasicBlock: Copy;
    type Type: CodegenObject;
    type Funclet;

    // FIXME(eddyb) find a common convention for all of the debuginfo-related
    // names (choose between `Dbg`, `Debug`, `DebugInfo`, `DI` etc.).
    type DIScope: Copy;
    type DILocation: Copy;
    type DIVariable: Copy;
}

pub trait Backend<'tcx>:
    Sized
    + BackendTypes
    + HasTyCtxt<'tcx>
    + LayoutOf<'tcx, LayoutOfResult = TyAndLayout<'tcx>>
    + FnAbiOf<'tcx, FnAbiOfResult = &'tcx FnAbi<'tcx, Ty<'tcx>>>
{
}

impl<'tcx, T> Backend<'tcx> for T where
    Self: BackendTypes
        + HasTyCtxt<'tcx>
        + LayoutOf<'tcx, LayoutOfResult = TyAndLayout<'tcx>>
        + FnAbiOf<'tcx, FnAbiOfResult = &'tcx FnAbi<'tcx, Ty<'tcx>>>
{
}

pub trait CodegenBackend {
    /// Locale resources for diagnostic messages - a string the content of the Fluent resource.
    /// Called before `init` so that all other functions are able to emit translatable diagnostics.
    fn locale_resource(&self) -> &'static str;

    fn init(&self, _sess: &Session) {}
    fn print(&self, _req: &PrintRequest, _out: &mut dyn PrintBackendInfo, _sess: &Session) {}
    fn target_features(&self, _sess: &Session, _allow_unstable: bool) -> Vec<Symbol> {
        vec![]
    }
    fn print_passes(&self) {}
    fn print_version(&self) {}

    /// If this plugin provides additional builtin targets, provide the one enabled by the options here.
    /// Be careful: this is called *before* init() is called.
    fn target_override(&self, _opts: &config::Options) -> Option<Target> {
        None
    }

    /// The metadata loader used to load rlib and dylib metadata.
    ///
    /// Alternative codegen backends may want to use different rlib or dylib formats than the
    /// default native static archives and dynamic libraries.
    fn metadata_loader(&self) -> Box<MetadataLoaderDyn> {
        Box::new(crate::back::metadata::DefaultMetadataLoader)
    }

    fn provide(&self, _providers: &mut Providers) {}
    fn provide_extern(&self, _providers: &mut ExternProviders) {}
    fn codegen_crate<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        metadata: EncodedMetadata,
        need_metadata_module: bool,
    ) -> Box<dyn Any>;

    /// This is called on the returned `Box<dyn Any>` from `codegen_backend`
    ///
    /// # Panics
    ///
    /// Panics when the passed `Box<dyn Any>` was not returned by `codegen_backend`.
    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn Any>,
        sess: &Session,
        outputs: &OutputFilenames,
    ) -> Result<(CodegenResults, FxIndexMap<WorkProductId, WorkProduct>), ErrorGuaranteed>;

    /// This is called on the returned `Box<dyn Any>` from `join_codegen`
    ///
    /// # Panics
    ///
    /// Panics when the passed `Box<dyn Any>` was not returned by `join_codegen`.
    fn link(
        &self,
        sess: &Session,
        codegen_results: CodegenResults,
        outputs: &OutputFilenames,
    ) -> Result<(), ErrorGuaranteed>;
}

pub trait ExtraBackendMethods:
    CodegenBackend + WriteBackendMethods + Sized + Send + Sync + DynSend + DynSync
{
    fn codegen_allocator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        module_name: &str,
        kind: AllocatorKind,
        alloc_error_handler_kind: AllocatorKind,
    ) -> Self::Module;
    /// This generates the codegen unit and returns it along with
    /// a `u64` giving an estimate of the unit's processing cost.
    fn compile_codegen_unit(
        &self,
        tcx: TyCtxt<'_>,
        cgu_name: Symbol,
    ) -> (ModuleCodegen<Self::Module>, u64);
    fn target_machine_factory(
        &self,
        sess: &Session,
        opt_level: config::OptLevel,
        target_features: &[String],
    ) -> TargetMachineFactoryFn<Self>;

    fn spawn_thread<F, T>(_time_trace: bool, f: F) -> std::thread::JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        std::thread::spawn(f)
    }

    fn spawn_named_thread<F, T>(
        _time_trace: bool,
        name: String,
        f: F,
    ) -> std::io::Result<std::thread::JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        std::thread::Builder::new().name(name).spawn(f)
    }
}

pub trait PrintBackendInfo {
    fn infallible_write_fmt(&mut self, args: fmt::Arguments<'_>);
}

impl PrintBackendInfo for String {
    fn infallible_write_fmt(&mut self, args: fmt::Arguments<'_>) {
        fmt::Write::write_fmt(self, args).unwrap();
    }
}

impl dyn PrintBackendInfo + '_ {
    pub fn write_fmt(&mut self, args: fmt::Arguments<'_>) {
        self.infallible_write_fmt(args);
    }
}
