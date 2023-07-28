use rustc_errors::{Applicability, Diagnostic, DiagnosticBuilder, ErrorGuaranteed};
use rustc_middle::mir::*;
use rustc_middle::ty;
use rustc_mir_dataflow::move_paths::{
    IllegalMoveOrigin, IllegalMoveOriginKind, LookupResult, MoveError, MovePathIndex,
};
use rustc_span::{BytePos, Span};

use crate::diagnostics::CapturedMessageOpt;
use crate::diagnostics::{DescribePlaceOpt, UseSpans};
use crate::prefixes::PrefixSet;
use crate::MirBorrowckCtxt;

// Often when desugaring a pattern match we may have many individual moves in
// MIR that are all part of one operation from the user's point-of-view. For
// example:
//
// let (x, y) = foo()
//
// would move x from the 0 field of some temporary, and y from the 1 field. We
// group such errors together for cleaner error reporting.
//
// Errors are kept separate if they are from places with different parent move
// paths. For example, this generates two errors:
//
// let (&x, &y) = (&String::new(), &String::new());
#[derive(Debug)]
enum GroupedMoveError<'tcx> {
    // Place expression can't be moved from,
    // e.g., match x[0] { s => (), } where x: &[String]
    MovesFromPlace {
        original_path: Place<'tcx>,
        span: Span,
        move_from: Place<'tcx>,
        kind: IllegalMoveOriginKind<'tcx>,
        binds_to: Vec<Local>,
    },
    // Part of a value expression can't be moved from,
    // e.g., match &String::new() { &x => (), }
    MovesFromValue {
        original_path: Place<'tcx>,
        span: Span,
        move_from: MovePathIndex,
        kind: IllegalMoveOriginKind<'tcx>,
        binds_to: Vec<Local>,
    },
    // Everything that isn't from pattern matching.
    OtherIllegalMove {
        original_path: Place<'tcx>,
        use_spans: UseSpans<'tcx>,
        kind: IllegalMoveOriginKind<'tcx>,
    },
}

impl<'a, 'tcx> MirBorrowckCtxt<'a, 'tcx> {
    pub(crate) fn report_move_errors(&mut self, move_errors: Vec<(Place<'tcx>, MoveError<'tcx>)>) {
        let grouped_errors = self.group_move_errors(move_errors);
        for error in grouped_errors {
            self.report(error);
        }
    }

    fn group_move_errors(
        &self,
        errors: Vec<(Place<'tcx>, MoveError<'tcx>)>,
    ) -> Vec<GroupedMoveError<'tcx>> {
        let mut grouped_errors = Vec::new();
        for (original_path, error) in errors {
            self.append_to_grouped_errors(&mut grouped_errors, original_path, error);
        }
        grouped_errors
    }

    fn append_to_grouped_errors(
        &self,
        grouped_errors: &mut Vec<GroupedMoveError<'tcx>>,
        original_path: Place<'tcx>,
        error: MoveError<'tcx>,
    ) {
        match error {
            MoveError::UnionMove { .. } => {
                unimplemented!("don't know how to report union move errors yet.")
            }
            MoveError::IllegalMove { cannot_move_out_of: IllegalMoveOrigin { location, kind } } => {
                // Note: that the only time we assign a place isn't a temporary
                // to a user variable is when initializing it.
                // If that ever stops being the case, then the ever initialized
                // flow could be used.
                if let Some(StatementKind::Assign(box (
                    place,
                    Rvalue::Use(Operand::Move(move_from)),
                ))) = self.body.basic_blocks[location.block]
                    .statements
                    .get(location.statement_index)
                    .map(|stmt| &stmt.kind)
                {
                    if let Some(local) = place.as_local() {
                        let local_decl = &self.body.local_decls[local];
                        // opt_match_place is the
                        // match_span is the span of the expression being matched on
                        // match *x.y { ... }        match_place is Some(*x.y)
                        //       ^^^^                match_span is the span of *x.y
                        //
                        // opt_match_place is None for let [mut] x = ... statements,
                        // whether or not the right-hand side is a place expression
                        if let LocalInfo::User(BindingForm::Var(VarBindingForm {
                            opt_match_place: Some((opt_match_place, match_span)),
                            binding_mode: _,
                            opt_ty_info: _,
                            pat_span: _,
                        })) = *local_decl.local_info()
                        {
                            let stmt_source_info = self.body.source_info(location);
                            self.append_binding_error(
                                grouped_errors,
                                kind,
                                original_path,
                                *move_from,
                                local,
                                opt_match_place,
                                match_span,
                                stmt_source_info.span,
                            );
                            return;
                        }
                    }
                }

                let move_spans = self.move_spans(original_path.as_ref(), location);
                grouped_errors.push(GroupedMoveError::OtherIllegalMove {
                    use_spans: move_spans,
                    original_path,
                    kind,
                });
            }
        }
    }

    fn append_binding_error(
        &self,
        grouped_errors: &mut Vec<GroupedMoveError<'tcx>>,
        kind: IllegalMoveOriginKind<'tcx>,
        original_path: Place<'tcx>,
        move_from: Place<'tcx>,
        bind_to: Local,
        match_place: Option<Place<'tcx>>,
        match_span: Span,
        statement_span: Span,
    ) {
        debug!(?match_place, ?match_span, "append_binding_error");

        let from_simple_let = match_place.is_none();
        let match_place = match_place.unwrap_or(move_from);

        match self.move_data.rev_lookup.find(match_place.as_ref()) {
            // Error with the match place
            LookupResult::Parent(_) => {
                for ge in &mut *grouped_errors {
                    if let GroupedMoveError::MovesFromPlace { span, binds_to, .. } = ge
                        && match_span == *span
                    {
                        debug!("appending local({bind_to:?}) to list");
                        if !binds_to.is_empty() {
                            binds_to.push(bind_to);
                        }
                        return;
                    }
                }
                debug!("found a new move error location");

                // Don't need to point to x in let x = ... .
                let (binds_to, span) = if from_simple_let {
                    (vec![], statement_span)
                } else {
                    (vec![bind_to], match_span)
                };
                grouped_errors.push(GroupedMoveError::MovesFromPlace {
                    span,
                    move_from,
                    original_path,
                    kind,
                    binds_to,
                });
            }
            // Error with the pattern
            LookupResult::Exact(_) => {
                let LookupResult::Parent(Some(mpi)) =
                    self.move_data.rev_lookup.find(move_from.as_ref())
                else {
                    // move_from should be a projection from match_place.
                    unreachable!("Probably not unreachable...");
                };
                for ge in &mut *grouped_errors {
                    if let GroupedMoveError::MovesFromValue {
                        span,
                        move_from: other_mpi,
                        binds_to,
                        ..
                    } = ge
                    {
                        if match_span == *span && mpi == *other_mpi {
                            debug!("appending local({bind_to:?}) to list");
                            binds_to.push(bind_to);
                            return;
                        }
                    }
                }
                debug!("found a new move error location");
                grouped_errors.push(GroupedMoveError::MovesFromValue {
                    span: match_span,
                    move_from: mpi,
                    original_path,
                    kind,
                    binds_to: vec![bind_to],
                });
            }
        };
    }

    fn report(&mut self, error: GroupedMoveError<'tcx>) {
        let (mut err, err_span) = {
            let (span, use_spans, original_path, kind) = match error {
                GroupedMoveError::MovesFromPlace { span, original_path, ref kind, .. }
                | GroupedMoveError::MovesFromValue { span, original_path, ref kind, .. } => {
                    (span, None, original_path, kind)
                }
                GroupedMoveError::OtherIllegalMove { use_spans, original_path, ref kind } => {
                    (use_spans.args_or_use(), Some(use_spans), original_path, kind)
                }
            };
            debug!(
                "report: original_path={:?} span={:?}, kind={:?} \
                   original_path.is_upvar_field_projection={:?}",
                original_path,
                span,
                kind,
                self.is_upvar_field_projection(original_path.as_ref())
            );
            (
                match kind {
                    &IllegalMoveOriginKind::BorrowedContent { target_place } => self
                        .report_cannot_move_from_borrowed_content(
                            original_path,
                            target_place,
                            span,
                            use_spans,
                        ),
                    &IllegalMoveOriginKind::InteriorOfTypeWithDestructor { container_ty: ty } => {
                        self.cannot_move_out_of_interior_of_drop(span, ty)
                    }
                    &IllegalMoveOriginKind::InteriorOfSliceOrArray { ty, is_index } => {
                        self.cannot_move_out_of_interior_noncopy(span, ty, Some(is_index))
                    }
                },
                span,
            )
        };

        self.add_move_hints(error, &mut err, err_span);
        self.buffer_error(err);
    }

    fn report_cannot_move_from_static(
        &mut self,
        place: Place<'tcx>,
        span: Span,
    ) -> DiagnosticBuilder<'a, ErrorGuaranteed> {
        let description = if place.projection.len() == 1 {
            format!("static item {}", self.describe_any_place(place.as_ref()))
        } else {
            let base_static = PlaceRef { local: place.local, projection: &[ProjectionElem::Deref] };

            format!(
                "{} as {} is a static item",
                self.describe_any_place(place.as_ref()),
                self.describe_any_place(base_static),
            )
        };

        self.cannot_move_out_of(span, &description)
    }

    fn report_cannot_move_from_borrowed_content(
        &mut self,
        move_place: Place<'tcx>,
        deref_target_place: Place<'tcx>,
        span: Span,
        use_spans: Option<UseSpans<'tcx>>,
    ) -> DiagnosticBuilder<'a, ErrorGuaranteed> {
        // Inspect the type of the content behind the
        // borrow to provide feedback about why this
        // was a move rather than a copy.
        let ty = deref_target_place.ty(self.body, self.infcx.tcx).ty;
        let upvar_field = self
            .prefixes(move_place.as_ref(), PrefixSet::All)
            .find_map(|p| self.is_upvar_field_projection(p));

        let deref_base = match deref_target_place.projection.as_ref() {
            [proj_base @ .., ProjectionElem::Deref] => {
                PlaceRef { local: deref_target_place.local, projection: &proj_base }
            }
            _ => bug!("deref_target_place is not a deref projection"),
        };

        if let PlaceRef { local, projection: [] } = deref_base {
            let decl = &self.body.local_decls[local];
            if decl.is_ref_for_guard() {
                let mut err = self.cannot_move_out_of(
                    span,
                    &format!("`{}` in pattern guard", self.local_names[local].unwrap()),
                );
                err.note(
                    "variables bound in patterns cannot be moved from \
                     until after the end of the pattern guard",
                );
                return err;
            } else if decl.is_ref_to_static() {
                return self.report_cannot_move_from_static(move_place, span);
            }
        }

        debug!("report: ty={:?}", ty);
        let mut err = match ty.kind() {
            ty::Array(..) | ty::Slice(..) => {
                self.cannot_move_out_of_interior_noncopy(span, ty, None)
            }
            ty::Closure(def_id, closure_args)
                if def_id.as_local() == Some(self.mir_def_id()) && upvar_field.is_some() =>
            {
                let closure_kind_ty = closure_args.as_closure().kind_ty();
                let closure_kind = match closure_kind_ty.to_opt_closure_kind() {
                    Some(kind @ (ty::ClosureKind::Fn | ty::ClosureKind::FnMut)) => kind,
                    Some(ty::ClosureKind::FnOnce) => {
                        bug!("closure kind does not match first argument type")
                    }
                    None => bug!("closure kind not inferred by borrowck"),
                };
                let capture_description =
                    format!("captured variable in an `{closure_kind}` closure");

                let upvar = &self.upvars[upvar_field.unwrap().index()];
                let upvar_hir_id = upvar.place.get_root_variable();
                let upvar_name = upvar.place.to_string(self.infcx.tcx);
                let upvar_span = self.infcx.tcx.hir().span(upvar_hir_id);

                let place_name = self.describe_any_place(move_place.as_ref());

                let place_description =
                    if self.is_upvar_field_projection(move_place.as_ref()).is_some() {
                        format!("{place_name}, a {capture_description}")
                    } else {
                        format!("{place_name}, as `{upvar_name}` is a {capture_description}")
                    };

                debug!(
                    "report: closure_kind_ty={:?} closure_kind={:?} place_description={:?}",
                    closure_kind_ty, closure_kind, place_description,
                );

                let mut diag = self.cannot_move_out_of(span, &place_description);

                diag.span_label(upvar_span, "captured outer variable");
                diag.span_label(
                    self.infcx.tcx.def_span(def_id),
                    format!("captured by this `{closure_kind}` closure"),
                );

                diag
            }
            _ => {
                let source = self.borrowed_content_source(deref_base);
                let move_place_ref = move_place.as_ref();
                match (
                    self.describe_place_with_options(
                        move_place_ref,
                        DescribePlaceOpt {
                            including_downcast: false,
                            including_tuple_field: false,
                        },
                    ),
                    self.describe_name(move_place_ref),
                    source.describe_for_named_place(),
                ) {
                    (Some(place_desc), Some(name), Some(source_desc)) => self.cannot_move_out_of(
                        span,
                        &format!("`{place_desc}` as enum variant `{name}` which is behind a {source_desc}"),
                    ),
                    (Some(place_desc), Some(name), None) => self.cannot_move_out_of(
                        span,
                        &format!("`{place_desc}` as enum variant `{name}`"),
                    ),
                    (Some(place_desc), _, Some(source_desc)) => self.cannot_move_out_of(
                        span,
                        &format!("`{place_desc}` which is behind a {source_desc}"),
                    ),
                    (_, _, _) => self.cannot_move_out_of(
                        span,
                        &source.describe_for_unnamed_place(self.infcx.tcx),
                    ),
                }
            }
        };
        let msg_opt = CapturedMessageOpt {
            is_partial_move: false,
            is_loop_message: false,
            is_move_msg: false,
            is_loop_move: false,
            maybe_reinitialized_locations_is_empty: true,
        };
        if let Some(use_spans) = use_spans {
            self.explain_captures(&mut err, span, span, use_spans, move_place, msg_opt);
        }
        err
    }

    fn add_move_hints(&self, error: GroupedMoveError<'tcx>, err: &mut Diagnostic, span: Span) {
        match error {
            GroupedMoveError::MovesFromPlace { mut binds_to, move_from, .. } => {
                self.add_borrow_suggestions(err, span);
                if binds_to.is_empty() {
                    let place_ty = move_from.ty(self.body, self.infcx.tcx).ty;
                    let place_desc = match self.describe_place(move_from.as_ref()) {
                        Some(desc) => format!("`{desc}`"),
                        None => "value".to_string(),
                    };

                    err.subdiagnostic(crate::session_diagnostics::TypeNoCopy::Label {
                        is_partial_move: false,
                        ty: place_ty,
                        place: &place_desc,
                        span,
                    });
                } else {
                    binds_to.sort();
                    binds_to.dedup();

                    self.add_move_error_details(err, &binds_to);
                }
            }
            GroupedMoveError::MovesFromValue { mut binds_to, .. } => {
                binds_to.sort();
                binds_to.dedup();
                self.add_move_error_suggestions(err, &binds_to);
                self.add_move_error_details(err, &binds_to);
            }
            // No binding. Nothing to suggest.
            GroupedMoveError::OtherIllegalMove { ref original_path, use_spans, .. } => {
                let span = use_spans.var_or_use();
                let place_ty = original_path.ty(self.body, self.infcx.tcx).ty;
                let place_desc = match self.describe_place(original_path.as_ref()) {
                    Some(desc) => format!("`{desc}`"),
                    None => "value".to_string(),
                };
                err.subdiagnostic(crate::session_diagnostics::TypeNoCopy::Label {
                    is_partial_move: false,
                    ty: place_ty,
                    place: &place_desc,
                    span,
                });

                use_spans.args_subdiag(err, |args_span| {
                    crate::session_diagnostics::CaptureArgLabel::MoveOutPlace {
                        place: place_desc,
                        args_span,
                    }
                });
            }
        }
    }

    fn add_borrow_suggestions(&self, err: &mut Diagnostic, span: Span) {
        match self.infcx.tcx.sess.source_map().span_to_snippet(span) {
            Ok(snippet) if snippet.starts_with('*') => {
                err.span_suggestion_verbose(
                    span.with_hi(span.lo() + BytePos(1)),
                    "consider removing the dereference here",
                    String::new(),
                    Applicability::MaybeIncorrect,
                );
            }
            _ => {
                err.span_suggestion_verbose(
                    span.shrink_to_lo(),
                    "consider borrowing here",
                    '&',
                    Applicability::MaybeIncorrect,
                );
            }
        }
    }

    fn add_move_error_suggestions(&self, err: &mut Diagnostic, binds_to: &[Local]) {
        let mut suggestions: Vec<(Span, String, String)> = Vec::new();
        for local in binds_to {
            let bind_to = &self.body.local_decls[*local];
            if let LocalInfo::User(BindingForm::Var(VarBindingForm { pat_span, .. })) =
                *bind_to.local_info()
            {
                let Ok(pat_snippet) = self.infcx.tcx.sess.source_map().span_to_snippet(pat_span)
                else {
                    continue;
                };
                let Some(stripped) = pat_snippet.strip_prefix('&') else {
                    suggestions.push((
                        bind_to.source_info.span.shrink_to_lo(),
                        "consider borrowing the pattern binding".to_string(),
                        "ref ".to_string(),
                    ));
                    continue;
                };
                let inner_pat_snippet = stripped.trim_start();
                let (pat_span, suggestion, to_remove) = if inner_pat_snippet.starts_with("mut")
                    && inner_pat_snippet["mut".len()..].starts_with(rustc_lexer::is_whitespace)
                {
                    let inner_pat_snippet = inner_pat_snippet["mut".len()..].trim_start();
                    let pat_span = pat_span.with_hi(
                        pat_span.lo()
                            + BytePos((pat_snippet.len() - inner_pat_snippet.len()) as u32),
                    );
                    (pat_span, String::new(), "mutable borrow")
                } else {
                    let pat_span = pat_span.with_hi(
                        pat_span.lo()
                            + BytePos(
                                (pat_snippet.len() - inner_pat_snippet.trim_start().len()) as u32,
                            ),
                    );
                    (pat_span, String::new(), "borrow")
                };
                suggestions.push((
                    pat_span,
                    format!("consider removing the {to_remove}"),
                    suggestion.to_string(),
                ));
            }
        }
        suggestions.sort_unstable_by_key(|&(span, _, _)| span);
        suggestions.dedup_by_key(|&mut (span, _, _)| span);
        for (span, msg, suggestion) in suggestions {
            err.span_suggestion_verbose(span, msg, suggestion, Applicability::MachineApplicable);
        }
    }

    fn add_move_error_details(&self, err: &mut Diagnostic, binds_to: &[Local]) {
        for (j, local) in binds_to.iter().enumerate() {
            let bind_to = &self.body.local_decls[*local];
            let binding_span = bind_to.source_info.span;

            if j == 0 {
                err.span_label(binding_span, "data moved here");
            } else {
                err.span_label(binding_span, "...and here");
            }

            if binds_to.len() == 1 {
                let place_desc = &format!("`{}`", self.local_names[*local].unwrap());
                err.subdiagnostic(crate::session_diagnostics::TypeNoCopy::Label {
                    is_partial_move: false,
                    ty: bind_to.ty,
                    place: &place_desc,
                    span: binding_span,
                });
            }
        }

        if binds_to.len() > 1 {
            err.note(
                "move occurs because these variables have types that don't implement the `Copy` \
                 trait",
            );
        }
    }
}
