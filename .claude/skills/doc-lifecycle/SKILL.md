---
name: doc-lifecycle
description: 'Run a document through structure, prose and accuracy phases with review and commit between each. Invoke with /doc-lifecycle <path-to-doc> [--no-diagram] [--passes N].'
disable-model-invocation: true
license: MIT
metadata:
  hermes:
    tags: [Documentation, Workflow, Review]
    category: documentation
    related_skills: []
---

# doc-lifecycle

Skill name: doc-lifecycle. Invoke with /doc-lifecycle <path>.

Runs one document through three phases: structure, prose, accuracy. The
orchestrator delegates each phase to a subagent, verifies the result, and
commits before starting the next phase. `<path-to-doc>` is the file the
lifecycle acts on for all three phases.

## Rules that apply to every phase

- One subagent per phase. Phase 3 gets a fresh subagent with no memory of
  phases 1 or 2, so it reads the document and its source material cold.
- The orchestrator verifies before committing: reads the diff, checks any
  mermaid blocks render as fenced code, greps for em dashes and for stale
  paths or references.
- Subagents never run `git checkout`, `restore`, `stash`, `reset`, or `clean`.
- Each commit touches the target document only, nothing else.
- If a phase produces no change, skip its commit and say so in the hand-off.
- The lifecycle runs to completion, including the phase 3 commit, without
  stopping for the owner unless phase 3 reports a major issue.
- Honour whatever house style the repository already declares, in a README,
  a CLAUDE.md, or a style guide. This skill does not restate one; find it
  and follow it.

## Phase 1: Structure

Subagent reorganises the document around its main flow. It adds diagrams,
in mermaid, only when the document describes a flow, a system, or a process
that a picture would make clearer, never for a procedural or checklist
document. `--no-diagram` skips diagrams outright. The subagent reads the
source material the document describes before drawing any diagram, so the
diagram matches the thing it depicts.

This phase keeps every fact and table. It does not rewrite prose and does
not check facts against the source material; that is phase 3's job.

Commit message: "Restructure <doc> around a flow diagram"

## Phase 2: Prose

Subagent applies the avoid-ai-writing skill in edit mode, technical voice,
for `--passes` rounds (default 2). Fetch before editing:

- https://raw.githubusercontent.com/conorbronsdon/avoid-ai-writing/refs/heads/main/plugins/avoid-ai-writing/skills/avoid-ai-writing/SKILL.md
- the `references/patterns.md` it points to, from the same repository

The subagent never touches code fences, tables, mermaid blocks, file paths,
identifiers, or quoted output. Source fidelity holds: no new facts, no
invented specifics, no changed claims, only how existing sentences read.

Commit message: "Rewrite <doc> prose, pass 1 and 2" (match `--passes`)

## Phase 3: Accuracy

A fresh subagent, with no context from phases 1 or 2, reads the document and
the code, data, or other source material it describes. It lists every
factual claim, each with concrete evidence (a file and line, a record, a
system state), and marks each one CORRECT, PARTIAL, WRONG, or UNVERIFIABLE.

The subagent fixes every PARTIAL and WRONG claim it can verify from the
source material, then the orchestrator commits this phase and the lifecycle
continues. It does not guess at a fix it cannot verify.

The subagent also flags any major issue: a WRONG claim whose correct answer
is a design decision, a contradiction between the document and the source
material unresolvable from that material alone, or a missing topic large
enough to change the document's shape. Only a major issue stops the
lifecycle. When one is found, the subagent reports each issue with a
suggested remedy, and the orchestrator waits for the owner's input before
continuing, rather than committing phase 3 immediately.

Anything smaller, a minor gap or an UNVERIFIABLE claim, is not a major
issue. It does not block the commit; it goes into the phase 4 hand-off.

Commit message: "Correct <doc> against the source material"

## Phase 4: Hand-off

Report to the owner:

- What changed in each phase, in plain terms.
- The three commit hashes (fewer if a phase was skipped).
- Any major issue from phase 3 that paused the lifecycle and how it was
  resolved.
- The minor gaps phase 3 found but did not block on, so the owner can
  decide whether to act on them.
