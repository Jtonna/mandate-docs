use crate::fake_repo::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn unknown_rule_reference_fails() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate);

    let lines = check(&mandate, &repo);

    assert_fails(
        lines,
        &["rule 'no-such-rule' is referenced by docs/architecture/mandate-parser.md but not defined"],
    );
}
