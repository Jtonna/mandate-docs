# Documentation index

## How docs/ is organised

- `architecture/`: how this system is designed, the parts, how they
  connect, and the flows through it. Nothing about another system or an
  external pattern.
- `reference/`: exact inputs, outputs, and options, structured for lookup,
  whether about this system or an external pattern it uses. No narrative
  or steps.
- `sop/`: numbered steps for a routine, repeatable task on this system. No
  rationale or architecture.

Every document under `docs/`, grouped by folder.

## architecture/

| Path | Type | Covers |
|---|---|---|
| `architecture/mandate-parser.md` | Architecture overview | Purpose and big-picture design of the mandate parser, validator and run command crate, with links to the documents below. |
| `architecture/run-command.md` | Architecture overview (one flow) | How one run flows end to end: discovery, snapshot, selection, per-mandate outcome, reporting, exit codes, and the rendered report layouts. |
| `architecture/ports-and-adapters.md` | Architecture overview (internals) | The file-by-file shape of the crate, the three driven ports' Rust signatures, and the `FileSystem` seam. |
| `architecture/testing.md` | Architecture overview (test wiring) | How the test suite is wired, the fixture cases, the test table, and how to run it. |

## reference/

| Path | Type | Covers |
|---|---|---|
| `reference/mandate-format-checks.md` | Reference | What mandate YAML parsing rejects and what mandate validation checks, with the message table. |
| `reference/HEXAGONAL_ARCHITECTURE_PORTS_AND_ADAPTERS_REFERENCE.md` | Reference (external pattern guide) | General reference material on the ports and adapters (hexagonal) pattern; not documentation of this system. |

## sop/

| Path | Type | Covers |
|---|---|---|
| `sop/handling-mandates.md` | Standard operating procedure | The steps for writing or changing a mandate file and keeping `.mandate/mandate.json` in step with it. |
