# MandateDocs

Template-driven documentation system for GitHub repositories. Manage structured documents with AI-assisted validation and human-approval gates.

## Documentation

See **[docs/index.md](docs/index.md)** for the complete documentation index.

Key references:
- **[Ports & Adapters Guide](docs/architecture/PORTS_AND_ADAPTERS_GUIDE.md)** — Hexagonal architecture pattern reference
- **[Custom Templates](docs/architecture/DOC_ENGINE_CUSTOM_TEMPLATES.md)** — Creating document types with formatter rules and quality standards

## Development

This project is built using SOPs (Standard Operating Procedures) as specification artifacts. Development follows three phases: SOP specification, technical design, then implementation and testing.

See **[ROADMAP.md](docs/ROADMAP.md)** for current status and milestones.

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
