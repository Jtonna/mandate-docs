# MandateDocs

Template-driven documentation system for GitHub repositories. Manage structured documents with AI-assisted validation and human-approval gates.

## Documentation

- **[Getting Started](docs/DOC_ENGINE_GETTING_STARTED.md)** — Fresh install, creating your first SOP, approving proposals
- **[Creating Custom Templates](docs/DOC_ENGINE_CUSTOM_TEMPLATES.md)** — Define your own document types with formatter rules and quality guidelines
- **[Architecture & Design](docs/DOC_ENGINE_ARCHITECTURE.md)** — Technical deep-dive into the system design, data flows, and extensibility model
- **[Ports & Adapters Guide](docs/PORTS_AND_ADAPTERS_GUIDE.md)** — Reference for the hexagonal architecture pattern used throughout

## Quick Start

\\\ash
npx mandate-docs init
npx mandate-docs new --template Template_SOP --title "My First SOP"
npx mandate-docs serve
\\\

## Features

- **Template-driven validation** — define document structure, required sections, and quality standards
- **Two-way sync** — Markdown files + SQLite database kept in sync with hash-based conflict detection
- **AI-assisted editing** — MCP tools for Claude Code and other AI assistants
- **Human approval gates** — all AI-proposed changes require explicit human review
- **Pluggable architecture** — swap storage backends, document formats, and conflict detection strategies

## License

MIT
