//! Parses mandate YAML into domain types. The serde structs live here and
//! never escape this module; only [`Mandate`] crosses into the domain.

use serde::Deserialize;
use std::fmt;

use crate::domain::mandate::{CodeLink, GovernedDoc, Mandate, Rule, RuleKind};

/// Everything that can go wrong turning mandate YAML text into a [`Mandate`].
#[derive(Debug)]
pub enum ParseError {
    /// The YAML did not match the mandate shape at all (missing field,
    /// unknown field, wrong type, malformed YAML, ...).
    Malformed(yaml_serde::Error),
    /// A rule declared `type: script` without a `run`, or `type: agent`
    /// without a `prompt` (or declared both, or neither).
    InvalidRule { id: String, reason: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Malformed(err) => write!(f, "malformed mandate: {err}"),
            ParseError::InvalidRule { id, reason } => {
                write!(f, "rule '{id}': {reason}")
            }
        }
    }
}

impl std::error::Error for ParseError {}

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
/// missing files, ...); that is [`crate::domain::validation::validate`]'s
/// job. This only checks that the YAML has the mandate shape: known fields,
/// and each rule matching its declared `type`.
pub fn parse_mandate(text: &str) -> Result<Mandate, ParseError> {
    let doc: MandateDoc = yaml_serde::from_str(text).map_err(ParseError::Malformed)?;

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

fn rule_from_doc(rule: RuleDoc) -> Result<Rule, ParseError> {
    let kind = match rule.kind.as_str() {
        "script" => match (rule.run, rule.prompt) {
            (Some(run), None) => RuleKind::Script { run },
            (None, _) => {
                return Err(ParseError::InvalidRule {
                    id: rule.id,
                    reason: "type: script requires 'run'".to_string(),
                })
            }
            (Some(_), Some(_)) => {
                return Err(ParseError::InvalidRule {
                    id: rule.id,
                    reason: "type: script must not declare 'prompt'".to_string(),
                })
            }
        },
        "agent" => match (rule.prompt, rule.run) {
            (Some(prompt), None) => RuleKind::Agent { prompt },
            (None, _) => {
                return Err(ParseError::InvalidRule {
                    id: rule.id,
                    reason: "type: agent requires 'prompt'".to_string(),
                })
            }
            (Some(_), Some(_)) => {
                return Err(ParseError::InvalidRule {
                    id: rule.id,
                    reason: "type: agent must not declare 'run'".to_string(),
                })
            }
        },
        other => {
            return Err(ParseError::InvalidRule {
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
/// port, so the application layer can parse mandate text without depending
/// on this module's serde types directly.
pub struct YamlMandateParser;

impl crate::domain::ports::driven::mandate_parser::MandateParser for YamlMandateParser {
    fn parse(&self, text: &str) -> Result<Mandate, String> {
        parse_mandate(text).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            matches!(result, Err(ParseError::Malformed(_))),
            "expected ParseError::Malformed, got {result:?}"
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
            matches!(result, Err(ParseError::Malformed(_))),
            "expected ParseError::Malformed, got {result:?}"
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
            matches!(result, Err(ParseError::Malformed(_))),
            "expected ParseError::Malformed, got {result:?}"
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
            matches!(result, Err(ParseError::InvalidRule { ref id, .. }) if id == "has-owner"),
            "expected ParseError::InvalidRule for 'has-owner', got {result:?}"
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
            matches!(result, Err(ParseError::InvalidRule { ref id, .. }) if id == "claims-match-code"),
            "expected ParseError::InvalidRule for 'claims-match-code', got {result:?}"
        );
    }
}
