use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn fails_when_a_governed_doc_is_missing() {
    let mandate = parse(MANDATE);
    let vm =
        FakeVirtualMachine::with_every_file_in(&mandate).remove("docs/sop/handling-mandates.md");

    let lines = check(&mandate, &vm);

    assert_fails(
        lines,
        &["governed document not found: docs/sop/handling-mandates.md"],
    );
}

#[test]
fn passes_once_the_doc_is_added_back() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate)
        .remove("docs/sop/handling-mandates.md")
        .add("docs/sop/handling-mandates.md");

    let lines = check(&mandate, &vm);

    assert_passes(lines);
}
