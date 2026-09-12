use crate::fake_virtual_machine::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn all_files_present_passes() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_passes(lines);
}
