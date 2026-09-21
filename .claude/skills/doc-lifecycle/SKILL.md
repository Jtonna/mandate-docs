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

Runs one document through three phases: structure, prose, accuracy. The
orchestrator delegates each phase to a subagent, verifies the result, and
commits before starting the next phase. `<path-to-doc>` is the file the
lifecycle acts on for all three phases.

## Rules that apply to every phase

- One subagent per phase. Phase 3 gets a fresh subagent with no memory of
  phases 1 or 2, so it reads the document and the code cold.
- The orchestrator verifies before committing: reads the diff, checks that
  mermaid blocks render as fenced code, greps for em dashes and for paths
  that no longer exist.
- Subagents never run `git checkout`, `git restore`, `git stash`, `git
  reset`, or `git clean`.
- Each commit touches the target document only, nothing else.
- If a phase produces no change, skip its commit and say so in the hand-off.
- SOP documents (files under `docs/sop/`) skip diagrams by default and keep
  the Purpose, Scope, Owner, Steps shape untouched.

## Phase 1: Structure

Subagent reorganises the document around a clear flow. Unless `--no-diagram`
is passed or the document is an SOP, it adds mermaid diagrams: a component
flow diagram, one sequence diagram of the main path, and optionally a
diagram of test wiring. The subagent reads the code the document describes
before drawing any diagram, so the diagram matches what the code does.

This phase keeps every fact and every table. It does not rewrite prose and
does not check facts against the code; that is phase 3's job.

Commit message: "Restructure <doc> around a flow diagram"

## Phase 2: Prose

Subagent applies the avoid-ai-writing skill in edit mode, technical voice,
for `--passes` rounds (default 2). Fetch the skill and its pattern
reference before editing:

- https://raw.githubusercontent.com/conorbronsdon/avoid-ai-writing/refs/heads/main/plugins/avoid-ai-writing/skills/avoid-ai-writing/SKILL.md
- the `references/patterns.md` it points to, from the same repository

The subagent never touches code fences, tables, mermaid blocks, file paths,
identifiers, or quoted output. Source fidelity holds: no new facts, no
invented specifics, no changed claims. This phase only changes how existing
sentences are written.

Commit message: "Rewrite <doc> prose, pass 1 and 2" (adjust the pass count
in the message to match `--passes`)

## Phase 3: Accuracy

A fresh subagent, with no context from phases 1 or 2, reads the document and
the code it describes. It lists every factual claim the document makes,
each with file:line evidence, and marks each one CORRECT, PARTIAL, WRONG, or
UNVERIFIABLE.

The subagent fixes PARTIAL and WRONG claims in the document wherever the fix
is certain from the code. It does not guess at a fix it cannot verify.

It returns two lists to the orchestrator:

1. Any WRONG claim whose fix needs a decision the subagent cannot make
   alone (a design choice, not just a correction).
2. Anything important the document should cover but currently does not.

The orchestrator stops here and asks the project owner about both lists
before committing this phase.

Commit message: "Correct <doc> against the code"

## Phase 4: Hand-off

Report to the owner:

- What changed in each phase, in plain terms.
- The three commit hashes (fewer if a phase was skipped).
- The open questions from phase 3 that still need an answer.

## Optional: architecture-SOP comparison

After both an architecture document and its matching SOP have each been
through this lifecycle, run one more optional step: compare the two for
gaps. Anything the architecture document assumes the SOP does not mention,
or a step the SOP requires that the architecture document does not explain,
is worth flagging even though nothing here fixes it automatically.

## House style this skill must preserve

Every phase writes plain sentences, avoids em dashes, wraps near 80 columns,
and keeps facts sourced from the actual code. Tests never read `docs/`, and
this skill does not change that: nothing in this lifecycle writes test
fixtures or touches `tests/`.
