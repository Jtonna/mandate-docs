# Mandate format checks

Lookup reference for what each mandate parse or validation error means. See
[`../architecture/mandate-parser.md`](../architecture/mandate-parser.md)
for how parsing and validation fit into the crate.

## What parsing rejects

`parse_mandate` in
`src/adapters/driven/mandate_parser/yaml_mandate_parser.rs` turns mandate
YAML text into a `Mandate`, or fails with one of two errors:

- `MandateParseError::Malformed(message)`, carrying the underlying
  `yaml_serde::Error`'s message as a `String`. This covers YAML that does not
  parse at all, a missing required field, and an unknown field at any level.
  Every serde struct (`MandateDoc`, `RuleDoc`, `GovernedDocDoc`,
  `CodeLinkDoc`) carries `#[serde(deny_unknown_fields)]`, so a typo such as
  `governes` for `governs` is an error, not silently dropped.
- `MandateParseError::InvalidRule { id, reason }`, produced after the YAML
  shape has parsed, when a rule's `type` and its fields disagree:
  `type: script` without `run`, `type: script` with a `prompt` present,
  `type: agent` without `prompt`, `type: agent` with a `run` present, or a
  `type` that is neither `script` nor `agent`.

Printed forms, as the CLI shows them under the mandate's file name, where
`<id>` is the rule's `id`, `<yaml error text>` is the YAML library's own
text, and `<type>` is the `type` value written in the file:

- `failed to parse: malformed mandate: <yaml error text>`
- `failed to parse: rule '<id>': type: script requires 'run'`
- `failed to parse: rule '<id>': type: script must not declare 'prompt'`
- `failed to parse: rule '<id>': type: agent requires 'prompt'`
- `failed to parse: rule '<id>': type: agent must not declare 'run'`
- `failed to parse: rule '<id>': unknown rule type '<type>'`

Parsing stops at the first shape problem, since serde matches one document
to one shape. Validation collects every problem instead.

## What validation checks

`validate` in `src/domain/model/validation.rs` takes a `Mandate` and a
`&FileTreeSnapshot` and returns a `ValidationReport { errors, warnings, ... }`.
Every check runs regardless of what earlier checks found, so an author sees
every problem at once. `report.is_valid()` is `true` exactly when `errors` is
empty; `warnings` never affects it.

Partial coverage is valid: a governed document that no `code` entry links to,
an empty `governs` list, and an empty `code` list are not errors. An empty
`rules` list is an error, since a mandate with no rules has nothing to check.

The CLI adapter's `validation_lines` method renders the report as one string
per finding, errors first in validator order, then warnings, each prefixed
`error: ` or `warning: ` as shown in the table below. The
`report_lines` method wraps these under each mandate's file name, and the
fixture helper's `check` returns them unchanged, so fixture cases assert
what a user would see.

| Variant | Message printed | Meaning |
|---|---|---|
| `ValidationError::NoRules` | `error: no rules defined; a mandate needs at least one` | `rules` is empty. |
| `ValidationError::DuplicateRuleId { id }` | `error: duplicate rule id '<id>'` | Two rules share an `id`. |
| `ValidationError::UnknownRule { doc, rule }` | `error: rule '<rule>' is referenced by <doc> but not defined` | A `governs` entry lists a rule id that no `rules` entry defines. |
| `ValidationError::CodeLinksUngovernedDoc { path, doc }` | `error: code <path> links <doc> which this mandate does not govern` | A `code` entry names a document not present in `governs`. |
| `ValidationError::DocMissing { doc }` | `error: governed document not found: <doc>` | A `governs` path is not a file under the root (`--root`, or the ancestor holding `.mandate`); a directory at that path also triggers it. |
| `ValidationError::CodeMissing { path }` | `error: missing source file: <path>` | A `code` path is not a file under the root (`--root`, or the ancestor holding `.mandate`); a directory at that path also triggers it. |
| `ValidationWarning::UnreferencedRule { id }` | `warning: rule '<id>' is defined but no document references it` | A rule in `rules` is referenced by no `governs` entry. |

`CodeLinksUngovernedDoc` is the check for the format rule that a mandate may
not link code to a document it does not govern. `UnreferencedRule` is the
check for the format rule that a rule referenced by no document is a warning,
not an error.
