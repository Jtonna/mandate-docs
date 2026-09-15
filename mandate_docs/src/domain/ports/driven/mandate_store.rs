//! The port the domain uses to discover and read mandate files under a
//! repository root.

use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MandateFile {
    pub file_name: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError(pub String);

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait MandateStore {
    /// File names (with extension) of the `.yaml` files directly in
    /// `<root>/.mandate/mandates/`, sorted. Filtering to `.yaml` is the
    /// adapter's job.
    fn list(&self, root: &Path) -> Result<Vec<String>, StoreError>;

    fn read(&self, root: &Path, file_name: &str) -> Result<MandateFile, StoreError>;
}
