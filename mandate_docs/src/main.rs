//! Composition root: `mandate [--root <dir>] [<mandate-file>.yaml ...]`.

use std::process::ExitCode;

use mandate::adapters::driven::fs_file_tree_source::FsFileTreeSource;
use mandate::adapters::driven::fs_mandate_store::FsMandateStore;
use mandate::adapters::driven::yaml::YamlMandateParser;
use mandate::adapters::driving::cli::{parse_args, render};
use mandate::application::run_mandates::RunMandates;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let invocation = match parse_args(&args) {
        Ok(invocation) => invocation,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let start_dir = match invocation.root {
        Some(root) => root,
        None => match std::env::current_dir() {
            Ok(dir) => dir,
            Err(err) => {
                eprintln!("failed to determine current directory: {err}");
                return ExitCode::FAILURE;
            }
        },
    };

    let source = FsFileTreeSource;
    let store = FsMandateStore;
    let parser = YamlMandateParser;
    let run = RunMandates::new(&source, &store, &parser);

    match run.execute(&start_dir, &invocation.mandates) {
        Ok(report) => {
            render(&report, &mut std::io::stdout());
            if report.is_valid() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
