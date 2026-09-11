use crate::fake_repo::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn missing_code_files_fail() {
    let mandate = parse(MANDATE);
    let repo = FakeRepo::with_every_file_in(&mandate)
        .remove("src/main.rs")
        .remove("tests/validation_fixtures.rs");

    let lines = check(&mandate, &repo);

    assert_fails(
        lines,
        &[
            "missing source file: src/main.rs",
            "missing source file: tests/validation_fixtures.rs",
        ],
    );
}
