# Run report types are named RunReport<Phase>

Status: accepted

## Context

The run command currently has one phase, mandate validation. A later phase,
running a mandate's rules against the codebase, is planned. The owner set
this naming convention when the run command was designed; the only
evidence for it in the code today is the current type's name.

## Decision

Each phase of the run command gets its own report type, named
`RunReport<Phase>`. The current phase's type is
`RunReportMandatesValidation`. A later rule-execution phase gets its own
`RunReport` type rather than growing this one.

## Consequences

Adding a phase means adding a new, separately named report type, not
widening an existing one with fields that only apply to some callers. A
reader can tell which phase a report describes from its type name alone.
