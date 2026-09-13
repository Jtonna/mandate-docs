//! A [`FileTreeSource`] backed by the real filesystem, rooted at a
//! directory. Walks recursively with `std::fs`; symlinks are not followed
//! (a symlink is recorded as a file, never traversed as a directory).

use std::fs;
use std::path::Path;

use crate::domain::file_tree::{EntryKind, FileTreeSnapshot};
use crate::domain::ports::file_tree_source::{FileTreeSource, SourceError};

pub struct FsFileTreeSource;

impl FileTreeSource for FsFileTreeSource {
    fn snapshot(&self, root: &Path) -> Result<FileTreeSnapshot, SourceError> {
        let mut snapshot = FileTreeSnapshot::empty();
        walk(root, root, &mut snapshot)?;
        Ok(snapshot)
    }

    fn has_directory(&self, dir: &Path, name: &str) -> Result<bool, SourceError> {
        // `Path::is_dir` reports `false` on a missing path or a permission
        // error rather than raising one, which is exactly the "never walk,
        // just answer false" contract the port asks for.
        Ok(dir.join(name).is_dir())
    }
}

fn walk(root: &Path, dir: &Path, snapshot: &mut FileTreeSnapshot) -> Result<(), SourceError> {
    let read_dir =
        fs::read_dir(dir).map_err(|err| SourceError(format!("{}: {err}", dir.display())))?;

    for entry in read_dir {
        let entry = entry.map_err(|err| SourceError(format!("{}: {err}", dir.display())))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|err| SourceError(format!("{}: {err}", path.display())))?;
        let relative_path = relative_forward_slash(root, &path);

        if metadata.is_dir() {
            snapshot.insert(relative_path, EntryKind::Directory);
            walk(root, &path, snapshot)?;
        } else {
            // Files, and symlinks (symlink_metadata never reports a
            // symlink itself as a directory), are recorded as files.
            snapshot.insert(relative_path, EntryKind::File);
        }
    }

    Ok(())
}

fn relative_forward_slash(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
