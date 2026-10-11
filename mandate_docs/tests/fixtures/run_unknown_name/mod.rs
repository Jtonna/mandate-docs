use crate::support::*;
use mandate::domain::usecases::run_mandates::RunError;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn unknown_name_errors_and_lists_both_available() {
    let mandate_a = parse(MANDATE);
    let text_b = MANDATE.replacen("name: Todo_App", "name: Todo_App_Two", 1);

    let vm = FakeVirtualMachine::with_every_file_in(&mandate_a)
        .with_mandate("a.yaml", MANDATE)
        .with_mandate("b.yaml", &text_b);

    let err = run(&vm, "/repo", &["missing.yaml"]).unwrap_err();

    match err {
        RunError::UnknownMandate { name, available } => {
            assert_eq!(name, "missing.yaml");
            assert_eq!(available, vec!["a.yaml".to_string(), "b.yaml".to_string()]);
        }
        other => panic!("expected UnknownMandate, got {other:?}"),
    }
}
