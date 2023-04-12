mod missing_cast_for_variadic_arg;
mod sized_unsized_cast;
mod wrong_number_of_generic_args;

pub use self::{
    missing_cast_for_variadic_arg::*, sized_unsized_cast::*, wrong_number_of_generic_args::*,
};

use crablangc_errors::{DiagnosticBuilder, DiagnosticId, ErrorGuaranteed};
use crablangc_session::Session;

pub trait StructuredDiagnostic<'tcx> {
    fn session(&self) -> &Session;

    fn code(&self) -> DiagnosticId;

    fn diagnostic(&self) -> DiagnosticBuilder<'tcx, ErrorGuaranteed> {
        let err = self.diagnostic_common();

        if self.session().teach(&self.code()) {
            self.diagnostic_extended(err)
        } else {
            self.diagnostic_regular(err)
        }
    }

    fn diagnostic_common(&self) -> DiagnosticBuilder<'tcx, ErrorGuaranteed>;

    fn diagnostic_regular(
        &self,
        err: DiagnosticBuilder<'tcx, ErrorGuaranteed>,
    ) -> DiagnosticBuilder<'tcx, ErrorGuaranteed> {
        err
    }

    fn diagnostic_extended(
        &self,
        err: DiagnosticBuilder<'tcx, ErrorGuaranteed>,
    ) -> DiagnosticBuilder<'tcx, ErrorGuaranteed> {
        err
    }
}
