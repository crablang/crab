use super::*;

// FIXME(#27438): right now the unit tests of libcrablangc_middle don't refer to any actual
//                functions generated in libcrablangc_data_structures (all
//                references are through generic functions), but statics are
//                referenced from time to time. Due to this bug we won't
//                actually correctly link in the statics unless we also
//                reference a function, so be sure to reference a dummy
//                function.
#[test]
fn noop() {
    crablangc_data_structures::__noop_fix_for_27438();
}
