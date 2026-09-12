use crate::fake_virtual_machine::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn missing_code_files_fail() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate)
        .remove("src/main.rs")
        .remove("tests/validation_fixtures.rs");

    let lines = check(&mandate, &vm);

    assert_fails(
        lines,
        &[
            "missing source file: src/main.rs",
            "missing source file: tests/validation_fixtures.rs",
        ],
    );
}
