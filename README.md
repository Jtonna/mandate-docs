# MandateDocs

Template-driven documentation system for GitHub repositories. Manage structured documents with AI-assisted validation and human-approval gates.

## Documentation

- **[Getting Started](docs/DOC_ENGINE_GETTING_STARTED.md)** — Fresh install, creating your first SOP, approving proposals
- **[Creating Custom Templates](docs/DOC_ENGINE_CUSTOM_TEMPLATES.md)** — Define your own document types with formatter rules and quality guidelines
- **[Architecture & Design](docs/DOC_ENGINE_ARCHITECTURE.md)** — Technical deep-dive into the system design, data flows, and extensibility model
- **[Ports & Adapters Guide](docs/PORTS_AND_ADAPTERS_GUIDE.md)** — Reference for the hexagonal architecture pattern used throughout

## Development

This project uses SOPs (Standard Operating Procedures) and structured design documents as specification artifacts. Development is tracked as discrete items — each can be an SOP (specification), a Task (implementation work), or Setup (infrastructure/tooling). See docs/SOP_*.md and below for roadmap and status.

### How We Work

1. **SOP rough draft** — we discuss and align on the specification (pass/fail)
2. **Hand off to agent** — Fable 5 or Opus 4.8 reviews, refines, and documents the SOP
3. **Implementation tasks** — carved from the reviewed SOP
4. **Technical design review** — once all Phase 1 SOPs are done, an agent creates the initial design document

## Roadmap

| Item | Type | Description | Status |
|---|---|---|---|
| **SOP: Project Init** | SOP | Initialize engine: scan docs/ → parse → infer templates → validate → build DB | inProgress |
| **SOP: Storage Layer** | SOP | SQLite adapter, document persistence, sync state tracking | readyForImplementation |
| **SOP: Formatting & Validation** | SOP | Markdown parser, template validation, link checking | readyForImplementation |
| **SOP: Sync Detection** | SOP | Hash-based conflict detection, resolution strategies | readyForImplementation |
| **SOP: Domain Models** | SOP | Document, Template, Proposal, SyncState types and invariants | readyForImplementation |
| **SOP: Port Interfaces** | SOP | StorageProvider, FormattingProvider, ConflictDetector contracts | readyForImplementation |
| **Initial Technical Design** | Task | Agent reviews Phase 1 SOPs and creates design document | blocked |
| **Phase 2: Implement Project Init** | Task | Build init flow per SOP_PROJECT_INIT | blocked |
| **Phase 3: Implement Document Create** | Task | Build create & validation flow | blocked |
| **Phase 4: Implement Approval Workflow** | Task | Build proposal storage & approval flow | blocked |
| **Phase 5: Build Web UI** | Task | Dashboard, browsing, proposal review | blocked |
| **Phase 6: Plugin Ecosystem** | Setup | Custom storage, formatters, extensibility | blocked |

## Quick Start (Alpha — not yet implemented)

mandate-docs init — Initialize engine in current repo
mandate-docs new --template Template_SOP --title "My First SOP" — Create a new document
mandate-docs serve — Start local web UI

## Features (Planned)

- **Template-driven validation** — define document structure, required sections, and quality standards
- **Two-way sync** — Markdown files + SQLite database kept in sync with hash-based conflict detection
- **AI-assisted editing** — MCP tools for Claude Code and other AI assistants
- **Human approval gates** — all AI-proposed changes require explicit human review
- **Pluggable architecture** — swap storage backends, document formats, and conflict detection strategies

## License

MIT
