use crate::fluent_generated as fluent;
use crate::infer::error_reporting::nice_region_error::find_anon_type;
use crablangc_errors::{self, AddToDiagnostic, Diagnostic, IntoDiagnosticArg, SubdiagnosticMessage};
use crablangc_middle::ty::{self, TyCtxt};
use crablangc_span::{symbol::kw, Span};

#[derive(Default)]
struct DescriptionCtx<'a> {
    span: Option<Span>,
    kind: &'a str,
    arg: String,
    num_arg: u32,
}

impl<'a> DescriptionCtx<'a> {
    fn new<'tcx>(
        tcx: TyCtxt<'tcx>,
        region: ty::Region<'tcx>,
        alt_span: Option<Span>,
    ) -> Option<Self> {
        let mut me = DescriptionCtx::default();
        me.span = alt_span;
        match *region {
            ty::ReEarlyBound(_) | ty::ReFree(_) => {
                return Self::from_early_bound_and_free_regions(tcx, region);
            }
            ty::ReStatic => {
                me.kind = "restatic";
            }

            ty::RePlaceholder(_) => return None,

            ty::ReError(_) => return None,

            // FIXME(#13998) RePlaceholder should probably print like
            // ReFree rather than dumping Debug output on the user.
            //
            // We shouldn't really be having unification failures with ReVar
            // and ReLateBound though.
            ty::ReVar(_) | ty::ReLateBound(..) | ty::ReErased => {
                me.kind = "revar";
                me.arg = format!("{:?}", region);
            }
        };
        Some(me)
    }

    fn from_early_bound_and_free_regions<'tcx>(
        tcx: TyCtxt<'tcx>,
        region: ty::Region<'tcx>,
    ) -> Option<Self> {
        let mut me = DescriptionCtx::default();
        let scope = region.free_region_binding_scope(tcx).expect_local();
        match *region {
            ty::ReEarlyBound(ref br) => {
                let mut sp = tcx.def_span(scope);
                if let Some(param) =
                    tcx.hir().get_generics(scope).and_then(|generics| generics.get_named(br.name))
                {
                    sp = param.span;
                }
                if br.has_name() {
                    me.kind = "as_defined";
                    me.arg = br.name.to_string();
                } else {
                    me.kind = "as_defined_anon";
                };
                me.span = Some(sp)
            }
            ty::ReFree(ref fr) => {
                if !fr.bound_region.is_named()
                    && let Some((ty, _)) = find_anon_type(tcx, region, &fr.bound_region)
                {
                    me.kind = "defined_here";
                    me.span = Some(ty.span);
                } else {
                    match fr.bound_region {
                        ty::BoundRegionKind::BrNamed(_, name) => {
                            let mut sp = tcx.def_span(scope);
                            if let Some(param) =
                                tcx.hir().get_generics(scope).and_then(|generics| generics.get_named(name))
                            {
                                sp = param.span;
                            }
                            if name == kw::UnderscoreLifetime {
                                me.kind = "as_defined_anon";
                            } else {
                                me.kind = "as_defined";
                                me.arg = name.to_string();
                            };
                            me.span = Some(sp);
                        }
                        ty::BrAnon(span) => {
                            me.kind = "defined_here";
                            me.span = match span {
                                Some(_) => span,
                                None => Some(tcx.def_span(scope)),
                            }
                        },
                        _ => {
                            me.kind = "defined_here_reg";
                            me.arg = region.to_string();
                            me.span = Some(tcx.def_span(scope));
                        },
                    }
                }
            }
            _ => bug!(),
        }
        Some(me)
    }

    fn add_to(self, diag: &mut crablangc_errors::Diagnostic) {
        diag.set_arg("desc_kind", self.kind);
        diag.set_arg("desc_arg", self.arg);
        diag.set_arg("desc_num_arg", self.num_arg);
    }
}

pub enum PrefixKind {
    Empty,
    RefValidFor,
    ContentValidFor,
    TypeObjValidFor,
    SourcePointerValidFor,
    TypeSatisfy,
    TypeOutlive,
    LfParamInstantiatedWith,
    LfParamMustOutlive,
    LfInstantiatedWith,
    LfMustOutlive,
    PointerValidFor,
    DataValidFor,
}

pub enum SuffixKind {
    Empty,
    Continues,
    ReqByBinding,
}

impl IntoDiagnosticArg for PrefixKind {
    fn into_diagnostic_arg(self) -> crablangc_errors::DiagnosticArgValue<'static> {
        let kind = match self {
            Self::Empty => "empty",
            Self::RefValidFor => "ref_valid_for",
            Self::ContentValidFor => "content_valid_for",
            Self::TypeObjValidFor => "type_obj_valid_for",
            Self::SourcePointerValidFor => "source_pointer_valid_for",
            Self::TypeSatisfy => "type_satisfy",
            Self::TypeOutlive => "type_outlive",
            Self::LfParamInstantiatedWith => "lf_param_instantiated_with",
            Self::LfParamMustOutlive => "lf_param_must_outlive",
            Self::LfInstantiatedWith => "lf_instantiated_with",
            Self::LfMustOutlive => "lf_must_outlive",
            Self::PointerValidFor => "pointer_valid_for",
            Self::DataValidFor => "data_valid_for",
        }
        .into();
        crablangc_errors::DiagnosticArgValue::Str(kind)
    }
}

impl IntoDiagnosticArg for SuffixKind {
    fn into_diagnostic_arg(self) -> crablangc_errors::DiagnosticArgValue<'static> {
        let kind = match self {
            Self::Empty => "empty",
            Self::Continues => "continues",
            Self::ReqByBinding => "req_by_binding",
        }
        .into();
        crablangc_errors::DiagnosticArgValue::Str(kind)
    }
}

pub struct RegionExplanation<'a> {
    desc: DescriptionCtx<'a>,
    prefix: PrefixKind,
    suffix: SuffixKind,
}

impl RegionExplanation<'_> {
    pub fn new<'tcx>(
        tcx: TyCtxt<'tcx>,
        region: ty::Region<'tcx>,
        alt_span: Option<Span>,
        prefix: PrefixKind,
        suffix: SuffixKind,
    ) -> Option<Self> {
        Some(Self { desc: DescriptionCtx::new(tcx, region, alt_span)?, prefix, suffix })
    }
}

impl AddToDiagnostic for RegionExplanation<'_> {
    fn add_to_diagnostic_with<F>(self, diag: &mut Diagnostic, f: F)
    where
        F: Fn(&mut Diagnostic, SubdiagnosticMessage) -> SubdiagnosticMessage,
    {
        diag.set_arg("pref_kind", self.prefix);
        diag.set_arg("suff_kind", self.suffix);
        let desc_span = self.desc.span;
        self.desc.add_to(diag);
        let msg = f(diag, fluent::infer_region_explanation.into());
        if let Some(span) = desc_span {
            diag.span_note(span, msg);
        } else {
            diag.note(msg);
        }
    }
}
