# Presentation lives in the driving adapter, not the domain

Status: accepted

## Context

The report needs to be turned into printed text. This could live in the
domain, in the CLI adapter, or in `main.rs`, and the domain should carry
only data.

## Decision

Report rendering (`validation_lines`, `report_lines`, `render`) belongs to
the CLI driving adapter, not the domain. The CLI adapter's `run` method
executes the use case and returns the report or `None`; it does not decide
the process exit code.

## Consequences

The domain carries only the data and nothing about how it is displayed.
Rendering logic can change, or grow another driving adapter with its own
rendering, without touching the domain or the use case.
