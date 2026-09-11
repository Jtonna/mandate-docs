//! Fixture-driven validation tests. Each case under
//! `tests/fixtures/validation/<CASE>/` carries its own `mandate.yaml` and
//! `test.rs`, using the shared [`fake_repo`] helpers to build an in-memory
//! repository and check the rendered report against expectations.

mod fake_repo;

/// Every case directory under `tests/fixtures/validation/` that is wired up
/// below with a `#[path]` mod line. Kept next to those lines so the two stay
/// in sync; [`every_case_directory_is_registered`] catches drift between
/// them (a new directory added without a mod line would otherwise compile
/// and silently run nothing).
const REGISTERED_CASES: &[&str] = &[
    "ALL_LINKS_PRESENT_PASS1",
    "CASE_MISMATCH_FAIL1",
    "CODE_MISSING_FAIL1",
    "DOC_MISSING_FAIL1",
    "DUPLICATE_RULE_FAIL1",
    "NO_RULES_FAIL1",
    "UNGOVERNED_DOC_FAIL1",
    "UNKNOWN_RULE_FAIL1",
    "UNREFERENCED_RULE_PASS1",
];

#[path = "fixtures/validation/ALL_LINKS_PRESENT_PASS1/test.rs"]
mod all_links_present_pass1;

#[path = "fixtures/validation/CASE_MISMATCH_FAIL1/test.rs"]
mod case_mismatch_fail1;

#[path = "fixtures/validation/CODE_MISSING_FAIL1/test.rs"]
mod code_missing_fail1;

#[path = "fixtures/validation/DOC_MISSING_FAIL1/test.rs"]
mod doc_missing_fail1;

#[path = "fixtures/validation/DUPLICATE_RULE_FAIL1/test.rs"]
mod duplicate_rule_fail1;

#[path = "fixtures/validation/NO_RULES_FAIL1/test.rs"]
mod no_rules_fail1;

#[path = "fixtures/validation/UNGOVERNED_DOC_FAIL1/test.rs"]
mod ungoverned_doc_fail1;

#[path = "fixtures/validation/UNKNOWN_RULE_FAIL1/test.rs"]
mod unknown_rule_fail1;

#[path = "fixtures/validation/UNREFERENCED_RULE_PASS1/test.rs"]
mod unreferenced_rule_pass1;

/// Guards against a case directory existing on disk without a matching
/// `#[path]` mod line above: such a directory compiles fine and its tests
/// simply never run, silently.
#[test]
fn every_case_directory_is_registered() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/validation");

    let mut found: Vec<String> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {dir}: {e}"))
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();

    let mut registered: Vec<String> = REGISTERED_CASES.iter().map(|s| s.to_string()).collect();
    registered.sort();

    let unregistered: Vec<&String> = found.iter().filter(|f| !registered.contains(f)).collect();
    let missing: Vec<&String> = registered.iter().filter(|r| !found.contains(r)).collect();

    assert!(
        unregistered.is_empty() && missing.is_empty(),
        "case directories and REGISTERED_CASES disagree: \
         directories with no #[path] mod line: {unregistered:?}; \
         registered names with no directory: {missing:?}"
    );
}
