#![feature(crablangc_private)]

extern crate crablangc_driver;
extern crate crablangc_interface;
extern crate crablangc_session;
extern crate crablangc_span;

use crablangc_interface::interface;
use crablangc_session::config::{Input, Options, OutputType, OutputTypes};
use crablangc_span::source_map::FileName;

use std::path::PathBuf;

fn main() {
    let src = r#"
    fn main() {}
    "#;

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        panic!("expected crablangc path");
    }

    let tmpdir = PathBuf::from(&args[1]);

    let mut sysroot = PathBuf::from(&args[3]);
    sysroot.pop();
    sysroot.pop();

    compile(src.to_string(), tmpdir.join("out"), sysroot.clone());

    compile(src.to_string(), tmpdir.join("out"), sysroot.clone());
}

fn compile(code: String, output: PathBuf, sysroot: PathBuf) {
    let mut opts = Options::default();
    opts.output_types = OutputTypes::new(&[(OutputType::Exe, None)]);
    opts.maybe_sysroot = Some(sysroot);

    if let Ok(linker) = std::env::var("CRABLANGC_LINKER") {
        opts.cg.linker = Some(linker.into());
    }

    let name = FileName::anon_source_code(&code);
    let input = Input::Str { name, input: code };

    let config = interface::Config {
        opts,
        crate_cfg: Default::default(),
        crate_check_cfg: Default::default(),
        input,
        output_file: Some(output),
        output_dir: None,
        file_loader: None,
        locale_resources: &[],
        lint_caps: Default::default(),
        parse_sess_created: None,
        register_lints: None,
        override_queries: None,
        make_codegen_backend: None,
        registry: crablangc_driver::diagnostics_registry(),
    };

    interface::run_compiler(config, |compiler| {
        // This runs all the passes prior to linking, too.
        let linker = compiler.enter(|queries| queries.linker());
        if let Ok(linker) = linker {
            linker.link();
        }
    });
}
