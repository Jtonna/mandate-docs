use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn starts_below_root_and_finds_it_by_walking_up() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate)
        .with_mandate("a.yaml", MANDATE)
        .at_root("/repo");

    let report = run(&vm, "/repo/src", &[]).unwrap();

    assert_eq!(report.root, "/repo");
    assert_eq!(report.mandates.len(), 1);
    assert!(report.is_valid());
}
