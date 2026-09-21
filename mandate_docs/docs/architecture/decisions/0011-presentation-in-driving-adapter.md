# Presentation and the exit-code split both live in the driving adapter and main.rs

Status: accepted

## Context

The report needs to be turned into printed text, and the finished run needs
to be turned into a process exit code. Both could live in the domain, in
the CLI adapter, or in `main.rs`, and the domain should carry only data.

## Decision

Report rendering (`validation_lines`, `report_lines`, `render`) belongs to
the CLI driving adapter, not the domain. The CLI adapter's `run` method
executes the use case and returns the report or `None`; `main.rs` only
wires the adapters and picks the exit code by calling `report.is_valid()`.

## Consequences

The domain carries only the data and nothing about how it is displayed or
exited. Keeping the exit code in `main.rs`, separate from `cli::run`, means
a planned long-running mode can keep running after an invalid report
instead of exiting, since nothing between the use case and `cli::run` calls
`std::process::exit`.
