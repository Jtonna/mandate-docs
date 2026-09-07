//! Fixture-driven validation tests. Walks every case under
//! `tests/fixtures/validation/`, parses its mandate through the public API,
//! validates it against each of the three OS-flavoured trees, and compares
//! the rendered report (sorted) against the case's `expected.txt`.

mod common;

use std::fs;
use std::path::PathBuf;

use mandate::adapters::yaml::parse_mandate;
use mandate::domain::validation::validate;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/validation")
}

#[test]
fn every_fixture_case_matches_its_expected_report_on_every_os() {
    let dir = fixtures_dir();
    let mut case_dirs: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    case_dirs.sort();

    assert!(
        case_dirs.len() >= 9,
        "expected at least 9 fixture cases under {}, found {}",
        dir.display(),
        case_dirs.len()
    );

    for case_dir in &case_dirs {
        let case = common::load_case(case_dir);

        let mandate = parse_mandate(&case.mandate_yaml).unwrap_or_else(|e| {
            panic!(
                "fixture case {}: mandate.yaml failed to parse: {e}",
                case.name
            )
        });

        for (os, tree_text) in &case.trees {
            let tree = common::load_tree(tree_text);
            let report = validate(&mandate, &tree);

            let mut actual_lines: Vec<String> = report
                .errors
                .iter()
                .map(|e| format!("error: {e}"))
                .chain(report.warnings.iter().map(|w| format!("warning: {w}")))
                .collect();
            actual_lines.sort();

            let mut expected_lines = case.expected_lines.clone();
            expected_lines.sort();

            assert_eq!(
                actual_lines, expected_lines,
                "fixture case {} on {os}: report lines did not match expected.txt",
                case.name
            );
            assert_eq!(
                report.is_valid(),
                case.expected_pass,
                "fixture case {} on {os}: expected pass={}, got is_valid()={}",
                case.name,
                case.expected_pass,
                report.is_valid()
            );
        }
    }
}
