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

Dependencies point one way: the adapters and the composition root import the
domain; the domain imports nothing outside `std`. `FileTree` is a driven
port because the domain calls it to ask whether a path exists; `src/adapters/cli.rs`
is the sole driving adapter, since nothing else calls into the crate yet.

Tests follow the split in `README.md` section 10, step 3:

| File | Kind | What it covers |
|---|---|---|
| `src/adapters/yaml.rs` | unit, in-file | The five parse-error cases, with inline YAML. |
| `src/domain/validation.rs` | unit, in-file | Every validation error and warning, and their `Display` strings, using a private `FakeTree` so the domain test module imports nothing from an adapter. |
| `tests/file_tree_contract.rs` | integration, contract | One assertion function run against both `InMemoryFileTree` and `FsFileTree`, the latter on a real temporary directory. |
| `src/adapters/cli.rs` | unit, in-file | Argument parsing and `execute`, using a private fake `FileTree` and in-memory writers. No process is launched. |
| `tests/common/mod.rs` | shared loader | Loads a fixture case directory for `tests/validation_fixtures.rs`. Not a test target itself. |
| `tests/validation_fixtures.rs` | integration | Every case under `tests/fixtures/validation/` parsed and validated against trees captured on Windows, Linux and macOS. |

Tests never read `.mandate/` or `docs/`, and no test runs the built binary.

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
`<CHECK>_PASS<n>` or `<CHECK>_FAIL<n>`. Each directory holds five files:

- `mandate.yaml`, the mandate under test.
- `tree.windows.txt`, `tree.linux.txt`, `tree.macos.txt`, one file listing
  per operating system.
- `expected.txt`, the report the case must produce.

A tree file's first line is the repository root exactly as that OS prints
it; every following line is one listing entry in that OS's format,
directories included. `tests/common/mod.rs` strips the root prefix,
converts backslashes to forward slashes, and strips any leading `./`, so
the same case can be checked against three different path conventions from
one set of repo-relative paths. An entry that is not under the root,
other than the root itself, is rejected with a panic naming the line,
so a hand-written tree cannot silently produce a wrong path.

`expected.txt` starts with `PASS` or `FAIL` on line 1, followed by the
exact `error:` and `warning:` lines the report must contain, in any order.

Every fixture `mandate.yaml` is a snapshot copy of
`.mandate/mandates/Mandate_Parser.yaml`, edited only where the case needs
it. It does not track the real mandate, so a later edit to
`Mandate_Parser.yaml` does not change what a fixture asserts.

| Case | Edit | Proves |
|---|---|---|
| `ALL_LINKS_PRESENT_PASS1` | None; an unedited copy of the real mandate and tree. | The real mandate validates clean on all three operating systems. |
| `CASE_MISMATCH_FAIL1` | Tree lists `docs/sop/Handling-Mandates.md` instead of `docs/sop/handling-mandates.md`. | Matching is case-sensitive on every operating system, and the case fails on all three trees. The rule is exact matching because the three disagree: Linux allows two names differing only by case in one directory, while Windows and macOS refuse it by default, so exact matching is the only behaviour that is the same everywhere. |
| `CODE_MISSING_FAIL1` | Tree omits `src/main.rs` and `tests/validation_fixtures.rs`. | `ValidationError::CodeMissing` fires once per missing source file. |
| `DOC_MISSING_FAIL1` | Tree omits the whole `docs/sop/` directory. | `ValidationError::DocMissing` fires for a governed document absent from the tree. |
| `DUPLICATE_RULE_FAIL1` | A second rule with id `claims-match-code` is appended to `rules`. | `ValidationError::DuplicateRuleId` fires on a repeated id. |
| `NO_RULES_FAIL1` | `rules` is emptied and every `governs` entry's `rules` list is emptied with it. | `ValidationError::NoRules` fires when a mandate defines no rules at all. |
| `UNGOVERNED_DOC_FAIL1` | A `code` entry gains a `docs` reference to `docs/architecture/other.md`, which no `governs` entry lists. | `ValidationError::CodeLinksUngovernedDoc` fires for format rule 2. |
| `UNKNOWN_RULE_FAIL1` | A `governs` entry references rule id `no-such-rule`, which no `rules` entry defines. | `ValidationError::UnknownRule` fires for a dangling rule reference. |
| `UNREFERENCED_RULE_PASS1` | A new rule `unused-rule` is added to `rules` but assigned to no document. | `ValidationWarning::UnreferencedRule` fires and does not fail the mandate, per format rule 3. |

## Running it

From `mandate_docs/`:

```
cargo run -- validate <mandate-file> --root <dir>
```

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

`cargo test` runs 28 tests: 23 unit tests under `src/` (7 in
`adapters::cli::tests`, 5 in `adapters::yaml::tests`, 11 in
`domain::validation::tests`), 2 in `tests/file_tree_contract.rs`, and 3 in
the `tests/validation_fixtures.rs` target: two unit tests of the loader in
`tests/common/mod.rs` (it strips each operating system's root format, and it
rejects an entry that is not under the root) and one test that walks all 9
cases under `tests/fixtures/validation/` against each of their 3 trees.

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
- Validation behaviour is proven by the fixture directories under
  `tests/fixtures/validation/`, not by prose alone, so a reader can see
  what passes and what fails, and why, without reading any Rust.
