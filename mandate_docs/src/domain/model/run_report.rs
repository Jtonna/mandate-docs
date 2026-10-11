//! [`RunReportMandatesValidation`]: a domain value collecting the results of
//! validating every mandate found in one run. Rendering it is the CLI
//! adapter's job (see `cli::report_lines`).

use crate::domain::ports::driven::mandate_parser::MandateParseError;

use super::validation::ValidationReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MandateResult {
    ParseFailed(MandateParseError),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::validation::{ValidationError, ValidationWarning};

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
                result: MandateResult::ParseFailed(MandateParseError::Malformed("bad".to_string())),
            }],
            ..all_valid
        };
        assert!(!one_parse_failed.is_valid());
    }
}
