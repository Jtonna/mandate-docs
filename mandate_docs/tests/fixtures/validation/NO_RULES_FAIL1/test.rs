use crate::fake_repo::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn no_rules_fails() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate);

    let lines = check(&mandate, &repo);

    assert_fails(lines, &["no rules defined; a mandate needs at least one"]);
}
