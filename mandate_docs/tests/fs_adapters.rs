//! Integration tests for the filesystem adapters: the real-disk
//! `FileTreeSource` and `MandateStore` implementations. Each test builds an
//! isolated tempdir tree, exercises the adapter, and asserts on the result.

use std::fs;
use std::path::Path;

use mandate::adapters::driven::file_system::os_file_system::OsFileSystem;
use mandate::adapters::driven::file_system::{DirEntry, FileSystem};
use mandate::adapters::driven::file_tree::fs_file_tree_source::FsFileTreeSource;
use mandate::adapters::driven::mandate_store::fs_mandate_store::FsMandateStore;
use mandate::domain::model::file_tree::EntryKind;
use mandate::domain::ports::driven::file_tree_source::FileTreeSource;
use mandate::domain::ports::driven::mandate_store::MandateStore;

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

    let store = FsMandateStore::new(OsFileSystem);
    let names = store.list(dir.path()).expect("list");

    assert_eq!(names, vec!["a.yaml".to_string(), "b.yaml".to_string()]);
}

#[test]
fn list_with_no_mandates_dir_returns_empty() {
    let dir = tempfile::tempdir().expect("tempdir");

    let store = FsMandateStore::new(OsFileSystem);
    let names = store.list(dir.path()).expect("list");

    assert!(names.is_empty());
}

#[test]
fn read_returns_the_text() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mandates_dir = dir.path().join(".mandate/mandates");
    write_file(&mandates_dir.join("a.yaml"), "name: Test\n");

    let store = FsMandateStore::new(OsFileSystem);
    let mandate_file = store.read(dir.path(), "a.yaml").expect("read");

    assert_eq!(mandate_file.file_name, "a.yaml");
    assert_eq!(mandate_file.text, "name: Test\n");
}

#[test]
fn read_of_a_missing_file_errs_naming_the_path() {
    let dir = tempfile::tempdir().expect("tempdir");

    let store = FsMandateStore::new(OsFileSystem);
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

    let store = FsMandateStore::new(OsFileSystem);
    let err = store
        .read(dir.path(), "../x.yaml")
        .expect_err("should error");

    assert!(!err.0.is_empty());
}

#[test]
fn list_errs_when_the_mandates_path_is_a_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(&dir.path().join(".mandate/mandates"), "not a directory");

    let store = FsMandateStore::new(OsFileSystem);
    let err = store.list(dir.path()).expect_err("should error");

    assert!(!err.0.is_empty());
}

#[test]
fn read_with_a_path_separator_in_the_file_name_errs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mandates_dir = dir.path().join(".mandate/mandates");
    write_file(&mandates_dir.join("sub/a.yaml"), "a");

    let store = FsMandateStore::new(OsFileSystem);
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

    let source = FsFileTreeSource::new(OsFileSystem);
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

    let source = FsFileTreeSource::new(OsFileSystem);
    let snapshot = source.snapshot(dir.path()).expect("snapshot");

    assert!(snapshot.is_empty());
}

#[test]
fn snapshot_matching_is_exact_case_not_folded() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(&dir.path().join("docs/a.md"), "a");

    let source = FsFileTreeSource::new(OsFileSystem);
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

    let source = FsFileTreeSource::new(OsFileSystem);
    let snapshot = source.snapshot(dir.path()).expect("snapshot");

    assert!(snapshot.is_dir(".hidden"));
    assert!(snapshot.is_file(".hidden/x"));
}

// An unreadable-directory test (permissions denied mid-walk) is skipped:
// tempfile dirs on Windows do not reliably support removing read access to
// prove a permission-denied read on this filesystem, so there is no
// portable way to construct that case here.

// --- OsFileSystem ------------------------------------------------------

#[test]
fn list_dir_reports_files_and_directories() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(&dir.path().join("a.txt"), "a");
    fs::create_dir_all(dir.path().join("sub")).expect("create sub");

    let file_system = OsFileSystem;
    let mut entries = file_system.list_dir(dir.path()).expect("list_dir");
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(
        entries,
        vec![
            DirEntry {
                name: "a.txt".to_string(),
                kind: EntryKind::File,
            },
            DirEntry {
                name: "sub".to_string(),
                kind: EntryKind::Directory,
            },
        ]
    );
}

#[test]
fn is_dir_is_false_for_a_missing_path() {
    let dir = tempfile::tempdir().expect("tempdir");

    let file_system = OsFileSystem;

    assert!(!file_system.is_dir(&dir.path().join("does-not-exist")));
}

#[test]
fn read_to_string_of_a_missing_file_errs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("missing.txt");

    let file_system = OsFileSystem;
    let err = file_system
        .read_to_string(&missing)
        .expect_err("should error");

    assert!(err.0.contains(&missing.display().to_string()));
}
