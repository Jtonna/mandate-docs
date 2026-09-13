use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn passes_with_a_warning_when_a_rule_is_unreferenced() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_passes_with_warnings(
        lines,
        &["rule 'unused-rule' is defined but no document references it"],
    );
}

#[test]
fn passes_clean_once_the_rule_is_referenced() {
    let mut mandate = parse(MANDATE);
    mandate.governs[0].rules.push("unused-rule".to_string());
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_passes(lines);
}
