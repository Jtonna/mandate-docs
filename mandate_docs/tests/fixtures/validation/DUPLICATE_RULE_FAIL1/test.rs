use crate::fake_virtual_machine::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn duplicate_rule_id_fails() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_fails(lines, &["duplicate rule id 'claims-match-code'"]);
}
