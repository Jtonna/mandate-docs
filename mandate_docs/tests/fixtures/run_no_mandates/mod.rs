use crate::support::*;
use mandate::domain::run_report::RunWarning;

#[test]
fn no_mandate_files_warns_and_reports_zero() {
    let vm = FakeVirtualMachine::empty().empty_mandate_folder();

    let report = run(&vm, "/repo", &[]).unwrap();

    assert_eq!(
        report.warnings,
        vec![RunWarning::NoMandatesFound {
            mandates_dir: "/repo/.mandate/mandates".to_string()
        }]
    );
    assert_eq!(report.mandates.len(), 0);
    assert!(report.is_valid());
}
