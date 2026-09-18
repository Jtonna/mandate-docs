//! Shared test helpers for the validation fixtures in this target: an
//! in-memory fake virtual machine to stand in for a repository's file
//! tree, and the assertions that check a validation report against what
//! a case expects. Private to `tests/fixtures/`; each case module
//! reaches it via `use crate::support::*;`.

use std::collections::HashMap;
use std::path::Path;

use mandate::adapters::driven::mandate_parser::yaml_mandate_parser::{
    parse_mandate, YamlMandateParser,
};
use mandate::domain::model::file_tree::{EntryKind, FileTreeSnapshot};
use mandate::domain::model::mandate::Mandate;
use mandate::domain::model::run_report::RunReportMandatesValidation;
use mandate::domain::model::validation::validate;
use mandate::domain::ports::driven::file_tree_source::{FileTreeSource, SourceError};
use mandate::domain::ports::driven::mandate_store::{MandateFile, MandateStore, StoreError};
use mandate::domain::usecases::run_mandates::{RunError, RunMandates};

/// An in-memory repository-relative file tree. Also stands in as the
/// mandate store: `with_mandate` records both the file's text (for
/// `MandateStore`) and its presence in the tree (for `FileTreeSource`).
pub struct FakeVirtualMachine {
    snapshot: FileTreeSnapshot,
    root: String,
    mandates: HashMap<String, String>,
}

impl FakeVirtualMachine {
    /// A virtual machine with no files at all, answering for "/repo".
    pub fn empty() -> Self {
        Self {
            snapshot: FileTreeSnapshot::empty(),
            root: "/repo".to_string(),
            mandates: HashMap::new(),
        }
    }

    /// A virtual machine that already has every document `mandate` governs
    /// and every source file `mandate` links, i.e. a machine that should
    /// pass validation as far as file presence goes.
    pub fn with_every_file_in(mandate: &Mandate) -> Self {
        Self {
            snapshot: FileTreeSnapshot::with_every_file_in(mandate),
            root: "/repo".to_string(),
            mandates: HashMap::new(),
        }
    }

    /// Adds `path` to the virtual machine as a file.
    #[allow(clippy::should_implement_trait)] // builder verb, not std::ops::Add
    pub fn add(mut self, path: impl Into<String>) -> Self {
        self.snapshot.insert(path, EntryKind::File);
        self
    }

    /// Removes `path` from the virtual machine. Panics if `path` is absent,
    /// to catch a typo in a fixture case rather than silently doing nothing.
    pub fn remove(mut self, path: &str) -> Self {
        if !self.snapshot.remove(path) {
            panic!("FakeVirtualMachine::remove: '{path}' is not present");
        }
        self
    }

    /// Moves `from` to `to`. Panics if `from` is absent.
    pub fn rename(mut self, from: &str, to: impl Into<String>) -> Self {
        if !self.snapshot.rename(from, to) {
            panic!("FakeVirtualMachine::rename: '{from}' is not present");
        }
        self
    }

    /// The directory this virtual machine answers for as a
    /// [`FileTreeSource`]. Default `"/repo"`.
    pub fn at_root(mut self, root: &str) -> Self {
        self.root = root.to_string();
        self
    }

    /// Registers a mandate file: `text` becomes readable through
    /// [`MandateStore`] under `file_name`, and `.mandate`,
    /// `.mandate/mandates`, and the file itself are added to the snapshot
    /// as a directory, a directory, and a file respectively.
    pub fn with_mandate(mut self, file_name: &str, text: &str) -> Self {
        self.mandates
            .insert(file_name.to_string(), text.to_string());
        self.snapshot.insert(".mandate", EntryKind::Directory);
        self.snapshot
            .insert(".mandate/mandates", EntryKind::Directory);
        self.snapshot
            .insert(format!(".mandate/mandates/{file_name}"), EntryKind::File);
        self
    }

    /// Marks the virtual machine's root as having a `.mandate/mandates`
    /// folder with no mandate files in it.
    pub fn empty_mandate_folder(mut self) -> Self {
        self.snapshot.insert(".mandate", EntryKind::Directory);
        self.snapshot
            .insert(".mandate/mandates", EntryKind::Directory);
        self
    }

    /// A snapshot of the virtual machine's current contents.
    pub fn snapshot(&self) -> FileTreeSnapshot {
        self.snapshot.clone()
    }
}

impl FileTreeSource for FakeVirtualMachine {
    fn snapshot(&self, root: &Path) -> Result<FileTreeSnapshot, SourceError> {
        let requested = root.to_string_lossy().replace('\\', "/");
        if requested == self.root {
            Ok(self.snapshot.clone())
        } else {
            Ok(FileTreeSnapshot::empty())
        }
    }

    fn has_directory(&self, dir: &Path, name: &str) -> Result<bool, SourceError> {
        let requested = dir.to_string_lossy().replace('\\', "/");
        Ok(requested == self.root && self.snapshot.is_dir(name))
    }
}

impl MandateStore for FakeVirtualMachine {
    fn list(&self, _root: &Path) -> Result<Vec<String>, StoreError> {
        let mut names: Vec<String> = self.mandates.keys().cloned().collect();
        names.sort();
        Ok(names)
    }

    /// Honours the same bare-name contract as the fs adapter: rejects any
    /// `file_name` containing a path separator or `..`, rather than
    /// resolving it.
    fn read(&self, _root: &Path, file_name: &str) -> Result<MandateFile, StoreError> {
        if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
            return Err(StoreError(format!(
                "invalid mandate file name: {file_name}"
            )));
        }

        self.mandates
            .get(file_name)
            .map(|text| MandateFile {
                file_name: file_name.to_string(),
                text: text.clone(),
            })
            .ok_or_else(|| StoreError(format!("unknown mandate file '{file_name}'")))
    }
}

/// Runs [`RunMandates`] against `vm`, selecting `selected` (empty selects
/// every mandate the store lists), starting discovery at `start_dir`.
pub fn run(
    vm: &FakeVirtualMachine,
    start_dir: &str,
    selected: &[&str],
) -> Result<RunReportMandatesValidation, RunError> {
    let parser = YamlMandateParser;
    let selected: Vec<String> = selected.iter().map(|s| s.to_string()).collect();
    RunMandates::new(vm, vm, &parser).execute(Path::new(start_dir), &selected)
}

/// Parses `yaml` into a [`Mandate`], panicking with the parse error if it is
/// malformed. Fixture mandate.yaml files are expected to parse; a case that
/// wants to test parse failure would not use this helper.
pub fn parse(yaml: &str) -> Mandate {
    parse_mandate(yaml).unwrap_or_else(|e| panic!("mandate.yaml failed to parse: {e}"))
}

/// Validates `mandate` against `vm` and returns the report lines exactly as
/// the CLI prints them, in validator order (see `ValidationReport::lines`).
pub fn check(mandate: &Mandate, vm: &FakeVirtualMachine) -> Vec<String> {
    validate(mandate, &vm.snapshot()).lines()
}

/// Asserts `lines` (as returned by [`check`]) reports no problems at all.
pub fn assert_passes(lines: Vec<String>) {
    assert_eq!(
        lines,
        Vec::<String>::new(),
        "expected no errors or warnings"
    );
}

/// Asserts `lines` reports no errors, and exactly the given warnings. Order
/// must match the validator's order: errors first, then warnings.
pub fn assert_passes_with_warnings(lines: Vec<String>, warnings: &[&str]) {
    let expected: Vec<String> = warnings.iter().map(|w| format!("warning: {w}")).collect();
    assert_eq!(lines, expected);
}

/// Asserts `lines` reports exactly the given errors and nothing else. Order
/// must match the validator's order: errors first, then warnings.
pub fn assert_fails(lines: Vec<String>, errors: &[&str]) {
    let expected: Vec<String> = errors.iter().map(|e| format!("error: {e}")).collect();
    assert_eq!(lines, expected);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandate::domain::model::mandate::{CodeLink, GovernedDoc, Rule, RuleKind};

    fn mandate() -> Mandate {
        Mandate {
            name: "Test".to_string(),
            description: None,
            rules: vec![Rule {
                id: "has-owner".to_string(),
                description: None,
                kind: RuleKind::Script {
                    run: "./check.sh".to_string(),
                },
            }],
            governs: vec![GovernedDoc {
                doc: "docs/a.md".to_string(),
                rules: vec!["has-owner".to_string()],
            }],
            code: vec![CodeLink {
                path: "src/a.ts".to_string(),
                docs: vec!["docs/a.md".to_string()],
            }],
        }
    }

    #[test]
    fn read_rejects_a_name_with_a_separator() {
        let vm = FakeVirtualMachine::empty();

        let err = vm.read(Path::new("/repo"), "../a.yaml").unwrap_err();

        assert_eq!(
            err,
            StoreError("invalid mandate file name: ../a.yaml".to_string())
        );
    }

    #[test]
    fn with_every_file_in_makes_a_mandate_pass() {
        let mandate = mandate();
        let vm = FakeVirtualMachine::with_every_file_in(&mandate);

        let lines = check(&mandate, &vm);

        assert_passes(lines);
    }

    #[test]
    #[should_panic(expected = "is not present")]
    fn remove_of_a_missing_path_panics_naming_the_path() {
        FakeVirtualMachine::empty().remove("docs/does-not-exist.md");
    }

    #[test]
    fn rename_moves_the_path() {
        let vm = FakeVirtualMachine::empty()
            .add("docs/a.md")
            .rename("docs/a.md", "docs/b.md");

        assert!(!vm.snapshot().exists("docs/a.md"));
        assert!(vm.snapshot().exists("docs/b.md"));
    }
}
