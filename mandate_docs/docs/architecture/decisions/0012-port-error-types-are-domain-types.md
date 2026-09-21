# Port error types are domain types, and the dependency rules are tested

Status: accepted

## Context

The driven ports wrap external technology, a filesystem and a YAML parser,
each with its own error types. Left unchecked, either library's error type
could leak into the public API, or the crate's dependency direction could
drift as it grows.

## Decision

Error types on the driven ports are declared as domain types, so the YAML
library's and the filesystem's internal error types never leave their
adapters. `RunError::Source(SourceError)` and `RunError::Store(StoreError)`
hold these typed port errors, so the use case's caller knows what went
wrong without losing information to a string. The dependency rules, domain
imports no adapter or vendor crate, adapters import no use case, only
`main.rs` constructs concrete adapters, are enforced by a test
(`tests/architecture.rs`) rather than by convention alone.

## Consequences

No vendor error type or library-specific detail is part of the crate's
public API. A dependency-rule violation is caught immediately by the test
suite rather than surviving as an unnoticed convention break.
