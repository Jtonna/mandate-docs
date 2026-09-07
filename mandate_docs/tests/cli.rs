//! Integration tests for the `mandate` binary: runs the real executable
//! against a temporary directory standing in for a repository root.

use std::fs;
use std::process::Command;

fn write_minimal_mandate(root: &std::path::Path) {
    fs::create_dir_all(root.join("docs")).expect("create docs dir");
    fs::create_dir_all(root.join("src")).expect("create src dir");

    fs::write(
        root.join("mandate.yaml"),
        r#"
name: Minimal
rules:
  - id: has-owner
    type: script
    run: ./check.sh
governs:
  - doc: docs/a.md
    rules:
      - has-owner
code:
  - path: src/a.ts
    docs:
      - docs/a.md
"#,
    )
    .expect("write mandate.yaml");
}

#[test]
fn cli_valid_mandate_exits_zero_and_prints_summary() {
    let dir = tempfile::tempdir().expect("create temp dir");
    write_minimal_mandate(dir.path());
    fs::write(dir.path().join("docs/a.md"), "content").expect("write doc");
    fs::write(dir.path().join("src/a.ts"), "content").expect("write code");

    let output = Command::new(env!("CARGO_BIN_EXE_mandate"))
        .arg("validate")
        .arg(dir.path().join("mandate.yaml"))
        .arg("--root")
        .arg(dir.path())
        .output()
        .expect("run mandate binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout: {stdout}");
    assert!(
        stdout.contains("is valid: 1 rules, 1 documents, 1 source files"),
        "stdout: {stdout}"
    );
}

#[test]
fn cli_missing_files_exit_one_and_list_errors() {
    let dir = tempfile::tempdir().expect("create temp dir");
    write_minimal_mandate(dir.path());
    // Deliberately do not write docs/a.md or src/a.ts.

    let output = Command::new(env!("CARGO_BIN_EXE_mandate"))
        .arg("validate")
        .arg(dir.path().join("mandate.yaml"))
        .arg("--root")
        .arg(dir.path())
        .output()
        .expect("run mandate binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "stdout: {stdout}");
    assert!(
        stdout.contains("governed document not found:"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("missing source file:"), "stdout: {stdout}");
}

#[test]
fn cli_unparseable_mandate_exits_one() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(
        dir.path().join("mandate.yaml"),
        r#"
name: Bad
rules:
  - id: has-owner
    type: script
    run: ./check.sh
governs: []
code: []
bogus_top_level_field: true
"#,
    )
    .expect("write mandate.yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_mandate"))
        .arg("validate")
        .arg(dir.path().join("mandate.yaml"))
        .arg("--root")
        .arg(dir.path())
        .output()
        .expect("run mandate binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(stderr.contains("failed to parse"), "stderr: {stderr}");
}

#[test]
fn cli_bad_arguments_print_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_mandate"))
        .output()
        .expect("run mandate binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(stderr.contains("usage:"), "stderr: {stderr}");
}
