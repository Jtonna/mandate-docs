use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn fails_when_a_rule_id_is_duplicated() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_fails(lines, &["duplicate rule id 'claims-match-code'"]);
}
