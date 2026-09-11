use crate::fake_repo::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn unreferenced_rule_is_warning_not_error() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate);

    let lines = check(&mandate, &repo);

    assert_passes_with_warnings(
        lines,
        &["rule 'unused-rule' is defined but no document references it"],
    );
}

#[test]
fn referencing_the_rule_removes_the_warning() {
    let mut mandate = parse(MANDATE);
    mandate.governs[0].rules.push("unused-rule".to_string());
    let repo = FakeRepo::with_every_file_in(&mandate);

    let lines = check(&mandate, &repo);

    assert_passes(lines);
}
