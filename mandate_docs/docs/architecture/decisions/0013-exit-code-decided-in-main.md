# The exit code is decided in main.rs, not in cli::run

Status: accepted

## Context

The finished run needs to be turned into a process exit code. This could
happen inside `cli::run`, right after rendering, or stay in `main.rs`.

## Decision

`cli::run` returns the report or `None` and never calls
`std::process::exit`. `main.rs` alone picks the exit code, by calling
`report.is_valid()` on what `cli::run` hands back.

## Consequences

Keeping the exit code in `main.rs`, separate from `cli::run`, means a
planned long-running mode can keep running after an invalid report instead
of exiting, since nothing between the use case and `cli::run` calls
`std::process::exit`.
