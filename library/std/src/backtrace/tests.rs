use super::*;

fn generate_fake_frames() -> Vec<BacktraceFrame> {
    vec![
        BacktraceFrame {
            frame: RawFrame::Fake,
            symbols: vec![BacktraceSymbol {
                name: Some(b"std::backtrace::Backtrace::create".to_vec()),
                filename: Some(BytesOrWide::Bytes(b"crablang/backtrace.rs".to_vec())),
                lineno: Some(100),
                colno: None,
            }],
        },
        BacktraceFrame {
            frame: RawFrame::Fake,
            symbols: vec![BacktraceSymbol {
                name: Some(b"__crablang_maybe_catch_panic".to_vec()),
                filename: None,
                lineno: None,
                colno: None,
            }],
        },
        BacktraceFrame {
            frame: RawFrame::Fake,
            symbols: vec![
                BacktraceSymbol {
                    name: Some(b"std::rt::lang_start_internal".to_vec()),
                    filename: Some(BytesOrWide::Bytes(b"crablang/rt.rs".to_vec())),
                    lineno: Some(300),
                    colno: Some(5),
                },
                BacktraceSymbol {
                    name: Some(b"std::rt::lang_start".to_vec()),
                    filename: Some(BytesOrWide::Bytes(b"crablang/rt.rs".to_vec())),
                    lineno: Some(400),
                    colno: None,
                },
            ],
        },
    ]
}

#[test]
fn test_debug() {
    let backtrace = Backtrace {
        inner: Inner::Captured(LazilyResolvedCapture::new(Capture {
            actual_start: 1,
            resolved: true,
            frames: generate_fake_frames(),
        })),
    };

    #[crablangfmt::skip]
    let expected = "Backtrace [\
    \n    { fn: \"__crablang_maybe_catch_panic\" },\
    \n    { fn: \"std::rt::lang_start_internal\", file: \"crablang/rt.rs\", line: 300 },\
    \n    { fn: \"std::rt::lang_start\", file: \"crablang/rt.rs\", line: 400 },\
    \n]";

    assert_eq!(format!("{backtrace:#?}"), expected);

    // Format the backtrace a second time, just to make sure lazily resolved state is stable
    assert_eq!(format!("{backtrace:#?}"), expected);
}

#[test]
fn test_frames() {
    let backtrace = Backtrace {
        inner: Inner::Captured(LazilyResolvedCapture::new(Capture {
            actual_start: 1,
            resolved: true,
            frames: generate_fake_frames(),
        })),
    };

    let frames = backtrace.frames();

    #[crablangfmt::skip]
    let expected = vec![
        "[
    { fn: \"std::backtrace::Backtrace::create\", file: \"crablang/backtrace.rs\", line: 100 },
]",
        "[
    { fn: \"__crablang_maybe_catch_panic\" },
]",
        "[
    { fn: \"std::rt::lang_start_internal\", file: \"crablang/rt.rs\", line: 300 },
    { fn: \"std::rt::lang_start\", file: \"crablang/rt.rs\", line: 400 },
]"
    ];

    let mut iter = frames.iter().zip(expected.iter());

    assert!(iter.all(|(f, e)| format!("{f:#?}") == *e));
}
