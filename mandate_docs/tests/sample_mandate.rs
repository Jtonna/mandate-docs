//! Integration tests exercising the public API against the real sample
//! mandate committed at `.mandate/mandates/SOP_Orders.yaml`.

use mandate::adapters::memory_tree::InMemoryFileTree;
use mandate::adapters::yaml::parse_mandate;
use mandate::domain::mandate::RuleKind;
use mandate::domain::validation::{validate, ValidationError};

const SAMPLE: &str = include_str!("../.mandate/mandates/SOP_Orders.yaml");

#[test]
fn parse_valid_sample() {
    let mandate = parse_mandate(SAMPLE).expect("sample mandate should parse");

    assert_eq!(mandate.name, "SOP_Orders");
    assert_eq!(mandate.rules.len(), 4);
    assert_eq!(mandate.governs.len(), 2);
    assert_eq!(mandate.code.len(), 3);

    assert!(matches!(mandate.rules[0].kind, RuleKind::Script { .. }));
    assert!(matches!(mandate.rules[1].kind, RuleKind::Script { .. }));
    assert!(matches!(mandate.rules[2].kind, RuleKind::Agent { .. }));
    assert!(matches!(mandate.rules[3].kind, RuleKind::Agent { .. }));
}

#[test]
fn sample_mandate_is_valid_when_every_linked_file_exists() {
    let mandate = parse_mandate(SAMPLE).expect("sample mandate should parse");
    let tree = InMemoryFileTree::new([
        "docs/architecture/order-lifecycle.md",
        "docs/architecture/http-api.md",
        "src/domain/order.ts",
        "src/application/place-order.ts",
        "src/adapters/http/order-controller.ts",
    ]);

    let report = validate(&mandate, &tree);

    assert!(report.is_valid(), "{:?}", report.errors);
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
}

#[test]
fn sample_mandate_reports_every_missing_file_when_tree_is_empty() {
    let mandate = parse_mandate(SAMPLE).expect("sample mandate should parse");
    let tree = InMemoryFileTree::new(Vec::<String>::new());

    let report = validate(&mandate, &tree);

    let doc_missing_count = report
        .errors
        .iter()
        .filter(|e| matches!(e, ValidationError::DocMissing { .. }))
        .count();
    let code_missing_count = report
        .errors
        .iter()
        .filter(|e| matches!(e, ValidationError::CodeMissing { .. }))
        .count();

    assert_eq!(doc_missing_count, 2, "{:?}", report.errors);
    assert_eq!(code_missing_count, 3, "{:?}", report.errors);
}
