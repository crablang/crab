use clap::crate_version;

use std::env;
use std::path::{Path, PathBuf};

use clap::{arg, ArgMatches, Command};

use mdbook::errors::Result as Result3;
use mdbook::MDBook;

fn main() {
    let crate_version = concat!("v", crate_version!());
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let d_arg = arg!(-d --"dest-dir" <DEST_DIR>
"The output directory for your book\n(Defaults to ./book when omitted)")
    .required(false)
    .value_parser(clap::value_parser!(PathBuf));

    let dir_arg = arg!([dir] "Root directory for the book\n\
                              (Defaults to the current directory when omitted)")
    .value_parser(clap::value_parser!(PathBuf));

    let matches = Command::new("crablangbook")
        .about("Build a book with mdBook")
        .author("Steve Klabnik <steve@steveklabnik.com>")
        .version(crate_version)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("build")
                .about("Build the book from the markdown files")
                .arg(d_arg)
                .arg(&dir_arg),
        )
        .subcommand(
            Command::new("test")
                .about("Tests that a book's CrabLang code samples compile")
                .arg(dir_arg),
        )
        .get_matches();

    // Check which subcomamnd the user ran...
    match matches.subcommand() {
        Some(("build", sub_matches)) => {
            if let Err(e) = build(sub_matches) {
                handle_error(e);
            }
        }
        Some(("test", sub_matches)) => {
            if let Err(e) = test(sub_matches) {
                handle_error(e);
            }
        }
        _ => unreachable!(),
    };
}

// Build command implementation
pub fn build(args: &ArgMatches) -> Result3<()> {
    let book_dir = get_book_dir(args);
    let mut book = load_book(&book_dir)?;

    // Set this to allow us to catch bugs in advance.
    book.config.build.create_missing = false;

    if let Some(dest_dir) = args.get_one::<PathBuf>("dest-dir") {
        book.config.build.build_dir = dest_dir.into();
    }

    book.build()?;

    Ok(())
}

fn test(args: &ArgMatches) -> Result3<()> {
    let book_dir = get_book_dir(args);
    let mut book = load_book(&book_dir)?;
    book.test(vec![])
}

fn get_book_dir(args: &ArgMatches) -> PathBuf {
    if let Some(p) = args.get_one::<PathBuf>("dir") {
        // Check if path is relative from current dir, or absolute...
        if p.is_relative() { env::current_dir().unwrap().join(p) } else { p.to_path_buf() }
    } else {
        env::current_dir().unwrap()
    }
}

fn load_book(book_dir: &Path) -> Result3<MDBook> {
    let mut book = MDBook::load(book_dir)?;
    book.config.set("output.html.input-404", "").unwrap();
    Ok(book)
}

fn handle_error(error: mdbook::errors::Error) -> ! {
    eprintln!("Error: {}", error);

    for cause in error.chain().skip(1) {
        eprintln!("\tCaused By: {}", cause);
    }

    ::std::process::exit(101);
}
