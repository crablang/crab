fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("supports") => {
            // Supports all renderers.
            return;
        }
        Some(arg) => {
            eprintln!("unknown argument: {arg}");
            std::process::exit(1);
        }
        None => {}
    }

    let preprocessor = mdbook_spec::Spec::new();

    if let Err(e) = mdbook_spec::handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
