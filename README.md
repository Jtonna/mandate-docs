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
mandate_docs/           the format, and the prototype
codebase-sandboxer/     the tool for testing against real codebases
docs/architecture/      reference material
```

## mandate_docs

The current stage of work. A mandate links documentation to the code it
describes and carries the rules that keep it honest.

[`mandate_docs/README.md`](mandate_docs/README.md) is the specification: the file
format, the reasoning behind each decision, and an explicit list of what has been
deferred. Start there.

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
