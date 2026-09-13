//! Validates a parsed [`Mandate`] against a [`FileTreeSnapshot`], reporting
//! every problem found rather than stopping at the first.

use std::fmt;

use super::file_tree::FileTreeSnapshot;
use super::mandate::Mandate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    NoRules,
    DuplicateRuleId { id: String },
    UnknownRule { doc: String, rule: String },
    CodeLinksUngovernedDoc { path: String, doc: String },
    DocMissing { doc: String },
    CodeMissing { path: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::NoRules => {
                write!(f, "no rules defined; a mandate needs at least one")
            }
            ValidationError::DuplicateRuleId { id } => write!(f, "duplicate rule id '{id}'"),
            ValidationError::UnknownRule { doc, rule } => {
                write!(f, "rule '{rule}' is referenced by {doc} but not defined")
            }
            ValidationError::CodeLinksUngovernedDoc { path, doc } => write!(
                f,
                "code {path} links {doc} which this mandate does not govern"
            ),
            ValidationError::DocMissing { doc } => {
                write!(f, "governed document not found: {doc}")
            }
            ValidationError::CodeMissing { path } => {
                write!(f, "missing source file: {path}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationWarning {
    UnreferencedRule { id: String },
}

impl fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationWarning::UnreferencedRule { id } => {
                write!(f, "rule '{id}' is defined but no document references it")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    /// Counts from the mandate that was checked, carried alongside the
    /// findings so a caller can render a "valid: N rules, N documents, N
    /// source files" summary without holding onto the `Mandate` itself.
    pub rule_count: usize,
    pub doc_count: usize,
    pub code_count: usize,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// The report rendered one line per finding, errors first in validator
    /// order, then warnings, each prefixed `error: ` or `warning: `.
    pub fn lines(&self) -> Vec<String> {
        self.errors
            .iter()
            .map(|e| format!("error: {e}"))
            .chain(self.warnings.iter().map(|w| format!("warning: {w}")))
            .collect()
    }
}

pub fn validate(mandate: &Mandate, snapshot: &FileTreeSnapshot) -> ValidationReport {
    use std::collections::HashSet;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if mandate.rules.is_empty() {
        errors.push(ValidationError::NoRules);
    }

    // Duplicate rule ids, and which ids are known / referenced.
    let mut seen_rule_ids: HashSet<&str> = HashSet::new();
    let mut known_rule_ids: HashSet<&str> = HashSet::new();
    for rule in &mandate.rules {
        if !seen_rule_ids.insert(rule.id.as_str()) {
            errors.push(ValidationError::DuplicateRuleId {
                id: rule.id.clone(),
            });
        }
        known_rule_ids.insert(rule.id.as_str());
    }

    let mut referenced_rule_ids: HashSet<&str> = HashSet::new();
    let mut governed_docs: HashSet<&str> = HashSet::new();

    for governed in &mandate.governs {
        governed_docs.insert(governed.doc.as_str());

        if !snapshot.is_file(&governed.doc) {
            errors.push(ValidationError::DocMissing {
                doc: governed.doc.clone(),
            });
        }

        for rule_id in &governed.rules {
            if known_rule_ids.contains(rule_id.as_str()) {
                referenced_rule_ids.insert(rule_id.as_str());
            } else {
                errors.push(ValidationError::UnknownRule {
                    doc: governed.doc.clone(),
                    rule: rule_id.clone(),
                });
            }
        }
    }

    for code_link in &mandate.code {
        if !snapshot.is_file(&code_link.path) {
            errors.push(ValidationError::CodeMissing {
                path: code_link.path.clone(),
            });
        }

        for doc in &code_link.docs {
            if !governed_docs.contains(doc.as_str()) {
                errors.push(ValidationError::CodeLinksUngovernedDoc {
                    path: code_link.path.clone(),
                    doc: doc.clone(),
                });
            }
        }
    }

    for rule in &mandate.rules {
        if !referenced_rule_ids.contains(rule.id.as_str()) {
            warnings.push(ValidationWarning::UnreferencedRule {
                id: rule.id.clone(),
            });
        }
    }

    ValidationReport {
        errors,
        warnings,
        rule_count: mandate.rules.len(),
        doc_count: mandate.governs.len(),
        code_count: mandate.code.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::mandate::{CodeLink, GovernedDoc, Rule, RuleKind};

    fn rule(id: &str) -> Rule {
        Rule {
            id: id.to_string(),
            description: None,
            kind: RuleKind::Script {
                run: "./check.sh".to_string(),
            },
        }
    }

    fn valid_mandate() -> Mandate {
        Mandate {
            name: "Test".to_string(),
            description: None,
            rules: vec![rule("has-owner")],
            governs: vec![GovernedDoc {
                doc: "docs/a.md".to_string(),
                rules: vec!["has-owner".to_string()],
            }],
            code: vec![CodeLink {
                path: "src/a.ts".to_string(),
                docs: vec!["docs/a.md".to_string()],
            }],
        }
    }

    #[test]
    fn validate_valid_mandate_with_all_files_present() {
        let mandate = valid_mandate();
        let snapshot = FileTreeSnapshot::with_every_file_in(&mandate);

        let report = validate(&mandate, &snapshot);

        assert!(report.is_valid());
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    }

    #[test]
    fn validate_no_rules_is_error() {
        let mut mandate = valid_mandate();
        mandate.rules.clear();
        mandate.governs[0].rules.clear();
        let snapshot = FileTreeSnapshot::with_every_file_in(&mandate);

        let report = validate(&mandate, &snapshot);

        assert!(!report.is_valid());
        assert!(report.errors.contains(&ValidationError::NoRules));
    }

    #[test]
    fn validate_duplicate_rule_id_is_error() {
        let mut mandate = valid_mandate();
        mandate.rules.push(rule("has-owner"));
        let snapshot = FileTreeSnapshot::with_every_file_in(&mandate);

        let report = validate(&mandate, &snapshot);

        assert!(!report.is_valid());
        assert!(report.errors.contains(&ValidationError::DuplicateRuleId {
            id: "has-owner".to_string()
        }));
    }

    #[test]
    fn validate_unknown_rule_reference_is_error() {
        let mut mandate = valid_mandate();
        mandate.governs[0].rules.push("no-such-rule".to_string());
        let snapshot = FileTreeSnapshot::with_every_file_in(&mandate);

        let report = validate(&mandate, &snapshot);

        assert!(!report.is_valid());
        assert!(report.errors.contains(&ValidationError::UnknownRule {
            doc: "docs/a.md".to_string(),
            rule: "no-such-rule".to_string(),
        }));
    }

    #[test]
    fn validate_code_linking_ungoverned_doc_is_error() {
        let mut mandate = valid_mandate();
        mandate.code[0].docs.push("docs/ungoverned.md".to_string());
        let snapshot = FileTreeSnapshot::with_every_file_in(&mandate);

        let report = validate(&mandate, &snapshot);

        assert!(!report.is_valid());
        assert!(report
            .errors
            .contains(&ValidationError::CodeLinksUngovernedDoc {
                path: "src/a.ts".to_string(),
                doc: "docs/ungoverned.md".to_string(),
            }));
    }

    #[test]
    fn validate_missing_doc_is_error() {
        let mandate = valid_mandate();
        let mut snapshot = FileTreeSnapshot::with_every_file_in(&mandate);
        snapshot.remove("docs/a.md");

        let report = validate(&mandate, &snapshot);

        assert!(!report.is_valid());
        assert!(report.errors.contains(&ValidationError::DocMissing {
            doc: "docs/a.md".to_string()
        }));
    }

    #[test]
    fn validate_missing_code_file_is_error() {
        let mandate = valid_mandate();
        let mut snapshot = FileTreeSnapshot::with_every_file_in(&mandate);
        snapshot.remove("src/a.ts");

        let report = validate(&mandate, &snapshot);

        assert!(!report.is_valid());
        assert!(report.errors.contains(&ValidationError::CodeMissing {
            path: "src/a.ts".to_string()
        }));
    }

    #[test]
    fn validate_a_directory_at_a_governed_doc_path_is_doc_missing() {
        let mandate = valid_mandate();
        let mut snapshot = FileTreeSnapshot::with_every_file_in(&mandate);
        snapshot.remove("docs/a.md");
        snapshot.insert("docs/a.md", crate::domain::file_tree::EntryKind::Directory);

        let report = validate(&mandate, &snapshot);

        assert!(!report.is_valid());
        assert!(report.errors.contains(&ValidationError::DocMissing {
            doc: "docs/a.md".to_string()
        }));
    }

    #[test]
    fn validate_unreferenced_rule_is_warning_not_error() {
        let mut mandate = valid_mandate();
        mandate.rules.push(rule("unused-rule"));
        let snapshot = FileTreeSnapshot::with_every_file_in(&mandate);

        let report = validate(&mandate, &snapshot);

        assert!(report.is_valid());
        assert_eq!(report.warnings.len(), 1);
        assert!(report
            .warnings
            .contains(&ValidationWarning::UnreferencedRule {
                id: "unused-rule".to_string()
            }));
    }

    #[test]
    fn validate_reports_all_problems_in_one_pass() {
        let mut mandate = valid_mandate();
        mandate.rules.clear(); // -> NoRules
        mandate.governs[0].rules.clear();
        mandate.governs[0].rules.push("no-such-rule".to_string()); // -> UnknownRule
        mandate.code[0].docs.push("docs/ungoverned.md".to_string()); // -> CodeLinksUngovernedDoc
        let snapshot = FileTreeSnapshot::with_every_file_in(&mandate);

        let report = validate(&mandate, &snapshot);

        assert!(!report.is_valid());
        assert!(report.errors.contains(&ValidationError::NoRules));
        assert!(report.errors.contains(&ValidationError::UnknownRule {
            doc: "docs/a.md".to_string(),
            rule: "no-such-rule".to_string(),
        }));
        assert!(report
            .errors
            .contains(&ValidationError::CodeLinksUngovernedDoc {
                path: "src/a.ts".to_string(),
                doc: "docs/ungoverned.md".to_string(),
            }));
        assert!(report.errors.len() >= 3);
    }

    #[test]
    fn validation_error_display_reads_as_a_sentence() {
        assert_eq!(
            ValidationError::CodeMissing {
                path: "src/domain/order.ts".to_string()
            }
            .to_string(),
            "missing source file: src/domain/order.ts"
        );
        assert_eq!(
            ValidationError::DocMissing {
                doc: "docs/x.md".to_string()
            }
            .to_string(),
            "governed document not found: docs/x.md"
        );
        assert_eq!(
            ValidationError::DuplicateRuleId {
                id: "x".to_string()
            }
            .to_string(),
            "duplicate rule id 'x'"
        );
    }

    #[test]
    fn validation_warning_display_reads_as_a_sentence() {
        assert_eq!(
            ValidationWarning::UnreferencedRule {
                id: "x".to_string()
            }
            .to_string(),
            "rule 'x' is defined but no document references it"
        );
    }

    #[test]
    fn report_lines_are_errors_then_warnings_in_validator_order() {
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
            report.lines(),
            vec![
                "error: no rules defined; a mandate needs at least one".to_string(),
                "error: duplicate rule id 'x'".to_string(),
                "warning: rule 'y' is defined but no document references it".to_string(),
            ]
        );
    }
}
