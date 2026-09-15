# Mandate parser, validator and run command

## Purpose

This is the first real software in the repository: a Rust crate that finds
a project's `.mandate` folder, parses every selected mandate YAML file into
a Rust value, validates each against the mandate format and against one
shared snapshot of the file tree, and reports every problem it finds in one
pass. It does not read or write `mandate.json` and does not execute a rule.
Those remain out of scope, as recorded in `README.md` section 5.

## Shape

The crate follows `docs/architecture/PORTS_AND_ADAPTERS_GUIDE.md`, scaled to
the size of the problem: three driven ports, three adapters, and one
application-layer use case that owns discovery, selection and reporting.

| File | Layer | Role |
|---|---|---|
| `src/domain/mandate.rs` | domain | Plain data types for a mandate: `Mandate`, `Rule`, `RuleKind`, `GovernedDoc`, `CodeLink`. No serde, no I/O. |
| `src/domain/file_tree.rs` | domain | `FileTreeSnapshot`, a value recording every file and directory under a root at one point in time. `EntryKind` distinguishes a file from a directory. |
| `src/domain/validation.rs` | domain | `validate`, `ValidationReport`, `ValidationError`, `ValidationWarning`. Checks a `Mandate` against a `&FileTreeSnapshot`. `ValidationReport::lines` renders the report one line per finding. |
| `src/domain/run_report.rs` | domain | `RunReportMandatesValidation`, the report for one run: root, snapshot entry count, warnings, and one outcome per mandate. `is_valid()` and `lines()`. |
| `src/domain/ports/driven/file_tree_source.rs` | port | `FileTreeSource`, snapshots a root and cheaply checks whether one directory holds another. |
| `src/domain/ports/driven/mandate_store.rs` | port | `MandateStore`, lists and reads mandate files under a root. |
| `src/domain/ports/driven/mandate_parser.rs` | port | `MandateParser`, turns mandate text into a `Mandate`. |
| `src/application/run_mandates.rs` | application | `RunMandates`, the use case: discovers the project root, takes the snapshot, selects mandates, parses and validates each, and fills the report. `RunError` for its failure modes. |
| `src/adapters/driven/fs_file_tree_source.rs` | driven adapter | `FsFileTreeSource`, a `FileTreeSource` that walks the real filesystem. |
| `src/adapters/driven/fs_mandate_store.rs` | driven adapter | `FsMandateStore`, a `MandateStore` that lists and reads `.yaml` files under `<root>/.mandate/mandates/`. |
| `src/adapters/driven/yaml.rs` | driven adapter | `YamlMandateParser`, a `MandateParser`, plus `ParseError`. Turns mandate YAML text into a `Mandate`. |
| `src/adapters/driving/cli.rs` | driving adapter | `parse_args` turns the process argument list into an `Invocation`. `render` writes a report's lines to a writer. Neither does any I/O of its own. `main.rs` maps `report.is_valid()` to the process exit code. |
| `src/main.rs` | composition root | Collects the process arguments, builds the real adapters, runs `RunMandates`, and prints and exits according to the report. |
| `src/lib.rs` | composition root | Declares the `adapters`, `application` and `domain` modules. |

Dependencies point one way: the adapters, the application layer and the
composition root import the domain; the domain imports nothing outside
`std`. `FileTreeSource`, `MandateStore` and `MandateParser` are driven ports
because `RunMandates` calls them to reach the filesystem and the YAML
parser; `src/adapters/driving/cli.rs` is the sole driving adapter, since nothing
else calls into the crate yet.

Tests follow the split in `README.md` section 10, step 3:

| File | Kind | What it covers |
|---|---|---|
| `src/adapters/driven/yaml.rs` | unit, in-file | The five parse-error cases, with inline YAML. |
| `src/domain/validation.rs` | unit, in-file | Every validation error and warning, and their `Display` strings, using a private fake `&FileTreeSnapshot` builder so the domain test module imports nothing from an adapter. |
| `src/application/run_mandates.rs` | unit, in-file | `RunMandates` behaviour with private doubles: discovery, selection, an unknown mandate name, a parse failure alongside a valid mandate, and an empty mandates folder. |
| `tests/fs_adapters.rs` | integration | `FsFileTreeSource` and `FsMandateStore` against real temporary directories. |
| `src/adapters/driving/cli.rs` | unit, in-file | Argument parsing and `render`, using in-memory writers and a hand-built report. No process is launched. |
| `tests/fixtures/support.rs` | unit, in-file | `FakeVirtualMachine` (implements `FileTreeSource` and `MandateStore`, holds paths and mandate texts, built from a mandate with every linked file present, then edited with `add`, `remove`, `rename` and `with_mandate`) plus `parse`, `check` and the `assert_*` helpers. Has its own unit tests. |
| `tests/fixtures/<case>/mod.rs` | integration | One fixture case, one or more tests, inside the `fixtures` target. |

Tests never read `docs/`, and no test runs the built binary. This repository
has no `.mandate/` folder to read.

The `serde`-derived structs (`MandateDoc`, `RuleDoc`, `GovernedDocDoc`,
`CodeLinkDoc`) live only in `src/adapters/driven/yaml.rs` and never leave that
module. `parse_mandate` maps each one into the plain domain types in
`src/domain/mandate.rs` before returning. This follows the guide's rule that
no library type may enter the domain: the domain types carry no `serde`
attributes and would compile unchanged if the YAML library were replaced.

## The ports

`FileTreeSource` has two methods:

```rust
fn snapshot(&self, root: &Path) -> Result<FileTreeSnapshot, SourceError>;
fn has_directory(&self, dir: &Path, name: &str) -> Result<bool, SourceError>;
```

`snapshot` walks the whole tree under `root`. `has_directory` answers
whether `dir` contains a directory entry named `name`, without building a
snapshot, and answers `false` rather than erroring when `dir` itself
cannot be read, so an unreadable ancestor is a reason it cannot be the
root and never a reason to stop looking further up. `FsFileTreeSource` is
the sole adapter, walking the real filesystem from `root`: every file and
directory underneath, `.git` and `target/` included, symlinks recorded as
files and never followed.

`MandateStore` has two methods:

```rust
fn list(&self, root: &Path) -> Result<Vec<String>, StoreError>;
fn read(&self, root: &Path, file_name: &str) -> Result<MandateFile, StoreError>;
```

`list` returns the file names of the `.yaml` files directly in
`<root>/.mandate/mandates/`, sorted; `read` returns one file's text.
`FsMandateStore` is the sole adapter.

`MandateParser` has one method, turning mandate text into a `Mandate` or a
parse error; `YamlMandateParser` in `src/adapters/driven/yaml.rs` is the sole
adapter, and is described in full in "What parsing rejects" below.

`FakeVirtualMachine` in `tests/fixtures/support.rs` implements both
`FileTreeSource` and `MandateStore` in one type, since a test scenario
naturally sets up a repository's files and its mandate texts together.

## The run command

`RunMandates` in `src/application/run_mandates.rs` is the use case behind
`mandate [--root <dir>] [<file>.yaml ...]`.

**Discovery.** With no `--root`, discovery starts at the current directory.
`RunMandates::discover_root` checks each ancestor in turn with
`FileTreeSource::has_directory(dir, ".mandate")`, a cheap existence check
that answers `false`, rather than erroring, when a directory cannot be
read; if the check comes back false, it moves to the parent and repeats.
Reaching the filesystem root without finding one is not an error; the
report prints a warning instead and exits 0. It never looks inside child
directories, so a nested project with its own `.mandate` is ignored. Only
the ancestor that turns out to hold `.mandate` is ever snapshotted; every
ancestor tried above it gets only the existence check, never a full walk of
its tree. This is recorded in the doc comment on `discover_root`; snapshotting
every ancestor while searching meant walking directories the current user
does not own, and a real run failed with "Access is denied" on an unrelated
system temp folder before it could even report `.mandate` was never found.

**Snapshot.** `RunMandates` shares the one snapshot taken at the
discovered root across every mandate validated in the run. `validate`
never takes its own snapshot.

**Selection.** `MandateStore::list` returns the `.yaml` file names directly
in `.mandate/mandates/`. Selection is by full file name, case-sensitive; the
`name:` field inside a mandate is descriptive only and plays no part in
selection. With no names given on the command line, every listed mandate
runs. An empty mandates folder is `RunWarning::NoMandatesFound`, not an
error. A name on the command line that matches no file is
`RunError::UnknownMandate` and stops the run before anything is validated.

**Per-mandate outcome.** Every selected mandate is read, parsed and
validated in turn. A parse failure is recorded in that mandate's slot as
`MandateResult::ParseFailed`, and the run continues with the rest; it does
not stop the run the way an unknown name does.

**Exit codes.** `0` when every selected mandate parsed and validated clean.
`1` if any mandate failed to parse or validated with errors. `1` also for
`RunError::UnknownMandate`, reported before any mandate runs. `0` for an
empty mandates folder or when no `.mandate` folder is found, since a
warning is not a failure.

## The report

`RunReportMandatesValidation` in `src/domain/run_report.rs` is the domain
value the run command produces. Fields: `location` (a `RunLocation` enum
with variants `Found { root, snapshot_entries }` or
`NotFound { searched_from }`), `warnings` (a `Vec<RunWarning>`), and
`mandates` (a `Vec<MandateOutcome>`, one per selected mandate, each an
`Enum` of `ParseFailed(String)` or `Validated(ValidationReport)`). `is_valid()`
is `true` when no outcome is a parse failure and every `ValidationReport` is
valid. `lines()` renders the whole report as one string per line, in the
layout below.

Nothing prints during the run: `RunMandates` only fills the report, and the
CLI's `render` writes it out once the run is finished.

The naming convention is `RunReport<Phase>`, so a later phase (rule
execution) gets its own `RunReport` type rather than growing this one.

Rendered layout when `.mandate` is found:

```
repository: <root>
snapshot: <n> entries

warning: no mandates found in <dir>      (only when it applies)

SOP_Orders.yaml
  error: governed document not found: docs/architecture/order-lifecycle.md
  invalid: 1 errors, 0 warnings

Networking.yaml
  valid: 3 rules, 2 documents, 7 source files

2 mandates checked, 1 invalid
```

When no `.mandate` folder is found from the starting directory upward:

```
warning: no .mandate folder found from <dir> up to the filesystem root

0 mandates checked, 0 invalid
```

Each mandate's block starts with its file name, then its `ValidationReport`
lines indented two spaces (or a `failed to parse: <message>` line for a
parse failure), then a summary line: `valid: ...` or
`invalid: <n> errors, <n> warnings`. The final line always reports the
total mandates checked and how many were invalid.

## What parsing rejects

`parse_mandate` in `src/adapters/driven/yaml.rs` turns mandate YAML text into a
`Mandate`, or fails with one of two errors:

- `ParseError::Malformed`, wrapping the underlying `yaml_serde::Error`. This
  covers YAML that does not parse at all, a missing required field, and an
  unknown field at any level, since every serde struct in `yaml.rs`
  (`MandateDoc`, `RuleDoc`, `GovernedDocDoc`, `CodeLinkDoc`) carries
  `#[serde(deny_unknown_fields)]`.
- `ParseError::InvalidRule { id, reason }`, produced after the YAML shape has
  already parsed, when a rule's `type` and its fields disagree: `type:
  script` without `run`, `type: script` with a `prompt` present, `type:
  agent` without `prompt`, `type: agent` with a `run` present, or a `type`
  that is neither `script` nor `agent`.

Parsing stops at the first shape problem, because that is how serde
deserialization works: one YAML document either matches the target shape or
it does not. Collecting every problem in one pass is what validation does
instead, once a `Mandate` value exists to check.

## What validation checks

`validate` in `src/domain/validation.rs` takes a `Mandate` and a
`&FileTreeSnapshot` and returns a `ValidationReport { errors, warnings, ... }`
built in one pass: every check in the function runs regardless of what
earlier checks found. `report.is_valid()` is `true` exactly when `errors`
is empty; `warnings` never affects it.

`ValidationReport::lines` renders the report as one string per finding,
errors first in validator order, then warnings, each prefixed `error: ` or
`warning: ` and using the `Display` text in the table below.
`RunReportMandatesValidation::lines`, described in "The report" below,
indents these lines under each mandate's file name, and the fixture
helper's `check` returns them unchanged, so the printed format and its
order are defined in one place and every fixture case asserts exactly
what a user would see.

| Variant | Message printed | Meaning |
|---|---|---|
| `ValidationError::NoRules` | `no rules defined; a mandate needs at least one` | `rules` is empty. |
| `ValidationError::DuplicateRuleId { id }` | `duplicate rule id '<id>'` | Two rules share an `id`. |
| `ValidationError::UnknownRule { doc, rule }` | `rule '<rule>' is referenced by <doc> but not defined` | A `governs` entry lists a rule id that no `rules` entry defines. |
| `ValidationError::CodeLinksUngovernedDoc { path, doc }` | `code <path> links <doc> which this mandate does not govern` | A `code` entry names a document not present in `governs`. |
| `ValidationError::DocMissing { doc }` | `governed document not found: <doc>` | A `governs` path does not exist under the given `--root`. |
| `ValidationError::CodeMissing { path }` | `missing source file: <path>` | A `code` path does not exist under the given `--root`. |
| `ValidationWarning::UnreferencedRule { id }` | `rule '<id>' is defined but no document references it` | A rule in `rules` is referenced by no `governs` entry. |

`CodeLinksUngovernedDoc` is the validator's check for format rule 2 in
`README.md` section 4 ("a mandate may not link code to a document it does not
govern"). `UnreferencedRule` is the check for format rule 3 ("a rule defined
but referenced by no document is valid... worth a warning, and not an
error"): the format treats it as expected, so `validate` reports it as a
warning rather than an error.

## Fixture cases

`tests/fixtures/` holds one test target, `main.rs`, which declares a private
`support` module and one `mod <case>;` line per case. Adding a case means
creating its directory and adding that one line, the same way `src/`
declares its own modules; this is acceptable because `mod` is Rust's
module tree, not a registry the project maintains on the side.

Each case is a directory under `tests/fixtures/`, named for the check it
proves, holding two files:

- `mandate.yaml`, the mandate under test.
- `mod.rs`, the case's own test module, starting with
  `use crate::support::*;` and `include_str!("mandate.yaml")`.

The validation test parses the mandate with `parse`, builds a
`FakeVirtualMachine` with every linked file present via
`FakeVirtualMachine::with_every_file_in`, applies the case's edit in code
(`remove`, `rename`, or a change to the parsed `Mandate` value), then calls
`check`, which calls `validate` and returns `ValidationReport::lines`
unchanged, so a case asserts the lines in validator order: errors first,
following the mandate's own order of `rules`, `governs` and `code`, then
warnings.
Then the test asserts the result with `assert_passes`,
`assert_passes_with_warnings` or `assert_fails`. Because the assertion is
exact, an unexpected extra line fails the test. Pass or fail is in each
test function's name, and a case can hold more than one test; three of
the nine original cases do.

The five `run_*` cases exercise `RunMandates` instead: they build a
`FakeVirtualMachine` with `with_mandate` to add named mandate texts, edit
it with `add`, `remove` or `rename` for missing files, run it through
`RunMandates`, and assert on `RunReportMandatesValidation::lines()` or on
the `RunError` returned.

Each `mandate.yaml` is an independent copy, edited only where the case
needs it; there is nothing else it is kept in sync with.

| Case | What the test does | Proves |
|---|---|---|
| `all_links_present` | `passes_when_every_linked_file_is_present` parses the mandate, builds a repo with every linked file present, and checks the report is empty. | An unedited mandate with every file present validates clean. |
| `case_mismatch` | `fails_when_only_the_case_differs` renames `docs/sop/handling-mandates.md` to `docs/sop/Handling-Mandates.md` in the repo and asserts `governed document not found: docs/sop/handling-mandates.md`; `passes_with_the_exact_path` checks the unedited repo, with the lowercase path present, and passes. | Matching is case-sensitive. Chosen because Linux allows two names differing only by case in one directory, while Windows and macOS refuse it by default, so exact matching is the only behaviour that is the same on all three. |
| `code_missing` | `fails_when_linked_source_files_are_missing` removes `src/main.rs` and `tests/fixtures/doc_missing/mod.rs` from the repo and asserts both `missing source file:` lines. | `ValidationError::CodeMissing` fires once per missing source file. |
| `doc_missing` | `fails_when_a_governed_doc_is_missing` removes `docs/sop/handling-mandates.md` from the repo and asserts `governed document not found: docs/sop/handling-mandates.md`; `passes_once_the_doc_is_added_back` removes it and adds it back, and asserts the report is empty. | `ValidationError::DocMissing` fires for a governed document absent from the repo, and clears once the file is present again. |
| `duplicate_rule` | `fails_when_a_rule_id_is_duplicated` adds a second rule entry with id `claims-match-code` and asserts `duplicate rule id 'claims-match-code'`. | `ValidationError::DuplicateRuleId` fires on a repeated id. |
| `no_rules` | `fails_when_no_rules_are_defined` empties the mandate's `rules` list and every `governs` entry's `rules` list, and asserts `no rules defined; a mandate needs at least one`. | `ValidationError::NoRules` fires when a mandate defines no rules at all. |
| `ungoverned_doc` | `fails_when_code_links_a_doc_this_mandate_does_not_govern` adds a `docs` reference to `docs/architecture/other.md` in the `code` entry for `src/domain/mandate.rs`, which no `governs` entry lists; the test asserts `code src/domain/mandate.rs links docs/architecture/other.md which this mandate does not govern`. | `ValidationError::CodeLinksUngovernedDoc` fires for format rule 2. |
| `unknown_rule` | `fails_when_a_document_references_an_undefined_rule` adds a rule reference `no-such-rule` to the `governs` entry for `docs/architecture/mandate-parser.md`, which no `rules` entry defines; the test asserts `rule 'no-such-rule' is referenced by docs/architecture/mandate-parser.md but not defined`. | `ValidationError::UnknownRule` fires for a dangling rule reference. |
| `unreferenced_rule` | `passes_with_a_warning_when_a_rule_is_unreferenced` adds an `unused-rule` entry to `rules` assigned to no document and asserts the report has no errors and exactly the warning `rule 'unused-rule' is defined but no document references it`; `passes_clean_once_the_rule_is_referenced` pushes `unused-rule` onto the first `governs` entry's `rules` list in code and asserts the report is then empty. | `ValidationWarning::UnreferencedRule` fires and does not fail the mandate, per format rule 3, and clears once the rule is referenced. |
| `run_all` | `two_mandates_run_and_one_is_invalid` builds two mandates and makes the second invalid by pointing one of its `code` links at a file the first mandate does not have (`src/domain/mandate_only_in_b.rs`), selects none on the command line, and asserts both mandates appear in the report, the report's exact `lines()`, and that the run is invalid. | Running with no names runs every mandate found, and one invalid mandate does not stop the others from being reported. |
| `run_named` | `selecting_one_name_runs_only_it` builds a repo with two mandates, names one (`b.yaml`) on the command line, and asserts the report has only that one outcome and is valid. | Selection by file name runs only the named mandate. |
| `run_unknown_name` | `unknown_name_errors_and_lists_both_available` builds a repo with two mandates, names a file that does not exist, and asserts `RunError::UnknownMandate` naming it and listing both `a.yaml` and `b.yaml` as available. | An unknown name is an error before anything runs. |
| `run_from_subdirectory` | `starts_below_root_and_finds_it_by_walking_up` builds a repo with a `.mandate` folder at `/repo` and starts discovery from `/repo/src`; asserts the report's root is `/repo` and the run is valid. | Discovery climbs from the starting directory to the nearest ancestor with `.mandate`. |
| `run_no_mandates` | `no_mandate_files_warns_and_reports_zero` builds a repo with an empty `.mandate/mandates` folder; asserts `RunWarning::NoMandatesFound` naming the mandates directory, zero mandates in the report, and that the run is valid. | An empty mandates folder is a warning, not a failure. |
| `run_no_mandate_folder` | `no_mandate_folder_anywhere_up_warns_and_reports_zero` starts discovery from a directory with no `.mandate` in any ancestor; asserts the warning message naming the starting directory, zero mandates in the report, and exit 0. | Finding no `.mandate` folder is a warning, not a failure. |

## Running it

From anywhere inside an adopting project:

```
mandate
```

finds the project's `.mandate` folder by climbing from the current
directory and validates every mandate it lists. To validate only some:

```
mandate SOP_Orders.yaml Networking.yaml
```

To start discovery somewhere other than the current directory:

```
mandate --root /path/to/project
```

This repository has no mandate of its own yet, so there is nothing here
to run it against.

Exit codes: `0` when every selected mandate is valid, including the cases
of an empty mandates folder and a missing `.mandate` folder (both warnings,
not failures); `1` when any mandate failed to parse or validated with errors,
or when a named mandate does not exist. `parse_args` in `src/adapters/driving/cli.rs`
produces the `usage: mandate [--root <dir>] [<mandate-file>.yaml ...]`
message when the command line does not match that shape.

`cargo test` runs 77 tests: 46 unit tests under `src/` (6 in
`adapters::cli::tests`, 5 in `adapters::yaml::tests`, 5 in
`domain::file_tree::tests`, 5 in `domain::run_report::tests`, 13 in
`domain::validation::tests`, 12 in `application::run_mandates::tests`),
10 in `tests/fs_adapters.rs`, and 21 in the `fixtures` target: 12 across
the nine validation cases, 6 across the six run cases, and 3 for the
support module.

## Decisions

- `yaml_serde` was chosen over `serde_yaml` because `serde_yaml` is archived
  and `yaml_serde` is the YAML organisation's maintained fork.
- Validation collects every problem in one pass, rather than stopping at the
  first, so an author fixing a mandate sees the full list at once instead of
  discovering errors one `cargo run` at a time.
- Unknown fields are parse errors, via `deny_unknown_fields` on every serde
  struct, so a typo like `governes` cannot silently be ignored and do
  nothing.
- Paths are kept exactly as written in the mandate. Nothing normalises them.
- A governed document that no `code` entry links to is not an error.
  Partial coverage is the normal state (`README.md` section 4).
- Empty `governs` and empty `code` lists are not errors. Only an empty
  `rules` list is (`ValidationError::NoRules`).
- The CLI is a driving adapter tested in-process, in `src/adapters/driving/cli.rs`
  itself, rather than through the built binary. The binary will gain
  startup side effects of its own, so no test launches it; `parse_args`
  and `render` are exercised directly instead. The exit code mapping
  lives in `main.rs` as one `if` on `is_valid()` and is not unit tested.
- Validation behaviour is proven by fixture cases inside one test target,
  each a directory with its mandate and its tests, built on one shared
  fake virtual machine, so a case is cheap to add and can prove several
  things.
- The file tree snapshot is a domain value, not a port the domain calls
  live, so `validate` takes a plain `&FileTreeSnapshot` and one snapshot
  can be shared across every mandate in a run without retaking it.
- Discovery checks each ancestor with a cheap existence check,
  `FileTreeSource::has_directory`, rather than snapshotting it, and
  snapshots only the ancestor that turns out to hold `.mandate`, because
  snapshotting every ancestor walked directories the current user does
  not own; a real run failed with "Access is denied" on a system temp
  folder before it could report `.mandate` was never found.
- Mandate selection is by file name, case-sensitive, never by the `name:`
  field inside the file, so two mandates cannot collide on a name a user
  did not choose as a file name.
- A run validates and reports every selected mandate rather than stopping
  at the first invalid one, matching the choice already made inside
  `validate` itself: an author sees every problem in one pass.
- Finding no `.mandate` folder is a warning with exit 0, the same as an
  empty mandates folder, since nothing to check is not a failure.
