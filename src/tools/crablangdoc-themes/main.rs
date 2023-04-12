use std::env::args;
use std::fs::read_dir;
use std::path::Path;
use std::process::{exit, Command};

const FILES_TO_IGNORE: &[&str] = &["light.css"];

fn get_folders<P: AsRef<Path>>(folder_path: P) -> Vec<String> {
    let mut ret = Vec::with_capacity(10);

    for entry in read_dir(folder_path.as_ref()).expect("read_dir failed") {
        let entry = entry.expect("Couldn't unwrap entry");
        let path = entry.path();

        if !path.is_file() {
            continue;
        }
        let filename = path.file_name().expect("file_name failed");
        if FILES_TO_IGNORE.iter().any(|x| x == &filename) {
            continue;
        }
        ret.push(format!("{}", path.display()));
    }
    ret
}

fn main() {
    let argv: Vec<String> = args().collect();

    if argv.len() < 3 {
        eprintln!("Needs crablangdoc binary path");
        exit(1);
    }
    let crablangdoc_bin = &argv[1];
    let themes_folder = &argv[2];
    let themes = get_folders(&themes_folder);
    if themes.is_empty() {
        eprintln!("No theme found in \"{}\"...", themes_folder);
        exit(1);
    }
    let arg_name = "--check-theme".to_owned();
    let status = Command::new(crablangdoc_bin)
        .args(&themes.iter().flat_map(|t| vec![&arg_name, t].into_iter()).collect::<Vec<_>>())
        .status()
        .expect("failed to execute child");
    if !status.success() {
        exit(1);
    }
}
