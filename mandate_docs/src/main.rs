//! Composition root: `mandate [--root <dir>] [<mandate-file>.yaml ...]`.

use std::process::ExitCode;

use mandate::adapters::driven::file_system::os_file_system::OsFileSystem;
use mandate::adapters::driven::file_tree::fs_file_tree_source::FsFileTreeSource;
use mandate::adapters::driven::mandate_parser::yaml_mandate_parser::YamlMandateParser;
use mandate::adapters::driven::mandate_store::fs_mandate_store::FsMandateStore;
use mandate::adapters::driving::cli::{self, parse_args};
use mandate::domain::usecases::run_mandates::RunMandates;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let invocation = match parse_args(&args) {
        Ok(invocation) => invocation,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let start_dir = match invocation.root.clone() {
        Some(root) => root,
        None => match std::env::current_dir() {
            Ok(dir) => dir,
            Err(err) => {
                eprintln!("failed to determine current directory: {err}");
                return ExitCode::FAILURE;
            }
        },
    };

    let source = FsFileTreeSource::new(OsFileSystem);
    let store = FsMandateStore::new(OsFileSystem);
    let parser = YamlMandateParser;
    let run_mandates = RunMandates::new(&source, &store, &parser);

    let report = cli::run(
        &invocation,
        &start_dir,
        &run_mandates,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );

    // The exit code decision stays here, on purpose, rather than moving
    // into cli::run: a planned long-running mode will need to keep running
    // after an invalid report instead of exiting.
    match report {
        Some(r) if r.is_valid() => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}
