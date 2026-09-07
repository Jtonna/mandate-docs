//! A [`FileTree`] backed by the real filesystem, rooted at a directory.

use std::path::PathBuf;

use crate::domain::ports::file_tree::FileTree;

pub struct FsFileTree {
    pub root: PathBuf,
}

impl FsFileTree {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl FileTree for FsFileTree {
    fn exists(&self, repo_relative_path: &str) -> bool {
        self.root.join(repo_relative_path).exists()
    }
}
