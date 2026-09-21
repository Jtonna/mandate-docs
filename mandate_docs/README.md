# mandate-docs

A project that adopts mandate carries a `.mandate/mandate.json`. It records
which mandates govern which documentation, and which documentation governs
which source files. This is what that file looks like, using this
repository's own documents and source files as the example, although this
repository does not yet carry one (see section 3):

```json
{
  "mandates": {
    "6aQ5ztd2": ".mandate/mandates/Mandate_Parser.yaml"
  },

  "docs": {
    "NoZvYf6I": "docs/architecture/mandate-parser-overview.md",
    "vX2qsQpN": "docs/sop/handling-mandates.md"
  },

  "code": {
    "AGp11cEp": "src/domain/model/mandate.rs",
    "0HYahGqe": "src/domain/model/validation.rs",
    "9O0dF9vv": "src/domain/ports/driven/file_tree_source.rs",
    "dmgg5Era": "src/adapters/driven/mandate_parser/yaml_mandate_parser.rs",
    "oYrdLikO": "src/adapters/driven/mandate_store/fs_mandate_store.rs",
    "uhWvYLkM": "src/adapters/driven/file_tree/fs_file_tree_source.rs",
    "j70aaooT": "src/adapters/driving/cli/mod.rs",
    "1heIzw3j": "src/main.rs",
    "2rXErShe": "tests/fs_adapters.rs",
    "JTBXrGc3": "tests/fixtures/doc_missing/mod.rs",
    "r7yc78R9": "tests/fixtures/support.rs"
  },

  "mandates_docs": {
    "6aQ5ztd2": {
      "NoZvYf6I": ["claims-match-code"],
      "vX2qsQpN": ["sop-shape", "commands-work", "claims-match-code"]
    }
  },

  "docs_code": {
    "NoZvYf6I": [
      "AGp11cEp", "0HYahGqe", "9O0dF9vv", "dmgg5Era", "oYrdLikO",
      "uhWvYLkM", "j70aaooT", "1heIzw3j", "2rXErShe", "JTBXrGc3",
      "r7yc78R9"
    ],
    "vX2qsQpN": ["0HYahGqe", "j70aaooT", "1heIzw3j"]
  }
}
```

In this example every indexed entry is reachable from the one mandate,
because the indexes are filled by hand and hold only what a mandate
references. Once a scan fills them instead (section 2, future scope), an
entry reachable from no mandate will be the normal state.

That file is a cache; the source of truth is the mandate beside it,
`.mandate/mandates/Mandate_Parser.yaml` in the example, which declares the
same linkage by path and adds the rules that maintain each document. Section
4 covers the mandate and the relationship between the two files.

The two files together are the entire prototype. Sections 1 to 3 are the JSON
format, section 4 the mandate, section 5 the scope, sections 6 to 8 the
reasoning behind each decision, section 9 the wider system this serves,
section 10 the workflow every change to the software follows, and the final
section what was deliberately left out.

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
"6aQ5ztd2": { "vX2qsQpN": ["sop-shape", "commands-work", "claims-match-code"] }
     ^             ^                    ^
     |             |                    the rules that apply to it
     |             the document it governs
     the mandate
```

The rule ids are references. Their bodies live in the mandate file, never here.
Because they nest under a mandate ID, two mandates can each define a rule called
`sop-shape` without colliding. Within a single mandate an id must be unique,
since `governs` references rules by id alone.

### `docs_code`

A junction table. Each key is a **doc** ID. Each value is the list of **code**
IDs that document governs.

Junction tables are named the way a lookup table is named in a relational
schema: the two tables they join, in the order they read.

```
"vX2qsQpN": ["0HYahGqe", "j70aaooT", "1heIzw3j"]
 ^                ^
 |                the code it governs
 the document
```

In words: *the handling-mandates document governs `validation.rs`,
`cli.rs` and `main.rs`.*

Following the full chain from `6aQ5ztd2` reaches every source file that the
`Mandate_Parser` mandate is ultimately responsible for, through the documents
in between.

The IDs are deliberately meaningless, so reading a chain means three lookups.
Resolved, one branch of the chain is:

```
6aQ5ztd2  .mandate/mandates/Mandate_Parser.yaml
  NoZvYf6I  docs/architecture/mandate-parser-overview.md
      AGp11cEp  src/domain/model/mandate.rs
      0HYahGqe  src/domain/model/validation.rs
      1heIzw3j  src/main.rs
  vX2qsQpN  docs/sop/handling-mandates.md
      0HYahGqe  src/domain/model/validation.rs
      1heIzw3j  src/main.rs
```

`validation.rs` and `main.rs` arrive through both documents and are subject
to both rule sets.

---

## 2. Coverage is computed, not recorded

`mandate.json` holds the minimum data needed to rebuild the database, ordered
for human scanning. Anything derivable from that data is left out of the
file and computed on load.

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

A `.mandate/` folder appears only in a project where mandate is installed.
This repository is the program and does not yet run mandate on itself, so it
has none:

```
README.md                     this document
Cargo.toml                    the Rust crate; unit tests in src/,
                              integration tests in tests/
docs/
  architecture/
    PORTS_AND_ADAPTERS_GUIDE.md   reference material, not indexed
    mandate-parser-overview.md    the parser, validator and run
                                  command: purpose and big picture
    run-command.md             how a run flows, end to end
    ports-and-adapters.md      the crate's ports, adapters and seam
    testing.md                 how the test suite is wired
    decisions/                 architecture decision records
  reference/
    mandate-format-checks.md  what parsing and validation reject
  sop/
    handling-mandates.md      how a mandate is written and validated
src/                          domain, ports, adapters, composition root
  domain/                     business logic: models, ports, invariants
    model/
      mandate.rs              Mandate, Rule, RuleKind, GovernedDoc, CodeLink
      file_tree.rs            FileTreeSnapshot, EntryKind
      validation.rs           validation logic, errors, warnings
      run_report.rs           RunReportMandatesValidation, MandateOutcome
    usecases/
      run_mandates.rs         RunMandates use case
    ports/
      driven/
        file_tree_source.rs   FileTreeSource port
        mandate_store.rs      MandateStore port
        mandate_parser.rs     MandateParser port
      driving/                reserved for future driving ports
  adapters/                   external integration
    driving/
      cli/
        mod.rs                command-line interface
    driven/
      file_system/
        mod.rs                FileSystem trait (adapter-internal seam)
        os_file_system.rs     OsFileSystem implementation
      file_tree/
        fs_file_tree_source.rs  FileTreeSource implementation
      mandate_store/
        fs_mandate_store.rs   MandateStore implementation
      mandate_parser/
        yaml_mandate_parser.rs  MandateParser implementation
  main.rs                     composition root
  lib.rs                      module declarations
tests/                        integration tests
  fs_adapters.rs              FsFileTreeSource and FsMandateStore over
                              OsFileSystem, run against real temporary
                              directories
  fixtures/                   one test target: main.rs declares
                              support and every case
    support.rs                the fake virtual machine and assertion
                              helpers
    <case>/                   one fixture case: mod.rs and
                              mandate.yaml, including the run_*
                              cases that exercise RunMandates
```

Nothing under `docs/` is sample data, and no test reads it. There is no
`.mandate/` here yet; test data lives under `tests/`. Section 10 is
the workflow every change to `src/` follows.

`.doc-engine/` appears in section 6 but not above, because nothing builds it
yet. It is the local database directory: gitignored, never committed, and
rebuilt from `.mandate/` on demand. Its internal layout is undefined at this
stage.

Every path inside `mandate.json` and inside a mandate is relative to the
repository root, which this folder stands in for. A path is never relative to
the file that contains it, so `mandate.json` records the mandate beside it as
`.mandate/mandates/Mandate_Parser.yaml` rather than
`mandates/Mandate_Parser.yaml`.

`docs/architecture/PORTS_AND_ADAPTERS_GUIDE.md` is reference material about a
pattern, not documentation of this system, and stays out of the `docs` index.
Every other document under `docs/` is indexed.

---

## 4. What a mandate is

A **mandate** is a file in `.mandate/mandates/`. It links documentation to the
code it describes, and it carries the rules that keep that documentation honest.

Read the example mandate shown in the intro, `Mandate_Parser.yaml`, alongside
this section; the fixture cases under `tests/` use a separate mandate for a
fictional todo app instead, described in section 10 step 3.

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
database would know `vX2qsQpN` requires `sop-shape` and have no idea what
`sop-shape` checks or how to run it, and `mandates` would point at a file that
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
`sop-shape` with no risk of collision.

Both mappings are transposed on the way across, which a parser has to do
deliberately. The mandate writes `code:` as source file to documents, because
that is how an author thinks about it. `docs_code` stores document to source
files, because that is the direction lookups run. `governs` transposes the same
way, from a list of documents into a map keyed by document ID. Paths become IDs
in both cases.

```
mandate                            mandate.json
  code: src/domain/model/validation.rs   docs_code:
    docs: [mandate-parser-overview.md]  NoZvYf6I: [0HYahGqe, ...]
```

### Four rules the format defines

The validator in `src/` checks rule 2 as an error and rule 3 as a warning.
Rules 1 and 4 concern more than one mandate, or the absence of one, and
nothing checks them yet.

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

The rules above constrain a mandate. Nothing yet defines what a loader does
when `mandate.json` is internally inconsistent, which is the likelier case
since it is generated and hand-editable:

- A rule id in `mandates_docs` that the mandate does not define.
- A junction table referencing an ID absent from its index.
- A `mandates` entry whose file no longer exists, which is exactly the state
  left behind when a mandate is deleted.

Reject the file, drop the offending entry, or load what parses and report the
rest: all three are defensible and none is chosen. A loader has to pick one, so
this is specified before the first loader is written.

### Partial coverage is the normal state

A new repository has no mandates at all. An established one has a few. Full
coverage from mandate to document to code is unlikely ever to be reached, and
the format assumes as much. Today every indexed entry is governed, because
the index is filled by hand: nothing is added to `docs` or `code` unless a
mandate already references it. Once a scan fills the indexes instead
(section 2, future scope), two kinds of uncovered entry become possible:

- A source file the scan finds that no document's `code` list names. It sits
  in `code` and appears in no `docs_code` list.
- A document the scan finds that no mandate governs. It sits in `docs` and
  appears in no `mandates_docs` list.

Both are visible by inspection, neither needs a marker.

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
5. **Mandate validation and the run command:** the `mandate` command finds a
   project's `.mandate` folder, parses and checks the mandates it selects
   against the format and against one shared snapshot of the file tree,
   and reports every problem found in one pass. See
   `docs/architecture/mandate-parser-overview.md`. This is the first piece
   of real software; it will validate this repository's own mandates once
   mandate is installed here.

**Not in scope yet:**

- The documentation type guidelines.
- Any execution. Nothing runs a rule, script or agent. The format declares them.
- Most tooling. Nothing reads or writes `mandate.json`, nothing scans the
  repository, and nothing regenerates the cache from the mandates.
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
"src/domain/model/validation.rs": [
  "docs/architecture/mandate-parser-overview.md",
  "docs/sop/handling-mandates.md"
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

- **Rename-stable:** The ID is not a function of the path, so the path can
  change freely.
- **Short:** 8 characters against a UUID's 36.
- **Stateless:** No counter to persist, no tombstones to carry.
- **Effectively collision-free:** 62 to the 8th power is about 218 trillion.
  At 1,000 files the odds of any collision are about 2 in a billion.

The cost is that a random ID does not sort or read as well as `001` when
scanning by eye. This is acceptable: the tables are separated by type, so
position already tells you what you are looking at, and `mandate.json` is
meant to be generated rather than hand-edited. The IDs in the example
`mandate.json` above were minted by hand from a random source, since no
generator exists yet.

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

## 10. Development workflow

Every change to the software in `src/` goes through the steps below, in order.
No step is skipped because a change looks small. The workflow exists so that
code, tests, documentation and mandates move together, which is the problem
this project addresses, applied to itself.

Three roles take part. The orchestrator reads, decides and reviews, and never
writes code. An implementer writes code and tests under a brief from the
orchestrator. A reviewer is a fresh agent that has not seen the
implementation. The same agent never implements and reviews one change. The
roles are described by what they do, not which model fills them, so the
assignment can change without changing the workflow.

### Step 1. Understand the problem and the ideal solution

Two separate statements, written before anything else, and neither mentions
code.

- The problem: what is wrong or missing, who it affects, and how you would
  know it was solved.
- The ideal solution: what the world looks like when the problem is gone,
  ignoring cost and effort.

They are kept apart because what gets built is usually a compromise on the
ideal, and the compromise should be visible rather than hidden inside the
problem statement.

The brief also lists every open decision the change will force. Section 4 and
the future scope section record several the format leaves undefined (what a
loader does with an inconsistent `mandate.json`, the ordering of a serialised
file, what the scan indexes). Each one the change touches is settled with the
user before step 2, and the answer is recorded in the brief.

Output: the brief.

### Step 2. Plan the solution in code

Map the ideal solution onto the architecture in
`docs/architecture/PORTS_AND_ADAPTERS_GUIDE.md`. The plan names:

- the domain types and the invariants they enforce;
- the ports, in domain vocabulary, with what each one needs and promises;
- the adapters, one per technology, and the in-memory fake for each driven
  port;
- the wiring in the composition root;
- the files to create or change;
- the tests, per layer: domain unit tests with fakes, a contract suite per
  port, integration tests per driven adapter, translation tests per driving
  adapter.

Patterns already present in `src/` win over new ones. A port for something
that will never be swapped and never needs faking is dropped from the plan
(guide, section 8, pitfall 3).

Output: the plan. The orchestrator approves it before any code is written.

### Step 3. Test-driven implementation

Red, green, refactor, one behaviour at a time:

1. Write one failing test that states a behaviour from the plan.
2. Run it and watch it fail for the expected reason.
3. Write the least code that makes it pass.
4. Run the whole suite.
5. Refactor with the suite green.

Domain tests use in-memory fakes and do no I/O. Every port has a contract
suite that runs against every implementation, the fake included. No
production code is written without a failing test first; a test written after
the code proves only that the code does what it does.

Tests are split the way Rust and Cargo split them, and the split is
structural, not a naming habit. A unit test lives in the same file as the
code, under `#[cfg(test)]`, compiles as part of the crate, and may reach
private items. A domain test module imports nothing from an adapter: if it
needs a port implementation it defines a small fake of its own. An
integration test lives in `tests/`, compiles as a separate crate, and can use
only the public API, so if it compiles the behaviour is reachable from
outside. Anything that touches disk, runs the binary, or crosses a layer
boundary is an integration test. Files in `tests/` are named by intent,
because Rust calls everything there an integration test while the
architecture guide uses that word only for a driven adapter hitting real
technology:

| File in `tests/` | Intent |
|---|---|
| `fs_adapters.rs` | `FsFileTreeSource` and `FsMandateStore` over `OsFileSystem`, and `OsFileSystem` directly, against real temporary directories: the driven adapters that touch disk. |
| `fixtures/<case>/mod.rs` | One fixture case inside the single `fixtures` target; the directory names the check and each test's name says whether it passes or fails. |

The command line is a driving adapter in `src/adapters/driving/cli/mod.rs`. Its
`run` method executes the use case and renders the report, with its own in-file
unit tests. No test runs the built binary, because the binary will gain startup
side effects; `main.rs` wires the adapters and maps the report's `is_valid()` to
the exit code.

Tests never read `docs/`, and this repository has no `.mandate/`. A test
that needs a mandate or a file tree gets it from its own case directory
under `tests/fixtures/`. Each fixture case is a directory named for the
check it proves, holding `mod.rs` and `mandate.yaml`; the test builds a
fake virtual machine from the mandate with the support module, edits
files or the parsed mandate, and asserts the exact report, so a case can
prove several things and each test's name says whether it passes or
fails. Every fixture `mandate.yaml` describes a fictional todo app that
does not exist and needs no path in it to exist on disk, so a case is
never confused for real documentation of this repository. A case is
registered with one `mod` line in `tests/fixtures/main.rs`, the same way
`src/` declares modules.

Output: the diff and a green suite. The test command and its output are kept
for step 7.

### Step 4. Independent review

A reviewer that has seen none of steps 1 to 3 receives the brief, the plan,
the diff, the test output, and the architecture guide. It checks four things:

- the code does what the plan says, no more and no less;
- the guide is followed: dependencies point inward, no adapter logic in the
  domain, no library type crosses a port;
- the tests exercise behaviour rather than implementation;
- anything the plan promised is missing.

Output: findings, each with a severity and a proposed change. The reviewer
proposes and never edits.

### Step 5. Evaluate and apply

The orchestrator reads every finding and marks it warranted, not warranted, or
deferred, with a reason for each. Warranted changes go back through step 3,
test first. The suite must be green again before moving on. If the changes
were large, step 4 runs again on the result.

Output: the decision on each finding, and the updated diff.

### Step 6. Documentation and mandates

This is the hardest step and is managed by hand until tooling exists. Two
questions, in order.

**What kind of change is this?**

| Kind | Meaning |
|---|---|
| New feature | A capability the code did not have. |
| Existing feature | A change to the behaviour of something already there. |
| New SOP | A process people follow that did not exist. |
| Existing SOP | A change to a process people already follow. |

**What does the documentation need?** The distinction between the two kinds
of file must hold:

- A **document** in `docs/` describes the system: how it is designed, why it
  is designed that way, what it does, how it is operated. It is about the
  code or the process.
- A **mandate** in `.mandate/mandates/` describes how a document is
  maintained: which code it governs and which rules keep it true. It is
  about the documentation and never about the code. Nothing that explains how
  the system works belongs in a mandate.

| Kind | In `docs/` | In `.mandate/mandates/` |
|---|---|---|
| New feature | Write a document of the right type, or a section in an existing document if that is where a reader would look. | Write a mandate governing the document, or add the document and its files to an existing mandate's `governs` and `code`. |
| Existing feature | Update every document whose claims the change affects. | Confirm the mandate still links the right files. Add any new file to `code`. |
| New SOP | Write it as an SOP. | Write a mandate. SOP rules are about shape, such as an owner or required sections, not about code. |
| Existing SOP | Update it. | Confirm the rules still fit. |

The mandate half of this step applies to a project where mandate is
installed. This repository does not yet run mandate on itself, so until it
does, step 6 here produces documents only; the mandate and `mandate.json`
entries are written when mandate is installed in this repository.

Technical documentation comes in types, and each has a shape a reader expects.
The guidelines per type are not written yet (section 5). The types this
project recognises:

| Type | What it holds | Where |
|---|---|---|
| Architecture document | How a component is built and why. | `docs/architecture/` |
| Decision record | One decision, the options considered, why one was chosen. | `docs/decisions/` |
| API reference | The surface a caller uses: commands, endpoints, formats, errors. | `docs/api/` |
| Network documentation | Hosts, addresses, routes, firewall rules. | `docs/network/` |
| Service interconnectivity | Which services talk to which, over what, under what contract. | `docs/services/` |
| Standard operating procedure | A process a person follows, written as ordered steps. | `docs/sop/` |
| Runbook | What to do when one specific thing goes wrong. | `docs/runbooks/` |
| Onboarding | How a new person becomes productive. | `docs/sop/` |

Only `docs/architecture/` and `docs/sop/` exist today. The others are
created when the first document of that type is written.

In an adopting project, until a generator exists, `mandate.json` is updated
by hand in the same step: every new document and source file gets an ID
minted under the rules in section 8, and the junction tables get their
entries. The scan does not exist either, so the `code` index holds the files
a change touched rather than every file in the tree, and coverage figures
mean nothing until it does.

This section's own claims are subject to this step. A change that populates
`src/` makes section 3 wrong, and section 3 is corrected in the same change.

Output: the document changes, the mandate changes, and the `mandate.json`
changes.

### Step 7. Hand-off, approval, commit

The user is notified with the brief, the plan, what was built, the test
output, the review findings with the decision on each, and the documentation
and mandate changes. Nothing is committed before the user approves.

If the user requests changes, code changes go back to step 3, and steps 4 to
6 run again on the result. Documentation and mandates are reviewed again
every time, because a code change can invalidate a claim that step 6 wrote
down.

On approval, the change is committed as one commit holding code, tests,
documents and mandates together, so the repository never holds code whose
documentation arrived in a different commit.

---

# OUT OF SCOPE / FUTURE SCOPE CONSIDERATION

Everything below is deliberately absent from the current design. It is recorded
here so that it is a decision rather than an oversight. Nothing in this section
should be built until the functional prototype works end to end.

### ID collision guard

The generator must check a newly minted ID against existing keys in its
table and regenerate on a hit. The odds are around 2 in a billion at this
scale, so it will not trigger at this scale, but the guard is three lines
and removes the need to ever reason about it again.

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

Random IDs mean a move never breaks a mapping, but the stored path does
become wrong. Git can identify the new location without any of that data
living in `mandate.json`:

```bash
git diff -M --name-status <old-ref> <new-ref>
# R096    src/domain/model/mandate.rs    src/core/mandate.rs
```

Git recomputes renames heuristically at diff time by fingerprinting files in
small chunks and scoring their similarity. It abandons this detection
entirely past a configurable candidate limit (`diff.renameLimit`), so very
large refactors report plain adds and deletes instead. The fallback is a
human correcting one path field.

### Splitting intent from resolved state

If concurrent branches become common, hashes and audit data should move to a
separate `mandate.lock.json`, leaving `mandate.json` as the human-and-agent
edited declaration. The reason is merge conflicts: generated data conflicts
on every parallel branch and is regenerable, so it belongs in a file you can
resolve by discarding and re-locking.

The current single-file layout keeps this cheap by confining anything
machine-written to its own subtree, so the split is a move rather than a
redesign.

### Sub-file anchors

Whole-file granularity means an unrelated typo fix in a long document marks
every mapping on it as changed. Optional anchors (a heading for a document,
a symbol or line range for code) would scope that. The cost is a parser per
language and anchors that themselves go out of date.

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
  something else, and nothing here says which. The boundary for source
  files is equally unstated: whether tests, generated output, vendored code, and
  build artifacts are indexed or skipped.
- What excludes a file. Whether `.gitignore` is honoured, or a separate ignore
  list exists.
- When it runs. On demand, on commit, or continuously.
- What happens to an entry whose file has disappeared.

Until this is settled, coverage percentages are only as meaningful as the
denominator, and the denominator is whatever the scan decided to index.

The run command's `FileTreeSnapshot`, described in
`docs/architecture/mandate-parser-overview.md`, is a first scan in this sense: it
indexes nothing yet, since it feeds validation only, and it skips
nothing, recording every file and directory under the root with no
filtering at all. Whether a future scan that fills `mandate.json`'s
indexes should skip the same nothing, or apply the exclusions this section
still leaves open, remains undecided.

### The rule execution contract

A rule declares `run` or `prompt` and nothing else. How either is invoked
is undefined:

- What `run: ./scripts/mandate/check-owner.sh` is relative to. Every path
  in the format resolves from the repository root, but a leading `./`
  conventionally reads as the current directory, and no `scripts/` folder
  exists in this repository.
- What a rule receives. Paths, file contents, both, on stdin, as arguments,
  as environment. A `type: agent` rule needs the linked documents and
  source files in its context; the format says the linkage exists and not
  how it is delivered.
- What a rule returns beyond a script's exit code. An agent reports
  findings, and nothing defines their shape.

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

No record currently exists of when the file was last locked or synced, by
which tool version, or by whom. This belongs with the lock-file split
rather than being added to the current structure.

### Documentation coverage reporting

Section 2 defines how an uncovered source file is identified: present in `code`,
absent from every `docs_code` list. Turning that into a reported percentage, a
threshold, or a CI gate is not built.
