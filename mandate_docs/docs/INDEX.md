# Documentation index

Every document under `docs/`, grouped by folder.

## architecture/

| Path | Type | Covers |
|---|---|---|
| `architecture/mandate-parser-overview.md` | Architecture overview | Purpose and big-picture design of the mandate parser, validator and run command crate, with links to the documents below. |
| `architecture/run-command.md` | Architecture overview (one flow) | How one run flows end to end: discovery, snapshot, selection, per-mandate outcome, reporting, exit codes, and the rendered report layouts. |
| `architecture/ports-and-adapters.md` | Architecture overview (internals) | The file-by-file shape of the crate, the three driven ports' Rust signatures, and the `FileSystem` seam. |
| `architecture/testing.md` | Architecture overview (test wiring) | How the test suite is wired, the fixture cases, the test table, and how to run it. |
| `architecture/PORTS_AND_ADAPTERS_GUIDE.md` | Reference (external pattern guide) | General reference material on the ports and adapters (hexagonal) pattern; not documentation of this system. |
| `architecture/decisions/README.md` | ADR index | What the ADRs in this folder are and the numbering rule. |
| `architecture/decisions/0001-yaml-library-choice.md` | ADR | Choosing `yaml_serde` over the archived `serde_yaml`. |
| `architecture/decisions/0002-unknown-fields-are-errors.md` | ADR | Rejecting unknown YAML fields as a parse error. |
| `architecture/decisions/0003-validation-collects-every-problem.md` | ADR | Collecting every problem in one pass, per mandate and per run. |
| `architecture/decisions/0004-partial-coverage-is-not-an-error.md` | ADR | Partial coverage between documents, code and rules is not an error. |
| `architecture/decisions/0005-exact-matching-no-normalization.md` | ADR | Matching paths and mandate names exactly, never normalised. |
| `architecture/decisions/0006-fixture-tests-share-real-adapters.md` | ADR | Fixture tests build real adapters over one shared fake, in one test target. |
| `architecture/decisions/0007-file-tree-snapshot-is-a-domain-value.md` | ADR | The file tree snapshot is a domain value, shared across a run, not a live port. |
| `architecture/decisions/0008-discovery-avoids-full-snapshots.md` | ADR | Discovery checks ancestors with a cheap check and snapshots only the found root. |
| `architecture/decisions/0009-missing-mandates-are-warnings.md` | ADR | A missing `.mandate` folder or empty mandates folder is a warning, not a failure. |
| `architecture/decisions/0010-cli-tested-in-process.md` | ADR | Testing the CLI adapter in-process rather than through the built binary. |
| `architecture/decisions/0011-presentation-in-driving-adapter.md` | ADR | Presentation and the exit-code split both live in the driving adapter and `main.rs`. |
| `architecture/decisions/0012-port-error-types-are-domain-types.md` | ADR | Port error types are domain types, and the dependency rules are enforced by a test. |

## reference/

| Path | Type | Covers |
|---|---|---|
| `reference/mandate-format-checks.md` | Reference | What mandate YAML parsing rejects and what mandate validation checks, with the message table. |

## sop/

| Path | Type | Covers |
|---|---|---|
| `sop/handling-mandates.md` | Standard operating procedure | The steps for writing or changing a mandate file and keeping `.mandate/mandate.json` in step with it. |
