//! Code to extract the universally quantified regions declared on a
//! function and the relationships between them. For example:
//!
//! ```
//! fn foo<'a, 'b, 'c: 'b>() { }
//! ```
//!
//! here we would return a map assigning each of `{'a, 'b, 'c}`
//! to an index, as well as the `FreeRegionMap` which can compute
//! relationships between them.
//!
//! The code in this file doesn't *do anything* with those results; it
//! just returns them for other code to use.

use either::Either;
use rustc_data_structures::fx::FxHashMap;
use rustc_errors::Diagnostic;
use rustc_hir as hir;
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_hir::lang_items::LangItem;
use rustc_hir::BodyOwnerKind;
use rustc_index::IndexVec;
use rustc_infer::infer::NllRegionVariableOrigin;
use rustc_middle::ty::fold::TypeFoldable;
use rustc_middle::ty::{self, InlineConstSubsts, InlineConstSubstsParts, RegionVid, Ty, TyCtxt};
use rustc_middle::ty::{InternalSubsts, SubstsRef};
use rustc_span::symbol::{kw, sym};
use rustc_span::Symbol;
use std::iter;

use crate::renumber::{BoundRegionInfo, RegionCtxt};
use crate::BorrowckInferCtxt;

#[derive(Debug)]
pub struct UniversalRegions<'tcx> {
    indices: UniversalRegionIndices<'tcx>,

    /// The vid assigned to `'static`
    pub fr_static: RegionVid,

    /// A special region vid created to represent the current MIR fn
    /// body. It will outlive the entire CFG but it will not outlive
    /// any other universal regions.
    pub fr_fn_body: RegionVid,

    /// We create region variables such that they are ordered by their
    /// `RegionClassification`. The first block are globals, then
    /// externals, then locals. So, things from:
    /// - `FIRST_GLOBAL_INDEX..first_extern_index` are global,
    /// - `first_extern_index..first_local_index` are external,
    /// - `first_local_index..num_universals` are local.
    first_extern_index: usize,

    /// See `first_extern_index`.
    first_local_index: usize,

    /// The total number of universal region variables instantiated.
    num_universals: usize,

    /// The "defining" type for this function, with all universal
    /// regions instantiated. For a closure or generator, this is the
    /// closure type, but for a top-level function it's the `FnDef`.
    pub defining_ty: DefiningTy<'tcx>,

    /// The return type of this function, with all regions replaced by
    /// their universal `RegionVid` equivalents.
    ///
    /// N.B., associated types in this type have not been normalized,
    /// as the name suggests. =)
    pub unnormalized_output_ty: Ty<'tcx>,

    /// The fully liberated input types of this function, with all
    /// regions replaced by their universal `RegionVid` equivalents.
    ///
    /// N.B., associated types in these types have not been normalized,
    /// as the name suggests. =)
    pub unnormalized_input_tys: &'tcx [Ty<'tcx>],

    pub yield_ty: Option<Ty<'tcx>>,
}

/// The "defining type" for this MIR. The key feature of the "defining
/// type" is that it contains the information needed to derive all the
/// universal regions that are in scope as well as the types of the
/// inputs/output from the MIR. In general, early-bound universal
/// regions appear free in the defining type and late-bound regions
/// appear bound in the signature.
#[derive(Copy, Clone, Debug)]
pub enum DefiningTy<'tcx> {
    /// The MIR is a closure. The signature is found via
    /// `ClosureSubsts::closure_sig_ty`.
    Closure(DefId, SubstsRef<'tcx>),

    /// The MIR is a generator. The signature is that generators take
    /// no parameters and return the result of
    /// `ClosureSubsts::generator_return_ty`.
    Generator(DefId, SubstsRef<'tcx>, hir::Movability),

    /// The MIR is a fn item with the given `DefId` and substs. The signature
    /// of the function can be bound then with the `fn_sig` query.
    FnDef(DefId, SubstsRef<'tcx>),

    /// The MIR represents some form of constant. The signature then
    /// is that it has no inputs and a single return value, which is
    /// the value of the constant.
    Const(DefId, SubstsRef<'tcx>),

    /// The MIR represents an inline const. The signature has no inputs and a
    /// single return value found via `InlineConstSubsts::ty`.
    InlineConst(DefId, SubstsRef<'tcx>),
}

impl<'tcx> DefiningTy<'tcx> {
    /// Returns a list of all the upvar types for this MIR. If this is
    /// not a closure or generator, there are no upvars, and hence it
    /// will be an empty list. The order of types in this list will
    /// match up with the upvar order in the HIR, typesystem, and MIR.
    pub fn upvar_tys(self) -> impl Iterator<Item = Ty<'tcx>> + 'tcx {
        match self {
            DefiningTy::Closure(_, substs) => Either::Left(substs.as_closure().upvar_tys()),
            DefiningTy::Generator(_, substs, _) => {
                Either::Right(Either::Left(substs.as_generator().upvar_tys()))
            }
            DefiningTy::FnDef(..) | DefiningTy::Const(..) | DefiningTy::InlineConst(..) => {
                Either::Right(Either::Right(iter::empty()))
            }
        }
    }

    /// Number of implicit inputs -- notably the "environment"
    /// parameter for closures -- that appear in MIR but not in the
    /// user's code.
    pub fn implicit_inputs(self) -> usize {
        match self {
            DefiningTy::Closure(..) | DefiningTy::Generator(..) => 1,
            DefiningTy::FnDef(..) | DefiningTy::Const(..) | DefiningTy::InlineConst(..) => 0,
        }
    }

    pub fn is_fn_def(&self) -> bool {
        matches!(*self, DefiningTy::FnDef(..))
    }

    pub fn is_const(&self) -> bool {
        matches!(*self, DefiningTy::Const(..) | DefiningTy::InlineConst(..))
    }

    pub fn def_id(&self) -> DefId {
        match *self {
            DefiningTy::Closure(def_id, ..)
            | DefiningTy::Generator(def_id, ..)
            | DefiningTy::FnDef(def_id, ..)
            | DefiningTy::Const(def_id, ..)
            | DefiningTy::InlineConst(def_id, ..) => def_id,
        }
    }
}

#[derive(Debug)]
struct UniversalRegionIndices<'tcx> {
    /// For those regions that may appear in the parameter environment
    /// ('static and early-bound regions), we maintain a map from the
    /// `ty::Region` to the internal `RegionVid` we are using. This is
    /// used because trait matching and type-checking will feed us
    /// region constraints that reference those regions and we need to
    /// be able to map them to our internal `RegionVid`. This is
    /// basically equivalent to an `InternalSubsts`, except that it also
    /// contains an entry for `ReStatic` -- it might be nice to just
    /// use a substs, and then handle `ReStatic` another way.
    indices: FxHashMap<ty::Region<'tcx>, RegionVid>,

    /// The vid assigned to `'static`. Used only for diagnostics.
    pub fr_static: RegionVid,
}

#[derive(Debug, PartialEq)]
pub enum RegionClassification {
    /// A **global** region is one that can be named from
    /// anywhere. There is only one, `'static`.
    Global,

    /// An **external** region is only relevant for
    /// closures, generators, and inline consts. In that
    /// case, it refers to regions that are free in the type
    /// -- basically, something bound in the surrounding context.
    ///
    /// Consider this example:
    ///
    /// ```ignore (pseudo-rust)
    /// fn foo<'a, 'b>(a: &'a u32, b: &'b u32, c: &'static u32) {
    ///   let closure = for<'x> |x: &'x u32| { .. };
    ///    //           ^^^^^^^ pretend this were legal syntax
    ///    //                   for declaring a late-bound region in
    ///    //                   a closure signature
    /// }
    /// ```
    ///
    /// Here, the lifetimes `'a` and `'b` would be **external** to the
    /// closure.
    ///
    /// If we are not analyzing a closure/generator/inline-const,
    /// there are no external lifetimes.
    External,

    /// A **local** lifetime is one about which we know the full set
    /// of relevant constraints (that is, relationships to other named
    /// regions). For a closure, this includes any region bound in
    /// the closure's signature. For a fn item, this includes all
    /// regions other than global ones.
    ///
    /// Continuing with the example from `External`, if we were
    /// analyzing the closure, then `'x` would be local (and `'a` and
    /// `'b` are external). If we are analyzing the function item
    /// `foo`, then `'a` and `'b` are local (and `'x` is not in
    /// scope).
    Local,
}

const FIRST_GLOBAL_INDEX: usize = 0;

impl<'tcx> UniversalRegions<'tcx> {
    /// Creates a new and fully initialized `UniversalRegions` that
    /// contains indices for all the free regions found in the given
    /// MIR -- that is, all the regions that appear in the function's
    /// signature. This will also compute the relationships that are
    /// known between those regions.
    pub fn new(
        infcx: &BorrowckInferCtxt<'_, 'tcx>,
        mir_def: LocalDefId,
        param_env: ty::ParamEnv<'tcx>,
    ) -> Self {
        UniversalRegionsBuilder { infcx, mir_def, param_env }.build()
    }

    /// Given a reference to a closure type, extracts all the values
    /// from its free regions and returns a vector with them. This is
    /// used when the closure's creator checks that the
    /// `ClosureRegionRequirements` are met. The requirements from
    /// `ClosureRegionRequirements` are expressed in terms of
    /// `RegionVid` entries that map into the returned vector `V`: so
    /// if the `ClosureRegionRequirements` contains something like
    /// `'1: '2`, then the caller would impose the constraint that
    /// `V[1]: V[2]`.
    pub fn closure_mapping(
        tcx: TyCtxt<'tcx>,
        closure_substs: SubstsRef<'tcx>,
        expected_num_vars: usize,
        closure_def_id: LocalDefId,
    ) -> IndexVec<RegionVid, ty::Region<'tcx>> {
        let mut region_mapping = IndexVec::with_capacity(expected_num_vars);
        region_mapping.push(tcx.lifetimes.re_static);
        tcx.for_each_free_region(&closure_substs, |fr| {
            region_mapping.push(fr);
        });

        for_each_late_bound_region_in_recursive_scope(tcx, tcx.local_parent(closure_def_id), |r| {
            region_mapping.push(r);
        });

        assert_eq!(
            region_mapping.len(),
            expected_num_vars,
            "index vec had unexpected number of variables"
        );

        region_mapping
    }

    /// Returns `true` if `r` is a member of this set of universal regions.
    pub fn is_universal_region(&self, r: RegionVid) -> bool {
        (FIRST_GLOBAL_INDEX..self.num_universals).contains(&r.index())
    }

    /// Classifies `r` as a universal region, returning `None` if this
    /// is not a member of this set of universal regions.
    pub fn region_classification(&self, r: RegionVid) -> Option<RegionClassification> {
        let index = r.index();
        if (FIRST_GLOBAL_INDEX..self.first_extern_index).contains(&index) {
            Some(RegionClassification::Global)
        } else if (self.first_extern_index..self.first_local_index).contains(&index) {
            Some(RegionClassification::External)
        } else if (self.first_local_index..self.num_universals).contains(&index) {
            Some(RegionClassification::Local)
        } else {
            None
        }
    }

    /// Returns an iterator over all the RegionVids corresponding to
    /// universally quantified free regions.
    pub fn universal_regions(&self) -> impl Iterator<Item = RegionVid> {
        (FIRST_GLOBAL_INDEX..self.num_universals).map(RegionVid::from_usize)
    }

    /// Returns `true` if `r` is classified as an local region.
    pub fn is_local_free_region(&self, r: RegionVid) -> bool {
        self.region_classification(r) == Some(RegionClassification::Local)
    }

    /// Returns the number of universal regions created in any category.
    pub fn len(&self) -> usize {
        self.num_universals
    }

    /// Returns the number of global plus external universal regions.
    /// For closures, these are the regions that appear free in the
    /// closure type (versus those bound in the closure
    /// signature). They are therefore the regions between which the
    /// closure may impose constraints that its creator must verify.
    pub fn num_global_and_external_regions(&self) -> usize {
        self.first_local_index
    }

    /// Gets an iterator over all the early-bound regions that have names.
    /// Iteration order may be unstable, so this should only be used when
    /// iteration order doesn't affect anything
    #[allow(rustc::potential_query_instability)]
    pub fn named_universal_regions<'s>(
        &'s self,
    ) -> impl Iterator<Item = (ty::Region<'tcx>, ty::RegionVid)> + 's {
        self.indices.indices.iter().map(|(&r, &v)| (r, v))
    }

    /// See `UniversalRegionIndices::to_region_vid`.
    pub fn to_region_vid(&self, r: ty::Region<'tcx>) -> RegionVid {
        self.indices.to_region_vid(r)
    }

    /// As part of the NLL unit tests, you can annotate a function with
    /// `#[rustc_regions]`, and we will emit information about the region
    /// inference context and -- in particular -- the external constraints
    /// that this region imposes on others. The methods in this file
    /// handle the part about dumping the inference context internal
    /// state.
    pub(crate) fn annotate(&self, tcx: TyCtxt<'tcx>, err: &mut Diagnostic) {
        match self.defining_ty {
            DefiningTy::Closure(def_id, substs) => {
                err.note(format!(
                    "defining type: {} with closure substs {:#?}",
                    tcx.def_path_str_with_substs(def_id, substs),
                    &substs[tcx.generics_of(def_id).parent_count..],
                ));

                // FIXME: It'd be nice to print the late-bound regions
                // here, but unfortunately these wind up stored into
                // tests, and the resulting print-outs include def-ids
                // and other things that are not stable across tests!
                // So we just include the region-vid. Annoying.
                for_each_late_bound_region_in_recursive_scope(tcx, def_id.expect_local(), |r| {
                    err.note(format!("late-bound region is {:?}", self.to_region_vid(r)));
                });
            }
            DefiningTy::Generator(def_id, substs, _) => {
                err.note(format!(
                    "defining type: {} with generator substs {:#?}",
                    tcx.def_path_str_with_substs(def_id, substs),
                    &substs[tcx.generics_of(def_id).parent_count..],
                ));

                // FIXME: As above, we'd like to print out the region
                // `r` but doing so is not stable across architectures
                // and so forth.
                for_each_late_bound_region_in_recursive_scope(tcx, def_id.expect_local(), |r| {
                    err.note(format!("late-bound region is {:?}", self.to_region_vid(r)));
                });
            }
            DefiningTy::FnDef(def_id, substs) => {
                err.note(format!(
                    "defining type: {}",
                    tcx.def_path_str_with_substs(def_id, substs),
                ));
            }
            DefiningTy::Const(def_id, substs) => {
                err.note(format!(
                    "defining constant type: {}",
                    tcx.def_path_str_with_substs(def_id, substs),
                ));
            }
            DefiningTy::InlineConst(def_id, substs) => {
                err.note(format!(
                    "defining inline constant type: {}",
                    tcx.def_path_str_with_substs(def_id, substs),
                ));
            }
        }
    }
}

struct UniversalRegionsBuilder<'cx, 'tcx> {
    infcx: &'cx BorrowckInferCtxt<'cx, 'tcx>,
    mir_def: LocalDefId,
    param_env: ty::ParamEnv<'tcx>,
}

const FR: NllRegionVariableOrigin = NllRegionVariableOrigin::FreeRegion;

impl<'cx, 'tcx> UniversalRegionsBuilder<'cx, 'tcx> {
    fn build(self) -> UniversalRegions<'tcx> {
        debug!("build(mir_def={:?})", self.mir_def);

        let param_env = self.param_env;
        debug!("build: param_env={:?}", param_env);

        assert_eq!(FIRST_GLOBAL_INDEX, self.infcx.num_region_vars());

        // Create the "global" region that is always free in all contexts: 'static.
        let fr_static =
            self.infcx.next_nll_region_var(FR, || RegionCtxt::Free(kw::Static)).as_var();

        // We've now added all the global regions. The next ones we
        // add will be external.
        let first_extern_index = self.infcx.num_region_vars();

        let defining_ty = self.defining_ty();
        debug!("build: defining_ty={:?}", defining_ty);

        let mut indices = self.compute_indices(fr_static, defining_ty);
        debug!("build: indices={:?}", indices);

        let typeck_root_def_id = self.infcx.tcx.typeck_root_def_id(self.mir_def.to_def_id());

        // If this is a 'root' body (not a closure/generator/inline const), then
        // there are no extern regions, so the local regions start at the same
        // position as the (empty) sub-list of extern regions
        let first_local_index = if self.mir_def.to_def_id() == typeck_root_def_id {
            first_extern_index
        } else {
            // If this is a closure, generator, or inline-const, then the late-bound regions from the enclosing
            // function/closures are actually external regions to us. For example, here, 'a is not local
            // to the closure c (although it is local to the fn foo):
            // fn foo<'a>() {
            //     let c = || { let x: &'a u32 = ...; }
            // }
            for_each_late_bound_region_in_recursive_scope(
                self.infcx.tcx,
                self.infcx.tcx.local_parent(self.mir_def),
                |r| {
                    debug!(?r);
                    if !indices.indices.contains_key(&r) {
                        let region_vid = {
                            let name = r.get_name_or_anon();
                            self.infcx.next_nll_region_var(FR, || {
                                RegionCtxt::LateBound(BoundRegionInfo::Name(name))
                            })
                        };

                        debug!(?region_vid);
                        indices.insert_late_bound_region(r, region_vid.as_var());
                    }
                },
            );

            // Any regions created during the execution of `defining_ty` or during the above
            // late-bound region replacement are all considered 'extern' regions
            self.infcx.num_region_vars()
        };

        // "Liberate" the late-bound regions. These correspond to
        // "local" free regions.

        let bound_inputs_and_output = self.compute_inputs_and_output(&indices, defining_ty);

        let inputs_and_output = self.infcx.replace_bound_regions_with_nll_infer_vars(
            FR,
            self.mir_def,
            bound_inputs_and_output,
            &mut indices,
        );
        // Converse of above, if this is a function/closure then the late-bound regions declared on its
        // signature are local.
        for_each_late_bound_region_in_item(self.infcx.tcx, self.mir_def, |r| {
            debug!(?r);
            if !indices.indices.contains_key(&r) {
                let region_vid = {
                    let name = r.get_name_or_anon();
                    self.infcx.next_nll_region_var(FR, || {
                        RegionCtxt::LateBound(BoundRegionInfo::Name(name))
                    })
                };

                debug!(?region_vid);
                indices.insert_late_bound_region(r, region_vid.as_var());
            }
        });

        let (unnormalized_output_ty, mut unnormalized_input_tys) =
            inputs_and_output.split_last().unwrap();

        // C-variadic fns also have a `VaList` input that's not listed in the signature
        // (as it's created inside the body itself, not passed in from outside).
        if let DefiningTy::FnDef(def_id, _) = defining_ty {
            if self.infcx.tcx.fn_sig(def_id).skip_binder().c_variadic() {
                let va_list_did = self.infcx.tcx.require_lang_item(
                    LangItem::VaList,
                    Some(self.infcx.tcx.def_span(self.mir_def)),
                );

                let reg_vid = self
                    .infcx
                    .next_nll_region_var(FR, || RegionCtxt::Free(Symbol::intern("c-variadic")))
                    .as_var();

                let region = ty::Region::new_var(self.infcx.tcx, reg_vid);
                let va_list_ty =
                    self.infcx.tcx.type_of(va_list_did).subst(self.infcx.tcx, &[region.into()]);

                unnormalized_input_tys = self.infcx.tcx.mk_type_list_from_iter(
                    unnormalized_input_tys.iter().copied().chain(iter::once(va_list_ty)),
                );
            }
        }

        let fr_fn_body = self
            .infcx
            .next_nll_region_var(FR, || RegionCtxt::Free(Symbol::intern("fn_body")))
            .as_var();

        let num_universals = self.infcx.num_region_vars();

        debug!("build: global regions = {}..{}", FIRST_GLOBAL_INDEX, first_extern_index);
        debug!("build: extern regions = {}..{}", first_extern_index, first_local_index);
        debug!("build: local regions  = {}..{}", first_local_index, num_universals);

        let yield_ty = match defining_ty {
            DefiningTy::Generator(_, substs, _) => Some(substs.as_generator().yield_ty()),
            _ => None,
        };

        UniversalRegions {
            indices,
            fr_static,
            fr_fn_body,
            first_extern_index,
            first_local_index,
            num_universals,
            defining_ty,
            unnormalized_output_ty: *unnormalized_output_ty,
            unnormalized_input_tys,
            yield_ty,
        }
    }

    /// Returns the "defining type" of the current MIR;
    /// see `DefiningTy` for details.
    fn defining_ty(&self) -> DefiningTy<'tcx> {
        let tcx = self.infcx.tcx;
        let typeck_root_def_id = tcx.typeck_root_def_id(self.mir_def.to_def_id());

        match tcx.hir().body_owner_kind(self.mir_def) {
            BodyOwnerKind::Closure | BodyOwnerKind::Fn => {
                let defining_ty = tcx.type_of(self.mir_def).subst_identity();

                debug!("defining_ty (pre-replacement): {:?}", defining_ty);

                let defining_ty =
                    self.infcx.replace_free_regions_with_nll_infer_vars(FR, defining_ty);

                match *defining_ty.kind() {
                    ty::Closure(def_id, substs) => DefiningTy::Closure(def_id, substs),
                    ty::Generator(def_id, substs, movability) => {
                        DefiningTy::Generator(def_id, substs, movability)
                    }
                    ty::FnDef(def_id, substs) => DefiningTy::FnDef(def_id, substs),
                    _ => span_bug!(
                        tcx.def_span(self.mir_def),
                        "expected defining type for `{:?}`: `{:?}`",
                        self.mir_def,
                        defining_ty
                    ),
                }
            }

            BodyOwnerKind::Const | BodyOwnerKind::Static(..) => {
                let identity_substs = InternalSubsts::identity_for_item(tcx, typeck_root_def_id);
                if self.mir_def.to_def_id() == typeck_root_def_id {
                    let substs =
                        self.infcx.replace_free_regions_with_nll_infer_vars(FR, identity_substs);
                    DefiningTy::Const(self.mir_def.to_def_id(), substs)
                } else {
                    // FIXME this line creates a dependency between borrowck and typeck.
                    //
                    // This is required for `AscribeUserType` canonical query, which will call
                    // `type_of(inline_const_def_id)`. That `type_of` would inject erased lifetimes
                    // into borrowck, which is ICE #78174.
                    //
                    // As a workaround, inline consts have an additional generic param (`ty`
                    // below), so that `type_of(inline_const_def_id).substs(substs)` uses the
                    // proper type with NLL infer vars.
                    let ty = tcx
                        .typeck(self.mir_def)
                        .node_type(tcx.local_def_id_to_hir_id(self.mir_def));
                    let substs = InlineConstSubsts::new(
                        tcx,
                        InlineConstSubstsParts { parent_substs: identity_substs, ty },
                    )
                    .substs;
                    let substs = self.infcx.replace_free_regions_with_nll_infer_vars(FR, substs);
                    DefiningTy::InlineConst(self.mir_def.to_def_id(), substs)
                }
            }
        }
    }

    /// Builds a hashmap that maps from the universal regions that are
    /// in scope (as a `ty::Region<'tcx>`) to their indices (as a
    /// `RegionVid`). The map returned by this function contains only
    /// the early-bound regions.
    fn compute_indices(
        &self,
        fr_static: RegionVid,
        defining_ty: DefiningTy<'tcx>,
    ) -> UniversalRegionIndices<'tcx> {
        let tcx = self.infcx.tcx;
        let typeck_root_def_id = tcx.typeck_root_def_id(self.mir_def.to_def_id());
        let identity_substs = InternalSubsts::identity_for_item(tcx, typeck_root_def_id);
        let fr_substs = match defining_ty {
            DefiningTy::Closure(_, substs)
            | DefiningTy::Generator(_, substs, _)
            | DefiningTy::InlineConst(_, substs) => {
                // In the case of closures, we rely on the fact that
                // the first N elements in the ClosureSubsts are
                // inherited from the `typeck_root_def_id`.
                // Therefore, when we zip together (below) with
                // `identity_substs`, we will get only those regions
                // that correspond to early-bound regions declared on
                // the `typeck_root_def_id`.
                assert!(substs.len() >= identity_substs.len());
                assert_eq!(substs.regions().count(), identity_substs.regions().count());
                substs
            }

            DefiningTy::FnDef(_, substs) | DefiningTy::Const(_, substs) => substs,
        };

        let global_mapping = iter::once((tcx.lifetimes.re_static, fr_static));
        let subst_mapping =
            iter::zip(identity_substs.regions(), fr_substs.regions().map(|r| r.as_var()));

        UniversalRegionIndices { indices: global_mapping.chain(subst_mapping).collect(), fr_static }
    }

    fn compute_inputs_and_output(
        &self,
        indices: &UniversalRegionIndices<'tcx>,
        defining_ty: DefiningTy<'tcx>,
    ) -> ty::Binder<'tcx, &'tcx ty::List<Ty<'tcx>>> {
        let tcx = self.infcx.tcx;
        match defining_ty {
            DefiningTy::Closure(def_id, substs) => {
                assert_eq!(self.mir_def.to_def_id(), def_id);
                let closure_sig = substs.as_closure().sig();
                let inputs_and_output = closure_sig.inputs_and_output();
                let bound_vars = tcx.mk_bound_variable_kinds_from_iter(
                    inputs_and_output
                        .bound_vars()
                        .iter()
                        .chain(iter::once(ty::BoundVariableKind::Region(ty::BrEnv))),
                );
                let br = ty::BoundRegion {
                    var: ty::BoundVar::from_usize(bound_vars.len() - 1),
                    kind: ty::BrEnv,
                };
                let env_region = ty::Region::new_late_bound(tcx, ty::INNERMOST, br);
                let closure_ty = tcx.closure_env_ty(def_id, substs, env_region).unwrap();

                // The "inputs" of the closure in the
                // signature appear as a tuple. The MIR side
                // flattens this tuple.
                let (&output, tuplized_inputs) =
                    inputs_and_output.skip_binder().split_last().unwrap();
                assert_eq!(tuplized_inputs.len(), 1, "multiple closure inputs");
                let &ty::Tuple(inputs) = tuplized_inputs[0].kind() else {
                    bug!("closure inputs not a tuple: {:?}", tuplized_inputs[0]);
                };

                ty::Binder::bind_with_vars(
                    tcx.mk_type_list_from_iter(
                        iter::once(closure_ty).chain(inputs).chain(iter::once(output)),
                    ),
                    bound_vars,
                )
            }

            DefiningTy::Generator(def_id, substs, movability) => {
                assert_eq!(self.mir_def.to_def_id(), def_id);
                let resume_ty = substs.as_generator().resume_ty();
                let output = substs.as_generator().return_ty();
                let generator_ty = Ty::new_generator(tcx, def_id, substs, movability);
                let inputs_and_output =
                    self.infcx.tcx.mk_type_list(&[generator_ty, resume_ty, output]);
                ty::Binder::dummy(inputs_and_output)
            }

            DefiningTy::FnDef(def_id, _) => {
                let sig = tcx.fn_sig(def_id).subst_identity();
                let sig = indices.fold_to_region_vids(tcx, sig);
                sig.inputs_and_output()
            }

            DefiningTy::Const(def_id, _) => {
                // For a constant body, there are no inputs, and one
                // "output" (the type of the constant).
                assert_eq!(self.mir_def.to_def_id(), def_id);
                let ty = tcx.type_of(self.mir_def).subst_identity();
                let ty = indices.fold_to_region_vids(tcx, ty);
                ty::Binder::dummy(tcx.mk_type_list(&[ty]))
            }

            DefiningTy::InlineConst(def_id, substs) => {
                assert_eq!(self.mir_def.to_def_id(), def_id);
                let ty = substs.as_inline_const().ty();
                ty::Binder::dummy(tcx.mk_type_list(&[ty]))
            }
        }
    }
}

trait InferCtxtExt<'tcx> {
    fn replace_free_regions_with_nll_infer_vars<T>(
        &self,
        origin: NllRegionVariableOrigin,
        value: T,
    ) -> T
    where
        T: TypeFoldable<TyCtxt<'tcx>>;

    fn replace_bound_regions_with_nll_infer_vars<T>(
        &self,
        origin: NllRegionVariableOrigin,
        all_outlive_scope: LocalDefId,
        value: ty::Binder<'tcx, T>,
        indices: &mut UniversalRegionIndices<'tcx>,
    ) -> T
    where
        T: TypeFoldable<TyCtxt<'tcx>>;

    fn replace_late_bound_regions_with_nll_infer_vars_in_recursive_scope(
        &self,
        mir_def_id: LocalDefId,
        indices: &mut UniversalRegionIndices<'tcx>,
    );

    fn replace_late_bound_regions_with_nll_infer_vars_in_item(
        &self,
        mir_def_id: LocalDefId,
        indices: &mut UniversalRegionIndices<'tcx>,
    );
}

impl<'cx, 'tcx> InferCtxtExt<'tcx> for BorrowckInferCtxt<'cx, 'tcx> {
    #[instrument(skip(self), level = "debug")]
    fn replace_free_regions_with_nll_infer_vars<T>(
        &self,
        origin: NllRegionVariableOrigin,
        value: T,
    ) -> T
    where
        T: TypeFoldable<TyCtxt<'tcx>>,
    {
        self.infcx.tcx.fold_regions(value, |region, _depth| {
            let name = region.get_name_or_anon();
            debug!(?region, ?name);

            self.next_nll_region_var(origin, || RegionCtxt::Free(name))
        })
    }

    #[instrument(level = "debug", skip(self, indices))]
    fn replace_bound_regions_with_nll_infer_vars<T>(
        &self,
        origin: NllRegionVariableOrigin,
        all_outlive_scope: LocalDefId,
        value: ty::Binder<'tcx, T>,
        indices: &mut UniversalRegionIndices<'tcx>,
    ) -> T
    where
        T: TypeFoldable<TyCtxt<'tcx>>,
    {
        let (value, _map) = self.tcx.replace_late_bound_regions(value, |br| {
            debug!(?br);
            let liberated_region =
                ty::Region::new_free(self.tcx, all_outlive_scope.to_def_id(), br.kind);
            let region_vid = {
                let name = match br.kind.get_name() {
                    Some(name) => name,
                    _ => sym::anon,
                };

                self.next_nll_region_var(origin, || RegionCtxt::Bound(BoundRegionInfo::Name(name)))
            };

            indices.insert_late_bound_region(liberated_region, region_vid.as_var());
            debug!(?liberated_region, ?region_vid);
            region_vid
        });
        value
    }

    /// Finds late-bound regions that do not appear in the parameter listing and adds them to the
    /// indices vector. Typically, we identify late-bound regions as we process the inputs and
    /// outputs of the closure/function. However, sometimes there are late-bound regions which do
    /// not appear in the fn parameters but which are nonetheless in scope. The simplest case of
    /// this are unused functions, like fn foo<'a>() { } (see e.g., #51351). Despite not being used,
    /// users can still reference these regions (e.g., let x: &'a u32 = &22;), so we need to create
    /// entries for them and store them in the indices map. This code iterates over the complete
    /// set of late-bound regions and checks for any that we have not yet seen, adding them to the
    /// inputs vector.
    #[instrument(skip(self, indices))]
    fn replace_late_bound_regions_with_nll_infer_vars_in_recursive_scope(
        &self,
        mir_def_id: LocalDefId,
        indices: &mut UniversalRegionIndices<'tcx>,
    ) {
        for_each_late_bound_region_in_recursive_scope(self.tcx, mir_def_id, |r| {
            debug!(?r);
            if !indices.indices.contains_key(&r) {
                let region_vid = {
                    let name = r.get_name_or_anon();
                    self.next_nll_region_var(FR, || {
                        RegionCtxt::LateBound(BoundRegionInfo::Name(name))
                    })
                };

                debug!(?region_vid);
                indices.insert_late_bound_region(r, region_vid.as_var());
            }
        });
    }

    #[instrument(skip(self, indices))]
    fn replace_late_bound_regions_with_nll_infer_vars_in_item(
        &self,
        mir_def_id: LocalDefId,
        indices: &mut UniversalRegionIndices<'tcx>,
    ) {
        for_each_late_bound_region_in_item(self.tcx, mir_def_id, |r| {
            debug!(?r);
            if !indices.indices.contains_key(&r) {
                let region_vid = {
                    let name = r.get_name_or_anon();
                    self.next_nll_region_var(FR, || {
                        RegionCtxt::LateBound(BoundRegionInfo::Name(name))
                    })
                };

                indices.insert_late_bound_region(r, region_vid.as_var());
            }
        });
    }
}

impl<'tcx> UniversalRegionIndices<'tcx> {
    /// Initially, the `UniversalRegionIndices` map contains only the
    /// early-bound regions in scope. Once that is all setup, we come
    /// in later and instantiate the late-bound regions, and then we
    /// insert the `ReFree` version of those into the map as
    /// well. These are used for error reporting.
    fn insert_late_bound_region(&mut self, r: ty::Region<'tcx>, vid: ty::RegionVid) {
        debug!("insert_late_bound_region({:?}, {:?})", r, vid);
        self.indices.insert(r, vid);
    }

    /// Converts `r` into a local inference variable: `r` can either
    /// be a `ReVar` (i.e., already a reference to an inference
    /// variable) or it can be `'static` or some early-bound
    /// region. This is useful when taking the results from
    /// type-checking and trait-matching, which may sometimes
    /// reference those regions from the `ParamEnv`. It is also used
    /// during initialization. Relies on the `indices` map having been
    /// fully initialized.
    pub fn to_region_vid(&self, r: ty::Region<'tcx>) -> RegionVid {
        if let ty::ReVar(..) = *r {
            r.as_var()
        } else if r.is_error() {
            // We use the `'static` `RegionVid` because `ReError` doesn't actually exist in the
            // `UniversalRegionIndices`. This is fine because 1) it is a fallback only used if
            // errors are being emitted and 2) it leaves the happy path unaffected.
            self.fr_static
        } else {
            *self
                .indices
                .get(&r)
                .unwrap_or_else(|| bug!("cannot convert `{:?}` to a region vid", r))
        }
    }

    /// Replaces all free regions in `value` with region vids, as
    /// returned by `to_region_vid`.
    pub fn fold_to_region_vids<T>(&self, tcx: TyCtxt<'tcx>, value: T) -> T
    where
        T: TypeFoldable<TyCtxt<'tcx>>,
    {
        tcx.fold_regions(value, |region, _| ty::Region::new_var(tcx, self.to_region_vid(region)))
    }
}

/// Iterates over the late-bound regions defined on `mir_def_id` and all of its
/// parents, up to the typeck root, and invokes `f` with the liberated form
/// of each one.
fn for_each_late_bound_region_in_recursive_scope<'tcx>(
    tcx: TyCtxt<'tcx>,
    mut mir_def_id: LocalDefId,
    mut f: impl FnMut(ty::Region<'tcx>),
) {
    let typeck_root_def_id = tcx.typeck_root_def_id(mir_def_id.to_def_id());

    // Walk up the tree, collecting late-bound regions until we hit the typeck root
    loop {
        for_each_late_bound_region_in_item(tcx, mir_def_id, &mut f);

        if mir_def_id.to_def_id() == typeck_root_def_id {
            break;
        } else {
            mir_def_id = tcx.local_parent(mir_def_id);
        }
    }
}

/// Iterates over the late-bound regions defined on `mir_def_id` and all of its
/// parents, up to the typeck root, and invokes `f` with the liberated form
/// of each one.
fn for_each_late_bound_region_in_item<'tcx>(
    tcx: TyCtxt<'tcx>,
    mir_def_id: LocalDefId,
    mut f: impl FnMut(ty::Region<'tcx>),
) {
    if !tcx.def_kind(mir_def_id).is_fn_like() {
        return;
    }

    for bound_var in tcx.late_bound_vars(tcx.hir().local_def_id_to_hir_id(mir_def_id)) {
        let ty::BoundVariableKind::Region(bound_region) = bound_var else { continue; };
        let liberated_region = ty::Region::new_free(tcx, mir_def_id.to_def_id(), bound_region);
        f(liberated_region);
    }
}
