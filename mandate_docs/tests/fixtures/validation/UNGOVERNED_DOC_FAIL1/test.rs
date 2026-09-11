use crate::fake_repo::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn code_linking_ungoverned_doc_fails() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate);

    let lines = check(&mandate, &repo);

    assert_fails(
        lines,
        &["code src/domain/mandate.rs links docs/architecture/other.md which this mandate does not govern"],
    );
}
