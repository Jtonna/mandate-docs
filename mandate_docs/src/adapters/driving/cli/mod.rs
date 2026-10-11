//! The CLI as a driving adapter: pure argument parsing and pure rendering,
//! both testable in-process without spawning the compiled binary.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::domain::model::run_report::{
    MandateResult, RunLocation, RunReportMandatesValidation, RunWarning,
};
use crate::domain::model::validation::ValidationReport;
use crate::domain::usecases::run_mandates::RunMandates;

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

/// The validation report rendered one line per finding, errors first in
/// validator order, then warnings, each prefixed `error: ` or `warning: `.
pub fn validation_lines(report: &ValidationReport) -> Vec<String> {
    report
        .errors
        .iter()
        .map(|e| format!("error: {e}"))
        .chain(report.warnings.iter().map(|w| format!("warning: {w}")))
        .collect()
}

/// The run report rendered exactly as the CLI prints it: the repository
/// location, any run-level warnings, then each mandate's outcome, then a
/// summary line.
pub fn report_lines(report: &RunReportMandatesValidation) -> Vec<String> {
    let mut lines = Vec::new();

    match &report.location {
        RunLocation::Found {
            root,
            snapshot_entries,
        } => {
            lines.push(format!("repository: {root}"));
            lines.push(format!("snapshot: {snapshot_entries} entries"));
            lines.push(String::new());
        }
        RunLocation::NotFound { searched_from } => {
            lines.push(format!(
                "warning: no .mandate folder found from {searched_from} \
                 up to the filesystem root"
            ));
            lines.push(String::new());
        }
    }

    for warning in &report.warnings {
        match warning {
            RunWarning::NoMandatesFound { mandates_dir } => {
                lines.push(format!("warning: no mandates found in {mandates_dir}"));
            }
        }
    }
    if !report.warnings.is_empty() {
        lines.push(String::new());
    }

    let mut invalid_count = 0;
    for outcome in &report.mandates {
        lines.push(outcome.file_name.clone());
        match &outcome.result {
            MandateResult::ParseFailed(message) => {
                invalid_count += 1;
                lines.push(format!("  failed to parse: {message}"));
            }
            MandateResult::Validated(validation) => {
                for line in validation_lines(validation) {
                    lines.push(format!("  {line}"));
                }
                if validation.is_valid() {
                    lines.push(format!(
                        "  valid: {} rules, {} documents, {} source files",
                        validation.rule_count, validation.doc_count, validation.code_count
                    ));
                } else {
                    invalid_count += 1;
                    lines.push(format!(
                        "  invalid: {} errors, {} warnings",
                        validation.errors.len(),
                        validation.warnings.len()
                    ));
                }
            }
        }
        lines.push(String::new());
    }

    lines.push(format!(
        "{} mandates checked, {} invalid",
        report.mandates.len(),
        invalid_count
    ));

    lines
}

/// Writes `report_lines(report)` to `out`, one per line.
pub fn render(report: &RunReportMandatesValidation, out: &mut dyn Write) {
    for line in report_lines(report) {
        let _ = writeln!(out, "{line}");
    }
}

/// Runs one invocation end to end: execute the use case, render the
/// report or the error, and hand the report back so the caller can decide
/// the exit code. Never exits the process itself.
pub fn run(
    invocation: &Invocation,
    start_dir: &Path,
    run_mandates: &RunMandates<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Option<RunReportMandatesValidation> {
    match run_mandates.execute(start_dir, &invocation.mandates) {
        Ok(report) => {
            render(&report, out);
            Some(report)
        }
        Err(e) => {
            let _ = writeln!(err, "{e}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let expected = report_lines(&report)
            .into_iter()
            .map(|line| line + "\n")
            .collect::<String>();
        assert_eq!(String::from_utf8(out).expect("utf8"), expected);
    }

    #[test]
    fn validation_lines_are_errors_then_warnings_in_validator_order() {
        use crate::domain::model::validation::{ValidationError, ValidationWarning};

        let report = ValidationReport {
            errors: vec![
                ValidationError::NoRules,
                ValidationError::DuplicateRuleId {
                    id: "x".to_string(),
                },
            ],
            warnings: vec![ValidationWarning::UnreferencedRule {
                id: "y".to_string(),
            }],
            ..Default::default()
        };

        assert_eq!(
            validation_lines(&report),
            vec![
                "error: no rules defined; a mandate needs at least one".to_string(),
                "error: duplicate rule id 'x'".to_string(),
                "warning: rule 'y' is defined but no document references it".to_string(),
            ]
        );
    }

    mod report_lines_tests {
        use super::*;
        use crate::domain::model::run_report::MandateOutcome;
        use crate::domain::model::validation::{ValidationError, ValidationWarning};
        use crate::domain::ports::driven::mandate_parser::MandateParseError;

        fn valid_report() -> ValidationReport {
            ValidationReport {
                errors: Vec::new(),
                warnings: Vec::new(),
                rule_count: 1,
                doc_count: 1,
                code_count: 1,
            }
        }

        fn invalid_report() -> ValidationReport {
            ValidationReport {
                errors: vec![ValidationError::DocMissing {
                    doc: "docs/a.md".to_string(),
                }],
                warnings: vec![ValidationWarning::UnreferencedRule {
                    id: "x".to_string(),
                }],
                rule_count: 1,
                doc_count: 1,
                code_count: 1,
            }
        }

        fn found(root: &str, snapshot_entries: usize) -> RunLocation {
            RunLocation::Found {
                root: root.to_string(),
                snapshot_entries,
            }
        }

        #[test]
        fn lines_renders_one_valid_and_one_invalid_mandate() {
            let run = RunReportMandatesValidation {
                location: found("/repo", 42),
                warnings: Vec::new(),
                mandates: vec![
                    MandateOutcome {
                        file_name: "a.yaml".to_string(),
                        result: MandateResult::Validated(valid_report()),
                    },
                    MandateOutcome {
                        file_name: "b.yaml".to_string(),
                        result: MandateResult::Validated(invalid_report()),
                    },
                ],
            };

            assert_eq!(
                report_lines(&run),
                vec![
                    "repository: /repo".to_string(),
                    "snapshot: 42 entries".to_string(),
                    String::new(),
                    "a.yaml".to_string(),
                    "  valid: 1 rules, 1 documents, 1 source files".to_string(),
                    String::new(),
                    "b.yaml".to_string(),
                    "  error: governed document not found: docs/a.md".to_string(),
                    "  warning: rule 'x' is defined but no document references it".to_string(),
                    "  invalid: 1 errors, 1 warnings".to_string(),
                    String::new(),
                    "2 mandates checked, 1 invalid".to_string(),
                ]
            );
            assert!(!run.is_valid());
        }

        #[test]
        fn lines_renders_a_parse_failure() {
            let run = RunReportMandatesValidation {
                location: found("/repo", 3),
                warnings: Vec::new(),
                mandates: vec![MandateOutcome {
                    file_name: "bad.yaml".to_string(),
                    result: MandateResult::ParseFailed(MandateParseError::Malformed(
                        "unknown field 'bogus'".to_string(),
                    )),
                }],
            };

            assert_eq!(
                report_lines(&run),
                vec![
                    "repository: /repo".to_string(),
                    "snapshot: 3 entries".to_string(),
                    String::new(),
                    "bad.yaml".to_string(),
                    "  failed to parse: malformed mandate: unknown field 'bogus'".to_string(),
                    String::new(),
                    "1 mandates checked, 1 invalid".to_string(),
                ]
            );
            assert!(!run.is_valid());
        }

        #[test]
        fn lines_renders_the_no_mandates_warning_with_zero_mandates() {
            let run = RunReportMandatesValidation {
                location: found("/repo", 5),
                warnings: vec![RunWarning::NoMandatesFound {
                    mandates_dir: ".mandate/mandates".to_string(),
                }],
                mandates: Vec::new(),
            };

            assert_eq!(
                report_lines(&run),
                vec![
                    "repository: /repo".to_string(),
                    "snapshot: 5 entries".to_string(),
                    String::new(),
                    "warning: no mandates found in .mandate/mandates".to_string(),
                    String::new(),
                    "0 mandates checked, 0 invalid".to_string(),
                ]
            );
            assert!(run.is_valid());
        }

        #[test]
        fn lines_renders_not_found_with_no_repository_or_snapshot_lines() {
            let run = RunReportMandatesValidation {
                location: RunLocation::NotFound {
                    searched_from: "/repo/src".to_string(),
                },
                warnings: Vec::new(),
                mandates: Vec::new(),
            };

            assert_eq!(
                report_lines(&run),
                vec![
                    "warning: no .mandate folder found from /repo/src up to \
                     the filesystem root"
                        .to_string(),
                    String::new(),
                    "0 mandates checked, 0 invalid".to_string(),
                ]
            );
            assert!(run.is_valid());
        }
    }
}
