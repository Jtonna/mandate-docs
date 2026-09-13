use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn fails_when_a_document_references_an_undefined_rule() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_fails(
        lines,
        &["rule 'no-such-rule' is referenced by docs/architecture/mandate-parser.md but not defined"],
    );
}
