use crate::errors::{FailCreateFileEncoder, FailSeekFile, FailWriteFile};
use crate::rmeta::def_path_hash_map::DefPathHashMapRef;
use crate::rmeta::table::TableBuilder;
use crate::rmeta::*;

use rustc_ast::expand::StrippedCfgItem;
use rustc_ast::Attribute;
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::{FxHashMap, FxIndexSet};
use rustc_data_structures::memmap::{Mmap, MmapMut};
use rustc_data_structures::stable_hasher::{Hash128, HashStable, StableHasher};
use rustc_data_structures::sync::{join, par_for_each_in, Lrc};
use rustc_data_structures::temp_dir::MaybeTempDir;
use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::{
    CrateNum, DefId, DefIndex, LocalDefId, CRATE_DEF_ID, CRATE_DEF_INDEX, LOCAL_CRATE,
};
use rustc_hir::definitions::DefPathData;
use rustc_hir::intravisit;
use rustc_hir::lang_items::LangItem;
use rustc_middle::middle::debugger_visualizer::DebuggerVisualizerFile;
use rustc_middle::middle::dependency_format::Linkage;
use rustc_middle::middle::exported_symbols::{
    metadata_symbol_name, ExportedSymbol, SymbolExportInfo,
};
use rustc_middle::mir::interpret;
use rustc_middle::query::LocalCrate;
use rustc_middle::query::Providers;
use rustc_middle::traits::specialization_graph;
use rustc_middle::ty::codec::TyEncoder;
use rustc_middle::ty::fast_reject::{self, SimplifiedType, TreatParams};
use rustc_middle::ty::{self, AssocItemContainer, SymbolName, Ty, TyCtxt};
use rustc_middle::util::common::to_readable_str;
use rustc_serialize::{opaque, Decodable, Decoder, Encodable, Encoder};
use rustc_session::config::{CrateType, OptLevel};
use rustc_session::cstore::{ForeignModule, LinkagePreference, NativeLib};
use rustc_span::hygiene::{ExpnIndex, HygieneEncodeContext, MacroKind};
use rustc_span::symbol::{sym, Symbol};
use rustc_span::{self, ExternalSource, FileName, SourceFile, Span, SpanData, SyntaxContext};
use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::io::{Read, Seek, Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

pub(super) struct EncodeContext<'a, 'tcx> {
    opaque: opaque::FileEncoder,
    tcx: TyCtxt<'tcx>,
    feat: &'tcx rustc_feature::Features,

    tables: TableBuilders,

    lazy_state: LazyState,
    span_shorthands: FxHashMap<Span, usize>,
    type_shorthands: FxHashMap<Ty<'tcx>, usize>,
    predicate_shorthands: FxHashMap<ty::PredicateKind<'tcx>, usize>,

    interpret_allocs: FxIndexSet<interpret::AllocId>,

    // This is used to speed up Span encoding.
    // The `usize` is an index into the `MonotonicVec`
    // that stores the `SourceFile`
    source_file_cache: (Lrc<SourceFile>, usize),
    // The indices (into the `SourceMap`'s `MonotonicVec`)
    // of all of the `SourceFiles` that we need to serialize.
    // When we serialize a `Span`, we insert the index of its
    // `SourceFile` into the `FxIndexSet`.
    // The order inside the `FxIndexSet` is used as on-disk
    // order of `SourceFiles`, and encoded inside `Span`s.
    required_source_files: Option<FxIndexSet<usize>>,
    is_proc_macro: bool,
    hygiene_ctxt: &'a HygieneEncodeContext,
    symbol_table: FxHashMap<Symbol, usize>,
}

/// If the current crate is a proc-macro, returns early with `LazyArray::default()`.
/// This is useful for skipping the encoding of things that aren't needed
/// for proc-macro crates.
macro_rules! empty_proc_macro {
    ($self:ident) => {
        if $self.is_proc_macro {
            return LazyArray::default();
        }
    };
}

macro_rules! encoder_methods {
    ($($name:ident($ty:ty);)*) => {
        $(fn $name(&mut self, value: $ty) {
            self.opaque.$name(value)
        })*
    }
}

impl<'a, 'tcx> Encoder for EncodeContext<'a, 'tcx> {
    encoder_methods! {
        emit_usize(usize);
        emit_u128(u128);
        emit_u64(u64);
        emit_u32(u32);
        emit_u16(u16);
        emit_u8(u8);

        emit_isize(isize);
        emit_i128(i128);
        emit_i64(i64);
        emit_i32(i32);
        emit_i16(i16);

        emit_raw_bytes(&[u8]);
    }
}

impl<'a, 'tcx, T> Encodable<EncodeContext<'a, 'tcx>> for LazyValue<T> {
    fn encode(&self, e: &mut EncodeContext<'a, 'tcx>) {
        e.emit_lazy_distance(self.position);
    }
}

impl<'a, 'tcx, T> Encodable<EncodeContext<'a, 'tcx>> for LazyArray<T> {
    fn encode(&self, e: &mut EncodeContext<'a, 'tcx>) {
        e.emit_usize(self.num_elems);
        if self.num_elems > 0 {
            e.emit_lazy_distance(self.position)
        }
    }
}

impl<'a, 'tcx, I, T> Encodable<EncodeContext<'a, 'tcx>> for LazyTable<I, T> {
    fn encode(&self, e: &mut EncodeContext<'a, 'tcx>) {
        e.emit_usize(self.encoded_size);
        e.emit_lazy_distance(self.position);
    }
}

impl<'a, 'tcx> Encodable<EncodeContext<'a, 'tcx>> for CrateNum {
    fn encode(&self, s: &mut EncodeContext<'a, 'tcx>) {
        if *self != LOCAL_CRATE && s.is_proc_macro {
            panic!("Attempted to encode non-local CrateNum {self:?} for proc-macro crate");
        }
        s.emit_u32(self.as_u32());
    }
}

impl<'a, 'tcx> Encodable<EncodeContext<'a, 'tcx>> for DefIndex {
    fn encode(&self, s: &mut EncodeContext<'a, 'tcx>) {
        s.emit_u32(self.as_u32());
    }
}

impl<'a, 'tcx> Encodable<EncodeContext<'a, 'tcx>> for ExpnIndex {
    fn encode(&self, s: &mut EncodeContext<'a, 'tcx>) {
        s.emit_u32(self.as_u32());
    }
}

impl<'a, 'tcx> Encodable<EncodeContext<'a, 'tcx>> for SyntaxContext {
    fn encode(&self, s: &mut EncodeContext<'a, 'tcx>) {
        rustc_span::hygiene::raw_encode_syntax_context(*self, &s.hygiene_ctxt, s);
    }
}

impl<'a, 'tcx> Encodable<EncodeContext<'a, 'tcx>> for ExpnId {
    fn encode(&self, s: &mut EncodeContext<'a, 'tcx>) {
        if self.krate == LOCAL_CRATE {
            // We will only write details for local expansions. Non-local expansions will fetch
            // data from the corresponding crate's metadata.
            // FIXME(#43047) FIXME(#74731) We may eventually want to avoid relying on external
            // metadata from proc-macro crates.
            s.hygiene_ctxt.schedule_expn_data_for_encoding(*self);
        }
        self.krate.encode(s);
        self.local_id.encode(s);
    }
}

impl<'a, 'tcx> Encodable<EncodeContext<'a, 'tcx>> for Span {
    fn encode(&self, s: &mut EncodeContext<'a, 'tcx>) {
        match s.span_shorthands.entry(*self) {
            Entry::Occupied(o) => SpanEncodingMode::Shorthand(*o.get()).encode(s),
            Entry::Vacant(v) => {
                let position = s.opaque.position();
                v.insert(position);
                SpanEncodingMode::Direct.encode(s);
                self.data().encode(s);
            }
        }
    }
}

impl<'a, 'tcx> Encodable<EncodeContext<'a, 'tcx>> for SpanData {
    fn encode(&self, s: &mut EncodeContext<'a, 'tcx>) {
        // Don't serialize any `SyntaxContext`s from a proc-macro crate,
        // since we don't load proc-macro dependencies during serialization.
        // This means that any hygiene information from macros used *within*
        // a proc-macro crate (e.g. invoking a macro that expands to a proc-macro
        // definition) will be lost.
        //
        // This can show up in two ways:
        //
        // 1. Any hygiene information associated with identifier of
        // a proc macro (e.g. `#[proc_macro] pub fn $name`) will be lost.
        // Since proc-macros can only be invoked from a different crate,
        // real code should never need to care about this.
        //
        // 2. Using `Span::def_site` or `Span::mixed_site` will not
        // include any hygiene information associated with the definition
        // site. This means that a proc-macro cannot emit a `$crate`
        // identifier which resolves to one of its dependencies,
        // which also should never come up in practice.
        //
        // Additionally, this affects `Span::parent`, and any other
        // span inspection APIs that would otherwise allow traversing
        // the `SyntaxContexts` associated with a span.
        //
        // None of these user-visible effects should result in any
        // cross-crate inconsistencies (getting one behavior in the same
        // crate, and a different behavior in another crate) due to the
        // limited surface that proc-macros can expose.
        //
        // IMPORTANT: If this is ever changed, be sure to update
        // `rustc_span::hygiene::raw_encode_expn_id` to handle
        // encoding `ExpnData` for proc-macro crates.
        if s.is_proc_macro {
            SyntaxContext::root().encode(s);
        } else {
            self.ctxt.encode(s);
        }

        if self.is_dummy() {
            return TAG_PARTIAL_SPAN.encode(s);
        }

        // The Span infrastructure should make sure that this invariant holds:
        debug_assert!(self.lo <= self.hi);

        if !s.source_file_cache.0.contains(self.lo) {
            let source_map = s.tcx.sess.source_map();
            let source_file_index = source_map.lookup_source_file_idx(self.lo);
            s.source_file_cache =
                (source_map.files()[source_file_index].clone(), source_file_index);
        }
        let (ref source_file, source_file_index) = s.source_file_cache;
        debug_assert!(source_file.contains(self.lo));

        if !source_file.contains(self.hi) {
            // Unfortunately, macro expansion still sometimes generates Spans
            // that malformed in this way.
            return TAG_PARTIAL_SPAN.encode(s);
        }

        // There are two possible cases here:
        // 1. This span comes from a 'foreign' crate - e.g. some crate upstream of the
        // crate we are writing metadata for. When the metadata for *this* crate gets
        // deserialized, the deserializer will need to know which crate it originally came
        // from. We use `TAG_VALID_SPAN_FOREIGN` to indicate that a `CrateNum` should
        // be deserialized after the rest of the span data, which tells the deserializer
        // which crate contains the source map information.
        // 2. This span comes from our own crate. No special handling is needed - we just
        // write `TAG_VALID_SPAN_LOCAL` to let the deserializer know that it should use
        // our own source map information.
        //
        // If we're a proc-macro crate, we always treat this as a local `Span`.
        // In `encode_source_map`, we serialize foreign `SourceFile`s into our metadata
        // if we're a proc-macro crate.
        // This allows us to avoid loading the dependencies of proc-macro crates: all of
        // the information we need to decode `Span`s is stored in the proc-macro crate.
        let (tag, metadata_index) = if source_file.is_imported() && !s.is_proc_macro {
            // To simplify deserialization, we 'rebase' this span onto the crate it originally came
            // from (the crate that 'owns' the file it references. These rebased 'lo' and 'hi'
            // values are relative to the source map information for the 'foreign' crate whose
            // CrateNum we write into the metadata. This allows `imported_source_files` to binary
            // search through the 'foreign' crate's source map information, using the
            // deserialized 'lo' and 'hi' values directly.
            //
            // All of this logic ensures that the final result of deserialization is a 'normal'
            // Span that can be used without any additional trouble.
            let metadata_index = {
                // Introduce a new scope so that we drop the 'lock()' temporary
                match &*source_file.external_src.lock() {
                    ExternalSource::Foreign { metadata_index, .. } => *metadata_index,
                    src => panic!("Unexpected external source {src:?}"),
                }
            };

            (TAG_VALID_SPAN_FOREIGN, metadata_index)
        } else {
            // Record the fact that we need to encode the data for this `SourceFile`
            let source_files =
                s.required_source_files.as_mut().expect("Already encoded SourceMap!");
            let (metadata_index, _) = source_files.insert_full(source_file_index);
            let metadata_index: u32 =
                metadata_index.try_into().expect("cannot export more than U32_MAX files");

            (TAG_VALID_SPAN_LOCAL, metadata_index)
        };

        // Encode the start position relative to the file start, so we profit more from the
        // variable-length integer encoding.
        let lo = self.lo - source_file.start_pos;

        // Encode length which is usually less than span.hi and profits more
        // from the variable-length integer encoding that we use.
        let len = self.hi - self.lo;

        tag.encode(s);
        lo.encode(s);
        len.encode(s);

        // Encode the index of the `SourceFile` for the span, in order to make decoding faster.
        metadata_index.encode(s);

        if tag == TAG_VALID_SPAN_FOREIGN {
            // This needs to be two lines to avoid holding the `s.source_file_cache`
            // while calling `cnum.encode(s)`
            let cnum = s.source_file_cache.0.cnum;
            cnum.encode(s);
        }
    }
}

impl<'a, 'tcx> Encodable<EncodeContext<'a, 'tcx>> for Symbol {
    fn encode(&self, s: &mut EncodeContext<'a, 'tcx>) {
        // if symbol preinterned, emit tag and symbol index
        if self.is_preinterned() {
            s.opaque.emit_u8(SYMBOL_PREINTERNED);
            s.opaque.emit_u32(self.as_u32());
        } else {
            // otherwise write it as string or as offset to it
            match s.symbol_table.entry(*self) {
                Entry::Vacant(o) => {
                    s.opaque.emit_u8(SYMBOL_STR);
                    let pos = s.opaque.position();
                    o.insert(pos);
                    s.emit_str(self.as_str());
                }
                Entry::Occupied(o) => {
                    let x = *o.get();
                    s.emit_u8(SYMBOL_OFFSET);
                    s.emit_usize(x);
                }
            }
        }
    }
}

impl<'a, 'tcx> TyEncoder for EncodeContext<'a, 'tcx> {
    const CLEAR_CROSS_CRATE: bool = true;

    type I = TyCtxt<'tcx>;

    fn position(&self) -> usize {
        self.opaque.position()
    }

    fn type_shorthands(&mut self) -> &mut FxHashMap<Ty<'tcx>, usize> {
        &mut self.type_shorthands
    }

    fn predicate_shorthands(&mut self) -> &mut FxHashMap<ty::PredicateKind<'tcx>, usize> {
        &mut self.predicate_shorthands
    }

    fn encode_alloc_id(&mut self, alloc_id: &rustc_middle::mir::interpret::AllocId) {
        let (index, _) = self.interpret_allocs.insert_full(*alloc_id);

        index.encode(self);
    }
}

// Shorthand for `$self.$tables.$table.set_some($def_id.index, $self.lazy_value($value))`, which would
// normally need extra variables to avoid errors about multiple mutable borrows.
macro_rules! record {
    ($self:ident.$tables:ident.$table:ident[$def_id:expr] <- $value:expr) => {{
        {
            let value = $value;
            let lazy = $self.lazy(value);
            $self.$tables.$table.set_some($def_id.index, lazy);
        }
    }};
}

// Shorthand for `$self.$tables.$table.set_some($def_id.index, $self.lazy_value($value))`, which would
// normally need extra variables to avoid errors about multiple mutable borrows.
macro_rules! record_array {
    ($self:ident.$tables:ident.$table:ident[$def_id:expr] <- $value:expr) => {{
        {
            let value = $value;
            let lazy = $self.lazy_array(value);
            $self.$tables.$table.set_some($def_id.index, lazy);
        }
    }};
}

macro_rules! record_defaulted_array {
    ($self:ident.$tables:ident.$table:ident[$def_id:expr] <- $value:expr) => {{
        {
            let value = $value;
            let lazy = $self.lazy_array(value);
            $self.$tables.$table.set($def_id.index, lazy);
        }
    }};
}

impl<'a, 'tcx> EncodeContext<'a, 'tcx> {
    fn emit_lazy_distance(&mut self, position: NonZeroUsize) {
        let pos = position.get();
        let distance = match self.lazy_state {
            LazyState::NoNode => bug!("emit_lazy_distance: outside of a metadata node"),
            LazyState::NodeStart(start) => {
                let start = start.get();
                assert!(pos <= start);
                start - pos
            }
            LazyState::Previous(last_pos) => {
                assert!(
                    last_pos <= position,
                    "make sure that the calls to `lazy*` \
                     are in the same order as the metadata fields",
                );
                position.get() - last_pos.get()
            }
        };
        self.lazy_state = LazyState::Previous(NonZeroUsize::new(pos).unwrap());
        self.emit_usize(distance);
    }

    fn lazy<T: ParameterizedOverTcx, B: Borrow<T::Value<'tcx>>>(&mut self, value: B) -> LazyValue<T>
    where
        T::Value<'tcx>: Encodable<EncodeContext<'a, 'tcx>>,
    {
        let pos = NonZeroUsize::new(self.position()).unwrap();

        assert_eq!(self.lazy_state, LazyState::NoNode);
        self.lazy_state = LazyState::NodeStart(pos);
        value.borrow().encode(self);
        self.lazy_state = LazyState::NoNode;

        assert!(pos.get() <= self.position());

        LazyValue::from_position(pos)
    }

    fn lazy_array<T: ParameterizedOverTcx, I: IntoIterator<Item = B>, B: Borrow<T::Value<'tcx>>>(
        &mut self,
        values: I,
    ) -> LazyArray<T>
    where
        T::Value<'tcx>: Encodable<EncodeContext<'a, 'tcx>>,
    {
        let pos = NonZeroUsize::new(self.position()).unwrap();

        assert_eq!(self.lazy_state, LazyState::NoNode);
        self.lazy_state = LazyState::NodeStart(pos);
        let len = values.into_iter().map(|value| value.borrow().encode(self)).count();
        self.lazy_state = LazyState::NoNode;

        assert!(pos.get() <= self.position());

        LazyArray::from_position_and_num_elems(pos, len)
    }

    fn encode_def_path_table(&mut self) {
        let table = self.tcx.def_path_table();
        if self.is_proc_macro {
            for def_index in std::iter::once(CRATE_DEF_INDEX)
                .chain(self.tcx.resolutions(()).proc_macros.iter().map(|p| p.local_def_index))
            {
                let def_key = self.lazy(table.def_key(def_index));
                let def_path_hash = table.def_path_hash(def_index);
                self.tables.def_keys.set_some(def_index, def_key);
                self.tables.def_path_hashes.set(def_index, def_path_hash);
            }
        } else {
            for (def_index, def_key, def_path_hash) in table.enumerated_keys_and_path_hashes() {
                let def_key = self.lazy(def_key);
                self.tables.def_keys.set_some(def_index, def_key);
                self.tables.def_path_hashes.set(def_index, *def_path_hash);
            }
        }
    }

    fn encode_def_path_hash_map(&mut self) -> LazyValue<DefPathHashMapRef<'static>> {
        self.lazy(DefPathHashMapRef::BorrowedFromTcx(self.tcx.def_path_hash_to_def_index_map()))
    }

    fn encode_source_map(&mut self) -> LazyTable<u32, Option<LazyValue<rustc_span::SourceFile>>> {
        let source_map = self.tcx.sess.source_map();
        let all_source_files = source_map.files();

        // By replacing the `Option` with `None`, we ensure that we can't
        // accidentally serialize any more `Span`s after the source map encoding
        // is done.
        let required_source_files = self.required_source_files.take().unwrap();

        let working_directory = &self.tcx.sess.opts.working_dir;

        let mut adapted = TableBuilder::default();

        // Only serialize `SourceFile`s that were used during the encoding of a `Span`.
        //
        // The order in which we encode source files is important here: the on-disk format for
        // `Span` contains the index of the corresponding `SourceFile`.
        for (on_disk_index, &source_file_index) in required_source_files.iter().enumerate() {
            let source_file = &all_source_files[source_file_index];
            // Don't serialize imported `SourceFile`s, unless we're in a proc-macro crate.
            assert!(!source_file.is_imported() || self.is_proc_macro);

            // At export time we expand all source file paths to absolute paths because
            // downstream compilation sessions can have a different compiler working
            // directory, so relative paths from this or any other upstream crate
            // won't be valid anymore.
            //
            // At this point we also erase the actual on-disk path and only keep
            // the remapped version -- as is necessary for reproducible builds.
            let mut source_file = match source_file.name {
                FileName::Real(ref original_file_name) => {
                    let adapted_file_name = source_map
                        .path_mapping()
                        .to_embeddable_absolute_path(original_file_name.clone(), working_directory);

                    if adapted_file_name != *original_file_name {
                        let mut adapted: SourceFile = (**source_file).clone();
                        adapted.name = FileName::Real(adapted_file_name);
                        adapted.name_hash = {
                            let mut hasher: StableHasher = StableHasher::new();
                            adapted.name.hash(&mut hasher);
                            hasher.finish::<Hash128>()
                        };
                        Lrc::new(adapted)
                    } else {
                        // Nothing to adapt
                        source_file.clone()
                    }
                }
                // expanded code, not from a file
                _ => source_file.clone(),
            };

            // We're serializing this `SourceFile` into our crate metadata,
            // so mark it as coming from this crate.
            // This also ensures that we don't try to deserialize the
            // `CrateNum` for a proc-macro dependency - since proc macro
            // dependencies aren't loaded when we deserialize a proc-macro,
            // trying to remap the `CrateNum` would fail.
            if self.is_proc_macro {
                Lrc::make_mut(&mut source_file).cnum = LOCAL_CRATE;
            }

            let on_disk_index: u32 =
                on_disk_index.try_into().expect("cannot export more than U32_MAX files");
            adapted.set_some(on_disk_index, self.lazy(source_file));
        }

        adapted.encode(&mut self.opaque)
    }

    fn encode_crate_root(&mut self) -> LazyValue<CrateRoot> {
        let tcx = self.tcx;
        let mut stats: Vec<(&'static str, usize)> = Vec::with_capacity(32);

        macro_rules! stat {
            ($label:literal, $f:expr) => {{
                let orig_pos = self.position();
                let res = $f();
                stats.push(($label, self.position() - orig_pos));
                res
            }};
        }

        // We have already encoded some things. Get their combined size from the current position.
        stats.push(("preamble", self.position()));

        let (crate_deps, dylib_dependency_formats) =
            stat!("dep", || (self.encode_crate_deps(), self.encode_dylib_dependency_formats()));

        let lib_features = stat!("lib-features", || self.encode_lib_features());

        let stability_implications =
            stat!("stability-implications", || self.encode_stability_implications());

        let (lang_items, lang_items_missing) = stat!("lang-items", || {
            (self.encode_lang_items(), self.encode_lang_items_missing())
        });

        let stripped_cfg_items = stat!("stripped-cfg-items", || self.encode_stripped_cfg_items());

        let diagnostic_items = stat!("diagnostic-items", || self.encode_diagnostic_items());

        let native_libraries = stat!("native-libs", || self.encode_native_libraries());

        let foreign_modules = stat!("foreign-modules", || self.encode_foreign_modules());

        _ = stat!("def-path-table", || self.encode_def_path_table());

        // Encode the def IDs of traits, for rustdoc and diagnostics.
        let traits = stat!("traits", || self.encode_traits());

        // Encode the def IDs of impls, for coherence checking.
        let impls = stat!("impls", || self.encode_impls());

        let incoherent_impls = stat!("incoherent-impls", || self.encode_incoherent_impls());

        _ = stat!("mir", || self.encode_mir());

        _ = stat!("def-ids", || self.encode_def_ids());

        let interpret_alloc_index = stat!("interpret-alloc-index", || {
            let mut interpret_alloc_index = Vec::new();
            let mut n = 0;
            trace!("beginning to encode alloc ids");
            loop {
                let new_n = self.interpret_allocs.len();
                // if we have found new ids, serialize those, too
                if n == new_n {
                    // otherwise, abort
                    break;
                }
                trace!("encoding {} further alloc ids", new_n - n);
                for idx in n..new_n {
                    let id = self.interpret_allocs[idx];
                    let pos = self.position() as u64;
                    interpret_alloc_index.push(pos);
                    interpret::specialized_encode_alloc_id(self, tcx, id);
                }
                n = new_n;
            }
            self.lazy_array(interpret_alloc_index)
        });

        // Encode the proc macro data. This affects `tables`, so we need to do this before we
        // encode the tables. This overwrites def_keys, so it must happen after
        // encode_def_path_table.
        let proc_macro_data = stat!("proc-macro-data", || self.encode_proc_macros());

        let tables = stat!("tables", || self.tables.encode(&mut self.opaque));

        let debugger_visualizers =
            stat!("debugger-visualizers", || self.encode_debugger_visualizers());

        // Encode exported symbols info. This is prefetched in `encode_metadata` so we encode
        // this as late as possible to give the prefetching as much time as possible to complete.
        let exported_symbols = stat!("exported-symbols", || {
            self.encode_exported_symbols(&tcx.exported_symbols(LOCAL_CRATE))
        });

        // Encode the hygiene data.
        // IMPORTANT: this *must* be the last thing that we encode (other than `SourceMap`). The
        // process of encoding other items (e.g. `optimized_mir`) may cause us to load data from
        // the incremental cache. If this causes us to deserialize a `Span`, then we may load
        // additional `SyntaxContext`s into the global `HygieneData`. Therefore, we need to encode
        // the hygiene data last to ensure that we encode any `SyntaxContext`s that might be used.
        let (syntax_contexts, expn_data, expn_hashes) = stat!("hygiene", || self.encode_hygiene());

        let def_path_hash_map = stat!("def-path-hash-map", || self.encode_def_path_hash_map());

        // Encode source_map. This needs to be done last, because encoding `Span`s tells us which
        // `SourceFiles` we actually need to encode.
        let source_map = stat!("source-map", || self.encode_source_map());

        let root = stat!("final", || {
            let attrs = tcx.hir().krate_attrs();
            self.lazy(CrateRoot {
                header: CrateHeader {
                    name: tcx.crate_name(LOCAL_CRATE),
                    triple: tcx.sess.opts.target_triple.clone(),
                    hash: tcx.crate_hash(LOCAL_CRATE),
                    is_proc_macro_crate: proc_macro_data.is_some(),
                },
                extra_filename: tcx.sess.opts.cg.extra_filename.clone(),
                stable_crate_id: tcx.def_path_hash(LOCAL_CRATE.as_def_id()).stable_crate_id(),
                required_panic_strategy: tcx.required_panic_strategy(LOCAL_CRATE),
                panic_in_drop_strategy: tcx.sess.opts.unstable_opts.panic_in_drop,
                edition: tcx.sess.edition(),
                has_global_allocator: tcx.has_global_allocator(LOCAL_CRATE),
                has_alloc_error_handler: tcx.has_alloc_error_handler(LOCAL_CRATE),
                has_panic_handler: tcx.has_panic_handler(LOCAL_CRATE),
                has_default_lib_allocator: attr::contains_name(&attrs, sym::default_lib_allocator),
                proc_macro_data,
                debugger_visualizers,
                compiler_builtins: attr::contains_name(&attrs, sym::compiler_builtins),
                needs_allocator: attr::contains_name(&attrs, sym::needs_allocator),
                needs_panic_runtime: attr::contains_name(&attrs, sym::needs_panic_runtime),
                no_builtins: attr::contains_name(&attrs, sym::no_builtins),
                panic_runtime: attr::contains_name(&attrs, sym::panic_runtime),
                profiler_runtime: attr::contains_name(&attrs, sym::profiler_runtime),
                symbol_mangling_version: tcx.sess.opts.get_symbol_mangling_version(),

                crate_deps,
                dylib_dependency_formats,
                lib_features,
                stability_implications,
                lang_items,
                diagnostic_items,
                lang_items_missing,
                stripped_cfg_items,
                native_libraries,
                foreign_modules,
                source_map,
                traits,
                impls,
                incoherent_impls,
                exported_symbols,
                interpret_alloc_index,
                tables,
                syntax_contexts,
                expn_data,
                expn_hashes,
                def_path_hash_map,
            })
        });

        let total_bytes = self.position();

        let computed_total_bytes: usize = stats.iter().map(|(_, size)| size).sum();
        assert_eq!(total_bytes, computed_total_bytes);

        if tcx.sess.opts.unstable_opts.meta_stats {
            self.opaque.flush();

            // Rewind and re-read all the metadata to count the zero bytes we wrote.
            let pos_before_rewind = self.opaque.file().stream_position().unwrap();
            let mut zero_bytes = 0;
            self.opaque.file().rewind().unwrap();
            let file = std::io::BufReader::new(self.opaque.file());
            for e in file.bytes() {
                if e.unwrap() == 0 {
                    zero_bytes += 1;
                }
            }
            assert_eq!(self.opaque.file().stream_position().unwrap(), pos_before_rewind);

            stats.sort_by_key(|&(_, usize)| usize);

            let prefix = "meta-stats";
            let perc = |bytes| (bytes * 100) as f64 / total_bytes as f64;

            eprintln!("{prefix} METADATA STATS");
            eprintln!("{} {:<23}{:>10}", prefix, "Section", "Size");
            eprintln!("{prefix} ----------------------------------------------------------------");
            for (label, size) in stats {
                eprintln!(
                    "{} {:<23}{:>10} ({:4.1}%)",
                    prefix,
                    label,
                    to_readable_str(size),
                    perc(size)
                );
            }
            eprintln!("{prefix} ----------------------------------------------------------------");
            eprintln!(
                "{} {:<23}{:>10} (of which {:.1}% are zero bytes)",
                prefix,
                "Total",
                to_readable_str(total_bytes),
                perc(zero_bytes)
            );
            eprintln!("{prefix}");
        }

        root
    }
}

struct AnalyzeAttrState {
    is_exported: bool,
    is_doc_hidden: bool,
}

/// Returns whether an attribute needs to be recorded in metadata, that is, if it's usable and
/// useful in downstream crates. Local-only attributes are an obvious example, but some
/// rustdoc-specific attributes can equally be of use while documenting the current crate only.
///
/// Removing these superfluous attributes speeds up compilation by making the metadata smaller.
///
/// Note: the `is_exported` parameter is used to cache whether the given `DefId` has a public
/// visibility: this is a piece of data that can be computed once per defid, and not once per
/// attribute. Some attributes would only be usable downstream if they are public.
#[inline]
fn analyze_attr(attr: &Attribute, state: &mut AnalyzeAttrState) -> bool {
    let mut should_encode = false;
    if rustc_feature::is_builtin_only_local(attr.name_or_empty()) {
        // Attributes marked local-only don't need to be encoded for downstream crates.
    } else if attr.doc_str().is_some() {
        // We keep all doc comments reachable to rustdoc because they might be "imported" into
        // downstream crates if they use `#[doc(inline)]` to copy an item's documentation into
        // their own.
        if state.is_exported {
            should_encode = true;
        }
    } else if attr.has_name(sym::doc) {
        // If this is a `doc` attribute that doesn't have anything except maybe `inline` (as in
        // `#[doc(inline)]`), then we can remove it. It won't be inlinable in downstream crates.
        if let Some(item_list) = attr.meta_item_list() {
            for item in item_list {
                if !item.has_name(sym::inline) {
                    should_encode = true;
                    if item.has_name(sym::hidden) {
                        state.is_doc_hidden = true;
                        break;
                    }
                }
            }
        }
    } else {
        should_encode = true;
    }
    should_encode
}

fn should_encode_span(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Mod
        | DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::Trait
        | DefKind::TyAlias
        | DefKind::ForeignTy
        | DefKind::TraitAlias
        | DefKind::AssocTy
        | DefKind::TyParam
        | DefKind::ConstParam
        | DefKind::LifetimeParam
        | DefKind::Fn
        | DefKind::Const
        | DefKind::Static(_)
        | DefKind::Ctor(..)
        | DefKind::AssocFn
        | DefKind::AssocConst
        | DefKind::Macro(_)
        | DefKind::ExternCrate
        | DefKind::Use
        | DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::OpaqueTy
        | DefKind::Field
        | DefKind::Impl { .. }
        | DefKind::Closure
        | DefKind::Generator => true,
        DefKind::ForeignMod | DefKind::GlobalAsm => false,
    }
}

fn should_encode_attrs(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Mod
        | DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::Trait
        | DefKind::TyAlias
        | DefKind::ForeignTy
        | DefKind::TraitAlias
        | DefKind::AssocTy
        | DefKind::Fn
        | DefKind::Const
        | DefKind::Static(_)
        | DefKind::AssocFn
        | DefKind::AssocConst
        | DefKind::Macro(_)
        | DefKind::Field
        | DefKind::Impl { .. } => true,
        // Tools may want to be able to detect their tool lints on
        // closures from upstream crates, too. This is used by
        // https://github.com/model-checking/kani and is not a performance
        // or maintenance issue for us.
        DefKind::Closure => true,
        DefKind::TyParam
        | DefKind::ConstParam
        | DefKind::Ctor(..)
        | DefKind::ExternCrate
        | DefKind::Use
        | DefKind::ForeignMod
        | DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::OpaqueTy
        | DefKind::LifetimeParam
        | DefKind::GlobalAsm
        | DefKind::Generator => false,
    }
}

fn should_encode_expn_that_defined(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Mod
        | DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::Trait
        | DefKind::Impl { .. } => true,
        DefKind::TyAlias
        | DefKind::ForeignTy
        | DefKind::TraitAlias
        | DefKind::AssocTy
        | DefKind::TyParam
        | DefKind::Fn
        | DefKind::Const
        | DefKind::ConstParam
        | DefKind::Static(_)
        | DefKind::Ctor(..)
        | DefKind::AssocFn
        | DefKind::AssocConst
        | DefKind::Macro(_)
        | DefKind::ExternCrate
        | DefKind::Use
        | DefKind::ForeignMod
        | DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::OpaqueTy
        | DefKind::Field
        | DefKind::LifetimeParam
        | DefKind::GlobalAsm
        | DefKind::Closure
        | DefKind::Generator => false,
    }
}

fn should_encode_visibility(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Mod
        | DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::Trait
        | DefKind::TyAlias
        | DefKind::ForeignTy
        | DefKind::TraitAlias
        | DefKind::AssocTy
        | DefKind::Fn
        | DefKind::Const
        | DefKind::Static(..)
        | DefKind::Ctor(..)
        | DefKind::AssocFn
        | DefKind::AssocConst
        | DefKind::Macro(..)
        | DefKind::Field => true,
        DefKind::Use
        | DefKind::ForeignMod
        | DefKind::TyParam
        | DefKind::ConstParam
        | DefKind::LifetimeParam
        | DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::OpaqueTy
        | DefKind::GlobalAsm
        | DefKind::Impl { .. }
        | DefKind::Closure
        | DefKind::Generator
        | DefKind::ExternCrate => false,
    }
}

fn should_encode_stability(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Mod
        | DefKind::Ctor(..)
        | DefKind::Variant
        | DefKind::Field
        | DefKind::Struct
        | DefKind::AssocTy
        | DefKind::AssocFn
        | DefKind::AssocConst
        | DefKind::TyParam
        | DefKind::ConstParam
        | DefKind::Static(..)
        | DefKind::Const
        | DefKind::Fn
        | DefKind::ForeignMod
        | DefKind::TyAlias
        | DefKind::OpaqueTy
        | DefKind::Enum
        | DefKind::Union
        | DefKind::Impl { .. }
        | DefKind::Trait
        | DefKind::TraitAlias
        | DefKind::Macro(..)
        | DefKind::ForeignTy => true,
        DefKind::Use
        | DefKind::LifetimeParam
        | DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::GlobalAsm
        | DefKind::Closure
        | DefKind::Generator
        | DefKind::ExternCrate => false,
    }
}

/// Whether we should encode MIR.
///
/// Computing, optimizing and encoding the MIR is a relatively expensive operation.
/// We want to avoid this work when not required. Therefore:
/// - we only compute `mir_for_ctfe` on items with const-eval semantics;
/// - we skip `optimized_mir` for check runs.
///
/// Return a pair, resp. for CTFE and for LLVM.
fn should_encode_mir(tcx: TyCtxt<'_>, def_id: LocalDefId) -> (bool, bool) {
    match tcx.def_kind(def_id) {
        // Constructors
        DefKind::Ctor(_, _) => {
            let mir_opt_base = tcx.sess.opts.output_types.should_codegen()
                || tcx.sess.opts.unstable_opts.always_encode_mir;
            (true, mir_opt_base)
        }
        // Constants
        DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::AssocConst
        | DefKind::Static(..)
        | DefKind::Const => (true, false),
        // Full-fledged functions + closures
        DefKind::AssocFn | DefKind::Fn | DefKind::Closure => {
            let generics = tcx.generics_of(def_id);
            let needs_inline = (generics.requires_monomorphization(tcx)
                || tcx.codegen_fn_attrs(def_id).requests_inline())
                && tcx.sess.opts.output_types.should_codegen();
            // The function has a `const` modifier or is in a `#[const_trait]`.
            let is_const_fn = tcx.is_const_fn_raw(def_id.to_def_id())
                || tcx.is_const_default_method(def_id.to_def_id());
            let always_encode_mir = tcx.sess.opts.unstable_opts.always_encode_mir;
            (is_const_fn, needs_inline || always_encode_mir)
        }
        // Generators require optimized MIR to compute layout.
        DefKind::Generator => (false, true),
        // The others don't have MIR.
        _ => (false, false),
    }
}

fn should_encode_variances(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::OpaqueTy
        | DefKind::Fn
        | DefKind::Ctor(..)
        | DefKind::AssocFn => true,
        DefKind::Mod
        | DefKind::Field
        | DefKind::AssocTy
        | DefKind::AssocConst
        | DefKind::TyParam
        | DefKind::ConstParam
        | DefKind::Static(..)
        | DefKind::Const
        | DefKind::ForeignMod
        | DefKind::TyAlias
        | DefKind::Impl { .. }
        | DefKind::Trait
        | DefKind::TraitAlias
        | DefKind::Macro(..)
        | DefKind::ForeignTy
        | DefKind::Use
        | DefKind::LifetimeParam
        | DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::GlobalAsm
        | DefKind::Closure
        | DefKind::Generator
        | DefKind::ExternCrate => false,
    }
}

fn should_encode_generics(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::Trait
        | DefKind::TyAlias
        | DefKind::ForeignTy
        | DefKind::TraitAlias
        | DefKind::AssocTy
        | DefKind::Fn
        | DefKind::Const
        | DefKind::Static(..)
        | DefKind::Ctor(..)
        | DefKind::AssocFn
        | DefKind::AssocConst
        | DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::OpaqueTy
        | DefKind::Impl { .. }
        | DefKind::Field
        | DefKind::TyParam
        | DefKind::Closure
        | DefKind::Generator => true,
        DefKind::Mod
        | DefKind::ForeignMod
        | DefKind::ConstParam
        | DefKind::Macro(..)
        | DefKind::Use
        | DefKind::LifetimeParam
        | DefKind::GlobalAsm
        | DefKind::ExternCrate => false,
    }
}

fn should_encode_type(tcx: TyCtxt<'_>, def_id: LocalDefId, def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::Ctor(..)
        | DefKind::Field
        | DefKind::Fn
        | DefKind::Const
        | DefKind::Static(..)
        | DefKind::TyAlias
        | DefKind::ForeignTy
        | DefKind::Impl { .. }
        | DefKind::AssocFn
        | DefKind::AssocConst
        | DefKind::Closure
        | DefKind::Generator
        | DefKind::ConstParam
        | DefKind::AnonConst
        | DefKind::InlineConst => true,

        DefKind::OpaqueTy => {
            let origin = tcx.opaque_type_origin(def_id);
            if let hir::OpaqueTyOrigin::FnReturn(fn_def_id) | hir::OpaqueTyOrigin::AsyncFn(fn_def_id) = origin
                && let hir::Node::TraitItem(trait_item) = tcx.hir().get_by_def_id(fn_def_id)
                && let (_, hir::TraitFn::Required(..)) = trait_item.expect_fn()
            {
                false
            } else {
                true
            }
        }

        DefKind::AssocTy => {
            let assoc_item = tcx.associated_item(def_id);
            match assoc_item.container {
                ty::AssocItemContainer::ImplContainer => true,
                // Always encode RPITITs, since we need to be able to project
                // from an RPITIT associated item to an opaque when installing
                // the default projection predicates in default trait methods
                // with RPITITs.
                ty::AssocItemContainer::TraitContainer => {
                    assoc_item.defaultness(tcx).has_value() || assoc_item.is_impl_trait_in_trait()
                }
            }
        }
        DefKind::TyParam => {
            let hir::Node::GenericParam(param) = tcx.hir().get_by_def_id(def_id) else { bug!() };
            let hir::GenericParamKind::Type { default, .. } = param.kind else { bug!() };
            default.is_some()
        }

        DefKind::Trait
        | DefKind::TraitAlias
        | DefKind::Mod
        | DefKind::ForeignMod
        | DefKind::Macro(..)
        | DefKind::Use
        | DefKind::LifetimeParam
        | DefKind::GlobalAsm
        | DefKind::ExternCrate => false,
    }
}

fn should_encode_fn_sig(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Fn | DefKind::AssocFn | DefKind::Ctor(_, CtorKind::Fn) => true,

        DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::Field
        | DefKind::Const
        | DefKind::Static(..)
        | DefKind::Ctor(..)
        | DefKind::TyAlias
        | DefKind::OpaqueTy
        | DefKind::ForeignTy
        | DefKind::Impl { .. }
        | DefKind::AssocConst
        | DefKind::Closure
        | DefKind::Generator
        | DefKind::ConstParam
        | DefKind::AnonConst
        | DefKind::InlineConst
        | DefKind::AssocTy
        | DefKind::TyParam
        | DefKind::Trait
        | DefKind::TraitAlias
        | DefKind::Mod
        | DefKind::ForeignMod
        | DefKind::Macro(..)
        | DefKind::Use
        | DefKind::LifetimeParam
        | DefKind::GlobalAsm
        | DefKind::ExternCrate => false,
    }
}

fn should_encode_constness(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Fn
        | DefKind::AssocFn
        | DefKind::Closure
        | DefKind::Impl { of_trait: true }
        | DefKind::Variant
        | DefKind::Ctor(..) => true,

        DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Field
        | DefKind::Const
        | DefKind::AssocConst
        | DefKind::AnonConst
        | DefKind::Static(..)
        | DefKind::TyAlias
        | DefKind::OpaqueTy
        | DefKind::Impl { of_trait: false }
        | DefKind::ForeignTy
        | DefKind::Generator
        | DefKind::ConstParam
        | DefKind::InlineConst
        | DefKind::AssocTy
        | DefKind::TyParam
        | DefKind::Trait
        | DefKind::TraitAlias
        | DefKind::Mod
        | DefKind::ForeignMod
        | DefKind::Macro(..)
        | DefKind::Use
        | DefKind::LifetimeParam
        | DefKind::GlobalAsm
        | DefKind::ExternCrate => false,
    }
}

fn should_encode_const(def_kind: DefKind) -> bool {
    match def_kind {
        DefKind::Const | DefKind::AssocConst | DefKind::AnonConst | DefKind::InlineConst => true,

        DefKind::Struct
        | DefKind::Union
        | DefKind::Enum
        | DefKind::Variant
        | DefKind::Ctor(..)
        | DefKind::Field
        | DefKind::Fn
        | DefKind::Static(..)
        | DefKind::TyAlias
        | DefKind::OpaqueTy
        | DefKind::ForeignTy
        | DefKind::Impl { .. }
        | DefKind::AssocFn
        | DefKind::Closure
        | DefKind::Generator
        | DefKind::ConstParam
        | DefKind::AssocTy
        | DefKind::TyParam
        | DefKind::Trait
        | DefKind::TraitAlias
        | DefKind::Mod
        | DefKind::ForeignMod
        | DefKind::Macro(..)
        | DefKind::Use
        | DefKind::LifetimeParam
        | DefKind::GlobalAsm
        | DefKind::ExternCrate => false,
    }
}

fn should_encode_fn_impl_trait_in_trait<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> bool {
    if let Some(assoc_item) = tcx.opt_associated_item(def_id)
        && assoc_item.container == ty::AssocItemContainer::TraitContainer
        && assoc_item.kind == ty::AssocKind::Fn
    {
        true
    } else {
        false
    }
}

impl<'a, 'tcx> EncodeContext<'a, 'tcx> {
    fn encode_attrs(&mut self, def_id: LocalDefId) {
        let tcx = self.tcx;
        let mut state = AnalyzeAttrState {
            is_exported: tcx.effective_visibilities(()).is_exported(def_id),
            is_doc_hidden: false,
        };
        let attr_iter = tcx
            .opt_local_def_id_to_hir_id(def_id)
            .map_or(Default::default(), |hir_id| tcx.hir().attrs(hir_id))
            .iter()
            .filter(|attr| analyze_attr(attr, &mut state));

        record_array!(self.tables.attributes[def_id.to_def_id()] <- attr_iter);

        let mut attr_flags = AttrFlags::empty();
        if state.is_doc_hidden {
            attr_flags |= AttrFlags::IS_DOC_HIDDEN;
        }
        self.tables.attr_flags.set(def_id.local_def_index, attr_flags);
    }

    fn encode_def_ids(&mut self) {
        self.encode_info_for_mod(CRATE_DEF_ID);

        // Proc-macro crates only export proc-macro items, which are looked
        // up using `proc_macro_data`
        if self.is_proc_macro {
            return;
        }

        let tcx = self.tcx;

        for local_id in tcx.iter_local_def_id() {
            let def_id = local_id.to_def_id();
            let def_kind = tcx.opt_def_kind(local_id);
            let Some(def_kind) = def_kind else { continue };
            self.tables.opt_def_kind.set_some(def_id.index, def_kind);
            if should_encode_span(def_kind) {
                let def_span = tcx.def_span(local_id);
                record!(self.tables.def_span[def_id] <- def_span);
            }
            if should_encode_attrs(def_kind) {
                self.encode_attrs(local_id);
            }
            if should_encode_expn_that_defined(def_kind) {
                record!(self.tables.expn_that_defined[def_id] <- self.tcx.expn_that_defined(def_id));
            }
            if should_encode_span(def_kind) && let Some(ident_span) = tcx.def_ident_span(def_id) {
                record!(self.tables.def_ident_span[def_id] <- ident_span);
            }
            if def_kind.has_codegen_attrs() {
                record!(self.tables.codegen_fn_attrs[def_id] <- self.tcx.codegen_fn_attrs(def_id));
            }
            if should_encode_visibility(def_kind) {
                let vis =
                    self.tcx.local_visibility(local_id).map_id(|def_id| def_id.local_def_index);
                record!(self.tables.visibility[def_id] <- vis);
            }
            if should_encode_stability(def_kind) {
                self.encode_stability(def_id);
                self.encode_const_stability(def_id);
                self.encode_default_body_stability(def_id);
                self.encode_deprecation(def_id);
            }
            if should_encode_variances(def_kind) {
                let v = self.tcx.variances_of(def_id);
                record_array!(self.tables.variances_of[def_id] <- v);
            }
            if should_encode_fn_sig(def_kind) {
                record!(self.tables.fn_sig[def_id] <- tcx.fn_sig(def_id));
            }
            if should_encode_generics(def_kind) {
                let g = tcx.generics_of(def_id);
                record!(self.tables.generics_of[def_id] <- g);
                record!(self.tables.explicit_predicates_of[def_id] <- self.tcx.explicit_predicates_of(def_id));
                let inferred_outlives = self.tcx.inferred_outlives_of(def_id);
                record_defaulted_array!(self.tables.inferred_outlives_of[def_id] <- inferred_outlives);

                for param in &g.params {
                    if let ty::GenericParamDefKind::Const { has_default: true, .. } = param.kind {
                        let default = self.tcx.const_param_default(param.def_id);
                        record!(self.tables.const_param_default[param.def_id] <- default);
                    }
                }
            }
            if should_encode_type(tcx, local_id, def_kind) {
                record!(self.tables.type_of[def_id] <- self.tcx.type_of(def_id));
            }
            if should_encode_constness(def_kind) {
                self.tables.constness.set_some(def_id.index, self.tcx.constness(def_id));
            }
            if let DefKind::Fn | DefKind::AssocFn = def_kind {
                self.tables.asyncness.set_some(def_id.index, tcx.asyncness(def_id));
                record_array!(self.tables.fn_arg_names[def_id] <- tcx.fn_arg_names(def_id));
                self.tables.is_intrinsic.set(def_id.index, tcx.is_intrinsic(def_id));
            }
            if let DefKind::TyParam = def_kind {
                let default = self.tcx.object_lifetime_default(def_id);
                record!(self.tables.object_lifetime_default[def_id] <- default);
            }
            if let DefKind::Trait = def_kind {
                record!(self.tables.trait_def[def_id] <- self.tcx.trait_def(def_id));
                record!(self.tables.super_predicates_of[def_id] <- self.tcx.super_predicates_of(def_id));

                let module_children = self.tcx.module_children_local(local_id);
                record_array!(self.tables.module_children_non_reexports[def_id] <-
                    module_children.iter().map(|child| child.res.def_id().index));
            }
            if let DefKind::TraitAlias = def_kind {
                record!(self.tables.trait_def[def_id] <- self.tcx.trait_def(def_id));
                record!(self.tables.super_predicates_of[def_id] <- self.tcx.super_predicates_of(def_id));
                record!(self.tables.implied_predicates_of[def_id] <- self.tcx.implied_predicates_of(def_id));
            }
            if let DefKind::Trait | DefKind::Impl { .. } = def_kind {
                let associated_item_def_ids = self.tcx.associated_item_def_ids(def_id);
                record_array!(self.tables.associated_item_or_field_def_ids[def_id] <-
                    associated_item_def_ids.iter().map(|&def_id| {
                        assert!(def_id.is_local());
                        def_id.index
                    })
                );
                for &def_id in associated_item_def_ids {
                    self.encode_info_for_assoc_item(def_id);
                }
            }
            if let DefKind::Generator = def_kind {
                self.encode_info_for_generator(local_id);
            }
            if let DefKind::Enum | DefKind::Struct | DefKind::Union = def_kind {
                self.encode_info_for_adt(local_id);
            }
            if let DefKind::Mod = def_kind {
                self.encode_info_for_mod(local_id);
            }
            if let DefKind::Macro(_) = def_kind {
                self.encode_info_for_macro(local_id);
            }
            if let DefKind::OpaqueTy = def_kind {
                self.encode_explicit_item_bounds(def_id);
                self.tables
                    .is_type_alias_impl_trait
                    .set(def_id.index, self.tcx.is_type_alias_impl_trait(def_id));
            }
            if tcx.impl_method_has_trait_impl_trait_tys(def_id)
                && let Ok(table) = self.tcx.collect_return_position_impl_trait_in_trait_tys(def_id)
            {
                record!(self.tables.trait_impl_trait_tys[def_id] <- table);
            }
            if should_encode_fn_impl_trait_in_trait(tcx, def_id) {
                let table = tcx.associated_types_for_impl_traits_in_associated_fn(def_id);
                record_defaulted_array!(self.tables.associated_types_for_impl_traits_in_associated_fn[def_id] <- table);
            }
        }

        let inherent_impls = tcx.with_stable_hashing_context(|hcx| {
            tcx.crate_inherent_impls(()).inherent_impls.to_sorted(&hcx, true)
        });
        for (def_id, impls) in inherent_impls {
            record_defaulted_array!(self.tables.inherent_impls[def_id.to_def_id()] <- impls.iter().map(|def_id| {
                assert!(def_id.is_local());
                def_id.index
            }));
        }

        for (def_id, res_map) in &tcx.resolutions(()).doc_link_resolutions {
            record!(self.tables.doc_link_resolutions[def_id.to_def_id()] <- res_map);
        }

        for (def_id, traits) in &tcx.resolutions(()).doc_link_traits_in_scope {
            record_array!(self.tables.doc_link_traits_in_scope[def_id.to_def_id()] <- traits);
        }
    }

    #[instrument(level = "trace", skip(self))]
    fn encode_info_for_adt(&mut self, local_def_id: LocalDefId) {
        let def_id = local_def_id.to_def_id();
        let tcx = self.tcx;
        let adt_def = tcx.adt_def(def_id);
        record!(self.tables.repr_options[def_id] <- adt_def.repr());

        let params_in_repr = self.tcx.params_in_repr(def_id);
        record!(self.tables.params_in_repr[def_id] <- params_in_repr);

        if adt_def.is_enum() {
            let module_children = tcx.module_children_local(local_def_id);
            record_array!(self.tables.module_children_non_reexports[def_id] <-
                module_children.iter().map(|child| child.res.def_id().index));
        } else {
            // For non-enum, there is only one variant, and its def_id is the adt's.
            debug_assert_eq!(adt_def.variants().len(), 1);
            debug_assert_eq!(adt_def.non_enum_variant().def_id, def_id);
            // Therefore, the loop over variants will encode its fields as the adt's children.
        }

        for (idx, variant) in adt_def.variants().iter_enumerated() {
            let data = VariantData {
                discr: variant.discr,
                idx,
                ctor: variant.ctor.map(|(kind, def_id)| (kind, def_id.index)),
                is_non_exhaustive: variant.is_field_list_non_exhaustive(),
            };
            record!(self.tables.variant_data[variant.def_id] <- data);

            record_array!(self.tables.associated_item_or_field_def_ids[variant.def_id] <- variant.fields.iter().map(|f| {
                assert!(f.did.is_local());
                f.did.index
            }));

            if let Some((CtorKind::Fn, ctor_def_id)) = variant.ctor {
                let fn_sig = tcx.fn_sig(ctor_def_id);
                // FIXME only encode signature for ctor_def_id
                record!(self.tables.fn_sig[variant.def_id] <- fn_sig);
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn encode_info_for_mod(&mut self, local_def_id: LocalDefId) {
        let tcx = self.tcx;
        let def_id = local_def_id.to_def_id();

        // If we are encoding a proc-macro crates, `encode_info_for_mod` will
        // only ever get called for the crate root. We still want to encode
        // the crate root for consistency with other crates (some of the resolver
        // code uses it). However, we skip encoding anything relating to child
        // items - we encode information about proc-macros later on.
        if self.is_proc_macro {
            // Encode this here because we don't do it in encode_def_ids.
            record!(self.tables.expn_that_defined[def_id] <- tcx.expn_that_defined(local_def_id));
        } else {
            let module_children = tcx.module_children_local(local_def_id);

            record_array!(self.tables.module_children_non_reexports[def_id] <-
                module_children.iter().filter(|child| child.reexport_chain.is_empty())
                    .map(|child| child.res.def_id().index));

            record_defaulted_array!(self.tables.module_children_reexports[def_id] <-
                module_children.iter().filter(|child| !child.reexport_chain.is_empty()));
        }
    }

    fn encode_explicit_item_bounds(&mut self, def_id: DefId) {
        debug!("EncodeContext::encode_explicit_item_bounds({:?})", def_id);
        let bounds = self.tcx.explicit_item_bounds(def_id).skip_binder();
        record_defaulted_array!(self.tables.explicit_item_bounds[def_id] <- bounds);
    }

    #[instrument(level = "debug", skip(self))]
    fn encode_info_for_assoc_item(&mut self, def_id: DefId) {
        let tcx = self.tcx;
        let item = tcx.associated_item(def_id);

        self.tables.defaultness.set_some(def_id.index, item.defaultness(tcx));
        self.tables.assoc_container.set_some(def_id.index, item.container);

        match item.container {
            AssocItemContainer::TraitContainer => {
                if let ty::AssocKind::Type = item.kind {
                    self.encode_explicit_item_bounds(def_id);
                }
            }
            AssocItemContainer::ImplContainer => {
                if let Some(trait_item_def_id) = item.trait_item_def_id {
                    self.tables.trait_item_def_id.set_some(def_id.index, trait_item_def_id.into());
                }
            }
        }
        if let Some(rpitit_info) = item.opt_rpitit_info {
            record!(self.tables.opt_rpitit_info[def_id] <- rpitit_info);
        }
    }

    fn encode_mir(&mut self) {
        if self.is_proc_macro {
            return;
        }

        let tcx = self.tcx;

        let keys_and_jobs = tcx.mir_keys(()).iter().filter_map(|&def_id| {
            let (encode_const, encode_opt) = should_encode_mir(tcx, def_id);
            if encode_const || encode_opt { Some((def_id, encode_const, encode_opt)) } else { None }
        });
        for (def_id, encode_const, encode_opt) in keys_and_jobs {
            debug_assert!(encode_const || encode_opt);

            debug!("EntryBuilder::encode_mir({:?})", def_id);
            if encode_opt {
                record!(self.tables.optimized_mir[def_id.to_def_id()] <- tcx.optimized_mir(def_id));
                record!(self.tables.closure_saved_names_of_captured_variables[def_id.to_def_id()]
                    <- tcx.closure_saved_names_of_captured_variables(def_id));

                if tcx.sess.opts.unstable_opts.drop_tracking_mir
                    && let DefKind::Generator = self.tcx.def_kind(def_id)
                    && let Some(witnesses) = tcx.mir_generator_witnesses(def_id)
                {
                    record!(self.tables.mir_generator_witnesses[def_id.to_def_id()] <- witnesses);
                }
            }
            if encode_const {
                record!(self.tables.mir_for_ctfe[def_id.to_def_id()] <- tcx.mir_for_ctfe(def_id));

                // FIXME(generic_const_exprs): this feels wrong to have in `encode_mir`
                let abstract_const = tcx.thir_abstract_const(def_id);
                if let Ok(Some(abstract_const)) = abstract_const {
                    record!(self.tables.thir_abstract_const[def_id.to_def_id()] <- abstract_const);
                }

                if should_encode_const(tcx.def_kind(def_id)) {
                    let qualifs = tcx.mir_const_qualif(def_id);
                    record!(self.tables.mir_const_qualif[def_id.to_def_id()] <- qualifs);
                    let body_id = tcx.hir().maybe_body_owned_by(def_id);
                    if let Some(body_id) = body_id {
                        let const_data = self.encode_rendered_const_for_body(body_id);
                        record!(self.tables.rendered_const[def_id.to_def_id()] <- const_data);
                    }
                }
            }
            record!(self.tables.promoted_mir[def_id.to_def_id()] <- tcx.promoted_mir(def_id));

            let instance = ty::InstanceDef::Item(def_id.to_def_id());
            let unused = tcx.unused_generic_params(instance);
            self.tables.unused_generic_params.set(def_id.local_def_index, unused);
        }

        // Encode all the deduced parameter attributes for everything that has MIR, even for items
        // that can't be inlined. But don't if we aren't optimizing in non-incremental mode, to
        // save the query traffic.
        if tcx.sess.opts.output_types.should_codegen()
            && tcx.sess.opts.optimize != OptLevel::No
            && tcx.sess.opts.incremental.is_none()
        {
            for &local_def_id in tcx.mir_keys(()) {
                if let DefKind::AssocFn | DefKind::Fn = tcx.def_kind(local_def_id) {
                    record_array!(self.tables.deduced_param_attrs[local_def_id.to_def_id()] <-
                        self.tcx.deduced_param_attrs(local_def_id.to_def_id()));
                }
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn encode_stability(&mut self, def_id: DefId) {
        // The query lookup can take a measurable amount of time in crates with many items. Check if
        // the stability attributes are even enabled before using their queries.
        if self.feat.staged_api || self.tcx.sess.opts.unstable_opts.force_unstable_if_unmarked {
            if let Some(stab) = self.tcx.lookup_stability(def_id) {
                record!(self.tables.lookup_stability[def_id] <- stab)
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn encode_const_stability(&mut self, def_id: DefId) {
        // The query lookup can take a measurable amount of time in crates with many items. Check if
        // the stability attributes are even enabled before using their queries.
        if self.feat.staged_api || self.tcx.sess.opts.unstable_opts.force_unstable_if_unmarked {
            if let Some(stab) = self.tcx.lookup_const_stability(def_id) {
                record!(self.tables.lookup_const_stability[def_id] <- stab)
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn encode_default_body_stability(&mut self, def_id: DefId) {
        // The query lookup can take a measurable amount of time in crates with many items. Check if
        // the stability attributes are even enabled before using their queries.
        if self.feat.staged_api || self.tcx.sess.opts.unstable_opts.force_unstable_if_unmarked {
            if let Some(stab) = self.tcx.lookup_default_body_stability(def_id) {
                record!(self.tables.lookup_default_body_stability[def_id] <- stab)
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn encode_deprecation(&mut self, def_id: DefId) {
        if let Some(depr) = self.tcx.lookup_deprecation(def_id) {
            record!(self.tables.lookup_deprecation_entry[def_id] <- depr);
        }
    }

    fn encode_rendered_const_for_body(&mut self, body_id: hir::BodyId) -> String {
        let hir = self.tcx.hir();
        let body = hir.body(body_id);
        rustc_hir_pretty::to_string(&(&hir as &dyn intravisit::Map<'_>), |s| {
            s.print_expr(&body.value)
        })
    }

    #[instrument(level = "debug", skip(self))]
    fn encode_info_for_macro(&mut self, def_id: LocalDefId) {
        let tcx = self.tcx;

        let hir::ItemKind::Macro(ref macro_def, _) = tcx.hir().expect_item(def_id).kind else {
            bug!()
        };
        self.tables.is_macro_rules.set(def_id.local_def_index, macro_def.macro_rules);
        record!(self.tables.macro_definition[def_id.to_def_id()] <- &*macro_def.body);
    }

    #[instrument(level = "debug", skip(self))]
    fn encode_info_for_generator(&mut self, def_id: LocalDefId) {
        let typeck_result: &'tcx ty::TypeckResults<'tcx> = self.tcx.typeck(def_id);
        let data = self.tcx.generator_kind(def_id).unwrap();
        let generator_diagnostic_data = typeck_result.get_generator_diagnostic_data();
        record!(self.tables.generator_kind[def_id.to_def_id()] <- data);
        record!(self.tables.generator_diagnostic_data[def_id.to_def_id()]  <- generator_diagnostic_data);
    }

    fn encode_native_libraries(&mut self) -> LazyArray<NativeLib> {
        empty_proc_macro!(self);
        let used_libraries = self.tcx.native_libraries(LOCAL_CRATE);
        self.lazy_array(used_libraries.iter())
    }

    fn encode_foreign_modules(&mut self) -> LazyArray<ForeignModule> {
        empty_proc_macro!(self);
        let foreign_modules = self.tcx.foreign_modules(LOCAL_CRATE);
        self.lazy_array(foreign_modules.iter().map(|(_, m)| m).cloned())
    }

    fn encode_hygiene(&mut self) -> (SyntaxContextTable, ExpnDataTable, ExpnHashTable) {
        let mut syntax_contexts: TableBuilder<_, _> = Default::default();
        let mut expn_data_table: TableBuilder<_, _> = Default::default();
        let mut expn_hash_table: TableBuilder<_, _> = Default::default();

        self.hygiene_ctxt.encode(
            &mut (&mut *self, &mut syntax_contexts, &mut expn_data_table, &mut expn_hash_table),
            |(this, syntax_contexts, _, _), index, ctxt_data| {
                syntax_contexts.set_some(index, this.lazy(ctxt_data));
            },
            |(this, _, expn_data_table, expn_hash_table), index, expn_data, hash| {
                if let Some(index) = index.as_local() {
                    expn_data_table.set_some(index.as_raw(), this.lazy(expn_data));
                    expn_hash_table.set_some(index.as_raw(), this.lazy(hash));
                }
            },
        );

        (
            syntax_contexts.encode(&mut self.opaque),
            expn_data_table.encode(&mut self.opaque),
            expn_hash_table.encode(&mut self.opaque),
        )
    }

    fn encode_proc_macros(&mut self) -> Option<ProcMacroData> {
        let is_proc_macro = self.tcx.sess.crate_types().contains(&CrateType::ProcMacro);
        if is_proc_macro {
            let tcx = self.tcx;
            let hir = tcx.hir();

            let proc_macro_decls_static = tcx.proc_macro_decls_static(()).unwrap().local_def_index;
            let stability = tcx.lookup_stability(CRATE_DEF_ID);
            let macros =
                self.lazy_array(tcx.resolutions(()).proc_macros.iter().map(|p| p.local_def_index));
            for (i, span) in self.tcx.sess.parse_sess.proc_macro_quoted_spans() {
                let span = self.lazy(span);
                self.tables.proc_macro_quoted_spans.set_some(i, span);
            }

            self.tables.opt_def_kind.set_some(LOCAL_CRATE.as_def_id().index, DefKind::Mod);
            record!(self.tables.def_span[LOCAL_CRATE.as_def_id()] <- tcx.def_span(LOCAL_CRATE.as_def_id()));
            self.encode_attrs(LOCAL_CRATE.as_def_id().expect_local());
            let vis = tcx.local_visibility(CRATE_DEF_ID).map_id(|def_id| def_id.local_def_index);
            record!(self.tables.visibility[LOCAL_CRATE.as_def_id()] <- vis);
            if let Some(stability) = stability {
                record!(self.tables.lookup_stability[LOCAL_CRATE.as_def_id()] <- stability);
            }
            self.encode_deprecation(LOCAL_CRATE.as_def_id());
            if let Some(res_map) = tcx.resolutions(()).doc_link_resolutions.get(&CRATE_DEF_ID) {
                record!(self.tables.doc_link_resolutions[LOCAL_CRATE.as_def_id()] <- res_map);
            }
            if let Some(traits) = tcx.resolutions(()).doc_link_traits_in_scope.get(&CRATE_DEF_ID) {
                record_array!(self.tables.doc_link_traits_in_scope[LOCAL_CRATE.as_def_id()] <- traits);
            }

            // Normally, this information is encoded when we walk the items
            // defined in this crate. However, we skip doing that for proc-macro crates,
            // so we manually encode just the information that we need
            for &proc_macro in &tcx.resolutions(()).proc_macros {
                let id = proc_macro;
                let proc_macro = hir.local_def_id_to_hir_id(proc_macro);
                let mut name = hir.name(proc_macro);
                let span = hir.span(proc_macro);
                // Proc-macros may have attributes like `#[allow_internal_unstable]`,
                // so downstream crates need access to them.
                let attrs = hir.attrs(proc_macro);
                let macro_kind = if attr::contains_name(attrs, sym::proc_macro) {
                    MacroKind::Bang
                } else if attr::contains_name(attrs, sym::proc_macro_attribute) {
                    MacroKind::Attr
                } else if let Some(attr) = attr::find_by_name(attrs, sym::proc_macro_derive) {
                    // This unwrap chain should have been checked by the proc-macro harness.
                    name = attr.meta_item_list().unwrap()[0]
                        .meta_item()
                        .unwrap()
                        .ident()
                        .unwrap()
                        .name;
                    MacroKind::Derive
                } else {
                    bug!("Unknown proc-macro type for item {:?}", id);
                };

                let mut def_key = self.tcx.hir().def_key(id);
                def_key.disambiguated_data.data = DefPathData::MacroNs(name);

                let def_id = id.to_def_id();
                self.tables.opt_def_kind.set_some(def_id.index, DefKind::Macro(macro_kind));
                self.tables.proc_macro.set_some(def_id.index, macro_kind);
                self.encode_attrs(id);
                record!(self.tables.def_keys[def_id] <- def_key);
                record!(self.tables.def_ident_span[def_id] <- span);
                record!(self.tables.def_span[def_id] <- span);
                record!(self.tables.visibility[def_id] <- ty::Visibility::Public);
                if let Some(stability) = stability {
                    record!(self.tables.lookup_stability[def_id] <- stability);
                }
            }

            Some(ProcMacroData { proc_macro_decls_static, stability, macros })
        } else {
            None
        }
    }

    fn encode_debugger_visualizers(&mut self) -> LazyArray<DebuggerVisualizerFile> {
        empty_proc_macro!(self);
        self.lazy_array(
            self.tcx
                .debugger_visualizers(LOCAL_CRATE)
                .iter()
                // Erase the path since it may contain privacy sensitive data
                // that we don't want to end up in crate metadata.
                // The path is only needed for the local crate because of
                // `--emit dep-info`.
                .map(DebuggerVisualizerFile::path_erased),
        )
    }

    fn encode_crate_deps(&mut self) -> LazyArray<CrateDep> {
        empty_proc_macro!(self);

        let deps = self
            .tcx
            .crates(())
            .iter()
            .map(|&cnum| {
                let dep = CrateDep {
                    name: self.tcx.crate_name(cnum),
                    hash: self.tcx.crate_hash(cnum),
                    host_hash: self.tcx.crate_host_hash(cnum),
                    kind: self.tcx.dep_kind(cnum),
                    extra_filename: self.tcx.extra_filename(cnum).clone(),
                    is_private: self.tcx.is_private_dep(cnum),
                };
                (cnum, dep)
            })
            .collect::<Vec<_>>();

        {
            // Sanity-check the crate numbers
            let mut expected_cnum = 1;
            for &(n, _) in &deps {
                assert_eq!(n, CrateNum::new(expected_cnum));
                expected_cnum += 1;
            }
        }

        // We're just going to write a list of crate 'name-hash-version's, with
        // the assumption that they are numbered 1 to n.
        // FIXME (#2166): This is not nearly enough to support correct versioning
        // but is enough to get transitive crate dependencies working.
        self.lazy_array(deps.iter().map(|(_, dep)| dep))
    }

    fn encode_lib_features(&mut self) -> LazyArray<(Symbol, Option<Symbol>)> {
        empty_proc_macro!(self);
        let tcx = self.tcx;
        let lib_features = tcx.lib_features(());
        self.lazy_array(lib_features.to_vec())
    }

    fn encode_stability_implications(&mut self) -> LazyArray<(Symbol, Symbol)> {
        empty_proc_macro!(self);
        let tcx = self.tcx;
        let implications = tcx.stability_implications(LOCAL_CRATE);
        self.lazy_array(implications.iter().map(|(k, v)| (*k, *v)))
    }

    fn encode_diagnostic_items(&mut self) -> LazyArray<(Symbol, DefIndex)> {
        empty_proc_macro!(self);
        let tcx = self.tcx;
        let diagnostic_items = &tcx.diagnostic_items(LOCAL_CRATE).name_to_id;
        self.lazy_array(diagnostic_items.iter().map(|(&name, def_id)| (name, def_id.index)))
    }

    fn encode_lang_items(&mut self) -> LazyArray<(DefIndex, LangItem)> {
        empty_proc_macro!(self);
        let lang_items = self.tcx.lang_items().iter();
        self.lazy_array(lang_items.filter_map(|(lang_item, def_id)| {
            def_id.as_local().map(|id| (id.local_def_index, lang_item))
        }))
    }

    fn encode_lang_items_missing(&mut self) -> LazyArray<LangItem> {
        empty_proc_macro!(self);
        let tcx = self.tcx;
        self.lazy_array(&tcx.lang_items().missing)
    }

    fn encode_stripped_cfg_items(&mut self) -> LazyArray<StrippedCfgItem<DefIndex>> {
        self.lazy_array(
            self.tcx
                .stripped_cfg_items(LOCAL_CRATE)
                .into_iter()
                .map(|item| item.clone().map_mod_id(|def_id| def_id.index)),
        )
    }

    fn encode_traits(&mut self) -> LazyArray<DefIndex> {
        empty_proc_macro!(self);
        self.lazy_array(self.tcx.traits(LOCAL_CRATE).iter().map(|def_id| def_id.index))
    }

    /// Encodes an index, mapping each trait to its (local) implementations.
    #[instrument(level = "debug", skip(self))]
    fn encode_impls(&mut self) -> LazyArray<TraitImpls> {
        empty_proc_macro!(self);
        let tcx = self.tcx;
        let mut fx_hash_map: FxHashMap<DefId, Vec<(DefIndex, Option<SimplifiedType>)>> =
            FxHashMap::default();

        for id in tcx.hir().items() {
            let DefKind::Impl { of_trait } = tcx.def_kind(id.owner_id) else {
                continue;
            };
            let def_id = id.owner_id.to_def_id();

            self.tables.defaultness.set_some(def_id.index, tcx.defaultness(def_id));
            self.tables.impl_polarity.set_some(def_id.index, tcx.impl_polarity(def_id));

            if of_trait && let Some(trait_ref) = tcx.impl_trait_ref(def_id) {
                record!(self.tables.impl_trait_ref[def_id] <- trait_ref);

                let trait_ref = trait_ref.instantiate_identity();
                let simplified_self_ty =
                    fast_reject::simplify_type(self.tcx, trait_ref.self_ty(), TreatParams::AsCandidateKey);
                fx_hash_map
                    .entry(trait_ref.def_id)
                    .or_default()
                    .push((id.owner_id.def_id.local_def_index, simplified_self_ty));

                let trait_def = tcx.trait_def(trait_ref.def_id);
                if let Some(mut an) = trait_def.ancestors(tcx, def_id).ok() {
                    if let Some(specialization_graph::Node::Impl(parent)) = an.nth(1) {
                        self.tables.impl_parent.set_some(def_id.index, parent.into());
                    }
                }

                // if this is an impl of `CoerceUnsized`, create its
                // "unsized info", else just store None
                if Some(trait_ref.def_id) == tcx.lang_items().coerce_unsized_trait() {
                    let coerce_unsized_info = tcx.coerce_unsized_info(def_id);
                    record!(self.tables.coerce_unsized_info[def_id] <- coerce_unsized_info);
                }
            }
        }

        let mut all_impls: Vec<_> = fx_hash_map.into_iter().collect();

        // Bring everything into deterministic order for hashing
        all_impls.sort_by_cached_key(|&(trait_def_id, _)| tcx.def_path_hash(trait_def_id));

        let all_impls: Vec<_> = all_impls
            .into_iter()
            .map(|(trait_def_id, mut impls)| {
                // Bring everything into deterministic order for hashing
                impls.sort_by_cached_key(|&(index, _)| {
                    tcx.hir().def_path_hash(LocalDefId { local_def_index: index })
                });

                TraitImpls {
                    trait_id: (trait_def_id.krate.as_u32(), trait_def_id.index),
                    impls: self.lazy_array(&impls),
                }
            })
            .collect();

        self.lazy_array(&all_impls)
    }

    #[instrument(level = "debug", skip(self))]
    fn encode_incoherent_impls(&mut self) -> LazyArray<IncoherentImpls> {
        empty_proc_macro!(self);
        let tcx = self.tcx;
        let mut all_impls: Vec<_> = tcx.crate_inherent_impls(()).incoherent_impls.iter().collect();
        tcx.with_stable_hashing_context(|mut ctx| {
            all_impls.sort_by_cached_key(|&(&simp, _)| {
                let mut hasher = StableHasher::new();
                simp.hash_stable(&mut ctx, &mut hasher);
                hasher.finish::<Fingerprint>()
            })
        });
        let all_impls: Vec<_> = all_impls
            .into_iter()
            .map(|(&simp, impls)| {
                let mut impls: Vec<_> =
                    impls.into_iter().map(|def_id| def_id.local_def_index).collect();
                impls.sort_by_cached_key(|&local_def_index| {
                    tcx.hir().def_path_hash(LocalDefId { local_def_index })
                });

                IncoherentImpls { self_ty: simp, impls: self.lazy_array(impls) }
            })
            .collect();

        self.lazy_array(&all_impls)
    }

    // Encodes all symbols exported from this crate into the metadata.
    //
    // This pass is seeded off the reachability list calculated in the
    // middle::reachable module but filters out items that either don't have a
    // symbol associated with them (they weren't translated) or if they're an FFI
    // definition (as that's not defined in this crate).
    fn encode_exported_symbols(
        &mut self,
        exported_symbols: &[(ExportedSymbol<'tcx>, SymbolExportInfo)],
    ) -> LazyArray<(ExportedSymbol<'static>, SymbolExportInfo)> {
        empty_proc_macro!(self);
        // The metadata symbol name is special. It should not show up in
        // downstream crates.
        let metadata_symbol_name = SymbolName::new(self.tcx, &metadata_symbol_name(self.tcx));

        self.lazy_array(
            exported_symbols
                .iter()
                .filter(|&(exported_symbol, _)| match *exported_symbol {
                    ExportedSymbol::NoDefId(symbol_name) => symbol_name != metadata_symbol_name,
                    _ => true,
                })
                .cloned(),
        )
    }

    fn encode_dylib_dependency_formats(&mut self) -> LazyArray<Option<LinkagePreference>> {
        empty_proc_macro!(self);
        let formats = self.tcx.dependency_formats(());
        for (ty, arr) in formats.iter() {
            if *ty != CrateType::Dylib {
                continue;
            }
            return self.lazy_array(arr.iter().map(|slot| match *slot {
                Linkage::NotLinked | Linkage::IncludedFromDylib => None,

                Linkage::Dynamic => Some(LinkagePreference::RequireDynamic),
                Linkage::Static => Some(LinkagePreference::RequireStatic),
            }));
        }
        LazyArray::default()
    }
}

/// Used to prefetch queries which will be needed later by metadata encoding.
/// Only a subset of the queries are actually prefetched to keep this code smaller.
fn prefetch_mir(tcx: TyCtxt<'_>) {
    if !tcx.sess.opts.output_types.should_codegen() {
        // We won't emit MIR, so don't prefetch it.
        return;
    }

    par_for_each_in(tcx.mir_keys(()), |&def_id| {
        let (encode_const, encode_opt) = should_encode_mir(tcx, def_id);

        if encode_const {
            tcx.ensure_with_value().mir_for_ctfe(def_id);
        }
        if encode_opt {
            tcx.ensure_with_value().optimized_mir(def_id);
        }
        if encode_opt || encode_const {
            tcx.ensure_with_value().promoted_mir(def_id);
        }
    })
}

// NOTE(eddyb) The following comment was preserved for posterity, even
// though it's no longer relevant as EBML (which uses nested & tagged
// "documents") was replaced with a scheme that can't go out of bounds.
//
// And here we run into yet another obscure archive bug: in which metadata
// loaded from archives may have trailing garbage bytes. Awhile back one of
// our tests was failing sporadically on the macOS 64-bit builders (both nopt
// and opt) by having ebml generate an out-of-bounds panic when looking at
// metadata.
//
// Upon investigation it turned out that the metadata file inside of an rlib
// (and ar archive) was being corrupted. Some compilations would generate a
// metadata file which would end in a few extra bytes, while other
// compilations would not have these extra bytes appended to the end. These
// extra bytes were interpreted by ebml as an extra tag, so they ended up
// being interpreted causing the out-of-bounds.
//
// The root cause of why these extra bytes were appearing was never
// discovered, and in the meantime the solution we're employing is to insert
// the length of the metadata to the start of the metadata. Later on this
// will allow us to slice the metadata to the precise length that we just
// generated regardless of trailing bytes that end up in it.

pub struct EncodedMetadata {
    // The declaration order matters because `mmap` should be dropped before `_temp_dir`.
    mmap: Option<Mmap>,
    // We need to carry MaybeTempDir to avoid deleting the temporary
    // directory while accessing the Mmap.
    _temp_dir: Option<MaybeTempDir>,
}

impl EncodedMetadata {
    #[inline]
    pub fn from_path(path: PathBuf, temp_dir: Option<MaybeTempDir>) -> std::io::Result<Self> {
        let file = std::fs::File::open(&path)?;
        let file_metadata = file.metadata()?;
        if file_metadata.len() == 0 {
            return Ok(Self { mmap: None, _temp_dir: None });
        }
        let mmap = unsafe { Some(Mmap::map(file)?) };
        Ok(Self { mmap, _temp_dir: temp_dir })
    }

    #[inline]
    pub fn raw_data(&self) -> &[u8] {
        self.mmap.as_deref().unwrap_or_default()
    }
}

impl<S: Encoder> Encodable<S> for EncodedMetadata {
    fn encode(&self, s: &mut S) {
        let slice = self.raw_data();
        slice.encode(s)
    }
}

impl<D: Decoder> Decodable<D> for EncodedMetadata {
    fn decode(d: &mut D) -> Self {
        let len = d.read_usize();
        let mmap = if len > 0 {
            let mut mmap = MmapMut::map_anon(len).unwrap();
            for _ in 0..len {
                (&mut mmap[..]).write(&[d.read_u8()]).unwrap();
            }
            mmap.flush().unwrap();
            Some(mmap.make_read_only().unwrap())
        } else {
            None
        };

        Self { mmap, _temp_dir: None }
    }
}

pub fn encode_metadata(tcx: TyCtxt<'_>, path: &Path) {
    let _prof_timer = tcx.prof.verbose_generic_activity("generate_crate_metadata");

    // Since encoding metadata is not in a query, and nothing is cached,
    // there's no need to do dep-graph tracking for any of it.
    tcx.dep_graph.assert_ignored();

    join(
        || encode_metadata_impl(tcx, path),
        || {
            if tcx.sess.threads() == 1 {
                return;
            }
            // Prefetch some queries used by metadata encoding.
            // This is not necessary for correctness, but is only done for performance reasons.
            // It can be removed if it turns out to cause trouble or be detrimental to performance.
            join(|| prefetch_mir(tcx), || tcx.exported_symbols(LOCAL_CRATE));
        },
    );
}

fn encode_metadata_impl(tcx: TyCtxt<'_>, path: &Path) {
    let mut encoder = opaque::FileEncoder::new(path)
        .unwrap_or_else(|err| tcx.sess.emit_fatal(FailCreateFileEncoder { err }));
    encoder.emit_raw_bytes(METADATA_HEADER);

    // Will be filled with the root position after encoding everything.
    encoder.emit_raw_bytes(&[0, 0, 0, 0]);

    let source_map_files = tcx.sess.source_map().files();
    let source_file_cache = (source_map_files[0].clone(), 0);
    let required_source_files = Some(FxIndexSet::default());
    drop(source_map_files);

    let hygiene_ctxt = HygieneEncodeContext::default();

    let mut ecx = EncodeContext {
        opaque: encoder,
        tcx,
        feat: tcx.features(),
        tables: Default::default(),
        lazy_state: LazyState::NoNode,
        span_shorthands: Default::default(),
        type_shorthands: Default::default(),
        predicate_shorthands: Default::default(),
        source_file_cache,
        interpret_allocs: Default::default(),
        required_source_files,
        is_proc_macro: tcx.sess.crate_types().contains(&CrateType::ProcMacro),
        hygiene_ctxt: &hygiene_ctxt,
        symbol_table: Default::default(),
    };

    // Encode the rustc version string in a predictable location.
    rustc_version(tcx.sess.cfg_version).encode(&mut ecx);

    // Encode all the entries and extra information in the crate,
    // culminating in the `CrateRoot` which points to all of it.
    let root = ecx.encode_crate_root();

    ecx.opaque.flush();

    let mut file = ecx.opaque.file();
    // We will return to this position after writing the root position.
    let pos_before_seek = file.stream_position().unwrap();

    // Encode the root position.
    let header = METADATA_HEADER.len();
    file.seek(std::io::SeekFrom::Start(header as u64))
        .unwrap_or_else(|err| tcx.sess.emit_fatal(FailSeekFile { err }));
    let pos = root.position.get();
    file.write_all(&[(pos >> 24) as u8, (pos >> 16) as u8, (pos >> 8) as u8, (pos >> 0) as u8])
        .unwrap_or_else(|err| tcx.sess.emit_fatal(FailWriteFile { err }));

    // Return to the position where we are before writing the root position.
    file.seek(std::io::SeekFrom::Start(pos_before_seek)).unwrap();

    // Record metadata size for self-profiling
    tcx.prof.artifact_size(
        "crate_metadata",
        "crate_metadata",
        file.metadata().unwrap().len() as u64,
    );
}

pub fn provide(providers: &mut Providers) {
    *providers = Providers {
        doc_link_resolutions: |tcx, def_id| {
            tcx.resolutions(())
                .doc_link_resolutions
                .get(&def_id)
                .unwrap_or_else(|| span_bug!(tcx.def_span(def_id), "no resolutions for a doc link"))
        },
        doc_link_traits_in_scope: |tcx, def_id| {
            tcx.resolutions(()).doc_link_traits_in_scope.get(&def_id).unwrap_or_else(|| {
                span_bug!(tcx.def_span(def_id), "no traits in scope for a doc link")
            })
        },
        traits: |tcx, LocalCrate| {
            let mut traits = Vec::new();
            for id in tcx.hir().items() {
                if matches!(tcx.def_kind(id.owner_id), DefKind::Trait | DefKind::TraitAlias) {
                    traits.push(id.owner_id.to_def_id())
                }
            }

            // Bring everything into deterministic order.
            traits.sort_by_cached_key(|&def_id| tcx.def_path_hash(def_id));
            tcx.arena.alloc_slice(&traits)
        },
        trait_impls_in_crate: |tcx, LocalCrate| {
            let mut trait_impls = Vec::new();
            for id in tcx.hir().items() {
                if matches!(tcx.def_kind(id.owner_id), DefKind::Impl { .. })
                    && tcx.impl_trait_ref(id.owner_id).is_some()
                {
                    trait_impls.push(id.owner_id.to_def_id())
                }
            }

            // Bring everything into deterministic order.
            trait_impls.sort_by_cached_key(|&def_id| tcx.def_path_hash(def_id));
            tcx.arena.alloc_slice(&trait_impls)
        },

        ..*providers
    }
}
