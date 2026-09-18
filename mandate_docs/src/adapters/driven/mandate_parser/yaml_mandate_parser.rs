//! Parses mandate YAML into domain types. The serde structs live here and
//! never escape this module; only [`Mandate`] crosses into the domain.

use serde::Deserialize;

use crate::domain::model::mandate::{CodeLink, GovernedDoc, Mandate, Rule, RuleKind};
use crate::domain::ports::driven::mandate_parser::MandateParseError;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MandateDoc {
    name: String,
    #[serde(default)]
    description: Option<String>,
    rules: Vec<RuleDoc>,
    governs: Vec<GovernedDocDoc>,
    code: Vec<CodeLinkDoc>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleDoc {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    run: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GovernedDocDoc {
    doc: String,
    rules: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeLinkDoc {
    path: String,
    docs: Vec<String>,
}

/// Parses the text of a mandate YAML file into a [`Mandate`].
///
/// Does not validate cross-references (unknown rule ids, ungoverned docs,
/// missing files, ...); that is [`crate::domain::model::validation::validate`]'s
/// job. This only checks that the YAML has the mandate shape: known fields,
/// and each rule matching its declared `type`.
pub fn parse_mandate(text: &str) -> Result<Mandate, MandateParseError> {
    let doc: MandateDoc =
        yaml_serde::from_str(text).map_err(|err| MandateParseError::Malformed(err.to_string()))?;

    let rules = doc
        .rules
        .into_iter()
        .map(rule_from_doc)
        .collect::<Result<Vec<_>, _>>()?;

    let governs = doc
        .governs
        .into_iter()
        .map(|g| GovernedDoc {
            doc: g.doc,
            rules: g.rules,
        })
        .collect();

    let code = doc
        .code
        .into_iter()
        .map(|c| CodeLink {
            path: c.path,
            docs: c.docs,
        })
        .collect();

    Ok(Mandate {
        name: doc.name,
        description: doc.description,
        rules,
        governs,
        code,
    })
}

fn rule_from_doc(rule: RuleDoc) -> Result<Rule, MandateParseError> {
    let kind = match rule.kind.as_str() {
        "script" => match (rule.run, rule.prompt) {
            (Some(run), None) => RuleKind::Script { run },
            (None, _) => {
                return Err(MandateParseError::InvalidRule {
                    id: rule.id,
                    reason: "type: script requires 'run'".to_string(),
                })
            }
            (Some(_), Some(_)) => {
                return Err(MandateParseError::InvalidRule {
                    id: rule.id,
                    reason: "type: script must not declare 'prompt'".to_string(),
                })
            }
        },
        "agent" => match (rule.prompt, rule.run) {
            (Some(prompt), None) => RuleKind::Agent { prompt },
            (None, _) => {
                return Err(MandateParseError::InvalidRule {
                    id: rule.id,
                    reason: "type: agent requires 'prompt'".to_string(),
                })
            }
            (Some(_), Some(_)) => {
                return Err(MandateParseError::InvalidRule {
                    id: rule.id,
                    reason: "type: agent must not declare 'run'".to_string(),
                })
            }
        },
        other => {
            return Err(MandateParseError::InvalidRule {
                id: rule.id,
                reason: format!("unknown rule type '{other}'"),
            })
        }
    };

    Ok(Rule {
        id: rule.id,
        description: rule.description,
        kind,
    })
}

/// Adapts [`parse_mandate`] to the [`crate::domain::ports::driven::mandate_parser::MandateParser`]
/// port, so a use case can parse mandate text without depending on this
/// module's serde types directly.
pub struct YamlMandateParser;

impl crate::domain::ports::driven::mandate_parser::MandateParser for YamlMandateParser {
    fn parse(&self, text: &str) -> Result<Mandate, MandateParseError> {
        parse_mandate(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_full_valid_mandate_into_every_field() {
        let yaml = r#"
name: Todo App Mandate
description: Keeps the todo app's docs honest.
rules:
  - id: has-owner
    type: script
    description: Checks the doc names an owner.
    run: ./scripts/check-owner.sh
  - id: claims-match-code
    type: agent
    prompt: Confirm the doc's claims match the linked code.
governs:
  - doc: docs/sop/todo-api.md
    rules:
      - has-owner
      - claims-match-code
  - doc: docs/sop/todo-ui.md
    rules:
      - has-owner
code:
  - path: src/api/todo.rs
    docs:
      - docs/sop/todo-api.md
  - path: src/ui/todo_list.rs
    docs:
      - docs/sop/todo-ui.md
"#;

        let result = parse_mandate(yaml);

        let expected = Mandate {
            name: "Todo App Mandate".to_string(),
            description: Some("Keeps the todo app's docs honest.".to_string()),
            rules: vec![
                Rule {
                    id: "has-owner".to_string(),
                    description: Some("Checks the doc names an owner.".to_string()),
                    kind: RuleKind::Script {
                        run: "./scripts/check-owner.sh".to_string(),
                    },
                },
                Rule {
                    id: "claims-match-code".to_string(),
                    description: None,
                    kind: RuleKind::Agent {
                        prompt: "Confirm the doc's claims match the linked code.".to_string(),
                    },
                },
            ],
            governs: vec![
                GovernedDoc {
                    doc: "docs/sop/todo-api.md".to_string(),
                    rules: vec!["has-owner".to_string(), "claims-match-code".to_string()],
                },
                GovernedDoc {
                    doc: "docs/sop/todo-ui.md".to_string(),
                    rules: vec!["has-owner".to_string()],
                },
            ],
            code: vec![
                CodeLink {
                    path: "src/api/todo.rs".to_string(),
                    docs: vec!["docs/sop/todo-api.md".to_string()],
                },
                CodeLink {
                    path: "src/ui/todo_list.rs".to_string(),
                    docs: vec!["docs/sop/todo-ui.md".to_string()],
                },
            ],
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn parse_missing_name_is_error() {
        let yaml = r#"
rules:
  - id: has-owner
    type: script
    run: ./check.sh
governs: []
code: []
"#;

        let result = parse_mandate(yaml);

        assert!(
            matches!(result, Err(MandateParseError::Malformed(_))),
            "expected MandateParseError::Malformed, got {result:?}"
        );
    }

    #[test]
    fn parse_unknown_top_level_field_is_error() {
        let yaml = r#"
name: X
rules:
  - id: has-owner
    type: script
    run: ./check.sh
governes: []
code: []
"#;

        let result = parse_mandate(yaml);

        assert!(
            matches!(result, Err(MandateParseError::Malformed(_))),
            "expected MandateParseError::Malformed, got {result:?}"
        );
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("governes"),
            "expected the error to name the offending field 'governes', got: {message}"
        );
    }

    #[test]
    fn parse_unknown_rule_field_is_error() {
        let yaml = r#"
name: X
rules:
  - id: has-owner
    type: script
    run: ./check.sh
    bogus: true
governs: []
code: []
"#;

        let result = parse_mandate(yaml);

        assert!(
            matches!(result, Err(MandateParseError::Malformed(_))),
            "expected MandateParseError::Malformed, got {result:?}"
        );
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("bogus"),
            "expected the error to name the offending field 'bogus', got: {message}"
        );
    }

    #[test]
    fn parse_script_rule_without_run_is_error() {
        let yaml = r#"
name: X
rules:
  - id: has-owner
    type: script
governs: []
code: []
"#;

        let result = parse_mandate(yaml);

        assert!(
            matches!(result, Err(MandateParseError::InvalidRule { ref id, .. }) if id == "has-owner"),
            "expected MandateParseError::InvalidRule for 'has-owner', got {result:?}"
        );
    }

    #[test]
    fn parse_agent_rule_without_prompt_is_error() {
        let yaml = r#"
name: X
rules:
  - id: claims-match-code
    type: agent
governs: []
code: []
"#;

        let result = parse_mandate(yaml);

        assert!(
            matches!(result, Err(MandateParseError::InvalidRule { ref id, .. }) if id == "claims-match-code"),
            "expected MandateParseError::InvalidRule for 'claims-match-code', got {result:?}"
        );
    }
}
