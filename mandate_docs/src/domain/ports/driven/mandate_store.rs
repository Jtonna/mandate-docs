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

/// The folder a project keeps its mandates in, relative to the project
/// root.
pub const MANDATE_DIR: &str = ".mandate";

/// The folder directly under [`MANDATE_DIR`] that holds mandate files.
pub const MANDATES_SUBDIR: &str = "mandates";

/// The folder a project keeps its mandate files in, relative to the
/// project root. `concat!` cannot build this from `MANDATE_DIR` and
/// `MANDATES_SUBDIR` on stable Rust (it only accepts literals, not
/// consts), so this is a separate literal; the `mandates_dir_matches_its_parts`
/// test below pins it to stay in sync with them.
pub const MANDATES_DIR: &str = ".mandate/mandates";

pub trait MandateStore {
    /// File names (with extension) of the `.yaml` files directly in
    /// `<root>/.mandate/mandates/`, sorted. Filtering to `.yaml` is the
    /// adapter's job.
    fn list(&self, root: &Path) -> Result<Vec<String>, StoreError>;

    /// Reads one mandate file by name. `file_name` is a bare file name, as
    /// returned by `list`: an implementation must reject any value
    /// containing a path separator or `..` with a `StoreError`, rather than
    /// resolving it.
    fn read(&self, root: &Path, file_name: &str) -> Result<MandateFile, StoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mandates_dir_matches_its_parts() {
        assert_eq!(MANDATES_DIR, format!("{MANDATE_DIR}/{MANDATES_SUBDIR}"));
    }
}
