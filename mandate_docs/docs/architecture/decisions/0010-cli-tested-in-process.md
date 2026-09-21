# Test the CLI adapter in-process, not through the built binary

Status: accepted

## Context

The CLI adapter's argument parsing and rendering could be tested either by
launching the compiled binary as a subprocess and inspecting its output, or
by calling its functions directly in-process. The binary is expected to
gain startup side effects of its own over time.

## Decision

The CLI is a driving adapter tested in-process, in
`src/adapters/driving/cli/mod.rs` itself: `parse_args` and `render` are
exercised directly, with in-memory writers, rather than through the built
binary. No test launches the binary.

## Consequences

Tests stay fast and avoid coupling to process startup behaviour that will
grow over time. The exit code mapping itself, one `if` on `is_valid()` in
`main.rs`, is not unit tested, since it lives in the composition root
rather than in the adapter.
