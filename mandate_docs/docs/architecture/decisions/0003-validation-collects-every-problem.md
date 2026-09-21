# Collect every problem in one pass, per mandate and per run

Status: accepted

## Context

An author fixing a mandate wants to see every problem at once rather than
discovering errors one `cargo run` at a time. The same question applies one
level up: should a run stop at the first invalid mandate, or check every
selected mandate regardless?

## Decision

`validate` in `src/domain/model/validation.rs` runs every check
unconditionally and returns a `ValidationReport` holding every error and
warning found, rather than stopping at the first. `RunMandates` applies the
same principle at the run level: it validates and reports every selected
mandate rather than stopping at the first invalid one.

## Consequences

An author sees the full list of problems in one pass, both within a single
mandate and across a run of several. A parse failure in one mandate does
not prevent the rest of the run from being reported; it is recorded in that
mandate's slot and the run continues.
