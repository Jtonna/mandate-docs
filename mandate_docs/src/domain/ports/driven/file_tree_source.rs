//! The port the domain uses to obtain a [`FileTreeSnapshot`] of a repository
//! root at a point in time.

use std::fmt;
use std::path::Path;

use crate::domain::file_tree::FileTreeSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceError(pub String);

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait FileTreeSource {
    fn snapshot(&self, root: &Path) -> Result<FileTreeSnapshot, SourceError>;

    /// Whether `dir` contains a directory entry named `name`, without
    /// building a full snapshot. Returns `Ok(false)` rather than an error
    /// when `dir` itself cannot be read (missing, no permission, ...): an
    /// unreadable ancestor is not a reason to stop looking further up, only
    /// a reason it cannot be the root.
    fn has_directory(&self, dir: &Path, name: &str) -> Result<bool, SourceError>;
}
