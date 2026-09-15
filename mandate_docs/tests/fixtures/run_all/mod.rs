use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn two_mandates_run_and_one_is_invalid() {
    let mandate_a = parse(MANDATE);
    let text_b = MANDATE
        .replacen("name: Todo_App", "name: Todo_App_Two", 1)
        .replacen("src/todo/task.rs", "src/todo/only_in_b.rs", 1);

    let vm = FakeVirtualMachine::with_every_file_in(&mandate_a)
        .with_mandate("a.yaml", MANDATE)
        .with_mandate("b.yaml", &text_b);

    let report = run(&vm, "/repo", &[]).unwrap();

    assert_eq!(report.mandates.len(), 2);
    assert!(!report.is_valid());
    assert_eq!(
        report.lines(),
        vec![
            "repository: /repo".to_string(),
            "snapshot: 30 entries".to_string(),
            String::new(),
            "a.yaml".to_string(),
            "  valid: 3 rules, 2 documents, 11 source files".to_string(),
            String::new(),
            "b.yaml".to_string(),
            "  error: missing source file: src/todo/only_in_b.rs".to_string(),
            "  invalid: 1 errors, 0 warnings".to_string(),
            String::new(),
            "2 mandates checked, 1 invalid".to_string(),
        ]
    );
}
