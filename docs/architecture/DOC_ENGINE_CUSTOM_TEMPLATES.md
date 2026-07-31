# Creating Custom Document Templates

Doc-engine ships with `Template_SOP` for procedural documents, but the real power comes from defining your own document types. This guide shows you how to build, test, and share a custom template.

## What Templates Are and Why You'd Create One

A template is a YAML file that defines a *document type*. It answers two questions:

1. **What shape must documents of this type have?** (required sections, ordering, formatting constraints) — enforced mechanically by the validator.
2. **What makes a document of this type *good*?** (tone, level of detail, style) — written as guidance that AI assistants follow when drafting or reviewing.

Create a template whenever you find yourself telling people "post-mortems should always include a timeline" or "ADRs need a Consequences section." Encoding that rule in a template means the engine enforces it on every document, forever — for humans and AI assistants alike.

## Anatomy of a Template YAML File

Every template has three parts:

```yaml
# 1. Metadata — identity and versioning
name: Template_Postmortem
version: 1.0.0
description: Incident post-mortem with timeline and action items

# 2. Formatter rules — the mechanical shape (validated automatically)
formatter:
  frontmatter:
    required: [title, incident_date, severity]
  sections:
    - heading: Summary
      required: true
    - heading: Timeline
      required: true
  constraints:
    max_heading_depth: 3

# 3. Agent SOP — quality guidance for AI assistants (natural language)
agent_sop: |
  Write in past tense. Be blameless: describe what systems and
  processes did, not what individuals failed to do.
```

- **Metadata** identifies the template. `version` matters: documents record which template version they were validated against, so you can evolve templates without breaking old docs.
- **Formatter rules** are hard constraints. A document that violates them fails `doc-engine validate` — no exceptions.
- **Agent SOP** is soft guidance. It never fails validation, but AI assistants read it before drafting or proposing changes, and use it to self-review.

## 1. Start with the Built-in Template_to_create_templates

Don't start from a blank file. Doc-engine includes a template *for creating templates*:

```bash
npx doc-engine new --template Template_to_create_templates --title "Incident Post-Mortems"
```

This scaffolds a new YAML file in `.doc-engine/templates/` with every field stubbed out and commented. It also means your template itself gets validated — the engine checks that your YAML is well-formed, that section rules are consistent, and that required fields are present. Templates are documents too.

## 2. Define Your Template Structure

Start with metadata. Keep the name in the `Template_` convention so it's recognizable in listings:

```yaml
name: Template_Postmortem
version: 1.0.0
description: >
  Blameless incident post-mortem. One document per incident,
  written within 5 business days of resolution.
```

Before writing any rules, sketch the sections a finished document should have. For a post-mortem, a reasonable skeleton is: Summary, Impact, Timeline, Root Cause, Action Items, Lessons Learned. Write the skeleton down first — the formatter rules simply formalize it.

## 3. Write Formatter Rules

Formatter rules define the shape every document must have. They cover three areas:

**Frontmatter** — required metadata at the top of each document:

```yaml
formatter:
  frontmatter:
    required: [title, incident_date, severity, owner]
    fields:
      severity:
        enum: [SEV1, SEV2, SEV3]
      incident_date:
        format: date
```

**Sections** — the headings a document must contain, in order:

```yaml
  sections:
    - heading: Summary
      required: true
      max_words: 150          # force a genuinely short summary
    - heading: Impact
      required: true
    - heading: Timeline
      required: true
      must_contain: table     # timelines must be tables, not prose
    - heading: Root Cause
      required: true
    - heading: Action Items
      required: true
      must_contain: task_list # "- [ ]" checkboxes, trackable
    - heading: Lessons Learned
      required: false
```

**Markdown constraints** — global formatting rules:

```yaml
  constraints:
    max_heading_depth: 3      # no #### or deeper
    ordered_sections: true    # sections must appear in the order above
    allow_html: false
    links:
      validate_internal: true # cross-doc links must resolve
```

A good rule of thumb: make a rule a formatter rule only if a machine can check it unambiguously. "Timeline must be a table" is a formatter rule. "Timeline should be detailed enough to reconstruct the incident" is not — that belongs in the agent SOP.

## 4. Write the Agent SOP

The agent SOP is the judgment layer. It's plain prose, addressed to an AI assistant that will draft or review documents of this type:

```yaml
agent_sop: |
  You are drafting or reviewing an incident post-mortem.

  Quality standards:
  - Blameless: never name individuals as causes. "The deploy
    script allowed X" — not "Alice forgot to X".
  - The Timeline must use absolute timestamps with timezone.
  - Root Cause should go at least two "whys" deep. If the root
    cause is "human error", keep digging — what allowed the
    error to have impact?
  - Every Action Item must have an owner and be independently
    completable. Reject vague items like "improve monitoring".

  When proposing changes, prefer minimal edits that preserve
  the original author's voice.
```

Write it like you'd brief a careful new teammate: concrete standards, examples of good and bad, and explicit priorities. Assistants use this both to *write* better first drafts and to *self-check* before submitting a proposal.

## 5. Test Your Template

Validate the template itself first:

```bash
npx doc-engine validate .doc-engine/templates/template_postmortem.yaml
```

Then dry-run it against a sample document. Create one deliberately good document and one deliberately broken one:

```bash
npx doc-engine new --template Template_Postmortem --title "Test Incident"
npx doc-engine validate docs/test-incident.md --explain
```

Check that:

- The good document passes cleanly
- The broken document fails with *understandable* messages (delete the Timeline section — does the error say so plainly?)
- The scaffolded document from `doc-engine new` already passes structure validation with placeholder content

If you use Claude Code, also test the agent SOP: ask Claude to draft a post-mortem for a made-up incident and see whether the output matches your standards. Vague SOP guidance produces vague documents — tighten the wording until drafts come out right.

## 6. Use Your Template

Once the template is committed, it works exactly like the built-ins:

```bash
npx doc-engine new --template Template_Postmortem --title "2026-07-28 API Outage"
```

The web UI groups documents by template, so all post-mortems appear together, and the MCP tools automatically apply your formatter rules and agent SOP to any document of this type.

When you later change the template, bump `version`. Existing documents keep validating against the version they were created under until you migrate them explicitly (`doc-engine migrate --template Template_Postmortem`), so a template change never breaks your existing docs overnight.

## Example: The Finished Post-Mortem Template

Putting it together, the complete file is under 60 lines — metadata (step 2), formatter (step 3), agent SOP (step 4) in one YAML file. Other document types follow the same pattern:

- **ADRs**: sections Context / Decision / Consequences; agent SOP demands stated alternatives.
- **Runbooks**: `must_contain: code` on the Steps section; agent SOP requires copy-pasteable commands.
- **Onboarding guides**: `max_words` caps everywhere; agent SOP forbids unexplained acronyms.

## Sharing Templates Across Teams

Templates are just YAML files in `.doc-engine/templates/`, so they travel with the repo — anyone who clones gets them automatically. To share beyond one repo:

- **Central template repo**: keep a `company-doc-templates` repository and pull templates in with `npx doc-engine templates add github.com/yourorg/company-doc-templates`. The engine records the source, so `doc-engine templates update` pulls newer versions.
- **Copy and diverge**: for teams with different needs, copying a template and renaming it (`Template_Postmortem_Platform`) is fine — templates are cheap.
- **Versioning discipline**: treat shared templates like APIs. Additive changes (a new optional section) are minor versions; new required sections are major versions, since they'll fail existing documents on migration.

A shared template is the cheapest form of documentation standards a company can have: write the rules once, and every repo, every author, and every AI assistant follows them.
