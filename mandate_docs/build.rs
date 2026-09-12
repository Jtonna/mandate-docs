//! Generates the `#[path]` module declarations for the validation fixture
//! test cases, so a case is registered simply by creating its directory
//! under `tests/fixtures/validation/` with a `test.rs` in it. No file
//! outside that directory needs to be touched.
//!
//! Also emits `DISCOVERED_CASES`, the sorted list of case directory names
//! found. Cargo's own directory-change detection can miss an empty new
//! directory or a deleted one, so `tests/validation_fixtures.rs` checks
//! this list against the directories actually on disk at test time and
//! fails loudly if the generated file has gone stale.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixtures_dir = PathBuf::from(manifest_dir).join("tests/fixtures/validation");

    println!("cargo:rerun-if-changed=tests/fixtures/validation");

    let mut entries: Vec<PathBuf> = fs::read_dir(&fixtures_dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", fixtures_dir.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    entries.sort();

    let mut mod_decls = String::new();
    let mut names: Vec<String> = Vec::new();
    let mut seen_mod_names: HashSet<String> = HashSet::new();
    for dir in entries {
        println!("cargo:rerun-if-changed={}", dir.display());

        let name = dir
            .file_name()
            .expect("case directory has a name")
            .to_string_lossy()
            .into_owned();

        let test_rs = dir.join("test.rs");
        if !test_rs.is_file() {
            panic!(
                "fixture case directory '{name}' has no test.rs (path: {})",
                test_rs.display()
            );
        }

        let mod_name = name.to_lowercase();
        if !is_valid_identifier(&mod_name) {
            panic!(
                "fixture case directory name '{name}' is not a valid Rust identifier when lowercased ('{mod_name}')"
            );
        }
        if !seen_mod_names.insert(mod_name.clone()) {
            let previous = names
                .iter()
                .find(|other| other.to_lowercase() == mod_name)
                .expect("a colliding mod name was inserted by some earlier directory");
            panic!(
                "fixture case directories '{previous}' and '{name}' collide: both lowercase to module name '{mod_name}'"
            );
        }

        let path = test_rs.to_string_lossy().replace('\\', "/");
        mod_decls.push_str(&format!("#[path = \"{path}\"] mod {mod_name};\n"));
        names.push(name);
    }

    let names_list = names
        .iter()
        .map(|n| format!("{n:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let generated =
        format!("{mod_decls}\npub const DISCOVERED_CASES: &[&str] = &[{names_list}];\n");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    let dest = PathBuf::from(out_dir).join("validation_cases.rs");
    fs::write(&dest, generated).unwrap_or_else(|e| panic!("write {}: {e}", dest.display()));
}

fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
