//! [`FileTreeSnapshot`]: a domain value holding an exact repository-relative
//! file tree at a point in time. Paths use forward slashes and are never
//! normalised; a snapshot knows exactly what it was told.

use std::collections::BTreeMap;

use super::mandate::Mandate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

/// An exact repository-relative file tree at a point in time.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileTreeSnapshot {
    entries: BTreeMap<String, EntryKind>,
}

impl FileTreeSnapshot {
    /// A snapshot with nothing in it.
    pub fn empty() -> Self {
        Self::default()
    }

    /// A snapshot holding every document `mandate` governs and every source
    /// file it links, each as a [`EntryKind::File`], plus every ancestor
    /// directory of each as a [`EntryKind::Directory`].
    pub fn with_every_file_in(mandate: &Mandate) -> Self {
        let mut snapshot = Self::empty();
        for governed in &mandate.governs {
            snapshot.insert_file_with_ancestors(&governed.doc);
        }
        for code in &mandate.code {
            snapshot.insert_file_with_ancestors(&code.path);
        }
        snapshot
    }

    fn insert_file_with_ancestors(&mut self, path: &str) {
        let parts: Vec<&str> = path.split('/').collect();
        let mut ancestor = String::new();
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            if !ancestor.is_empty() {
                ancestor.push('/');
            }
            ancestor.push_str(part);
            self.entries
                .entry(ancestor.clone())
                .or_insert(EntryKind::Directory);
        }
        self.entries.insert(path.to_string(), EntryKind::File);
    }

    /// Records `path` with `kind`, overwriting any prior entry at `path`.
    pub fn insert(&mut self, path: impl Into<String>, kind: EntryKind) {
        self.entries.insert(path.into(), kind);
    }

    /// Removes `path`. Returns `true` if it was present.
    pub fn remove(&mut self, path: &str) -> bool {
        self.entries.remove(path).is_some()
    }

    /// Moves `from` to `to`, keeping its kind. Returns `false` (and does
    /// nothing) if `from` is absent.
    pub fn rename(&mut self, from: &str, to: impl Into<String>) -> bool {
        match self.entries.remove(from) {
            Some(kind) => {
                self.entries.insert(to.into(), kind);
                true
            }
            None => false,
        }
    }

    /// Whether `path` is present at all, file or directory.
    pub fn exists(&self, path: &str) -> bool {
        self.entries.contains_key(path)
    }

    /// Whether `path` is present and recorded as a file.
    pub fn is_file(&self, path: &str) -> bool {
        self.entries.get(path) == Some(&EntryKind::File)
    }

    /// Whether `path` is present and recorded as a directory.
    pub fn is_dir(&self, path: &str) -> bool {
        self.entries.get(path) == Some(&EntryKind::Directory)
    }

    /// The number of entries recorded, files and directories together.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no entries are recorded at all.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every recorded path, in the snapshot's own order.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::mandate::{CodeLink, GovernedDoc, Rule, RuleKind};

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
                doc: "docs/sop/a.md".to_string(),
                rules: vec!["has-owner".to_string()],
            }],
            code: vec![CodeLink {
                path: "src/domain/a.ts".to_string(),
                docs: vec!["docs/sop/a.md".to_string()],
            }],
        }
    }

    #[test]
    fn insert_makes_a_path_exist_with_its_kind() {
        let mut snapshot = FileTreeSnapshot::empty();
        snapshot.insert("docs/a.md", EntryKind::File);
        snapshot.insert("docs", EntryKind::Directory);

        assert!(snapshot.exists("docs/a.md"));
        assert!(snapshot.is_file("docs/a.md"));
        assert!(!snapshot.is_dir("docs/a.md"));
        assert!(snapshot.is_dir("docs"));
        assert!(!snapshot.is_file("docs"));
        assert!(!snapshot.exists("docs/missing.md"));
    }

    #[test]
    fn remove_returns_false_when_absent() {
        let mut snapshot = FileTreeSnapshot::empty();
        assert!(!snapshot.remove("docs/a.md"));

        snapshot.insert("docs/a.md", EntryKind::File);
        assert!(snapshot.remove("docs/a.md"));
        assert!(!snapshot.exists("docs/a.md"));
    }

    #[test]
    fn rename_keeps_the_kind_and_reports_absence() {
        let mut snapshot = FileTreeSnapshot::empty();
        assert!(!snapshot.rename("docs/a.md", "docs/b.md"));

        snapshot.insert("docs/a.md", EntryKind::File);
        assert!(snapshot.rename("docs/a.md", "docs/b.md"));
        assert!(!snapshot.exists("docs/a.md"));
        assert!(snapshot.is_file("docs/b.md"));
    }

    #[test]
    fn with_every_file_in_adds_ancestor_directories_and_linked_files() {
        let snapshot = FileTreeSnapshot::with_every_file_in(&mandate());

        assert!(snapshot.is_file("docs/sop/a.md"));
        assert!(snapshot.is_file("src/domain/a.ts"));
        assert!(snapshot.is_dir("docs"));
        assert!(snapshot.is_dir("docs/sop"));
        assert!(snapshot.is_dir("src"));
        assert!(snapshot.is_dir("src/domain"));
        assert_eq!(snapshot.len(), 6);
    }

    #[test]
    fn empty_snapshot_has_no_entries() {
        let snapshot = FileTreeSnapshot::empty();
        assert_eq!(snapshot.len(), 0);
        assert!(snapshot.is_empty());
        assert_eq!(snapshot.paths().count(), 0);
    }
}
