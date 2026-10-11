//! [`OsFileSystem`]: the real-disk [`FileSystem`](super::FileSystem)
//! implementation, backing both filesystem driven adapters in production.

use std::fs;
use std::path::Path;

use super::{DirEntry, FileSystem, FileSystemError};
use crate::domain::model::file_tree::EntryKind;

/// The real operating system filesystem. Carries no state, so it is
/// `Copy` and can be handed to both filesystem adapters by value.
#[derive(Debug, Clone, Copy)]
pub struct OsFileSystem;

impl FileSystem for OsFileSystem {
    fn list_dir(&self, dir: &Path) -> Result<Vec<DirEntry>, FileSystemError> {
        let read_dir = fs::read_dir(dir)
            .map_err(|err| FileSystemError(format!("{}: {err}", dir.display())))?;

        let mut entries = Vec::new();
        for entry in read_dir {
            let entry =
                entry.map_err(|err| FileSystemError(format!("{}: {err}", dir.display())))?;
            let path = entry.path();
            // `symlink_metadata` never follows a symlink, so a symlink is
            // always reported as a file, never traversed as a directory.
            let metadata = fs::symlink_metadata(&path)
                .map_err(|err| FileSystemError(format!("{}: {err}", path.display())))?;
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let kind = if metadata.is_dir() {
                EntryKind::Directory
            } else {
                EntryKind::File
            };
            entries.push(DirEntry {
                name: name.to_string(),
                kind,
            });
        }
        Ok(entries)
    }

    fn read_to_string(&self, path: &Path) -> Result<String, FileSystemError> {
        fs::read_to_string(path)
            .map_err(|err| FileSystemError(format!("{}: {err}", path.display())))
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
