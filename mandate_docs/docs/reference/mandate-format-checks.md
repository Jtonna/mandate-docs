# Mandate format checks

This is a reference for the mandate parser and validator, for a reader who
needs to look up what a specific parse or validation error means. See
[`../architecture/mandate-parser.md`](../architecture/mandate-parser.md)
for how parsing and validation fit into the crate as a whole.

## What parsing rejects

`parse_mandate` in `src/adapters/driven/mandate_parser/yaml_mandate_parser.rs` turns mandate YAML text into a
`Mandate`, or fails with one of two errors:

- `MandateParseError::Malformed(message)`, carrying the underlying
  `yaml_serde::Error`'s message as a `String`. `yaml_serde` is used in
  place of `serde_yaml`, which is archived and no longer maintained. This
  covers YAML that does not parse at all, a missing required field, and an
  unknown field at any level, since every serde struct
  (`MandateDoc`, `RuleDoc`, `GovernedDocDoc`, `CodeLinkDoc`) carries
  `#[serde(deny_unknown_fields)]`, so a typo'd field name such as
  `governes` for `governs` fails parsing loudly instead of being dropped
  silently.
- `MandateParseError::InvalidRule { id, reason }`, produced after the YAML
  shape has already parsed, when a rule's `type` and its fields disagree:
  `type: script` without `run`, `type: script` with a `prompt` present,
  `type: agent` without `prompt`, `type: agent` with a `run` present, or a
  `type` that is neither `script` nor `agent`.

Parsing stops at the first shape problem, because that is how serde
deserialization works: one YAML document either matches the target shape or
it does not. Collecting every problem in one pass is what validation does
instead, once a `Mandate` value exists to check.

## What validation checks

`validate` in `src/domain/model/validation.rs` takes a `Mandate` and a
`&FileTreeSnapshot` and returns a `ValidationReport { errors, warnings, ... }`
built in one pass: every check in the function runs regardless of what
earlier checks found, so an author fixing a mandate sees every problem at
once rather than discovering errors one run at a time. `report.is_valid()`
is `true` exactly when `errors` is empty; `warnings` never affects it.

A mandate rarely links every document to code, or every rule to a document,
especially while it is being written, so partial coverage is expected and
not an error: a governed document that no `code` entry links to, an empty
`governs` list, and an empty `code` list are all valid. Only an empty
`rules` list is an error, since a mandate with no rules at all has nothing
to validate against.

The CLI adapter's `validation_lines` method renders the report as one
string per finding, errors first in validator order, then warnings, each
prefixed `error: ` or `warning: ` and using the `Display` text in the
table below. The `report_lines` method wraps these under each mandate's
file name, and the fixture helper's `check` returns them unchanged, so the
printed format and its order are defined in one place and every fixture
case asserts exactly what a user would see.

| Variant | Message printed | Meaning |
|---|---|---|
| `ValidationError::NoRules` | `no rules defined; a mandate needs at least one` | `rules` is empty. |
| `ValidationError::DuplicateRuleId { id }` | `duplicate rule id '<id>'` | Two rules share an `id`. |
| `ValidationError::UnknownRule { doc, rule }` | `rule '<rule>' is referenced by <doc> but not defined` | A `governs` entry lists a rule id that no `rules` entry defines. |
| `ValidationError::CodeLinksUngovernedDoc { path, doc }` | `code <path> links <doc> which this mandate does not govern` | A `code` entry names a document not present in `governs`. |
| `ValidationError::DocMissing { doc }` | `governed document not found: <doc>` | A `governs` path does not exist under the given `--root`. |
| `ValidationError::CodeMissing { path }` | `missing source file: <path>` | A `code` path does not exist under the given `--root`. |
| `ValidationWarning::UnreferencedRule { id }` | `rule '<id>' is defined but no document references it` | A rule in `rules` is referenced by no `governs` entry. |

`CodeLinksUngovernedDoc` is the validator's check for the mandate format
rule that a mandate may not link code to a document it does not govern.
`UnreferencedRule` is the check for the format rule that a rule defined
but referenced by no document is valid and worth a warning, not an error:
the format treats it as expected, so `validate` reports it as a warning
rather than an error.
