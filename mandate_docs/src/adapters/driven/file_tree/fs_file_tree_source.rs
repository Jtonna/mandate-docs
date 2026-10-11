//! A [`FileTreeSource`] backed by a [`FileSystem`], walking recursively
//! from a root directory. Symlinks are not followed (a symlink is
//! recorded as a file, never traversed as a directory) because the
//! underlying `FileSystem::list_dir` reports them as files.

use std::path::Path;

use crate::adapters::driven::file_system::FileSystem;
use crate::domain::model::file_tree::{EntryKind, FileTreeSnapshot};
use crate::domain::ports::driven::file_tree_source::{FileTreeSource, SourceError};

pub struct FsFileTreeSource<F: FileSystem> {
    fs: F,
}

impl<F: FileSystem> FsFileTreeSource<F> {
    pub fn new(fs: F) -> Self {
        Self { fs }
    }
}

impl<F> FileTreeSource for FsFileTreeSource<F>
where
    F: FileSystem,
{
    fn snapshot(&self, root: &Path) -> Result<FileTreeSnapshot, SourceError> {
        let mut snapshot = FileTreeSnapshot::empty();
        self.walk(root, root, &mut snapshot)?;
        Ok(snapshot)
    }

    fn has_directory(&self, dir: &Path, name: &str) -> Result<bool, SourceError> {
        // `FileSystem::is_dir` reports `false` on a missing path or a
        // permission error rather than raising one, which is exactly the
        // "never walk, just answer false" contract the port asks for.
        Ok(self.fs.is_dir(&dir.join(name)))
    }
}

impl<F> FsFileTreeSource<F>
where
    F: FileSystem,
{
    fn walk(
        &self,
        root: &Path,
        dir: &Path,
        snapshot: &mut FileTreeSnapshot,
    ) -> Result<(), SourceError> {
        let entries = self
            .fs
            .list_dir(dir)
            .map_err(|err| SourceError(err.to_string()))?;

        for entry in entries {
            let path = dir.join(&entry.name);
            let relative_path = relative_forward_slash(root, &path);

            match entry.kind {
                EntryKind::Directory => {
                    snapshot.insert(relative_path, EntryKind::Directory);
                    self.walk(root, &path, snapshot)?;
                }
                EntryKind::File => {
                    snapshot.insert(relative_path, EntryKind::File);
                }
            }
        }

        Ok(())
    }
}

fn relative_forward_slash(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
