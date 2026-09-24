//! Shared test helpers for the validation fixtures in this target: an
//! in-memory fake virtual machine to stand in for a repository's file
//! tree, and the assertions that check a validation report against what
//! a case expects. Private to `tests/fixtures/`; each case module
//! reaches it via `use crate::support::*;`.

use std::collections::HashMap;
use std::path::Path;

use mandate::adapters::driven::file_system::{DirEntry, FileSystem, FileSystemError};
use mandate::adapters::driven::file_tree::fs_file_tree_source::FsFileTreeSource;
use mandate::adapters::driven::mandate_parser::yaml_mandate_parser::{
    parse_mandate, YamlMandateParser,
};
use mandate::adapters::driven::mandate_store::fs_mandate_store::FsMandateStore;
use mandate::adapters::driving::cli::validation_lines;
use mandate::domain::model::file_tree::{EntryKind, FileTreeSnapshot};
use mandate::domain::model::mandate::Mandate;
use mandate::domain::model::run_report::RunReportMandatesValidation;
use mandate::domain::model::validation::validate;
use mandate::domain::usecases::run_mandates::{RunError, RunMandates};

/// An in-memory repository-relative file tree. Also stands in as the
/// mandate store: `with_mandate` records both the file's text and its
/// presence in the tree. Implements [`FileSystem`], the seam the real
/// [`FsFileTreeSource`] and [`FsMandateStore`] adapters are built on, so
/// `run` below exercises the real adapters over fake data.
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

    /// The directory this virtual machine answers for as a filesystem.
    /// Default `"/repo"`.
    pub fn at_root(mut self, root: &str) -> Self {
        self.root = root.to_string();
        self
    }

    /// Registers a mandate file: `text` becomes readable under
    /// `file_name`, and `.mandate`, `.mandate/mandates`, and the file
    /// itself are added to the snapshot as a directory, a directory, and
    /// a file respectively.
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

impl FakeVirtualMachine {
    /// The path relative to this machine's root, forward-slash normalised,
    /// with the root itself mapping to `""`. `None` if `path` is not the
    /// root or under it.
    fn relative_of(&self, path: &Path) -> Option<String> {
        let requested = path.to_string_lossy().replace('\\', "/");
        if requested == self.root {
            Some(String::new())
        } else {
            requested
                .strip_prefix(&format!("{}/", self.root))
                .map(str::to_string)
        }
    }
}

impl FileSystem for FakeVirtualMachine {
    /// Errors unless `dir` is the root or a directory recorded in the
    /// snapshot. Otherwise answers every entry whose path's parent is
    /// exactly `dir`, i.e. its direct children only.
    fn list_dir(&self, dir: &Path) -> Result<Vec<DirEntry>, FileSystemError> {
        let relative_dir = self
            .relative_of(dir)
            .ok_or_else(|| FileSystemError(format!("{}: not under this root", dir.display())))?;
        if !relative_dir.is_empty() && !self.snapshot.is_dir(&relative_dir) {
            return Err(FileSystemError(format!(
                "{}: no such directory",
                dir.display()
            )));
        }

        let mut entries = Vec::new();
        for path in self.snapshot.paths() {
            let (parent, name) = match path.rsplit_once('/') {
                Some((parent, name)) => (parent, name),
                None => ("", path),
            };
            if parent == relative_dir {
                let kind = if self.snapshot.is_dir(path) {
                    EntryKind::Directory
                } else {
                    EntryKind::File
                };
                entries.push(DirEntry {
                    name: name.to_string(),
                    kind,
                });
            }
        }
        Ok(entries)
    }

    /// Serves mandate text recorded by `with_mandate`, keyed by the bare
    /// file name under `.mandate/mandates/`.
    fn read_to_string(&self, path: &Path) -> Result<String, FileSystemError> {
        let unknown = || FileSystemError(format!("{}: unknown path", path.display()));
        let relative = self.relative_of(path).ok_or_else(unknown)?;
        let file_name = relative
            .strip_prefix(".mandate/mandates/")
            .ok_or_else(unknown)?;
        self.mandates.get(file_name).cloned().ok_or_else(unknown)
    }

    fn is_dir(&self, path: &Path) -> bool {
        match self.relative_of(path) {
            Some(relative) if relative.is_empty() => true,
            Some(relative) => self.snapshot.is_dir(&relative),
            None => false,
        }
    }

    fn exists(&self, path: &Path) -> bool {
        match self.relative_of(path) {
            Some(relative) if relative.is_empty() => true,
            Some(relative) => self.snapshot.exists(&relative),
            None => false,
        }
    }
}

/// Runs [`RunMandates`] against `vm`, selecting `selected` (empty selects
/// every mandate the store lists), starting discovery at `start_dir`. Goes
/// through the real [`FsFileTreeSource`] and [`FsMandateStore`] adapters,
/// each built over `vm` as the [`FileSystem`], so the real adapter code
/// runs against fake data.
pub fn run(
    vm: &FakeVirtualMachine,
    start_dir: &str,
    selected: &[&str],
) -> Result<RunReportMandatesValidation, RunError> {
    let parser = YamlMandateParser;
    let selected: Vec<String> = selected.iter().map(|s| s.to_string()).collect();
    let source = FsFileTreeSource::new(vm);
    let store = FsMandateStore::new(vm);
    RunMandates::new(&source, &store, &parser).execute(Path::new(start_dir), &selected)
}

/// Parses `yaml` into a [`Mandate`], panicking with the parse error if it is
/// malformed. Fixture mandate.yaml files are expected to parse; a case that
/// wants to test parse failure would not use this helper.
pub fn parse(yaml: &str) -> Mandate {
    parse_mandate(yaml).unwrap_or_else(|e| panic!("mandate.yaml failed to parse: {e}"))
}

/// Validates `mandate` against `vm` and returns the report lines exactly as
/// the CLI prints them, in validator order (see `cli::validation_lines`).
pub fn check(mandate: &Mandate, vm: &FakeVirtualMachine) -> Vec<String> {
    validation_lines(&validate(mandate, &vm.snapshot()))
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
    fn list_dir_of_the_root_answers_its_direct_children_only() {
        let vm = FakeVirtualMachine::empty().with_mandate("a.yaml", "name: Test\n");

        let mut entries = vm.list_dir(Path::new("/repo")).expect("list_dir");
        entries.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(
            entries,
            vec![DirEntry {
                name: ".mandate".to_string(),
                kind: EntryKind::Directory,
            }]
        );
    }

    #[test]
    fn read_to_string_of_an_unknown_path_errs() {
        let vm = FakeVirtualMachine::empty();

        let err = vm
            .read_to_string(Path::new("/repo/.mandate/mandates/missing.yaml"))
            .unwrap_err();

        assert!(!err.0.is_empty());
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
