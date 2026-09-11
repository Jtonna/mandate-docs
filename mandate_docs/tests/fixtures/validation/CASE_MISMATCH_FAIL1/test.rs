use crate::fake_repo::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn wrong_case_path_fails() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate).rename(
        "docs/sop/handling-mandates.md",
        "docs/sop/Handling-Mandates.md",
    );

    let lines = check(&mandate, &repo);

    assert_fails(
        lines,
        &["governed document not found: docs/sop/handling-mandates.md"],
    );
}

#[test]
fn exact_lowercase_path_passes() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate);

    let lines = check(&mandate, &repo);

    assert_passes(lines);
}
