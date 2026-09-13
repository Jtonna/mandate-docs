//! The CLI as a driving adapter: pure argument parsing and pure execution,
//! both testable in-process without spawning the compiled binary.

use crate::adapters::yaml::parse_mandate;
use crate::domain::ports::file_tree::FileTree;
use crate::domain::validation::validate;

const USAGE: &str = "usage: mandate validate <mandate-file> --root <dir>";

/// The parsed command line: which mandate file to validate and against
/// which repository root.
#[derive(Debug)]
pub struct Invocation {
    pub mandate_path: String,
    pub root: String,
}

/// Parses the process arguments (excluding argv[0]) into an [`Invocation`].
pub fn parse_args(args: &[String]) -> Result<Invocation, String> {
    let mut args = args.iter();
    let (Some(command), Some(mandate_path)) = (args.next(), args.next()) else {
        return Err(USAGE.to_string());
    };
    if command != "validate" {
        return Err(format!("unknown command '{command}'; expected 'validate'"));
    }
    match (args.next().map(String::as_str), args.next()) {
        (Some("--root"), Some(root)) => Ok(Invocation {
            mandate_path: mandate_path.clone(),
            root: root.clone(),
        }),
        _ => Err(USAGE.to_string()),
    }
}

/// Parses `mandate_text`, validates it against `tree`, and writes the report
/// to `out` / `err` exactly as the CLI prints it. Returns the process exit
/// code: 0 if the mandate is valid, 1 otherwise (parse failure included).
pub fn execute(
    mandate_path: &str,
    mandate_text: &str,
    tree: &dyn FileTree,
    out: &mut dyn std::io::Write,
    err: &mut dyn std::io::Write,
) -> i32 {
    let mandate = match parse_mandate(mandate_text) {
        Ok(mandate) => mandate,
        Err(parse_err) => {
            let _ = writeln!(err, "failed to parse '{mandate_path}': {parse_err}");
            return 1;
        }
    };

    let report = validate(&mandate, tree);

    for line in report.lines() {
        let _ = writeln!(out, "{line}");
    }

    if report.is_valid() {
        let _ = writeln!(
            out,
            "mandate '{}' is valid: {} rules, {} documents, {} source files",
            mandate.name,
            mandate.rules.len(),
            mandate.governs.len(),
            mandate.code.len()
        );
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::file_tree::FileTree;
    use std::collections::HashSet;

    /// A minimal in-file fake for [`FileTree`], kept private to this test
    /// module so a domain-facing test never crosses a layer boundary.
    struct FakeTree(HashSet<String>);

    impl FakeTree {
        fn new<I, S>(paths: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: Into<String>,
        {
            Self(paths.into_iter().map(Into::into).collect())
        }
    }

    impl FileTree for FakeTree {
        fn exists(&self, repo_relative_path: &str) -> bool {
            self.0.contains(repo_relative_path)
        }
    }

    const MINIMAL_MANDATE: &str = r#"
name: Minimal
rules:
  - id: has-owner
    type: script
    run: ./check.sh
governs:
  - doc: docs/a.md
    rules:
      - has-owner
code:
  - path: src/a.ts
    docs:
      - docs/a.md
"#;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_args_with_no_args_is_usage_error() {
        let result = parse_args(&args(&[]));

        let err = result.expect_err("expected an error");
        assert!(err.contains("usage:"), "got: {err}");
    }

    #[test]
    fn parse_args_with_wrong_command_is_unknown_command_error() {
        let result = parse_args(&args(&["bogus", "mandate.yaml"]));

        let err = result.expect_err("expected an error");
        assert!(err.contains("unknown command"), "got: {err}");
    }

    #[test]
    fn parse_args_without_root_is_usage_error() {
        let result = parse_args(&args(&["validate", "mandate.yaml"]));

        let err = result.expect_err("expected an error");
        assert!(err.contains("usage:"), "got: {err}");
    }

    #[test]
    fn parse_args_with_validate_and_root_succeeds() {
        let result = parse_args(&args(&["validate", "mandate.yaml", "--root", "."]));

        let invocation = result.expect("expected success");
        assert_eq!(invocation.mandate_path, "mandate.yaml");
        assert_eq!(invocation.root, ".");
    }

    #[test]
    fn execute_with_valid_mandate_and_all_files_present_returns_zero_and_prints_summary() {
        let tree = FakeTree::new(["docs/a.md", "src/a.ts"]);
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute("mandate.yaml", MINIMAL_MANDATE, &tree, &mut out, &mut err);

        assert_eq!(code, 0);
        let out = String::from_utf8(out).expect("utf8 stdout");
        assert!(
            out.contains("is valid: 1 rules, 1 documents, 1 source files"),
            "out: {out}"
        );
        assert!(err.is_empty(), "err: {}", String::from_utf8_lossy(&err));
    }

    #[test]
    fn execute_with_missing_files_returns_one_and_lists_errors() {
        let tree = FakeTree::new(Vec::<String>::new());
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute("mandate.yaml", MINIMAL_MANDATE, &tree, &mut out, &mut err);

        assert_eq!(code, 1);
        let out = String::from_utf8(out).expect("utf8 stdout");
        assert!(out.contains("governed document not found:"), "out: {out}");
        assert!(out.contains("missing source file:"), "out: {out}");
    }

    #[test]
    fn execute_with_unparseable_mandate_returns_one_and_prints_parse_error() {
        let yaml = r#"
name: Bad
rules:
  - id: has-owner
    type: script
    run: ./check.sh
governs: []
code: []
bogus_top_level_field: true
"#;
        let tree = FakeTree::new(Vec::<String>::new());
        let mut out = Vec::new();
        let mut err = Vec::new();

        let code = execute("mandate.yaml", yaml, &tree, &mut out, &mut err);

        assert_eq!(code, 1);
        let err = String::from_utf8(err).expect("utf8 stderr");
        assert!(err.contains("failed to parse"), "err: {err}");
        assert!(out.is_empty(), "out: {}", String::from_utf8_lossy(&out));
    }
}
