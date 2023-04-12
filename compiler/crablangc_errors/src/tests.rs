use crate::error::{TranslateError, TranslateErrorKind};
use crate::fluent_bundle::*;
use crate::translation::Translate;
use crate::FluentBundle;
use crablangc_data_structures::sync::Lrc;
use crablangc_error_messages::fluent_bundle::resolver::errors::{ReferenceKind, ResolverError};
use crablangc_error_messages::langid;
use crablangc_error_messages::DiagnosticMessage;

struct Dummy {
    bundle: FluentBundle,
}

impl Translate for Dummy {
    fn fluent_bundle(&self) -> Option<&Lrc<FluentBundle>> {
        None
    }

    fn fallback_fluent_bundle(&self) -> &FluentBundle {
        &self.bundle
    }
}

fn make_dummy(ftl: &'static str) -> Dummy {
    let resource = FluentResource::try_new(ftl.into()).expect("Failed to parse an FTL string.");

    let langid_en = langid!("en-US");

    #[cfg(parallel_compiler)]
    let mut bundle = FluentBundle::new_concurrent(vec![langid_en]);

    #[cfg(not(parallel_compiler))]
    let mut bundle = FluentBundle::new(vec![langid_en]);

    bundle.add_resource(resource).expect("Failed to add FTL resources to the bundle.");

    Dummy { bundle }
}

#[test]
fn wellformed_fluent() {
    let dummy = make_dummy("mir_build_borrow_of_moved_value = borrow of moved value
    .label = value moved into `{$name}` here
    .occurs_because_label = move occurs because `{$name}` has type `{$ty}` which does not implement the `Copy` trait
    .value_borrowed_label = value borrowed here after move
    .suggestion = borrow this binding in the pattern to avoid moving the value");

    let mut args = FluentArgs::new();
    args.set("name", "Foo");
    args.set("ty", "std::string::String");
    {
        let message = DiagnosticMessage::FluentIdentifier(
            "mir_build_borrow_of_moved_value".into(),
            Some("suggestion".into()),
        );

        assert_eq!(
            dummy.translate_message(&message, &args).unwrap(),
            "borrow this binding in the pattern to avoid moving the value"
        );
    }

    {
        let message = DiagnosticMessage::FluentIdentifier(
            "mir_build_borrow_of_moved_value".into(),
            Some("value_borrowed_label".into()),
        );

        assert_eq!(
            dummy.translate_message(&message, &args).unwrap(),
            "value borrowed here after move"
        );
    }

    {
        let message = DiagnosticMessage::FluentIdentifier(
            "mir_build_borrow_of_moved_value".into(),
            Some("occurs_because_label".into()),
        );

        assert_eq!(
            dummy.translate_message(&message, &args).unwrap(),
            "move occurs because `\u{2068}Foo\u{2069}` has type `\u{2068}std::string::String\u{2069}` which does not implement the `Copy` trait"
        );

        {
            let message = DiagnosticMessage::FluentIdentifier(
                "mir_build_borrow_of_moved_value".into(),
                Some("label".into()),
            );

            assert_eq!(
                dummy.translate_message(&message, &args).unwrap(),
                "value moved into `\u{2068}Foo\u{2069}` here"
            );
        }
    }
}

#[test]
fn misformed_fluent() {
    let dummy = make_dummy("mir_build_borrow_of_moved_value = borrow of moved value
    .label = value moved into `{name}` here
    .occurs_because_label = move occurs because `{$oops}` has type `{$ty}` which does not implement the `Copy` trait
    .suggestion = borrow this binding in the pattern to avoid moving the value");

    let mut args = FluentArgs::new();
    args.set("name", "Foo");
    args.set("ty", "std::string::String");
    {
        let message = DiagnosticMessage::FluentIdentifier(
            "mir_build_borrow_of_moved_value".into(),
            Some("value_borrowed_label".into()),
        );

        let err = dummy.translate_message(&message, &args).unwrap_err();
        assert!(
            matches!(
                &err,
                TranslateError::Two {
                    primary: box TranslateError::One {
                        kind: TranslateErrorKind::PrimaryBundleMissing,
                        ..
                    },
                    fallback: box TranslateError::One {
                        kind: TranslateErrorKind::AttributeMissing { attr: "value_borrowed_label" },
                        ..
                    }
                }
            ),
            "{err:#?}"
        );
        assert_eq!(
            format!("{err}"),
            "failed while formatting fluent string `mir_build_borrow_of_moved_value`: \nthe attribute `value_borrowed_label` was missing\nhelp: add `.value_borrowed_label = <message>`\n"
        );
    }

    {
        let message = DiagnosticMessage::FluentIdentifier(
            "mir_build_borrow_of_moved_value".into(),
            Some("label".into()),
        );

        let err = dummy.translate_message(&message, &args).unwrap_err();
        if let TranslateError::Two {
            primary: box TranslateError::One { kind: TranslateErrorKind::PrimaryBundleMissing, .. },
            fallback: box TranslateError::One { kind: TranslateErrorKind::Fluent { errs }, .. },
        } = &err
            && let [FluentError::ResolverError(ResolverError::Reference(
                ReferenceKind::Message { id, .. }
                    | ReferenceKind::Variable { id, .. },
            ))] = &**errs
            && id == "name"
        {} else {
            panic!("{err:#?}")
        };
        assert_eq!(
            format!("{err}"),
            "failed while formatting fluent string `mir_build_borrow_of_moved_value`: \nargument `name` exists but was not referenced correctly\nhelp: try using `{$name}` instead\n"
        );
    }

    {
        let message = DiagnosticMessage::FluentIdentifier(
            "mir_build_borrow_of_moved_value".into(),
            Some("occurs_because_label".into()),
        );

        let err = dummy.translate_message(&message, &args).unwrap_err();
        if let TranslateError::Two {
            primary: box TranslateError::One { kind: TranslateErrorKind::PrimaryBundleMissing, .. },
            fallback: box TranslateError::One { kind: TranslateErrorKind::Fluent { errs }, .. },
        } = &err
            && let [FluentError::ResolverError(ResolverError::Reference(
                ReferenceKind::Message { id, .. }
                    | ReferenceKind::Variable { id, .. },
            ))] = &**errs
            && id == "oops"
        {} else {
            panic!("{err:#?}")
        };
        assert_eq!(
            format!("{err}"),
            "failed while formatting fluent string `mir_build_borrow_of_moved_value`: \nthe fluent string has an argument `oops` that was not found.\nhelp: the arguments `name` and `ty` are available\n"
        );
    }
}
