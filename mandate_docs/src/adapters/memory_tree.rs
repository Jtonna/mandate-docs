//! An in-memory [`FileTree`]. The fake used in domain tests, and also a real
//! adapter in its own right (per the architecture guide, a hand-rolled fake
//! can double as a local-dev adapter).

use std::collections::HashSet;

use crate::domain::ports::file_tree::FileTree;

pub struct InMemoryFileTree {
    paths: HashSet<String>,
}

impl InMemoryFileTree {
    pub fn new<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            paths: paths.into_iter().map(Into::into).collect(),
        }
    }
}

impl FileTree for InMemoryFileTree {
    fn exists(&self, repo_relative_path: &str) -> bool {
        self.paths.contains(repo_relative_path)
    }
}
