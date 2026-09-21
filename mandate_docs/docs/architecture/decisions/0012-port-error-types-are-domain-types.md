# Port error types are domain types

Status: accepted

## Context

The driven ports wrap external technology, a filesystem and a YAML parser,
each with its own error types. Left unchecked, either library's error type
could leak into the public API.

## Decision

Error types on the driven ports are declared as domain types, so the YAML
library's and the filesystem's internal error types never leave their
adapters. `RunError::Source(SourceError)` and `RunError::Store(StoreError)`
hold these typed port errors, so the use case's caller knows what went
wrong without losing information to a string.

## Consequences

No vendor error type or library-specific detail is part of the crate's
public API. A caller can match on the typed port error rather than parsing
a string to tell one failure from another.
