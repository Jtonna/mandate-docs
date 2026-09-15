use crate::support::*;
use mandate::domain::run_report::RunLocation;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn starts_below_root_and_finds_it_by_walking_up() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate)
        .with_mandate("a.yaml", MANDATE)
        .at_root("/repo");

    let report = run(&vm, "/repo/src", &[]).unwrap();

    match &report.location {
        RunLocation::Found { root, .. } => assert_eq!(root, "/repo"),
        other => panic!("expected Found, got {other:?}"),
    }
    assert_eq!(report.mandates.len(), 1);
    assert!(report.is_valid());
}
