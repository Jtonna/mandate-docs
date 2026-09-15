use crate::support::*;
use mandate::domain::run_report::RunLocation;

#[test]
fn no_mandate_folder_anywhere_up_warns_and_reports_zero() {
    let vm = FakeVirtualMachine::empty();

    // The virtual machine only answers `has_directory` for its own root,
    // "/repo" (see support.rs), so starting from "/elsewhere" walks up to
    // the filesystem root without ever finding a ".mandate" entry.
    let report = run(&vm, "/elsewhere", &[]).unwrap();

    assert_eq!(
        report.location,
        RunLocation::NotFound {
            searched_from: "/elsewhere".to_string()
        }
    );
    assert_eq!(
        report.lines(),
        vec![
            "warning: no .mandate folder found from /elsewhere up to the \
             filesystem root"
                .to_string(),
            String::new(),
            "0 mandates checked, 0 invalid".to_string(),
        ]
    );
    assert!(report.is_valid());
}
