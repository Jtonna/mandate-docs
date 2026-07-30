# MandateDocs

Template-driven documentation system for GitHub repositories. Manage structured documents with AI-assisted validation and human-approval gates.

## Documentation

- **[Getting Started](docs/DOC_ENGINE_GETTING_STARTED.md)** — Fresh install, creating your first SOP, approving proposals
- **[Creating Custom Templates](docs/DOC_ENGINE_CUSTOM_TEMPLATES.md)** — Define your own document types with formatter rules and quality guidelines
- **[Architecture & Design](docs/DOC_ENGINE_ARCHITECTURE.md)** — Technical deep-dive into the system design, data flows, and extensibility model
- **[Ports & Adapters Guide](docs/PORTS_AND_ADAPTERS_GUIDE.md)** — Reference for the hexagonal architecture pattern used throughout

## Development

This project uses SOPs (Standard Operating Procedures) as specification artifacts. See docs/SOP_*.md for development roadmap and implementation status.

## Roadmap

| Phase | Focus | Deliverable | Status |
|---|---|---|---|
| **Phase 1** | System specification | Complete dev SOPs defining engine architecture, storage, formatting, sync | inProgress |
| **Phase 2** | Project init SOP | Implement mandate-docs init flow (scan → parse → build DB) | eadyForImplementation |
| **Phase 3** | Document create SOP | Implement mandate-docs new and validation | locked |
| **Phase 4** | Approval workflow SOP | Implement proposal storage, approval flow, file sync | locked |
| **Phase 5** | Web UI SOP | Dashboard, doc browsing, proposal review interface | locked |
| **Phase 6** | Plugin ecosystem | Custom storage drivers, formatters, extensibility | locked |

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
