# Project Init: Technical Design for Mandate-Docs

**Status:** Architecture design complete  
**Pattern:** Hexagonal (ports & adapters)  
**Purpose:** Define mandate-docs system architecture as a template-driven documentation system with git-friendly storage and AI agent integration

---

## 1. System Layers

Mandate-docs is organized as a pure hexagonal architecture with four concentric layers:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      PRESENTATION LAYER                              │
│  (Driving adapters: MCP tools, CLI, REST API, Web UI handlers)      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │            APPLICATION CORE (DOMAIN)                        │    │
│  │                                                              │    │
│  │  Workflows (Use Cases):                                     │    │
│  │  - InitializeEngine                                         │    │
│  │  - GetDocument                                              │    │
│  │  - ProposeChange                                            │    │
│  │  - ApproveProposal                                          │    │
│  │  - ValidateDocument                                         │    │
│  │  - SyncRepository                                           │    │
│  │                                                              │    │
│  │  Domain Models (entities with invariants):                  │    │
│  │  - Document                                                 │    │
│  │  - Template                                                 │    │
│  │  - Proposal                                                 │    │
│  │  - SyncState                                                │    │
│  │  - ValidationResult                                         │    │
│  │                                                              │    │
│  │  Validation & Sync Logic (pure computation)                 │    │
│  │  - Template rule evaluation                                 │    │
│  │  - Link resolution and checking                             │    │
│  │  - Conflict detection state machine                         │    │
│  │  - Proposal concurrent-write safety                         │    │
│  │                                                              │    │
│  │        ▲ PORT BOUNDARY: Outbound (secondary)                │    │
│  │        │ Interfaces the domain depends on                   │    │
│  └────────┼──────────────────────────────────────────────────────┘    │
│           │                                                            │
│  ┌────────┴──────────────────────────────────────────────────────┐   │
│  │              PORTS (Domain-owned interfaces)                  │   │
│  │                                                                │   │
│  │  - StorageProvider (save/load structured documents)          │   │
│  │  - FormattingProvider (parse/render Markdown)               │   │
│  │  - ConflictDetector (fingerprint & compare content)         │   │
│  │  - TemplateRegistry (load template definitions)            │   │
│  │                                                                │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                       │
├─────────────────────────────────────────────────────────────────────┤
│                      ADAPTER LAYER                                    │
│        (Driven adapters: implementations of ports)                   │
│                                                                       │
│  Storage:           FormattingProviders:      Utilities:            │
│  - SQLiteStorage    - MarkdownFormatter       - HashConflict        │
│  - PostgresStorage  - AsciiDocFormatter       - FileSystemTemplate  │
│  (custom adapters)  (custom adapters)         (custom adapters)     │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

### Layer responsibilities

| Layer | Role | Concerns | Imports |
|-------|------|----------|---------|
| **Domain** | Business logic, state machines, workflows | Validation rules, sync logic, proposal safety | Domain types, ports (interfaces) |
| **Ports** | Contracts between core and adapters | Abstract capabilities, error types, domain models | Domain types only |
| **Adapters (Driven)** | Technology-specific implementations | SQL schema, file I/O, hashing, ORM mapping | Ports, domain types, tech SDKs |
| **Presentation** | Request/response translation, CLI/MCP handling | Parsing, serialization, error mapping to transport format | Domain workflows, types |

**Dependency flow rule:** All imports point inward or sideways. Never: adapters ← domain, presentation ← domain (only calls, no imports).

---

## 2. Driving Adapters (Presentation)

Driving adapters translate external requests into domain use-case invocations. They are thin translation layers—no business logic, no decisions.

### 2.1 MCP Tools (Claude AI Agent Integration)

Three tools, one-to-one mapped to core use cases:

#### `doc-get`
- **Use case:** `GetDocument`
- **Input:** `{ documentId: string, checkSync?: boolean }`
- **Output:** `{ document: Document, template: TemplateInfo, syncState: SyncState } | SyncConflictError`
- **Side effects:** None (read-only)
- **Agent SOP:** Tool result includes agent-specific guidance from the document's template (quality standards, review checklist, previous edits)
- **Sync check:** Every read checks if file hash matches DB; returns conflict structure if mismatched, never stale content

#### `doc-propose`
- **Use case:** `ProposeChange`
- **Input:** `{ documentId: string, draftContent: string, authorNote?: string }`
- **Output:** `{ proposalId: string, validationResult: ValidationResult } | ValidationResult`
  - If valid: returns proposal ID; store complete
  - If invalid: returns validation result; no proposal stored
- **Side effects:** Writes to DB only (proposal row); never touches physical files
- **Idempotent:** No—each call creates a new proposal if valid; agents should cite proposal IDs to deduplicate

#### `doc-validate`
- **Use case:** `ValidateDocument`
- **Input:** `{ documentId: string, contentToValidate: string }`
- **Output:** `ValidationResult` (structured: `{ passed: bool, failures: RuleFailure[] }`)
- **Side effects:** None
- **Agent iteration:** Designed for tight inner loops—validate, read failures, revise, repeat at millisecond speeds

### 2.2 REST API Routes

Thin HTTP layer serving the same use cases. Routes map 1:1 to use cases:

| Route | Method | Use case | Handler |
|-------|--------|----------|---------|
| `/api/documents/:id` | GET | `GetDocument` | Fetch + sync check |
| `/api/documents/:id/validate` | POST | `ValidateDocument` | Validate draft |
| `/api/documents/:id/proposals` | POST | `ProposeChange` | Create proposal |
| `/api/documents/:id/proposals/:pid/approve` | POST | `ApproveProposal` | Approve (auth required) |
| `/api/sync` | POST | `SyncRepository` | Detect/resolve conflicts |
| `/api/templates` | GET | Query (N/A for Phase 1) | List available templates |

**Error handling:** All use-case exceptions are caught, translated to HTTP status + JSON error body (no stacktraces to clients).

### 2.3 Web UI (React / browser-based)

Read-and-review client for documents, proposals, templates:

- **Document browser:** Lists documents grouped by template; displays rendered content; shows sync state badges (in-sync, conflicted, pending-approval)
- **Proposal viewer:** Side-by-side diff of current vs. proposed; shows validation status; approval button (for authorized users) calls `ApproveProposal` use case
- **Sync controls:** Manual "rebuild DB" and "sync --prefer-db" buttons for conflict resolution
- **No logic:** All state mutation calls the same API routes

### 2.4 CLI Adapter (Phase 1+ or optional)

Command-line interface for local workflows:

```bash
doc-engine init <repo-path>           # InitializeEngine
doc-engine get <document-id>          # GetDocument
doc-engine validate <id> <file>       # ValidateDocument
doc-engine propose <id> <file>        # ProposeChange (stores proposal)
doc-engine approve <proposal-id>      # ApproveProposal (requires auth)
doc-engine sync [--prefer-files|--prefer-db]  # SyncRepository
```

CLI handlers parse arguments, build commands, invoke use cases, format output (JSON, tables, YAML). No business rules in this layer.

### 2.5 Composition Root

Single entry point that wires all adapters to ports:

```typescript
// main.ts — only file allowed to import everything
function buildApp(config: Config) {
  // Driven adapters (implementations of ports)
  const storage = buildStorageAdapter(config.storage);
  const formatter = buildFormatterAdapter(config.formatting);
  const conflictDetector = buildConflictDetector(config.conflict);
  const templateRegistry = buildTemplateRegistry(config.templates);

  // Domain workflows (use cases)
  const engine = new MandateDocsEngine({
    storage, formatter, conflictDetector, templateRegistry
  });

  // Driving adapters (presentation)
  const mcpHandlers = new McpToolHandlers(engine);
  const apiRoutes = new ApiRoutes(engine);
  const webUI = new WebUIServer(apiRoutes);
  const cliHandlers = new CliHandlers(engine);

  return { mcpHandlers, apiRoutes, webUI, cliHandlers };
}
```

---

## 3. Core Domain

The domain contains workflows (use cases) and state machines—pure logic with no I/O or framework dependencies.

### 3.1 Workflows (Use Cases)

Each is a class taking ports via constructor injection; orchestrates domain models and validates invariants.

#### `InitializeEngine`

**Purpose:** One-time setup on a fresh repo.

```typescript
class InitializeEngine {
  constructor(
    private storage: StorageProvider,
    private formatter: FormattingProvider,
    private templates: TemplateRegistry
  ) {}

  async execute(docsDirPath: string, config: InitConfig): Promise<InitResult> {
    // 1. Scan docs directory
    // 2. For each .md file:
    //    a. Parse with formatter (yields ParsedDocument)
    //    b. Infer template from frontmatter
    //    c. Validate against template rules
    //    d. Compute content hash
    //    e. Save to DB: document row + sync_state row (hashes, in_sync)
    // 3. Return summary (docs initialized, templates bound, validation results)
  }
}
```

**Invariants enforced:**
- All documents must match a known template
- No duplicate document IDs
- All DB writes succeed or all roll back (atomic)

#### `GetDocument`

**Purpose:** Retrieve a document with sync check—never return stale content.

```typescript
class GetDocument {
  constructor(
    private storage: StorageProvider,
    private formatter: FormattingProvider,
    private conflictDetector: ConflictDetector
  ) {}

  async execute(docId: DocumentId): Promise<Document | SyncConflictError> {
    // 1. Load document from storage (DB)
    // 2. Fingerprint physical file
    // 3. Compare: DBHash vs FileHash
    // 4. If in_sync: return document with ParsedContent
    // 5. If file_ahead or diverged: return SyncConflictError (no content)
    // 6. Update sync_state row with comparison result
  }
}
```

**Invariants enforced:**
- If sync conflict detected, document is locked (sync_state.locked = true)
- Never return Document if conflict

#### `ValidateDocument`

**Purpose:** Validate a draft against its template—no side effects.

```typescript
class ValidateDocument {
  constructor(private templates: TemplateRegistry) {}

  async execute(
    docId: DocumentId,
    draftContent: string,
    template: Template
  ): Promise<ValidationResult> {
    // 1. Parse draft with formatter (yields ParsedDocument)
    // 2. For each validation rule in template:
    //    a. Check required sections present
    //    b. Check ordering (e.g., frontmatter before headings)
    //    c. Validate frontmatter schema (YAML)
    //    d. Check links resolve (cross-document references)
    //    e. Custom rule lambdas (e.g., "max 10 headings")
    // 3. Return ValidationResult:
    //    - passed: boolean
    //    - failures: [{ ruleId, section, message, lineRange }]
  }
}
```

**Invariants enforced:**
- Validation runs only against ParsedDocument, never raw strings
- Link checks are atomic—all must resolve or all links are marked unresolved

#### `ProposeChange`

**Purpose:** Draft a change, validate, store as proposal in DB—never write files.

```typescript
class ProposeChange {
  constructor(
    private storage: StorageProvider,
    private validateDoc: ValidateDocument
  ) {}

  async execute(
    docId: DocumentId,
    draftContent: string,
    authorNote: string
  ): Promise<Proposal | ValidationResult> {
    // 1. Load current document from storage
    // 2. If document is locked (sync conflict), reject (DocumentLockedError)
    // 3. Validate draft against template
    // 4. If invalid, return ValidationResult (no proposal stored)
    // 5. If valid:
    //    a. Capture current document's content hash as base_hash
    //    b. Store Proposal row (DB only):
    //       - docId, draftContent, baseHash, validationResult (passed),
    //       - status: pending, author, created_at
    //    c. Return Proposal with proposalId
  }
}
```

**Invariants enforced:**
- Proposals cannot be created against locked documents
- base_hash must match document's current hash (optimistic concurrency control)
- Proposal is only stored if validation passes
- Physical files are never touched

#### `ApproveProposal`

**Purpose:** Human approval: re-validate, check base hash still matches, write to disk and DB atomically.

```typescript
class ApproveProposal {
  constructor(
    private storage: StorageProvider,
    private formatter: FormattingProvider,
    private validateDoc: ValidateDocument
  ) {}

  async execute(
    proposalId: ProposalId,
    authorizedUserId: string
  ): Promise<void | ApprovalError> {
    // 1. Load proposal from storage
    // 2. Load current document
    // 3. If document is locked, reject (DocumentLockedError)
    // 4. Validate proposal's draft against template (re-validate)
    // 5. If invalid, reject (ValidationFailedError)
    // 6. If proposal.base_hash ≠ document.content_hash, reject (BaseHashMismatchError)
    //    → Proposal is stale; agent/human must re-propose
    // 7. If all checks pass:
    //    a. Render draft to physical file (formatter)
    //    b. In a single transaction:
    //       - Update document row: update content, content_hash, validation_status = passed
    //       - Update proposal row: status = approved, resolved_at = now
    //       - Update sync_state: db_hash = new hash, verdict = in_sync
    //    c. On DB commit failure: restore file from prior backup
    //    d. Return success
  }
}
```

**Invariants enforced:**
- Approval is the *only* path to writing physical files through the engine
- File write and DB update are atomic (both succeed or both roll back)
- base_hash check prevents clobbering concurrent edits
- Documents are locked during approval (transient lock during transaction)

#### `SyncRepository`

**Purpose:** Detect and optionally resolve sync divergence across all documents.

```typescript
class SyncRepository {
  constructor(
    private storage: StorageProvider,
    private formatter: FormattingProvider,
    private conflictDetector: ConflictDetector
  ) {}

  async execute(
    mode: 'detect' | 'prefer-files' | 'prefer-db'
  ): Promise<SyncReport> {
    // 1. Iterate all documents
    // 2. For each: fingerprint physical file, compare to DB hash
    // 3. Update sync_state rows with new verdict
    // 4. Collect conflicts (file_ahead, diverged)
    //
    // 5. If mode == 'detect': return report, no writes
    // 6. If mode == 'prefer-files':
    //    a. Re-initialize: drop all document rows
    //    b. Re-scan and re-parse all .md files (like InitializeEngine)
    //    c. Lose all pending proposals (they live only in DB)
    // 7. If mode == 'prefer-db':
    //    a. For each conflicted document:
    //       - Re-render from DB's parsed model
    //       - Overwrite physical file
    //       - Update sync_state to in_sync
    //
    // 8. Return report: conflicts resolved, locked documents unlocked
  }
}
```

**Invariants enforced:**
- `prefer-files` is destructive (loses proposals); CLI prompts for confirmation
- `prefer-db` is per-document surgical; safe to run repeatedly
- After sync, no locked documents should remain (all conflicts resolved)

### 3.2 State Machines & Pure Logic

#### Sync State Machine

```
                     file_ahead          db_ahead
                   (file changed)    (approval pending)
                         │                │
                         ▼                ▼
in_sync ◄───────────────────────────────────────────►  conflicted
  ▲                                                         ▲
  │                                                         │
  └─────────────────── diverged ──────────────────────────┘
       (both changed)
```

- **in_sync:** DB content hash == file content hash; document unlocked
- **file_ahead:** File hash newer than DB; file edited outside engine; document locked
- **db_ahead:** DB hash newer than file; approval written to DB but not yet flushed to file (transient); document unlocked
- **diverged:** Both changed; manual resolution required; document locked
- **locked:** Boolean flag; proposals and approvals rejected against locked documents

#### Proposal Concurrency Logic

Proposals are immutable once created. A proposal cannot be approved if:
- `proposal.base_hash ≠ current_document.content_hash` → proposal is stale

This is optimistic concurrency control: proposal holds a snapshot hash; approval checks it matches before writing.

#### Validation State Machine

```
Draft ──validate──►  valid: pass          invalid: fail
                       │                       │
                       ▼                       ▼
                  + propose          + return ValidationResult
                       │                       │
                       ▼                       ▼
                   Proposal           (Agent iterates)
                   (in DB)
                       │
                       ├─► (conflicted) ─► locked
                       │
                       └─► approve ──► (written to disk + DB)
```

---

## 4. Ports (Domain Interfaces)

Interfaces owned by the domain, expressed in domain vocabulary, technology-agnostic. All error types are domain errors, never infrastructure exceptions.

### 4.1 StorageProvider

Abstracts persistence of structured documents, templates, and proposals.

```typescript
interface StorageProvider {
  // Documents
  getDocument(id: DocumentId): Promise<StoredDocument | null>;
  saveDocument(doc: StoredDocument): Promise<void>;
  listDocuments(filters?: QueryFilter): Promise<DocumentSummary[]>;
  listByTemplate(template: TemplateName): Promise<DocumentSummary[]>;

  // Proposals
  saveProposal(proposal: Proposal): Promise<ProposalId>;
  getProposal(id: ProposalId): Promise<Proposal | null>;
  listProposals(docId: DocumentId): Promise<Proposal[]>;
  updateProposal(proposal: Proposal): Promise<void>;

  // Sync state
  recordSyncState(state: SyncState): Promise<void>;
  getSyncState(docId: DocumentId): Promise<SyncState | null>;

  // Templates
  getTemplate(name: TemplateName, version?: string): Promise<Template | null>;
  saveTemplate(template: Template): Promise<void>;
  listTemplates(): Promise<Template[]>;

  // Atomic operations
  beginTransaction(): Promise<Transaction>;
  // (Transaction = { commit(), rollback() })

  // Errors: DuplicateDocument, DocumentNotFound, StorageUnavailable, TransactionAborted
}

interface Transaction {
  commit(): Promise<void>;
  rollback(): Promise<void>;
  // (Passed to nested operations; must not escape domain)
}
```

**Contract semantics:**
- Writes are transactional: all or nothing within one transaction
- Reads are consistent: no dirty reads
- sync_state is a singleton per document (upsert semantics)
- Proposals are immutable after creation (but status can change)
- No guarantees on ordering or indexing (adapters are free to optimize)

### 4.2 FormattingProvider

Abstracts document parsing and rendering (markdown ↔ structured model).

```typescript
interface FormattingProvider {
  // Parsing: raw text → structured model
  parse(rawContent: string): ParsedDocument;
  // Throws: ParseError if content is malformed

  // Rendering: structured model → raw text (must be deterministic)
  render(doc: ParsedDocument): string;
  // Rendering must be stable: render(parse(X)) == X (within whitespace normalization)

  // Physical file I/O
  readPhysical(path: DocPath): Promise<string>;
  // Throws: FileNotFound, FileReadError

  writePhysical(path: DocPath, content: string): Promise<void>;
  // Throws: FileWriteError, PathInvalid
  // Writes atomically or with rollback mechanism (see ApproveProposal)

  // Normalization (for deterministic hashing)
  normalize(rawContent: string): string;
  // Returns: content with normalized line endings, trailing whitespace removed
  // Used before hashing so CRLF vs LF doesn't cause false conflicts
}
```

**Contract semantics:**
- `parse()` returns structured model; errors if content is invalid
- `render()` must be deterministic and stable; render(parse(X)) ≈ X (whitespace-normalized)
- Physical I/O methods may throw; adapters translate to port errors
- Normalization is used before hashing; all adapters must normalize identically

### 4.3 ConflictDetector

Abstracts fingerprinting and comparison logic.

```typescript
interface ConflictDetector {
  // Fingerprint a string (usually SHA-256 of normalized content)
  fingerprint(content: string, normalized?: boolean): ContentHash;
  // If normalized=true, call formatter.normalize() first

  // Compare two hashes and return sync verdict
  compare(
    dbHash: ContentHash,
    fileHash: ContentHash
  ): SyncVerdict;
  // Returns: 'in_sync' | 'file_ahead' | 'db_ahead' | 'diverged'
  // (Logic: if both equal, in_sync; if only file changed, file_ahead, etc.)
}

type ContentHash = { algorithm: string; value: string };
type SyncVerdict = 'in_sync' | 'file_ahead' | 'db_ahead' | 'diverged';
```

**Contract semantics:**
- Fingerprints must be deterministic: same content → same hash
- Normalization happens inside fingerprinting (trailing whitespace, line endings)
- Comparison is pure logic; idempotent

### 4.4 TemplateRegistry

Abstracts loading and querying template definitions.

```typescript
interface TemplateRegistry {
  // Load a specific template
  getTemplate(name: TemplateName, version?: string): Promise<Template | null>;
  // If version is omitted, returns latest

  // List all available templates
  listTemplates(): Promise<Template[]>;
  // Returns summaries: { name, latest_version, description }

  // Infer template from document frontmatter
  inferTemplate(frontmatter: Frontmatter): Promise<Template | null>;
  // Frontmatter typically has a "template" key; adapters look it up
}
```

**Contract semantics:**
- Templates are immutable once published (versioning)
- getTemplate() never returns null if the template exists (throws TemplateNotFound if missing)
- inferTemplate() may return null if frontmatter has no template key (caller decides default)

---

## 5. Driven Adapters (Implementations)

Concrete implementations of the four ports. Each adapter owns its technology; all error mapping happens at the boundary.

### 5.1 SQLiteStorageProvider

**Technology:** SQLite (via better-sqlite3 in Node.js).

**Schema:**

```sql
-- Templates: immutable, versioned
CREATE TABLE templates (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  version TEXT NOT NULL,
  formatter_rules JSON NOT NULL,  -- validation rules
  agent_sop TEXT,                 -- agent guidance (markdown)
  source TEXT,                     -- where template came from (embedded/url/file)
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(name, version)
);

-- Documents: one per .md file
CREATE TABLE documents (
  id TEXT PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  title TEXT,
  template_id INTEGER NOT NULL,
  template_version TEXT NOT NULL,  -- pin to version used at validation time
  parsed_model_json TEXT NOT NULL,  -- structured document
  content_hash TEXT NOT NULL,
  validation_status TEXT,           -- 'passed' | 'failed' | null
  validation_errors_json TEXT,
  updated_at DATETIME,
  FOREIGN KEY (template_id) REFERENCES templates(id)
);

-- Proposals: drafts awaiting approval
CREATE TABLE proposals (
  id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL,
  base_hash TEXT NOT NULL,          -- hash of doc at proposal time (optimistic lock)
  draft_content TEXT NOT NULL,      -- full markdown
  validation_result_json TEXT NOT NULL,
  status TEXT DEFAULT 'pending',    -- 'pending' | 'approved' | 'rejected'
  author TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  resolved_at DATETIME,
  FOREIGN KEY (document_id) REFERENCES documents(id)
);

-- Sync state: one row per document
CREATE TABLE sync_state (
  document_id TEXT PRIMARY KEY,
  db_hash TEXT NOT NULL,
  last_file_hash TEXT NOT NULL,
  verdict TEXT DEFAULT 'in_sync',   -- 'in_sync' | 'file_ahead' | 'db_ahead' | 'diverged'
  locked BOOLEAN DEFAULT 0,
  last_checked_at DATETIME,
  FOREIGN KEY (document_id) REFERENCES documents(id)
);

CREATE INDEX idx_documents_template ON documents(template_id);
CREATE INDEX idx_proposals_document ON proposals(document_id);
CREATE INDEX idx_proposals_status ON proposals(status);
```

**Adapter responsibilities:**
- Execute all queries in the schema above
- Map domain types to rows and back (Proposal → row, row → Proposal)
- Manage transactions: `beginTransaction()` opens a handle, return it, use it for nested operations, commit/rollback atomically
- Translate SQLite errors to domain errors:
  - `UNIQUE constraint violation` → `DuplicateDocument`
  - `FOREIGN KEY violation` → `TemplateNotFound` or `DocumentNotFound`
  - `SQLITE_IOERR` → `StorageUnavailable`
- Migrate schema on startup (if needed)

**Error handling:**
- `getDocument(id)` returns null if not found; never throws
- `saveDocument(id)` throws `DuplicateDocument` if ID already exists
- All writes inside a failed transaction are rolled back automatically

### 5.2 MarkdownFormattingProvider

**Technology:** Markdown parsing and rendering.

**Parsing:**
- Read frontmatter (YAML in `---...---` fence)
- Parse heading hierarchy (H1-H6, track nesting)
- Extract links (inline `[text](url)` and reference links)
- Capture code blocks, quotes, lists (preserve structure)
- Result: `ParsedDocument` with sections, metadata, links, etc.

**Rendering:**
- Reconstruct Markdown from `ParsedDocument`
- Deterministic: always produces identical output for same input (whitespace-normalized)
- Preserves formatting conventions (consistent indentation, line breaks)

**Normalization (for hashing):**
- CRLF → LF (Windows line endings standardized)
- Trailing whitespace removed from each line
- Trailing newlines at EOF normalized to single newline
- YAML frontmatter formatted consistently

**Adapter responsibilities:**
- Use a Markdown parser (e.g., `remark` in Node.js)
- Implement `parse()` → `ParsedDocument`
- Implement `render()` → consistent string output
- Implement `normalize()` → normalized string for hashing
- Throw `ParseError` if frontmatter is invalid YAML or structure is malformed

### 5.3 HashConflictDetector

**Technology:** SHA-256 hashing (or configurable).

**Fingerprint:**
- Hash normalized content (using `FormattingProvider.normalize()`)
- Return `ContentHash { algorithm: 'sha256', value: '<hex>' }`

**Compare:**
- Compare two `ContentHash` values
- Logic:
  - `dbHash == fileHash` → 'in_sync'
  - `dbHash != null && fileHash == null` → 'file_deleted' (variant of file_ahead)
  - `dbHash == null && fileHash != null` → 'file_created' (variant of db_ahead)
  - `dbHash changed && fileHash unchanged` → 'db_ahead'
  - `dbHash unchanged && fileHash changed` → 'file_ahead'
  - `dbHash changed && fileHash changed && both != old` → 'diverged'

**Invariants:**
- Hashing is deterministic: same normalized content always yields same hash
- Comparison is pure: idempotent, no side effects

### 5.4 FileSystemTemplateRegistry (Phase 1)

**Technology:** Load templates from `.doc-engine/templates/` directory.

**Loading:**
- Scan `.doc-engine/templates/` for `*.template.yaml` files
- Parse each YAML file as a `Template` struct
- Infer template name from filename (e.g., `sop.template.yaml` → `TemplateName: 'sop'`)
- Version from YAML field `version: "1.0"`

**Inference:**
- Frontmatter in documents has a `template: <name>` field
- Registry looks up `<name>` in loaded templates

**Error handling:**
- Missing template file → `TemplateNotFound`
- Invalid YAML → `TemplateParseError`

**Future extensions:**
- `HttpTemplateRegistry` (load from remote URL)
- `PostgresTemplateRegistry` (shared across repos)

---

## 6. Data Models

Domain types with invariants—pure value objects and aggregates.

### 6.1 Document Aggregate

```typescript
interface Document {
  id: DocumentId;           // unique identifier (UUID or slug)
  path: DocPath;            // relative path in docs/ (e.g., "architecture.md")
  title: string;            // extracted from H1 or frontmatter
  template: TemplateName;   // which template validates this document
  frontmatter: Frontmatter; // parsed YAML metadata

  sections: Section[];      // parsed heading hierarchy
  links: Link[];            // outbound links (internal and external)

  contentHash: ContentHash; // current fingerprint (for sync)
  validationStatus: 'passed' | 'failed' | null;
  validationErrors: RuleFailure[]; // if failed

  updatedAt: Date;          // timestamp of last approval
}

interface Section {
  id: string;               // anchor (e.g., "section-1-2")
  level: 1 | 2 | 3 | 4 | 5 | 6; // heading level
  title: string;
  content: string;          // raw markdown between this heading and next
  subsections: Section[];
}

interface Link {
  type: 'internal' | 'external';
  text: string;
  target: string;           // URL or document ID
  location: LineRange;      // where in the file

  // Only for internal links:
  resolvedDocId?: DocumentId;
  status: 'resolved' | 'unresolved' | 'broken'; // broken = doc was deleted
}

type Frontmatter = Record<string, unknown>; // arbitrary YAML
type DocPath = string & { readonly __kind: 'DocPath' };
type DocumentId = string & { readonly __kind: 'DocumentId' };
type TemplateName = string & { readonly __kind: 'TemplateName' };
type ContentHash = { algorithm: string; value: string };
type LineRange = { start: number; end: number };
```

**Invariants:**
- `id` is globally unique
- `path` is unique and valid (no `../`, no `..` escapes)
- `sections` is a tree (parent → children)
- `validationStatus` is `null` only if validation hasn't run yet
- `validationErrors` is non-empty if status is 'failed', empty if 'passed'
- All internal links in `links` are checked at validation time

### 6.2 Template

```typescript
interface Template {
  name: TemplateName;
  version: string;          // semantic versioning (e.g., "1.0.0")
  description: string;

  // Validation rules (applied during ValidateDocument)
  rules: ValidationRule[];

  // Agent guidance
  agentSop: string;         // markdown describing how agent should edit this doc type

  // Metadata
  source: 'embedded' | 'url' | 'file'; // where it came from
  publishedAt: Date;
}

interface ValidationRule {
  id: string;               // unique within template (e.g., "required-summary")
  name: string;             // human-readable
  description: string;      // what this rule checks
  severity: 'error' | 'warning'; // error = must fix, warning = should fix

  // Rule logic: custom function or declarative spec
  check: (doc: ParsedDocument) => boolean;
  // or
  spec: {
    type: 'required-section' | 'forbidden-section' | 'max-headings' | 'link-check' | ...;
    section?: string; // heading title
    count?: number;
  };

  errorMessage: string;     // what to show if rule fails (may include {section}, {count})
}

type ParsedDocument = {
  frontmatter: Frontmatter;
  sections: Section[];
  links: Link[];
  raw: string; // original markdown
};
```

**Invariants:**
- `name` and `version` uniquely identify a template
- `rules` may be empty (template has no validation)
- `check` function is pure (no I/O, no side effects)
- `agentSop` is markdown and must be valid (parseable)

### 6.3 Proposal Aggregate

```typescript
interface Proposal {
  id: ProposalId;
  documentId: DocumentId;
  baseHash: ContentHash;      // hash of document when proposal was created (optimistic lock)
  draftContent: string;       // full markdown (what agent proposed)

  validationResult: ValidationResult; // validation run on draft
  status: 'pending' | 'approved' | 'rejected';

  author: string;             // who/what created the proposal
  createdAt: Date;
  resolvedAt: Date | null;    // when it was approved/rejected
  approvedBy?: string;        // who approved (if status == 'approved')
}

interface ValidationResult {
  passed: boolean;
  failures: RuleFailure[];    // empty if passed
  warnings: RuleFailure[];    // non-blocking issues
}

interface RuleFailure {
  ruleId: string;
  section: string;            // heading title or "frontmatter"
  message: string;            // human-readable failure description
  location: LineRange;        // where in draft
}

type ProposalId = string & { readonly __kind: 'ProposalId' };
```

**Invariants:**
- `baseHash` is captured when proposal is created (immutable thereafter)
- `validationResult` is computed once; never recomputed (see ApproveProposal for re-validation)
- `status` transitions: `pending` → `approved` or `pending` → `rejected` (not reversible)
- `resolvedAt` is set when status changes away from `pending`

### 6.4 SyncState

```typescript
interface SyncState {
  documentId: DocumentId;
  dbHash: ContentHash;        // content hash in database (document row)
  lastFileHash: ContentHash;  // content hash of physical file (last time we checked)

  verdict: 'in_sync' | 'file_ahead' | 'db_ahead' | 'diverged';
  locked: boolean;            // true if verdict == 'file_ahead' or 'diverged'
  lastCheckedAt: Date;

  // Resolution options (computed, not stored)
  resolution?: {
    options: ['rebuild' | 'sync-prefer-db' | 'manual'];
    explanation: string;
  };
}
```

**Invariants:**
- Exactly one `SyncState` per document (upsert semantics)
- `locked` is true if verdict is 'file_ahead' or 'diverged'
- `locked` is false otherwise
- `lastFileHash` is updated whenever we scan the physical file

### 6.5 Domain Errors

All exceptions that escape the domain are domain errors, not infrastructure exceptions:

```typescript
// Base
abstract class DomainError extends Error {
  constructor(public readonly code: string, message: string) {
    super(message);
  }
}

// Document errors
class DocumentNotFound extends DomainError {}
class DuplicateDocument extends DomainError {}
class DocumentLocked extends DomainError {} // sync conflict; no writes allowed

// Proposal errors
class ProposalNotFound extends DomainError {}
class BaseHashMismatch extends DomainError {} // proposal is stale
class ValidationFailed extends DomainError { failures: RuleFailure[] }

// Template errors
class TemplateNotFound extends DomainError {}
class TemplateInvalid extends DomainError {}

// Sync errors
class SyncConflictError extends DomainError {
  verdict: SyncVerdict;
  dbHash: ContentHash;
  fileHash: ContentHash;
  resolution: string[]; // ['rebuild' | 'sync-prefer-db' | 'manual']
}

// Storage errors
class StorageUnavailable extends DomainError {}
class TransactionAborted extends DomainError {}

// File I/O errors
class FileNotFound extends DomainError {}
class FileReadError extends DomainError {}
class FileWriteError extends DomainError {} // used during approval; triggers rollback
class PhysicalWriteFailed extends DomainError {} // critical; approval fails

// Parse errors
class ParseError extends DomainError {} // markdown or YAML malformed
```

Adapters catch technology-specific exceptions (SQLite errors, fs errors, parse exceptions) and re-throw as domain errors.

---

## 7. Integration Points

### 7.1 Dependency Injection & Wiring

All wiring happens in the composition root (`main.ts`). Ports are injected into use cases at construction.

```typescript
// Pseudocode (TypeScript)

function buildEngine(config: Config): MandateDocsEngine {
  // Step 1: Instantiate adapters (driven)
  const db = new SQLiteStorageProvider(config.storage.dbPath);
  const fmt = new MarkdownFormattingProvider();
  const conflict = new HashConflictDetector(fmt);
  const templates = new FileSystemTemplateRegistry(config.templatesDir);

  // Step 2: Instantiate use cases, inject ports
  const init = new InitializeEngine(db, fmt, templates);
  const get = new GetDocument(db, fmt, conflict);
  const validate = new ValidateDocument(templates);
  const propose = new ProposeChange(db, validate);
  const approve = new ApproveProposal(db, fmt, validate);
  const sync = new SyncRepository(db, fmt, conflict);

  // Step 3: Wire into an engine facade
  const engine = {
    init, get, validate, propose, approve, sync
  };

  // Step 4: Instantiate driving adapters (presentation)
  const mcp = new McpToolHandlers(engine);
  const api = new ApiRoutes(engine);
  const cli = new CliHandlers(engine);

  return { engine, mcp, api, cli };
}
```

**Key rules:**
- Domain code never imports adapters
- Adapters import ports (interfaces)
- Adapters import domain types
- Composition root imports everything
- No `new` statements inside use cases
- All external dependencies injected via constructor

### 7.2 Control Flow Example: Proposing a Change

```
User (CLI or MCP tool)
    │
    ├─ doc-propose { docId, draftContent, authorNote }
    │
    ▼
[CliHandler or McpToolHandler]
    │
    ├─ Parse input, build ProposeChangeCommand
    ├─ Catch and format exceptions
    │
    ▼
ProposeChange Use Case
    │
    ├─ Query storage: getDocument(docId)
    │   └─▶ SQLiteStorageProvider.getDocument()
    │       └─▶ SELECT * FROM documents WHERE id = ?
    │       └─▶ Map to Document domain type
    │
    ├─ Check: if document.locked, throw DocumentLockedError
    │
    ├─ Invoke ValidateDocument(draftContent)
    │   └─▶ FormattingProvider.parse(draftContent)
    │       └─▶ MarkdownFormattingProvider: extract frontmatter, sections, links
    │       └─▶ ParsedDocument
    │
    │   └─▶ For each template rule:
    │       ├─ Check required sections present
    │       ├─ Validate link targets (cross-document lookup)
    │       └─▶ ValidationResult { passed, failures }
    │
    ├─ If validation failed: return ValidationResult to caller (no DB write)
    │
    ├─ If validation passed:
    │   ├─ Compute current doc hash: ConflictDetector.fingerprint(document.raw)
    │   ├─ Create Proposal {
    │   │    documentId, baseHash: currentHash, draftContent,
    │   │    validationResult: passed, author, createdAt
    │   │  }
    │   │
    │   └─ Query storage: saveProposal(proposal)
    │       └─▶ SQLiteStorageProvider.saveProposal()
    │           └─▶ INSERT INTO proposals (...) VALUES (...)
    │           └─▶ Return ProposalId
    │
    ▼
Return ProposalId to caller
    │
    └─ Human later reviews and clicks "approve"
```

### 7.3 Data Flow: Sync Check on Every Read

```
GetDocument Use Case
    │
    ├─ Load from storage: getDocument(docId)
    │   └─▶ SQLiteStorageProvider: SELECT from documents
    │       └─▶ Document + content_hash
    │
    ├─ Read physical file: FormattingProvider.readPhysical(path)
    │   └─▶ MarkdownFormattingProvider: fs.readFile(path)
    │       └─▶ rawFileContent
    │
    ├─ Fingerprint physical file: ConflictDetector.fingerprint(rawFileContent)
    │   └─▶ HashConflictDetector: SHA-256 of normalized content
    │       └─▶ fileHash
    │
    ├─ Compare: ConflictDetector.compare(dbHash, fileHash)
    │   └─▶ Sync verdict (in_sync | file_ahead | diverged)
    │
    ├─ Update sync state:
    │   └─▶ StorageProvider.recordSyncState(SyncState { verdict, locked, ... })
    │       └─▶ SQLiteStorageProvider: UPDATE or INSERT into sync_state
    │
    ├─ Decision:
    │   ├─ If in_sync: return Document (success)
    │   └─ If file_ahead or diverged: return SyncConflictError (no content)
    │
    ▼
Return to caller
```

---

## 8. Plugin Points

The architecture is designed for extension without modifying core code. Every port is a plugin API.

### 8.1 Storage Adapters

**Built-in:** `SQLiteStorageProvider`  
**Custom examples:**
- `PostgresStorageProvider` — multi-writer database for shared orgs
- `DynamoDBStorageProvider` — serverless AWS
- `InMemoryStorageProvider` — testing and local development (no persistence)

**How to add:**
1. Implement the `StorageProvider` interface
2. Register in config:
   ```yaml
   storage:
     adapter: postgres
     options: { connectionString: "postgres://..." }
   ```
3. Composition root loads via registry

**Contract test kit:**
- Shared test suite (`contracts/storage.test.ts`) runs against all implementations
- Validates: save/load idempotency, transaction semantics, error handling

### 8.2 Formatting Adapters

**Built-in:** `MarkdownFormattingProvider`  
**Custom examples:**
- `AsciiDocFormattingProvider` — AsciiDoc format instead of Markdown
- `ReStructuredTextProvider` — Sphinx/RST format
- `HtmlFormattingProvider` — HTML with frontmatter in comments

**How to add:**
1. Implement `FormattingProvider` interface
2. Ensure deterministic rendering: `render(parse(X)) ≈ X`
3. Register in config:
   ```yaml
   formatting:
     adapter: asciidoc
   ```

**Contract test kit:**
- Verify parsing round-trips consistently
- Validate normalization produces identical hashes for whitespace variants

### 8.3 Conflict Detector Adapters

**Built-in:** `HashConflictDetector` (SHA-256)  
**Custom examples:**
- `GitBlobHashDetector` — use git's internal blob hashes (repo-aware)
- `CrcDetector` — lighter weight for non-critical docs
- `TimestampDetector` — mtime-based (risky, use with caution)

**How to add:**
1. Implement `ConflictDetector` interface
2. Ensure consistent fingerprinting
3. Register in config

### 8.4 Template Registry Adapters

**Built-in:** `FileSystemTemplateRegistry` (YAML files in repo)  
**Custom examples:**
- `HttpTemplateRegistry` — load from remote URL (shared templates)
- `PostgresTemplateRegistry` — templates in shared database
- `GitHubTemplateRegistry` — fetch from GitHub raw content

**How to add:**
1. Implement `TemplateRegistry` interface
2. Handle versioning
3. Register in config

### 8.5 Future: Notification & Approval Adapters

**Not in Phase 1, but planned:** Ports for:
- `ProposalNotifier` — notify via Slack/email/webhook when proposal is ready
- `ApprovalGateway` — custom approval logic (LDAP groups, Okta, custom)

---

## 9. Known Constraints

Decisions locked in by the architecture; expected to change as Phase 1 SOPs are written.

### 9.1 Architectural Constraints

1. **DB is source of truth for state, not content.**
   - Physical files are the readable artifact (git-friendly)
   - DB holds parsed structure, validation results, proposals, sync state
   - This split is intentional; changing it requires core redesign

2. **Human approval gate is mandatory.**
   - The only path to writing physical files is through `ApproveProposal`
   - This is enforced in the use case, not in config or access control
   - Agents cannot directly write; they must draft (propose) and await approval

3. **Sync conflicts lock documents.**
   - When divergence is detected (file_ahead or diverged), the document is locked
   - No new proposals or approvals until conflict is resolved
   - This prevents silent clobbering; explicit resolution is required

4. **Proposals are immutable once created.**
   - A proposal's base_hash is captured at creation time
   - If a conflicting change is approved, the proposal remains valid but becomes stale
   - Stale proposals are rejected at approval time (base hash mismatch)
   - This is optimistic concurrency control for the proposal model

5. **Validation is template-based.**
   - Rules are declarative (required sections, ordering, link checks, etc.)
   - Validation logic is pure; no I/O inside validators
   - Validation results are stored (immutable); re-validation happens on approval

### 9.2 Phase 1 Scope Limitations

The following are *not* implemented in Phase 1:

- **User authentication & authorization:** Approval gate will accept any caller; LDAP/SAML/Okta integration deferred
- **Audit logging:** Approvals are recorded but not queryable in a dedicated audit log
- **Workflow state machines:** Documents don't have custom statuses (e.g., "draft", "review", "published")
- **Templated content generation:** Templates are validation rules only; no content stubs or boilerplate
- **Parallel proposal handling:** Exactly one proposal per document can be pending at a time (not enforced; first-come-first-served)
- **Link target backlinks:** Know which documents link *to* a document, but not exposed in the API

### 9.3 Data Model Assumptions

- **Document IDs are slugs or UUIDs.** Implementation detail; treated as opaque strings
- **Physical file paths are relative to `docs/` directory.** Assumption may change if multi-tree support is added
- **Templates are versioned.** A document pins itself to the template version it validated against; schema migration required if templates break
- **No partial validation.** All template rules are evaluated; no "skip this rule" per-document overrides in Phase 1
- **Link validation is document-local.** Cross-repo links are external and marked as external (status: unresolved)

### 9.4 Integration Assumptions

- **MCP tool handlers are stateless.** Each call is independent; no session or multi-call workflows
- **CLI handlers are synchronous.** No background jobs; all operations block the terminal (OK for Phase 1; may need async for large repos)
- **Web UI is client-rendered.** All state mutation goes through REST API; no server-side sessions

### 9.5 Performance Constraints

- **No caching in Phase 1.** Every GetDocument call re-validates sync state (file I/O + hash computation)
  - Acceptable for teams with <100 documents and <100 GB
  - Future: add cache invalidation (git hooks, inotify)
- **Template matching is linear scan.** Inferring template from frontmatter scans all templates (OK for <100 templates)
- **No full-text search.** Content queries run as `grep`-style passes (acceptable for Phase 1)

### 9.6 Security & Compliance Constraints

- **No encryption.** DB and files are stored plaintext; suitable for private repos only
- **No audit trail.** Proposals are recorded but not immutable; history lost on DB rebuild
- **Approval gate is informational.** No cryptographic signatures or non-repudiation
- **No secrets management.** Template agent SOPs may reference credentials; no built-in secret masking

### 9.7 Extensibility Constraints (Phase 1)

- **Plugin loading is config-driven at startup.** No runtime hot-loading
- **Adapter contract versioning is semver.** Breaking changes to a port require major version bump
- **No adapter isolation.** A buggy third-party storage adapter can crash the whole system (no sandboxing)

### 9.8 Expected Refinements as SOPs Are Finalized

- **Approval workflow SOP** may define multi-stage reviews (auto-review + human sign-off)
- **Link checking SOP** may expand to external link validation (HTTP checks)
- **Conflict resolution SOP** may add heuristics (prefer file or DB based on document type)
- **Template SOP** may define versioning & migration strategy
- **Agent interaction SOP** will guide prompt engineering for proposal generation

---

## Summary

Mandate-docs is a **hexagonal, ports-driven system** that treats documentation as structured, validated records—while keeping physical Markdown files git-friendly and human-editable.

The core is isolated from infrastructure decisions:
- **Ports** define what the domain needs (storage, formatting, conflict detection)
- **Adapters** provide implementations (SQLite, Markdown, SHA-256)
- **Driving adapters** translate external requests (CLI, MCP tools, API, web UI)
- **Use cases** orchestrate domain logic with zero I/O or framework coupling

Every port is a plugin API: custom storage backends, custom formatters, custom conflict detectors, and custom notification channels can be added without modifying the core. Tests run against the domain in milliseconds; integration tests validate adapters in isolation; contract tests ensure all implementations of a port behave identically.

This architecture trades upfront structure for long-term flexibility—ideal for a system expected to evolve as Phase 1 SOPs are finalized and the product scales to large teams and organizations.
