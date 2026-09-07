//! The `FileTree` port contract: every implementation must agree on what
//! `exists` means for a known path and an unknown one.

use std::fs;

use mandate::adapters::fs_tree::FsFileTree;
use mandate::adapters::memory_tree::InMemoryFileTree;
use mandate::domain::ports::file_tree::FileTree;

fn file_tree_contract(tree: &dyn FileTree) {
    assert!(tree.exists("docs/a.md"), "known path should exist");
    assert!(
        !tree.exists("docs/does-not-exist.md"),
        "unknown path should not exist"
    );
}

#[test]
fn in_memory_file_tree_satisfies_contract() {
    let tree = InMemoryFileTree::new(["docs/a.md"]);
    file_tree_contract(&tree);
}

#[test]
fn fs_file_tree_satisfies_contract() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("docs")).expect("create docs dir");
    fs::write(dir.path().join("docs/a.md"), "content").expect("write file");

    let tree = FsFileTree::new(dir.path());
    file_tree_contract(&tree);
}
