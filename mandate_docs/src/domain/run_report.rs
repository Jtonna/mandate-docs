//! [`RunReportMandatesValidation`]: a domain value collecting the results of
//! validating every mandate found in one run, and its rendering.

use super::validation::ValidationReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MandateResult {
    ParseFailed(String),
    Validated(ValidationReport),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MandateOutcome {
    pub file_name: String,
    pub result: MandateResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunWarning {
    NoMandatesFound { mandates_dir: String },
}

/// Where a run discovered its repository, or that it found none. There is
/// no repository in the `NotFound` case, so there is no root path and no
/// snapshot to report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunLocation {
    Found {
        root: String,
        snapshot_entries: usize,
    },
    NotFound {
        searched_from: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReportMandatesValidation {
    pub location: RunLocation,
    pub warnings: Vec<RunWarning>,
    pub mandates: Vec<MandateOutcome>,
}

impl RunReportMandatesValidation {
    /// No mandate failed to parse, and every validated mandate is valid.
    /// An empty `mandates` list is always valid, whether because no
    /// `.mandate` folder was found or because the mandates folder was
    /// empty.
    pub fn is_valid(&self) -> bool {
        self.mandates.iter().all(|outcome| match &outcome.result {
            MandateResult::ParseFailed(_) => false,
            MandateResult::Validated(report) => report.is_valid(),
        })
    }

    pub fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        match &self.location {
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

        for warning in &self.warnings {
            match warning {
                RunWarning::NoMandatesFound { mandates_dir } => {
                    lines.push(format!("warning: no mandates found in {mandates_dir}"));
                }
            }
        }
        if !self.warnings.is_empty() {
            lines.push(String::new());
        }

        let mut invalid_count = 0;
        for outcome in &self.mandates {
            lines.push(outcome.file_name.clone());
            match &outcome.result {
                MandateResult::ParseFailed(message) => {
                    invalid_count += 1;
                    lines.push(format!("  failed to parse: {message}"));
                }
                MandateResult::Validated(report) => {
                    for line in report.lines() {
                        lines.push(format!("  {line}"));
                    }
                    if report.is_valid() {
                        lines.push(format!(
                            "  valid: {} rules, {} documents, {} source files",
                            report.rule_count, report.doc_count, report.code_count
                        ));
                    } else {
                        invalid_count += 1;
                        lines.push(format!(
                            "  invalid: {} errors, {} warnings",
                            report.errors.len(),
                            report.warnings.len()
                        ));
                    }
                }
            }
            lines.push(String::new());
        }

        lines.push(format!(
            "{} mandates checked, {} invalid",
            self.mandates.len(),
            invalid_count
        ));

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::validation::{ValidationError, ValidationWarning};

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
            run.lines(),
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
                result: MandateResult::ParseFailed("unknown field 'bogus'".to_string()),
            }],
        };

        assert_eq!(
            run.lines(),
            vec![
                "repository: /repo".to_string(),
                "snapshot: 3 entries".to_string(),
                String::new(),
                "bad.yaml".to_string(),
                "  failed to parse: unknown field 'bogus'".to_string(),
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
            run.lines(),
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
            run.lines(),
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

    #[test]
    fn is_valid_true_only_when_no_parse_failures_and_all_reports_valid() {
        let all_valid = RunReportMandatesValidation {
            location: found("/repo", 1),
            warnings: Vec::new(),
            mandates: vec![MandateOutcome {
                file_name: "a.yaml".to_string(),
                result: MandateResult::Validated(valid_report()),
            }],
        };
        assert!(all_valid.is_valid());

        let one_invalid = RunReportMandatesValidation {
            mandates: vec![MandateOutcome {
                file_name: "a.yaml".to_string(),
                result: MandateResult::Validated(invalid_report()),
            }],
            ..all_valid.clone()
        };
        assert!(!one_invalid.is_valid());

        let one_parse_failed = RunReportMandatesValidation {
            mandates: vec![MandateOutcome {
                file_name: "a.yaml".to_string(),
                result: MandateResult::ParseFailed("bad".to_string()),
            }],
            ..all_valid
        };
        assert!(!one_parse_failed.is_valid());
    }
}
