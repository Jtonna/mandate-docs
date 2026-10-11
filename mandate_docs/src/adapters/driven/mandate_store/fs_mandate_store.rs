//! A [`MandateStore`] backed by a [`FileSystem`], reading and listing
//! `.yaml` files under `<root>/.mandate/mandates/`.

use std::ffi::OsStr;
use std::path::Path;

use crate::adapters::driven::file_system::FileSystem;
use crate::domain::model::file_tree::EntryKind;
use crate::domain::ports::driven::mandate_store::{
    MandateFile, MandateStore, StoreError, MANDATES_DIR,
};

pub struct FsMandateStore<F: FileSystem> {
    fs: F,
}

impl<F: FileSystem> FsMandateStore<F> {
    pub fn new(fs: F) -> Self {
        Self { fs }
    }
}

impl<F> MandateStore for FsMandateStore<F>
where
    F: FileSystem,
{
    fn list(&self, root: &Path) -> Result<Vec<String>, StoreError> {
        let mandates_dir = root.join(MANDATES_DIR);
        if !self.fs.exists(&mandates_dir) {
            return Ok(Vec::new());
        }

        let entries = self
            .fs
            .list_dir(&mandates_dir)
            .map_err(|err| StoreError(err.to_string()))?;

        let mut names: Vec<String> = entries
            .into_iter()
            .filter(|entry| {
                entry.kind == EntryKind::File
                    && Path::new(&entry.name).extension() == Some(OsStr::new("yaml"))
            })
            .map(|entry| entry.name)
            .collect();
        names.sort();
        Ok(names)
    }

    fn read(&self, root: &Path, file_name: &str) -> Result<MandateFile, StoreError> {
        if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
            return Err(StoreError(format!(
                "invalid mandate file name: {file_name}"
            )));
        }

        let path = root.join(MANDATES_DIR).join(file_name);
        let text = self
            .fs
            .read_to_string(&path)
            .map_err(|err| StoreError(err.to_string()))?;

        Ok(MandateFile {
            file_name: file_name.to_string(),
            text,
        })
    }
}
