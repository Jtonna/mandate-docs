use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn passes_when_every_linked_file_is_present() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_passes(lines);
}
