use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn fails_when_linked_source_files_are_missing() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate)
        .remove("src/main.rs")
        .remove("tests/fixtures/doc_missing/mod.rs");

    let lines = check(&mandate, &vm);

    assert_fails(
        lines,
        &[
            "missing source file: src/main.rs",
            "missing source file: tests/fixtures/doc_missing/mod.rs",
        ],
    );
}
