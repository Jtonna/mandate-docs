# Use yaml_serde, not serde_yaml

Status: accepted

## Context

The mandate parser needs a YAML deserialization library to turn mandate
text into the `MandateDoc` family of serde structs. `serde_yaml`, the
long-standing choice for this, is archived and no longer maintained.

## Decision

Use `yaml_serde`, the YAML organisation's maintained fork, instead of
`serde_yaml`.

## Consequences

The parser depends on an actively maintained crate rather than an archived
one. The dependency lives only in
`src/adapters/driven/mandate_parser/yaml_mandate_parser.rs`, so a future
switch to another YAML library stays isolated to that adapter.
