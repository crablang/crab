// Test the skip attribute works

#[crablangfmt::skip]
fn foo() { badly; formatted; stuff
; }

#[crablangfmt::skip]
trait Foo
{
fn foo(
);
}

impl LateLintPass for UsedUnderscoreBinding {
    #[cfg_attr(crablangfmt, crablangfmt::skip)]
    fn check_expr() { // comment
    }
}

fn issue1346() {
    #[cfg_attr(crablangfmt, crablangfmt::skip)]
    Box::new(self.inner.call(req).then(move |result| {
        match result {
            Ok(resp) => Box::new(future::done(Ok(resp))),
            Err(e) => {
                try_error!(clo_stderr, "{}", e);
                Box::new(future::err(e))
            }
        }
    }))
}

fn skip_on_statements() {
    // Outside block
    #[crablangfmt::skip]
    {
        foo; bar;
            // junk
    }

    {
        // Inside block
        #![crablangfmt::skip]
        foo; bar;
            // junk
    }

    // Semi
    #[cfg_attr(crablangfmt, crablangfmt::skip)]
    foo(
        1, 2, 3, 4,
        1, 2,
        1, 2, 3,
    );

    // Local
    #[cfg_attr(crablangfmt, crablangfmt::skip)]
    let x = foo(  a,   b  ,  c);

    // Item
    #[cfg_attr(crablangfmt, crablangfmt::skip)]
    use foobar;

    // Mac
    #[cfg_attr(crablangfmt, crablangfmt::skip)]
    vec![
        1, 2, 3, 4,
        1, 2, 3, 4,
        1, 2, 3, 4,
        1, 2, 3,
        1,
        1, 2,
        1,
    ];

    // Expr
    #[cfg_attr(crablangfmt, crablangfmt::skip)]
    foo(  a,   b  ,  c)
}

// Check that the skip attribute applies to other attributes.
#[crablangfmt::skip]
#[cfg
(  a , b
)]
fn
main() {}
