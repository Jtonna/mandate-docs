use crate::fake_virtual_machine::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn code_linking_ungoverned_doc_fails() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_fails(
        lines,
        &["code src/domain/mandate.rs links docs/architecture/other.md which this mandate does not govern"],
    );
}
