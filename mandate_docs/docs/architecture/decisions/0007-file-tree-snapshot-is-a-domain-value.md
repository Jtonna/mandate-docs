# The file tree snapshot is a domain value, not a live port

Status: accepted

## Context

`validate` needs to check a mandate's `governs` and `code` paths against
the state of the file tree. Every mandate in a run could either be checked
against a freshly taken snapshot of its own, or against one snapshot shared
across the whole run.

## Decision

`FileTreeSnapshot` is a plain domain value, not a port the domain calls
live. `validate` takes a plain `&FileTreeSnapshot` rather than a port
reference, and `RunMandates` takes one snapshot at the discovered root and
shares it across every mandate validated in that run.

## Consequences

Every mandate in a run is validated against the exact same view of the file
tree, taken once, rather than against a tree that could change between
mandates. `validate` itself has no dependency on I/O or on the ports.
