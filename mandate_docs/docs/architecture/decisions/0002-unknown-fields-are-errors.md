# Reject unknown fields as a parse error

Status: accepted

## Context

A mandate YAML file is hand-written, so a typo in a field name, such as
`governes` instead of `governs`, is a real risk. Silently ignoring a field
serde does not recognise would let such a typo pass parsing with the typo'd
field simply dropped.

## Decision

Every serde struct in the parser (`MandateDoc`, `RuleDoc`, `GovernedDocDoc`,
`CodeLinkDoc`) carries `#[serde(deny_unknown_fields)]`, so an unknown field
at any level is a `MandateParseError::Malformed` parse error.

## Consequences

A typo'd field name fails parsing loudly instead of being ignored. This is
bundled into the same `Malformed` error as YAML that does not parse at all
and a missing required field, since all three are shape problems serde
detects the same way.
