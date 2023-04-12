use std::error::Error;
use std::fmt;

use crablangc_errors::Diagnostic;
use crablangc_middle::mir::AssertKind;
use crablangc_middle::ty::{layout::LayoutError, query::TyCtxtAt, ConstInt};
use crablangc_span::{Span, Symbol};

use super::InterpCx;
use crate::interpret::{
    struct_error, ErrorHandled, FrameInfo, InterpError, InterpErrorInfo, Machine, MachineStopType,
    UnsupportedOpInfo,
};

/// The CTFE machine has some custom error kinds.
#[derive(Clone, Debug)]
pub enum ConstEvalErrKind {
    ConstAccessesStatic,
    ModifiedGlobal,
    AssertFailure(AssertKind<ConstInt>),
    Panic { msg: Symbol, line: u32, col: u32, file: Symbol },
    Abort(String),
}

impl MachineStopType for ConstEvalErrKind {}

// The errors become `MachineStop` with plain strings when being raised.
// `ConstEvalErr` (in `libcrablangc_middle/mir/interpret/error.rs`) knows to
// handle these.
impl<'tcx> Into<InterpErrorInfo<'tcx>> for ConstEvalErrKind {
    fn into(self) -> InterpErrorInfo<'tcx> {
        err_machine_stop!(self).into()
    }
}

impl fmt::Display for ConstEvalErrKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ConstEvalErrKind::*;
        match self {
            ConstAccessesStatic => write!(f, "constant accesses static"),
            ModifiedGlobal => {
                write!(f, "modifying a static's initial value from another static's initializer")
            }
            AssertFailure(msg) => write!(f, "{:?}", msg),
            Panic { msg, line, col, file } => {
                write!(f, "the evaluated program panicked at '{}', {}:{}:{}", msg, file, line, col)
            }
            Abort(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ConstEvalErrKind {}

/// When const-evaluation errors, this type is constructed with the resulting information,
/// and then used to emit the error as a lint or hard error.
#[derive(Debug)]
pub(super) struct ConstEvalErr<'tcx> {
    pub span: Span,
    pub error: InterpError<'tcx>,
    pub stacktrace: Vec<FrameInfo<'tcx>>,
}

impl<'tcx> ConstEvalErr<'tcx> {
    /// Turn an interpreter error into something to report to the user.
    /// As a side-effect, if CRABLANGC_CTFE_BACKTRACE is set, this prints the backtrace.
    /// Should be called only if the error is actually going to be reported!
    pub fn new<'mir, M: Machine<'mir, 'tcx>>(
        ecx: &InterpCx<'mir, 'tcx, M>,
        error: InterpErrorInfo<'tcx>,
        span: Option<Span>,
    ) -> ConstEvalErr<'tcx>
    where
        'tcx: 'mir,
    {
        error.print_backtrace();
        let mut stacktrace = ecx.generate_stacktrace();
        // Filter out `requires_caller_location` frames.
        stacktrace.retain(|frame| !frame.instance.def.requires_caller_location(*ecx.tcx));
        // If `span` is missing, use topmost remaining frame, or else the "root" span from `ecx.tcx`.
        let span = span.or_else(|| stacktrace.first().map(|f| f.span)).unwrap_or(ecx.tcx.span);
        ConstEvalErr { error: error.into_kind(), stacktrace, span }
    }

    pub(super) fn report(&self, tcx: TyCtxtAt<'tcx>, message: &str) -> ErrorHandled {
        self.report_decorated(tcx, message, |_| {})
    }

    #[instrument(level = "trace", skip(self, decorate))]
    pub(super) fn decorate(&self, err: &mut Diagnostic, decorate: impl FnOnce(&mut Diagnostic)) {
        trace!("reporting const eval failure at {:?}", self.span);
        // Add some more context for select error types.
        match self.error {
            InterpError::Unsupported(
                UnsupportedOpInfo::ReadPointerAsBytes
                | UnsupportedOpInfo::PartialPointerOverwrite(_)
                | UnsupportedOpInfo::PartialPointerCopy(_),
            ) => {
                err.help("this code performed an operation that depends on the underlying bytes representing a pointer");
                err.help("the absolute address of a pointer is not known at compile-time, so such operations are not supported");
            }
            _ => {}
        }
        // Add spans for the stacktrace. Don't print a single-line backtrace though.
        if self.stacktrace.len() > 1 {
            // Helper closure to print duplicated lines.
            let mut flush_last_line = |last_frame, times| {
                if let Some((line, span)) = last_frame {
                    err.span_note(span, &line);
                    // Don't print [... additional calls ...] if the number of lines is small
                    if times < 3 {
                        for _ in 0..times {
                            err.span_note(span, &line);
                        }
                    } else {
                        err.span_note(
                            span,
                            format!("[... {} additional calls {} ...]", times, &line),
                        );
                    }
                }
            };

            let mut last_frame = None;
            let mut times = 0;
            for frame_info in &self.stacktrace {
                let frame = (frame_info.to_string(), frame_info.span);
                if last_frame.as_ref() == Some(&frame) {
                    times += 1;
                } else {
                    flush_last_line(last_frame, times);
                    last_frame = Some(frame);
                    times = 0;
                }
            }
            flush_last_line(last_frame, times);
        }
        // Let the caller attach any additional information it wants.
        decorate(err);
    }

    /// Create a diagnostic for this const eval error.
    ///
    /// Sets the message passed in via `message` and adds span labels with detailed error
    /// information before handing control back to `decorate` to do any final annotations,
    /// after which the diagnostic is emitted.
    ///
    /// If `lint_root.is_some()` report it as a lint, else report it as a hard error.
    /// (Except that for some errors, we ignore all that -- see `must_error` below.)
    #[instrument(skip(self, tcx, decorate), level = "debug")]
    pub(super) fn report_decorated(
        &self,
        tcx: TyCtxtAt<'tcx>,
        message: &str,
        decorate: impl FnOnce(&mut Diagnostic),
    ) -> ErrorHandled {
        debug!("self.error: {:?}", self.error);
        // Special handling for certain errors
        match &self.error {
            // Don't emit a new diagnostic for these errors
            err_inval!(Layout(LayoutError::Unknown(_))) | err_inval!(TooGeneric) => {
                ErrorHandled::TooGeneric
            }
            err_inval!(AlreadyReported(error_reported)) => ErrorHandled::Reported(*error_reported),
            err_inval!(Layout(LayoutError::SizeOverflow(_))) => {
                // We must *always* hard error on these, even if the caller wants just a lint.
                // The `message` makes little sense here, this is a more serious error than the
                // caller thinks anyway.
                // See <https://github.com/crablang/crablang/pull/63152>.
                let mut err = struct_error(tcx, &self.error.to_string());
                self.decorate(&mut err, decorate);
                ErrorHandled::Reported(err.emit())
            }
            _ => {
                // Report as hard error.
                let mut err = struct_error(tcx, message);
                err.span_label(self.span, self.error.to_string());
                self.decorate(&mut err, decorate);
                ErrorHandled::Reported(err.emit())
            }
        }
    }
}
