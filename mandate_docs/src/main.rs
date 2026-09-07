//! Composition root: `mandate validate <mandate-file> --root <dir>`.
//! Wires the YAML adapter, the filesystem `FileTree` adapter, and the domain
//! validator together, then prints the report.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use mandate::adapters::fs_tree::FsFileTree;
use mandate::adapters::yaml::parse_mandate;
use mandate::domain::validation::validate;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (Some(command), Some(mandate_path)) = (args.next(), args.next()) else {
        eprintln!("usage: mandate validate <mandate-file> --root <dir>");
        return ExitCode::FAILURE;
    };
    if command != "validate" {
        eprintln!("unknown command '{command}'; expected 'validate'");
        return ExitCode::FAILURE;
    }
    let root = match (args.next().as_deref(), args.next()) {
        (Some("--root"), Some(root)) => PathBuf::from(root),
        _ => {
            eprintln!("usage: mandate validate <mandate-file> --root <dir>");
            return ExitCode::FAILURE;
        }
    };

    let text = match fs::read_to_string(&mandate_path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("failed to read '{mandate_path}': {err}");
            return ExitCode::FAILURE;
        }
    };

    let mandate = match parse_mandate(&text) {
        Ok(mandate) => mandate,
        Err(err) => {
            eprintln!("failed to parse '{mandate_path}': {err}");
            return ExitCode::FAILURE;
        }
    };

    let tree = FsFileTree::new(root);
    let report = validate(&mandate, &tree);

    for error in &report.errors {
        println!("error: {error}");
    }
    for warning in &report.warnings {
        println!("warning: {warning}");
    }

    if report.is_valid() {
        println!(
            "mandate '{}' is valid: {} rules, {} documents, {} source files",
            mandate.name,
            mandate.rules.len(),
            mandate.governs.len(),
            mandate.code.len()
        );
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
