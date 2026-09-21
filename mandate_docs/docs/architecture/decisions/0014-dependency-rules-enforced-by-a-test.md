# Dependency rules are enforced by a test, not by convention

Status: accepted

## Context

The crate's dependency direction, domain, ports, adapters, use cases, could
drift as it grows, with nothing catching a violation until it is noticed by
a reviewer.

## Decision

Three dependency rules are enforced by `tests/architecture.rs`: the domain
imports no adapter or vendor crate, driven adapters import no use case, and
only `main.rs` constructs concrete adapters.

## Consequences

A dependency-rule violation is caught immediately by the test suite rather
than surviving as an unnoticed convention break. The rules are checked on
every test run, not only when a reviewer happens to look for them.
