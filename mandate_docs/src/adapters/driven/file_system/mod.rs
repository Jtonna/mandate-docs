//! Adapter-internal seam shared by the two filesystem-backed driven
//! adapters in `super::mandate_store` and `super::file_tree`. This is not
//! a domain port. It exists only so each adapter can be built over the
//! real OS filesystem or over an in-memory fake in tests, while sharing
//! one filesystem walk between them. The domain and the use case never
//! see this trait.

pub mod os_file_system;

use std::fmt;
use std::path::Path;

use crate::domain::model::file_tree::EntryKind;

/// One directory entry as a file system reports it: a bare name and its
/// kind. The caller joins the name onto the directory it came from to get
/// a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub kind: EntryKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSystemError(pub String);

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The filesystem operations the two adapters in this module's siblings
/// need, factored out so each can run over the real OS filesystem
/// (`os_file_system`) or an in-memory fake in tests.
pub trait FileSystem {
    /// Entries directly inside `dir`, in no guaranteed order. A symlink is
    /// reported as a file, never as a directory. Entries carry bare names;
    /// the caller joins them onto `dir` to get a path. Errors if `dir`
    /// cannot be read.
    fn list_dir(&self, dir: &Path) -> Result<Vec<DirEntry>, FileSystemError>;

    /// The text of the file at `path`.
    fn read_to_string(&self, path: &Path) -> Result<String, FileSystemError>;

    /// Whether `path` exists and is a directory. Never errors: unreadable
    /// or missing answers `false`.
    fn is_dir(&self, path: &Path) -> bool;

    /// Whether `path` exists at all, file or directory. Never errors:
    /// unreadable answers `false`.
    fn exists(&self, path: &Path) -> bool;
}

// So the two sibling adapters can share one filesystem by reference,
// letting a test build both over the same fake.
impl<F: FileSystem + ?Sized> FileSystem for &F {
    fn list_dir(&self, dir: &Path) -> Result<Vec<DirEntry>, FileSystemError> {
        (**self).list_dir(dir)
    }

    fn read_to_string(&self, path: &Path) -> Result<String, FileSystemError> {
        (**self).read_to_string(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        (**self).is_dir(path)
    }

    fn exists(&self, path: &Path) -> bool {
        (**self).exists(path)
    }
}
