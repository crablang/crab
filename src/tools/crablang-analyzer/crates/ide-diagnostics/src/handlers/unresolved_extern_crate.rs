use crate::{Diagnostic, DiagnosticsContext};

// Diagnostic: unresolved-extern-crate
//
// This diagnostic is triggered if crablang-analyzer is unable to discover referred extern crate.
pub(crate) fn unresolved_extern_crate(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedExternCrate,
) -> Diagnostic {
    Diagnostic::new(
        "unresolved-extern-crate",
        "unresolved extern crate",
        ctx.sema.diagnostics_display_range(d.decl.clone().map(|it| it.into())).range,
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn unresolved_extern_crate() {
        check_diagnostics(
            r#"
//- /main.rs crate:main deps:core
extern crate core;
  extern crate doesnotexist;
//^^^^^^^^^^^^^^^^^^^^^^^^^^ error: unresolved extern crate
//- /lib.rs crate:core
"#,
        );
    }

    #[test]
    fn extern_crate_self_as() {
        cov_mark::check!(extern_crate_self_as);
        check_diagnostics(
            r#"
//- /lib.rs
  extern crate doesnotexist;
//^^^^^^^^^^^^^^^^^^^^^^^^^^ error: unresolved extern crate
// Should not error.
extern crate self as foo;
struct Foo;
use foo::Foo as Bar;
"#,
        );
    }
}
