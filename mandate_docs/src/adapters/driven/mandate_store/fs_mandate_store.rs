//! A [`MandateStore`] backed by the real filesystem, reading and listing
//! `.yaml` files under `<root>/.mandate/mandates/`.

use std::fs;
use std::path::Path;

use crate::domain::ports::driven::mandate_store::{
    MandateFile, MandateStore, StoreError, MANDATES_DIR,
};

pub struct FsMandateStore;

impl MandateStore for FsMandateStore {
    fn list(&self, root: &Path) -> Result<Vec<String>, StoreError> {
        let mandates_dir = root.join(MANDATES_DIR);
        if !mandates_dir.exists() {
            return Ok(Vec::new());
        }

        let read_dir = fs::read_dir(&mandates_dir)
            .map_err(|err| StoreError(format!("{}: {err}", mandates_dir.display())))?;

        let mut names = Vec::new();
        for entry in read_dir {
            let entry =
                entry.map_err(|err| StoreError(format!("{}: {err}", mandates_dir.display())))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                continue;
            }
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                names.push(file_name.to_string());
            }
        }
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
        let text = fs::read_to_string(&path)
            .map_err(|err| StoreError(format!("{}: {err}", path.display())))?;

        Ok(MandateFile {
            file_name: file_name.to_string(),
            text,
        })
    }
}
