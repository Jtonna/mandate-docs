use crate::fake_virtual_machine::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn unknown_rule_reference_fails() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_fails(
        lines,
        &["rule 'no-such-rule' is referenced by docs/architecture/mandate-parser.md but not defined"],
    );
}
