# Discovery checks ancestors cheaply and snapshots only the found root

Status: accepted

## Context

Finding the project's `.mandate` folder means walking upward from the
starting directory through an unknown number of ancestors. A real run once
failed with "Access is denied" on an unrelated system temp folder while
snapshotting every ancestor tried, before it could even report that
`.mandate` was never found; snapshotting walks directories the current user
does not necessarily own or have permission to read.

## Decision

Discovery checks each ancestor with `FileTreeSource::has_directory`, a
cheap existence check that answers `false` rather than erroring when a
directory cannot be read, instead of snapshotting it. Only the ancestor
that turns out to hold `.mandate` is ever snapshotted; every ancestor tried
above it gets only the existence check.

## Consequences

An unreadable ancestor above the project root no longer aborts discovery
with a permission error; it is simply a reason that directory cannot be the
root, and discovery moves on to its parent. Only one full tree walk happens
per run, at the root that was actually found.
