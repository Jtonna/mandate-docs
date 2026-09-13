use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn fails_when_no_rules_are_defined() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_fails(lines, &["no rules defined; a mandate needs at least one"]);
}
