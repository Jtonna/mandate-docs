# Doc-Engine Architecture & Design

*Technical overview for developers and architects.*

## 1. System Overview

Doc-engine is a documentation system with an unusual constraint set: documents must be **plain Markdown files in a git repo** (reviewable, diffable, greppable) *and* **structured, validated records** (queryable, template-checked, safely editable by AI agents). It resolves this with two storage layers kept in two-way sync:

- A **SQLite database** (`.doc-engine/engine.db`) — the source of truth for structure, validation state, and pending proposals.
- **Physical Markdown files** (`docs/`) — the human- and git-facing artifact.

A hash-based sync layer detects divergence between the two and forces explicit resolution rather than silent overwrites. On top of the core sits an agent integration surface: MCP tools that let AI assistants read documents, draft changes, and validate them against templates — with all writes gated behind a human approval step.

The system follows a strict ports & adapters (hexagonal) architecture: a pure domain core, port interfaces owned by the domain, and swappable adapters for storage, formatting, and conflict detection.

```
        DRIVING                    CORE                      DRIVEN
 ┌───────────────┐         ┌─────────────────┐         ┌───────────────┐
 │  MCP tools    │──┐      │     DOMAIN      │      ┌──│ SQLiteStorage │
 │  (doc-get,    │  │      │                 │      │  │   Provider    │
 │   doc-propose,│  │      │  workflows:     │      │  └───────────────┘
 │   doc-validate│  ├─────▶│   init, query,  │──────┤  ┌───────────────┐
 └───────────────┘  │      │   propose,      │      ├──│ MarkdownFmt   │
 ┌───────────────┐  │      │   approve, sync │      │  │   Provider    │
 │  API routes   │──┤      │                 │      │  └───────────────┘
 └───────────────┘  │      │  validation     │      │  ┌───────────────┐
 ┌───────────────┐  │      │  sync logic     │      └──│ HashConflict  │
 │  Web UI       │──┘      │  link checking  │         │   Detector    │
 └───────────────┘         └─────────────────┘         └───────────────┘
                    PORTS ──▲                ▲── PORTS
                 (use cases)      (StorageProvider, FormattingProvider,
                                   ConflictDetector)
```

## 2. Architecture Layers

### Domain

Pure logic, no I/O, no SDK types. Contains:

- **Workflows** (use cases): `InitializeEngine`, `GetDocument`, `ProposeChange`, `ApproveProposal`, `ValidateDocument`, `SyncRepository`. Each is a class taking its ports via constructor injection.
- **Sync logic**: the state machine deciding whether DB and files agree, and what resolution options exist when they don't.
- **Validation**: template rule evaluation (required sections, ordering, frontmatter schemas, link resolution) expressed over a parsed document model — never over raw Markdown strings.
- **Models**: `Document`, `Template`, `Proposal`, `SyncState`, `ValidationResult` — domain types with invariants (e.g., a `Proposal` cannot transition to `approved` if its base hash no longer matches the live document).

### Ports

Interfaces owned by the domain, in domain vocabulary:

```ts
interface StorageProvider {
  getDocument(id: DocumentId): Promise<StoredDocument | null>;
  saveDocument(doc: StoredDocument): Promise<void>;
  listByTemplate(t: TemplateName): Promise<DocumentSummary[]>;
  saveProposal(p: Proposal): Promise<void>;
  recordSyncState(s: SyncState): Promise<void>;
}

interface FormattingProvider {
  parse(raw: string): ParsedDocument;        // file → structured model
  render(doc: ParsedDocument): string;       // structured model → file
  readPhysical(path: DocPath): Promise<string>;
  writePhysical(path: DocPath, content: string): Promise<void>;
}

interface ConflictDetector {
  fingerprint(content: string): ContentHash;
  compare(dbHash: ContentHash, fileHash: ContentHash): SyncVerdict;
}
```

Note what the ports do *not* say: no SQL, no file-system paths beyond an opaque `DocPath`, no hashing algorithm. Any implementation satisfying the contract is a drop-in.

### Adapters

- **SQLiteStorageProvider** — implements `StorageProvider` over better-sqlite3; owns schema migrations, transactions, and mapping between rows and domain types.
- **MarkdownFormattingProvider** — implements `FormattingProvider`; parses frontmatter + heading structure into `ParsedDocument`, renders back deterministically (stable rendering is what makes hash comparison meaningful).
- **HashConflictDetector** — implements `ConflictDetector` with SHA-256 over *normalized* content (line endings and trailing whitespace normalized before hashing, so a CRLF checkout doesn't read as a conflict).

Adapters translate errors at the boundary: a SQLite constraint violation surfaces as `DuplicateDocument`, never as a driver exception.

### Presentation (driving adapters)

- **MCP tool handlers** — translate MCP tool calls into use-case commands (Section 6).
- **API routes** — a small HTTP layer serving the same use cases for the web UI.
- **Web UI** — read-and-review client of the API: browsing, rendering, sync badges, proposal diffs. It contains no business logic; approval clicks call the same `ApproveProposal` use case the CLI does.

## 3. Data Flow

### Initialization (fresh install)

```
scan docs/ ──▶ parse each file ──▶ match to template ──▶ validate ──▶ build DB
                (FormattingProvider)                                 (StorageProvider)
```

`InitializeEngine` walks the docs directory, parses each file, infers its template from frontmatter, validates, computes its content hash, and writes document + sync-state rows. The DB is therefore always *derivable* from the files — a deliberate property that makes the DB disposable (Section 4).

### Query flow (`doc-get`)

```
request ──▶ load DB record ──▶ hash physical file ──▶ compare
                                        │
                     match ─────────────┼──────────── mismatch
                       │                                 │
              return document                   return SyncConflict
                                                (no content served)
```

Every read is also a sync check. A conflicted document is never served silently — the caller (human or agent) gets a structured conflict result with both hashes and resolution options.

### Change flow (propose → approve)

```
draft ──▶ validate vs template ──▶ store Proposal (DB only) ──▶ human review
                 │ fail                                              │ approve
                 ▼                                                   ▼
        ValidationResult                              re-validate + re-check base hash
        returned to caller                                           │
        for iteration                              write physical file + update DB row
                                                     (single transaction boundary)
```

Proposals live only in the database — the working tree stays clean while drafts iterate. Approval is the *only* path that writes a physical file through the engine, and it performs the file write and DB update together: DB row updated in a transaction that commits only after the physical write succeeds, with the previous file content retained for rollback if the commit fails.

### Sync detection

Runs on every read, on `doc-engine sync`, and on engine startup. `compare()` yields one of: `in_sync`, `file_ahead` (file edited outside the engine), `db_ahead` (approved change not yet flushed — normally transient), or `diverged` (both changed). Anything other than `in_sync` flips the document's `sync_state` row and, for `file_ahead`/`diverged`, raises a conflict.

## 4. Two-Way Sync Model

**The DB is the source of truth** for *structure and state*: validation results, template bindings, proposal history, approval records. **Physical files are the readable artifact**: what git tracks, what PRs review, what grep finds.

This split creates a classic dual-write problem; doc-engine's answer has three parts:

1. **Hash-based detection, not prevention.** The engine cannot stop `vim` or `git pull` from changing files — so it doesn't try. It fingerprints content on both sides and detects drift at the next interaction.
2. **Conflicts lock DB writes.** When a document is in conflict, the domain refuses new proposals and approvals against it (`DocumentLockedError`) until the conflict is resolved. This prevents an approval from clobbering an unseen human edit.
3. **Both directions are recoverable:**
   - `rebuild` — files win entirely: drop and reconstruct the DB by re-running initialization. Used after cloning, DB corruption, or `--prefer-files` resolution. Cheap because the DB is fully derivable from files. Pending proposals are the one thing lost — they exist only in the DB, which is why teams commit `.doc-engine/` when proposals must survive.
   - `sync --prefer-db` — DB wins: re-render each conflicted document from the DB's parsed model and overwrite the physical file. Used when the DB holds an approved change that a stray edit stomped.

The asymmetry is intentional: file-wins recovery is total (rebuild everything), DB-wins recovery is surgical (per document). Git remains the ultimate backstop for file content either way.

## 5. Plugin Points

Every port is a plugin API. The composition root selects adapters from configuration:

```yaml
# .doc-engine/config.yaml
storage:
  adapter: sqlite            # or: postgres, custom
  options: { path: .doc-engine/engine.db }
formatting:
  adapter: markdown          # or: asciidoc, custom
conflict:
  adapter: sha256
```

```ts
// composition root — the only file that knows every implementation
const storage = storageRegistry[config.storage.adapter](config.storage.options);
const fmt     = formatterRegistry[config.formatting.adapter]();
const engine  = buildEngine({ storage, fmt, conflict });
```

- **Custom storage drivers**: a `PostgresStorageProvider` makes doc-engine multi-writer for large orgs (shared DB, many repos). The domain is unchanged — the port contract is the entire integration surface.
- **Custom formatters**: an `AsciiDocFormattingProvider` swaps the file format; because validation runs against `ParsedDocument`, not raw text, all template rules keep working.
- **Custom detectors and future adapters**: e.g., a git-aware `ConflictDetector` that uses blob hashes, or notification adapters (a `ProposalNotifier` port is the planned extension point for Slack/email review pings).

Third-party adapters load via dynamic import of a module exporting `(options) => Port`, verified against the port contract at startup — fail fast, never mid-request. A published contract-test kit lets adapter authors prove conformance.

## 6. MCP Tool Integration

Three tools, mapped one-to-one onto use cases:

| Tool | Use case | Writes? |
|---|---|---|
| `doc-get` | `GetDocument` | No |
| `doc-propose` | `ProposeChange` | DB only (proposal row) |
| `doc-validate` | `ValidateDocument` | No |

- **`doc-get`** returns the document plus its template's agent SOP, so the calling agent receives quality guidance alongside content. If the document is in conflict, the tool returns the conflict structure instead — agents cannot read stale content unknowingly.
- **`doc-propose`** accepts a full draft, validates it, and either stores a proposal (returning its ID) or returns the `ValidationResult` for iteration. It never writes a physical file — the human approval gate is architectural, not procedural.
- **`doc-validate`** is the agent's inner loop: validate a draft, read the failures, revise, repeat — with zero side effects.

The MCP handlers are thin driving adapters: parse tool input → build command → invoke use case → serialize result. No business decisions live in them.

## 7. Database Schema (High Level)

```
templates    (id, name, version, formatter_rules_json, agent_sop, source)
documents    (id, path, title, template_id, template_version,
              parsed_model_json, content_hash, validation_status)
proposals    (id, document_id, base_hash, draft_model_json,
              validation_result_json, status, author, created_at, resolved_at)
sync_state   (document_id, db_hash, last_file_hash, verdict,
              locked, last_checked_at)
```

Key relationships: `documents.template_id → templates` (with `template_version` pinning documents to the version they validated against); `proposals.document_id → documents` with `base_hash` capturing the document state the draft was written against — approval fails if `base_hash` no longer matches, giving optimistic concurrency for proposals. `sync_state` is one row per document; `locked` is the flag the domain checks before permitting writes.

## 8. Error Handling

- **Sync conflicts** → `SyncConflictError` carrying both hashes and resolution options. The document's `sync_state.locked` is set; proposals and approvals against it are refused until resolved. Reads return the conflict structure, never stale content.
- **Validation failures** → never exceptions; a `ValidationResult` value with per-rule failures (rule id, section, message, location). This is what makes agent iteration cheap — the result is designed to be machine-readable feedback, not a stack trace.
- **Link validation** → cross-document references are checked at validation time against the documents table. A link to a missing document is a validation failure; deleting a document that others link to is refused unless `--force`, and the web UI surfaces backlinks so authors can fix inbound references first.
- **Adapter failures** → translated at the boundary into port-declared errors (`StorageUnavailable`, `PhysicalWriteFailed`); an approval interrupted between file write and DB commit rolls back the file from retained prior content.

## 9. Extensibility Model

- **Dependency injection throughout.** Use cases receive ports via constructors; nothing in the domain constructs an adapter. All wiring lives in one composition root, resolved at startup from config — a missing binding fails startup, never a request.
- **Configuration-driven selection.** The registry pattern above keys adapter factories by config string. Adding an adapter means registering a factory; the core is untouched.
- **Versioned adapter contracts.** Port changes are semver events: additive optional methods are minor; signature changes are major. Optional capabilities extend rather than break — `interface TransactionalStorage extends StorageProvider` — with the core feature-detecting at startup. Breaking changes version the port (`StorageProviderV2`) with a V1→V2 shim during the deprecation window, so third-party adapters keep working.

The result is the standard hexagonal asymmetry, applied to documentation tooling: template and workflow changes touch only the domain; storage and format changes touch only adapters; and the human-approval gate on writes is enforced by the shape of the use cases themselves, not by convention.
