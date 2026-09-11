use crate::fake_repo::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn missing_governed_doc_fails() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate).remove("docs/sop/handling-mandates.md");

    let lines = check(&mandate, &repo);

    assert_fails(
        lines,
        &["governed document not found: docs/sop/handling-mandates.md"],
    );
}

#[test]
fn adding_the_file_back_passes() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate)
        .remove("docs/sop/handling-mandates.md")
        .add("docs/sop/handling-mandates.md");

    let lines = check(&mandate, &repo);

    assert_passes(lines);
}
