//! Architecture enforcement from docs/architecture/PORTS_AND_ADAPTERS_GUIDE.md
//! section 7: fail the build if domain/ imports from adapters/ or vendor.
//! This test reads the crate's own source, never docs/ or the binary.

use std::fs;
use std::path::{Path, PathBuf};

#[test]
/// Domain must not import from adapters or vendor packages.
fn domain_isolation() {
    let violations = check_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/domain"),
        |line| {
            line.contains("crate::adapters")
                || line.contains("mandate::adapters")
                || (line.contains("serde") && !line.starts_with("// "))
                || (line.contains("yaml_serde") && !line.starts_with("// "))
        },
    );
    assert!(
        violations.is_empty(),
        "domain/ imports forbidden:\n{}",
        violations.join("\n")
    );
}

#[test]
/// Adapters must not import domain usecases.
fn adapters_no_usecases() {
    let violations = check_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/adapters"),
        |line| line.contains("domain::usecases"),
    );
    assert!(
        violations.is_empty(),
        "adapters/ imports usecases:\n{}",
        violations.join("\n")
    );
}

#[test]
/// Only src/main.rs and defining files mention concrete adapters.
/// Domain and usecase tests must use fakes, never construct real adapters.
fn adapter_construction_localized() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let adapters = [
        ("FsFileTreeSource", "fs_file_tree_source.rs"),
        ("FsMandateStore", "fs_mandate_store.rs"),
        ("YamlMandateParser", "yaml_mandate_parser.rs"),
    ];
    let mut rs_files = Vec::new();
    collect_rs_files(&src_dir, &mut rs_files);

    let mut bad = Vec::new();
    for (ctor, def_file) in adapters {
        for file_path in &rs_files {
            let s = file_path.to_string_lossy();
            if s.ends_with(def_file) || s.ends_with("main.rs") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(file_path) {
                for (line_no, line) in content.lines().enumerate() {
                    if line.contains(ctor) {
                        let rel = file_path.strip_prefix(&src_dir).unwrap_or(file_path);
                        bad.push(format!("{}:{}: {}", rel.display(), line_no + 1, ctor));
                    }
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "adapters mentioned outside main.rs:\n{}",
        bad.join("\n")
    );
}

fn check_files<F>(dir: &Path, f: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    let mut out = Vec::new();
    walk_check(dir, dir, &f, &mut out);
    out
}

fn walk_check<F>(root: &Path, curr: &Path, f: &F, out: &mut Vec<String>)
where
    F: Fn(&str) -> bool,
{
    if let Ok(entries) = fs::read_dir(curr) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_check(root, &path, f, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (i, line) in content.lines().enumerate() {
                        let trimmed = line.split("//").next().unwrap_or("").trim();
                        if f(trimmed) {
                            let rel = path.strip_prefix(root).unwrap_or(&path);
                            out.push(format!("{}:{}: {}", rel.display(), i + 1, line));
                        }
                    }
                }
            }
        }
    }
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}
