# Partial coverage between documents, code and rules is not an error

Status: accepted

## Context

A mandate rarely links every document to code, or every rule to a
document, especially while it is being written. The validator needs to
decide which gaps are a real problem and which are the normal, expected
state of a mandate in progress.

## Decision

A governed document that no `code` entry links to is not an error; partial
coverage between documents and code is expected. Empty `governs` and empty
`code` lists are also not errors. Only an empty `rules` list is an error
(`ValidationError::NoRules`), since a mandate with no rules at all has
nothing to validate against.

## Consequences

`validate` reports fewer false positives while a mandate is incomplete, at
the cost of not being able to detect an accidentally empty `governs` or
`code` list as a mistake. The single required condition, at least one rule,
keeps the rest of the format permissive by default.
