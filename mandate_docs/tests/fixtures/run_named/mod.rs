use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn selecting_one_name_runs_only_it() {
    let mandate_a = parse(MANDATE);
    let text_b = MANDATE.replacen("name: Todo_App", "name: Todo_App_Two", 1);

    let vm = FakeVirtualMachine::with_every_file_in(&mandate_a)
        .with_mandate("a.yaml", MANDATE)
        .with_mandate("b.yaml", &text_b);

    let report = run(&vm, "/repo", &["b.yaml"]).unwrap();

    assert_eq!(report.mandates.len(), 1);
    assert_eq!(report.mandates[0].file_name, "b.yaml");
    assert!(report.is_valid());
}
