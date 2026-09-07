//! Shared loading code for the fixture-driven validation tests. Not a test
//! binary itself (the classic `tests/common/mod.rs` pattern), just helpers.

use std::fs;
use std::path::Path;

use mandate::adapters::memory_tree::InMemoryFileTree;

/// One `tests/fixtures/validation/<CASE>/` directory, loaded into memory.
pub struct Case {
    pub name: String,
    pub mandate_yaml: String,
    pub trees: Vec<(&'static str, String)>,
    pub expected_pass: bool,
    pub expected_lines: Vec<String>,
}

/// Turns the text of one `tree.<os>.txt` fixture file into an
/// [`InMemoryFileTree`]. Line 1 is the repository root exactly as that OS
/// prints it; every following non-empty line is one directory-listing entry
/// for that OS, root prefix and all.
pub fn load_tree(text: &str) -> InMemoryFileTree {
    let mut lines = text.lines();
    let root = lines.next().unwrap_or("").trim();

    let paths: Vec<String> = lines
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            if line == root {
                // The root itself, relisted as an entry: not a file.
                return None;
            }
            let stripped = line
                .strip_prefix(root)
                .and_then(|rest| rest.strip_prefix(['/', '\\']))
                .unwrap_or_else(|| panic!("tree entry '{line}' is not under root '{root}'"));
            let forward = stripped.replace('\\', "/");
            let mut p = forward.as_str();
            loop {
                if let Some(rest) = p.strip_prefix("./") {
                    p = rest;
                } else if let Some(rest) = p.strip_prefix('/') {
                    p = rest;
                } else {
                    break;
                }
            }
            Some(p.to_string())
        })
        .filter(|p| !p.is_empty())
        .collect();

    InMemoryFileTree::new(paths)
}

/// Loads one fixture case directory: the mandate, its three OS-flavoured
/// trees, and the expected report.
pub fn load_case(dir: &Path) -> Case {
    let name = dir
        .file_name()
        .expect("fixture case has a directory name")
        .to_string_lossy()
        .into_owned();

    let mandate_yaml =
        fs::read_to_string(dir.join("mandate.yaml")).expect("fixture case has mandate.yaml");

    let trees = [
        ("windows", "tree.windows.txt"),
        ("linux", "tree.linux.txt"),
        ("macos", "tree.macos.txt"),
    ]
    .into_iter()
    .map(|(os, file)| {
        let text = fs::read_to_string(dir.join(file))
            .unwrap_or_else(|_| panic!("fixture case {name} is missing {file}"));
        (os, text)
    })
    .collect();

    let expected_text =
        fs::read_to_string(dir.join("expected.txt")).expect("fixture case has expected.txt");
    let mut expected_lines_iter = expected_text.lines();
    let verdict = expected_lines_iter
        .next()
        .unwrap_or_else(|| panic!("fixture case {name} has an empty expected.txt"));
    let expected_pass = match verdict.trim() {
        "PASS" => true,
        "FAIL" => false,
        other => {
            panic!("fixture case {name}: expected.txt line 1 must be PASS or FAIL, got '{other}'")
        }
    };
    let expected_lines: Vec<String> = expected_lines_iter
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();

    Case {
        name,
        mandate_yaml,
        trees,
        expected_pass,
        expected_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandate::domain::ports::file_tree::FileTree;

    #[test]
    fn load_tree_strips_each_os_root_format() {
        let windows = "C:\\work\\repo\nC:\\work\\repo\\docs\nC:\\work\\repo\\docs\\a.md\n";
        let linux = ".\n./docs\n./docs/a.md\n";
        let macos = "/Users/dev/repo\n/Users/dev/repo/docs\n/Users/dev/repo/docs/a.md\n";

        assert!(load_tree(windows).exists("docs/a.md"));
        assert!(load_tree(linux).exists("docs/a.md"));
        assert!(load_tree(macos).exists("docs/a.md"));
    }

    #[test]
    #[should_panic]
    fn load_tree_rejects_entry_outside_root() {
        let text = "/Users/dev/repo\n/Users/dev/repository/x\n";
        load_tree(text);
    }
}
