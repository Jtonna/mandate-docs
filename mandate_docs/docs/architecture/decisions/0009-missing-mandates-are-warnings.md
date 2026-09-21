# A missing `.mandate` folder or empty mandates folder is a warning, not a failure

Status: accepted

## Context

A run can find no `.mandate` folder anywhere from the starting directory up
to the filesystem root, or it can find a `.mandate` folder whose mandates
directory is empty. Neither case means anything went wrong; there is simply
nothing to check yet.

## Decision

Both cases are warnings (`RunWarning::NoMandatesFound` for an empty
mandates folder; a printed warning naming the search path when no
`.mandate` folder is found at all), and both result in exit code `0`.

## Consequences

Running the command in a project that has not adopted mandate yet, or that
has adopted it but written no mandates, succeeds with a warning rather than
failing. A caller scripting around the exit code only sees failure when a
mandate that does exist actually has a problem.
