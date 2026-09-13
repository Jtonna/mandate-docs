//! Integration tests for the filesystem adapters: the real-disk
//! `FileTreeSource` and `MandateStore` implementations. Each test builds an
//! isolated tempdir tree, exercises the adapter, and asserts on the result.

use std::fs;
use std::path::Path;

use mandate::adapters::fs_file_tree_source::FsFileTreeSource;
use mandate::adapters::fs_mandate_store::FsMandateStore;
use mandate::domain::ports::file_tree_source::FileTreeSource;
use mandate::domain::ports::mandate_store::MandateStore;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

// --- FsMandateStore ------------------------------------------------------

#[test]
fn list_returns_only_yaml_files_sorted() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mandates_dir = dir.path().join(".mandate/mandates");
    write_file(&mandates_dir.join("b.yaml"), "b");
    write_file(&mandates_dir.join("a.yaml"), "a");
    write_file(&mandates_dir.join("c.yml"), "c");
    write_file(&mandates_dir.join("notes.txt"), "notes");
    write_file(&mandates_dir.join("sub/d.yaml"), "d");

    let store = FsMandateStore;
    let names = store.list(dir.path()).expect("list");

    assert_eq!(names, vec!["a.yaml".to_string(), "b.yaml".to_string()]);
}

#[test]
fn list_with_no_mandates_dir_returns_empty() {
    let dir = tempfile::tempdir().expect("tempdir");

    let store = FsMandateStore;
    let names = store.list(dir.path()).expect("list");

    assert!(names.is_empty());
}

#[test]
fn read_returns_the_text() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mandates_dir = dir.path().join(".mandate/mandates");
    write_file(&mandates_dir.join("a.yaml"), "name: Test\n");

    let store = FsMandateStore;
    let mandate_file = store.read(dir.path(), "a.yaml").expect("read");

    assert_eq!(mandate_file.file_name, "a.yaml");
    assert_eq!(mandate_file.text, "name: Test\n");
}

#[test]
fn read_of_a_missing_file_errs_naming_the_path() {
    let dir = tempfile::tempdir().expect("tempdir");

    let store = FsMandateStore;
    let err = store
        .read(dir.path(), "missing.yaml")
        .expect_err("should error");

    let expected_path = dir.path().join(".mandate/mandates").join("missing.yaml");
    assert!(
        err.0.contains(&expected_path.display().to_string()),
        "error {:?} should name the path {}",
        err,
        expected_path.display()
    );
}

#[test]
fn read_with_path_traversal_errs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mandates_dir = dir.path().join(".mandate/mandates");
    fs::create_dir_all(&mandates_dir).expect("create mandates dir");
    write_file(&dir.path().join("x.yaml"), "escaped");

    let store = FsMandateStore;
    let err = store
        .read(dir.path(), "../x.yaml")
        .expect_err("should error");

    assert!(!err.0.is_empty());
}

#[test]
fn read_with_a_path_separator_in_the_file_name_errs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mandates_dir = dir.path().join(".mandate/mandates");
    write_file(&mandates_dir.join("sub/a.yaml"), "a");

    let store = FsMandateStore;
    let err = store
        .read(dir.path(), "sub/a.yaml")
        .expect_err("should error");

    assert!(!err.0.is_empty());
}

// --- FsFileTreeSource ------------------------------------------------------

#[test]
fn snapshot_of_a_tree_matches_exactly() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(&dir.path().join("docs/a.md"), "a");
    write_file(&dir.path().join("docs/sub/b.md"), "b");
    write_file(&dir.path().join("src/c.ts"), "c");
    write_file(&dir.path().join("top.txt"), "top");

    let source = FsFileTreeSource;
    let snapshot = source.snapshot(dir.path()).expect("snapshot");

    assert!(snapshot.is_dir("docs"));
    assert!(snapshot.is_dir("docs/sub"));
    assert!(snapshot.is_dir("src"));
    assert!(snapshot.is_file("docs/a.md"));
    assert!(snapshot.is_file("docs/sub/b.md"));
    assert!(snapshot.is_file("src/c.ts"));
    assert!(snapshot.is_file("top.txt"));
    assert_eq!(snapshot.len(), 7);

    let mut paths: Vec<&str> = snapshot.paths().collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        vec![
            "docs",
            "docs/a.md",
            "docs/sub",
            "docs/sub/b.md",
            "src",
            "src/c.ts",
            "top.txt",
        ]
    );
}

#[test]
fn snapshot_of_an_empty_root_is_empty() {
    let dir = tempfile::tempdir().expect("tempdir");

    let source = FsFileTreeSource;
    let snapshot = source.snapshot(dir.path()).expect("snapshot");

    assert!(snapshot.is_empty());
}

#[test]
fn snapshot_matching_is_exact_case_not_folded() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(&dir.path().join("docs/a.md"), "a");

    let source = FsFileTreeSource;
    let snapshot = source.snapshot(dir.path()).expect("snapshot");

    // Proves exact-name matching, not the OS's case-insensitive lookup:
    // Windows would happily open "Docs/A.md" on disk, but the snapshot
    // must not report it as present under a different case.
    assert!(snapshot.exists("docs/a.md"));
    assert!(!snapshot.exists("Docs/A.md"));
}

#[test]
fn hidden_directories_are_included() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(&dir.path().join(".hidden/x"), "x");

    let source = FsFileTreeSource;
    let snapshot = source.snapshot(dir.path()).expect("snapshot");

    assert!(snapshot.is_dir(".hidden"));
    assert!(snapshot.is_file(".hidden/x"));
}

// An unreadable-directory test (permissions denied mid-walk) is skipped:
// tempfile dirs on Windows do not reliably support removing read access to
// prove a permission-denied read on this filesystem, so there is no
// portable way to construct that case here.
