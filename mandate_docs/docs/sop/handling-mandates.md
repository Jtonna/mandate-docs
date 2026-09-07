# Handling mandate files

## Purpose

A mandate links documentation to the code it describes and carries the rules
that keep that documentation honest. Writing one by hand, or changing one,
touches two files that must stay in step: the mandate itself and
`.mandate/mandate.json`. This procedure is the sequence that keeps them
consistent and keeps the mandate parseable and valid.

## Scope

Covers writing a new mandate file in `.mandate/mandates/`, changing an
existing one, and updating `.mandate/mandate.json` to match. It does not
cover writing the documents a mandate governs, which follow the guidelines
for their document type. It does not cover running a rule: nothing runs
rules yet, per `README.md` section 5.

## Owner

Jacob Tonna

## Steps

1. Decide which documents the mandate governs, and which source files each
   document describes. Write down every path exactly as it will appear in
   the mandate: relative to the repository root, with forward slashes, never
   relative to the mandate file itself.

2. Create `.mandate/mandates/<Name>.yaml` with the five fields `name`,
   `description` (optional), `rules`, `governs` and `code`, following the
   shape of `.mandate/mandates/Mandate_Parser.yaml`. Every rule is either `type:
   script` with a `run` command, or `type: agent` with a `prompt`. Every path
   listed under a `code` entry's `docs` must name a document present in
   `governs`.

3. Validate the mandate. From `mandate_docs/`, run:

   ```
   cargo run -- validate .mandate/mandates/<Name>.yaml --root .
   ```

   Exit code `0` with the line `mandate '<name>' is valid: ...` means this
   step is done.

4. Fix every reported error before moving on.

   | Message | What to do |
   |---|---|
   | `no rules defined; a mandate needs at least one` | Add at least one entry under `rules`. |
   | `duplicate rule id '<id>'` | Rename one of the two rules sharing that `id`. |
   | `rule '<rule>' is referenced by <doc> but not defined` | Either add a `rules` entry with that `id`, or remove the reference from that document's `rules` list under `governs`. |
   | `code <path> links <doc> which this mandate does not govern` | Add `<doc>` to `governs`, or remove it from that `code` entry's `docs` list. |
   | `governed document not found: <doc>` | Create the document at that path, or correct the path if it was mistyped. |
   | `missing source file: <path>` | Create the file, or correct the path if it was mistyped. |
   | `failed to parse '<path>': ...` | A shape problem: an unknown field, a missing required field, or a rule missing its `run` (for `type: script`) or its `prompt` (for `type: agent`). The message names the offending field or rule id; fix that field and re-run. |

5. Decide about each warning, of the form `rule '<id>' is defined but no
   document references it`. Either reference the rule from a document's
   `rules` list under `governs`, or remove the rule. Leaving it unreferenced
   is allowed; it is a warning, not an error.

6. Update `.mandate/mandate.json` by hand. No generator exists yet. For the
   new mandate, and for each new document and each new source file it
   introduces, mint an 8-character base62 ID from a random source and check
   it is not already a key in the relevant table (`mandates`, `docs`, or
   `code`). Add the mandate to `mandates`, each new document to `docs`, each
   new source file to `code`; add the rule ids that apply to each document
   under `mandates_docs`; add each document's linked source files under
   `docs_code`. `README.md` section 1 shows the shape of all five keys.

7. Include the mandate, the documents it governs, and the
   `.mandate/mandate.json` update in the same change as the code they
   describe, per `README.md` section 10 step 7.

## When this SOP changes

This procedure covers what exists today: hand-written mandates and a
hand-updated `mandate.json`. It will grow as mandate processing gains more
features, such as a `mandate.json` generator, a repository scan, and rule
execution. Each such feature updates this document in its own step 6, the
documentation-and-mandates step of `README.md` section 10.
