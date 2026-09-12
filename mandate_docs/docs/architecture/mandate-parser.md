# Mandate parser and validator

## Purpose

This is the first real software in the repository: a Rust crate that parses
one mandate YAML file into a Rust value, validates it against the mandate
format and against the files it names, and reports every problem it finds in
one pass. It does not read or write `mandate.json`, does not scan the
repository for mandates, documents or source files, and does not execute a
rule. Those remain out of scope, as recorded in `README.md` section 5.

## Shape

The crate follows `docs/architecture/PORTS_AND_ADAPTERS_GUIDE.md`, scaled to
the size of the problem: one port, two adapters, no application layer,
because there is no use case beyond "parse, then validate".

| File | Layer | Role |
|---|---|---|
| `src/domain/mandate.rs` | domain | Plain data types for a mandate: `Mandate`, `Rule`, `RuleKind`, `GovernedDoc`, `CodeLink`. No serde, no I/O. |
| `src/domain/validation.rs` | domain | `validate`, `ValidationReport`, `ValidationError`, `ValidationWarning`. Checks a `Mandate` against a `FileTree`. |
| `src/domain/ports/file_tree.rs` | port | The `FileTree` trait the domain depends on. |
| `src/adapters/yaml.rs` | driven adapter | `parse_mandate` and `ParseError`. Turns mandate YAML text into a `Mandate`. |
| `src/adapters/memory_tree.rs` | driven adapter | `InMemoryFileTree`, a `FileTree` backed by a set of paths. |
| `src/adapters/fs_tree.rs` | driven adapter | `FsFileTree`, a `FileTree` backed by the real filesystem. |
| `src/adapters/cli.rs` | driving adapter | `parse_args` turns the process argument list into an `Invocation`. `execute` parses the mandate text, validates it, and prints the report to the given writers, returning the exit code. Neither function does any I/O of its own. |
| `src/main.rs` | composition root | Collects the process arguments, reads the mandate file, builds an `FsFileTree`, calls `execute`, and maps its exit code to the process exit status. |
| `src/lib.rs` | composition root | Declares the `adapters` and `domain` modules. |

Dependencies point one way: the adapters and the composition root import
the domain; the domain imports nothing outside `std`. `FileTree` is a
driven port because the domain calls it to ask whether a path exists;
`src/adapters/cli.rs` is the sole driving adapter, since nothing else calls
into the crate yet.

Tests follow the split in `README.md` section 10, step 3:

| File | Kind | What it covers |
|---|---|---|
| `src/adapters/yaml.rs` | unit, in-file | The five parse-error cases, with inline YAML. |
| `src/domain/validation.rs` | unit, in-file | Every validation error and warning, and their `Display` strings, using a private `FakeTree` so the domain test module imports nothing from an adapter. |
| `tests/file_tree_contract.rs` | integration, contract | One assertion function run against both `InMemoryFileTree` and `FsFileTree`, the latter on a real temporary directory. |
| `src/adapters/cli.rs` | unit, in-file | Argument parsing and `execute`, using a private fake `FileTree` and in-memory writers. No process is launched. |
| `tests/fake_virtual_machine/mod.rs` | shared helper | An in-memory `FakeVirtualMachine` that implements `FileTree`, built from a mandate with every linked file present, then edited with `add`, `remove` and `rename`, plus `parse`, `check` and the `assert_*` helpers. Has its own unit tests. |
| `tests/validation_fixtures.rs` | integration | The root that includes each case's `test.rs`. |
| `tests/fixtures/validation/<CASE>/test.rs` | integration | One or more tests per case. |

Tests never read `docs/`, and no test runs the built binary. This repository
has no `.mandate/` folder to read.

The `serde`-derived structs (`MandateDoc`, `RuleDoc`, `GovernedDocDoc`,
`CodeLinkDoc`) live only in `src/adapters/yaml.rs` and never leave that
module. `parse_mandate` maps each one into the plain domain types in
`src/domain/mandate.rs` before returning. This follows the guide's rule that
no library type may enter the domain: the domain types carry no `serde`
attributes and would compile unchanged if the YAML library were replaced.

## The port

`FileTree` has one method:

```rust
fn exists(&self, repo_relative_path: &str) -> bool
```

Two adapters implement it. `InMemoryFileTree` holds a set of path strings and
is both the fake used in domain tests and, per the architecture guide, a
usable adapter in its own right. `FsFileTree` holds a root directory and
answers by joining the root with the given path and checking whether the
result exists on disk.

Both implementations are held to the same contract by
`tests/file_tree_contract.rs`, which runs one assertion function, taking a
`&dyn FileTree`, against each: a known path exists, and an unknown path does
not.

## What parsing rejects

`parse_mandate` in `src/adapters/yaml.rs` turns mandate YAML text into a
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

`validate` in `src/domain/validation.rs` takes a `Mandate` and a `&dyn
FileTree` and returns a `ValidationReport { errors, warnings }` built in one
pass: every check in the function runs regardless of what earlier checks
found. `report.is_valid()` is `true` exactly when `errors` is empty;
`warnings` never affects it.

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

`tests/fixtures/validation/` holds one directory per case, named
`<CHECK>_PASS<n>` or `<CHECK>_FAIL<n>`. Each directory holds two files:

- `mandate.yaml`, the mandate under test.
- `test.rs`, the case's own test module.

The test parses the mandate with `fake_virtual_machine::parse`, builds a
`FakeVirtualMachine` with every linked file present via
`FakeVirtualMachine::with_every_file_in`, applies the case's edit in code
(`remove`, `rename`, or a change to the parsed `Mandate` value), then calls
`fake_virtual_machine::check` and asserts the exact
report lines with `assert_passes`, `assert_passes_with_warnings` or
`assert_fails`. Because the assertion is exact, an unexpected extra line
fails the test. A case can hold more than one test, and three of the nine
do. Registering a case means adding one `#[path]` line to
`tests/validation_fixtures.rs`, naming the case's `test.rs`. A test in that
file reads the case directories and fails if any directory has no
registration line, or any registered name has no directory.

Each `mandate.yaml` is an independent copy, edited only where the case
needs it; there is nothing else it is kept in sync with.

| Case | What the test does | Proves |
|---|---|---|
| `ALL_LINKS_PRESENT_PASS1` | Parses the mandate, builds a repo with every linked file present, and checks the report is empty. | An unedited mandate with every file present validates clean. |
| `CASE_MISMATCH_FAIL1` | Two tests. One renames `docs/sop/handling-mandates.md` to `docs/sop/Handling-Mandates.md` in the repo and asserts `governed document not found: docs/sop/handling-mandates.md`. The other checks the unedited repo, with the lowercase path present, passes. | Matching is case-sensitive. Chosen because Linux allows two names differing only by case in one directory, while Windows and macOS refuse it by default, so exact matching is the only behaviour that is the same on all three. |
| `CODE_MISSING_FAIL1` | Removes `src/main.rs` and `tests/validation_fixtures.rs` from the repo and asserts both `missing source file:` lines. | `ValidationError::CodeMissing` fires once per missing source file. |
| `DOC_MISSING_FAIL1` | Two tests. One removes `docs/sop/handling-mandates.md` from the repo and asserts `governed document not found: docs/sop/handling-mandates.md`. The other removes it and adds it back, and asserts the report is empty. | `ValidationError::DocMissing` fires for a governed document absent from the repo, and clears once the file is present again. |
| `DUPLICATE_RULE_FAIL1` | The mandate's `rules` list has a second entry with id `claims-match-code`; the test asserts `duplicate rule id 'claims-match-code'`. | `ValidationError::DuplicateRuleId` fires on a repeated id. |
| `NO_RULES_FAIL1` | The mandate's `rules` list is empty, and every `governs` entry's `rules` list is empty with it; the test asserts `no rules defined; a mandate needs at least one`. | `ValidationError::NoRules` fires when a mandate defines no rules at all. |
| `UNGOVERNED_DOC_FAIL1` | The `code` entry for `src/domain/mandate.rs` gains a `docs` reference to `docs/architecture/other.md`, which no `governs` entry lists; the test asserts `code src/domain/mandate.rs links docs/architecture/other.md which this mandate does not govern`. | `ValidationError::CodeLinksUngovernedDoc` fires for format rule 2. |
| `UNKNOWN_RULE_FAIL1` | The `governs` entry for `docs/architecture/mandate-parser.md` gains a rule reference `no-such-rule`, which no `rules` entry defines; the test asserts `rule 'no-such-rule' is referenced by docs/architecture/mandate-parser.md but not defined`. | `ValidationError::UnknownRule` fires for a dangling rule reference. |
| `UNREFERENCED_RULE_PASS1` | Two tests. One parses the mandate, whose `rules` list has an added `unused-rule` entry assigned to no document, and asserts the report has no errors and exactly the warning `rule 'unused-rule' is defined but no document references it`. The other pushes `unused-rule` onto the first `governs` entry's `rules` list in code and asserts the report is then empty. | `ValidationWarning::UnreferencedRule` fires and does not fail the mandate, per format rule 3, and clears once the rule is referenced. |

## Running it

Run from an adopting project's root, with `--root .`:

```
cargo run -- validate <mandate-file> --root .
```

This repository has no mandate of its own yet, so there is nothing here
to run it against.

On success, `execute` prints one line per error (`error: <message>`, none in
this case), one line per warning (`warning: <message>`), then:

```
mandate '<name>' is valid: <n> rules, <n> documents, <n> source files
```

and exits `0`. On any validation error, `execute` prints the `error:` and
`warning:` lines and returns `1` with no success line. `parse_args` in
`src/adapters/cli.rs` produces the `usage: mandate validate <mandate-file>
--root <dir>` message when the command line does not match that shape
(missing arguments, a command other than `validate`, or a missing
`--root`), and `execute` produces `failed to parse '<path>': <error>` when
`parse_mandate` returns a `ParseError`. `src/main.rs` is the only place
that can print `failed to read '<path>': <os error>`, since only it reads
the mandate file from disk before calling `execute`.

`cargo test` runs 41 tests: 23 unit tests under `src/` (7 in
`adapters::cli::tests`, 5 in `adapters::yaml::tests`, 11 in
`domain::validation::tests`), 2 in `tests/file_tree_contract.rs`, and 16 in
the `tests/validation_fixtures.rs` target: 3 unit tests of
`FakeVirtualMachine` in `tests/fake_virtual_machine/mod.rs`, 12 tests
across the nine cases under
`tests/fixtures/validation/`, and one guard that every case directory has
its `#[path]` registration line.

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
- The CLI is a driving adapter tested in-process, in `src/adapters/cli.rs`
  itself, rather than through the built binary. The binary will gain
  startup side effects of its own, so no test launches it; `parse_args` and
  `execute` are exercised directly instead.
- Validation behaviour is proven by fixture directories whose data and a
  short test decide the outcome, built on one shared fake virtual machine
  rather
  than per-case file listings, so a case is cheap to add and can prove
  several things.
