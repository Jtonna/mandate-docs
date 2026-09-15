//! The CLI as a driving adapter: pure argument parsing and pure rendering,
//! both testable in-process without spawning the compiled binary.

use std::io::Write;
use std::path::PathBuf;

use crate::domain::run_report::RunReportMandatesValidation;

const USAGE: &str = "usage: mandate [--root <dir>] [<mandate-file>.yaml ...]";

/// The parsed command line: an optional discovery root, and the mandate
/// file names to select (empty means "run every discovered mandate").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub root: Option<PathBuf>,
    pub mandates: Vec<String>,
}

/// Parses the process arguments (excluding argv[0]) into an [`Invocation`].
pub fn parse_args(args: &[String]) -> Result<Invocation, String> {
    let mut root = None;
    let mut mandates = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--root" {
            match iter.next() {
                Some(value) => root = Some(PathBuf::from(value)),
                None => return Err(USAGE.to_string()),
            }
        } else if let Some(option) = arg.strip_prefix("--") {
            return Err(format!("unknown option '{option}'"));
        } else if arg.starts_with('-') && arg.len() > 1 {
            return Err(format!("unknown option '{}'", &arg[1..]));
        } else {
            mandates.push(arg.clone());
        }
    }

    Ok(Invocation { root, mandates })
}

/// Writes `report.lines()` to `out`, one per line.
pub fn render(report: &RunReportMandatesValidation, out: &mut dyn Write) {
    for line in report.lines() {
        let _ = writeln!(out, "{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::run_report::{RunLocation, RunWarning};

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_args_with_no_args_has_no_root_and_no_mandates() {
        let result = parse_args(&args(&[])).expect("expected success");

        assert_eq!(result.root, None);
        assert_eq!(result.mandates, Vec::<String>::new());
    }

    #[test]
    fn parse_args_with_root_first_then_mandates() {
        let result =
            parse_args(&args(&["--root", "x", "a.yaml", "b.yaml"])).expect("expected success");

        assert_eq!(result.root, Some(PathBuf::from("x")));
        assert_eq!(
            result.mandates,
            vec!["a.yaml".to_string(), "b.yaml".to_string()]
        );
    }

    #[test]
    fn parse_args_with_mandate_before_root() {
        let result = parse_args(&args(&["a.yaml", "--root", "x"])).expect("expected success");

        assert_eq!(result.root, Some(PathBuf::from("x")));
        assert_eq!(result.mandates, vec!["a.yaml".to_string()]);
    }

    #[test]
    fn parse_args_with_root_and_no_value_is_usage_error() {
        let result = parse_args(&args(&["--root"]));

        let err = result.expect_err("expected an error");
        assert!(err.contains("usage:"), "got: {err}");
    }

    #[test]
    fn parse_args_with_unknown_option_is_unknown_option_error() {
        let result = parse_args(&args(&["--bogus"]));

        let err = result.expect_err("expected an error");
        assert!(err.contains("unknown option"), "got: {err}");
        assert!(err.contains("bogus"), "got: {err}");
    }

    #[test]
    fn render_writes_report_lines_joined_by_newline() {
        let report = RunReportMandatesValidation {
            location: RunLocation::Found {
                root: "/repo".to_string(),
                snapshot_entries: 1,
            },
            warnings: vec![RunWarning::NoMandatesFound {
                mandates_dir: ".mandate/mandates".to_string(),
            }],
            mandates: Vec::new(),
        };
        let mut out = Vec::new();

        render(&report, &mut out);

        let expected = report
            .lines()
            .into_iter()
            .map(|line| line + "\n")
            .collect::<String>();
        assert_eq!(String::from_utf8(out).expect("utf8"), expected);
    }
}
