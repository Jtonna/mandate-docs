use crate::support::*;

const MANDATE: &str = include_str!("mandate.yaml");

#[test]
fn fails_when_only_the_case_differs() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate)
        .rename("docs/sop/adding-a-task.md", "docs/sop/Adding-A-Task.md");

    let lines = check(&mandate, &vm);

    assert_fails(
        lines,
        &["governed document not found: docs/sop/adding-a-task.md"],
    );
}

#[test]
fn passes_with_the_exact_path() {
    let mandate = parse(MANDATE);
    let vm = FakeVirtualMachine::with_every_file_in(&mandate);

    let lines = check(&mandate, &vm);

    assert_passes(lines);
}
