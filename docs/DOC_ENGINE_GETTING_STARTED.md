# Getting Started with Doc-Engine

Welcome! This guide walks you through your first hour with doc-engine — from a fresh install to your first approved document.

## What is Doc-Engine?

Doc-engine is an installable documentation system for GitHub repositories. It manages structured documents — SOPs, runbooks, technical guides — as plain Markdown files in your repo, backed by a local database that validates every document against a template. Templates define what shape a document must have (required sections, structure rules) and what quality standards it should meet. Because doc-engine exposes MCP tools, AI assistants like Claude Code can read your docs, propose changes, and validate them against your templates — but nothing gets written until *you* approve it.

## Prerequisites

- A GitHub repository you can commit to
- Git and a terminal (basic CLI comfort)
- Familiarity with Markdown
- Optional: Claude Code, if you want AI-assisted editing

That's it. No database setup, no server configuration.

## 1. Fresh Install

Install doc-engine into your repo:

```bash
npx doc-engine init
```

On first run, the engine:

1. Creates a `docs/` directory (if you don't have one) and a `.doc-engine/` directory for its database
2. Copies in the built-in templates, including `Template_SOP`
3. Scans your `docs/` folder, parses any existing Markdown files, and **auto-builds its database from what it finds**

You don't seed the database manually — the engine derives everything from your templates and files. If the database is ever deleted or corrupted, the same scan rebuilds it (see Troubleshooting).

Commit the result:

```bash
git add docs/ .doc-engine/
git commit -m "Install doc-engine"
```

## 2. Creating Your First SOP

Create a new document from the built-in SOP template:

```bash
npx doc-engine new --template Template_SOP --title "Deploying to Production"
```

This generates `docs/deploying-to-production.md` pre-filled with the sections `Template_SOP` requires — typically Purpose, Prerequisites, Steps, and Rollback. Open it in your editor and fill in each section.

When you're done, validate it:

```bash
npx doc-engine validate docs/deploying-to-production.md
```

Validation checks your document against the template's rules: are all required sections present? Are steps numbered? Is the frontmatter complete? Fix anything it flags, re-run, and you're done — the engine records the document in its database automatically.

## 3. Reading and Browsing Docs in the Web UI

Doc-engine ships a small local web UI:

```bash
npx doc-engine serve
```

Open `http://localhost:4400` in your browser. From here you can:

- **Browse** all documents, grouped by template type
- **Read** rendered Markdown with working cross-document links
- **See sync status** — a green check means the file on disk matches the database; a warning badge means they've drifted
- **Review pending proposals** (more on this below)

The web UI is read-and-review only by design. Editing happens in your editor or through an AI assistant.

## 4. MCP Tools for Claude Code Users

If you use Claude Code, register doc-engine as an MCP server:

```bash
claude mcp add doc-engine -- npx doc-engine mcp
```

Claude then has three tools available:

| Tool | What it does |
|---|---|
| `doc-get` | Fetches a document, checking first that the file and database agree |
| `doc-propose` | Drafts a change as a *proposal* — validated against the template, but not written to disk |
| `doc-validate` | Checks a document (or draft) against its template without changing anything |

A typical prompt: *"Read the deployment SOP and propose an update adding the new smoke-test step."* Claude will call `doc-get`, draft the edit, run `doc-validate` until the draft passes, then call `doc-propose`. Nothing touches your files yet — the change waits for your approval.

## 5. Reviewing and Approving Suggested Changes

All proposed changes — whether from an AI assistant or a teammate using `doc-engine propose` — land in a review queue. See what's pending:

```bash
npx doc-engine proposals list
```

Review one (or use the web UI's Proposals tab, which shows a side-by-side diff):

```bash
npx doc-engine proposals show 12
```

Then decide:

```bash
npx doc-engine proposals approve 12   # or: reject 12
```

On approval, the engine writes the change to **both** the Markdown file and the database in one step, keeping them in sync. Commit the updated file like any other change. Rejected proposals are kept for reference but never touch your files.

## 6. Common Workflows

**Creating a doc:** `doc-engine new` → fill in sections → `doc-engine validate` → commit.

**Editing directly:** Edit the Markdown file → `doc-engine validate` → `doc-engine sync` to update the database → commit. (The engine will also detect the edit next time it runs and prompt you to sync.)

**AI-assisted editing:** Ask Claude to propose a change → review the proposal in the web UI → approve or reject → commit.

**Team review flow:** Proposals work well with pull requests — approve locally, commit, and let your normal PR review catch anything the template can't.

The rule of thumb: *direct edits are fine, but always validate; AI edits always flow through proposals.*

## Troubleshooting

**"Sync conflict detected" when reading a doc.**
Someone edited the Markdown file outside the engine (or a git pull brought in changes) while the database held a different version. Doc-engine compares content hashes and refuses to guess. Resolve it explicitly:

```bash
npx doc-engine sync --prefer-files   # the files on disk are correct
npx doc-engine sync --prefer-db      # the database version is correct
```

In most git-centric teams, `--prefer-files` is the right answer after a pull — your repo is what your teammates reviewed.

**Database is missing, corrupted, or you cloned a repo without it.**
Rebuild from scratch — the Markdown files contain everything needed:

```bash
npx doc-engine rebuild
```

This re-scans `docs/`, re-parses every file against its template, and reconstructs the database. Nothing is lost as long as the files are intact.

**Validation keeps failing and you're not sure why.**
Run with `--explain` to see which template rule failed and where:

```bash
npx doc-engine validate docs/my-doc.md --explain
```

**A proposal won't approve.**
Approvals re-validate at approval time. If the underlying file changed since the proposal was drafted, the engine blocks the approval to prevent overwriting newer work. Re-run the proposal against the current version.

## Next Steps

The default `Template_SOP` covers procedural docs, but you can define your own document types — incident post-mortems, ADRs, onboarding guides — with their own structure and quality rules. See **Creating Custom Document Templates** to learn how.
