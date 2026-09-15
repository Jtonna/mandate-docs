//! [`RunMandates`]: the use case that discovers a repository's `.mandate`
//! folder by walking up from a starting directory, then parses and
//! validates every selected mandate against one shared snapshot of the
//! repository.

use std::fmt;
use std::path::{Path, PathBuf};

use crate::domain::file_tree::FileTreeSnapshot;
use crate::domain::ports::driven::file_tree_source::{FileTreeSource, SourceError};
use crate::domain::ports::driven::mandate_parser::MandateParser;
use crate::domain::ports::driven::mandate_store::{MandateStore, StoreError};
use crate::domain::run_report::{
    MandateOutcome, MandateResult, RunLocation, RunReportMandatesValidation, RunWarning,
};
use crate::domain::validation::validate;

/// Borrows its three ports for `'ports` and never owns them.
pub struct RunMandates<'ports> {
    source: &'ports dyn FileTreeSource,
    store: &'ports dyn MandateStore,
    parser: &'ports dyn MandateParser,
}

impl<'ports> RunMandates<'ports> {
    pub fn new(
        source: &'ports dyn FileTreeSource,
        store: &'ports dyn MandateStore,
        parser: &'ports dyn MandateParser,
    ) -> Self {
        Self {
            source,
            store,
            parser,
        }
    }

    /// Discovers the repository root by walking up from `start_dir` looking
    /// for a top-level `.mandate` directory entry, then parses and
    /// validates every mandate in `selected` (or every mandate the store
    /// lists, if `selected` is empty) against one shared snapshot of that
    /// root.
    ///
    /// Never finding a `.mandate` folder is not a failure: it means there
    /// is nothing to check, the same as an empty mandates folder. The
    /// report comes back `Ok` with a `NotFound` location, zero mandates
    /// checked, and no store or snapshot calls made.
    pub fn execute(
        &self,
        start_dir: &Path,
        selected: &[String],
    ) -> Result<RunReportMandatesValidation, RunError> {
        let Some((root, snapshot)) = self.discover_root(start_dir)? else {
            return Ok(RunReportMandatesValidation {
                location: RunLocation::NotFound {
                    searched_from: path_str(start_dir),
                },
                warnings: Vec::new(),
                mandates: Vec::new(),
            });
        };
        let root_str = path_str(&root);

        let available = self.store.list(&root)?;
        let names_to_run = self.select(&available, selected)?;

        let mut warnings = Vec::new();
        if available.is_empty() {
            warnings.push(no_mandates_warning(&root_str));
        }

        let mut mandates = Vec::new();
        for name in names_to_run {
            mandates.push(self.check_one(&root, &snapshot, name)?);
        }

        Ok(RunReportMandatesValidation {
            location: RunLocation::Found {
                root: root_str,
                snapshot_entries: snapshot.len(),
            },
            warnings,
            mandates,
        })
    }

    /// Picks which mandate names to run: every name in `available` when
    /// `selected` is empty, otherwise just `selected`, after checking every
    /// selected name actually appears in `available`.
    fn select(&self, available: &[String], selected: &[String]) -> Result<Vec<String>, RunError> {
        if selected.is_empty() {
            return Ok(available.to_vec());
        }

        for name in selected {
            if !available.contains(name) {
                return Err(RunError::UnknownMandate {
                    name: name.clone(),
                    available: available.to_vec(),
                });
            }
        }

        Ok(available
            .iter()
            .filter(|name| selected.contains(name))
            .cloned()
            .collect())
    }

    /// Reads, parses, and validates one mandate by name against `snapshot`.
    fn check_one(
        &self,
        root: &Path,
        snapshot: &FileTreeSnapshot,
        name: String,
    ) -> Result<MandateOutcome, RunError> {
        let file = self.store.read(root, &name)?;

        let result = match self.parser.parse(&file.text) {
            Err(message) => MandateResult::ParseFailed(message),
            Ok(mandate) => MandateResult::Validated(validate(&mandate, snapshot)),
        };

        Ok(MandateOutcome {
            file_name: name,
            result,
        })
    }

    /// Walks up from `start_dir` looking for a `.mandate` directory entry,
    /// checking each ancestor with a cheap existence check before ever
    /// calling `snapshot`. Snapshotting every ancestor while searching
    /// would mean walking directories the current user does not own on the
    /// way up ("Access is denied" on an unrelated system temp folder,
    /// hit this way in a real run), so only the level that turns out to be
    /// the root is ever snapshotted. A directory that cannot be read simply
    /// answers `false`, so an unreadable ancestor never stops the search.
    /// Returns `Ok(None)` if the filesystem root is reached with no
    /// `.mandate` folder found.
    fn discover_root(
        &self,
        start_dir: &Path,
    ) -> Result<Option<(PathBuf, FileTreeSnapshot)>, RunError> {
        let mut current = start_dir.to_path_buf();
        loop {
            let has_mandate = self.source.has_directory(&current, ".mandate")?;

            if has_mandate {
                let snapshot = self.source.snapshot(&current)?;
                return Ok(Some((current, snapshot)));
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return Ok(None),
            }
        }
    }
}

/// Renders `path` with forward slashes regardless of platform, so a report
/// built on Windows reads the same as one built anywhere else.
fn path_str(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// The warning for a discovered root whose mandates folder is empty.
/// `root_str` already ends in "/" when the root is a drive root (e.g. "/"
/// or "C:/"); trim it first so the joined path never double-slashes.
fn no_mandates_warning(root_str: &str) -> RunWarning {
    let root_trimmed = root_str.trim_end_matches('/');
    RunWarning::NoMandatesFound {
        mandates_dir: format!("{root_trimmed}/.mandate/mandates"),
    }
}

// This sits below the type on purpose: read what the run does before what
// can go wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// A name in `selected` is not among the mandate files the store lists
    /// for the discovered root. `available` is that full list.
    UnknownMandate {
        name: String,
        available: Vec<String>,
    },
    Source(String),
    Store(String),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::UnknownMandate { name, available } => write!(
                f,
                "unknown mandate '{name}'; available mandates are: {}",
                available.join(", ")
            ),
            RunError::Source(message) => write!(f, "{message}"),
            RunError::Store(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for RunError {}

impl From<SourceError> for RunError {
    fn from(e: SourceError) -> Self {
        RunError::Source(e.to_string())
    }
}

impl From<StoreError> for RunError {
    fn from(e: StoreError) -> Self {
        RunError::Store(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};
    use std::path::Path;

    use crate::application::run_mandates::{RunError, RunMandates};
    use crate::domain::file_tree::{EntryKind, FileTreeSnapshot};
    use crate::domain::mandate::{CodeLink, GovernedDoc, Mandate, Rule, RuleKind};
    use crate::domain::ports::driven::file_tree_source::{FileTreeSource, SourceError};
    use crate::domain::ports::driven::mandate_parser::MandateParser;
    use crate::domain::ports::driven::mandate_store::{MandateFile, MandateStore, StoreError};
    use crate::domain::run_report::{MandateResult, RunLocation, RunWarning};

    fn path_str(p: &Path) -> String {
        p.to_string_lossy().replace('\\', "/")
    }

    struct FakeSource {
        /// Directories that answer `true` for `has_directory(dir, ".mandate")`.
        targets: HashSet<String>,
        /// The snapshot to hand back when `snapshot` is called on a
        /// directory in `targets`.
        snapshots: HashMap<String, FileTreeSnapshot>,
        /// Directories where `snapshot` returns an error, simulating a
        /// permission-denied ancestor. `has_directory` still answers
        /// normally for these (`Ok(false)`, never an error) so a caller
        /// that does existence checks first is never affected by them.
        unreadable: HashSet<String>,
        has_directory_calls: RefCell<Vec<String>>,
        snapshot_calls: RefCell<Vec<String>>,
    }

    impl FakeSource {
        fn new() -> Self {
            Self {
                targets: HashSet::new(),
                snapshots: HashMap::new(),
                unreadable: HashSet::new(),
                has_directory_calls: RefCell::new(Vec::new()),
                snapshot_calls: RefCell::new(Vec::new()),
            }
        }

        /// `dir` answers `true` for `has_directory(dir, ".mandate")`.
        fn with_target(mut self, dir: &str) -> Self {
            self.targets.insert(dir.to_string());
            self
        }

        /// The snapshot `snapshot(dir)` returns.
        fn with_snapshot(mut self, dir: &str, snapshot: FileTreeSnapshot) -> Self {
            self.snapshots.insert(dir.to_string(), snapshot);
            self
        }

        /// `snapshot(dir)` errors, as if `dir` could not be read.
        fn unreadable(mut self, dir: &str) -> Self {
            self.unreadable.insert(dir.to_string());
            self
        }

        fn has_directory_call_count(&self) -> usize {
            self.has_directory_calls.borrow().len()
        }

        fn snapshot_call_count(&self) -> usize {
            self.snapshot_calls.borrow().len()
        }
    }

    impl FileTreeSource for FakeSource {
        fn snapshot(&self, root: &Path) -> Result<FileTreeSnapshot, SourceError> {
            let key = path_str(root);
            self.snapshot_calls.borrow_mut().push(key.clone());
            if self.unreadable.contains(&key) {
                return Err(SourceError(format!("{key}: Access is denied")));
            }
            Ok(self.snapshots.get(&key).cloned().unwrap_or_default())
        }

        fn has_directory(&self, dir: &Path, name: &str) -> Result<bool, SourceError> {
            let key = path_str(dir);
            self.has_directory_calls.borrow_mut().push(key.clone());
            Ok(name == ".mandate" && self.targets.contains(&key))
        }
    }

    struct FakeStore {
        files: HashMap<String, String>,
        reads: RefCell<Vec<String>>,
    }

    impl FakeStore {
        fn new() -> Self {
            Self {
                files: HashMap::new(),
                reads: RefCell::new(Vec::new()),
            }
        }

        fn with_mandate(mut self, name: &str, text: &str) -> Self {
            self.files.insert(name.to_string(), text.to_string());
            self
        }
    }

    impl MandateStore for FakeStore {
        fn list(&self, _root: &Path) -> Result<Vec<String>, StoreError> {
            let mut names: Vec<String> = self.files.keys().cloned().collect();
            names.sort();
            Ok(names)
        }

        fn read(&self, _root: &Path, file_name: &str) -> Result<MandateFile, StoreError> {
            self.reads.borrow_mut().push(file_name.to_string());
            self.files
                .get(file_name)
                .map(|text| MandateFile {
                    file_name: file_name.to_string(),
                    text: text.clone(),
                })
                .ok_or_else(|| StoreError(format!("unknown mandate file '{file_name}'")))
        }
    }

    struct FakeParser {
        mandates: HashMap<String, Result<Mandate, String>>,
    }

    impl FakeParser {
        fn new() -> Self {
            Self {
                mandates: HashMap::new(),
            }
        }

        fn ok(mut self, text: &str, mandate: Mandate) -> Self {
            self.mandates.insert(text.to_string(), Ok(mandate));
            self
        }

        fn err(mut self, text: &str, message: &str) -> Self {
            self.mandates
                .insert(text.to_string(), Err(message.to_string()));
            self
        }
    }

    impl MandateParser for FakeParser {
        fn parse(&self, text: &str) -> Result<Mandate, String> {
            self.mandates
                .get(text)
                .cloned()
                .unwrap_or_else(|| Err(format!("no fake mapping for text: {text}")))
        }
    }

    fn snapshot_with_mandate_folder() -> FileTreeSnapshot {
        let mut snapshot = FileTreeSnapshot::empty();
        snapshot.insert(".mandate", EntryKind::Directory);
        snapshot.insert(".mandate/mandates", EntryKind::Directory);
        snapshot
    }

    fn mandate(name: &str) -> Mandate {
        Mandate {
            name: name.to_string(),
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
    fn found_at_the_start_dir() {
        let source = FakeSource::new()
            .with_target("/p")
            .with_snapshot("/p", snapshot_with_mandate_folder());
        let store = FakeStore::new();
        let parser = FakeParser::new();
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/p"), &[]).unwrap();

        assert_eq!(
            report.location,
            RunLocation::Found {
                root: "/p".to_string(),
                snapshot_entries: 2
            }
        );
        assert_eq!(source.has_directory_call_count(), 1);
        assert_eq!(source.snapshot_call_count(), 1);
    }

    #[test]
    fn found_two_levels_up() {
        let source = FakeSource::new()
            .with_target("/p")
            .with_snapshot("/p", snapshot_with_mandate_folder());
        let store = FakeStore::new();
        let parser = FakeParser::new();
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/p/a/b"), &[]).unwrap();

        assert_eq!(
            report.location,
            RunLocation::Found {
                root: "/p".to_string(),
                snapshot_entries: 2
            }
        );
        // One existence check per level walked (/p/a/b, /p/a, /p), but the
        // full tree is only ever snapshotted once, at the level that
        // actually turned out to be the root.
        assert_eq!(source.has_directory_call_count(), 3);
        assert_eq!(source.snapshot_call_count(), 1);
    }

    #[test]
    fn not_found_reports_not_found_and_never_snapshots() {
        let source = FakeSource::new();
        let store = FakeStore::new();
        let parser = FakeParser::new();
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/p/a/b"), &[]).unwrap();

        assert_eq!(
            report.location,
            RunLocation::NotFound {
                searched_from: "/p/a/b".to_string()
            }
        );
        assert!(report.mandates.is_empty());
        assert!(report.is_valid());
        assert_eq!(source.snapshot_call_count(), 0);
    }

    #[test]
    fn a_child_directorys_mandate_folder_is_ignored() {
        // "/p" has no .mandate itself; only its child "/p/sub" does. Walking
        // up from "/p" must never look into children, so this still
        // reports not found.
        let source = FakeSource::new().with_target("/p/sub");
        let store = FakeStore::new();
        let parser = FakeParser::new();
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/p"), &[]).unwrap();

        assert_eq!(
            report.location,
            RunLocation::NotFound {
                searched_from: "/p".to_string()
            }
        );
    }

    #[test]
    fn an_unreadable_ancestor_does_not_error_and_the_walk_continues() {
        // "/p/a" cannot be snapshotted (simulating a permission error), but
        // discovery only ever asks it a cheap existence question, which
        // answers `false` rather than erroring, so the walk continues past
        // it up to "/p" and succeeds.
        let source = FakeSource::new()
            .unreadable("/p/a")
            .with_target("/p")
            .with_snapshot("/p", snapshot_with_mandate_folder());
        let store = FakeStore::new();
        let parser = FakeParser::new();
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/p/a/b"), &[]).unwrap();

        assert_eq!(
            report.location,
            RunLocation::Found {
                root: "/p".to_string(),
                snapshot_entries: 2
            }
        );
        assert_eq!(source.snapshot_call_count(), 1);
    }

    #[test]
    fn all_mandates_run_in_list_order() {
        let source = FakeSource::new()
            .with_target("/p")
            .with_snapshot("/p", snapshot_with_mandate_folder());
        let store = FakeStore::new()
            .with_mandate("a.yaml", "text-a")
            .with_mandate("b.yaml", "text-b");
        let parser = FakeParser::new()
            .ok("text-a", mandate("A"))
            .ok("text-b", mandate("B"));
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/p"), &[]).unwrap();

        let names: Vec<&str> = report
            .mandates
            .iter()
            .map(|o| o.file_name.as_str())
            .collect();
        assert_eq!(names, vec!["a.yaml", "b.yaml"]);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn named_selection_runs_only_those() {
        let source = FakeSource::new()
            .with_target("/p")
            .with_snapshot("/p", snapshot_with_mandate_folder());
        let store = FakeStore::new()
            .with_mandate("a.yaml", "text-a")
            .with_mandate("b.yaml", "text-b");
        let parser = FakeParser::new()
            .ok("text-a", mandate("A"))
            .ok("text-b", mandate("B"));
        let run = RunMandates::new(&source, &store, &parser);

        let report = run
            .execute(Path::new("/p"), &["b.yaml".to_string()])
            .unwrap();

        let names: Vec<&str> = report
            .mandates
            .iter()
            .map(|o| o.file_name.as_str())
            .collect();
        assert_eq!(names, vec!["b.yaml"]);
    }

    #[test]
    fn unknown_name_errors_and_reads_nothing() {
        let source = FakeSource::new()
            .with_target("/p")
            .with_snapshot("/p", snapshot_with_mandate_folder());
        let store = FakeStore::new()
            .with_mandate("a.yaml", "text-a")
            .with_mandate("b.yaml", "text-b");
        let parser = FakeParser::new();
        let run = RunMandates::new(&source, &store, &parser);

        let err = run
            .execute(Path::new("/p"), &["missing.yaml".to_string()])
            .unwrap_err();

        match err {
            RunError::UnknownMandate { name, available } => {
                assert_eq!(name, "missing.yaml");
                assert_eq!(available, vec!["a.yaml".to_string(), "b.yaml".to_string()]);
            }
            other => panic!("expected UnknownMandate, got {other:?}"),
        }
        assert!(store.reads.borrow().is_empty());
    }

    #[test]
    fn parse_failure_in_one_mandate_still_validates_the_other() {
        let source = FakeSource::new()
            .with_target("/p")
            .with_snapshot("/p", snapshot_with_mandate_folder());
        let store = FakeStore::new()
            .with_mandate("a.yaml", "bad-text")
            .with_mandate("b.yaml", "text-b");
        let parser = FakeParser::new()
            .err("bad-text", "unknown field 'bogus'")
            .ok("text-b", mandate("B"));
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/p"), &[]).unwrap();

        let a = &report.mandates[0];
        assert_eq!(a.file_name, "a.yaml");
        assert_eq!(
            a.result,
            MandateResult::ParseFailed("unknown field 'bogus'".to_string())
        );

        let b = &report.mandates[1];
        assert_eq!(b.file_name, "b.yaml");
        assert!(matches!(b.result, MandateResult::Validated(_)));
    }

    #[test]
    fn empty_list_warns_with_mandates_dir_and_zero_mandates() {
        let source = FakeSource::new()
            .with_target("/p")
            .with_snapshot("/p", snapshot_with_mandate_folder());
        let store = FakeStore::new();
        let parser = FakeParser::new();
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/p"), &[]).unwrap();

        assert_eq!(
            report.warnings,
            vec![RunWarning::NoMandatesFound {
                mandates_dir: "/p/.mandate/mandates".to_string()
            }]
        );
        assert!(report.mandates.is_empty());
    }

    #[test]
    fn empty_list_at_a_drive_root_does_not_double_slash_mandates_dir() {
        let source = FakeSource::new()
            .with_target("/")
            .with_snapshot("/", snapshot_with_mandate_folder());
        let store = FakeStore::new();
        let parser = FakeParser::new();
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/"), &[]).unwrap();

        assert_eq!(
            report.warnings,
            vec![RunWarning::NoMandatesFound {
                mandates_dir: "/.mandate/mandates".to_string()
            }]
        );
    }

    #[test]
    fn same_snapshot_used_for_every_mandate_and_snapshotted_only_once() {
        let source = FakeSource::new()
            .with_target("/p")
            .with_snapshot("/p", snapshot_with_mandate_folder());
        let store = FakeStore::new()
            .with_mandate("a.yaml", "text-a")
            .with_mandate("b.yaml", "text-b");
        let parser = FakeParser::new()
            .ok("text-a", mandate("A"))
            .ok("text-b", mandate("B"));
        let run = RunMandates::new(&source, &store, &parser);

        let report = run.execute(Path::new("/p/a"), &[]).unwrap();

        assert_eq!(report.mandates.len(), 2);
        // One existence check per level walked (/p/a, then /p), and the
        // tree is snapshotted exactly once, at the found root, never once
        // per mandate even though two mandates were validated.
        assert_eq!(source.has_directory_call_count(), 2);
        assert_eq!(source.snapshot_call_count(), 1);
    }
}
