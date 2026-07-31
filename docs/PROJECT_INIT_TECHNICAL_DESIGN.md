# Project Init: Technical Design for Mandate-Docs

**Status:** Draft awaiting agent review  
**Purpose:** Define the mandate-docs system architecture using hexagonal (ports & adapters) pattern

---

## Instructions for Agent

**Review the following documents in this repository:**
1. `PORTS_AND_ADAPTERS_GUIDE.md` — architectural pattern reference
2. `DOC_ENGINE_ARCHITECTURE.md` — high-level system design and data flows

**Then propose the mandate-docs project architecture by filling in the sections below:**

Use the hexagonal pattern to structure your answer:
- **Driving adapters** — what calls into the system (CLI, MCP tools, API, web UI)
- **Core domain** — pure logic and workflows
- **Ports** — interfaces owned by the domain
- **Driven adapters** — implementations (storage, formatting, conflict detection)
- **Data models** — domain types and invariants

---

## 1. System Layers

Define the four-layer structure (Domain, Ports, Adapters, Presentation).

---

## 2. Driving Adapters (Presentation)

What interfaces call into the system? (MCP tools, CLI, REST API, web UI)

---

## 3. Core Domain

What workflows (use cases) live here? What is pure logic with no I/O?

---

## 4. Ports (Domain Interfaces)

What contracts do the adapters implement? Define 3–5 key port interfaces.

---

## 5. Driven Adapters (Implementations)

What concrete implementations of those ports exist? (SQLite, Markdown formatter, hash conflict detector)

---

## 6. Data Models

Define the key domain types (Document, Template, Proposal, SyncState, etc.) and their invariants.

---

## 7. Integration Points

How do layers communicate? Show dependency flow: driving adapters → domain → ports → driven adapters.

---

## 8. Plugin Points

Where can the system be extended? (Custom storage, custom formatters, custom validators)

---

## 9. Known Constraints

What decisions are locked in? What will change as Phase 1 SOPs are written?

---

**Next step:** Once this draft is reviewed, we refine it as each Phase 1 SOP is completed, then lock the architecture before implementation begins.
