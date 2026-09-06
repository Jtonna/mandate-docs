# mandate-docs

Open `.mandate/mandate.json`. It records which mandates govern which
documentation, and which documentation governs which source files. This is the
sample file in full, not an excerpt:

```json
{
  "mandates": {
    "Mmx0HcpS": ".mandate/mandates/SOP_Orders.yaml"
  },

  "docs": {
    "1Gd7nwKu": "docs/architecture/order-lifecycle.md",
    "DgSBsJQF": "docs/architecture/persistence.md",
    "9HVTVBD5": "docs/architecture/http-api.md",
    "fd2e8Rm0": "docs/sop/onboarding.md"
  },

  "code": {
    "YOiJkyjO": "src/domain/order.ts",
    "I4qG9bvE": "src/domain/ports/order-repository.ts",
    "34NXIPF8": "src/application/place-order.ts",
    "f1wy2t8V": "src/adapters/http/order-controller.ts",
    "c48AaPgP": "src/adapters/postgres/order-repository.ts",
    "BtTe0lD6": "src/adapters/http/health-controller.ts"
  },

  "mandates_docs": {
    "Mmx0HcpS": {
      "1Gd7nwKu": ["has-owner", "sections-present", "claims-match-code"],
      "9HVTVBD5": ["claims-match-code", "endpoints-exist"]
    }
  },

  "docs_code": {
    "1Gd7nwKu": ["YOiJkyjO", "34NXIPF8", "f1wy2t8V"],
    "9HVTVBD5": ["f1wy2t8V"]
  }
}
```

Two documents and three source files are in the indexes but reachable from no
mandate. That is the normal state, and section 4 explains why.

That file is a cache. The source of truth is
`.mandate/mandates/SOP_Orders.yaml`, which declares the same linkage by path and
adds the rules that maintain each document. Section 4 covers the mandate and the
relationship between the two files.

The two files together are the entire prototype. Sections 1 to 3 are the JSON
format, section 4 the mandate, section 5 the scope, sections 6 to 8 the
reasoning behind each decision, section 9 the wider system this serves, and the
final section what was deliberately left out.

---

## 1. The five top-level keys

Three are lookup tables. Two are the mappings between them.

The chain runs in one direction: a mandate governs documents, and a document
governs source files.

```
mandates  --mandates_docs-->  docs  --docs_code-->  code
```

The three indexes are filled by scanning the repository, not by mandates. Every
mandate file, every documentation file, and every source file gets an entry
whether or not anything links to it. The two junction tables are filled from the
mandates alone. That split is why a document can sit in `docs` with no mandate
governing it, and why a source file can sit in `code` with no document covering
it.

### `mandates`

An index of every mandate file, keyed by ID. The value is the path relative to
the repository root. A mandate defines how its documentation is created and
maintained, and carries the rules that validate it.

### `docs`

An index of every documentation file, keyed by ID. Same shape.

### `code`

An index of every source file, keyed by ID. Same shape, same rules. The three
indexes are separate namespaces, so an ID is only ever ambiguous if you read it
without knowing which table it came from.

### `mandates_docs`

A junction table that also carries rule assignments. Each key is a **mandate**
ID. Each value maps a **doc** ID to the list of rule ids that mandate applies to
that document.

```
"Mmx0HcpS": { "1Gd7nwKu": ["has-owner", "sections-present", "claims-match-code"] }
     ^             ^                    ^
     |             |                    the rules that apply to it
     |             the document it governs
     the mandate
```

The rule ids are references. Their bodies live in the mandate file, never here.
Because they nest under a mandate ID, two mandates can each define a rule called
`has-owner` without colliding. Within a single mandate an id must be unique,
since `governs` references rules by id alone.

### `docs_code`

A junction table. Each key is a **doc** ID. Each value is the list of **code**
IDs that document governs.

Junction tables are named the way a lookup table is named in a relational
schema: the two tables they join, in the order they read.

```
"1Gd7nwKu": ["YOiJkyjO", "34NXIPF8", "f1wy2t8V"]
 ^                ^
 |                the code it governs
 the document
```

In words: *the order lifecycle document governs `order.ts`, `place-order.ts`
and `order-controller.ts`.*

Following the full chain from `Mmx0HcpS` reaches every source file that the
`SOP_Orders` mandate is ultimately responsible for, through the documents in
between.

The IDs are deliberately meaningless, so reading a chain means three lookups.
Resolved, the sample chain is:

```
Mmx0HcpS  .mandate/mandates/SOP_Orders.yaml
  1Gd7nwKu  docs/architecture/order-lifecycle.md
      YOiJkyjO  src/domain/order.ts
      34NXIPF8  src/application/place-order.ts
      f1wy2t8V  src/adapters/http/order-controller.ts
  9HVTVBD5  docs/architecture/http-api.md
      f1wy2t8V  src/adapters/http/order-controller.ts
```

`order-controller.ts` arrives through both documents and is subject to both rule
sets. The three source files not listed are in `code` and reachable from no
mandate.

---

## 2. Coverage is computed, not recorded

`mandate.json` holds the minimum data needed to rebuild the database, ordered
for human scanning. Anything derivable from that data is left out of the file
and computed on load.

Two things are derived rather than stored:

**The reverse mapping.** `docs_code` answers "which source files does this
document govern" directly. The opposite question is answered by inverting the
table in memory at load time, which is where fast lookups belong. Storing both
directions in the file would duplicate the relationship in two places that can
disagree.

**Coverage.** A source file with no documentation has an entry in `code` and
appears in no `docs_code` list:

```
in code, in no docs_code list  ->  scanned, uncovered
```

This needs no marker in the file, because the `code` index *is* the scan record.
A file has an ID only because a scan gave it one, so presence in `code` already
means it was scanned. Absence from every `docs_code` list is therefore
unambiguous, and "what percentage of the codebase is documented" stays a
question the file can answer.

Recomputing on every load also keeps coverage correct without a rebuild step,
which matters because it changes on every scan.

---

## 3. Folder layout

```
README.md                     this document
.mandate/
  mandate.json                the rebuild cache
  mandates/
    SOP_Orders.yaml           a mandate
docs/                         stand-in documentation tree (empty)
src/                          stand-in source tree (empty)
```

`.doc-engine/` appears in section 6 but not above, because nothing builds it
yet. It is the local database directory: gitignored, never committed, and
rebuilt from `.mandate/` on demand. Its internal layout is undefined at this
stage.

Every path inside `mandate.json` and inside a mandate is relative to the
repository root, which this folder stands in for. A path is never relative to
the file that contains it, so `mandate.json` records the mandate beside it as
`.mandate/mandates/SOP_Orders.yaml` rather than `mandates/SOP_Orders.yaml`.

The paths point at files that do not exist, and that is intentional: this stage
records relationships only, and nothing yet reads the files themselves.

The trees get populated at the point where a feature needs real file contents to
demonstrate. Content hashing is the first candidate.

---

## 4. What a mandate is

A **mandate** is a file in `.mandate/mandates/`. It links documentation to the
code it describes, and it carries the rules that keep that documentation honest.

Read `.mandate/mandates/SOP_Orders.yaml` alongside this section.

A mandate is a middleman. It sits between documents and source files, and it
adds the one thing a plain link cannot carry: how the documentation is
maintained.

This section describes rules in the present tense because that is how the format
defines them. Nothing executes a rule today. Section 5 lists what actually
exists.

### The source of truth is the mandate, not the JSON

This is the most important relationship in the system, and it runs opposite to
what the file sizes suggest.

```
.mandate/mandates/*.yaml     source of truth, hand-written
        |
        |  parse
        v
     database                what the application runs against
        |
        |  serialise
        v
.mandate/mandate.json        cache, derived, regenerable
```

The database can be rebuilt two ways:

1. **Fast.** Load `mandate.json`. It already holds the linkage in the shape the
   database wants.
2. **Slow.** Parse every file in `.mandate/mandates/`. Same result, more work.

Both paths must produce the same database. That constraint is what decides
whether a given field belongs in `mandate.json`: if the fast path would lose it,
it has to be there.

### When the two files disagree

Both are committed, so a branch can easily carry a mandate edit without a
regenerated cache. When they disagree, **the mandates win**. `mandate.json` is
never authoritative about anything, so a conflict is resolved by regenerating it
rather than by reconciling the two.

This is a likelier failure than documentation drifting from code, because it
needs no more than editing a YAML and forgetting a step. Nothing detects it yet:
there is no staleness check and no regenerate command, and a loader taking the
fast path would read the stale cache without complaint. Both are listed in the
future scope section.

### Losing either file

The two are not equally replaceable, and the asymmetry is the point.

**Losing `mandate.json` costs nothing.** Parse the mandates and write it again.
A corrupted copy, a merge conflict, or a stale one is never a loss, because the
data it holds exists in full somewhere else.

**Losing the mandates is permanent.** The linkage survives, since `mandate.json`
holds all three indexes and both junction tables. Everything that never crosses
over is gone: every rule body, every rule `type`, every `description`. The
database would know `1Gd7nwKu` requires `has-owner` and have no idea what
`has-owner` checks or how to run it, and `mandates` would point at a file that
no longer exists.

Restoring from `mandate.json` alone recovers the map and none of the behaviour.
Mandates are hand-written and cannot be regenerated from anything.

### What a mandate contains

Five fields, in this order. Four are required; `description` is optional.

- **`name`**: the mandate's identity.
- **`description`**: human-facing only. It says what the mandate is meant to
  cover, for whoever opens the file. It is never passed to an agent when a rule
  runs, so it cannot influence a result. Optional.
- **`rules`**: defined once, referenced by `id`. At least one is required. A
  rule is either `type: script`, which runs a command and passes on exit 0, or
  `type: agent`, which runs a prompt against an AI agent. The type is written
  explicitly so a reader can tell which rules are deterministic enough to gate
  CI on and which are judgements that vary between runs. A rule's own
  `description` is human-facing like the mandate's: only `prompt` reaches the
  agent, so a rule's behaviour is defined by `run` or `prompt` alone and editing
  a description can never change a result.
- **`governs`**: the documents this mandate covers, and which rules apply to
  each. A rule can apply to several documents without being written twice.
- **`code`**: the source files, and which of the governed documents each one
  links to. This is what a rule reads when it needs to check a document against
  code.

Because `governs` and `code` live in the mandate, the file is self-contained. It
can be written by hand and dropped into `.mandate/mandates/` with no other file
touched.

### What crosses into `mandate.json`, and what does not

`mandate.json` holds linkage. It never holds a rule body or a description.

| In the mandate | In `mandate.json` |
|---|---|
| `name` | the entry in `mandates` |
| `description` | nothing. Stays in the mandate. |
| `rules` bodies | nothing. Stays in the mandate. |
| rule **ids**, per document | the arrays inside `mandates_docs` |
| `governs` | `mandates_docs` |
| `code` | `docs_code` |

Rule ids cross over because `governs` would otherwise be lost on a fast rebuild.
Rule bodies do not, because running a rule means reading the mandate anyway.
Ids nest under their mandate's ID, so two mandates can each define a rule called
`has-owner` with no risk of collision.

Both mappings are transposed on the way across, which a parser has to do
deliberately. The mandate writes `code:` as source file to documents, because
that is how an author thinks about it. `docs_code` stores document to source
files, because that is the direction lookups run. `governs` transposes the same
way, from a list of documents into a map keyed by document ID. Paths become IDs
in both cases.

```
mandate                          mandate.json
  code: src/domain/order.ts        docs_code:
    docs: [order-lifecycle.md]       1Gd7nwKu: [YOiJkyjO, ...]
```

### Four rules the format defines

No tooling exists yet, so nothing checks these today. They are the contract a
parser will have to implement, not behaviour you can currently rely on.

1. **Two mandates may declare the same code-to-document link.** The edges are
   merged. Declaring an identical edge twice changes nothing. Their rules do not
   merge: each mandate keeps its own rule set for that document, and both sets
   apply. `mandates_docs` nests rule ids under the mandate that owns them
   precisely so two mandates can govern one document without interfering.
2. **A mandate may not link code to a document it does not govern.** Every path
   in `code:` must name a document present in `governs:`. Anything else is a
   validation error in the mandate.
3. **A rule defined but referenced by no document is valid.** It is dead weight,
   worth a warning, and not an error. Mandates are hand-written and a rule may
   be staged before the document it will apply to exists.
4. **A document with no mandate has no code links.** Only a mandate declares a
   document-to-code edge, so an unmandated document cannot have one. This is
   expected rather than a gap: a link arrives with the rules that maintain it,
   or it does not arrive.

### What is not yet defined about validity

The rules above constrain a mandate. Nothing yet defines what a loader does when
`mandate.json` is internally inconsistent, which is the likelier case since it is
generated and hand-editable:

- A rule id in `mandates_docs` that the mandate does not define.
- A junction table referencing an ID absent from its index.
- A `mandates` entry whose file no longer exists, which is exactly the state left
  behind when a mandate is deleted.

Reject the file, drop the offending entry, or load what parses and report the
rest: all three are defensible and none is chosen. A loader has to pick one, so
this is specified before the first loader is written.

### Partial coverage is the normal state

A new repository has no mandates at all. An established one has a few. Full
coverage from mandate to document to code is unlikely ever to be reached, and
the format assumes as much:

- Source files in `code` that no document covers. In the sample:
  `order-repository.ts` (both of them) and `health-controller.ts`.
- Documents in `docs` that no mandate governs. In the sample: `persistence.md`,
  which documents code nobody has written a mandate for, and `onboarding.md`, an
  SOP describing a process rather than any source file.
- Both are visible by inspection, neither needs a marker.

### The document format

Markdown files inside this repository. External documentation and other formats
are not supported, deliberately, to keep the first version small.

---

## 5. What this prototype covers

This is a small-step prototype. It is not the full system described in
section 9. It builds the foundation that the full system depends on: knowing
which documentation covers which code, and where the rules for maintaining it
are written down.

**In scope:**

1. **The mandate format:** A self-contained YAML file declaring the documents it
   governs, the source files those documents describe, and the rules that
   validate them.
2. **The linkage format:** `mandate.json`, a serialised copy of that linkage in
   the shape the database wants.
3. **Stable identity:** The mapping survives a file being renamed or moved,
   because it references an ID rather than a path. The stored path still goes
   wrong and needs correcting, by hand for now.
4. **Two rebuild paths:** From `mandate.json` for speed, or from the mandate
   files for correctness. Both produce the same database.

**Not in scope yet:**

- The documentation type guidelines.
- Any execution. Nothing runs a rule, script or agent. The format declares them.
- Any tooling. Nothing reads, writes, or validates these files programmatically.
- Drift detection between a document and its code.

Each of those depends on the mapping existing first. Nothing else blocks them.

---

## 6. Why the mandate is a committed file

The application runs off a database instance that exists only on the
developer's machine. It is not committed, and it is not shared. That creates
two problems:

1. **Onboarding:** A new developer clones the repo and has no database. They
   need to rebuild it deterministically, from something that *is* committed.
2. **Drift:** A developer or a coding agent edits a documentation file or a
   source file directly. The local database still holds the old relationship.
   Nothing tells anyone the two have diverged.

Everything in `.mandate/` is committed, which solves both. The local database is
derived and always disposable.

```
.mandate/mandates/*.yaml   committed, hand-written, the source of truth
.mandate/mandate.json      committed, generated, a cache of the same linkage
      |
      |  sync
      v
.doc-engine/               local only, gitignored, rebuilt on demand,
                           never authoritative
```

Note the two committed files serve different purposes. The mandates are what a
human writes and reviews. `mandate.json` is what the loader reads, and it is
regenerable from the mandates at any time, so a corrupted or conflicted copy is
never a loss.

The relationship between the committed files and `.doc-engine/` is the same one
npm has between `package-lock.json` and `node_modules/`. You commit the
description. You regenerate the artifact. If the artifact is ever wrong, you
delete it and sync again.

---

## 7. Why paths are not used as keys

The obvious design is to write paths directly into the mapping:

```json
"src/application/place-order.ts": [
  "docs/architecture/order-lifecycle.md",
  "docs/architecture/persistence.md"
]
```

This is rejected for one reason, and it is not file size.

**Renames:** With an index, moving a file changes exactly one string: its value
in `code`. Every mapping that references it stays correct, because mappings
reference the ID, not the location. Without an index, moving a file requires
rewriting every line that mentions it.

At 300 files with 2 documents each, a folder rename is the difference between a
one-line diff and a six-hundred-line diff. One is reviewable. The other gets
approved without being read, which is the same as not having review at all.

Byte savings are a secondary benefit and are not the justification.

---

## 8. Why IDs are random

IDs are 8 characters of base62, generated once when a file first enters the
index, from a cryptographic random source. They are never derived from the path
and never reused.

Four alternatives were considered and rejected:

| Scheme | Why not |
|---|---|
| **Hash of the path** | A rename produces a different ID, which destroys every mapping that referenced the file. This defeats the entire purpose of having an index. |
| **UUID v4** | Survives renames correctly, but is 36 characters, longer than most of the relative paths it is standing in for, so it saves nothing and costs readability. |
| **Sequential with tombstones** | Works, but requires retired entries to stay in the file forever so a deleted ID is never reissued. |
| **Sequential with a counter** | Works, but the counter is state that must be persisted and kept correct. If it is ever lost or regenerated from the surviving entries, a deleted ID gets reissued and stale mappings silently point at an unrelated file. |

Random IDs avoid all four failure modes without bookkeeping:

- **Rename-stable:** The ID is not a function of the path, so the path can change
  freely.
- **Short:** 8 characters against a UUID's 36.
- **Stateless:** No counter to persist, no tombstones to carry.
- **Effectively collision-free:** 62 to the 8th power is about 218 trillion. At
  1,000 files the odds of any collision are about 2 in a billion.

The cost is that a random ID does not sort or read as well as `001` when scanning
by eye. This is acceptable: the tables are separated by type, so position already
tells you what you are looking at, and `mandate.json` is meant to be generated
rather than hand-edited. No generator exists yet, which is why the sample was
written by hand.

---

## 9. What the full system is for

Documentation fails in two distinct ways, and most tooling only addresses the
first.

**It gets written badly.** Network documentation, microservice interconnectivity
documentation, and standard operating procedures each have a shape. A reader
knows what they expect to find in an SOP and in what order. When an author or an
AI agent writes one without that shape in mind, the result is prose that
technically covers the topic and is useless in the moment someone needs it.

**It stops being true.** The code changes. The document does not. Nobody notices
until someone follows the document and it fails them. By then the document has
been wrong for months, and the damage is worse than having had no document,
because it was trusted.

mandate-docs addresses both.

### Authoring

Documentation is written against guidelines for its type. A network document
follows the network document guidelines; an SOP follows the SOP guidelines.
Developers and AI agents both write against the same standard, so a reader
cannot tell which one produced a given document.

### Maintaining

A **rule engine** lets the user define rules that AI agents enforce against the
documentation. Rules are the user's to write, not the system's to dictate. The
motivating example: *verify every factual claim this document makes against the
codebase*. An agent reads the claim, reads the code, and reports where they
disagree.

Because the rules are user-defined, the engine is not limited to accuracy. A
rule can require an SOP to name an owner, require every endpoint in an API
document to exist, or require a diagram to be regenerated when its subject
changes.

### The byproduct

Once claims can be checked against the codebase, keeping documentation current
with the codebase comes from the same machinery. That capability is valuable,
but it is a consequence of the rule engine rather than the goal of the project.

---

# OUT OF SCOPE / FUTURE SCOPE CONSIDERATION

Everything below is deliberately absent from the current design. It is recorded
here so that it is a decision rather than an oversight. Nothing in this section
should be built until the functional prototype works end to end.

### ID collision guard

The generator must check a newly minted ID against existing keys in its table
and regenerate on a hit. The odds are around 2 in a billion at this scale, so it
will not trigger at this scale, but the guard is three lines and removes the need
to ever reason about it again.

### Content hashes and drift detection

Values in `docs` and `code` are currently bare path strings, so there is nowhere
to record what a file looked like at the last sync. Drift detection needs that
record. The likely shape is a hash of the normalized file contents stored
alongside the path, which turns the value from a string into an object and is
therefore a structural change, not an additive one.

Open question: whether the hash lives on the index entry, on the mapping, or
both. An index hash answers "has this file changed". A hash recorded on the
mapping at the moment a link was last confirmed answers the more useful
question: *which side moved, and when*.

### Link state and metadata

There is currently no way to express *how* a document relates to code (whether
it specifies, describes, or merely mentions it), nor whether a link has been
reviewed since either side last changed. Adding this means the mapping value
stops being a flat list of IDs and becomes a list of objects.

Candidate states, if pursued: `verified`, `doc-drift`, `code-drift`,
`dual-drift`, `broken`, `unverified`.

### Rename and move recovery

Random IDs mean a move never breaks a mapping, but the stored path does become
wrong. Git can identify the new location without any of that data living in
`mandate.json`:

```bash
git diff -M --name-status <old-ref> <new-ref>
# R096    src/domain/order.ts    src/core/order.ts
```

Git recomputes renames heuristically at diff time by fingerprinting files in
small chunks and scoring their similarity. It abandons this detection entirely
past a configurable candidate limit (`diff.renameLimit`), so very large refactors
report plain adds and deletes instead. The fallback is a human correcting one
path field.

### Splitting intent from resolved state

If concurrent branches become common, hashes and audit data should move to a
separate `mandate.lock.json`, leaving `mandate.json` as the human-and-agent
edited declaration. The reason is merge conflicts: generated data conflicts on
every parallel branch and is regenerable, so it belongs in a file you can resolve
by discarding and re-locking.

The current single-file layout keeps this cheap by confining anything
machine-written to its own subtree, so the split is a move rather than a
redesign.

### Sub-file anchors

Whole-file granularity means an unrelated typo fix in a long document marks every
mapping on it as changed. Optional anchors (a heading for a document, a symbol
or line range for code) would scope that. The cost is a parser per language and
anchors that themselves go out of date.

If pursued, add the field to the schema before it is needed. Ignoring an unused
key is free; introducing one later is a migration.

### Reverse index

Section 2 covers why the reverse direction is computed on load rather than
stored. No reverse index is planned for `mandate.json` itself.

### What the scan is

Section 1 says the three indexes are filled by scanning the repository. What
that scan is remains undefined:

- Which files count. Section 4 says documentation is Markdown in this
  repository, but by that rule this README would be indexed and it is not. So
  classification is by directory, by extension, by configuration, or by
  something else, and the sample does not say which. The boundary for source
  files is equally unstated: whether tests, generated output, vendored code, and
  build artifacts are indexed or skipped.
- What excludes a file. Whether `.gitignore` is honoured, or a separate ignore
  list exists.
- When it runs. On demand, on commit, or continuously.
- What happens to an entry whose file has disappeared.

Until this is settled, coverage percentages are only as meaningful as the
denominator, and the denominator is whatever the scan decided to index.

### The rule execution contract

A rule declares `run` or `prompt` and nothing else. How either is invoked is
undefined:

- What `run: ./scripts/mandate/check-owner.sh` is relative to. Every path in the
  format resolves from the repository root, but a leading `./` conventionally
  reads as the current directory, and no `scripts/` folder exists in the sample.
- What a rule receives. Paths, file contents, both, on stdin, as arguments, as
  environment. A `type: agent` rule needs the linked documents and source files
  in its context; the format says the linkage exists and not how it is delivered.
- What a rule returns beyond a script's exit code. An agent reports findings, and
  nothing defines their shape.

Nothing executes rules yet, so none of this blocks the current stage. It is the
first thing that must be specified when execution is built.

### Staleness detection and regeneration

Nothing checks whether `mandate.json` still matches the mandates it was
generated from, and no command regenerates it. Both are needed before a loader
can trust the fast rebuild path. The likely shape is a hash of the mandate files
recorded in `mandate.json`, which pairs naturally with the content hashing
already described above.

### Canonical ordering of `mandate.json`

Section 2 says the file is ordered for human scanning and never says what that
ordering is. Two generators would then produce different files from identical
mandates, and a regeneration would churn the diff for no reason. A serialiser
needs a stated rule: key order within the file, entry order within each table,
and whether ID arrays are sorted or follow declaration order.

### Mandate type inheritance

Every mandate is currently written per document, so two SOPs that should follow
the same standard repeat the same rules. A type file defining the rules any SOP
must satisfy, with instances declaring `type: sop` to inherit them, would fix
that. Deferred on purpose: per-document mandates are simpler to read and to
parse, and nothing about them blocks inheritance being added later.

### Mandate coverage reporting

Section 2 covers documentation coverage of code. Mandate coverage is a separate
question: how much of the documentation has a mandate governing it, and how much
of the codebase is reachable from any mandate at all. The data is already
present. No reporting is built.

### Audit trail

No record currently exists of when the file was last locked or synced, by which
tool version, or by whom. This belongs with the lock-file split rather than being
added to the current structure.

### Documentation coverage reporting

Section 2 defines how an uncovered source file is identified: present in `code`,
absent from every `docs_code` list. Turning that into a reported percentage, a
threshold, or a CI gate is not built.
