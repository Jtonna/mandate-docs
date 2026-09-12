use crate::fake_virtual_machine::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn no_rules_fails() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_fails(lines, &["no rules defined; a mandate needs at least one"]);
}
