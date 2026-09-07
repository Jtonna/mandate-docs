//! The port the domain uses to ask whether a repository-relative path exists.

pub trait FileTree {
    fn exists(&self, repo_relative_path: &str) -> bool;
}
