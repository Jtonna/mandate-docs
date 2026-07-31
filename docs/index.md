# MandateDocs Documentation Index

Master reference for all documentation files. Use this to navigate the knowledge base.

---

## Architecture

System design, patterns, and technical reference.

| File | Purpose |
|------|---------|
| **[PORTS_AND_ADAPTERS_GUIDE.md](architecture/PORTS_AND_ADAPTERS_GUIDE.md)** | Comprehensive reference for hexagonal architecture pattern — principles, anatomy, port design, adapter isolation, dependency flow, module structure, pitfalls, best practices, extensibility. |
| **[DOC_ENGINE_ARCHITECTURE.md](architecture/DOC_ENGINE_ARCHITECTURE.md)** | High-level system overview with ASCII diagrams. Data flows for initialization, query, change, sync detection. Two-way sync model with file/DB recovery strategies. MCP tool mapping, database schema, error handling. |
| **[PROJECT_INIT_TECHNICAL_DESIGN.md](architecture/PROJECT_INIT_TECHNICAL_DESIGN.md)** | Complete technical design for mandate-docs using hexagonal pattern. Defines layers, driving adapters, core domain, ports, driven adapters, data models, integration points, plugin points, locked constraints. |
| **[DOC_ENGINE_CUSTOM_TEMPLATES.md](architecture/DOC_ENGINE_CUSTOM_TEMPLATES.md)** | Guide for creating custom document templates with YAML structure. Shows anatomy (metadata, formatter rules, agent SOP), testing strategy, and sharing templates across teams. |

---

## Specifications (SOPs)

Standard Operating Procedures defining how the system works. Development SOPs include implementation status, work notes, and detailed specifications.

| File | Purpose | Status |
|------|---------|--------|
| **[SOP_TEMPLATE.md](sop/SOP_TEMPLATE.md)** | Blueprint for all development SOPs. Structure: header, implementation status table, work notes, actual SOP sections (Overview, Requirements, Scope, Architecture, Flow, Error Handling, Data Structures, Integration Points, Testing, Known Unknowns). | Template |
| **[SOP_PROJECT_INIT.md](sop/SOP_PROJECT_INIT.md)** | Specification for the Project Init flow (mandate-docs initialization: scan → parse → infer templates → validate → build DB). | inProgress |

---

## Templates

Document type templates (YAML files) defining structure and quality standards for different document types.

*(None yet — will be added as templates are created.)*

---

## How to Use This Index

- **Starting a new document type?** → Read **DOC_ENGINE_CUSTOM_TEMPLATES.md** for the pattern
- **Understanding the system architecture?** → Start with **PORTS_AND_ADAPTERS_GUIDE.md**, then read **DOC_ENGINE_ARCHITECTURE.md** and **PROJECT_INIT_TECHNICAL_DESIGN.md**
- **Implementing a feature?** → Find the relevant SOP in the **Specifications** section
- **Extending mandate-docs?** → Read **PROJECT_INIT_TECHNICAL_DESIGN.md** sections on plugin points and custom adapters

---

**Last updated:** 2026-07-31  
**Branch:** master
