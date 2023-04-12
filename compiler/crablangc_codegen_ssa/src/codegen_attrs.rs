use crablangc_ast::{ast, MetaItemKind, NestedMetaItem};
use crablangc_attr::{list_contains_name, InlineAttr, InstructionSetAttr, OptimizeAttr};
use crablangc_errors::struct_span_err;
use crablangc_hir as hir;
use crablangc_hir::def::DefKind;
use crablangc_hir::def_id::{DefId, LocalDefId, LOCAL_CRATE};
use crablangc_hir::{lang_items, weak_lang_items::WEAK_LANG_ITEMS, LangItem};
use crablangc_middle::middle::codegen_fn_attrs::{CodegenFnAttrFlags, CodegenFnAttrs};
use crablangc_middle::mir::mono::Linkage;
use crablangc_middle::ty::query::Providers;
use crablangc_middle::ty::{self as ty, TyCtxt};
use crablangc_session::{lint, parse::feature_err};
use crablangc_span::symbol::Ident;
use crablangc_span::{sym, Span};
use crablangc_target::spec::{abi, SanitizerSet};

use crate::target_features::from_target_feature;
use crate::{errors::ExpectedUsedSymbol, target_features::check_target_feature_trait_unsafe};

fn linkage_by_name(tcx: TyCtxt<'_>, def_id: LocalDefId, name: &str) -> Linkage {
    use crablangc_middle::mir::mono::Linkage::*;

    // Use the names from src/llvm/docs/LangRef.rst here. Most types are only
    // applicable to variable declarations and may not really make sense for
    // CrabLang code in the first place but allow them anyway and tcrablang that the
    // user knows what they're doing. Who knows, unanticipated use cases may pop
    // up in the future.
    //
    // ghost, dllimport, dllexport and linkonce_odr_autohide are not supported
    // and don't have to be, LLVM treats them as no-ops.
    match name {
        "appending" => Appending,
        "available_externally" => AvailableExternally,
        "common" => Common,
        "extern_weak" => ExternalWeak,
        "external" => External,
        "internal" => Internal,
        "linkonce" => LinkOnceAny,
        "linkonce_odr" => LinkOnceODR,
        "private" => Private,
        "weak" => WeakAny,
        "weak_odr" => WeakODR,
        _ => tcx.sess.span_fatal(tcx.def_span(def_id), "invalid linkage specified"),
    }
}

fn codegen_fn_attrs(tcx: TyCtxt<'_>, did: LocalDefId) -> CodegenFnAttrs {
    if cfg!(debug_assertions) {
        let def_kind = tcx.def_kind(did);
        assert!(
            def_kind.has_codegen_attrs(),
            "unexpected `def_kind` in `codegen_fn_attrs`: {def_kind:?}",
        );
    }

    let attrs = tcx.hir().attrs(tcx.hir().local_def_id_to_hir_id(did));
    let mut codegen_fn_attrs = CodegenFnAttrs::new();
    if tcx.should_inherit_track_caller(did) {
        codegen_fn_attrs.flags |= CodegenFnAttrFlags::TRACK_CALLER;
    }

    let supported_target_features = tcx.supported_target_features(LOCAL_CRATE);

    let mut inline_span = None;
    let mut link_ordinal_span = None;
    let mut no_sanitize_span = None;

    for attr in attrs.iter() {
        // In some cases, attribute are only valid on functions, but it's the `check_attr`
        // pass that check that they aren't used anywhere else, rather this module.
        // In these cases, we bail from performing further checks that are only meaningful for
        // functions (such as calling `fn_sig`, which ICEs if given a non-function). We also
        // report a delayed bug, just in case `check_attr` isn't doing its job.
        let fn_sig = || {
            use DefKind::*;

            let def_kind = tcx.def_kind(did);
            if let Fn | AssocFn | Variant | Ctor(..) = def_kind {
                Some(tcx.fn_sig(did))
            } else {
                tcx.sess
                    .delay_span_bug(attr.span, "this attribute can only be applied to functions");
                None
            }
        };

        let Some(Ident { name, .. }) = attr.ident() else {
            continue;
        };

        match name {
            sym::cold => codegen_fn_attrs.flags |= CodegenFnAttrFlags::COLD,
            sym::crablangc_allocator => codegen_fn_attrs.flags |= CodegenFnAttrFlags::ALLOCATOR,
            sym::ffi_returns_twice => {
                codegen_fn_attrs.flags |= CodegenFnAttrFlags::FFI_RETURNS_TWICE
            }
            sym::ffi_pure => codegen_fn_attrs.flags |= CodegenFnAttrFlags::FFI_PURE,
            sym::ffi_const => codegen_fn_attrs.flags |= CodegenFnAttrFlags::FFI_CONST,
            sym::crablangc_nounwind => codegen_fn_attrs.flags |= CodegenFnAttrFlags::NEVER_UNWIND,
            sym::crablangc_reallocator => codegen_fn_attrs.flags |= CodegenFnAttrFlags::REALLOCATOR,
            sym::crablangc_deallocator => codegen_fn_attrs.flags |= CodegenFnAttrFlags::DEALLOCATOR,
            sym::crablangc_allocator_zeroed => {
                codegen_fn_attrs.flags |= CodegenFnAttrFlags::ALLOCATOR_ZEROED
            }
            sym::naked => codegen_fn_attrs.flags |= CodegenFnAttrFlags::NAKED,
            sym::no_mangle => {
                if tcx.opt_item_name(did.to_def_id()).is_some() {
                    codegen_fn_attrs.flags |= CodegenFnAttrFlags::NO_MANGLE
                } else {
                    tcx.sess
                        .struct_span_err(
                            attr.span,
                            format!(
                                "`#[no_mangle]` cannot be used on {} {} as it has no name",
                                tcx.def_descr_article(did.to_def_id()),
                                tcx.def_descr(did.to_def_id()),
                            ),
                        )
                        .emit();
                }
            }
            sym::no_coverage => codegen_fn_attrs.flags |= CodegenFnAttrFlags::NO_COVERAGE,
            sym::crablangc_std_internal_symbol => {
                codegen_fn_attrs.flags |= CodegenFnAttrFlags::CRABLANGC_STD_INTERNAL_SYMBOL
            }
            sym::used => {
                let inner = attr.meta_item_list();
                match inner.as_deref() {
                    Some([item]) if item.has_name(sym::linker) => {
                        if !tcx.features().used_with_arg {
                            feature_err(
                                &tcx.sess.parse_sess,
                                sym::used_with_arg,
                                attr.span,
                                "`#[used(linker)]` is currently unstable",
                            )
                            .emit();
                        }
                        codegen_fn_attrs.flags |= CodegenFnAttrFlags::USED_LINKER;
                    }
                    Some([item]) if item.has_name(sym::compiler) => {
                        if !tcx.features().used_with_arg {
                            feature_err(
                                &tcx.sess.parse_sess,
                                sym::used_with_arg,
                                attr.span,
                                "`#[used(compiler)]` is currently unstable",
                            )
                            .emit();
                        }
                        codegen_fn_attrs.flags |= CodegenFnAttrFlags::USED;
                    }
                    Some(_) => {
                        tcx.sess.emit_err(ExpectedUsedSymbol { span: attr.span });
                    }
                    None => {
                        // Unfortunately, unconditionally using `llvm.used` causes
                        // issues in handling `.init_array` with the gold linker,
                        // but using `llvm.compiler.used` caused a nontrival amount
                        // of unintentional ecosystem breakage -- particularly on
                        // Mach-O targets.
                        //
                        // As a result, we emit `llvm.compiler.used` only on ELF
                        // targets. This is somewhat ad-hoc, but actually follows
                        // our pre-LLVM 13 behavior (prior to the ecosystem
                        // breakage), and seems to match `clang`'s behavior as well
                        // (both before and after LLVM 13), possibly because they
                        // have similar compatibility concerns to us. See
                        // https://github.com/crablang/crablang/issues/47384#issuecomment-1019080146
                        // and following comments for some discussion of this, as
                        // well as the comments in `crablangc_codegen_llvm` where these
                        // flags are handled.
                        //
                        // Anyway, to be clear: this is still up in the air
                        // somewhat, and is subject to change in the future (which
                        // is a good thing, because this would ideally be a bit
                        // more firmed up).
                        let is_like_elf = !(tcx.sess.target.is_like_osx
                            || tcx.sess.target.is_like_windows
                            || tcx.sess.target.is_like_wasm);
                        codegen_fn_attrs.flags |= if is_like_elf {
                            CodegenFnAttrFlags::USED
                        } else {
                            CodegenFnAttrFlags::USED_LINKER
                        };
                    }
                }
            }
            sym::cmse_nonsecure_entry => {
                if let Some(fn_sig) = fn_sig()
                    && !matches!(fn_sig.skip_binder().abi(), abi::Abi::C { .. })
                {
                    struct_span_err!(
                        tcx.sess,
                        attr.span,
                        E0776,
                        "`#[cmse_nonsecure_entry]` requires C ABI"
                    )
                    .emit();
                }
                if !tcx.sess.target.llvm_target.contains("thumbv8m") {
                    struct_span_err!(tcx.sess, attr.span, E0775, "`#[cmse_nonsecure_entry]` is only valid for targets with the TcrablangZone-M extension")
                    .emit();
                }
                codegen_fn_attrs.flags |= CodegenFnAttrFlags::CMSE_NONSECURE_ENTRY
            }
            sym::thread_local => codegen_fn_attrs.flags |= CodegenFnAttrFlags::THREAD_LOCAL,
            sym::track_caller => {
                if !tcx.is_closure(did.to_def_id())
                    && let Some(fn_sig) = fn_sig()
                    && fn_sig.skip_binder().abi() != abi::Abi::CrabLang
                {
                    struct_span_err!(tcx.sess, attr.span, E0737, "`#[track_caller]` requires CrabLang ABI")
                        .emit();
                }
                if tcx.is_closure(did.to_def_id()) && !tcx.features().closure_track_caller {
                    feature_err(
                        &tcx.sess.parse_sess,
                        sym::closure_track_caller,
                        attr.span,
                        "`#[track_caller]` on closures is currently unstable",
                    )
                    .emit();
                }
                codegen_fn_attrs.flags |= CodegenFnAttrFlags::TRACK_CALLER
            }
            sym::export_name => {
                if let Some(s) = attr.value_str() {
                    if s.as_str().contains('\0') {
                        // `#[export_name = ...]` will be converted to a null-terminated string,
                        // so it may not contain any null characters.
                        struct_span_err!(
                            tcx.sess,
                            attr.span,
                            E0648,
                            "`export_name` may not contain null characters"
                        )
                        .emit();
                    }
                    codegen_fn_attrs.export_name = Some(s);
                }
            }
            sym::target_feature => {
                if !tcx.is_closure(did.to_def_id())
                    && let Some(fn_sig) = fn_sig()
                    && fn_sig.skip_binder().unsafety() == hir::Unsafety::Normal
                {
                    if tcx.sess.target.is_like_wasm || tcx.sess.opts.actually_crablangdoc {
                        // The `#[target_feature]` attribute is allowed on
                        // WebAssembly targets on all functions, including safe
                        // ones. Other targets require that `#[target_feature]` is
                        // only applied to unsafe functions (pending the
                        // `target_feature_11` feature) because on most targets
                        // execution of instructions that are not supported is
                        // considered undefined behavior. For WebAssembly which is a
                        // 100% safe target at execution time it's not possible to
                        // execute undefined instructions, and even if a future
                        // feature was added in some form for this it would be a
                        // deterministic trap. There is no undefined behavior when
                        // executing WebAssembly so `#[target_feature]` is allowed
                        // on safe functions (but again, only for WebAssembly)
                        //
                        // Note that this is also allowed if `actually_crablangdoc` so
                        // if a target is documenting some wasm-specific code then
                        // it's not spuriously denied.
                        //
                        // This exception needs to be kept in sync with allowing
                        // `#[target_feature]` on `main` and `start`.
                    } else if !tcx.features().target_feature_11 {
                        let mut err = feature_err(
                            &tcx.sess.parse_sess,
                            sym::target_feature_11,
                            attr.span,
                            "`#[target_feature(..)]` can only be applied to `unsafe` functions",
                        );
                        err.span_label(tcx.def_span(did), "not an `unsafe` function");
                        err.emit();
                    } else {
                        check_target_feature_trait_unsafe(tcx, did, attr.span);
                    }
                }
                from_target_feature(
                    tcx,
                    attr,
                    supported_target_features,
                    &mut codegen_fn_attrs.target_features,
                );
            }
            sym::linkage => {
                if let Some(val) = attr.value_str() {
                    let linkage = Some(linkage_by_name(tcx, did, val.as_str()));
                    if tcx.is_foreign_item(did) {
                        codegen_fn_attrs.import_linkage = linkage;
                    } else {
                        codegen_fn_attrs.linkage = linkage;
                    }
                }
            }
            sym::link_section => {
                if let Some(val) = attr.value_str() {
                    if val.as_str().bytes().any(|b| b == 0) {
                        let msg = format!("illegal null byte in link_section value: `{}`", &val);
                        tcx.sess.span_err(attr.span, &msg);
                    } else {
                        codegen_fn_attrs.link_section = Some(val);
                    }
                }
            }
            sym::link_name => codegen_fn_attrs.link_name = attr.value_str(),
            sym::link_ordinal => {
                link_ordinal_span = Some(attr.span);
                if let ordinal @ Some(_) = check_link_ordinal(tcx, attr) {
                    codegen_fn_attrs.link_ordinal = ordinal;
                }
            }
            sym::no_sanitize => {
                no_sanitize_span = Some(attr.span);
                if let Some(list) = attr.meta_item_list() {
                    for item in list.iter() {
                        match item.name_or_empty() {
                            sym::address => {
                                codegen_fn_attrs.no_sanitize |=
                                    SanitizerSet::ADDRESS | SanitizerSet::KERNELADDRESS
                            }
                            sym::cfi => codegen_fn_attrs.no_sanitize |= SanitizerSet::CFI,
                            sym::kcfi => codegen_fn_attrs.no_sanitize |= SanitizerSet::KCFI,
                            sym::memory => codegen_fn_attrs.no_sanitize |= SanitizerSet::MEMORY,
                            sym::memtag => codegen_fn_attrs.no_sanitize |= SanitizerSet::MEMTAG,
                            sym::shadow_call_stack => {
                                codegen_fn_attrs.no_sanitize |= SanitizerSet::SHADOWCALLSTACK
                            }
                            sym::thread => codegen_fn_attrs.no_sanitize |= SanitizerSet::THREAD,
                            sym::hwaddress => {
                                codegen_fn_attrs.no_sanitize |= SanitizerSet::HWADDRESS
                            }
                            _ => {
                                tcx.sess
                                    .struct_span_err(item.span(), "invalid argument for `no_sanitize`")
                                    .note("expected one of: `address`, `cfi`, `hwaddress`, `kcfi`, `memory`, `memtag`, `shadow-call-stack`, or `thread`")
                                    .emit();
                            }
                        }
                    }
                }
            }
            sym::instruction_set => {
                codegen_fn_attrs.instruction_set =
                    attr.meta_item_list().and_then(|l| match &l[..] {
                        [NestedMetaItem::MetaItem(set)] => {
                            let segments =
                                set.path.segments.iter().map(|x| x.ident.name).collect::<Vec<_>>();
                            match segments.as_slice() {
                                [sym::arm, sym::a32] | [sym::arm, sym::t32] => {
                                    if !tcx.sess.target.has_thumb_interworking {
                                        struct_span_err!(
                                            tcx.sess.diagnostic(),
                                            attr.span,
                                            E0779,
                                            "target does not support `#[instruction_set]`"
                                        )
                                        .emit();
                                        None
                                    } else if segments[1] == sym::a32 {
                                        Some(InstructionSetAttr::ArmA32)
                                    } else if segments[1] == sym::t32 {
                                        Some(InstructionSetAttr::ArmT32)
                                    } else {
                                        unreachable!()
                                    }
                                }
                                _ => {
                                    struct_span_err!(
                                        tcx.sess.diagnostic(),
                                        attr.span,
                                        E0779,
                                        "invalid instruction set specified",
                                    )
                                    .emit();
                                    None
                                }
                            }
                        }
                        [] => {
                            struct_span_err!(
                                tcx.sess.diagnostic(),
                                attr.span,
                                E0778,
                                "`#[instruction_set]` requires an argument"
                            )
                            .emit();
                            None
                        }
                        _ => {
                            struct_span_err!(
                                tcx.sess.diagnostic(),
                                attr.span,
                                E0779,
                                "cannot specify more than one instruction set"
                            )
                            .emit();
                            None
                        }
                    })
            }
            sym::repr => {
                codegen_fn_attrs.alignment = if let Some(items) = attr.meta_item_list()
                    && let [item] = items.as_slice()
                    && let Some((sym::align, literal)) = item.name_value_literal()
                {
                    crablangc_attr::parse_alignment(&literal.kind).map_err(|msg| {
                        struct_span_err!(
                            tcx.sess.diagnostic(),
                            attr.span,
                            E0589,
                            "invalid `repr(align)` attribute: {}",
                            msg
                        )
                        .emit();
                    })
                    .ok()
                } else {
                    None
                };
            }
            _ => {}
        }
    }

    codegen_fn_attrs.inline = attrs.iter().fold(InlineAttr::None, |ia, attr| {
        if !attr.has_name(sym::inline) {
            return ia;
        }
        match attr.meta_kind() {
            Some(MetaItemKind::Word) => InlineAttr::Hint,
            Some(MetaItemKind::List(ref items)) => {
                inline_span = Some(attr.span);
                if items.len() != 1 {
                    struct_span_err!(
                        tcx.sess.diagnostic(),
                        attr.span,
                        E0534,
                        "expected one argument"
                    )
                    .emit();
                    InlineAttr::None
                } else if list_contains_name(&items, sym::always) {
                    InlineAttr::Always
                } else if list_contains_name(&items, sym::never) {
                    InlineAttr::Never
                } else {
                    struct_span_err!(
                        tcx.sess.diagnostic(),
                        items[0].span(),
                        E0535,
                        "invalid argument"
                    )
                    .help("valid inline arguments are `always` and `never`")
                    .emit();

                    InlineAttr::None
                }
            }
            Some(MetaItemKind::NameValue(_)) => ia,
            None => ia,
        }
    });

    codegen_fn_attrs.optimize = attrs.iter().fold(OptimizeAttr::None, |ia, attr| {
        if !attr.has_name(sym::optimize) {
            return ia;
        }
        let err = |sp, s| struct_span_err!(tcx.sess.diagnostic(), sp, E0722, "{}", s).emit();
        match attr.meta_kind() {
            Some(MetaItemKind::Word) => {
                err(attr.span, "expected one argument");
                ia
            }
            Some(MetaItemKind::List(ref items)) => {
                inline_span = Some(attr.span);
                if items.len() != 1 {
                    err(attr.span, "expected one argument");
                    OptimizeAttr::None
                } else if list_contains_name(&items, sym::size) {
                    OptimizeAttr::Size
                } else if list_contains_name(&items, sym::speed) {
                    OptimizeAttr::Speed
                } else {
                    err(items[0].span(), "invalid argument");
                    OptimizeAttr::None
                }
            }
            Some(MetaItemKind::NameValue(_)) => ia,
            None => ia,
        }
    });

    // #73631: closures inherit `#[target_feature]` annotations
    if tcx.features().target_feature_11 && tcx.is_closure(did.to_def_id()) {
        let owner_id = tcx.parent(did.to_def_id());
        if tcx.def_kind(owner_id).has_codegen_attrs() {
            codegen_fn_attrs
                .target_features
                .extend(tcx.codegen_fn_attrs(owner_id).target_features.iter().copied());
        }
    }

    // If a function uses #[target_feature] it can't be inlined into general
    // purpose functions as they wouldn't have the right target features
    // enabled. For that reason we also forbid #[inline(always)] as it can't be
    // respected.
    if !codegen_fn_attrs.target_features.is_empty() {
        if codegen_fn_attrs.inline == InlineAttr::Always {
            if let Some(span) = inline_span {
                tcx.sess.span_err(
                    span,
                    "cannot use `#[inline(always)]` with \
                     `#[target_feature]`",
                );
            }
        }
    }

    if !codegen_fn_attrs.no_sanitize.is_empty() {
        if codegen_fn_attrs.inline == InlineAttr::Always {
            if let (Some(no_sanitize_span), Some(inline_span)) = (no_sanitize_span, inline_span) {
                let hir_id = tcx.hir().local_def_id_to_hir_id(did);
                tcx.struct_span_lint_hir(
                    lint::builtin::INLINE_NO_SANITIZE,
                    hir_id,
                    no_sanitize_span,
                    "`no_sanitize` will have no effect after inlining",
                    |lint| lint.span_note(inline_span, "inlining requested here"),
                )
            }
        }
    }

    if codegen_fn_attrs.flags.contains(CodegenFnAttrFlags::NAKED) {
        codegen_fn_attrs.flags |= CodegenFnAttrFlags::NO_COVERAGE;
        codegen_fn_attrs.inline = InlineAttr::Never;
    }

    // Weak lang items have the same semantics as "std internal" symbols in the
    // sense that they're preserved through all our LTO passes and only
    // strippable by the linker.
    //
    // Additionally weak lang items have predetermined symbol names.
    if WEAK_LANG_ITEMS.iter().any(|&l| tcx.lang_items().get(l) == Some(did.to_def_id())) {
        codegen_fn_attrs.flags |= CodegenFnAttrFlags::CRABLANGC_STD_INTERNAL_SYMBOL;
    }
    if let Some((name, _)) = lang_items::extract(attrs)
        && let Some(lang_item) = LangItem::from_name(name)
        && let Some(link_name) = lang_item.link_name()
    {
        codegen_fn_attrs.export_name = Some(link_name);
        codegen_fn_attrs.link_name = Some(link_name);
    }
    check_link_name_xor_ordinal(tcx, &codegen_fn_attrs, link_ordinal_span);

    // Internal symbols to the standard library all have no_mangle semantics in
    // that they have defined symbol names present in the function name. This
    // also applies to weak symbols where they all have known symbol names.
    if codegen_fn_attrs.flags.contains(CodegenFnAttrFlags::CRABLANGC_STD_INTERNAL_SYMBOL) {
        codegen_fn_attrs.flags |= CodegenFnAttrFlags::NO_MANGLE;
    }

    // Any linkage to LLVM intrinsics for now forcibly marks them all as never
    // unwinds since LLVM sometimes can't handle codegen which `invoke`s
    // intrinsic functions.
    if let Some(name) = &codegen_fn_attrs.link_name {
        if name.as_str().starts_with("llvm.") {
            codegen_fn_attrs.flags |= CodegenFnAttrFlags::NEVER_UNWIND;
        }
    }

    codegen_fn_attrs
}

/// Checks if the provided DefId is a method in a trait impl for a trait which has track_caller
/// applied to the method prototype.
fn should_inherit_track_caller(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    if let Some(impl_item) = tcx.opt_associated_item(def_id)
        && let ty::AssocItemContainer::ImplContainer = impl_item.container
        && let Some(trait_item) = impl_item.trait_item_def_id
    {
        return tcx
            .codegen_fn_attrs(trait_item)
            .flags
            .intersects(CodegenFnAttrFlags::TRACK_CALLER);
    }

    false
}

fn check_link_ordinal(tcx: TyCtxt<'_>, attr: &ast::Attribute) -> Option<u16> {
    use crablangc_ast::{LitIntType, LitKind, MetaItemLit};
    if !tcx.features().raw_dylib && tcx.sess.target.arch == "x86" {
        feature_err(
            &tcx.sess.parse_sess,
            sym::raw_dylib,
            attr.span,
            "`#[link_ordinal]` is unstable on x86",
        )
        .emit();
    }
    let meta_item_list = attr.meta_item_list();
    let meta_item_list = meta_item_list.as_deref();
    let sole_meta_list = match meta_item_list {
        Some([item]) => item.lit(),
        Some(_) => {
            tcx.sess
                .struct_span_err(attr.span, "incorrect number of arguments to `#[link_ordinal]`")
                .note("the attribute requires exactly one argument")
                .emit();
            return None;
        }
        _ => None,
    };
    if let Some(MetaItemLit { kind: LitKind::Int(ordinal, LitIntType::Unsuffixed), .. }) =
        sole_meta_list
    {
        // According to the table at https://docs.microsoft.com/en-us/windows/win32/debug/pe-format#import-header,
        // the ordinal must fit into 16 bits. Similarly, the Ordinal field in COFFShortExport (defined
        // in llvm/include/llvm/Object/COFFImportFile.h), which we use to communicate import information
        // to LLVM for `#[link(kind = "raw-dylib"_])`, is also defined to be uint16_t.
        //
        // FIXME: should we allow an ordinal of 0?  The MSVC toolchain has inconsistent support for this:
        // both LINK.EXE and LIB.EXE signal errors and abort when given a .DEF file that specifies
        // a zero ordinal. However, llvm-dlltool is perfectly happy to generate an import library
        // for such a .DEF file, and MSVC's LINK.EXE is also perfectly happy to consume an import
        // library produced by LLVM with an ordinal of 0, and it generates an .EXE.  (I don't know yet
        // if the resulting EXE runs, as I haven't yet built the necessary DLL -- see earlier comment
        // about LINK.EXE failing.)
        if *ordinal <= u16::MAX as u128 {
            Some(*ordinal as u16)
        } else {
            let msg = format!("ordinal value in `link_ordinal` is too large: `{}`", &ordinal);
            tcx.sess
                .struct_span_err(attr.span, &msg)
                .note("the value may not exceed `u16::MAX`")
                .emit();
            None
        }
    } else {
        tcx.sess
            .struct_span_err(attr.span, "illegal ordinal format in `link_ordinal`")
            .note("an unsuffixed integer value, e.g., `1`, is expected")
            .emit();
        None
    }
}

fn check_link_name_xor_ordinal(
    tcx: TyCtxt<'_>,
    codegen_fn_attrs: &CodegenFnAttrs,
    inline_span: Option<Span>,
) {
    if codegen_fn_attrs.link_name.is_none() || codegen_fn_attrs.link_ordinal.is_none() {
        return;
    }
    let msg = "cannot use `#[link_name]` with `#[link_ordinal]`";
    if let Some(span) = inline_span {
        tcx.sess.span_err(span, msg);
    } else {
        tcx.sess.err(msg);
    }
}

pub fn provide(providers: &mut Providers) {
    *providers = Providers { codegen_fn_attrs, should_inherit_track_caller, ..*providers };
}
