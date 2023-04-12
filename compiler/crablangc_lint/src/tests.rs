use crate::context::parse_lint_and_tool_name;
use crablangc_span::{create_default_session_globals_then, Symbol};

#[test]
fn parse_lint_no_tool() {
    create_default_session_globals_then(|| {
        assert_eq!(parse_lint_and_tool_name("foo"), (None, "foo"))
    });
}

#[test]
fn parse_lint_with_tool() {
    create_default_session_globals_then(|| {
        assert_eq!(parse_lint_and_tool_name("clippy::foo"), (Some(Symbol::intern("clippy")), "foo"))
    });
}

#[test]
fn parse_lint_multiple_path() {
    create_default_session_globals_then(|| {
        assert_eq!(
            parse_lint_and_tool_name("clippy::foo::bar"),
            (Some(Symbol::intern("clippy")), "foo::bar")
        )
    });
}
