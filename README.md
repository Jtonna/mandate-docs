# mandate-docs

A system for writing documentation to a standard, and then holding it to that
standard automatically.

Documentation fails two ways. It gets written badly, because network
documentation, microservice interconnectivity documentation, and standard
operating procedures each have a shape an author may not know. And it stops
being true, because the code changes and the document does not.

mandate-docs addresses both. Documentation is authored against guidelines for
its type, and a rule engine lets the user define rules that AI agents enforce
against it. The motivating rule: verify every factual claim a document makes
against the codebase.

## Where things are

```
mandate_docs/           the format, the prototype, and reference material
codebase-sandboxer/     the tool for testing against real codebases
```

## mandate_docs

The current stage of work, and a testing ground for the real system. A mandate
links documentation to the code it describes and carries the rules that keep it
honest.

The folder now holds two things side by side. The sample mandate `SOP_Orders`
points at files that do not exist and governs documents nobody has written; it
exists to exercise the format. Beside it is the first real software: a Rust
crate that parses a mandate and validates it, with real documentation and a
real mandate governing that documentation. Nothing runs a rule yet.

[`mandate_docs/README.md`](mandate_docs/README.md) is the specification: the file
format, the reasoning behind each decision, and an explicit list of what has been
deferred. Start there.

The eventual goal is to point the system at this repository and let it maintain
its own documentation, improving the tool by using it on itself. The first step
is taken, since the tool validates the mandate that governs its own
documentation. It still needs the scan, the rule engine, and rule execution,
all recorded as out of scope in the specification.

## codebase-sandboxer

The tool used to test documentation generation and maintenance against real
codebases.

Testing this system means running it against genuine repositories, not invented
ones. Real repositories have the directory depth, file counts, and naming
inconsistency that a hand-built sample never will. They are also untrusted code:
cloning one onto the host puts its hooks, submodules, and build scripts one
careless command away from running.

[`codebase-sandboxer/`](codebase-sandboxer/) clones a repository into an
isolated container, one repository per container:

```bash
./codebase-sandboxer/run.sh https://github.com/spring-projects/spring-petclinic
```

One shared image holds git and a shell. Each repository lives in its own named
volume, mounted read-only into a container with no network, running as a
non-root user. Cloning happens in a throwaway container that is deleted as soon
as it finishes. Nothing from a cloned repository is ever executed.

Sandboxes are named for the repository they hold:

```
mandate-ext-test-repo-<owner>-<repo>
```

Several can exist at once, which is the point. Documentation rules need to be
exercised against a spread of languages, layouts, and project sizes before the
results mean anything.

[`codebase-sandboxer/README.md`](codebase-sandboxer/README.md) covers usage,
naming, the isolation each layer provides, and what it deliberately does not
protect against.
