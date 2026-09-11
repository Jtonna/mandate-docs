use crate::fake_repo::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn duplicate_rule_id_fails() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate);

    let lines = check(&mandate, &repo);

    assert_fails(lines, &["duplicate rule id 'claims-match-code'"]);
}
