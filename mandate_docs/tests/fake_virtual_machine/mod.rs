//! A shared in-memory fake virtual machine for the validation fixture tests.
//!
//! Each fixture case builds one of these to represent "what files exist on
//! disk" and hands it straight to `validate` (it implements `FileTree`
//! itself, no `InMemoryFileTree` involved), so a case can express its
//! specific edit ("this file is missing", "this file got renamed") in code
//! instead of maintaining a parallel OS directory listing.

use std::collections::HashSet;

use mandate::adapters::yaml::parse_mandate;
use mandate::domain::mandate::Mandate;
use mandate::domain::ports::file_tree::FileTree;
use mandate::domain::validation::validate;

/// An in-memory set of repository-relative paths.
pub struct FakeVirtualMachine {
    paths: HashSet<String>,
}

impl FakeVirtualMachine {
    /// A virtual machine with no files at all.
    pub fn empty() -> Self {
        Self {
            paths: HashSet::new(),
        }
    }

    /// A virtual machine that already has every document `mandate` governs
    /// and every source file `mandate` links, i.e. a machine that should
    /// pass validation as far as file presence goes.
    pub fn with_every_file_in(mandate: &Mandate) -> Self {
        let mut vm = Self::empty();
        for governed in &mandate.governs {
            vm = vm.add(&governed.doc);
        }
        for code in &mandate.code {
            vm = vm.add(&code.path);
        }
        vm
    }

    /// Adds `path` to the virtual machine.
    pub fn add(mut self, path: impl Into<String>) -> Self {
        self.paths.insert(path.into());
        self
    }

    /// Removes `path` from the virtual machine. Panics if `path` is absent,
    /// to catch a typo in a fixture case rather than silently doing nothing.
    pub fn remove(mut self, path: &str) -> Self {
        if !self.paths.remove(path) {
            panic!("FakeVirtualMachine::remove: '{path}' is not present");
        }
        self
    }

    /// Moves `from` to `to`. Panics if `from` is absent.
    pub fn rename(self, from: &str, to: impl Into<String>) -> Self {
        let vm = self.remove(from);
        vm.add(to)
    }
}

impl FileTree for FakeVirtualMachine {
    fn exists(&self, repo_relative_path: &str) -> bool {
        self.paths.contains(repo_relative_path)
    }
}

/// Parses `yaml` into a [`Mandate`], panicking with the parse error if it is
/// malformed. Fixture mandate.yaml files are expected to parse; a case that
/// wants to test parse failure would not use this helper.
pub fn parse(yaml: &str) -> Mandate {
    parse_mandate(yaml).unwrap_or_else(|e| panic!("mandate.yaml failed to parse: {e}"))
}

/// Validates `mandate` against `vm` and renders the report exactly as the
/// CLI prints it (`error: <Display>` / `warning: <Display>`), sorted so
/// comparisons are order-independent.
pub fn check(mandate: &Mandate, vm: &FakeVirtualMachine) -> Vec<String> {
    let report = validate(mandate, vm);
    let mut lines: Vec<String> = report
        .errors
        .iter()
        .map(|e| format!("error: {e}"))
        .chain(report.warnings.iter().map(|w| format!("warning: {w}")))
        .collect();
    lines.sort();
    lines
}

/// Asserts `lines` (as returned by [`check`]) reports no problems at all.
pub fn assert_passes(lines: Vec<String>) {
    assert_eq!(
        lines,
        Vec::<String>::new(),
        "expected no errors or warnings"
    );
}

/// Asserts `lines` reports no errors, and exactly the given warnings
/// (order-independent).
pub fn assert_passes_with_warnings(lines: Vec<String>, warnings: &[&str]) {
    let mut expected: Vec<String> = warnings.iter().map(|w| format!("warning: {w}")).collect();
    expected.sort();
    assert_eq!(lines, expected);
}

/// Asserts `lines` reports exactly the given errors (order-independent) and
/// nothing else.
pub fn assert_fails(lines: Vec<String>, errors: &[&str]) {
    let mut expected: Vec<String> = errors.iter().map(|e| format!("error: {e}")).collect();
    expected.sort();
    assert_eq!(lines, expected);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandate::domain::mandate::{CodeLink, GovernedDoc, Rule, RuleKind};

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

        assert!(!vm.exists("docs/a.md"));
        assert!(vm.exists("docs/b.md"));
    }
}
