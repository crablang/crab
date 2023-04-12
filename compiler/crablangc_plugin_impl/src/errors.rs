//! Errors emitted by plugin_impl

use crablangc_macros::Diagnostic;
use crablangc_span::Span;

#[derive(Diagnostic)]
#[diag(plugin_impl_load_plugin_error)]
pub struct LoadPluginError {
    #[primary_span]
    pub span: Span,
    pub msg: String,
}

#[derive(Diagnostic)]
#[diag(plugin_impl_malformed_plugin_attribute, code = "E0498")]
pub struct MalformedPluginAttribute {
    #[primary_span]
    #[label]
    pub span: Span,
}
