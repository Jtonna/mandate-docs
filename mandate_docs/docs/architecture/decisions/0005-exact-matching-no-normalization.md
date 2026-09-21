# Match paths and mandate names exactly, never normalised

Status: accepted

## Context

Paths inside a mandate and mandate file names both need a matching rule
against the file tree and the command line. Case folding or path
normalisation are common conveniences, but behave differently across
operating systems: Linux allows two file names differing only by case in
one directory, while Windows and macOS refuse it by default.

## Decision

Paths are kept exactly as written in the mandate; nothing normalises them.
Mandate selection on the command line is by full file name, case-sensitive,
never by the descriptive `name:` field inside the file, so two mandates
cannot collide on a name a user did not choose as a file name.

## Consequences

Exact matching is the one behaviour that is the same on all three major
operating systems. A path or file name that differs only by case is treated
as a different, missing entity rather than silently matched.
