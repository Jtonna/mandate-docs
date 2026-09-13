//! Composition root: `mandate validate <mandate-file> --root <dir>`.

use std::fs;
use std::process::ExitCode;

use mandate::adapters::cli::{execute, parse_args};
use mandate::adapters::fs_tree::FsFileTree;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let invocation = match parse_args(&args) {
        Ok(invocation) => invocation,
        Err(message) => return fail(message),
    };
    let text = match fs::read_to_string(&invocation.mandate_path) {
        Ok(text) => text,
        Err(err) => {
            return fail(format!(
                "failed to read '{}': {err}",
                invocation.mandate_path
            ))
        }
    };

    let tree = FsFileTree::new(invocation.root);
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let code = execute(
        &invocation.mandate_path,
        &text,
        &tree,
        &mut stdout,
        &mut stderr,
    );
    if code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn fail(message: impl AsRef<str>) -> ExitCode {
    eprintln!("{}", message.as_ref());
    ExitCode::FAILURE
}
