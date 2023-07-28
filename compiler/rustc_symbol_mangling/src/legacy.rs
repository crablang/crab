use rustc_data_structures::stable_hasher::{Hash64, HashStable, StableHasher};
use rustc_hir::def_id::CrateNum;
use rustc_hir::definitions::{DefPathData, DisambiguatedDefPathData};
use rustc_middle::ty::print::{PrettyPrinter, Print, Printer};
use rustc_middle::ty::{self, Instance, Ty, TyCtxt, TypeVisitableExt};
use rustc_middle::ty::{GenericArg, GenericArgKind};
use rustc_middle::util::common::record_time;

use std::fmt::{self, Write};
use std::mem::{self, discriminant};

pub(super) fn mangle<'tcx>(
    tcx: TyCtxt<'tcx>,
    instance: Instance<'tcx>,
    instantiating_crate: Option<CrateNum>,
) -> String {
    let def_id = instance.def_id();

    // We want to compute the "type" of this item. Unfortunately, some
    // kinds of items (e.g., closures) don't have an entry in the
    // item-type array. So walk back up the find the closest parent
    // that DOES have an entry.
    let mut ty_def_id = def_id;
    let instance_ty;
    loop {
        let key = tcx.def_key(ty_def_id);
        match key.disambiguated_data.data {
            DefPathData::TypeNs(_) | DefPathData::ValueNs(_) => {
                instance_ty = tcx.type_of(ty_def_id).instantiate_identity();
                debug!(?instance_ty);
                break;
            }
            _ => {
                // if we're making a symbol for something, there ought
                // to be a value or type-def or something in there
                // *somewhere*
                ty_def_id.index = key.parent.unwrap_or_else(|| {
                    bug!(
                        "finding type for {:?}, encountered def-id {:?} with no \
                         parent",
                        def_id,
                        ty_def_id
                    );
                });
            }
        }
    }

    // Erase regions because they may not be deterministic when hashed
    // and should not matter anyhow.
    let instance_ty = tcx.erase_regions(instance_ty);

    let hash = get_symbol_hash(tcx, instance, instance_ty, instantiating_crate);

    let mut printer = SymbolPrinter { tcx, path: SymbolPath::new(), keep_within_component: false };
    printer
        .print_def_path(
            def_id,
            if let ty::InstanceDef::DropGlue(_, _) = instance.def {
                // Add the name of the dropped type to the symbol name
                &*instance.args
            } else {
                &[]
            },
        )
        .unwrap();

    if let ty::InstanceDef::ThreadLocalShim(..) = instance.def {
        let _ = printer.write_str("{{tls-shim}}");
    }

    if let ty::InstanceDef::VTableShim(..) = instance.def {
        let _ = printer.write_str("{{vtable-shim}}");
    }

    if let ty::InstanceDef::ReifyShim(..) = instance.def {
        let _ = printer.write_str("{{reify-shim}}");
    }

    printer.path.finish(hash)
}

fn get_symbol_hash<'tcx>(
    tcx: TyCtxt<'tcx>,

    // instance this name will be for
    instance: Instance<'tcx>,

    // type of the item, without any generic
    // parameters substituted; this is
    // included in the hash as a kind of
    // safeguard.
    item_type: Ty<'tcx>,

    instantiating_crate: Option<CrateNum>,
) -> Hash64 {
    let def_id = instance.def_id();
    let args = instance.args;
    debug!("get_symbol_hash(def_id={:?}, parameters={:?})", def_id, args);

    tcx.with_stable_hashing_context(|mut hcx| {
        let mut hasher = StableHasher::new();

        record_time(&tcx.sess.perf_stats.symbol_hash_time, || {
            // the main symbol name is not necessarily unique; hash in the
            // compiler's internal def-path, guaranteeing each symbol has a
            // truly unique path
            tcx.def_path_hash(def_id).hash_stable(&mut hcx, &mut hasher);

            // Include the main item-type. Note that, in this case, the
            // assertions about `has_param` may not hold, but this item-type
            // ought to be the same for every reference anyway.
            assert!(!item_type.has_erasable_regions());
            hcx.while_hashing_spans(false, |hcx| {
                item_type.hash_stable(hcx, &mut hasher);

                // If this is a function, we hash the signature as well.
                // This is not *strictly* needed, but it may help in some
                // situations, see the `run-make/a-b-a-linker-guard` test.
                if let ty::FnDef(..) = item_type.kind() {
                    item_type.fn_sig(tcx).hash_stable(hcx, &mut hasher);
                }

                // also include any type parameters (for generic items)
                args.hash_stable(hcx, &mut hasher);

                if let Some(instantiating_crate) = instantiating_crate {
                    tcx.def_path_hash(instantiating_crate.as_def_id())
                        .stable_crate_id()
                        .hash_stable(hcx, &mut hasher);
                }

                // We want to avoid accidental collision between different types of instances.
                // Especially, `VTableShim`s and `ReifyShim`s may overlap with their original
                // instances without this.
                discriminant(&instance.def).hash_stable(hcx, &mut hasher);
            });
        });

        // 64 bits should be enough to avoid collisions.
        hasher.finish::<Hash64>()
    })
}

// Follow C++ namespace-mangling style, see
// https://en.wikipedia.org/wiki/Name_mangling for more info.
//
// It turns out that on macOS you can actually have arbitrary symbols in
// function names (at least when given to LLVM), but this is not possible
// when using unix's linker. Perhaps one day when we just use a linker from LLVM
// we won't need to do this name mangling. The problem with name mangling is
// that it seriously limits the available characters. For example we can't
// have things like &T in symbol names when one would theoretically
// want them for things like impls of traits on that type.
//
// To be able to work on all platforms and get *some* reasonable output, we
// use C++ name-mangling.
#[derive(Debug)]
struct SymbolPath {
    result: String,
    temp_buf: String,
}

impl SymbolPath {
    fn new() -> Self {
        let mut result =
            SymbolPath { result: String::with_capacity(64), temp_buf: String::with_capacity(16) };
        result.result.push_str("_ZN"); // _Z == Begin name-sequence, N == nested
        result
    }

    fn finalize_pending_component(&mut self) {
        if !self.temp_buf.is_empty() {
            let _ = write!(self.result, "{}{}", self.temp_buf.len(), self.temp_buf);
            self.temp_buf.clear();
        }
    }

    fn finish(mut self, hash: Hash64) -> String {
        self.finalize_pending_component();
        // E = end name-sequence
        let _ = write!(self.result, "17h{hash:016x}E");
        self.result
    }
}

struct SymbolPrinter<'tcx> {
    tcx: TyCtxt<'tcx>,
    path: SymbolPath,

    // When `true`, `finalize_pending_component` isn't used.
    // This is needed when recursing into `path_qualified`,
    // or `path_generic_args`, as any nested paths are
    // logically within one component.
    keep_within_component: bool,
}

// HACK(eddyb) this relies on using the `fmt` interface to get
// `PrettyPrinter` aka pretty printing of e.g. types in paths,
// symbol names should have their own printing machinery.

impl<'tcx> Printer<'tcx> for &mut SymbolPrinter<'tcx> {
    type Error = fmt::Error;

    type Path = Self;
    type Region = Self;
    type Type = Self;
    type DynExistential = Self;
    type Const = Self;

    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn print_region(self, _region: ty::Region<'_>) -> Result<Self::Region, Self::Error> {
        Ok(self)
    }

    fn print_type(mut self, ty: Ty<'tcx>) -> Result<Self::Type, Self::Error> {
        match *ty.kind() {
            // Print all nominal types as paths (unlike `pretty_print_type`).
            ty::FnDef(def_id, args)
            | ty::Alias(ty::Projection | ty::Opaque, ty::AliasTy { def_id, args, .. })
            | ty::Closure(def_id, args)
            | ty::Generator(def_id, args, _) => self.print_def_path(def_id, args),

            // The `pretty_print_type` formatting of array size depends on
            // -Zverbose flag, so we cannot reuse it here.
            ty::Array(ty, size) => {
                self.write_str("[")?;
                self = self.print_type(ty)?;
                self.write_str("; ")?;
                if let Some(size) = size.try_to_bits(self.tcx().data_layout.pointer_size) {
                    write!(self, "{size}")?
                } else if let ty::ConstKind::Param(param) = size.kind() {
                    self = param.print(self)?
                } else {
                    self.write_str("_")?
                }
                self.write_str("]")?;
                Ok(self)
            }

            ty::Alias(ty::Inherent, _) => panic!("unexpected inherent projection"),

            _ => self.pretty_print_type(ty),
        }
    }

    fn print_dyn_existential(
        mut self,
        predicates: &'tcx ty::List<ty::PolyExistentialPredicate<'tcx>>,
    ) -> Result<Self::DynExistential, Self::Error> {
        let mut first = true;
        for p in predicates {
            if !first {
                write!(self, "+")?;
            }
            first = false;
            self = p.print(self)?;
        }
        Ok(self)
    }

    fn print_const(self, ct: ty::Const<'tcx>) -> Result<Self::Const, Self::Error> {
        // only print integers
        match (ct.kind(), ct.ty().kind()) {
            (ty::ConstKind::Value(ty::ValTree::Leaf(scalar)), ty::Int(_) | ty::Uint(_)) => {
                // The `pretty_print_const` formatting depends on -Zverbose
                // flag, so we cannot reuse it here.
                let signed = matches!(ct.ty().kind(), ty::Int(_));
                write!(
                    self,
                    "{:#?}",
                    ty::ConstInt::new(scalar, signed, ct.ty().is_ptr_sized_integral())
                )?;
            }
            _ => self.write_str("_")?,
        }
        Ok(self)
    }

    fn path_crate(self, cnum: CrateNum) -> Result<Self::Path, Self::Error> {
        self.write_str(self.tcx.crate_name(cnum).as_str())?;
        Ok(self)
    }
    fn path_qualified(
        self,
        self_ty: Ty<'tcx>,
        trait_ref: Option<ty::TraitRef<'tcx>>,
    ) -> Result<Self::Path, Self::Error> {
        // Similar to `pretty_path_qualified`, but for the other
        // types that are printed as paths (see `print_type` above).
        match self_ty.kind() {
            ty::FnDef(..) | ty::Alias(..) | ty::Closure(..) | ty::Generator(..)
                if trait_ref.is_none() =>
            {
                self.print_type(self_ty)
            }

            _ => self.pretty_path_qualified(self_ty, trait_ref),
        }
    }

    fn path_append_impl(
        self,
        print_prefix: impl FnOnce(Self) -> Result<Self::Path, Self::Error>,
        _disambiguated_data: &DisambiguatedDefPathData,
        self_ty: Ty<'tcx>,
        trait_ref: Option<ty::TraitRef<'tcx>>,
    ) -> Result<Self::Path, Self::Error> {
        self.pretty_path_append_impl(
            |mut cx| {
                cx = print_prefix(cx)?;

                if cx.keep_within_component {
                    // HACK(eddyb) print the path similarly to how `FmtPrinter` prints it.
                    cx.write_str("::")?;
                } else {
                    cx.path.finalize_pending_component();
                }

                Ok(cx)
            },
            self_ty,
            trait_ref,
        )
    }
    fn path_append(
        mut self,
        print_prefix: impl FnOnce(Self) -> Result<Self::Path, Self::Error>,
        disambiguated_data: &DisambiguatedDefPathData,
    ) -> Result<Self::Path, Self::Error> {
        self = print_prefix(self)?;

        // Skip `::{{extern}}` blocks and `::{{constructor}}` on tuple/unit structs.
        if let DefPathData::ForeignMod | DefPathData::Ctor = disambiguated_data.data {
            return Ok(self);
        }

        if self.keep_within_component {
            // HACK(eddyb) print the path similarly to how `FmtPrinter` prints it.
            self.write_str("::")?;
        } else {
            self.path.finalize_pending_component();
        }

        write!(self, "{}", disambiguated_data.data)?;

        Ok(self)
    }
    fn path_generic_args(
        mut self,
        print_prefix: impl FnOnce(Self) -> Result<Self::Path, Self::Error>,
        args: &[GenericArg<'tcx>],
    ) -> Result<Self::Path, Self::Error> {
        self = print_prefix(self)?;

        let args =
            args.iter().cloned().filter(|arg| !matches!(arg.unpack(), GenericArgKind::Lifetime(_)));

        if args.clone().next().is_some() {
            self.generic_delimiters(|cx| cx.comma_sep(args))
        } else {
            Ok(self)
        }
    }
}

impl<'tcx> PrettyPrinter<'tcx> for &mut SymbolPrinter<'tcx> {
    fn should_print_region(&self, _region: ty::Region<'_>) -> bool {
        false
    }
    fn comma_sep<T>(mut self, mut elems: impl Iterator<Item = T>) -> Result<Self, Self::Error>
    where
        T: Print<'tcx, Self, Output = Self, Error = Self::Error>,
    {
        if let Some(first) = elems.next() {
            self = first.print(self)?;
            for elem in elems {
                self.write_str(",")?;
                self = elem.print(self)?;
            }
        }
        Ok(self)
    }

    fn generic_delimiters(
        mut self,
        f: impl FnOnce(Self) -> Result<Self, Self::Error>,
    ) -> Result<Self, Self::Error> {
        write!(self, "<")?;

        let kept_within_component = mem::replace(&mut self.keep_within_component, true);
        self = f(self)?;
        self.keep_within_component = kept_within_component;

        write!(self, ">")?;

        Ok(self)
    }
}

impl fmt::Write for SymbolPrinter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Name sanitation. LLVM will happily accept identifiers with weird names, but
        // gas doesn't!
        // gas accepts the following characters in symbols: a-z, A-Z, 0-9, ., _, $
        // NVPTX assembly has more strict naming rules than gas, so additionally, dots
        // are replaced with '$' there.

        for c in s.chars() {
            if self.path.temp_buf.is_empty() {
                match c {
                    'a'..='z' | 'A'..='Z' | '_' => {}
                    _ => {
                        // Underscore-qualify anything that didn't start as an ident.
                        self.path.temp_buf.push('_');
                    }
                }
            }
            match c {
                // Escape these with $ sequences
                '@' => self.path.temp_buf.push_str("$SP$"),
                '*' => self.path.temp_buf.push_str("$BP$"),
                '&' => self.path.temp_buf.push_str("$RF$"),
                '<' => self.path.temp_buf.push_str("$LT$"),
                '>' => self.path.temp_buf.push_str("$GT$"),
                '(' => self.path.temp_buf.push_str("$LP$"),
                ')' => self.path.temp_buf.push_str("$RP$"),
                ',' => self.path.temp_buf.push_str("$C$"),

                '-' | ':' | '.' if self.tcx.has_strict_asm_symbol_naming() => {
                    // NVPTX doesn't support these characters in symbol names.
                    self.path.temp_buf.push('$')
                }

                // '.' doesn't occur in types and functions, so reuse it
                // for ':' and '-'
                '-' | ':' => self.path.temp_buf.push('.'),

                // Avoid crashing LLVM in certain (LTO-related) situations, see #60925.
                'm' if self.path.temp_buf.ends_with(".llv") => self.path.temp_buf.push_str("$u6d$"),

                // These are legal symbols
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | '$' => self.path.temp_buf.push(c),

                _ => {
                    self.path.temp_buf.push('$');
                    for c in c.escape_unicode().skip(1) {
                        match c {
                            '{' => {}
                            '}' => self.path.temp_buf.push('$'),
                            c => self.path.temp_buf.push(c),
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
