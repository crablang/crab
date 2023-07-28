use crate::base;
use crate::traits::*;
use rustc_index::bit_set::BitSet;
use rustc_index::IndexVec;
use rustc_middle::mir;
use rustc_middle::mir::interpret::ErrorHandled;
use rustc_middle::mir::traversal;
use rustc_middle::ty::layout::{FnAbiOf, HasTyCtxt, TyAndLayout};
use rustc_middle::ty::{self, Instance, Ty, TyCtxt, TypeFoldable, TypeVisitableExt};
use rustc_target::abi::call::{FnAbi, PassMode};

use std::iter;

mod analyze;
mod block;
pub mod constant;
pub mod coverageinfo;
pub mod debuginfo;
mod intrinsic;
mod locals;
pub mod operand;
pub mod place;
mod rvalue;
mod statement;

use self::debuginfo::{FunctionDebugContext, PerLocalVarDebugInfo};
use self::operand::{OperandRef, OperandValue};
use self::place::PlaceRef;

// Used for tracking the state of generated basic blocks.
enum CachedLlbb<T> {
    /// Nothing created yet.
    None,

    /// Has been created.
    Some(T),

    /// Nothing created yet, and nothing should be.
    Skip,
}

/// Master context for codegenning from MIR.
pub struct FunctionCx<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>> {
    instance: Instance<'tcx>,

    mir: &'tcx mir::Body<'tcx>,

    debug_context: Option<FunctionDebugContext<Bx::DIScope, Bx::DILocation>>,

    llfn: Bx::Function,

    cx: &'a Bx::CodegenCx,

    fn_abi: &'tcx FnAbi<'tcx, Ty<'tcx>>,

    /// When unwinding is initiated, we have to store this personality
    /// value somewhere so that we can load it and re-use it in the
    /// resume instruction. The personality is (afaik) some kind of
    /// value used for C++ unwinding, which must filter by type: we
    /// don't really care about it very much. Anyway, this value
    /// contains an alloca into which the personality is stored and
    /// then later loaded when generating the DIVERGE_BLOCK.
    personality_slot: Option<PlaceRef<'tcx, Bx::Value>>,

    /// A backend `BasicBlock` for each MIR `BasicBlock`, created lazily
    /// as-needed (e.g. RPO reaching it or another block branching to it).
    // FIXME(eddyb) rename `llbbs` and other `ll`-prefixed things to use a
    // more backend-agnostic prefix such as `cg` (i.e. this would be `cgbbs`).
    cached_llbbs: IndexVec<mir::BasicBlock, CachedLlbb<Bx::BasicBlock>>,

    /// The funclet status of each basic block
    cleanup_kinds: Option<IndexVec<mir::BasicBlock, analyze::CleanupKind>>,

    /// When targeting MSVC, this stores the cleanup info for each funclet BB.
    /// This is initialized at the same time as the `landing_pads` entry for the
    /// funclets' head block, i.e. when needed by an unwind / `cleanup_ret` edge.
    funclets: IndexVec<mir::BasicBlock, Option<Bx::Funclet>>,

    /// This stores the cached landing/cleanup pad block for a given BB.
    // FIXME(eddyb) rename this to `eh_pads`.
    landing_pads: IndexVec<mir::BasicBlock, Option<Bx::BasicBlock>>,

    /// Cached unreachable block
    unreachable_block: Option<Bx::BasicBlock>,

    /// Cached terminate upon unwinding block
    terminate_block: Option<Bx::BasicBlock>,

    /// The location where each MIR arg/var/tmp/ret is stored. This is
    /// usually an `PlaceRef` representing an alloca, but not always:
    /// sometimes we can skip the alloca and just store the value
    /// directly using an `OperandRef`, which makes for tighter LLVM
    /// IR. The conditions for using an `OperandRef` are as follows:
    ///
    /// - the type of the local must be judged "immediate" by `is_llvm_immediate`
    /// - the operand must never be referenced indirectly
    ///     - we should not take its address using the `&` operator
    ///     - nor should it appear in a place path like `tmp.a`
    /// - the operand must be defined by an rvalue that can generate immediate
    ///   values
    ///
    /// Avoiding allocs can also be important for certain intrinsics,
    /// notably `expect`.
    locals: locals::Locals<'tcx, Bx::Value>,

    /// All `VarDebugInfo` from the MIR body, partitioned by `Local`.
    /// This is `None` if no var`#[non_exhaustive]`iable debuginfo/names are needed.
    per_local_var_debug_info:
        Option<IndexVec<mir::Local, Vec<PerLocalVarDebugInfo<'tcx, Bx::DIVariable>>>>,

    /// Caller location propagated if this function has `#[track_caller]`.
    caller_location: Option<OperandRef<'tcx, Bx::Value>>,
}

impl<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>> FunctionCx<'a, 'tcx, Bx> {
    pub fn monomorphize<T>(&self, value: T) -> T
    where
        T: Copy + TypeFoldable<TyCtxt<'tcx>>,
    {
        debug!("monomorphize: self.instance={:?}", self.instance);
        self.instance.subst_mir_and_normalize_erasing_regions(
            self.cx.tcx(),
            ty::ParamEnv::reveal_all(),
            ty::EarlyBinder::bind(value),
        )
    }
}

enum LocalRef<'tcx, V> {
    Place(PlaceRef<'tcx, V>),
    /// `UnsizedPlace(p)`: `p` itself is a thin pointer (indirect place).
    /// `*p` is the fat pointer that references the actual unsized place.
    /// Every time it is initialized, we have to reallocate the place
    /// and update the fat pointer. That's the reason why it is indirect.
    UnsizedPlace(PlaceRef<'tcx, V>),
    /// The backend [`OperandValue`] has already been generated.
    Operand(OperandRef<'tcx, V>),
    /// Will be a `Self::Operand` once we get to its definition.
    PendingOperand,
}

impl<'tcx, V: CodegenObject> LocalRef<'tcx, V> {
    fn new_operand(layout: TyAndLayout<'tcx>) -> LocalRef<'tcx, V> {
        if layout.is_zst() {
            // Zero-size temporaries aren't always initialized, which
            // doesn't matter because they don't contain data, but
            // we need something in the operand.
            LocalRef::Operand(OperandRef::zero_sized(layout))
        } else {
            LocalRef::PendingOperand
        }
    }
}

///////////////////////////////////////////////////////////////////////////

#[instrument(level = "debug", skip(cx))]
pub fn codegen_mir<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>>(
    cx: &'a Bx::CodegenCx,
    instance: Instance<'tcx>,
) {
    assert!(!instance.args.has_infer());

    let llfn = cx.get_fn(instance);

    let mir = cx.tcx().instance_mir(instance.def);

    let fn_abi = cx.fn_abi_of_instance(instance, ty::List::empty());
    debug!("fn_abi: {:?}", fn_abi);

    let debug_context = cx.create_function_debug_context(instance, &fn_abi, llfn, &mir);

    let start_llbb = Bx::append_block(cx, llfn, "start");
    let mut start_bx = Bx::build(cx, start_llbb);

    if mir.basic_blocks.iter().any(|bb| {
        bb.is_cleanup || matches!(bb.terminator().unwind(), Some(mir::UnwindAction::Terminate))
    }) {
        start_bx.set_personality_fn(cx.eh_personality());
    }

    let cleanup_kinds =
        base::wants_new_eh_instructions(cx.tcx().sess).then(|| analyze::cleanup_kinds(&mir));

    let cached_llbbs: IndexVec<mir::BasicBlock, CachedLlbb<Bx::BasicBlock>> =
        mir.basic_blocks
            .indices()
            .map(|bb| {
                if bb == mir::START_BLOCK { CachedLlbb::Some(start_llbb) } else { CachedLlbb::None }
            })
            .collect();

    let mut fx = FunctionCx {
        instance,
        mir,
        llfn,
        fn_abi,
        cx,
        personality_slot: None,
        cached_llbbs,
        unreachable_block: None,
        terminate_block: None,
        cleanup_kinds,
        landing_pads: IndexVec::from_elem(None, &mir.basic_blocks),
        funclets: IndexVec::from_fn_n(|_| None, mir.basic_blocks.len()),
        locals: locals::Locals::empty(),
        debug_context,
        per_local_var_debug_info: None,
        caller_location: None,
    };

    fx.per_local_var_debug_info = fx.compute_per_local_var_debug_info(&mut start_bx);

    // Evaluate all required consts; codegen later assumes that CTFE will never fail.
    let mut all_consts_ok = true;
    for const_ in &mir.required_consts {
        if let Err(err) = fx.eval_mir_constant(const_) {
            all_consts_ok = false;
            match err {
                // errored or at least linted
                ErrorHandled::Reported(_) => {}
                ErrorHandled::TooGeneric => {
                    span_bug!(const_.span, "codegen encountered polymorphic constant: {:?}", err)
                }
            }
        }
    }
    if !all_consts_ok {
        // We leave the IR in some half-built state here, and rely on this code not even being
        // submitted to LLVM once an error was raised.
        return;
    }

    let memory_locals = analyze::non_ssa_locals(&fx);

    // Allocate variable and temp allocas
    let local_values = {
        let args = arg_local_refs(&mut start_bx, &mut fx, &memory_locals);

        let mut allocate_local = |local| {
            let decl = &mir.local_decls[local];
            let layout = start_bx.layout_of(fx.monomorphize(decl.ty));
            assert!(!layout.ty.has_erasable_regions());

            if local == mir::RETURN_PLACE && fx.fn_abi.ret.is_indirect() {
                debug!("alloc: {:?} (return place) -> place", local);
                let llretptr = start_bx.get_param(0);
                return LocalRef::Place(PlaceRef::new_sized(llretptr, layout));
            }

            if memory_locals.contains(local) {
                debug!("alloc: {:?} -> place", local);
                if layout.is_unsized() {
                    LocalRef::UnsizedPlace(PlaceRef::alloca_unsized_indirect(&mut start_bx, layout))
                } else {
                    LocalRef::Place(PlaceRef::alloca(&mut start_bx, layout))
                }
            } else {
                debug!("alloc: {:?} -> operand", local);
                LocalRef::new_operand(layout)
            }
        };

        let retptr = allocate_local(mir::RETURN_PLACE);
        iter::once(retptr)
            .chain(args.into_iter())
            .chain(mir.vars_and_temps_iter().map(allocate_local))
            .collect()
    };
    fx.initialize_locals(local_values);

    // Apply debuginfo to the newly allocated locals.
    fx.debug_introduce_locals(&mut start_bx);

    // The builders will be created separately for each basic block at `codegen_block`.
    // So drop the builder of `start_llbb` to avoid having two at the same time.
    drop(start_bx);

    // Codegen the body of each block using reverse postorder
    for (bb, _) in traversal::reverse_postorder(&mir) {
        fx.codegen_block(bb);
    }
}

/// Produces, for each argument, a `Value` pointing at the
/// argument's value. As arguments are places, these are always
/// indirect.
fn arg_local_refs<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>>(
    bx: &mut Bx,
    fx: &mut FunctionCx<'a, 'tcx, Bx>,
    memory_locals: &BitSet<mir::Local>,
) -> Vec<LocalRef<'tcx, Bx::Value>> {
    let mir = fx.mir;
    let mut idx = 0;
    let mut llarg_idx = fx.fn_abi.ret.is_indirect() as usize;

    let mut num_untupled = None;

    let args = mir
        .args_iter()
        .enumerate()
        .map(|(arg_index, local)| {
            let arg_decl = &mir.local_decls[local];
            let arg_ty = fx.monomorphize(arg_decl.ty);

            if Some(local) == mir.spread_arg {
                // This argument (e.g., the last argument in the "rust-call" ABI)
                // is a tuple that was spread at the ABI level and now we have
                // to reconstruct it into a tuple local variable, from multiple
                // individual LLVM function arguments.
                let ty::Tuple(tupled_arg_tys) = arg_ty.kind() else {
                    bug!("spread argument isn't a tuple?!");
                };

                let layout = bx.layout_of(arg_ty);

                // FIXME: support unsized params in "rust-call" ABI
                if layout.is_unsized() {
                    span_bug!(
                        arg_decl.source_info.span,
                        "\"rust-call\" ABI does not support unsized params",
                    );
                }

                let place = PlaceRef::alloca(bx, layout);
                for i in 0..tupled_arg_tys.len() {
                    let arg = &fx.fn_abi.args[idx];
                    idx += 1;
                    if let PassMode::Cast(_, true) = arg.mode {
                        llarg_idx += 1;
                    }
                    let pr_field = place.project_field(bx, i);
                    bx.store_fn_arg(arg, &mut llarg_idx, pr_field);
                }
                assert_eq!(
                    None,
                    num_untupled.replace(tupled_arg_tys.len()),
                    "Replaced existing num_tupled"
                );

                return LocalRef::Place(place);
            }

            if fx.fn_abi.c_variadic && arg_index == fx.fn_abi.args.len() {
                let va_list = PlaceRef::alloca(bx, bx.layout_of(arg_ty));
                bx.va_start(va_list.llval);

                return LocalRef::Place(va_list);
            }

            let arg = &fx.fn_abi.args[idx];
            idx += 1;
            if let PassMode::Cast(_, true) = arg.mode {
                llarg_idx += 1;
            }

            if !memory_locals.contains(local) {
                // We don't have to cast or keep the argument in the alloca.
                // FIXME(eddyb): We should figure out how to use llvm.dbg.value instead
                // of putting everything in allocas just so we can use llvm.dbg.declare.
                let local = |op| LocalRef::Operand(op);
                match arg.mode {
                    PassMode::Ignore => {
                        return local(OperandRef::zero_sized(arg.layout));
                    }
                    PassMode::Direct(_) => {
                        let llarg = bx.get_param(llarg_idx);
                        llarg_idx += 1;
                        return local(OperandRef::from_immediate_or_packed_pair(
                            bx, llarg, arg.layout,
                        ));
                    }
                    PassMode::Pair(..) => {
                        let (a, b) = (bx.get_param(llarg_idx), bx.get_param(llarg_idx + 1));
                        llarg_idx += 2;

                        return local(OperandRef {
                            val: OperandValue::Pair(a, b),
                            layout: arg.layout,
                        });
                    }
                    _ => {}
                }
            }

            if arg.is_sized_indirect() {
                // Don't copy an indirect argument to an alloca, the caller
                // already put it in a temporary alloca and gave it up.
                // FIXME: lifetimes
                let llarg = bx.get_param(llarg_idx);
                llarg_idx += 1;
                LocalRef::Place(PlaceRef::new_sized(llarg, arg.layout))
            } else if arg.is_unsized_indirect() {
                // As the storage for the indirect argument lives during
                // the whole function call, we just copy the fat pointer.
                let llarg = bx.get_param(llarg_idx);
                llarg_idx += 1;
                let llextra = bx.get_param(llarg_idx);
                llarg_idx += 1;
                let indirect_operand = OperandValue::Pair(llarg, llextra);

                let tmp = PlaceRef::alloca_unsized_indirect(bx, arg.layout);
                indirect_operand.store(bx, tmp);
                LocalRef::UnsizedPlace(tmp)
            } else {
                let tmp = PlaceRef::alloca(bx, arg.layout);
                bx.store_fn_arg(arg, &mut llarg_idx, tmp);
                LocalRef::Place(tmp)
            }
        })
        .collect::<Vec<_>>();

    if fx.instance.def.requires_caller_location(bx.tcx()) {
        let mir_args = if let Some(num_untupled) = num_untupled {
            // Subtract off the tupled argument that gets 'expanded'
            args.len() - 1 + num_untupled
        } else {
            args.len()
        };
        assert_eq!(
            fx.fn_abi.args.len(),
            mir_args + 1,
            "#[track_caller] instance {:?} must have 1 more argument in their ABI than in their MIR",
            fx.instance
        );

        let arg = fx.fn_abi.args.last().unwrap();
        match arg.mode {
            PassMode::Direct(_) => (),
            _ => bug!("caller location must be PassMode::Direct, found {:?}", arg.mode),
        }

        fx.caller_location = Some(OperandRef {
            val: OperandValue::Immediate(bx.get_param(llarg_idx)),
            layout: arg.layout,
        });
    }

    args
}
