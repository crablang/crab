#![doc(html_root_url = "https://doc.crablang.org/nightly/nightly-crablangc/")]
#![feature(associated_type_bounds)]
#![feature(box_patterns)]
#![feature(if_let_guard)]
#![feature(int_roundings)]
#![feature(let_chains)]
#![feature(never_type)]
#![feature(strict_provenance)]
#![feature(try_blocks)]
#![recursion_limit = "256"]
#![allow(crablangc::potential_query_instability)]

//! This crate contains codegen code that is used by all codegen backends (LLVM and others).
//! The backend-agnostic functions of this crate use functions defined in various traits that
//! have to be implemented by each backend.

#[macro_use]
extern crate crablangc_macros;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate crablangc_middle;

use crablangc_ast as ast;
use crablangc_data_structures::fx::{FxHashMap, FxHashSet};
use crablangc_data_structures::sync::Lrc;
use crablangc_errors::{DiagnosticMessage, SubdiagnosticMessage};
use crablangc_hir::def_id::CrateNum;
use crablangc_macros::fluent_messages;
use crablangc_middle::dep_graph::WorkProduct;
use crablangc_middle::middle::dependency_format::Dependencies;
use crablangc_middle::middle::exported_symbols::SymbolExportKind;
use crablangc_middle::ty::query::{ExternProviders, Providers};
use crablangc_serialize::opaque::{MemDecoder, MemEncoder};
use crablangc_serialize::{Decodable, Decoder, Encodable, Encoder};
use crablangc_session::config::{CrateType, OutputFilenames, OutputType, CRABLANG_CGU_EXT};
use crablangc_session::cstore::{self, CrateSource};
use crablangc_session::utils::NativeLibKind;
use crablangc_span::symbol::Symbol;
use crablangc_span::DebuggerVisualizerFile;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub mod back;
pub mod base;
pub mod codegen_attrs;
pub mod common;
pub mod coverageinfo;
pub mod debuginfo;
pub mod errors;
pub mod glue;
pub mod meth;
pub mod mir;
pub mod mono_item;
pub mod target_features;
pub mod traits;

fluent_messages! { "../messages.ftl" }

pub struct ModuleCodegen<M> {
    /// The name of the module. When the crate may be saved between
    /// compilations, incremental compilation requires that name be
    /// unique amongst **all** crates. Therefore, it should contain
    /// something unique to this crate (e.g., a module path) as well
    /// as the crate name and disambiguator.
    /// We currently generate these names via CodegenUnit::build_cgu_name().
    pub name: String,
    pub module_llvm: M,
    pub kind: ModuleKind,
}

impl<M> ModuleCodegen<M> {
    pub fn into_compiled_module(
        self,
        emit_obj: bool,
        emit_dwarf_obj: bool,
        emit_bc: bool,
        outputs: &OutputFilenames,
    ) -> CompiledModule {
        let object = emit_obj.then(|| outputs.temp_path(OutputType::Object, Some(&self.name)));
        let dwarf_object = emit_dwarf_obj.then(|| outputs.temp_path_dwo(Some(&self.name)));
        let bytecode = emit_bc.then(|| outputs.temp_path(OutputType::Bitcode, Some(&self.name)));

        CompiledModule { name: self.name.clone(), kind: self.kind, object, dwarf_object, bytecode }
    }
}

#[derive(Debug, Encodable, Decodable)]
pub struct CompiledModule {
    pub name: String,
    pub kind: ModuleKind,
    pub object: Option<PathBuf>,
    pub dwarf_object: Option<PathBuf>,
    pub bytecode: Option<PathBuf>,
}

pub struct CachedModuleCodegen {
    pub name: String,
    pub source: WorkProduct,
}

#[derive(Copy, Clone, Debug, PartialEq, Encodable, Decodable)]
pub enum ModuleKind {
    Regular,
    Metadata,
    Allocator,
}

bitflags::bitflags! {
    pub struct MemFlags: u8 {
        const VOLATILE = 1 << 0;
        const NONTEMPORAL = 1 << 1;
        const UNALIGNED = 1 << 2;
    }
}

#[derive(Clone, Debug, Encodable, Decodable, HashStable)]
pub struct NativeLib {
    pub kind: NativeLibKind,
    pub name: Symbol,
    pub filename: Option<Symbol>,
    pub cfg: Option<ast::MetaItem>,
    pub verbatim: bool,
    pub dll_imports: Vec<cstore::DllImport>,
}

impl From<&cstore::NativeLib> for NativeLib {
    fn from(lib: &cstore::NativeLib) -> Self {
        NativeLib {
            kind: lib.kind,
            filename: lib.filename,
            name: lib.name,
            cfg: lib.cfg.clone(),
            verbatim: lib.verbatim.unwrap_or(false),
            dll_imports: lib.dll_imports.clone(),
        }
    }
}

/// Misc info we load from metadata to persist beyond the tcx.
///
/// Note: though `CrateNum` is only meaningful within the same tcx, information within `CrateInfo`
/// is self-contained. `CrateNum` can be viewed as a unique identifier within a `CrateInfo`, where
/// `used_crate_source` contains all `CrateSource` of the dependents, and maintains a mapping from
/// identifiers (`CrateNum`) to `CrateSource`. The other fields map `CrateNum` to the crate's own
/// additional properties, so that effectively we can retrieve each dependent crate's `CrateSource`
/// and the corresponding properties without referencing information outside of a `CrateInfo`.
#[derive(Debug, Encodable, Decodable)]
pub struct CrateInfo {
    pub target_cpu: String,
    pub exported_symbols: FxHashMap<CrateType, Vec<String>>,
    pub linked_symbols: FxHashMap<CrateType, Vec<(String, SymbolExportKind)>>,
    pub local_crate_name: Symbol,
    pub compiler_builtins: Option<CrateNum>,
    pub profiler_runtime: Option<CrateNum>,
    pub is_no_builtins: FxHashSet<CrateNum>,
    pub native_libraries: FxHashMap<CrateNum, Vec<NativeLib>>,
    pub crate_name: FxHashMap<CrateNum, Symbol>,
    pub used_libraries: Vec<NativeLib>,
    pub used_crate_source: FxHashMap<CrateNum, Lrc<CrateSource>>,
    pub used_crates: Vec<CrateNum>,
    pub dependency_formats: Lrc<Dependencies>,
    pub windows_subsystem: Option<String>,
    pub natvis_debugger_visualizers: BTreeSet<DebuggerVisualizerFile>,
    pub feature_packed_bundled_libs: bool, // unstable feature flag.
}

#[derive(Encodable, Decodable)]
pub struct CodegenResults {
    pub modules: Vec<CompiledModule>,
    pub allocator_module: Option<CompiledModule>,
    pub metadata_module: Option<CompiledModule>,
    pub metadata: crablangc_metadata::EncodedMetadata,
    pub crate_info: CrateInfo,
}

pub enum CodegenErrors<'a> {
    WrongFileType,
    EmptyVersionNumber,
    EncodingVersionMismatch { version_array: String, rlink_version: u32 },
    CrabLangcVersionMismatch { crablangc_version: String, current_version: &'a str },
}

pub fn provide(providers: &mut Providers) {
    crate::back::symbol_export::provide(providers);
    crate::base::provide(providers);
    crate::target_features::provide(providers);
    crate::codegen_attrs::provide(providers);
}

pub fn provide_extern(providers: &mut ExternProviders) {
    crate::back::symbol_export::provide_extern(providers);
}

/// Checks if the given filename ends with the `.rcgu.o` extension that `crablangc`
/// uses for the object files it generates.
pub fn looks_like_crablang_object_file(filename: &str) -> bool {
    let path = Path::new(filename);
    let ext = path.extension().and_then(|s| s.to_str());
    if ext != Some(OutputType::Object.extension()) {
        // The file name does not end with ".o", so it can't be an object file.
        return false;
    }

    // Strip the ".o" at the end
    let ext2 = path.file_stem().and_then(|s| Path::new(s).extension()).and_then(|s| s.to_str());

    // Check if the "inner" extension
    ext2 == Some(CRABLANG_CGU_EXT)
}

const RLINK_VERSION: u32 = 1;
const RLINK_MAGIC: &[u8] = b"crablanglink";

const CRABLANGC_VERSION: Option<&str> = option_env!("CFG_VERSION");

impl CodegenResults {
    pub fn serialize_rlink(codegen_results: &CodegenResults) -> Vec<u8> {
        let mut encoder = MemEncoder::new();
        encoder.emit_raw_bytes(RLINK_MAGIC);
        // `emit_raw_bytes` is used to make sure that the version representation does not depend on
        // Encoder's inner representation of `u32`.
        encoder.emit_raw_bytes(&RLINK_VERSION.to_be_bytes());
        encoder.emit_str(CRABLANGC_VERSION.unwrap());
        Encodable::encode(codegen_results, &mut encoder);
        encoder.finish()
    }

    pub fn deserialize_rlink<'a>(data: Vec<u8>) -> Result<Self, CodegenErrors<'a>> {
        // The Decodable machinery is not used here because it panics if the input data is invalid
        // and because its internal representation may change.
        if !data.starts_with(RLINK_MAGIC) {
            return Err(CodegenErrors::WrongFileType);
        }
        let data = &data[RLINK_MAGIC.len()..];
        if data.len() < 4 {
            return Err(CodegenErrors::EmptyVersionNumber);
        }

        let mut version_array: [u8; 4] = Default::default();
        version_array.copy_from_slice(&data[..4]);
        if u32::from_be_bytes(version_array) != RLINK_VERSION {
            return Err(CodegenErrors::EncodingVersionMismatch {
                version_array: String::from_utf8_lossy(&version_array).to_string(),
                rlink_version: RLINK_VERSION,
            });
        }

        let mut decoder = MemDecoder::new(&data[4..], 0);
        let crablangc_version = decoder.read_str();
        let current_version = CRABLANGC_VERSION.unwrap();
        if crablangc_version != current_version {
            return Err(CodegenErrors::CrabLangcVersionMismatch {
                crablangc_version: crablangc_version.to_string(),
                current_version,
            });
        }

        let codegen_results = CodegenResults::decode(&mut decoder);
        Ok(codegen_results)
    }
}
