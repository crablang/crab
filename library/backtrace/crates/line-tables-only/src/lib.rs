#[cfg(test)]
mod tests {
    use backtrace::Backtrace;
    use libc::c_void;
    use std::path::Path;
    use std::ptr::addr_of_mut;

    pub type Callback = extern "C" fn(data: *mut c_void);

    extern "C" {
        fn foo(cb: Callback, data: *mut c_void);
    }

    extern "C" fn store_backtrace(data: *mut c_void) {
        let bt = backtrace::Backtrace::new();
        unsafe { *data.cast::<Option<Backtrace>>() = Some(bt) };
    }

    fn assert_contains(
        backtrace: &Backtrace,
        expected_name: &str,
        expected_file: &str,
        expected_line: u32,
    ) {
        let expected_file = Path::new(expected_file);

        for frame in backtrace.frames() {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    if name.as_bytes() == expected_name.as_bytes() {
                        assert!(symbol.filename().unwrap().ends_with(expected_file));
                        assert_eq!(symbol.lineno(), Some(expected_line));
                        return;
                    }
                }
            }
        }

        panic!("symbol {expected_name:?} not found in backtrace: {backtrace:?}");
    }

    /// Verifies that when debug info includes only lines tables the generated
    /// backtrace is still generated successfully. The test exercises behaviour
    /// that failed previously when compiling with clang -g1.
    ///
    /// The test case uses C rather than rust, since at that time when it was
    /// written the debug info generated at level 1 in rustc was essentially
    /// the same as at level 2.
    #[test]
    #[cfg_attr(windows, ignore)]
    fn backtrace_works_with_line_tables_only() {
        let mut backtrace: Option<Backtrace> = None;
        unsafe { foo(store_backtrace, addr_of_mut!(backtrace).cast::<c_void>()) };
        let backtrace = backtrace.expect("backtrace");
        assert_contains(&backtrace, "foo", "src/callback.c", 13);
        assert_contains(&backtrace, "bar", "src/callback.c", 9);
        assert_contains(&backtrace, "baz", "src/callback.c", 5);
    }
}
