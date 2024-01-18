use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

macro_rules! style_error {
    ($bad:expr, $path:expr, $($arg:tt)*) => {
        *$bad = true;
        eprint!("error in {}: ", $path.display());
        eprintln!("{}", format_args!($($arg)*));
    };
}

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Please pass a src directory as the first argument");
        std::process::exit(1);
    });

    let mut bad = false;
    if let Err(e) = check_directory(&Path::new(&arg), &mut bad) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
    if bad {
        eprintln!("some style checks failed");
        std::process::exit(1);
    }
    eprintln!("passed!");
}

fn check_directory(dir: &Path, bad: &mut bool) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            check_directory(&path, bad)?;
            continue;
        }

        if !matches!(
            path.extension().and_then(|p| p.to_str()),
            Some("md") | Some("html")
        ) {
            // This may be extended in the future if other file types are needed.
            style_error!(bad, path, "expected only md or html in src");
        }

        let contents = fs::read_to_string(&path)?;
        if contents.contains("#![feature") {
            style_error!(bad, path, "#![feature] attributes are not allowed");
        }
        if contents.contains('\r') {
            style_error!(
                bad,
                path,
                "CR characters not allowed, must use LF line endings"
            );
        }
        if contents.contains('\t') {
            style_error!(bad, path, "tab characters not allowed, use spaces");
        }
        if !contents.ends_with('\n') {
            style_error!(bad, path, "file must end with a newline");
        }
        for line in contents.lines() {
            if line.ends_with(' ') {
                style_error!(bad, path, "lines must not end with spaces");
            }
        }
        cmark_check(&path, bad, &contents)?;
    }
    Ok(())
}

fn cmark_check(path: &Path, bad: &mut bool, contents: &str) -> Result<(), Box<dyn Error>> {
    use pulldown_cmark::{BrokenLink, CodeBlockKind, Event, Options, Parser, Tag};

    macro_rules! cmark_error {
        ($bad:expr, $path:expr, $range:expr, $($arg:tt)*) => {
            *$bad = true;
            let lineno = contents[..$range.start].chars().filter(|&ch| ch == '\n').count() + 1;
            eprint!("error in {} (line {}): ", $path.display(), lineno);
            eprintln!("{}", format_args!($($arg)*));
        }
    }

    let options = Options::all();
    // Can't use `bad` because it would get captured in closure.
    let mut link_err = false;
    let mut cb = |link: BrokenLink<'_>| {
        cmark_error!(
            &mut link_err,
            path,
            link.span,
            "broken {:?} link (reference `{}`)",
            link.link_type,
            link.reference
        );
        None
    };
    let parser = Parser::new_with_broken_link_callback(contents, options, Some(&mut cb));

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                cmark_error!(
                    bad,
                    path,
                    range,
                    "indented code blocks should use triple backtick-style \
                    with a language identifier"
                );
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(languages))) => {
                if languages.is_empty() {
                    cmark_error!(
                        bad,
                        path,
                        range,
                        "code block should include an explicit language",
                    );
                }
            }
            _ => {}
        }
    }
    *bad |= link_err;
    Ok(())
}
