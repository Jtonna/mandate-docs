use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn fails_when_code_links_a_doc_this_mandate_does_not_govern() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_fails(
        lines,
        &["code src/todo/task.rs links docs/architecture/other.md which this mandate does not govern"],
    );
}
